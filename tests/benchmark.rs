#![feature(async_closure)]
use bitcoin::blockdata::transaction::EcdsaSighashType;
use bitcoin::util::bip32::{ChildNumber, ExtendedPrivKey, ExtendedPubKey};
use bitcoin::{secp256k1, Script};
use bitcoincore_rpc_async::RpcApi as AsyncRpcApi;
use bitcoind::bitcoincore_rpc::json::{
    ImportMultiRequest, ImportMultiRequestScriptPubkey, ImportMultiRescanSince,
};
use bitcoind::bitcoincore_rpc::RpcApi;
use bitcoind::{BitcoinD, Conf};
use chrono::TimeZone;
use chrono::Utc;
use futures::FutureExt;
use log::info;
use nomic::app::Dest;
use nomic::app::{InnerApp, Nom};
use nomic::bitcoin::adapter::Adapter;
use nomic::bitcoin::checkpoint::CheckpointStatus;
use nomic::bitcoin::checkpoint::Config as CheckpointConfig;
use nomic::bitcoin::deposit_index::{Deposit, DepositInfo};
use nomic::bitcoin::header_queue::Config as HeaderQueueConfig;
use nomic::bitcoin::relayer::DepositAddress;
use nomic::bitcoin::relayer::Relayer;
use nomic::bitcoin::signatory::SignatorySet;
use nomic::bitcoin::signer::Signer;
use nomic::bitcoin::threshold_sig::Pubkey;
use nomic::bitcoin::Config as BitcoinConfig;
use nomic::constants::DEFAULT_MAX_SCAN_CHECKPOINTS_CONFIRMATIONS;
use nomic::error::{Error, Result};
use nomic::utils::*;
use nomic::utils::{
    declare_validator, poll_for_active_sigset, poll_for_blocks, poll_for_updated_balance,
    populate_bitcoin_block, retry, set_time, setup_test_app, setup_test_signer,
    test_bitcoin_client, NomicTestWallet,
};
use orga::abci::Node;
use orga::client::{
    wallet::{DerivedKey, Unsigned},
    AppClient,
};
use orga::coins::{Address, Amount};
use orga::encoding::Encode;
use orga::macros::build_call;
use orga::plugins::{load_privkey, Time, MIN_FEE};
use orga::tendermint::client::HttpClient;
use rand::Rng;
use reqwest::StatusCode;
use serial_test::serial;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::ops::Deref;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::{Arc, Once};
use std::time::Duration;
use tempfile::tempdir;
use tokio::sync::mpsc;

static INIT: Once = Once::new();

fn app_client() -> AppClient<InnerApp, InnerApp, orga::tendermint::client::HttpClient, Nom, Unsigned>
{
    nomic::app_client("http://localhost:26657")
}

async fn generate_deposit_address(address: &Address) -> Result<DepositAddress> {
    let (sigset, threshold): (SignatorySet, (u64, u64)) = app_client()
        .query(|app: InnerApp| {
            Ok((
                app.bitcoin.checkpoints.active_sigset()?,
                app.bitcoin.checkpoints.config.sigset_threshold,
            ))
        })
        .await?;
    info!(
        "Generating deposit address for {} with sigset index: {} ...",
        address, sigset.index
    );
    let script = sigset.output_script(
        Dest::Address(*address).commitment_bytes()?.as_slice(),
        threshold,
    )?;

    Ok(DepositAddress {
        deposit_addr: bitcoin::Address::from_script(&script, bitcoin::Network::Regtest)
            .unwrap()
            .to_string(),
        sigset_index: sigset.index(),
    })
}

pub async fn broadcast_deposit_addr(
    dest_addr: String,
    sigset_index: u32,
    relayer: String,
    deposit_addr: String,
) -> Result<()> {
    info!("Broadcasting deposit address to relayer...");
    let dest_addr = dest_addr.parse().unwrap();

    let commitment = Dest::Address(dest_addr).encode()?;

    let url = format!("{}/address", relayer,);
    let client = reqwest::Client::new();
    let res = client
        .post(url)
        .query(&[
            ("sigset_index", &sigset_index.to_string()),
            ("deposit_addr", &deposit_addr),
        ])
        .body(commitment)
        .send()
        .await
        .unwrap();

    match res.status() {
        StatusCode::OK => Ok(()),
        _ => Err(Error::Relayer(format!("{}", res.text().await.unwrap()))),
    }
}

async fn set_recovery_address(nomic_account: NomicTestWallet) -> Result<()> {
    info!("Setting recovery address...");

    app_client()
        .with_wallet(nomic_account.wallet)
        .call(
            move |app| build_call!(app.accounts.take_as_funding((MIN_FEE).into())),
            move |app| {
                build_call!(app
                    .bitcoin
                    .set_recovery_script(Adapter::new(nomic_account.script.clone())))
            },
        )
        .await?;
    info!("Validator declared");
    Ok(())
}

async fn deposit_bitcoin(
    address: Address,
    btc: bitcoin::Amount,
    wallet: Arc<bitcoind::bitcoincore_rpc::Client>,
    times: u64,
    parallel: bool,
) -> Result<()> {
    let deposit_address = generate_deposit_address(&address).await.unwrap();
    broadcast_deposit_addr(
        address.to_string(),
        deposit_address.sigset_index,
        "http://localhost:8999".to_string(),
        deposit_address.deposit_addr.clone(),
    )
    .await?;

    if parallel {
        let mut handles = vec![];
        for _ in 0..times {
            let new_wallet = wallet.clone();
            let deposit_address = deposit_address.clone();
            let handle = tokio::spawn(async move {
                new_wallet
                    .send_to_address(
                        &bitcoin::Address::from_str(&deposit_address.deposit_addr).unwrap(),
                        btc,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                    )
                    .unwrap();
            });
            handles.push(handle);
        }
        for handle in handles {
            handle.await.expect("One of the tasks failed");
        }
    } else {
        for _ in 0..times {
            let new_wallet = wallet.clone();
            let deposit_address = deposit_address.clone();
            new_wallet
                .send_to_address(
                    &bitcoin::Address::from_str(&deposit_address.deposit_addr).unwrap(),
                    btc,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                )
                .unwrap();
        }
    }

    Ok(())
}

async fn withdraw_bitcoin(
    nomic_account: &NomicTestWallet,
    amount: bitcoin::Amount,
    dest_address: &bitcoin::Address,
) -> Result<()> {
    let dest_script = nomic::bitcoin::adapter::Adapter::new(dest_address.script_pubkey());
    let usats = amount.to_sat() * 1_000_000;
    app_client()
        .with_wallet(nomic_account.wallet.clone())
        .call(
            move |app| build_call!(app.withdraw_nbtc(dest_script, Amount::from(usats))),
            |app| build_call!(app.app_noop()),
        )
        .await?;
    Ok(())
}

async fn get_signatory_script() -> Result<Script> {
    Ok(app_client()
        .query(|app: InnerApp| {
            let tx = app.bitcoin.checkpoints.emergency_disbursal_txs()?;
            Ok(tx[0].output[1].script_pubkey.clone())
        })
        .await?)
}

fn client_provider() -> AppClient<InnerApp, InnerApp, HttpClient, Nom, DerivedKey> {
    let val_priv_key = load_privkey().unwrap();
    let wallet = DerivedKey::from_secret_key(val_priv_key);
    app_client().with_wallet(wallet)
}

#[tokio::test]
#[serial]
#[ignore]
async fn benchmark_deposit() {
    INIT.call_once(|| {
        pretty_env_logger::init();
        let genesis_time = Utc.with_ymd_and_hms(2022, 10, 5, 0, 0, 0).unwrap();
        let time = Time::from_seconds(genesis_time.timestamp());
        set_time(time);
    });

    let mut conf = Conf::default();
    conf.args.push("-txindex");
    let bitcoind = BitcoinD::with_conf(bitcoind::downloaded_exe_path().unwrap(), &conf).unwrap();
    let rpc_url = bitcoind.rpc_url();
    let cookie_file = bitcoind.params.cookie_file.clone();
    let btc_client = test_bitcoin_client(rpc_url.clone(), cookie_file.clone()).await;

    let block_data = populate_bitcoin_block(&btc_client).await;

    let home = tempdir().unwrap();
    let path = home.into_path();

    let node_path = path.clone();
    let signer_path = path.clone();
    let xpriv = generate_bitcoin_key(bitcoin::Network::Regtest).unwrap();
    fs::create_dir_all(signer_path.join("signer")).unwrap();
    fs::write(
        signer_path.join("signer/xpriv"),
        xpriv.to_string().as_bytes(),
    )
    .unwrap();
    let xpub = ExtendedPubKey::from_priv(&secp256k1::Secp256k1::new(), &xpriv);
    let header_relayer_path = path.clone();

    std::env::set_var("NOMIC_HOME_DIR", &path);

    let headers_config = HeaderQueueConfig {
        encoded_trusted_header: Adapter::new(block_data.block_header)
            .encode()
            .unwrap()
            .try_into()
            .unwrap(),
        trusted_height: block_data.height,
        retargeting: false,
        min_difficulty_blocks: true,
        max_length: 59,
        ..Default::default()
    };

    let checkpoint_config = CheckpointConfig {
        user_fee_factor: 21000,
        max_inputs: 1_000_000_000,
        ..Default::default()
    };
    let funded_accounts = setup_test_app(
        &path,
        4,
        Some(headers_config),
        Some(checkpoint_config),
        None,
    );

    let node = Node::<nomic::app::App>::new(node_path, Some("nomic-e2e"), Default::default());
    let _node_child = node.await.run().await.unwrap();

    let rpc_addr = "http://localhost:26657".to_string();

    let mut relayer = Relayer::new(
        test_bitcoin_client(rpc_url.clone(), cookie_file.clone()).await,
        rpc_addr.clone(),
    );
    let headers = relayer.start_header_relay();

    let relayer = Relayer::new(
        test_bitcoin_client(rpc_url.clone(), cookie_file.clone()).await,
        rpc_addr.clone(),
    );
    let deposits = relayer.start_deposit_relay(&header_relayer_path, 60 * 60 * 12);

    let mut relayer = Relayer::new(
        test_bitcoin_client(rpc_url.clone(), cookie_file.clone()).await,
        rpc_addr.clone(),
    );
    let checkpoints = relayer.start_checkpoint_relay();

    let mut relayer = Relayer::new(
        test_bitcoin_client(rpc_url.clone(), cookie_file.clone()).await,
        rpc_addr.clone(),
    );
    let checkpoints_conf =
        relayer.start_checkpoint_conf_relay(DEFAULT_MAX_SCAN_CHECKPOINTS_CONFIRMATIONS);

    let mut relayer = Relayer::new(
        test_bitcoin_client(rpc_url.clone(), cookie_file.clone()).await,
        rpc_addr.clone(),
    );
    let disbursal = relayer.start_emergency_disbursal_transaction_relay();

    let signer = async {
        tokio::time::sleep(Duration::from_secs(10)).await;
        setup_test_signer(&signer_path, client_provider)
            .start()
            .await
    };

    let (tx, mut rx) = mpsc::channel(100);
    let shutdown_listener = async {
        rx.recv().await;
        Err::<(), Error>(Error::Test("Signer shutdown initiated".to_string()))
    };

    let slashable_signer_xpriv = generate_bitcoin_key(bitcoin::Network::Regtest).unwrap();
    let slashable_signer_xpub = ExtendedPubKey::from_priv(
        &secp256k1::Secp256k1::new(),
        &slashable_signer_xpriv.clone(),
    );
    let slashable_signer = async {
        tokio::time::sleep(Duration::from_secs(15)).await;
        let privkey_bytes = funded_accounts[2].privkey.secret_bytes();
        let privkey = orga::secp256k1::SecretKey::from_slice(&privkey_bytes).unwrap();
        let signer = Signer::new(
            address_from_privkey(&funded_accounts[2].privkey),
            vec![slashable_signer_xpriv],
            0.1,
            1.0,
            0,
            None,
            || {
                let wallet = DerivedKey::from_secret_key(privkey);
                app_client().with_wallet(wallet)
            },
            None,
        )
        .start();

        match futures::try_join!(signer, shutdown_listener) {
            Err(Error::Test(_)) | Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    };

    let test = async {
        let val_priv_key = load_privkey().unwrap();
        let nomic_wallet = DerivedKey::from_secret_key(val_priv_key);
        let consensus_key = load_consensus_key(&path)?;
        declare_validator(consensus_key, nomic_wallet, 100_000)
            .await
            .unwrap();
        app_client()
            .with_wallet(DerivedKey::from_secret_key(val_priv_key))
            .call(
                |app| build_call!(app.accounts.take_as_funding(MIN_FEE.into())),
                |app| build_call!(app.bitcoin.set_signatory_key(xpub.into())),
            )
            .await?;

        let privkey_bytes = funded_accounts[2].privkey.secret_bytes();
        let privkey: orga::secp256k1::SecretKey =
            orga::secp256k1::SecretKey::from_slice(&privkey_bytes).unwrap();
        declare_validator([0; 32], funded_accounts[2].wallet.clone(), 4_000)
            .await
            .unwrap();
        app_client()
            .with_wallet(DerivedKey::from_secret_key(privkey))
            .call(
                |app| build_call!(app.accounts.take_as_funding(MIN_FEE.into())),
                |app| build_call!(app.bitcoin.set_signatory_key(slashable_signer_xpub.into())),
            )
            .await?;

        let wallet = retry(|| bitcoind.create_wallet("nomic-integration-test"), 10).unwrap();
        let wallet_rc = Arc::new(wallet);
        let wallet_address = wallet_rc.get_new_address(None, None).unwrap();
        let async_wallet_address =
            bitcoincore_rpc_async::bitcoin::Address::from_str(&wallet_address.to_string()).unwrap();
        // let withdraw_address = wallet.get_new_address(None, None).unwrap();

        let mut labels = vec![];
        for i in 0..funded_accounts.len() {
            labels.push(format!("funded-account-{}", i));
        }

        let mut import_multi_reqest = vec![];
        for (i, account) in funded_accounts.iter().enumerate() {
            import_multi_reqest.push(ImportMultiRequest {
                timestamp: ImportMultiRescanSince::Now,
                descriptor: None,
                script_pubkey: Some(ImportMultiRequestScriptPubkey::Script(&account.script)),
                redeem_script: None,
                witness_script: None,
                pubkeys: &[],
                keys: &[],
                range: None,
                internal: None,
                watchonly: Some(true),
                label: Some(&labels[i]),
                keypool: None,
            });
        }

        wallet_rc
            .import_multi(import_multi_reqest.as_slice(), None)
            .unwrap();

        set_recovery_address(funded_accounts[0].clone())
            .await
            .unwrap();

        btc_client
            .generate_to_address(1200, &async_wallet_address)
            .await
            .unwrap();

        poll_for_bitcoin_header(2200).await.unwrap();

        let expected_balance = 0;
        let balance = poll_for_updated_balance(funded_accounts[0].address, expected_balance).await;
        assert_eq!(balance, expected_balance);

        poll_for_active_sigset().await;
        poll_for_signatory_key(consensus_key).await;

        deposit_bitcoin(
            funded_accounts[0].address.clone(),
            bitcoin::Amount::from_btc(0.001).unwrap(),
            wallet_rc.clone(),
            10000,
            true,
        )
        .await
        .unwrap();

        let expected_balance = 0;
        let balance = poll_for_updated_balance(funded_accounts[0].address, expected_balance).await;
        assert_eq!(balance, expected_balance);

        loop {
            let deposits = reqwest::get(format!(
                "http://localhost:8999/pending_deposits?receiver={}",
                &funded_accounts[0].address
            ))
            .await
            .unwrap()
            .json::<Vec<DepositInfo>>()
            .await
            .unwrap();

            if !deposits.is_empty() {
                println!("Deposits: {:?}", deposits);
                break;
            }
            tokio::time::sleep(Duration::from_secs(5)).await;
        }

        btc_client
            .generate_to_address(4, &async_wallet_address)
            .await
            .unwrap();

        poll_for_bitcoin_header(2204).await.unwrap();
        poll_for_signing_checkpoint().await;

        let expected_balance = 0;
        let balance = poll_for_updated_balance(funded_accounts[0].address, expected_balance).await;
        assert_eq!(balance, expected_balance);

        let confirmed_index = app_client()
            .query(|app: InnerApp| Ok(app.bitcoin.checkpoints.confirmed_index))
            .await
            .unwrap();
        assert_eq!(confirmed_index, None);

        // balance only gets updated after moving pass bitcoin header & checkpoint has completed
        poll_for_completed_checkpoint(1).await;

        let transaction_data = app_client()
            .query(|app: InnerApp| {
                let checkpoint = app.bitcoin.checkpoints.get(0)?;
                let building_checkpoint = checkpoint.checkpoint_tx()?.into_inner();
                Ok(building_checkpoint)
            })
            .await
            .unwrap();
        println!("Data: {:?}", transaction_data);

        // Wait sometime for checkpoint to be broadcasted on the network
        tokio::time::sleep(Duration::from_secs(5)).await;
        btc_client
            .generate_to_address(1, &async_wallet_address)
            .await
            .unwrap();
        poll_for_bitcoin_header(2205).await.unwrap();
        poll_for_confirmed_checkpoint(0).await;

        // what does this do?
        tx.send(Some(())).await.unwrap();

        Err::<(), Error>(Error::Test("Test completed successfully".to_string()))
    };

    poll_for_blocks().await;

    match futures::try_join!(
        headers,
        deposits,
        checkpoints,
        checkpoints_conf,
        disbursal,
        signer,
        slashable_signer,
        test
    ) {
        Err(Error::Test(_)) => (),
        Ok(_) => (),
        other => {
            other.unwrap();
        }
    }
}
