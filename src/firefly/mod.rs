pub mod schema;

use anyhow::{bail, Context, Result};
use chrono::{Datelike, Duration, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime};
use reqwest::{
    blocking::{Client, ClientBuilder, Response},
    header::{HeaderMap, HeaderValue},
};

use crate::{
    config::{BankData, Configuration},
    firefly::schema::{
        TransactionCreate, TransactionType, TransactionUpdate, TransactionsCreate,
        TransactionsUpdate,
    },
};

use self::schema::{Transaction, TransactionsGet};

pub fn get_client(config: &Configuration) -> Result<Client> {
    // Prepare default headers for firefly api.
    let mut headers = HeaderMap::new();
    headers.insert("accept", HeaderValue::from_str("application/vnd.api+json")?);
    headers.insert(
        "Authorization",
        HeaderValue::from_str(&format!("Bearer {}", config.token))?,
    );
    headers.insert("Content-Type", HeaderValue::from_str("application/json")?);

    let client = ClientBuilder::new().default_headers(headers).build()?;

    Ok(client)
}

/// Request all transactions and order them by ascending time.
pub fn get_transactions_for_account(
    config: &Configuration,
    account_id: usize,
) -> Result<Vec<(usize, Transaction)>> {
    let url = format!(
        "http://localhost:8081/api/v1/accounts/{account_id}/transactions\
        ?limit=10000\
        &page=1\
        &type=all"
    );

    let response = get_client(config)?.get(url).send()?.text()?;
    let response: TransactionsGet = serde_yaml::from_str(&response)?;

    // Extract the transaction id and the first transaction split of each transaction.
    let mut transactions: Vec<(usize, Transaction)> = response
        .data
        .into_iter()
        .map(|data| {
            (
                data.id,
                data.attributes.transactions.get(0).cloned().unwrap(),
            )
        })
        .collect();

    // Sort the transactions by date in ascending order.
    transactions.sort_by(|a, b| a.1.date.partial_cmp(&b.1.date).unwrap());

    Ok(transactions)
}

pub fn history_replay(
    config: &Configuration,
    account_id: usize,
    bank_data: &BankData,
    transactions: Vec<(usize, Transaction)>,
) -> Result<()> {
    let first_date = transactions.get(0).unwrap().1.date;

    // Track the current year/month
    // This allows us to detect the end of a month.
    let mut year = first_date.year();
    let mut month = first_date.month();

    // The runner money amount for the historic replay
    let mut money: i64 = 0;
    let mut reconciliation: Option<(usize, Transaction)> = None;

    // Iterate through all transactions.
    // They're ordered by date in ascending order!
    for (id, transaction) in transactions {
        // Check if the next transaction is in the next month.
        // If so, we can wrap up the current month and create a reconciliation if necessary.
        if year != transaction.date.year() || month != transaction.date.month() {
            // Check if there exists an expected bank balance in the bank data for this month.
            let expected_balance = bank_data.get(&year).and_then(|year| year.get(&month));

            // Check if the current value is the same as the expected value.
            // If it isn't, create (or adjust) a reconciliation transaction.
            if let Some(expected) = expected_balance {
                if expected != &money {
                    create_or_update_reconciliation(
                        config,
                        account_id,
                        reconciliation,
                        expected,
                        &mut money,
                        year,
                        month,
                    )?;
                }
            }

            // Notify the user about the new month and start the next month.
            println!(
                "End of month {year}-{month}: Balance {:.2}",
                money as f64 / 100.0
            );
            month = transaction.date.month();
            year = transaction.date.year();
            // Reset the reconciliation
            reconciliation = None;
        }

        // Convert the float to int, since float is always problematic.
        // Round to prevent floating point problems.
        let amount = transaction.amount * 100.0;
        let amount = amount.round() as i64;

        // Add/subtract the amount, based on the transaction type.
        match transaction.transaction_type {
            TransactionType::OpeningBalance => money += amount,
            TransactionType::Deposit => money += amount,
            TransactionType::Withdrawal => money -= amount,
            TransactionType::Transfer => {
                if transaction.destination_id == Some(account_id) {
                    money += amount;
                } else if transaction.source_id == Some(account_id) {
                    money -= amount;
                }
            }
            TransactionType::Reconciliation => (),
        }

        if transaction.description.starts_with("Reconciliation") {
            reconciliation = Some((id, transaction.clone()));
        }
    }

    println!(
        "Current balance {year}-{month}: Balance {:.2}",
        money as f64 / 100.0
    );

    Ok(())
}

/// Built-in Firefly reconciliations sadly cannot represent positive values.
/// They're implicitly negative.
///
/// Since my history has some cases of positive reconciliatons, I have to work around this issue by
/// using simple `Deposit` and `Withdrawal` transactions.
pub fn create_or_update_reconciliation(
    config: &Configuration,
    account_id: usize,
    reconciliation: Option<(usize, Transaction)>,
    expected: &i64,
    money: &mut i64,
    mut year: i32,
    mut month: u32,
) -> Result<()> {
    // The diff between the current and target money.
    // 100 actual -> 150 expected = 50 diff
    let mut diff = expected - *money;
    let mut diff_as_float = diff as f64 / 100.0;

    if let Some((id, reconciliation)) = reconciliation {
        // In case there already exists a reconciliation, update the value of that
        // reconciliation to the new expected value.

        // Calculate the new reconciliation amount.
        let target_value = if reconciliation.transaction_type == TransactionType::Withdrawal {
            -reconciliation.amount + diff_as_float
        } else {
            reconciliation.amount + diff_as_float
        };

        // In case there doesn't exists a reconciliation, create a new one
        println!(
            "Found diff of {:.2} from {:.2} to {:.2}",
            diff_as_float,
            (*money as f64 / 100.0),
            (*expected as f64 / 100.0),
        );

        //println!("{reconciliation:#?}");

        let (transaction_type, _, _) = determine_transaction_props(account_id, target_value);

        // Firefly doesn't allow to convert a withdrawal to a deposit or vice versa.
        // Hence, if the type changes we have to delete the old reconciliation and create a new
        // one.
        // Check the happy path first, as we can just do an early return in that case.
        if transaction_type == reconciliation.transaction_type {
            println!(
                "Updating existing reconciliation '{}' from {:.2} to {:.2}",
                reconciliation.description, reconciliation.amount, target_value
            );

            // Build the update struct for this transaction
            let update = TransactionsUpdate {
                error_if_duplicate_hash: false,
                apply_rules: false,
                transactions: vec![TransactionUpdate {
                    transaction_type,
                    transaction_journal_id: reconciliation.transaction_journal_id,
                    amount: target_value.abs(),
                }],
            };

            let url = format!("{}/api/v1/transactions/{}", config.base_url, id);
            let response = get_client(config)?.put(url).json(&update).send()?;
            check_response(response).context(format!(
                "Failed response while updating reconciliation with payload:\n{update:#?}"
            ))?;

            *money += diff;

            return Ok(());
        }

        println!("Delete existing reconciliation");

        // The type has changed, so we have to delete the old and create a new transaction.
        let url = format!("{}/api/v1/transactions/{}", config.base_url, id);
        let response = get_client(config)?.delete(url).send()?;
        check_response(response).context("Failed response while deleting reconciliation")?;

        // Set the new diff with the incorporated deleted reconciliaton.
        diff_as_float = target_value;
        diff = (diff_as_float * 100.0).round() as i64;
    };

    // In case there doesn't exists a reconciliation, create a new one
    println!(
        "Creating reconciliation with amount of {:.2} from {:.2} to {:.2}",
        diff_as_float,
        (*money as f64 / 100.0),
        (*expected as f64 / 100.0),
    );

    let description = format!("Reconciliation {year} - {month}");
    // Calculate the next month, so we can get the last day of the current month.
    if month == 12 {
        month = 1;
        year += 1;
    } else {
        month += 1;
    }

    // Determine the transaction type based on whether money is added or removed.
    let (transaction_type, source_id, destination_id) =
        determine_transaction_props(account_id, diff_as_float);

    // Get the last day of the month
    let date = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let time = NaiveTime::from_hms_milli_opt(23, 59, 59, 999).unwrap();
    let datetime = NaiveDateTime::new(date, time) - Duration::days(1);
    let datetime = datetime
        .and_local_timezone(FixedOffset::east_opt(3600 * 2).unwrap())
        .unwrap();

    // Build the update struct for this transaction
    let create = TransactionsCreate {
        error_if_duplicate_hash: false,
        apply_rules: true,
        transactions: vec![TransactionCreate {
            transaction_type,
            description,
            date: datetime,
            amount: diff_as_float.abs(),
            source_id,
            destination_id,
        }],
    };

    let url = format!("{}/api/v1/transactions", config.base_url);
    let response = get_client(config)?.post(url).json(&create).send()?;
    check_response(response).context(format!(
        "Failed response while creating reconciliation with payload:\n{create:#?}"
    ))?;

    *money += diff;

    Ok(())
}

pub fn check_response(response: Response) -> Result<()> {
    let status = response.status().as_u16();
    if !(200..=299).contains(&status) {
        bail!("Got status {status} with payload:\n{}", response.text()?);
    }

    Ok(())
}

/// Return the type, source and target for a reconciliation transaction based on a given value.
pub fn determine_transaction_props(
    account_id: usize,
    value: f64,
) -> (TransactionType, Option<usize>, Option<usize>) {
    if value.is_sign_negative() {
        (TransactionType::Withdrawal, Some(account_id), Some(1))
    } else {
        (TransactionType::Deposit, Some(1), Some(account_id))
    }
}
