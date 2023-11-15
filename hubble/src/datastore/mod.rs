pub mod hasura;
pub mod postgres;

use std::future::Future;

use graphql_client::{GraphQLQuery, Response};
use sqlx::{types::time::OffsetDateTime, PgConnection};

pub type Jsonb = serde_json::Value;
pub type Timestamptz = OffsetDateTime;
pub type Bigint = i64;
pub type Smallint = i32;

pub trait Datastore {
    fn do_post<Q: GraphQLQuery + SqlxQuery>(
        &self,
        v: Q::Variables,
    ) -> impl Future<Output = color_eyre::Result<Response<Q::ResponseData>>> + '_
    where
        <Q as GraphQLQuery>::Variables: 'static;
}

pub trait SqlxQuery: GraphQLQuery {
    fn query<'a>(
        tx: &'a mut PgConnection,
        variables: <Self as GraphQLQuery>::Variables,
    ) -> impl Future<Output = color_eyre::Result<Response<<Self as GraphQLQuery>::ResponseData>>> + 'a
    where
        <Self as GraphQLQuery>::Variables: 'static;
}

pub mod either {
    use super::*;

    #[derive(Debug, Clone)]
    pub enum Either<A, B> {
        A(A),
        B(B),
    }

    impl<A, B> Datastore for Either<A, B>
    where
        A: Datastore,
        B: Datastore,
    {
        fn do_post<Q: GraphQLQuery + SqlxQuery>(
            &self,
            v: Q::Variables,
        ) -> impl Future<Output = color_eyre::Result<Response<Q::ResponseData>>> + '_
        where
            <Q as GraphQLQuery>::Variables: 'static,
        {
            async move {
                match self {
                    Either::A(a) => a.do_post::<Q>(v).await,
                    Either::B(b) => b.do_post::<Q>(v).await,
                }
            }
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

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.graphql",
    query_path = "src/graphql/operations.graphql",
    response_derives = "Clone, Debug, Default",
    normalization = "rust",
    skip_serializing_none
)]
pub struct InsertDemoTx;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.graphql",
    query_path = "src/graphql/operations.graphql",
    response_derives = "Clone, Debug, Default",
    normalization = "rust",
    skip_serializing_none
)]
pub struct InsertDemoQueue;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.graphql",
    query_path = "src/graphql/operations.graphql",
    response_derives = "Clone, Debug",
    normalization = "rust",
    skip_serializing_none
)]
pub struct GetLatestQueue;
