use core::{fmt::Debug, future::Future};

use futures::{stream::FuturesUnordered, StreamExt};
use tracing::debug;

#[derive(Debug)]
pub struct RaceClient<C> {
    pub clients: Vec<C>,
}

#[derive(Clone, Debug, Copy)]
pub struct RaceClientId {
    index: usize,
}

pub struct RaceClientResponse<T> {
    pub race_client_id: RaceClientId,
    pub response: T,
}

impl <T> RaceClientResponse<T> {
    fn new(index: usize, response: T) -> Self {
        RaceClientResponse {
            race_client_id: RaceClientId { index },
            response,
        }
    }
}

impl<C: Clone> Clone for RaceClient<C> {
    fn clone(&self) -> Self {
        let clients = self.clients.clone();
        Self {
            clients,
        }
    }
}

impl<C> RaceClient<C> {
    pub fn new(clients: Vec<C>) -> Self {
        Self {
            clients,
        }
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
        race_client_id: Option<RaceClientId>,
        f: F,
    ) -> Result<RaceClientResponse<T>, E> {
        let mut futures: FuturesUnordered<_> = self
            .clients
            .iter()
            .enumerate()
            .filter(|(i, _)| race_client_id.as_ref().is_none_or(|id| &id.index == i))
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
                    return Ok(RaceClientResponse::new(i, res));
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
        race_client_id: Option<RaceClientId>,
        f: F,
    ) -> Result<Option<RaceClientResponse<T>>, E> {
        let mut futures: FuturesUnordered<_> = self
            .clients
            .iter()
            .enumerate()
            .filter(|(i, _)| race_client_id.as_ref().is_none_or(|id| &id.index == i))
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
                    return Ok(Some(RaceClientResponse::new(i, res)));
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
