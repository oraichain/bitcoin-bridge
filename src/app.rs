use crate::bitcoin::Bitcoin;

use orga::plugins::sdk_compat::{sdk, sdk::Tx as SdkTx, ConvertSdkTx};
use orga::prelude::*;
use orga::Error;
use serde::{Deserialize, Serialize};

pub const CHAIN_ID: &str = "nomic-testnet-4";
pub type App = DefaultPlugins<Nom, InnerApp, CHAIN_ID>;

#[derive(State, Debug, Clone)]
pub struct Nom(());
impl Symbol for Nom {
    const INDEX: u8 = 69;
}

const DEV_ADDRESS: &str = "nomic14z79y3yrghqx493mwgcj0qd2udy6lm26lmduah";
const STRATEGIC_RESERVE_ADDRESS: &str = "nomic1d5n325zrf4elfu0heqd59gna5j6xyunhev23cj";
const VALIDATOR_BOOTSTRAP_ADDRESS: &str = "nomic1fd9mxxt84lw3jdcsmjh6jy8m6luafhqd8dcqeq";

#[derive(State, Call, Query, Client)]
pub struct InnerApp {
    pub accounts: Accounts<Nom>,
    pub staking: Staking<Nom>,
    pub atom_airdrop: Airdrop<Nom>,

    community_pool: Coin<Nom>,
    incentive_pool: Coin<Nom>,

    staking_rewards: Faucet<Nom>,
    dev_rewards: Faucet<Nom>,
    community_pool_rewards: Faucet<Nom>,
    incentive_pool_rewards: Faucet<Nom>,

    pub bitcoin: Bitcoin,
    pub reward_timer: RewardTimer,
}

impl InnerApp {
    #[call]
    pub fn noop(&mut self) {}

    #[call]
    pub fn deposit_rewards(&mut self) -> Result<()> {
        self.accounts.give_from_funding_all()?;
        self.bitcoin.accounts.give_from_funding_all()?;
        Ok(())
    }

    pub fn community_pool(&self) -> Coin<Nom> {
        self.community_pool.amount.into()
    }

    pub fn incentive_pool(&self) -> Coin<Nom> {
        self.incentive_pool.amount.into()
    }

    pub fn staking_rewards(&self) -> Faucet<Nom> {
        self.staking_rewards.clone()
    }

    pub fn dev_rewards(&self) -> Faucet<Nom> {
        self.dev_rewards.clone()
    }

    pub fn community_pool_rewards(&self) -> Faucet<Nom> {
        self.community_pool_rewards.clone()
    }

    pub fn incentive_pool_rewards(&self) -> Faucet<Nom> {
        self.incentive_pool_rewards.clone()
    }
}

#[cfg(feature = "full")]
mod abci {
    use super::*;

    impl BeginBlock for InnerApp {
        fn begin_block(&mut self, ctx: &BeginBlockCtx) -> Result<()> {
            self.staking.begin_block(ctx)?;

            let has_stake = self.staking.staked()? > 0;
            if has_stake {
                let reward = self.staking_rewards.mint()?;
                self.staking.give(reward)?;
            }

            let dev_reward = self.dev_rewards.mint()?;
            let dev_address = DEV_ADDRESS.parse().unwrap();
            self.accounts.deposit(dev_address, dev_reward)?;

            let cp_reward = self.community_pool_rewards.mint()?;
            self.community_pool.give(cp_reward)?;

            let ip_reward = self.incentive_pool_rewards.mint()?;
            self.incentive_pool.give(ip_reward)?;

            self.bitcoin.begin_block(ctx)?;

            let now = ctx.header.time.as_ref().unwrap().seconds;
            let has_nbtc_rewards = self.bitcoin.reward_pool.amount > 0;
            if self.reward_timer.tick(now) && has_stake && has_nbtc_rewards {
                let reward_rate = (Amount::new(1) / Amount::new(2377))?; // ~0.00042069
                let reward_amount = (self.bitcoin.reward_pool.amount * reward_rate)?.amount()?;
                let reward = self.bitcoin.reward_pool.take(reward_amount)?;
                self.staking.give(reward)?;
            }

            Ok(())
        }
    }

    impl EndBlock for InnerApp {
        fn end_block(&mut self, ctx: &EndBlockCtx) -> Result<()> {
            self.staking.end_block(ctx)
        }
    }
}

#[derive(State, Query, Call, Client)]
pub struct Airdrop<S: Symbol> {
    claimable: Accounts<S>,
}

impl<S: Symbol> Airdrop<S> {
    #[query]
    pub fn balance(&self, address: Address) -> Result<Option<Amount>> {
        let exists = self.claimable.exists(address)?;
        if !exists {
            return Ok(None);
        }

        let balance = self.claimable.balance(address)?;
        Ok(Some(balance))
    }

    #[call]
    pub fn claim(&mut self) -> Result<()> {
        let signer = self
            .context::<Signer>()
            .ok_or_else(|| Error::Signer("No Signer context available".into()))?
            .signer
            .ok_or_else(|| Error::Coins("Unauthorized account action".into()))?;

        let amount = self.claimable.balance(signer)?;
        self.claimable.take_as_funding(amount)
    }

    pub fn accounts(self) -> Accounts<S> {
        self.claimable
    }
}

impl ConvertSdkTx for InnerApp {
    type Output = PaidCall<<InnerApp as Call>::Call>;

    fn convert(&self, sdk_tx: &SdkTx) -> Result<PaidCall<<InnerApp as Call>::Call>> {
        let sender_address = sdk_tx.sender_address()?;

        if sdk_tx.msg.len() != 1 {
            return Err(Error::App("Invalid number of messages".into()));
        }
        let msg = &sdk_tx.msg[0];

        type AppCall = <InnerApp as Call>::Call;
        type AccountCall = <Accounts<Nom> as Call>::Call;
        type StakingCall = <Staking<Nom> as Call>::Call;
        type AirdropCall = <Airdrop<Nom> as Call>::Call;
        type BitcoinCall = <Bitcoin as Call>::Call;

        let get_amount = |coin: Option<&sdk::Coin>, expected_denom| -> Result<Amount> {
            let coin = coin.map_or_else(|| Err(Error::App("Empty amount".into())), Ok)?;
            if coin.denom != expected_denom {
                return Err(Error::App(format!(
                    "Invalid denom in amount: {}",
                    coin.denom,
                )));
            }

            let amount: u64 = coin.amount.parse()?;
            Ok(Amount::new(amount))
        };

        // TODO: move message validation/parsing into orga (e.g. with a message enum)

        match msg.type_.as_str() {
            "cosmos-sdk/MsgSend" => {
                let msg: sdk::MsgSend = serde_json::value::from_value(msg.value.clone())
                    .map_err(|e| Error::App(e.to_string()))?;

                let from: Address = msg
                    .from_address
                    .parse()
                    .map_err(|e: bech32::Error| Error::App(e.to_string()))?;
                if from != sender_address {
                    return Err(Error::App(
                        "'from_address' must match sender address".to_string(),
                    ));
                }

                let to: Address = msg
                    .to_address
                    .parse()
                    .map_err(|e: bech32::Error| Error::App(e.to_string()))?;

                if msg.amount.len() != 1 {
                    return Err(Error::App(
                        "'amount' must have exactly one element".to_string(),
                    ));
                }

                match msg.amount[0].denom.as_str() {
                    "unom" => {
                        let amount = get_amount(msg.amount.first(), "unom")?;

                        let funding_call = AccountCall::MethodTakeAsFunding(MIN_FEE.into(), vec![]);
                        let funding_call_bytes = funding_call.encode()?;
                        let payer_call = AppCall::FieldAccounts(funding_call_bytes);

                        let transfer_call = AccountCall::MethodTransfer(to, amount, vec![]);
                        let transfer_call_bytes = transfer_call.encode()?;
                        let paid_call = AppCall::FieldAccounts(transfer_call_bytes);

                        Ok(PaidCall {
                            payer: payer_call,
                            paid: paid_call,
                        })
                    }
                    "nsat" => {
                        let amount = get_amount(msg.amount.first(), "nsat")?;

                        let funding_call = BitcoinCall::MethodTransfer(to, amount, vec![]);
                        let funding_call_bytes = funding_call.encode()?;
                        let payer_call = AppCall::FieldBitcoin(funding_call_bytes);

                        Ok(PaidCall {
                            payer: payer_call,
                            paid: AppCall::MethodNoop(vec![]),
                        })
                    }
                    _ => Err(Error::App("Unknown denom".to_string())),
                }
            }

            "cosmos-sdk/MsgDelegate" => {
                let msg: sdk::MsgDelegate = serde_json::value::from_value(msg.value.clone())
                    .map_err(|e| Error::App(e.to_string()))?;

                let del_addr: Address = msg
                    .delegator_address
                    .parse()
                    .map_err(|e: bech32::Error| Error::App(e.to_string()))?;
                if del_addr != sender_address {
                    return Err(Error::App(
                        "'delegator_address' must match sender address".to_string(),
                    ));
                }

                let val_addr: Address = msg
                    .validator_address
                    .parse()
                    .map_err(|e: bech32::Error| Error::App(e.to_string()))?;
                let amount: u64 = get_amount(msg.amount.as_ref(), "unom")?.into();

                let funding_amt = MIN_FEE + amount;
                let funding_call = AccountCall::MethodTakeAsFunding(funding_amt.into(), vec![]);
                let funding_call_bytes = funding_call.encode()?;
                let payer_call = AppCall::FieldAccounts(funding_call_bytes);

                let delegate_call =
                    StakingCall::MethodDelegateFromSelf(val_addr, amount.into(), vec![]);
                let delegate_call_bytes = delegate_call.encode()?;
                let paid_call = AppCall::FieldStaking(delegate_call_bytes);

                Ok(PaidCall {
                    payer: payer_call,
                    paid: paid_call,
                })
            }

            "cosmos-sdk/MsgBeginRedelegate" => {
                let msg: sdk::MsgBeginRedelegate = serde_json::value::from_value(msg.value.clone())
                    .map_err(|e| Error::App(e.to_string()))?;

                let del_addr: Address = msg
                    .delegator_address
                    .parse()
                    .map_err(|e: bech32::Error| Error::App(e.to_string()))?;
                if del_addr != sender_address {
                    return Err(Error::App(
                        "'delegator_address' must match sender address".to_string(),
                    ));
                }

                let val_src_addr: Address = msg
                    .validator_src_address
                    .parse()
                    .map_err(|e: bech32::Error| Error::App(e.to_string()))?;
                let val_dst_addr: Address = msg
                    .validator_dst_address
                    .parse()
                    .map_err(|e: bech32::Error| Error::App(e.to_string()))?;

                let amount = get_amount(msg.amount.as_ref(), "unom")?;

                let funding_amt = MIN_FEE;
                let funding_call = AccountCall::MethodTakeAsFunding(funding_amt.into(), vec![]);
                let funding_call_bytes = funding_call.encode()?;
                let payer_call = AppCall::FieldAccounts(funding_call_bytes);

                let redelegate_call =
                    StakingCall::MethodRedelegateSelf(val_src_addr, val_dst_addr, amount, vec![]);
                let redelegate_call_bytes = redelegate_call.encode()?;
                let paid_call = AppCall::FieldStaking(redelegate_call_bytes);

                Ok(PaidCall {
                    payer: payer_call,
                    paid: paid_call,
                })
            }

            "cosmos-sdk/MsgUndelegate" => {
                let msg: sdk::MsgUndelegate = serde_json::value::from_value(msg.value.clone())
                    .map_err(|e| Error::App(e.to_string()))?;

                let del_addr: Address = msg
                    .delegator_address
                    .parse()
                    .map_err(|e: bech32::Error| Error::App(e.to_string()))?;
                if del_addr != sender_address {
                    return Err(Error::App(
                        "'delegator_address' must match sender address".to_string(),
                    ));
                }

                let val_addr: Address = msg
                    .validator_address
                    .parse()
                    .map_err(|e: bech32::Error| Error::App(e.to_string()))?;
                let amount = get_amount(msg.amount.as_ref(), "unom")?;

                let funding_amt = MIN_FEE;
                let funding_call = AccountCall::MethodTakeAsFunding(funding_amt.into(), vec![]);
                let funding_call_bytes = funding_call.encode()?;
                let payer_call = AppCall::FieldAccounts(funding_call_bytes);

                let undelegate_call = StakingCall::MethodUnbondSelf(val_addr, amount, vec![]);
                let undelegate_call_bytes = undelegate_call.encode()?;
                let paid_call = AppCall::FieldStaking(undelegate_call_bytes);

                Ok(PaidCall {
                    payer: payer_call,
                    paid: paid_call,
                })
            }

            "nomic/MsgClaimRewards" => {
                let msg = msg
                    .value
                    .as_object()
                    .ok_or_else(|| Error::App("Invalid message value".to_string()))?;
                if !msg.is_empty() {
                    return Err(Error::App("Message should be empty".to_string()));
                }

                let claim_call = StakingCall::MethodClaimAll(vec![]);
                let claim_call_bytes = claim_call.encode()?;
                let payer_call = AppCall::FieldStaking(claim_call_bytes);

                let paid_call = AppCall::MethodDepositRewards(vec![]);

                Ok(PaidCall {
                    payer: payer_call,
                    paid: paid_call,
                })
            }

            "nomic/MsgClaimAirdrop" => {
                let msg = msg
                    .value
                    .as_object()
                    .ok_or_else(|| Error::App("Invalid message value".to_string()))?;
                if !msg.is_empty() {
                    return Err(Error::App("Message should be empty".to_string()));
                }

                let claim_call = AirdropCall::MethodClaim(vec![]);
                let claim_call_bytes = claim_call.encode()?;
                let payer_call = AppCall::FieldAtomAirdrop(claim_call_bytes);

                let give_call = AccountCall::MethodGiveFromFundingAll(vec![]);
                let give_call_bytes = give_call.encode()?;
                let paid_call = AppCall::FieldAccounts(give_call_bytes);

                Ok(PaidCall {
                    payer: payer_call,
                    paid: paid_call,
                })
            }

            "nomic/MsgWithdraw" => {
                let msg: MsgWithdraw = serde_json::value::from_value(msg.value.clone())
                    .map_err(|e| Error::App(e.to_string()))?;

                let dest_addr: bitcoin::Address = msg
                    .dst_address
                    .parse()
                    .map_err(|e: bitcoin::util::address::Error| Error::App(e.to_string()))?;
                let dest_script = crate::bitcoin::adapter::Adapter::new(dest_addr.script_pubkey());

                let amount: u64 = msg
                    .amount
                    .parse()
                    .map_err(|e: std::num::ParseIntError| Error::App(e.to_string()))?;

                let funding_amt = MIN_FEE;
                let funding_call = AccountCall::MethodTakeAsFunding(funding_amt.into(), vec![]);
                let funding_call_bytes = funding_call.encode()?;
                let payer_call = AppCall::FieldAccounts(funding_call_bytes);

                let withdraw_call = BitcoinCall::MethodWithdraw(dest_script, amount.into(), vec![]);
                let withdraw_call_bytes = withdraw_call.encode()?;
                let paid_call = AppCall::FieldBitcoin(withdraw_call_bytes);

                Ok(PaidCall {
                    payer: payer_call,
                    paid: paid_call,
                })
            }

            _ => Err(Error::App("Unsupported message type".into())),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct MsgWithdraw {
    pub amount: String,
    pub dst_address: String,
}

const REWARD_TIMER_PERIOD: i64 = 120;

#[derive(State, Call, Query, Client)]
pub struct RewardTimer {
    last_period: i64,
}

impl RewardTimer {
    pub fn tick(&mut self, now: i64) -> bool {
        if now - self.last_period < REWARD_TIMER_PERIOD {
            return false;
        }

        self.last_period = now;
        true
    }
}
