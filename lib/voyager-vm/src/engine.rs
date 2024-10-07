use std::{future::Future, time::Duration};

use futures::{stream, FutureExt, Stream, StreamExt};
use tokio::time::sleep;
use tracing::error;
use unionlabs::ErrorReporter;

use crate::{defer, now, seq, BoxDynError, Captures, Queue, QueueError, QueueMessage};

pub struct Engine<'a, T: QueueMessage, Q: Queue<T>> {
    store: &'a T::Context,
    queue: &'a Q,
    optimizer: &'a T::Filter,
}

impl<'a, T: QueueMessage, Q: Queue<T>> Engine<'a, T, Q> {
    pub fn new(store: &'a T::Context, queue: &'a Q, filter: &'a T::Filter) -> Self {
        Self {
            store,
            queue,
            optimizer: filter,
        }
    }

    pub fn run(self) -> impl Stream<Item = Result<T::Data, BoxDynError>> + Send + Captures<'a> {
        futures::stream::try_unfold(self, |this| async move {
            sleep(Duration::from_millis(10)).await;
            let res = this.step().await;
            res.map(move |x| x.map(|x| (x, this)))
        })
        .flat_map(|x| stream::iter(x.transpose()))
    }

    pub(crate) fn step<'b>(
        &'b self,
    ) -> impl Future<Output = Result<Option<Option<T::Data>>, BoxDynError>>
           + Captures<'a>
           + Captures<'b>
           + Send {
        // yield back to the runtime and throttle a bit, prevents 100% cpu usage while still allowing for a fast spin-loop
        sleep(Duration::from_millis(10)).then(|()| {
            self.queue
                .process::<_, _, Option<T::Data>>(self.optimizer, |op| {
                    op.clone().process(self.store, 0).map(|res| match res {
                        Ok(op) => (None, Ok(op.into_iter().collect())),
                        Err(QueueError::Fatal(fatal)) => {
                            let full_err = ErrorReporter(&*fatal);
                            error!(error = %full_err, "fatal error");
                            (None, Err(full_err.to_string()))
                        }
                        Err(QueueError::Retry(retry)) => {
                            // TODO: Add some backoff logic here based on `full_err`?
                            let full_err = ErrorReporter(&*retry);
                            error!(error = %full_err, "retryable error");
                            (None, Ok(vec![seq([defer(now() + 3), op])]))
                        }
                    })
                })
                .map(|data| match data {
                    Ok(data) => Ok(Some(data.flatten())),
                    Err(err) => Err(err.into()),
                })
        })
    }
}
