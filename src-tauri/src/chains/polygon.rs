use super::ethereum::EthereumAdapter;
use super::types::*;
use super::ChainId;

#[derive(Debug)]
pub struct PolygonAdapter {
    inner: EthereumAdapter,
}

impl PolygonAdapter {
    pub fn new(rpc_url: String) -> Self {
        Self {
            inner: EthereumAdapter::new(rpc_url, "Polygon", "MATIC"),
        }
    }
}

#[async_trait::async_trait]
impl ChainAdapter for PolygonAdapter {
    async fn get_balance(&self, wallet: &WalletInfo) -> Result<ChainBalance, String> {
        let mut balance = self.inner.get_balance(wallet).await?;
        balance.total_usd_value = balance.native_balance * 1.2;
        Ok(balance)
    }

    async fn get_fee_estimate(&self, wallet: &WalletInfo) -> Result<ChainFeeEstimate, String> {
        let mut estimate = self.inner.get_fee_estimate(wallet).await?;
        estimate.max_fee *= 0.01;
        estimate.avg_fee *= 0.01;
        estimate.fee_currency = "MATIC".to_string();
        estimate.estimated_time_seconds = 2;
        Ok(estimate)
    }

    async fn build_transfer(
        &self,
        wallet: &WalletInfo,
        to: &str,
        amount: f64,
    ) -> Result<ChainTransaction, String> {
        let mut tx = self.inner.build_transfer(wallet, to, amount).await?;
        tx.chain_id = ChainId::Polygon;
        Ok(tx)
    }

    async fn submit_transaction(&self, tx: ChainTransaction) -> Result<String, String> {
        self.inner.submit_transaction(tx).await
    }

    async fn get_status(&self) -> Result<ChainStatus, String> {
        let mut status = self.inner.get_status().await?;
        status.chain_id = ChainId::Polygon;
        Ok(status)
    }
}
