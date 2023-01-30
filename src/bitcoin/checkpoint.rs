use super::{
    adapter::Adapter,
    signatory::SignatorySet,
    threshold_sig::{Signature, ThresholdSig},
    ConsensusKey, Xpub,
};
use crate::bitcoin::Nbtc;
use crate::error::{Error, Result};
use bitcoin::hashes::Hash;
use bitcoin::{blockdata::transaction::EcdsaSighashType, hashes::hex::ToHex};
// use bitcoin_hashes::Hash;
use derive_more::{Deref, DerefMut};
use orga::{
    call::Call,
    client::Client,
    collections::{map::ReadOnly, ChildMut, Deque, Map, Ref},
    context::GetContext,
    encoding::{Decode, Encode, LengthVec},
    plugins::Time,
    prelude::Context,
    query::Query,
    state::State,
    Error as OrgaError, Result as OrgaResult,
};
use orga::{describe::Describe, prelude::Accounts};
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, str::FromStr};

pub const MIN_CHECKPOINT_INTERVAL: u64 = 60 * 5;
pub const MAX_CHECKPOINT_INTERVAL: u64 = 60 * 60 * 8;
pub const MAX_INPUTS: u64 = 40;
pub const MAX_OUTPUTS: u64 = 200;
pub const FEE_RATE: u64 = 1;
pub const MAX_AGE: u64 = 60 * 60 * 24 * 7 * 3;
//TODO: Find actual amount for this
pub const EMERGENCY_DISBURSAL_MIN_TX_AMT: u64 = 0;
pub const EMERGENCY_DISBURSAL_LOCK_TIME_INTERVAL: u32 = 60 * 24 * 7; //one week
pub const EMERGENCY_DISBURSAL_MAX_TX_SIZE: u64 = 50_000; //50kB

#[derive(Debug, Encode, Decode, Default, Serialize, Deserialize)]
pub enum CheckpointStatus {
    #[default]
    Building,
    Signing,
    Complete,
}

// TODO: make it easy to derive State for simple types like this
impl State for CheckpointStatus {
    fn attach(&mut self, _: orga::store::Store) -> OrgaResult<()> {
        Ok(())
    }

    fn flush(&mut self) -> OrgaResult<()> {
        Ok(())
    }
}

impl Query for CheckpointStatus {
    type Query = ();

    fn query(&self, _: ()) -> OrgaResult<()> {
        Ok(())
    }
}

impl Call for CheckpointStatus {
    type Call = ();

    fn call(&mut self, _: ()) -> OrgaResult<()> {
        Ok(())
    }
}

impl<U: Send + Clone> Client<U> for CheckpointStatus {
    type Client = orga::client::PrimitiveClient<Self, U>;

    fn create_client(parent: U) -> Self::Client {
        orga::client::PrimitiveClient::new(parent)
    }
}

impl Describe for CheckpointStatus {
    fn describe() -> orga::describe::Descriptor {
        orga::describe::Builder::new::<Self>().build()
    }
}

#[derive(
    State, Call, Query, Client, Debug, Encode, Decode, Serialize, Deserialize, Default, Describe,
)]
pub struct Input {
    pub prevout: Adapter<bitcoin::OutPoint>,
    pub script_pubkey: Adapter<bitcoin::Script>,
    pub redeem_script: Adapter<bitcoin::Script>,
    pub sigset_index: u32,
    pub dest: LengthVec<u16, u8>,
    pub amount: u64,
    pub est_witness_vsize: u64,
}

impl Input {
    pub fn to_txin(&self, sigs: &ThresholdSig) -> Result<bitcoin::TxIn> {
        let mut witness = sigs.to_witness()?;
        if sigs.done() {
            witness.push(self.redeem_script.to_bytes());
        }

        Ok(bitcoin::TxIn {
            previous_output: *self.prevout,
            script_sig: bitcoin::Script::new(),
            sequence: u32::MAX,
            witness: bitcoin::Witness::from_vec(witness),
        })
    }

    pub fn est_vsize(&self) -> u64 {
        self.est_witness_vsize + 40
    }
}

pub type Output = Adapter<bitcoin::TxOut>;

#[derive(
    State, Call, Query, Client, Debug, Encode, Decode, Default, Serialize, Deserialize, Describe,
)]
pub struct Checkpoint {
    pub status: CheckpointStatus,
    pub inputs: Deque<Input>,
    pub emergency_disbursal_txs: Deque<Adapter<bitcoin::Transaction>>,
    signed_inputs: u16,
    pub outputs: Deque<Output>,
    pub sig_queue: SignatureQueue,
    pub sigset: SignatorySet,
}

impl Checkpoint {
    pub fn get_to_sign_msgs(
        &self,
        sigs: &Deque<ThresholdSig>,
        xpub: &Xpub,
        msgs: &mut Vec<([u8; 32], u32)>,
    ) -> Result<()> {
        let secp = bitcoin::secp256k1::Secp256k1::verification_only();

        for i in 0..self.inputs.len() {
            let input = self.inputs.get(i)?.unwrap();
            let pubkey = xpub
                .derive_pub(
                    &secp,
                    &[bitcoin::util::bip32::ChildNumber::from_normal_idx(
                        input.sigset_index,
                    )?],
                )?
                .public_key;
            let sigs = sigs.get(i)?.unwrap();
            if sigs.needs_sig(pubkey.into())? {
                msgs.push((sigs.message(), input.sigset_index));
            }
        }

        Ok(())
    }

    #[query]
    pub fn to_sign(&self, xpub: Xpub) -> Result<Vec<([u8; 32], u32)>> {
        let mut msgs = vec![];

        // TODO: get signatures for active group in signature queue
        self.get_to_sign_msgs(&self.sig_queue.emergency_disbursal, &xpub, &mut msgs)?;
        self.get_to_sign_msgs(&self.sig_queue.inputs, &xpub, &mut msgs)?;

        Ok(msgs)
    }

    pub fn create_time(&self) -> u64 {
        self.sigset.create_time()
    }

    pub fn tx(&self) -> Result<(bitcoin::Transaction, u64)> {
        let mut tx = bitcoin::Transaction {
            version: 1,
            lock_time: 0,
            input: vec![],
            output: vec![],
        };

        let mut est_vsize = 0;

        // TODO: use deque iterator
        for i in 0..self.inputs.len() {
            let input = self.inputs.get(i)?.unwrap();
            let sigs = &*self.sig_queue.inputs.get(i)?.unwrap();
            tx.input.push(input.to_txin(sigs)?);
            est_vsize += input.est_witness_vsize;
        }

        // TODO: use deque iterator
        for i in 0..self.outputs.len() {
            let output = self.outputs.get(i)?.unwrap();
            tx.output.push((**output).clone());
        }

        est_vsize += tx.size() as u64;

        Ok((tx, est_vsize))
    }

    #[query]
    pub fn get_tvl(&self) -> Result<u64> {
        let mut tvl = 0;
        for i in 0..self.inputs.len() {
            if let Some(input) = self.inputs.get(i)? {
                tvl += input.amount;
            }
        }

        Ok(tvl)
    }
}

#[derive(
    State, Call, Query, Client, Encode, Decode, Default, Serialize, Deserialize, Describe, Debug,
)]
pub struct CheckpointQueue {
    pub(super) queue: Deque<Checkpoint>,
    pub(super) index: u32,
}

#[derive(
    State, Call, Query, Client, Encode, Decode, Default, Serialize, Deserialize, Describe, Debug,
)]
pub struct SignatureQueue {
    pub(super) inputs: Deque<ThresholdSig>,
    pub(super) emergency_disbursal: Deque<ThresholdSig>,
}

#[derive(Deref)]
pub struct CompletedCheckpoint<'a>(Ref<'a, Checkpoint>);

#[derive(Deref, Debug)]
pub struct SigningCheckpoint<'a>(Ref<'a, Checkpoint>);

impl<'a, U: Clone> Client<U> for SigningCheckpoint<'a> {
    type Client = ();

    fn create_client(_: U) {}
}

impl<'a> Query for SigningCheckpoint<'a> {
    type Query = ();

    fn query(&self, _: ()) -> OrgaResult<()> {
        Ok(())
    }
}

#[derive(Deref, DerefMut)]
pub struct SigningCheckpointMut<'a>(ChildMut<'a, u64, Checkpoint>);

impl<'a> SigningCheckpointMut<'a> {
    pub fn sign(&mut self, xpub: Xpub, sigs: LengthVec<u16, Signature>) -> Result<()> {
        let secp = bitcoin::secp256k1::Secp256k1::verification_only();

        let mut sig_index = 0;
        for i in 0..self.inputs.len() {
            let mut input = self.inputs.get_mut(i)?.unwrap();

            let pubkey = xpub
                .derive_pub(
                    &secp,
                    &[bitcoin::util::bip32::ChildNumber::from_normal_idx(
                        input.sigset_index,
                    )?],
                )?
                .public_key
                .into();

            let mut input_sigs = self.sig_queue.inputs.get_mut(i)?.unwrap();
            if !input_sigs.contains_key(pubkey)? {
                continue;
            }

            if input_sigs.done() {
                sig_index += 1;
                continue;
            }

            if sig_index > sigs.len() {
                return Err(OrgaError::App("Not enough signatures supplied".to_string()).into());
            }

            let sig = sigs[sig_index];
            sig_index += 1;

            input_sigs.sign(pubkey, sig)?;

            if input_sigs.done() {
                self.signed_inputs += 1;
            }
        }

        if sig_index != sigs.len() {
            return Err(OrgaError::App("Excess signatures supplied".to_string()).into());
        }

        Ok(())
    }

    pub fn done(&self) -> bool {
        self.signed_inputs as u64 == self.inputs.len()
    }

    pub fn advance(self) -> Result<()> {
        let mut checkpoint = self.0;

        checkpoint.status = CheckpointStatus::Complete;

        Ok(())
    }
}

#[derive(Deref)]
pub struct BuildingCheckpoint<'a>(Ref<'a, Checkpoint>);

#[derive(Deref, DerefMut)]
pub struct BuildingCheckpointMut<'a>(ChildMut<'a, u64, Checkpoint>);

impl<'a> BuildingCheckpointMut<'a> {
    pub fn push_input(
        &mut self,
        prevout: bitcoin::OutPoint,
        sigset: &SignatorySet,
        dest: &[u8],
        amount: u64,
    ) -> Result<u64> {
        let script_pubkey = sigset.output_script(dest)?;
        let redeem_script = sigset.redeem_script(dest)?;

        let input = Input {
            prevout: Adapter::new(prevout),
            script_pubkey: Adapter::new(script_pubkey),
            redeem_script: Adapter::new(redeem_script),
            sigset_index: sigset.index(),
            dest: dest.encode()?.try_into()?,
            amount,
            est_witness_vsize: sigset.est_witness_vsize(),
        };
        let est_vsize = input.est_vsize();
        self.inputs.push_back(input)?;

        // TODO: make it possible to populate instance in memory then push,
        // rather than push then access and modify
        self.sig_queue.inputs.push_back(ThresholdSig::new())?;
        let inputs_len = self.inputs.len();
        let mut sigs = self.sig_queue.inputs.get_mut(inputs_len - 1)?.unwrap();
        sigs.from_sigset(sigset)?;

        Ok(est_vsize)
    }

    pub fn advance(
        self,
    ) -> Result<(
        bitcoin::OutPoint,
        u64,
        Vec<(ReadOnly<Input>, ReadOnly<ThresholdSig>)>,
        Vec<ReadOnly<Output>>,
    )> {
        let mut checkpoint = self.0;

        checkpoint.status = CheckpointStatus::Signing;

        let reserve_out = bitcoin::TxOut {
            value: 0, // will be updated after counting ins/outs and fees
            script_pubkey: checkpoint.sigset.output_script(&vec![0u8])?, // TODO: double-check safety
        };
        checkpoint.outputs.push_front(Adapter::new(reserve_out))?;

        let mut excess_inputs = vec![];
        while checkpoint.inputs.len() > MAX_INPUTS {
            let removed_input = checkpoint.inputs.pop_back()?.unwrap();
            let removed_sigs = checkpoint.sig_queue.inputs.pop_back()?.unwrap();
            excess_inputs.push((removed_input, removed_sigs));
        }

        let mut excess_outputs = vec![];
        while checkpoint.outputs.len() > MAX_OUTPUTS {
            let removed_output = checkpoint.outputs.pop_back()?.unwrap();
            excess_outputs.push(removed_output);
        }

        let mut in_amount = 0;
        for i in 0..checkpoint.inputs.len() {
            let input = checkpoint.inputs.get(i)?.unwrap();
            in_amount += input.amount;
        }

        let mut out_amount = 0;
        for i in 0..checkpoint.outputs.len() {
            let output = checkpoint.outputs.get(i)?.unwrap();
            out_amount += output.value;
        }

        let mut signing = SigningCheckpointMut(checkpoint);

        let (mut tx, est_vsize) = signing.tx()?;
        let fee = est_vsize * FEE_RATE;
        let reserve_value = in_amount - out_amount - fee;
        let mut reserve_out = signing.outputs.get_mut(0)?.unwrap();
        reserve_out.value = reserve_value;
        tx.output[0].value = reserve_value;

        use bitcoin::hashes::Hash;
        let mut sc = bitcoin::util::sighash::SighashCache::new(&tx);
        for i in 0..signing.inputs.len() {
            let mut input = signing.inputs.get_mut(i)?.unwrap();
            let sighash_type = EcdsaSighashType::All;
            let sighash = sc.segwit_signature_hash(
                i as usize,
                &input.redeem_script,
                input.amount,
                sighash_type,
            )?;
            let mut sigs = signing.sig_queue.inputs.get_mut(i)?.unwrap();
            sigs.set_message(sighash.into_inner());
        }

        let reserve_outpoint = bitcoin::OutPoint {
            txid: tx.txid(),
            vout: 0,
        };

        Ok((
            reserve_outpoint,
            reserve_value,
            excess_inputs,
            excess_outputs,
        ))
    }
}

impl CheckpointQueue {
    pub fn reset(&mut self) -> OrgaResult<()> {
        self.index = 0;
        super::clear_deque(&mut self.queue)?;

        Ok(())
    }

    #[query]
    pub fn get(&self, index: u32) -> Result<Ref<'_, Checkpoint>> {
        let index = self.get_deque_index(index)?;
        Ok(self.queue.get(index as u64)?.unwrap())
    }

    pub fn get_mut(&mut self, index: u32) -> Result<ChildMut<'_, u64, Checkpoint>> {
        let index = self.get_deque_index(index)?;
        Ok(self.queue.get_mut(index as u64)?.unwrap())
    }

    fn get_deque_index(&self, index: u32) -> Result<u32> {
        let start = self.index + 1 - (self.queue.len() as u32);
        if index > self.index || index < start {
            Err(OrgaError::App("Index out of bounds".to_string()).into())
        } else {
            Ok(index - start)
        }
    }

    pub fn len(&self) -> Result<u32> {
        Ok(u32::try_from(self.queue.len())?)
    }

    #[query]
    pub fn index(&self) -> u32 {
        self.index
    }

    #[query]
    pub fn all(&self) -> Result<Vec<(u32, Ref<'_, Checkpoint>)>> {
        // TODO: return iterator
        // TODO: use Deque iterator

        let mut out = Vec::with_capacity(self.queue.len() as usize);

        for i in 0..self.queue.len() {
            let index = self.index - (i as u32);
            let checkpoint = self.queue.get(index as u64)?.unwrap();
            out.push((index, checkpoint));
        }

        Ok(out)
    }

    #[query]
    pub fn completed(&self) -> Result<Vec<CompletedCheckpoint<'_>>> {
        // TODO: return iterator
        // TODO: use Deque iterator

        let mut out = vec![];

        for i in 0..self.queue.len() {
            let checkpoint = self.queue.get(i)?.unwrap();

            if !matches!(checkpoint.status, CheckpointStatus::Complete) {
                break;
            }

            out.push(CompletedCheckpoint(checkpoint));
        }

        Ok(out)
    }

    #[query]
    pub fn last_completed_tx(&self) -> Result<Adapter<bitcoin::Transaction>> {
        let index = if self.signing()?.is_some() {
            self.index.checked_sub(2)
        } else {
            self.index.checked_sub(1)
        }
        .ok_or_else(|| Error::Orga(OrgaError::App("No completed checkpoints yet".to_string())))?;

        Ok(Adapter::new(self.get(index)?.tx()?.0))
    }

    #[query]
    pub fn completed_txs(&self) -> Result<Vec<Adapter<bitcoin::Transaction>>> {
        self.completed()?
            .into_iter()
            .map(|c| Ok(Adapter::new(c.tx()?.0)))
            .collect()
    }

    #[query]
    pub fn signing(&self) -> Result<Option<SigningCheckpoint<'_>>> {
        if self.queue.len() < 2 {
            return Ok(None);
        }

        let second = self.get(self.index - 1)?;
        if !matches!(second.status, CheckpointStatus::Signing) {
            return Ok(None);
        }

        Ok(Some(SigningCheckpoint(second)))
    }

    pub fn signing_mut(&mut self) -> Result<Option<SigningCheckpointMut>> {
        if self.queue.len() < 2 {
            return Ok(None);
        }

        let second = self.get_mut(self.index - 1)?;
        if !matches!(second.status, CheckpointStatus::Signing) {
            return Ok(None);
        }

        Ok(Some(SigningCheckpointMut(second)))
    }

    pub fn building(&self) -> Result<BuildingCheckpoint> {
        let last = self.get(self.index)?;
        Ok(BuildingCheckpoint(last))
    }

    pub fn building_mut(&mut self) -> Result<BuildingCheckpointMut> {
        let last = self.get_mut(self.index)?;
        Ok(BuildingCheckpointMut(last))
    }

    pub fn maybe_step(&mut self, sig_keys: &Map<ConsensusKey, Xpub>) -> Result<()> {
        #[cfg(not(feature = "full"))]
        unimplemented!();

        #[cfg(feature = "full")]
        {
            if self.signing()?.is_some() {
                return Ok(());
            }

            if !self.queue.is_empty() {
                let now = self
                    .context::<Time>()
                    .ok_or_else(|| OrgaError::App("No time context".to_string()))?
                    .seconds as u64;
                let elapsed = now - self.building()?.create_time();
                if elapsed < MIN_CHECKPOINT_INTERVAL {
                    return Ok(());
                }

                if elapsed < MAX_CHECKPOINT_INTERVAL || self.index == 0 {
                    let building = self.building()?;
                    let has_pending_deposit = if self.index == 0 {
                        building.inputs.len() > 0
                    } else {
                        building.inputs.len() > 1
                    };

                    let has_pending_withdrawal = building.outputs.len() > 0;

                    if !has_pending_deposit && !has_pending_withdrawal {
                        return Ok(());
                    }
                }

                while let Some(first) = self.queue.front()? {
                    if now - first.create_time() <= MAX_AGE {
                        break;
                    }

                    self.queue.pop_front()?;
                }
            }

            if self.maybe_push(sig_keys)?.is_none() {
                return Ok(());
            }

            if self.index > 0 {
                let second = self.get_mut(self.index - 1)?;
                let sigset = second.sigset.clone();
                let (reserve_outpoint, reserve_value, excess_inputs, excess_outputs) =
                    BuildingCheckpointMut(second).advance()?;

                let mut building = self.building_mut()?;

                building.push_input(
                    reserve_outpoint,
                    &sigset,
                    &vec![0u8], // TODO: double-check safety
                    reserve_value,
                )?;

                for (input, sigs) in excess_inputs {
                    let shares = sigs.shares()?;
                    let data = input.into_inner().into();
                    building.inputs.push_back(data)?;
                    building.sig_queue.inputs.push_back(ThresholdSig::new())?;
                    building
                        .sig_queue
                        .inputs
                        .back_mut()?
                        .unwrap()
                        .from_shares(shares)?;
                }

                for output in excess_outputs {
                    let data = output.into_inner();
                    building.outputs.push_back(data)?;
                }
            }

            Ok(())
        }
    }

    fn maybe_push(
        &mut self,
        sig_keys: &Map<ConsensusKey, Xpub>,
    ) -> Result<Option<BuildingCheckpointMut>> {
        #[cfg(not(feature = "full"))]
        unimplemented!();

        #[cfg(feature = "full")]
        {
            let mut index = self.index;
            if !self.queue.is_empty() {
                index += 1;
            }

            let sigset = SignatorySet::from_validator_ctx(index, sig_keys)?;

            if sigset.possible_vp() == 0 {
                return Ok(None);
            }

            if !sigset.has_quorum() {
                return Ok(None);
            }

            self.index = index;

            self.queue.push_back(Checkpoint {
                sigset,
                ..Default::default()
            })?;

            let building = self.building_mut()?;
            Ok(Some(building))
        }
    }

    #[query]
    pub fn active_sigset(&self) -> Result<SignatorySet> {
        Ok(self.building()?.sigset.clone())
    }

    #[call]
    pub fn sign(&mut self, xpub: Xpub, sigs: LengthVec<u16, Signature>) -> Result<()> {
        super::exempt_from_fee()?;

        let mut signing = self
            .signing_mut()?
            .ok_or_else(|| Error::Orga(OrgaError::App("No checkpoint to be signed".to_string())))?;

        signing.sign(xpub, sigs)?;

        if signing.done() {
            println!("done. {:?}", signing.tx()?);
            signing.advance()?;
        }

        Ok(())
    }

    #[query]
    pub fn to_sign(&self, xpub: Xpub) -> Result<Vec<([u8; 32], u32)>> {
        self.signing()?
            .ok_or_else(|| OrgaError::App("No checkpoint to be signed".to_string()))?
            .to_sign(xpub)
    }

    #[query]
    pub fn sigset(&self, index: u32) -> Result<SignatorySet> {
        Ok(self.get(index)?.sigset.clone())
    }
}
