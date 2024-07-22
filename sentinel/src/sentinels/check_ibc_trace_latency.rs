#![allow(non_camel_case_types)]

use core::time::Duration;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::prelude::*;
use either::Either;
use graphql_client::{GraphQLQuery, Response};
use json_value_merge::Merge;
use serde::{Deserialize, Serialize};
use tokio::time::timeout;
use tracing::{error, info};
use url::Url;

use crate::{args::RunCmd, report::Report, sentinel::Sentinel};

type timestamptz = chrono::DateTime<Utc>;

#[derive(Serialize, Deserialize)]
pub struct CheckIbcTraceLatency {
    /// The number of iterations or the timestamp to check until.
    #[serde(with = "either::serde_untagged")]
    pub until: Either<usize, chrono::DateTime<Utc>>,

    /// The number of transfers to check per iteration.
    pub limit: i64,

    /// The url of the graphql endpoint.
    pub url: Url,

    /// The timeout per fetch request.
    pub timeout: Duration,

    #[serde(skip_serializing, skip_deserializing, default)]
    client: Option<reqwest::Client>,
    #[serde(skip_serializing, skip_deserializing, default)]
    report: Option<Report>,
}

impl CheckIbcTraceLatency {
    async fn try_run(&self) -> Result<()> {
        let client = self.client.clone().unwrap_or_default();

        let mut source_time = Utc::now();
        let end_time = self
            .until
            .right()
            .unwrap_or(DateTime::<Utc>::from_timestamp(0, 0).unwrap());
        let end_iter = self.until.left().unwrap_or(usize::MAX);

        for _ in 0..end_iter {
            if source_time > end_time {
                break;
            }

            let variables = last_transfers_with_traces::Variables {
                source_time,
                limit: self.limit,
            };
            let body = LastTransfersWithTraces::build_query(variables);

            info!("fetching last {} transfers with traces", self.limit);
            let raw = timeout(
                self.timeout,
                client.post(self.url.clone()).json(&body).send(),
            )
            .await
            .inspect_err(|err| {
                error!(
                    "fetching transfers exceeded {:?} deadline: {}",
                    self.timeout, err
                );
            })?
            .inspect_err(|err| {
                error!("fetching transfers failed: {}", err);
            })?
            .text()
            .await?;

            let response: Response<last_transfers_with_traces::ResponseData> =
                serde_json::from_str(&raw).inspect_err(|err| {
                    error!("decoding transfers failed: {}\n{}", err, raw);
                })?;

            let transfers = response
                .data
                .ok_or(anyhow!("response data is empty"))?
                .v0_transfers;

            for transfer in transfers {
                for _trace in transfer.traces {
                    todo!(
                        "
                    - if time difference between different stages is too big, emit warning.
                    - if a stage is absent, emit error.
                    - if the transfer isn't finished, and the time delta between now and the stage
                    is greater than some limit, emit warning
                    "
                    );
                }
                source_time = transfer.source_timestamp.unwrap();
            }
        }
        Ok(())
    }
}

impl Default for CheckIbcTraceLatency {
    fn default() -> Self {
        CheckIbcTraceLatency {
            until: Either::Left(3),
            limit: 50,
            timeout: Duration::from_secs(3),
            url: Url::parse("https://graphql.union.build/v1/graphql").unwrap(),
            client: Some(reqwest::Client::new()),
            report: None,
        }
    }
}

#[async_trait]
impl Sentinel for CheckIbcTraceLatency {
    fn name(&self) -> &str {
        "check_ibc_trace_latency"
    }

    fn description(&self) -> &str {
        "fetches the latest n transfers checks the time delta between each trace."
    }

    async fn run(&mut self) {
        let result = self.try_run().await;
        self.report = Some(Report::new(self.name(), result));
    }

    fn report(&mut self) -> Report {
        self.report.take().unwrap()
    }

    fn configure(&mut self, args: &RunCmd) {
        #![allow(clippy::needless_borrows_for_generic_args)]
        let name = self.name();

        if let Some(override_) = args.overrides.get(name) {
            let mut me = serde_json::to_value(&self)
                .expect("FetchLatestTransfers should be serializable to JSON");
            me.merge(override_);
            let mut me: Self = serde_json::from_value(me)
                .expect("FetchLatestTransfers should be deserializable from JSON");
            std::mem::swap(self, &mut me);
        }
    }
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries.graphql",
    response_derives = "Debug"
)]
pub struct LastTransfersWithTraces;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_check_ibc_trace_latency_works() {
        let mut sentinel = CheckIbcTraceLatency::default();
        sentinel.try_run().await.expect(&format!(
            "checking last {} transfers and traces should work",
            sentinel.limit
        ));
    }
}
