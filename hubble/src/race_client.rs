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

impl<T> RaceClientResponse<T> {
    fn new(index: usize, response: T) -> Self {
        Self {
            race_client_id: RaceClientId { index },
            response,
        }
    }
}

impl<C: Clone> Clone for RaceClient<C> {
    fn clone(&self) -> Self {
        Self {
            clients: self.clients.clone(),
        }
    }
}

impl<C> RaceClient<C> {
    pub fn new(clients: Vec<C>) -> Self {
        Self { clients }
    }

    /// A helper to filter and map client futures for racing.
    fn map_clients<'a, T, E, FUT, F>(
        &'a self,
        specific_client_id: Option<RaceClientId>,
        f: F,
    ) -> FuturesUnordered<impl Future<Output = (usize, Result<T, E>)> + 'a>
    where
        E: Debug,
        FUT: Future<Output = Result<T, E>> + 'a,
        F: Fn(&'a C) -> FUT,
    {
        self.clients
            .iter()
            .enumerate()
            .filter(|(i, _)| specific_client_id.map_or(true, |id| id.index == *i))
            .map(|(i, client)| {
                let future = f(client);
                async move {
                    let result = future.await;
                    (i, result)
                }
            })
            .collect()
    }

    /// Runs the provided closure over the clients, returning the first encountered `Ok` result.
    pub async fn race<'a, T, E, FUT, F>(
        &'a self,
        specific_client_id: Option<RaceClientId>,
        f: F,
    ) -> Result<RaceClientResponse<T>, E>
    where
        E: Debug,
        FUT: Future<Output = Result<T, E>> + 'a,
        F: Fn(&'a C) -> FUT,
    {
        let mut selected_clients = self.map_clients(specific_client_id, f);
        let mut first_error: Option<E> = None;

        while let Some((index, result)) = selected_clients.next().await {
            match result {
                Ok(response) => return Ok(RaceClientResponse::new(index, response)),
                Err(err) => {
                    debug!(client_index = index, ?err, "Error racing client request");
                    if first_error.is_none() {
                        first_error = Some(err);
                    }
                }
            }
        }

        Err(first_error.expect("No clients were available to race"))
    }

    /// Runs the provided closure over the clients, returning the first encountered `Ok(Some)` result.
    pub async fn race_some<'a, T, E, FUT, F>(
        &'a self,
        specific_client_id: Option<RaceClientId>,
        f: F,
    ) -> Result<Option<RaceClientResponse<T>>, E>
    where
        E: Debug,
        FUT: Future<Output = Result<Option<T>, E>> + 'a,
        F: Fn(&'a C) -> FUT,
    {
        let mut selected_clients = self.map_clients(specific_client_id, f);
        let mut first_error: Option<E> = None;

        while let Some((index, result)) = selected_clients.next().await {
            match result {
                Ok(Some(response)) => return Ok(Some(RaceClientResponse::new(index, response))),
                Ok(None) => continue, // Skip None results.
                Err(err) => {
                    debug!(client_index = index, ?err, "Error racing client request");
                    if first_error.is_none() {
                        first_error = Some(err);
                    }
                }
            }
        }

        if let Some(err) = first_error {
            return Err(err);
        }
        Ok(None)
    }
}
