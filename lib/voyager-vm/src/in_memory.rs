use std::{
    collections::BTreeMap,
    future::Future,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc, Mutex,
    },
};

use derive_where::derive_where;
use either::Either;
use tracing::{debug, info_span, warn, Instrument};

use crate::{
    filter::{FilterResult, InterestFilter},
    pass::Pass,
    Captures, Op, Queue, QueueMessage,
};

#[derive_where(Debug, Clone)]
pub struct InMemoryQueue<T: QueueMessage> {
    idx: Arc<AtomicU32>,
    ready: Arc<Mutex<BTreeMap<u32, Item<T>>>>,
    done: Arc<Mutex<BTreeMap<u32, Item<T>>>>,
    #[allow(clippy::type_complexity)]
    optimizer_queue: Arc<Mutex<BTreeMap<String, BTreeMap<u32, Item<T>>>>>,
}

#[derive_where(Debug, PartialEq, Clone)]
pub(crate) struct Item<T: QueueMessage> {
    #[allow(dead_code)] // used in debug
    parents: Vec<u32>,
    op: Op<T>,
}

impl<T: QueueMessage> Queue<T> for InMemoryQueue<T> {
    type Error = std::convert::Infallible;
    type Config = ();

    fn new(_cfg: Self::Config) -> impl Future<Output = Result<Self, Self::Error>> {
        futures::future::ok(Self {
            idx: Arc::new(AtomicU32::default()),
            done: Arc::new(Mutex::new(BTreeMap::default())),
            ready: Arc::new(Mutex::new(BTreeMap::default())),
            optimizer_queue: Arc::new(Mutex::new(BTreeMap::default())),
        })
    }

    fn enqueue<'a>(
        &'a self,
        op: Op<T>,
        filter: &'a T::Filter,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + 'a {
        debug!(?op, "enqueueing new item");

        let mut optimizer_queue = self.optimizer_queue.lock().expect("mutex is poisoned");
        let mut ready = self.ready.lock().expect("mutex is poisoned");

        for op in op.normalize() {
            match filter.check_interest(&op) {
                FilterResult::Interest(tag) => {
                    optimizer_queue.entry(tag.to_owned()).or_default().insert(
                        self.idx.fetch_add(1, Ordering::SeqCst),
                        Item {
                            parents: vec![],
                            op,
                        },
                    );
                }
                FilterResult::NoInterest => {
                    ready.insert(
                        self.idx.fetch_add(1, Ordering::SeqCst),
                        Item {
                            parents: vec![],
                            op,
                        },
                    );
                }
            }
        }

        debug!("enqueued new item");

        futures::future::ok(())
    }

    async fn process<'a, F, Fut, R>(
        &'a self,
        filter: &'a T::Filter,
        f: F,
    ) -> Result<Option<R>, Self::Error>
    where
        F: (FnOnce(Op<T>) -> Fut) + Send + Captures<'a>,
        Fut: Future<Output = (R, Result<Vec<Op<T>>, String>)> + Send + Captures<'a>,
        R: Send + Sync + 'static,
    {
        let op = {
            let mut queue = self.ready.lock().expect("mutex is poisoned");
            let op = queue.pop_first();

            drop(queue);

            op
        };

        match op {
            Some((id, item)) => {
                let span = info_span!("processing item", %id);

                self.done
                    .lock()
                    .expect("mutex is poisoned")
                    .insert(id, item.clone());

                let (r, res) = f(item.op.clone()).instrument(span).await;
                match res {
                    Ok(ops) => {
                        let mut optimizer_queue =
                            self.optimizer_queue.lock().expect("mutex is poisoned");
                        let mut ready = self.ready.lock().expect("mutex is poisoned");

                        for op in ops.into_iter().flat_map(Op::normalize) {
                            match filter.check_interest(&op) {
                                FilterResult::Interest(tag) => {
                                    optimizer_queue.entry(tag.to_owned()).or_default().insert(
                                        self.idx.fetch_add(1, Ordering::SeqCst),
                                        Item {
                                            parents: vec![id],
                                            op,
                                        },
                                    );
                                }
                                FilterResult::NoInterest => {
                                    ready.insert(
                                        self.idx.fetch_add(1, Ordering::SeqCst),
                                        Item {
                                            parents: vec![id],
                                            op,
                                        },
                                    );
                                }
                            }
                        }

                        Ok(Some(r))
                    }
                    Err(why) => panic!("{why}"),
                }
            }
            None => {
                // trace!("queue is empty, sleeping for 1 second");

                // sleep(Duration::from_secs(1)).await;

                Ok(None)
            }
        }
    }

    #[allow(clippy::manual_async_fn)]
    fn optimize<'a, O: Pass<T>>(
        &'a self,
        tag: &'a str,
        optimizer: &'a O,
    ) -> impl Future<Output = Result<(), Either<Self::Error, O::Error>>> + 'a {
        async move {
            let tagged_optimizer_queue = {
                let mut optimizer_queue = self.optimizer_queue.lock().expect("poisoned");
                let Some(tagged_optimizer_queue) = optimizer_queue.remove(tag) else {
                    warn!(%tag, "no items with tag");
                    return Ok(());
                };

                drop(optimizer_queue);

                tagged_optimizer_queue
            };

            let (ids, ops): (Vec<_>, Vec<_>) = tagged_optimizer_queue.clone().into_iter().unzip();

            let res = optimizer
                .run_pass(ops.into_iter().map(|item| item.op).collect())
                .await
                .map_err(Either::Right)?;

            let mut optimizer_queue = self.optimizer_queue.lock().expect("poisoned");
            let mut ready = self.ready.lock().expect("poisoned");
            let mut done = self.done.lock().expect("poisoned");

            done.append(&mut tagged_optimizer_queue.clone());

            for (parents_idxs, op) in res.ready {
                ready.insert(
                    self.idx.fetch_add(1, Ordering::SeqCst),
                    Item {
                        parents: parents_idxs.iter().map(|&i| &ids[i]).copied().collect(),
                        op,
                    },
                );
            }

            for (parents_idxs, op, tag) in res.optimize_further {
                optimizer_queue.entry(tag.clone()).or_default().insert(
                    self.idx.fetch_add(1, Ordering::SeqCst),
                    Item {
                        parents: parents_idxs.iter().map(|&i| &ids[i]).copied().collect(),
                        op,
                    },
                );
            }

            Ok(())
        }
    }
}
