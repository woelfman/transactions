//! Payment processing engine.
use std::collections::HashMap;
use std::path::Path;

use eyre::Result;
use serde::{Deserialize, Serialize, Serializer};

pub type ClientId = u16;
pub type TransactionId = u32;

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

/// Transaction record
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Record {
    /// Transaction type
    pub r#type: Type,
    /// Client ID
    pub client: ClientId,
    /// Transaction ID
    pub tx: TransactionId,
    /// Value of transaction to 4 decimal places of precision
    pub amount: Option<f64>,
    /// Flag a transaction as disputed
    #[serde(skip)]
    disputed: bool,
}

impl Record {
    pub fn new(r#type: Type, client: ClientId, tx: TransactionId, amount: Option<f64>) -> Self {
        Self {
            r#type,
            client,
            tx,
            amount,
            disputed: false,
        }
    }

    /// Parse a CSV file into transaction records
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Vec<Record>> {
        let mut results: Vec<Record> = Vec::new();

        let mut reader = csv::ReaderBuilder::new()
            .flexible(true)
            .trim(csv::Trim::All)
            .from_path(path)?;
        for result in reader.deserialize() {
            let record: Record = result?;
            log::trace!("record: {:?}", record);
            results.push(record);
        }

        Ok(results)
    }

    /// Compose transaction records into a `Client` map.
    pub fn process(raw_records: &[Record]) -> Result<HashMap<ClientId, Client>> {
        let mut clients: HashMap<ClientId, Client> = HashMap::new();
        let mut records: HashMap<TransactionId, Record> = HashMap::new();

        for record in raw_records {
            process_record(record.clone(), &mut clients, &mut records)?;
        }

        Ok(clients)
    }
}

// Process a transaction `record` into the `clients` hashmap.
//
// # Arguments
//
// * `record`: The transaction record to add to the `clients` map
// * `clients`: Map of client transaction records
// * `records`: Map of raw input records
fn process_record(
    record: Record,
    clients: &mut HashMap<ClientId, Client>,
    records: &mut HashMap<TransactionId, Record>,
) -> Result<()> {
    use Type::*;
    let client = clients
        .entry(record.client)
        .or_insert_with(|| Client::new(record.client));

    if client.locked {
        return Ok(());
    }

    log::debug!("{:?}", record);
    match record.r#type {
        Chargeback => {
            if let Some(chargeback_record) = records.get(&record.tx) {
                if let Some(amount) = chargeback_record.amount {
                    if chargeback_record.client == client.client
                        && chargeback_record.disputed
                        && client.held >= amount
                    {
                        client.locked = true;
                        client.held -= amount;
                    } else {
                        log::warn!("Invalid chargeback: {:?}", chargeback_record);
                    }
                }
            } else {
                log::warn!("Failed to find chargeback record: {}", record.tx);
            }
        }
        Deposit => {
            if let Some(amount) = record.amount {
                if !records.contains_key(&record.tx) && amount > 0.0 {
                    client.available += amount;
                    records.insert(record.tx, record);
                }
            }
        }
        Dispute => {
            if let Some(dispute_record) = records.get_mut(&record.tx) {
                if let Some(amount) = dispute_record.amount {
                    if dispute_record.client == client.client
                        && !dispute_record.disputed
                        && client.available >= amount
                    {
                        client.available -= amount;
                        client.held += amount;
                        dispute_record.disputed = true;
                    }
                }
            }
        }
        Resolve => {
            if let Some(resolve_record) = records.get_mut(&record.tx) {
                if let Some(amount) = resolve_record.amount {
                    if resolve_record.disputed && client.held >= amount {
                        client.available += amount;
                        client.held -= amount;
                        resolve_record.disputed = false;
                    }
                }
            }
        }
        Withdrawal => {
            if let Some(amount) = record.amount {
                if client.available >= amount && amount > 0.0 {
                    client.available -= amount;
                };
            }
        }
    }

    client.total = client.available + client.held;

    Ok(())
}

/// Account summary
#[derive(Default, Clone, Debug, Serialize, PartialEq)]
pub struct Client {
    /// Client ID.
    pub client: ClientId,
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

impl Client {
    pub fn new(client: ClientId) -> Self {
        Self {
            client,
            ..Default::default()
        }
    }
}

// Round funds to 4 decimal places of precision.
fn serialize_funds<S>(x: &f64, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_f64((x * 10000.0).round() / 10000.0)
}
