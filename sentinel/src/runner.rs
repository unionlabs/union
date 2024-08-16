use futures::stream::{self, StreamExt};
use tracing::{info_span, Instrument};

use crate::{report::Report, sentinel::Sentinel};

pub async fn run(
    limit: impl Into<Option<usize>>,
    sentinels: Vec<Box<dyn Sentinel>>,
) -> Vec<Report> {
    let max = sentinels.len();

    stream::iter(sentinels.into_iter())
        .map(|mut sentinel| {
            let name = sentinel.name();
            let span = info_span!("sentinel", name);

            async move {
                sentinel.setup().instrument(span.clone()).await;
                sentinel.run().instrument(span.clone()).await;
                sentinel.teardown().instrument(span).await;
                sentinel.report()
            }
        })
        .buffered(limit.into().unwrap_or(max))
        .collect()
        .await
}
