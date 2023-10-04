pub mod get;

use anyhow::Result;
use reqwest::blocking::Client;

use crate::firefly::get::TransactionsResponse;

/// Request all transactions and order them by ascending time.
pub fn get_transactions_for_account(
    client: &Client,
    account_id: usize,
) -> Result<TransactionsResponse> {
    let url = format!(
        "http://localhost:8081/api/v1/accounts/{account_id}/transactions\
        ?limit=50\
        &page=1\
        &type=all"
    );

    let response = client.get(url).send()?.text()?;
    let response: TransactionsResponse = serde_yaml::from_str(&response)?;

    Ok(response)
}
