use std::future::Future;

use graphql_client::{GraphQLQuery, Response};
use reqwest::{Client, Url};
use tracing::error;

use super::Datastore;

#[derive(Debug, Clone)]
pub struct HasuraDataStore {
    client: Client,
    url: Url,
    secret: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HasuraConfig {
    pub url: Url,
    pub secret: String,
}

impl Datastore for HasuraDataStore {
    /// Performs a GraphQL post request (which may query or mutate).
    /// It injects the x-hasura-admin-secret header.
    ///
    /// # Errors
    /// - On network related errors.
    /// - If the graphql endpoint populates the errors field.
    #[allow(clippy::manual_async_fn)]
    fn do_post<Q: GraphQLQuery>(
        &self,
        v: Q::Variables,
    ) -> impl Future<Output = color_eyre::Result<Response<Q::ResponseData>>> + '_
    where
        <Q as GraphQLQuery>::Variables: 'static,
    {
        async move {
            let body = Q::build_query(v);
            let response = self
                .client
                .post(self.url.clone())
                .json(&body)
                .header("x-hasura-admin-secret", &self.secret)
                .send()
                .await?
                .text()
                .await?;

            let response: Response<Q::ResponseData> = match serde_json::from_str(&response) {
                Err(err) => {
                    error!("received unexpected response from hasura: {}", response);
                    return Err(err.into());
                }
                Ok(response) => response,
            };

            // GraphQL APIs return errors as an error field in the JSON. We convert the errors to the
            // error variant.
            if let Some(err) = response
                .errors
                .as_ref()
                .map(|errors| color_eyre::eyre::eyre!("api returned error: {:?}", errors))
            {
                return Err(err);
            }
            Ok(response)
        }
    }
}

impl HasuraDataStore {
    pub fn new(client: Client, url: Url, secret: String) -> Self {
        Self {
            client,
            url,
            secret,
        }
    }
}
