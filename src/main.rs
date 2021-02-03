use std::time::Duration;

use anyhow::Result;
use bdk::blockchain::noop_progress;
use bdk::blockchain::Blockchain;
use bdk::blockchain::EsploraBlockchain;
use bdk::database::MemoryDatabase;
use bdk::FeeRate;
use bitcoin::{Amount, Network};
use bitcoin_harness::BitcoindRpcApi;
use std::str::FromStr;
use url::Url;

const BITCOIND_RPC_PORT: u16 = 7041;
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
        tokio::time::sleep(Duration::from_secs(1)).await;
        bitcoind_client
            .generatetoaddress(1, reward_address.clone(), None)
            .await?;
    }
}

#[tokio::main]
async fn setup() {
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
}

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

    let bdk_url = format!("http://localhost:{}", ELECTRUM_HTTP_PORT);

    let blockchain = EsploraBlockchain::new(&bdk_url, Some(2));

    let bdk_wallet = bdk::Wallet::new(
        "wpkh(tprv8ZgxMBicQKsPdpkqS7Eair4YxjcuuvDPNYmKX3sCniCf16tHEVrjjiSXEkFRnUH77yXc6ZcwHHcLNfjdi5qUvw3VDfgYiH5mNsj5izuiu2N/0/1/*)",
        None,
        Network::Regtest,
        MemoryDatabase::default(),
        blockchain).await.unwrap();

    let address = bdk_wallet.get_new_address().unwrap();
    println!("funded address: {}", address);

    // sync wallet
    bdk_wallet.sync(noop_progress(), None).await.unwrap();

    // get some money for our bdk wallet if n balance
    while bdk_wallet.get_balance().unwrap() <= 200_000 {
        miner
            .send_to_address("miner", address.clone(), Amount::from_sat(100_000_000))
            .await
            .unwrap();

        bdk_wallet.sync(noop_progress(), None).await.unwrap();
    }

    let balance = bdk_wallet.get_balance().unwrap();
    println!("balance: {}", balance);

    let address = bdk_wallet.get_new_address().unwrap();
    println!("target address: {}", address);

    let (psbt, _details) = {
        let mut builder = bdk_wallet.build_tx();
        builder
            .add_recipient(address.script_pubkey(), 200_000)
            .fee_rate(FeeRate::from_sat_per_vb(5.0));
        builder.finish().unwrap()
    };

    let (signed_psbt, finalized) = bdk_wallet.sign(psbt, None).unwrap();
    debug_assert!(finalized);

    let txid = bdk_wallet
        .broadcast(signed_psbt.extract_tx())
        .await
        .unwrap();
    println!("txid: {}", txid);

    let block_header_old = bdk_wallet.client().get_height().await.unwrap();
    println!("Current block height: {} ", block_header_old);
    bitcoind_client
        .generatetoaddress(2, miner_address, None)
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_secs(5)).await;
    // update to get the latest 2 blocks
    bdk_wallet.sync(noop_progress(), None).await.unwrap();
    let block_header_new = bdk_wallet.client().get_height().await.unwrap();
    println!("New block height: {} ", block_header_new);
}
