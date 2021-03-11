use anyhow::Result;
use bdk::bitcoin::hashes::hex::FromHex;
use bdk::bitcoin::{Address, Amount, Script};
use bdk_test::Wallet;
use std::str::FromStr;
use std::time::Duration;
use url::Url;

const ELECTRUM_RPC_PORT: u16 = 60401;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let bdk_url = {
        let input = format!("tcp://@localhost:{}", ELECTRUM_RPC_PORT);
        Url::parse(&input).unwrap()
    };

    let mut bdk_wallet = Wallet::new(&bdk_url).await?;

    for _ in 0..1 {
        tokio::time::sleep(Duration::from_millis(500)).await;

        let latest_block_height = bdk_wallet.get_latest_block_height()?;

        log::info!("Latest block height is {}", latest_block_height);
    }

    //let signed = bdk_wallet.sign_and_finalize(tx_lock.into()).await?;
    //let script = signed.input.first().unwrap().script_sig.clone();
    let address = Address::from_str("bcrt1qmlapatq6h97hyxmea7wcrg5k6jxnjhg7g2ca5q")?;
    let script = address.script_pubkey();

    bdk_wallet.subscribe_to_script(script.clone()).unwrap();

    loop {
        tokio::time::sleep(Duration::from_millis(500)).await;
        dbg!(bdk_wallet.status_of_script(script.clone())?);
    }
    //let txid = bdk_wallet.broadcast(signed).await?;

    Ok(())
}
