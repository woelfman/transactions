//! Payment processing engine.
use std::collections::HashMap;
use std::path::Path;

use eyre::Result;
use serde::{Deserialize, Serialize, Serializer};

/// Types of Transactions
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Type {
    /// The final state of a dispute and represents the client reversing a transaction.
    Chargeback,
    /// A credit to the client's asset account.
    Deposit,
    /// A client's claim that a transaction was erroneous.
    Dispute,
    /// A resolution to a dispute, releasing the associated held funds.
    Resolve,
    /// A debit to the client's asset account.
    Withdrawal,
}

/// Transaction `Record`s of an individual client.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Client {
    pub records: Vec<Record>,
}

impl Client {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    /// Generate an `Output` summary of the client records.
    pub fn output(&self) -> Output {
        use Type::*;
        let client = self.records[0].client;
        let mut output = Output {
            client,
            ..Default::default()
        };

        for record in &self.records {
            log::trace!("{:?}", record);
            match record.r#type {
                Chargeback => {
                    if let Some(chargeback_record) = self
                        .records
                        .iter()
                        .find(|r| r.tx == record.tx && r.r#type != Chargeback)
                    {
                        let amount = chargeback_record.amount.expect("Missing required amount");
                        output.locked = true;
                        output.held -= amount;
                        output.total -= amount;
                    }
                }
                Deposit => {
                    let amount = record.amount.expect("Missing required amount");
                    output.available += amount;
                    output.total += amount;
                }
                Dispute => {
                    if let Some(dispute_record) = self
                        .records
                        .iter()
                        .find(|r| r.tx == record.tx && r.r#type != Dispute)
                    {
                        let amount = dispute_record.amount.expect("Missing required amount");
                        output.available -= amount;
                        output.held += amount;
                    }
                }
                Resolve => {
                    if let Some(resolve_record) = self
                        .records
                        .iter()
                        .find(|r| r.tx == record.tx && r.r#type != Resolve)
                    {
                        let amount = resolve_record.amount.expect("Missing required amount");
                        output.available += amount;
                        output.held -= amount;
                    }
                }
                Withdrawal => {
                    let amount = record.amount.expect("Missing required amount");
                    if output.available >= amount {
                        output.available -= amount;
                    };
                    if output.total >= amount {
                        output.total -= amount;
                    };
                }
            }
        }

        output
    }
}

/// Transaction record
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Record {
    /// Transaction type
    pub r#type: Type,
    /// Client ID
    pub client: u16,
    /// Transaction ID
    pub tx: u32,
    /// Value of transaction to 4 decimal places of precision
    pub amount: Option<f64>,
}

impl Record {
    /// Parse a CSV file into transaction records
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Vec<Record>> {
        let mut results: Vec<Record> = Vec::new();

        let mut reader = csv::Reader::from_path(path)?;
        for result in reader.deserialize() {
            let record: Record = result?;
            log::trace!("record: {:?}", record);
            results.push(record);
        }

        Ok(results)
    }

    /// Compose transaction records into a `Client` map.
    pub fn process(records: &[Record]) -> Result<HashMap<u16, Client>> {
        let mut clients: HashMap<u16, Client> = HashMap::new();

        for record in records {
            let client = clients.entry(record.client).or_insert_with(Client::new);
            client.records.push(record.clone());
        }

        Ok(clients)
    }
}

/// Account summary
#[derive(Default, Serialize)]
pub struct Output {
    /// Client ID.
    pub client: u16,
    /// The total funds that are available for traiding, staking, withdrawal, etc.
    #[serde(serialize_with = "serialize_funds")]
    pub available: f64,
    /// The total funds that are held for dispute.
    #[serde(serialize_with = "serialize_funds")]
    pub held: f64,
    /// The total funds that are available or held.
    #[serde(serialize_with = "serialize_funds")]
    pub total: f64,
    /// Whether the account is locked.
    pub locked: bool,
}

// Round funds to 4 decimal places of precision.
fn serialize_funds<S>(x: &f64, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_f64((x * 10000.0).round() / 10000.0)
}
