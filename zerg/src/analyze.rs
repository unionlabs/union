use std::{collections::HashMap, fmt::Display};

use csv::{ReaderBuilder, WriterBuilder};

use crate::process::TransactionReport;

pub fn analyze(input: String, output: String) -> HashMap<String, ChannelBenchmark> {
    let mut reader = ReaderBuilder::new()
        .has_headers(false)
        .from_path(input)
        .unwrap();
    let iter = reader.deserialize::<TransactionReport>();

    let mut durations: HashMap<String, Vec<(u64, u64)>> = HashMap::new();
    let mut incomplete_transfers: HashMap<String, u64> = HashMap::new();
    let mut complete_transfers: HashMap<String, u64> = HashMap::new();

    for report in iter.flatten() {
        let id = report.src;
        match (report.execution_duration, report.finalization_duration) {
            (Some(execution_duration), Some(finalization_duration)) => {
                if let Some(times) = durations.get_mut(&id) {
                    times.push((execution_duration, finalization_duration));
                } else {
                    durations.insert(
                        id.clone(),
                        vec![(execution_duration, finalization_duration)],
                    );
                }

                if let Some(channel_complete_transfers) = complete_transfers.get_mut(&id) {
                    *channel_complete_transfers += 1;
                } else {
                    complete_transfers.insert(id, 1);
                }
            }
            (None, None) => {
                if let Some(channel_incomplete_transfers) = incomplete_transfers.get_mut(&id) {
                    *channel_incomplete_transfers += 1;
                } else {
                    incomplete_transfers.insert(id, 1);
                }
            }
            _ => {
                tracing::error!("Analyze: Malformed data for transaction {}. Expected to have both execution and finalization duration or neither.", report.uuid)
            }
        }
    }

    let mut benchmarks: HashMap<String, ChannelBenchmark> = HashMap::new();

    durations.keys().for_each(|channel_port_id| {
        let channel_durations = durations[channel_port_id].clone();
        let (mut execution_durations, mut finalization_durations): (Vec<u64>, Vec<u64>) =
            channel_durations.into_iter().unzip();
        execution_durations.sort();
        finalization_durations.sort();

        let median_execution_duration = median(&execution_durations);
        let mean_execution_duration: u64 = mean(&execution_durations);
        let max_execution_duration = *execution_durations.last().expect("durations is not empty");
        let min_execution_duration = *execution_durations.first().expect("durations is not empty");

        let median_finalization_duration = median(&finalization_durations);
        let mean_finalization_duration: u64 = mean(&finalization_durations);
        let max_finalization_duration = *finalization_durations
            .iter()
            .max()
            .expect("durations is not empty");
        let min_finalization_duration = *finalization_durations
            .iter()
            .min()
            .expect("durations is not empty");

        let incomplete_transfers = *incomplete_transfers.get(channel_port_id).unwrap_or(&0);
        let complete_transfers = *complete_transfers.get(channel_port_id).unwrap_or(&0);

        benchmarks.insert(
            channel_port_id.to_string(),
            ChannelBenchmark {
                incomplete_transfers,
                complete_transfers,
                mean_execution_duration,
                median_execution_duration,
                max_execution_duration,
                min_execution_duration,
                mean_finalization_duration,
                median_finalization_duration,
                max_finalization_duration,
                min_finalization_duration,
                from: channel_port_id.to_string(),
            },
        );
    });

    let mut writer = WriterBuilder::new()
        .has_headers(false)
        .from_path(output)
        .unwrap();

    benchmarks.iter().for_each(|report| {
        writer.serialize(report).unwrap();
        println!("{}", report.1);
    });

    benchmarks
}

fn median(values: &[u64]) -> u64 {
    if values.len() % 2 == 0 {
        let mid = values.len() / 2;
        (values[mid - 1] + values[mid]) / 2
    } else {
        values[values.len() / 2]
    }
}

fn mean(values: &[u64]) -> u64 {
    values.iter().sum::<u64>() / (values.len() as u64)
}

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Clone)]
pub struct ChannelBenchmark {
    mean_execution_duration: u64,
    median_execution_duration: u64,
    max_execution_duration: u64,
    min_execution_duration: u64,
    mean_finalization_duration: u64,
    median_finalization_duration: u64,
    max_finalization_duration: u64,
    min_finalization_duration: u64,
    incomplete_transfers: u64,
    complete_transfers: u64,
    from: String,
}

fn as_min_sec(duration: u64) -> String {
    format!("{}m{}s", duration / 60, duration % 60)
}

fn as_to_from(src_id: String) -> String {
    match src_id.as_str() {
        "union-testnet-3" => "Union -> Sepolia".to_string(),
        "union-testnet-4" => "Union -> Sepolia".to_string(),
        "11155111" => "Sepolia -> Union".to_string(),
        _ => format!("From {}", src_id),
    }
}

impl Display for ChannelBenchmark {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}\n\
            {:<10}  {:<10}  {:<10}\n\
            {:<10}  {:<10}  {:<10}\n\
            {:<10}  {:<10}  {:<10}\n\
            {:<10}  {:<10}  {:<10}\n\
            {:<10}  {:<10}  {:<10}\n\
            Dropped: {}\n\
            Completed: {}\n",
            as_to_from(self.from.clone()),
            "Stat",
            "Exec",
            "Final",
            "Min",
            as_min_sec(self.min_execution_duration),
            as_min_sec(self.min_finalization_duration),
            "Max",
            as_min_sec(self.max_execution_duration),
            as_min_sec(self.max_finalization_duration),
            "Mean",
            as_min_sec(self.mean_execution_duration),
            as_min_sec(self.mean_finalization_duration),
            "Median",
            as_min_sec(self.median_execution_duration),
            as_min_sec(self.median_finalization_duration),
            self.incomplete_transfers,
            self.complete_transfers,
        )
    }
}
