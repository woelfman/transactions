//! Integration tests
use assert_fs::prelude::*;
use transactions::{Client, ClientId, Record};

use std::collections::HashMap;

#[test]
fn from_path() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("transactions.csv")?;
    file.write_str(
        r#"type,client,tx,amount
deposit,1,1,1.0
deposit,2,2,2.0
deposit,1,3,2.0
withdrawal,1,4,1.5
withdrawal,2,5,3.0
"#,
    )?;

    let records = transactions::Record::from_path(file.path())?;
    let expected_records = vec![
        Record::new(transactions::Type::Deposit, 1, 1, Some(1.0)),
        Record::new(transactions::Type::Deposit, 2, 2, Some(2.0)),
        Record::new(transactions::Type::Deposit, 1, 3, Some(2.0)),
        Record::new(transactions::Type::Withdrawal, 1, 4, Some(1.5)),
        Record::new(transactions::Type::Withdrawal, 2, 5, Some(3.0)),
    ];
    assert_eq!(records, expected_records);

    Ok(())
}

#[test]
fn process() -> Result<(), Box<dyn std::error::Error>> {
    let records: Vec<Record> = vec![
        Record::new(transactions::Type::Deposit, 1, 1, Some(1.0)),
        Record::new(transactions::Type::Deposit, 2, 2, Some(2.0)),
        Record::new(transactions::Type::Deposit, 1, 3, Some(2.0)),
        Record::new(transactions::Type::Withdrawal, 1, 4, Some(1.5)),
        Record::new(transactions::Type::Withdrawal, 2, 5, Some(3.0)),
        Record::new(transactions::Type::Withdrawal, 2, 6, Some(-3.0)),
    ];
    let expected_result: HashMap<ClientId, Client> = [
        (
            1,
            transactions::Client {
                client: 1,
                available: 1.5,
                held: 0.0,
                total: 1.5,
                locked: false,
            },
        ),
        (
            2,
            transactions::Client {
                client: 2,
                available: 2.0,
                held: 0.0,
                total: 2.0,
                locked: false,
            },
        ),
    ]
    .iter()
    .cloned()
    .collect();

    let result = transactions::Record::process(&records)?;
    assert_eq!(result, expected_result);

    Ok(())
}
