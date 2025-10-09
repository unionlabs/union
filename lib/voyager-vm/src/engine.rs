use std::{future::Future, marker::PhantomData, time::Duration};

use futures::{FutureExt, Stream, TryStreamExt, stream::try_unfold};
use tokio::time::sleep;

use crate::{BoxDynError, HandlerFactory, Queue, QueueMessage, filter::InterestFilter, process};

pub struct Engine<'a, T, Q, H, F> {
    handler: H,
    queue: &'a Q,
    filter: &'a F,
    __t: PhantomData<fn() -> T>,
}

impl<'a, T, Q, H, F> Engine<'a, T, Q, H, F>
where
    T: QueueMessage,
    Q: Queue<T>,
    H: HandlerFactory<T>,
    F: InterestFilter<T>,
{
    pub fn new(handler_factory: H, queue: &'a Q, filter: &'a F) -> Self {
        Self {
            handler: handler_factory,
            queue,
            filter,
            __t: PhantomData,
        }
    }

    pub fn run(
        self,
    ) -> impl Stream<Item = Result<T::Data, BoxDynError>> + Send + use<'a, T, Q, H, F> {
        try_unfold(self, async |this| {
            this.step().await.map(move |x| Some((x, this)))
        })
        .try_filter_map(async |e| Ok(e))
    }

    pub(crate) fn step<'b>(
        &'b self,
    ) -> impl Future<Output = Result<Option<T::Data>, BoxDynError>> + use<'a, 'b, T, Q, H, F> + Send
    {
        // yield back to the runtime and throttle a bit, prevents 100% cpu usage while still allowing for a fast spin-loop
        sleep(Duration::from_millis(10)).then(|()| {
            self.queue
                .process::<_, _, Option<T::Data>, _>(self.filter, async |op, id| {
                    process(op, &self.handler.make_handler(id), 0)
                        .map(|res| match res {
                            Ok(op) => (None, Ok(op.into_iter().collect())),
                            Err(err) => (None, Err(err)),
                        })
                        .await
                })
                .map(|data| match data {
                    Ok(data) => Ok(data.flatten()),
                    Err(err) => Err(err.into()),
                })
        })
    }
}
