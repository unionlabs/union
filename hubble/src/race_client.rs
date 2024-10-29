use core::{fmt::Debug, future::Future};
use std::sync::atomic::{AtomicUsize, Ordering};

use futures::{stream::FuturesUnordered, StreamExt};
use tracing::debug;

#[derive(Debug)]
pub struct RaceClient<C> {
    pub clients: Vec<C>,
    fastest: AtomicUsize,
}

impl<C: Clone> Clone for RaceClient<C> {
    fn clone(&self) -> Self {
        let clients = self.clients.clone();
        let fastest = self.fastest.load(Ordering::Relaxed);
        Self {
            clients,
            fastest: fastest.into(),
        }
    }
}

impl<C> RaceClient<C> {
    pub fn new(clients: Vec<C>) -> Self {
        Self {
            clients,
            fastest: AtomicUsize::new(0),
        }
    }

    pub fn fastest_index(&self) -> usize {
        self.fastest.load(Ordering::Relaxed)
    }

    /// Run the provided closure over the clients, returning the first encountered Ok, or if all error, the first
    /// encountered Err.
    pub async fn race<
        'a,
        T,
        E: Debug,
        FUT: Future<Output = Result<T, E>> + 'a,
        F: Fn(&'a C) -> FUT,
    >(
        &'a self,
        f: F,
    ) -> Result<T, E> {
        let mut futures: FuturesUnordered<_> = self
            .clients
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let f = f(c);
                async move {
                    let res = f.await;
                    (i, res)
                }
            })
            .collect();
        let mut error = None;

        loop {
            match futures.next().await {
                Some((i, Ok(res))) => {
                    self.fastest.store(i, Ordering::Relaxed);
                    return Ok(res);
                }
                Some((_, Err(err))) => {
                    debug!("error racing client requests: {:?}", err);
                    if error.is_none() {
                        error = Some(err)
                    }
                }
                None => break,
            }
        }

        Err(error.unwrap())
    }

    /// Run the provided closure over the clients, returning the first encountered Ok, or if all error, the first
    /// encountered Err.
    pub async fn race_some<
        'a,
        T,
        E: Debug,
        FUT: Future<Output = Result<Option<T>, E>> + 'a,
        F: Fn(&'a C) -> FUT,
    >(
        &'a self,
        f: F,
    ) -> Result<Option<T>, E> {
        let mut futures: FuturesUnordered<_> = self
            .clients
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let f = f(c);
                async move {
                    let res = f.await;
                    (i, res)
                }
            })
            .collect();
        let mut error = None;

        loop {
            match futures.next().await {
                Some((i, Ok(Some(res)))) => {
                    self.fastest.store(i, Ordering::Relaxed);
                    return Ok(Some(res));
                }
                Some((_, Ok(None))) => continue,
                Some((_, Err(err))) => {
                    debug!("error racing client requests: {:?}", err);
                    if error.is_none() {
                        error = Some(err)
                    }
                }
                None => break,
            }
        }
        if let Some(err) = error {
            return Err(err);
        }
        Ok(None)
    }
}
