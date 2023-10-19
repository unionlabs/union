use std::collections::HashMap;

use csv::{ReaderBuilder, WriterBuilder};

use crate::process::TransactionReport;

pub fn analyze(input: String, output: String) -> HashMap<String, ChannelBenchmark> {
    let mut reader = ReaderBuilder::new()
        .has_headers(false)
        .from_path(input)
        .unwrap();
    let iter = reader.deserialize::<TransactionReport>();

    let mut durations: HashMap<String, Vec<u64>> = HashMap::new();
    let mut incomplete_transfers: HashMap<String, u64> = HashMap::new();
    let mut complete_transfers: HashMap<String, u64> = HashMap::new();

    iter.for_each(|report_maybe| {
        if let Ok(report) = report_maybe {
            let id = get_channel_id(report.uuid);
            if report.completed {
                let duration = report
                    .duration
                    .expect("Compleated transactions should have durations");
                if let Some(times) = durations.get_mut(&id) {
                    times.push(duration);
                } else {
                    durations.insert(id.clone(), vec![duration]);
                }
                if let Some(channel_complete_transfers) = complete_transfers.get_mut(&id) {
                    *channel_complete_transfers += 1;
                } else {
                    complete_transfers.insert(id, 1);
                }
            } else if let Some(channel_incomplete_transfers) = incomplete_transfers.get_mut(&id) {
                println!(
                    "Additional incomplete transfer on {} ({} + 1 incomplete)",
                    id, channel_incomplete_transfers
                );
                *channel_incomplete_transfers += 1;
            } else {
                println!("Incomplete transfer on {}", id);
                incomplete_transfers.insert(id, 1);
            }
        }
    });

    let mut benchmarks: HashMap<String, ChannelBenchmark> = HashMap::new();

    durations.keys().for_each(|channel_port_id| {
        let mut channel_durations = durations[channel_port_id].clone();
        channel_durations.sort();

        let median_transfer_duration = channel_durations[channel_durations.len() / 2];
        let mean_transfer_duration: u64 =
            channel_durations.iter().sum::<u64>() / (channel_durations.len() as u64);
        let max_transfer_duration = *channel_durations.last().expect("durations is not empty");
        let min_transfer_duration = *channel_durations.first().expect("durations is not empty");
        let incomplete_transfers = *incomplete_transfers.get(channel_port_id).unwrap_or(&0);
        let complete_transfers = *complete_transfers.get(channel_port_id).unwrap_or(&0);

        benchmarks.insert(
            channel_port_id.to_string(),
            ChannelBenchmark {
                mean_transfer_duration,
                median_transfer_duration,
                max_transfer_duration,
                min_transfer_duration,
                incomplete_transfers,
                complete_transfers,
            },
        );
    });

    let mut writer = WriterBuilder::new()
        .has_headers(false)
        .from_path(output)
        .unwrap();

    benchmarks.iter().for_each(|report| {
        writer.serialize(report).unwrap();
    });

    dbg!(benchmarks)
}

fn get_channel_id(uuid: String) -> String {
    let parts = uuid.split('/');
    let id = parts.take(2);
    let id: Vec<&str> = id.collect();
    let id: Vec<String> = vec![id[0].to_owned(), id[1].to_owned()];

    format!("{}/{}", id[0], id[1])
}

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Clone)]
pub struct ChannelBenchmark {
    mean_transfer_duration: u64,
    median_transfer_duration: u64,
    max_transfer_duration: u64,
    min_transfer_duration: u64,
    incomplete_transfers: u64,
    complete_transfers: u64,
}
