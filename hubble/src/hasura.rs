use graphql_client::{GraphQLQuery, Response};
use reqwest::Client;

type Jsonb = serde_json::Value;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.graphql",
    query_path = "src/graphql/operations.graphql",
    response_derives = "Debug",
    normalization = "rust",
    skip_serializing_none
)]
pub struct InsertBlock;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.graphql",
    query_path = "src/graphql/operations.graphql",
    response_derives = "Debug",
    normalization = "rust",
    skip_serializing_none
)]
pub struct GetLatestBlock;

pub async fn do_post<Q: GraphQLQuery>(
    auth: &str,
    url: &str,
    client: &Client,
    variables: Q::Variables,
) -> color_eyre::Result<Response<Q::ResponseData>> {
    let body = Q::build_query(variables);
    let response: Response<Q::ResponseData> = client
        .post(url)
        .json(&body)
        .header("x-hasura-admin-secret", auth)
        .send()
        .await?
        .json()
        .await?;
    Ok(response)
}
