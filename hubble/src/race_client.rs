use core::{fmt::Debug, future::Future};

use futures::{stream::FuturesUnordered, StreamExt};
use tracing::debug;

#[derive(Debug, Clone)]
pub struct RaceClient<C> {
    pub clients: Vec<C>,
}

impl<C> RaceClient<C> {
    pub fn new(clients: Vec<C>) -> Self {
        Self { clients }
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
        let mut futures: FuturesUnordered<_> = self.clients.iter().map(f).collect();
        let mut error = None;

        loop {
            match futures.next().await {
                Some(Ok(res)) => return Ok(res),
                Some(Err(err)) => {
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
        let mut futures: FuturesUnordered<_> = self.clients.iter().map(f).collect();
        let mut error = None;

        loop {
            match futures.next().await {
                Some(Ok(Some(res))) => return Ok(Some(res)),
                Some(Ok(None)) => continue,
                Some(Err(err)) => {
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
