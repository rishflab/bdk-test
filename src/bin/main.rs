use anyhow::Result;
use bdk_test::Wallet;
use std::time::Duration;
use url::Url;

const ELECTRUM_RPC_PORT: u16 = 60401;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    // note: you need to generate 101 blocks first
    // note2: miner wallet name is `miner`
    // run setup function above ;)

    let bdk_url = {
        let input = format!("tcp://@localhost:{}", ELECTRUM_RPC_PORT);
        Url::parse(&input).unwrap()
    };

    let mut bdk_wallet = Wallet::new(&bdk_url).await?;

    loop {
        tokio::time::sleep(Duration::from_millis(500)).await;

        let latest_block_height = bdk_wallet.get_latest_block_height()?;

        log::info!("Latest block height is {}", latest_block_height);
    }
}
