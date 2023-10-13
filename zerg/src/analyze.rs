use std::collections::HashMap;

use csv::{ReaderBuilder, WriterBuilder};

#[derive(Debug, serde::Deserialize, Eq, PartialEq, Clone)]
struct Transaction {
    uuid: String,
    sender: String,
    timestamp: u64,
    transaction_state: TransactionState,
    chain_id: String,
}

#[derive(Debug, serde::Deserialize, Eq, PartialEq, Clone)]
enum TransactionState {
    SentFrom,
    ReceivedOn,
}

#[derive(Debug, serde::Serialize, PartialEq, Clone)]
struct TransactionReport {
    uuid: String,
    completed: bool,
    arrived_on: Option<u64>,
    duration: Option<u64>,
}

pub fn analyze(input_file_path: String) {
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
                completed: true,
                arrived_on: Some(received_on.timestamp),
                duration: Some(received_on.timestamp - sent_from.timestamp),
            },
            None => TransactionReport {
                uuid: sent_from.uuid,
                completed: false,
                arrived_on: None,
                duration: None,
            },
        })
        .collect();

    let mut writer = WriterBuilder::new()
        .has_headers(false)
        .from_path("zerg-report.csv")
        .unwrap();

    reports.iter().for_each(|report| {
        writer.serialize(report).unwrap();
    })
}
