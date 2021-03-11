use anyhow::Result;
use bdk::electrum_client::ElectrumApi;
use bdk::electrum_client::{GetHistoryRes, HeaderNotification};
use bitcoin::Script;
use std::collections::HashMap;
use url::Url;

pub struct Wallet {
    electrum: bdk::electrum_client::Client,
    latest_block: HeaderNotification,
    script_history: HashMap<Script, Vec<GetHistoryRes>>,
}

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
        })
    }

    pub fn get_latest_block_height(&mut self) -> Result<u64> {
        while let Some(new_block) = self.electrum.block_headers_pop().unwrap() {
            self.latest_block = new_block
        }

        Ok(self.latest_block.height as u64)
    }

    pub fn subscribe_to_script(&self, script: Script) -> Result<()> {
        self.electrum.script_subscribe(&script).unwrap();

        Ok(())
    }

    pub fn status_of_script(&mut self, script: Script) -> Result<ScriptStatus> {
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
                if remaining.len() > 0 {
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
}
