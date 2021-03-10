use std::time::Duration;
use ::bitcoin::{Txid, util::psbt::PartiallySignedTransaction};
use anyhow::Result;
use bdk::{
    bitcoin::Network,
    blockchain::{ElectrumBlockchain, noop_progress},
    database::MemoryDatabase,
    electrum_client::{Client, ElectrumApi},
    FeeRate,
};
use bdk::blockchain::{Blockchain};
use bitcoin::{Address, Amount, Transaction};
use reqwest::Method;
use tokio::time::interval;
use url::Url;

pub struct BdkWallet {
    pub inner: bdk::Wallet<ElectrumBlockchain, MemoryDatabase>,
    //pub client: Client,
}

impl BdkWallet {
    // pub async fn new_alice( url: &Url, network: Network) -> Result<Self> {
    //     match Client::new(url.as_str()) {
    //         Ok(client) => {
    //             let bdk_wallet = bdk::Wallet::new(
    //                 "wpkh([c258d2e4/84h/1h/0h]tpubDDYkZojQFQjht8Tm4jsS3iuEmKjTiEGjG6KnuFNKKJb5A6ZUCUZKdvLdSDWofKi4ToRCwb9poe1XdqfUnP4jaJjCB2Zwv11ZLgSbnZSNecE/0/*)",
    //                 Some("wpkh([c258d2e4/84h/1h/0h]tpubDDYkZojQFQjht8Tm4jsS3iuEmKjTiEGjG6KnuFNKKJb5A6ZUCUZKdvLdSDWofKi4ToRCwb9poe1XdqfUnP4jaJjCB2Zwv11ZLgSbnZSNecE/1/*)"),
    //                 bitcoin::Network::Regtest,
    //                 MemoryDatabase::default(),
    //                 ElectrumBlockchain::from(client)
    //             )?;
    //
    //             Ok(Self {
    //                 inner: bdk_wallet,
    //                 //client: client2,
    //             })
    //         }
    //         Err(err) => {
    //             println!("{:?}", err);
    //             Err(anyhow::Error::msg("could not init bdk wallet"))
    //         },
    //     }
    // }

    // pub async fn new_bob(client: Client, network: Network) -> Result<Self> {
    //
    //
    //     let bdk_wallet = bdk::Wallet::new(
    //         "wpkh([c258d2e4/84h/1h/0h]tpubDDYkZojQFQjht8Tm4jsS3iuEmKjTiEGjG6KnuFNKKJb5A6ZUCUZKdvLdSDWofKi4ToRCwb9poe1XdqfUnP4jaJjCB2Zwv11ZLgSbnZSNecE/0/*)",
    //         Some("wpkh([c258d2e4/84h/1h/0h]tpubDDYkZojQFQjht8Tm4jsS3iuEmKjTiEGjG6KnuFNKKJb5A6ZUCUZKdvLdSDWofKi4ToRCwb9poe1XdqfUnP4jaJjCB2Zwv11ZLgSbnZSNecE/1/*)"),
    //         network,
    //         MemoryDatabase::default(),
    //         ElectrumBlockchain::from(client)
    //     )?;
    //
    //     Ok(Self {
    //         inner: bdk_wallet,
    //         //client: client2,
    //     })
    //
    //
    // }



    pub fn get_network(&self) -> bitcoin::Network {
        self.inner.network()
    }
}

pub async fn build_tx_lock_psbt(
    wallet: &bdk::Wallet<ElectrumBlockchain, MemoryDatabase>,
    output_address: Address,
    output_amount: Amount,
) -> Result<PartiallySignedTransaction> {

    // let (psbt, _details) = wallet.create_tx(
    //     bdk::TxBuilder::with_recipients(vec![(
    //         output_address.script_pubkey(),
    //         output_amount.as_sat()
    //     )])
    //         // .enable_rbf()
    //         // .do_not_spend_change()
    //         // .fee_rate(FeeRate::from_sat_per_vb(5.0)),
    // )?;


    let (psbt, _details) = wallet.create_tx(
        bdk::TxBuilder::with_recipients(vec![(output_address.script_pubkey(), output_amount.as_sat())])
            .enable_rbf()
            .do_not_spend_change()
            .fee_rate(FeeRate::from_sat_per_vb(5.0))
    )?;


    Ok(psbt)
}

pub async fn get_block_height(electrum_http_url: Url) -> String {

    let client = reqwest::Client::new();
    let resp = client.request(Method::GET, electrum_http_url).send().await.unwrap();
    let height = resp.text().await.unwrap();

    height
}

// async fn sign_tx_lock( wallet: &bdk::Wallet<ElectrumBlockchain, MemoryDatabase>, tx_lock: TxLock) -> Result<Transaction> {
//     let psbt = PartiallySignedTransaction::from(tx_lock);
//
//     let (signed_psbt, _finalized) = wallet.sign(psbt, None)?;
//
//     let tx = signed_psbt.extract_tx();
//
//     Ok(tx)
// }


// #[async_trait]
// impl BroadcastSignedTransaction for BdkWallet {
//     async fn broadcast_signed_transaction(&self, transaction: Transaction) -> Result<Txid> {
//         let txid = self.inner.broadcast(transaction)?;
//         tracing::info!("Bitcoin tx broadcasted! TXID = {}", txid);
//         Ok(txid)
//     }
// }
//
// // TODO: For retry, use `backoff::ExponentialBackoff` in production as opposed
// // to `ConstantBackoff`.
// #[async_trait]
// impl WatchForRawTransaction for BdkWallet {
//     async fn watch_for_raw_transaction(&self, txid: Txid) -> Transaction {
//         (|| async { Ok(self.get_raw_transaction(txid).await?) })
//             .retry(ConstantBackoff::new(Duration::from_secs(1)))
//             .await
//             .expect("transient errors to be retried")
//     }
// }
//
// #[async_trait]
// impl GetRawTransaction for BdkWallet {
//     // todo: potentially replace with option
//     async fn get_raw_transaction(&self, txid: Txid) -> Result<Transaction> {
//         let tx = self.inner.client().unwrap().get_tx(&txid)?;
//         Ok(tx.unwrap())
//     }
// }
//


// #[async_trait]
// impl TransactionBlockHeight for BdkWallet {
//     async fn transaction_block_height(&self, txid: Txid) -> BlockHeight {
//         let merkle_res = self.client.block_header();
//         let height = merkle_res.block_height;
//
//         BlockHeight::new(height as u32)
//     }
// }
//
// #[async_trait]
// impl WaitForTransactionFinality for BdkWallet {
//     async fn wait_for_transaction_finality(&self, txid: Txid, config: Config) -> Result<()> {
//         // TODO(Franck): This assumes that bitcoind runs with txindex=1
//
//         // Divide by 4 to not check too often yet still be aware of the new block early
//         // on.
//         let mut interval = interval(config.bitcoin_avg_block_time / 4);
//
//         loop {
//             let tx = self.get_raw_transaction(txid).await?;
//             if let Some(confirmations) = tx.confirmations {
//                 if confirmations >= config.bitcoin_finality_confirmations {
//                     break;
//                 }
//             }
//             interval.tick().await;
//         }
//
//         Ok(())
//     }
// }
//
