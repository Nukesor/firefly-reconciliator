use chrono::{DateTime, Utc};
use serde_aux::prelude::*;
use serde_derive::{Deserialize, Serialize};

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct TransactionsResponse {
    pub data: Vec<Data>,
}

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct Data {
    #[serde(rename = "type")]
    trans_type: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    id: usize,
    attributes: TransactionAttributes,
}

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct TransactionAttributes {
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    user: usize,
    transactions: Vec<Transaction>,
}

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct Transaction {
    #[serde(rename = "type")]
    transaction_type: TransactionType,
    description: String,
    date: DateTime<Utc>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    amount: f64,
    #[serde(deserialize_with = "deserialize_option_number_from_string")]
    source_id: Option<usize>,
    #[serde(deserialize_with = "deserialize_option_number_from_string")]
    destination_id: Option<usize>,
}

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub enum TransactionType {
    #[serde(rename = "withdrawal")]
    Withdrawal,
    #[serde(rename = "deposit")]
    Deposit,
    #[serde(rename = "transfer")]
    Transfer,
    #[serde(rename = "reconciliation")]
    Reconciliation,
    #[serde(rename = "opening balance")]
    OpeningBalance,
}
