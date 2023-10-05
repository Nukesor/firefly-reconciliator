use chrono::{DateTime, FixedOffset};
use serde_aux::prelude::*;
use serde_derive::{Deserialize, Serialize};

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

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct TransactionsGet {
    pub data: Vec<TransactionsGetData>,
}

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct TransactionsGetData {
    #[serde(rename = "type")]
    pub trans_type: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub id: usize,
    pub attributes: TransactionAttributes,
}

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct TransactionAttributes {
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub user: usize,
    pub transactions: Vec<Transaction>,
}

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct Transaction {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub transaction_journal_id: usize,
    #[serde(rename = "type")]
    pub transaction_type: TransactionType,
    pub description: String,
    pub date: DateTime<FixedOffset>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub amount: f64,
    #[serde(deserialize_with = "deserialize_option_number_from_string")]
    pub source_id: Option<usize>,
    #[serde(deserialize_with = "deserialize_option_number_from_string")]
    pub destination_id: Option<usize>,
}

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct TransactionsCreate {
    pub error_if_duplicate_hash: bool,
    pub apply_rules: bool,
    pub transactions: Vec<TransactionCreate>,
}

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct TransactionCreate {
    #[serde(rename = "type")]
    pub transaction_type: TransactionType,
    pub description: String,
    pub date: DateTime<FixedOffset>,
    pub amount: f64,
    pub source_id: Option<usize>,
    pub destination_id: Option<usize>,
}

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct TransactionsUpdate {
    pub error_if_duplicate_hash: bool,
    pub apply_rules: bool,
    pub transactions: Vec<TransactionUpdate>,
}

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct TransactionUpdate {
    #[serde(rename = "type")]
    pub transaction_type: TransactionType,
    pub transaction_journal_id: usize,
    pub amount: f64,
}
