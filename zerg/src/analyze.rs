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
pub struct TransactionReport {
    uuid: String,
    completed: bool,
    arrived_on: Option<u64>,
    duration: Option<u64>,
}

pub fn analyze(input_file_path: String) -> Vec<TransactionReport> {
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
                    println!("WARNING: Processed `RecievedOn` packet without matching `SentFrom` packet.");
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
    });

    reports
}

#[cfg(test)]
mod tests {
    use super::*;

    mod analyze {
        use super::*;

        #[test]
        fn should_drop_transactions_with_no_sent_packet() {
            let reports = analyze("resources/test/output.csv".to_owned());
            assert!(reports.iter().all(|report| {
                report.uuid != "wasm.union14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s3e9fe2/channel-2/11"
            }))
        }

        #[test]
        fn should_capture_all_full_transactions() {
            let reports = analyze("resources/test/output.csv".to_owned());
            let mut full_tx_uuids = [
                "wasm.union14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s3e9fe2/channel-2/3",
                "wasm.union14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s3e9fe2/channel-2/9",
                "wasm.union14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s3e9fe2/channel-2/10",
                "ucs01-relay/channel-0/1",
                "ucs01-relay/channel-0/2",
                "ucs01-relay/channel-0/3",
            ];
            let mut reported_full_tx_uuids = vec![];

            reports.iter().for_each(|report| {
                if report.completed {
                    reported_full_tx_uuids.push(report.uuid.clone());
                }
            });

            full_tx_uuids.sort();
            reported_full_tx_uuids.sort();

            assert!(full_tx_uuids.iter().eq(reported_full_tx_uuids.iter()))
        }

        #[test]
        fn should_capture_all_incomplete_transactions() {
            let reports = analyze("resources/test/output.csv".to_owned());
            let mut full_tx_uuids = [
                "wasm.union14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s3e9fe2/channel-2/1",
                "wasm.union14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s3e9fe2/channel-2/2",
                "wasm.union14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s3e9fe2/channel-2/4",
                "wasm.union14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s3e9fe2/channel-2/5",
                "wasm.union14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s3e9fe2/channel-2/6",
                "wasm.union14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s3e9fe2/channel-2/7",
                "wasm.union14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s3e9fe2/channel-2/8",
            ];
            let mut reported_full_tx_uuids = vec![];

            reports.iter().for_each(|report| {
                if !report.completed {
                    reported_full_tx_uuids.push(report.uuid.clone());
                }
            });

            full_tx_uuids.sort();
            reported_full_tx_uuids.sort();

            assert!(full_tx_uuids.iter().eq(reported_full_tx_uuids.iter()))
        }
    }
}
