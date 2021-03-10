use anyhow::Result;
use bdk::blockchain::{noop_progress, ConfigurableBlockchain, LogProgress};
use bdk::blockchain::{Blockchain, ElectrumBlockchain};
use bdk::database::MemoryDatabase;
use bdk::electrum_client::{Client, ElectrumApi};
use bdk::FeeRate;
use bitcoin::consensus::serialize;
use bitcoin::util::psbt::PartiallySignedTransaction;
use bitcoin::{Amount, Network, Transaction};
use bitcoin_harness::BitcoindRpcApi;
use hyper::body::Buf;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use url::Url;

use bdk_test::Wallet;

const BITCOIND_RPC_PORT: u16 = 7041;
const ELECTRUM_RPC_PORT: u16 = 60401;
const ELECTRUM_HTTP_PORT: u16 = 3012;
const USERNAME: &str = "admin";
const PASSWORD: &str = "123";

#[tokio::main]
async fn main() {
    // note: you need to generate 101 blocks first
    // note2: miner wallet name is `miner`
    // run setup function above ;)

    let bitcoind_url = {
        let input = format!(
            "http://{}:{}@localhost:{}",
            USERNAME, PASSWORD, BITCOIND_RPC_PORT
        );
        Url::parse(&input).unwrap()
    };

    let bitcoind_client = bitcoin_harness::Client::new(bitcoind_url.clone());
    let miner = bitcoind_client.with_wallet("miner").unwrap();
    let miner_address = bitcoin::Address::from_str("2NFpmXDDezMjEtifbmJaxq8wwvAaDMrhvGe").unwrap(); // just a hardcoded address

    let bdk_url = {
        let input = format!("tcp://@localhost:{}", ELECTRUM_RPC_PORT);
        Url::parse(&input).unwrap()
    };

    let bdk_wallet = Wallet::new(&bdk_url).await?;

    bdk_wallet.sync().await.unwrap();

    // get some money for our bdk wallet if n balance
    while bdk_wallet.balance().await.unwrap() <= 200_000 {
        miner
            .send_to_address("miner", address.clone(), Amount::from_sat(100_000_000))
            .await
            .unwrap();

        bdk_wallet.sync().await.unwrap();
    }

    let balance = bdk_wallet.balance().await.unwrap();
    println!("balance: {}", balance);

    let address = bdk_wallet.new_address().await.unwrap();
    println!("target address: {}", address);

    let (psbt, details) = bdk_wallet
        .create_tx(
            bdk::TxBuilder::with_recipients(vec![(address.script_pubkey(), 200_000)])
                .fee_rate(FeeRate::from_sat_per_vb(5.0)),
        )
        .unwrap();

    let (signed_psbt, finalized) = bdk_wallet.sign(psbt, None).unwrap();
    debug_assert!(finalized);

    let txid = bdk_wallet.broadcast(signed_psbt.extract_tx()).unwrap();
    println!("txid: {}", txid);

    let block_header_old = bdk_wallet.client().get_height().unwrap();
    println!("Current block height: {} ", block_header_old);
    bitcoind_client
        .generatetoaddress(2, miner_address, None)
        .await
        .unwrap();

    // update to get the latest 2 blocks
    bdk_wallet.sync(noop_progress(), None).unwrap();
}
