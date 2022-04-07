//! Integration tests
use assert_fs::prelude::*;

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
        transactions::Record {
            r#type: transactions::Type::Deposit,
            client: 1,
            tx: 1,
            amount: Some(1.0),
        },
        transactions::Record {
            r#type: transactions::Type::Deposit,
            client: 2,
            tx: 2,
            amount: Some(2.0),
        },
        transactions::Record {
            r#type: transactions::Type::Deposit,
            client: 1,
            tx: 3,
            amount: Some(2.0),
        },
        transactions::Record {
            r#type: transactions::Type::Withdrawal,
            client: 1,
            tx: 4,
            amount: Some(1.5),
        },
        transactions::Record {
            r#type: transactions::Type::Withdrawal,
            client: 2,
            tx: 5,
            amount: Some(3.0),
        },
    ];
    assert_eq!(records, expected_records);

    Ok(())
}

#[test]
fn process() -> Result<(), Box<dyn std::error::Error>> {
    let records: Vec<transactions::Record> = vec![
        transactions::Record {
            r#type: transactions::Type::Deposit,
            client: 1,
            tx: 1,
            amount: Some(1.0),
        },
        transactions::Record {
            r#type: transactions::Type::Deposit,
            client: 2,
            tx: 2,
            amount: Some(2.0),
        },
        transactions::Record {
            r#type: transactions::Type::Deposit,
            client: 1,
            tx: 3,
            amount: Some(2.0),
        },
        transactions::Record {
            r#type: transactions::Type::Withdrawal,
            client: 1,
            tx: 4,
            amount: Some(1.5),
        },
        transactions::Record {
            r#type: transactions::Type::Withdrawal,
            client: 2,
            tx: 5,
            amount: Some(3.0),
        },
    ];
    let expected_result: HashMap<u16, transactions::Client> = [
        (
            1,
            transactions::Client {
                records: vec![
                    transactions::Record {
                        r#type: transactions::Type::Deposit,
                        client: 1,
                        tx: 1,
                        amount: Some(1.0),
                    },
                    transactions::Record {
                        r#type: transactions::Type::Deposit,
                        client: 1,
                        tx: 3,
                        amount: Some(2.0),
                    },
                    transactions::Record {
                        r#type: transactions::Type::Withdrawal,
                        client: 1,
                        tx: 4,
                        amount: Some(1.5),
                    },
                ],
            },
        ),
        (
            2,
            transactions::Client {
                records: vec![
                    transactions::Record {
                        r#type: transactions::Type::Deposit,
                        client: 2,
                        tx: 2,
                        amount: Some(2.0),
                    },
                    transactions::Record {
                        r#type: transactions::Type::Withdrawal,
                        client: 2,
                        tx: 5,
                        amount: Some(3.0),
                    },
                ],
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
