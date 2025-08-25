use serde::{Deserialize, Serialize};
use solana_transaction_status_client_types::EncodedTransactionWithStatusMeta;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FetchTransactionParam {
    pub signature: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FetchTransactionsNearByParam {
    pub signature: String,
    pub backward: Option<u32>,
    pub forward: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct BlockTransaction {
    pub slot: u64,
    pub block_time: Option<i64>,
    pub block_height: Option<u64>,
    pub transactions: Option<Vec<EncodedTransactionWithStatusMeta>>,
}

#[cfg(feature = "client")]
pub mod client {
    use reqwest::Method;
    use solana_transaction_status_client_types::EncodedConfirmedTransactionWithStatusMeta;
    use crate::api::api_error::ApiError;
    use crate::api::client::Client;
    use crate::api::transaction::{BlockTransaction, FetchTransactionParam, FetchTransactionsNearByParam};

    pub async fn fetch_transaction(
        params: FetchTransactionParam,
    ) -> Result<Option<EncodedConfirmedTransactionWithStatusMeta>, ApiError> {
        Client::request(Method::POST, "/transaction/fetch", params).await
    }

    pub async fn fetch_transaction_near_by(
        params: FetchTransactionsNearByParam,
    ) -> Result<Option<Vec<BlockTransaction>>, ApiError> {
        Client::request(Method::POST, "/transaction/fetch_near_by", params).await
    }
}

