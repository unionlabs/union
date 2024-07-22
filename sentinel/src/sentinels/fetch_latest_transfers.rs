#![allow(non_camel_case_types)]
use std::time::Duration;

use anyhow::{bail, Result};
use async_trait::async_trait;
use graphql_client::{GraphQLQuery, Response};
use json_value_merge::Merge;
use serde::{Deserialize, Serialize};
use tokio::time::timeout;
use tracing::{error, info};
use url::Url;

use crate::{args::RunCmd, report::Report, sentinel::Sentinel};

/// A sentinel which checks that the last n transfers are fetchable within a given time.
#[derive(Serialize, Deserialize)]
pub struct FetchLatestTransfers {
    /// The number of transfers to fetch.
    pub limit: i64,
    /// The url of the graphql endpoint.
    pub url: Url,
    /// The request timeout. If the request cannot be completed within the
    /// duration, the test is considered failed.
    pub timeout: Duration,

    #[serde(skip_serializing, skip_deserializing, default)]
    client: Option<reqwest::Client>,
    #[serde(skip_serializing, skip_deserializing, default)]
    report: Option<Report>,
}

impl Default for FetchLatestTransfers {
    fn default() -> Self {
        FetchLatestTransfers {
            limit: 50,
            timeout: Duration::from_secs(3),
            url: Url::parse("https://graphql.union.build/v1/graphql").unwrap(),
            client: Some(reqwest::Client::new()),
            report: None,
        }
    }
}

type timestamptz = String;
type numeric = u128;
type jsonb = serde_json::Value;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries.graphql",
    response_derives = "Debug"
)]
pub struct LastTransfersLimit;

impl FetchLatestTransfers {
    async fn try_run(&self) -> Result<()> {
        let client = self.client.clone().unwrap_or_default();
        let variables = last_transfers_limit::Variables { limit: self.limit };
        let body = LastTransfersLimit::build_query(variables);

        info!("fetching last {} transfers", self.limit);
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

        let response: Response<last_transfers_limit::ResponseData> = serde_json::from_str(&raw)
            .inspect_err(|err| {
                error!("decoding transfers failed: {}\n{}", err, raw);
            })?;

        if response.data.is_none() {
            bail!("response had no data: {:?}", response)
        }
        Ok(())
    }
}

#[async_trait]
impl Sentinel for FetchLatestTransfers {
    fn name(&self) -> &str {
        "fetch_latest_transfers"
    }

    fn description(&self) -> &str {
        "fetches the latest n transfers and times the request."
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_latest_transfers_works() {
        let mut sentinel = FetchLatestTransfers::default();
        sentinel.try_run().await.expect(&format!(
            "fetching last {} transfers should work",
            sentinel.limit
        ));
    }
}
