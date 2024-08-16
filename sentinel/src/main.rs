mod args;
mod report;
mod runner;
mod sentinel;
mod sentinels;

use std::process::ExitCode;

use clap::Parser;

use crate::{
    args::{Cli, Command},
    sentinel::Sentinel,
};

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();

    tracing_subscriber::fmt().init();

    let tests: Vec<Box<dyn Sentinel>> = vec![
        Box::new(sentinels::fetch_latest_transfers::FetchLatestTransfers::default()),
        Box::new(sentinels::check_ibc_trace_latency::CheckIbcTraceLatency::default()),
    ];

    let mut exit = ExitCode::SUCCESS;

    match cli.cmd {
        Command::Info(_) => tests.iter().for_each(|test| {
            println!("{} - {}", test.name(), test.description());
        }),
        Command::Run(cmd) => {
            let tests: Vec<Box<dyn Sentinel>> = tests
                .into_iter()
                .map(|mut test| {
                    test.configure(&cmd);
                    test as Box<dyn Sentinel>
                })
                .filter(|test| {
                    if let Some(filter) = &cmd.filter {
                        filter.captures(test.name()).is_some()
                    } else {
                        true
                    }
                })
                .collect();

            if tests.is_empty() {
                println!("no tests to run, exiting.");
                return ExitCode::SUCCESS;
            }

            let reports = runner::run(None, tests).await;

            let mut failed = 0;
            let mut ok = 0;
            reports.into_iter().for_each(|report| {
                if report.result.is_err() {
                    exit = ExitCode::FAILURE;
                    failed += 1;
                } else {
                    ok += 1;
                }
                println!("{}", report)
            });

            let outcome = if failed == 0 { "passed" } else { "failed" };

            println!();
            println!(
                "sentinel result: {}. {} passed; {} failed;",
                outcome, ok, failed
            );
        }
    }
    exit
}
