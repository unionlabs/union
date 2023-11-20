use std::collections::HashMap;

use csv::{ReaderBuilder, WriterBuilder};

#[derive(Debug, serde::Deserialize, Eq, PartialEq, Clone)]
struct Transaction {
    uuid: String,
    sender: String,
    execution_timestamp: u64,
    finalization_timestamp: u64,
    transaction_state: TransactionState,
    chain_id: String,
}

#[derive(Debug, serde::Deserialize, Eq, PartialEq, Clone)]
enum TransactionState {
    SentFrom,
    ReceivedOn,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Clone)]
pub struct TransactionReport {
    pub uuid: String,
    pub src: String,
    pub executed_at: Option<u64>,
    pub execution_duration: Option<u64>,
    pub finalized_at: Option<u64>,
    pub finalization_duration: Option<u64>,
}

pub fn process(input_file_path: String) -> Vec<TransactionReport> {
    let mut reader = ReaderBuilder::new()
        .has_headers(false)
        .from_path(input_file_path)
        .unwrap();
    let iter = reader.deserialize();

    let mut transactions: HashMap<String, (Transaction, Option<Transaction>)> = HashMap::new();

    for res in iter {
        let rec: Transaction = res.unwrap();
        match rec.transaction_state {
            TransactionState::SentFrom => {
                transactions.insert(rec.uuid.clone(), (rec, None));
            }
            TransactionState::ReceivedOn => match &transactions.get(&rec.uuid) {
                Some((sent_from, _)) => {
                    transactions.insert(rec.uuid.clone(), (sent_from.clone(), Some(rec)));
                }
                None => {
                    tracing::warn!(
                        "Processed `ReceivedOn` packet without matching `SentFrom` packet."
                    );
                    continue;
                }
            },
        }
    }

    let reports: Vec<TransactionReport> = transactions
        .into_values()
        .map(|(sent_from, received_on_maybe)| match received_on_maybe {
            Some(received_on) => TransactionReport {
                uuid: sent_from.uuid,
                src: sent_from.chain_id,
                executed_at: Some(received_on.execution_timestamp),
                execution_duration: Some(
                    received_on.execution_timestamp - sent_from.execution_timestamp,
                ),
                finalized_at: Some(received_on.finalization_timestamp),
                finalization_duration: Some(
                    received_on.finalization_timestamp - sent_from.execution_timestamp,
                ),
            },
            None => TransactionReport {
                uuid: sent_from.uuid,
                src: sent_from.chain_id,
                executed_at: None,
                execution_duration: None,
                finalized_at: None,
                finalization_duration: None,
            },
        })
        .collect();

    let mut writer = WriterBuilder::new()
        .has_headers(false)
        .from_path("zerg-report.csv")
        .unwrap();

    reports.iter().for_each(|report| {
        writer.serialize(report).unwrap();
    });

    reports
}
