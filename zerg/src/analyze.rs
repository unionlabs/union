use std::{collections::HashMap, fmt::Display};

use csv::{ReaderBuilder, WriterBuilder};

use crate::process::TransactionReport;

pub fn analyze(input: String, output: String) -> HashMap<String, ChannelBenchmark> {
    let mut reader = ReaderBuilder::new()
        .has_headers(false)
        .from_path(input)
        .unwrap();
    let iter = reader.deserialize::<TransactionReport>();

    let mut durations: HashMap<String, Durations> = HashMap::new();
    let mut incomplete_transfers: HashMap<String, u64> = HashMap::new();
    let mut complete_transfers: HashMap<String, u64> = HashMap::new();

    iter.for_each(|report_maybe| {
        if let Ok(report) = report_maybe {
            let id = report.src;
            if report.completed {
                let execution_duration = report
                    .execution_duration
                    .expect("Compleated transactions should have durations");
                let finalization_duration = report
                    .finalization_duration
                    .expect("Compleated transactions should have durations");

                if let Some(times) = durations.get_mut(&id) {
                    times.push(execution_duration, finalization_duration);
                } else {
                    durations.insert(
                        id.clone(),
                        Durations::new(execution_duration, finalization_duration),
                    );
                }

                if let Some(channel_complete_transfers) = complete_transfers.get_mut(&id) {
                    *channel_complete_transfers += 1;
                } else {
                    complete_transfers.insert(id, 1);
                }
            } else if let Some(channel_incomplete_transfers) = incomplete_transfers.get_mut(&id) {
                *channel_incomplete_transfers += 1;
            } else {
                incomplete_transfers.insert(id, 1);
            }
        }
    });

    let mut benchmarks: HashMap<String, ChannelBenchmark> = HashMap::new();

    durations.keys().for_each(|channel_port_id| {
        let channel_durations = durations[channel_port_id].clone();
        let mut execution_durations = channel_durations.execution_durations.clone();
        let mut finalization_durations = channel_durations.finalization_durations.clone();
        execution_durations.sort();
        finalization_durations.sort();

        let median_transfer_execution_duration = execution_durations[execution_durations.len() / 2];
        let mean_transfer_execution_duration: u64 =
            execution_durations.iter().sum::<u64>() / (execution_durations.len() as u64);
        let max_transfer_execution_duration =
            *execution_durations.last().expect("durations is not empty");
        let min_transfer_execution_duration =
            *execution_durations.first().expect("durations is not empty");

        let median_transfer_finalization_duration =
            finalization_durations[finalization_durations.len() / 2];
        let mean_transfer_finalization_duration: u64 =
            finalization_durations.iter().sum::<u64>() / (finalization_durations.len() as u64);
        let max_transfer_finalization_duration = *finalization_durations
            .last()
            .expect("durations is not empty");
        let min_transfer_finalization_duration = *finalization_durations
            .first()
            .expect("durations is not empty");

        let incomplete_transfers = *incomplete_transfers.get(channel_port_id).unwrap_or(&0);
        let complete_transfers = *complete_transfers.get(channel_port_id).unwrap_or(&0);

        benchmarks.insert(
            channel_port_id.to_string(),
            ChannelBenchmark {
                incomplete_transfers,
                complete_transfers,
                mean_transfer_execution_duration,
                median_transfer_execution_duration,
                max_transfer_execution_duration,
                min_transfer_execution_duration,
                mean_transfer_finalization_duration,
                median_transfer_finalization_duration,
                max_transfer_finalization_duration,
                min_transfer_finalization_duration,
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

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Clone)]
pub struct Durations {
    pub execution_durations: Vec<u64>,
    pub finalization_durations: Vec<u64>,
}

impl Durations {
    fn new(execution_duration: u64, finalization_duration: u64) -> Self {
        Self {
            execution_durations: vec![execution_duration],
            finalization_durations: vec![finalization_duration],
        }
    }
    fn push(&mut self, execution_duration: u64, finalization_duration: u64) {
        self.execution_durations.push(execution_duration);
        self.finalization_durations.push(finalization_duration);
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Clone)]
pub struct ChannelBenchmark {
    mean_transfer_execution_duration: u64,
    median_transfer_execution_duration: u64,
    max_transfer_execution_duration: u64,
    min_transfer_execution_duration: u64,
    mean_transfer_finalization_duration: u64,
    median_transfer_finalization_duration: u64,
    max_transfer_finalization_duration: u64,
    min_transfer_finalization_duration: u64,
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
            as_min_sec(self.min_transfer_execution_duration),
            as_min_sec(self.min_transfer_finalization_duration),
            "Max",
            as_min_sec(self.max_transfer_execution_duration),
            as_min_sec(self.max_transfer_finalization_duration),
            "Mean",
            as_min_sec(self.mean_transfer_execution_duration),
            as_min_sec(self.mean_transfer_finalization_duration),
            "Median",
            as_min_sec(self.median_transfer_execution_duration),
            as_min_sec(self.median_transfer_finalization_duration),
            self.incomplete_transfers,
            self.complete_transfers,
        )
    }
}
