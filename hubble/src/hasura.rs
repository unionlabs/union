use std::future::Future;

use graphql_client::{GraphQLQuery, Response};
use reqwest::{Client, Url};

type Jsonb = serde_json::Value;
type Timestamptz = String;
pub trait Datastore {
    fn do_post<Q: GraphQLQuery>(
        &self,
        v: Q::Variables,
    ) -> impl Future<Output = color_eyre::Result<Response<Q::ResponseData>>> + '_
    where
        <Q as GraphQLQuery>::Variables: 'static;
}

#[derive(Clone)]
pub struct HasuraDataStore {
    client: Client,
    url: Url,
    secret: String,
}

impl Datastore for HasuraDataStore {
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
            let response: Response<Q::ResponseData> = self
                .client
                .post(self.url.clone())
                .json(&body)
                .header("x-hasura-admin-secret", &self.secret)
                .send()
                .await?
                .json()
                .await?;
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

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.graphql",
    query_path = "src/graphql/operations.graphql",
    response_derives = "Debug, Default",
    normalization = "rust",
    skip_serializing_none
)]
pub struct InsertBlock;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.graphql",
    query_path = "src/graphql/operations.graphql",
    response_derives = "Debug, Default",
    normalization = "rust",
    skip_serializing_none
)]
pub struct GetLatestBlock;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.graphql",
    query_path = "src/graphql/operations.graphql",
    response_derives = "Debug, Default",
    normalization = "rust",
    skip_serializing_none
)]
pub struct InsertChain;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.graphql",
    query_path = "src/graphql/operations.graphql",
    response_derives = "Clone, Debug, Default",
    normalization = "rust",
    skip_serializing_none
)]
pub struct InsertBlocksMany;
