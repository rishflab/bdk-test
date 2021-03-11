use anyhow::Result;
use bdk::electrum_client::ElectrumApi;
use bdk::electrum_client::{GetHistoryRes, HeaderNotification};
use bitcoin::Script;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use url::Url;

pub struct Wallet {
    electrum: bdk::electrum_client::Client,
    latest_block: HeaderNotification,
    script_history: HashMap<Script, Vec<GetHistoryRes>>,
    interval: Duration,
    last_ping: Instant,
}

#[derive(Debug, Copy, Clone)]
pub enum ScriptStatus {
    Unseen,
    InMempool,
    Confirmed { depth: u64 },
}

impl Wallet {
    pub async fn new(url: &Url) -> Result<Self> {
        let electrum = bdk::electrum_client::Client::from_config(
            url.as_str(),
            bdk::electrum_client::ConfigBuilder::new().retry(2).build(),
        )
        .unwrap();
        let latest_block = electrum.block_headers_subscribe().unwrap();

        Ok(Self {
            electrum,
            latest_block,
            script_history: Default::default(),
            interval: Duration::from_secs(5),
            last_ping: Instant::now() - Duration::from_secs(5),
        })
    }

    pub fn get_latest_block_height(&mut self) -> Result<u64> {
        self.ping();
        while let Some(new_block) = self.electrum.block_headers_pop().unwrap() {
            self.latest_block = new_block
        }

        Ok(self.latest_block.height as u64)
    }

    pub fn subscribe_to_script(&self, script: Script) -> Result<()> {
        self.electrum.script_subscribe(&script).unwrap();

        Ok(())
    }

    fn ping(&mut self) {
        if self.last_ping.elapsed() > self.interval {
            self.electrum.ping().unwrap();
            self.last_ping = Instant::now();
        }
    }

    pub fn status_of_script(&mut self, script: Script) -> Result<ScriptStatus> {
        self.ping();
        let blocktip = self.get_latest_block_height()?;

        if std::iter::from_fn(|| self.electrum.script_pop(&script).unwrap())
            .last()
            .is_some()
        {
            let history = self.electrum.script_get_history(&script).unwrap();

            self.script_history.insert(script.clone(), history);
        }

        let history = self.script_history.entry(script).or_default();

        match history.as_slice() {
            [] => Ok(ScriptStatus::Unseen),
            [single, remaining @ ..] => {
                if !remaining.is_empty() {
                    log::warn!("Found more than a single history entry for script. This is highly unexpected and those history entries will be ignored.")
                }

                if single.height <= 0 {
                    Ok(ScriptStatus::InMempool)
                } else {
                    Ok(ScriptStatus::Confirmed {
                        depth: blocktip - (single.height as u64),
                    })
                }
            }
        }
    }

    // pub async fn broadcast(&self, transaction: Transaction) -> Result<Txid> {
    //     let txid = transaction.txid();
    //
    //     self.inner
    //         .lock()
    //         .await
    //         .broadcast(transaction)
    //         .with_context(|| format!("failed to broadcast Bitcoin  transaction: {}", txid))?;
    //
    //     tracing::info!("Published Bitcoin transaction: {}", txid);
    //
    //     Ok(txid)
    // }
    //
    // pub async fn sign_and_finalize(&self, psbt: PartiallySignedTransaction) -> Result<Transaction> {
    //     let (signed_psbt, finalized) = self.inner.lock().await.sign(psbt, None)?;
    //
    //     if !finalized {
    //         bail!("PSBT is not finalized")
    //     }
    //
    //     let tx = signed_psbt.extract_tx();
    //
    //     Ok(tx)
    // }
}
