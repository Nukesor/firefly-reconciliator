pub mod get;

use anyhow::Result;
use chrono::Datelike;
use reqwest::blocking::Client;

use crate::{config::BankData, firefly::get::TransactionType};

use self::get::{Transaction, TransactionsResponse};

/// Request all transactions and order them by ascending time.
pub fn get_transactions_for_account(
    client: &Client,
    account_id: usize,
) -> Result<Vec<Transaction>> {
    let url = format!(
        "http://localhost:8081/api/v1/accounts/{account_id}/transactions\
        ?limit=10000\
        &page=1\
        &type=all"
    );

    let response = client.get(url).send()?.text()?;
    let response: TransactionsResponse = serde_yaml::from_str(&response)?;

    // Extract the first transaction split of each transaction
    let mut transactions: Vec<Transaction> = response
        .data
        .into_iter()
        .map(|data| data.attributes.transactions.get(0).cloned().unwrap())
        .collect();

    // Sort the transactions by date in ascending order.
    transactions.sort_by(|a, b| a.date.partial_cmp(&b.date).unwrap());

    Ok(transactions)
}

pub fn history_replay(
    account_id: usize,
    bank_data: BankData,
    transactions: Vec<Transaction>,
) -> Result<()> {
    let first_date = transactions.get(0).unwrap().date;

    // Track the current year/month
    // This allows us to detect the end of a month.
    let mut year = first_date.year();
    let mut month = first_date.month();

    // The runner money amount for the historic replay
    let mut money: i64 = 0;

    for transaction in transactions {
        if year != transaction.date.year() || month != transaction.date.month() {
            let expected = bank_data.get(&year).map(|year| year.get(&month)).flatten();

            // We have an expected value for this account and the current year-month combination.
            // Check if the current value is the same as the expected value.
            // If it isn't, create (or adjust) a reconciliation transaction.
            if let Some(expected) = expected {
                if expected != &money {
                    let diff = expected - money;

                    println!(
                        "Creating reconciliation with amount of {:.2} from {:.2} to {:.2}",
                        (diff as f64 / 100.0),
                        (money as f64 / 100.0),
                        (*expected as f64 / 100.0),
                    );
                    money += diff;
                }
            }

            println!(
                "End of month {year}-{month}: Balance {:.2}",
                money as f64 / 100.0
            );
            month = transaction.date.month();
            year = transaction.date.year();
        }

        // Convert the float to int, since float is always problematic.
        // Round to prevent floating point problems.
        let amount = transaction.amount * 100.0;
        let amount = amount.round() as i64;

        // Add/subtract the amount, based on the transaction type.
        match transaction.transaction_type {
            TransactionType::Withdrawal | TransactionType::Reconciliation => money -= amount,
            TransactionType::Deposit | TransactionType::OpeningBalance => money += amount,
            TransactionType::Transfer => {
                if transaction.destination_id == Some(account_id) {
                    money += amount
                } else if transaction.source_id == Some(account_id) {
                    money -= amount
                }
            }
        }
    }

    println!(
        "Current balance {year}-{month}: Balance {:.2}",
        money as f64 / 100.0
    );

    Ok(())
}
