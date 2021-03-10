use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use bdk::blockchain::{noop_progress, ConfigurableBlockchain, LogProgress};
use bdk::blockchain::{Blockchain, ElectrumBlockchain};
use bdk::database::MemoryDatabase;
use bdk::electrum_client::{Client, ElectrumApi};
use bitcoin::consensus::serialize;
use bitcoin::util::psbt::PartiallySignedTransaction;
use bitcoin::{Amount, Network, Transaction};
use bitcoin_harness::BitcoindRpcApi;
use hyper::body::Buf;
use url::Url;

use bdk::FeeRate;
use std::str::FromStr;

const BITCOIND_RPC_PORT: u16 = 7041;
const ELECTRUM_RPC_PORT: u16 = 60401;
const ELECTRUM_HTTP_PORT: u16 = 3012;
const USERNAME: &str = "admin";
const PASSWORD: &str = "123";

/// Create a test wallet, generate enough block to fund it and activate segwit.
/// Generate enough blocks to make the passed `spendable_quantity` spendable.
/// Spawn a tokio thread to mine a new block every second.
pub async fn init_wallet(spendable_quantity: u32, wallet_name: &str, node_url: Url) -> Result<()> {
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

async fn mine(
    bitcoind_client: bitcoin_harness::Client,
    reward_address: bitcoin::Address,
) -> Result<()> {
    loop {
        tokio::time::delay_for(Duration::from_secs(1)).await;
        bitcoind_client
            .generatetoaddress(1, reward_address.clone(), None)
            .await?;
    }
}

#[tokio::main]
async fn main() {
    // note: you need to generate 101 blocks first
    // note2: miner wallet name is `miner`
    let bitcoind_url = {
        let input = format!(
            "http://{}:{}@localhost:{}",
            USERNAME, PASSWORD, BITCOIND_RPC_PORT
        );
        Url::parse(&input).unwrap()
    };

    init_wallet(1, "miner", bitcoind_url).await.unwrap();

    loop {}
}
