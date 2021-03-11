use anyhow::Context;
use anyhow::Result;
use bdk::{
    bitcoin::Network,
    blockchain::{noop_progress, ElectrumBlockchain},
    database::MemoryDatabase,
    electrum_client::ElectrumApi,
};
use bitcoin::{Address, Amount};
use reqwest::Method;
use std::sync::Arc;
use tokio::sync::Mutex;
use url::Url;

pub struct Wallet {
    inner: Arc<Mutex<bdk::Wallet<ElectrumBlockchain, MemoryDatabase>>>,
}

impl Wallet {
    pub async fn new(url: &Url) -> Result<Self> {
        let client = bdk::electrum_client::Client::new(url.as_str()).unwrap();

        let header_notif = client.block_headers_subscribe().unwrap();
        dbg!(header_notif);

        let blockchain = ElectrumBlockchain::from(client);

        let bdk_wallet = bdk::Wallet::new(
            "wpkh(tprv8ZgxMBicQKsPdpkqS7Eair4YxjcuuvDPNYmKX3sCniCf16tHEVrjjiSXEkFRnUH77yXc6ZcwHHcLNfjdi5qUvw3VDfgYiH5mNsj5izuiu2N/0/0/*)",
            None,
            Network::Regtest,
            MemoryDatabase::default(),
            blockchain).unwrap();

        Ok(Self {
            inner: Arc::new(Mutex::new(bdk_wallet)),
        })
    }

    pub async fn balance(&self) -> Result<Amount> {
        let balance = self
            .inner
            .lock()
            .await
            .get_balance()
            .context("Failed to calculate Bitcoin balance")?;

        Ok(Amount::from_sat(balance))
    }

    pub async fn get_block_height(&self, electrum_http_url: Url) -> String {
        let client = reqwest::Client::new();
        let resp = client
            .request(Method::GET, electrum_http_url)
            .send()
            .await
            .unwrap();
        let height = resp.text().await.unwrap();

        height
    }

    pub async fn new_address(&self) -> Result<Address> {
        let address = self
            .inner
            .lock()
            .await
            .get_new_address()
            .context("Failed to get new Bitcoin address")?;

        Ok(address)
    }

    pub async fn sync(&self) -> Result<()> {
        self.inner
            .lock()
            .await
            .sync(noop_progress(), None)
            .context("Failed to sync balance of Bitcoin wallet")?;

        Ok(())
    }
}
