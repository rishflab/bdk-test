use anyhow::Result;
use bdk_test::Wallet;
use url::Url;

const ELECTRUM_RPC_PORT: u16 = 60401;

#[tokio::main]
async fn main() -> Result<()> {
    // note: you need to generate 101 blocks first
    // note2: miner wallet name is `miner`
    // run setup function above ;)

    let bdk_url = {
        let input = format!("tcp://@localhost:{}", ELECTRUM_RPC_PORT);
        Url::parse(&input).unwrap()
    };

    let bdk_wallet = Wallet::new(&bdk_url).await?;

    bdk_wallet.sync().await.unwrap();

    Ok(())
}
