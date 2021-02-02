mod bdk_wallet;

use bitcoin_harness::BitcoindRpcApi;
use anyhow::Result;
use url::Url;
use std::time::Duration;
use crate::bdk_wallet::BdkWallet;
use bitcoin::{Network, Amount, Transaction};
use bitcoin::consensus::serialize;
use bdk::blockchain::{noop_progress, ConfigurableBlockchain, LogProgress};
use bdk::blockchain::ElectrumBlockchain;
use bdk::blockchain::CompactFiltersBlockchain;
use hyper::body::Buf;
use bdk::electrum_client::{Client, ElectrumApi};
use bdk::database::MemoryDatabase;
use bitcoin::util::psbt::PartiallySignedTransaction;
use bdk::blockchain::compact_filters::{CompactFiltersBlockchainConfig, Peer, Mempool};
use bdk::blockchain::AnyBlockchain::CompactFilters;
use std::sync::Arc;

const BITCOIND_RPC_PORT: u16 = 7041;
const ELECTRUM_RPC_PORT: u16 = 60401;
const ELECTRUM_HTTP_PORT: u16 = 3012;
const USERNAME: &str = "admin";
const PASSWORD: &str = "123";

/// Create a test wallet, generate enough block to fund it and activate segwit.
   /// Generate enough blocks to make the passed `spendable_quantity` spendable.
   /// Spawn a tokio thread to mine a new block every second.
pub async fn init_wallet(spendable_quantity: u32, wallet_name: &str,  node_url: Url) -> Result<()> {

    println!("node url: {}", node_url);

    let bitcoind_client = bitcoin_harness::Client::new(node_url.clone());

    bitcoind_client
        .createwallet(&wallet_name, None, None, None, None)
        .await?;

    let reward_address = bitcoind_client
        .with_wallet(wallet_name)?
        .getnewaddress(None, None)
        .await?;

    bitcoind_client
        .generatetoaddress(101 + spendable_quantity, reward_address.clone(), None)
        .await?;
    let _ = tokio::spawn(mine(bitcoind_client, reward_address));

    Ok(())
}


async fn mine(bitcoind_client: bitcoin_harness::Client, reward_address: bitcoin::Address) -> Result<()> {
    loop {
        tokio::time::delay_for(Duration::from_secs(1)).await;
        bitcoind_client
            .generatetoaddress(1, reward_address.clone(), None)
            .await?;
    }
}

#[tokio::main]
async fn main() {
    let bitcoind_url = {
        let input = format!(
            "http://{}:{}@localhost:{}",
            USERNAME, PASSWORD, BITCOIND_RPC_PORT
        );
        Url::parse(&input).unwrap()
    };

    let bdk_url = {
        let input = format!(
            "tcp://@localhost:{}",
            ELECTRUM_RPC_PORT
        );
        Url::parse(&input).unwrap()
    };

    let electrum_http_url = {
        let input = format!(
            "http://127.0.0.1:{}/blocks/tip/height",
            ELECTRUM_HTTP_PORT
        );
        Url::parse(&input).unwrap()
    };

    let bitcoind_client = bitcoin_harness::Client::new(bitcoind_url.clone());
    let network = bitcoind_client.network().await.unwrap();
    println!("network: {}", network);

    //init_wallet(5, "alice", bitcoind_url).await.unwrap();

    //tokio::time::delay_for(Duration::from_secs(5)).await;


    let client = bdk::electrum_client::Client::new(bdk_url.as_str()).unwrap();

    //let client = Client::new("ssl://electrum.blockstream.info:60002").unwrap();

    let blockchain = ElectrumBlockchain::from(client);

    let bdk_wallet = bdk::Wallet::new(
        "wpkh([c258d2e4/84h/1h/0h]tpubDDYkZojQFQjht8Tm4jsS3iuEmKjTiEGjG6KnuFNKKJb5A6ZUCUZKdvLdSDWofKi4ToRCwb9poe1XdqfUnP4jaJjCB2Zwv11ZLgSbnZSNecE/0/*)",
        Some("wpkh([c258d2e4/84h/1h/0h]tpubDDYkZojQFQjht8Tm4jsS3iuEmKjTiEGjG6KnuFNKKJb5A6ZUCUZKdvLdSDWofKi4ToRCwb9poe1XdqfUnP4jaJjCB2Zwv11ZLgSbnZSNecE/1/*)"),
        network,
        MemoryDatabase::default(),
        blockchain).unwrap();

    let addr =  bdk_wallet.get_new_address().unwrap();

    let addr2 =  bdk_wallet.get_new_address().unwrap();

    println!("addr: {}", addr);

    //bdk_wallet.sync(noop_progress(), None).unwrap();

    // need to mine ~100+ blocks to allow use of coinbase transactions to fun tx lock
    bitcoind_client
        .generatetoaddress(101, addr.clone(), None)
        .await.unwrap();


    bdk_wallet.sync(noop_progress(), None).unwrap();

    let balance = bdk_wallet.get_balance().unwrap();
    println!("balance: {}", balance);

    let psbt = bdk_wallet::build_tx_lock_psbt(&bdk_wallet, addr.clone(), Amount::from_sat(20000)).await.unwrap();

    //println!("psbt: {:?}", psbt);

    let (signed_psbt, _) = bdk_wallet.sign(psbt, None).unwrap();

    let txid = bdk_wallet.broadcast(signed_psbt.extract_tx()).unwrap();

    bitcoind_client
        .generatetoaddress(1, addr.clone(), None)
        .await.unwrap();

    //bdk_wallet.sync(noop_progress(), None).unwrap();

    bitcoind_client
        .generatetoaddress(2, addr, None)
        .await.unwrap();

    let height = bdk_wallet::get_block_height(electrum_http_url).await;
    let height: usize = height.parse().unwrap();

    println!("height: {:?}", height);

    let client = bdk::electrum_client::Client::new(bdk_url.as_str()).unwrap();
    let merkle = client.transaction_get_merkle(&txid, height).unwrap();

    println!("merkle height: {:?}", merkle);




    // println!("initialised bdk wallet");





}
