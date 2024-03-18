use std::future::Future;

use graphql_client::{GraphQLQuery, Response};
use sqlx::PgConnection;

use super::{
    get_latest_block,
    insert_blocks_many::{self, V0TransactionsArrRelInsertInput},
    insert_chain, Datastore, GetLatestBlock, InsertBlocksMany, InsertChain, InsertDemoTx,
    SqlxQuery,
};

#[derive(Debug, Clone)]
pub struct PostgresDatastore {
    pool: sqlx::PgPool,
}

impl PostgresDatastore {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

impl Datastore for PostgresDatastore {
    fn do_post<Q>(
        &self,
        v: Q::Variables,
    ) -> impl Future<Output = color_eyre::Result<Response<Q::ResponseData>>> + '_
    where
        Q: GraphQLQuery + SqlxQuery,
        <Q as GraphQLQuery>::Variables: 'static,
    {
        async {
            let mut tx = self.pool.begin().await?;
            let result = Q::query(tx.as_mut(), v).await?;
            tx.commit().await?;
            Ok(result)
        }
    }
}

impl SqlxQuery for InsertDemoTx {
    fn query<'a>(
        _tx: &'a mut PgConnection,
        _variables: <Self as GraphQLQuery>::Variables,
    ) -> impl Future<Output = color_eyre::Result<Response<<Self as GraphQLQuery>::ResponseData>>> + 'a
    where
        <Self as GraphQLQuery>::Variables: 'static,
    {
        async { unimplemented!() }
    }
}

impl SqlxQuery for GetLatestBlock {
    fn query<'a>(
        tx: &'a mut PgConnection,
        variables: <Self as GraphQLQuery>::Variables,
    ) -> impl Future<Output = color_eyre::Result<Response<<Self as GraphQLQuery>::ResponseData>>> + 'a
    where
        <Self as GraphQLQuery>::Variables: 'static,
    {
        async move {
            let chain = sqlx::query!(
                "SELECT id FROM \"v0\".chains WHERE chain_id = $1 LIMIT 1",
                variables.chain_id
            )
            .fetch_optional(tx.as_mut())
            .await?;

            if let Some(chain) = chain {
                let block = sqlx::query!("SELECT height FROM \"v0\".blocks WHERE chain_id = $1 ORDER BY height DESC NULLS LAST LIMIT 1", chain.id).fetch_optional(tx.as_mut()).await?;

                Ok(Response {
                    errors: None,
                    extensions: None,
                    data: Some(get_latest_block::ResponseData {
                        v0_blocks: if let Some(block) = block {
                            vec![get_latest_block::GetLatestBlockV0Blocks {
                                height: block.height,
                            }]
                        } else {
                            vec![]
                        },
                        v0_chains: vec![get_latest_block::GetLatestBlockV0Chains {
                            id: chain.id as i64,
                        }],
                    }),
                })
            } else {
                Ok(Response {
                    errors: None,
                    extensions: None,
                    data: Some(get_latest_block::ResponseData {
                        v0_blocks: vec![],
                        v0_chains: vec![],
                    }),
                })
            }
        }
    }
}

impl SqlxQuery for InsertChain {
    fn query(
        tx: &mut PgConnection,
        variables: <Self as GraphQLQuery>::Variables,
    ) -> impl Future<Output = color_eyre::Result<Response<<Self as GraphQLQuery>::ResponseData>>>
    where
        <Self as GraphQLQuery>::Variables: 'static,
    {
        async move {
            let chain = sqlx::query!(
                "INSERT INTO \"v0\".chains (chain_id, name) VALUES ($1, $2) RETURNING id",
                variables.chain_id.clone(),
                variables.chain_id,
            )
            .fetch_one(tx.as_mut())
            .await?;

            Ok(Response {
                errors: None,
                extensions: None,
                data: Some(insert_chain::ResponseData {
                    insert_v0_chains_one: Some(insert_chain::InsertChainInsertV0ChainsOne {
                        id: chain.id as i64,
                    }),
                }),
            })
        }
    }
}

impl SqlxQuery for InsertBlocksMany {
    fn query<'a>(
        tx: &'a mut PgConnection,
        variables: <Self as GraphQLQuery>::Variables,
    ) -> impl Future<Output = color_eyre::Result<Response<<Self as GraphQLQuery>::ResponseData>>> + 'a
    where
        <Self as GraphQLQuery>::Variables: 'static,
    {
        async move {
            for block in variables.objects {
                let block_id = sqlx::query!(
                    "INSERT INTO \"v0\".blocks (chain_id, hash, height, is_finalized, data, time) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
                    block.chain_id.unwrap() as i32,
                    block.hash.unwrap(),
                    block.height.unwrap(),
                    block.is_finalized.unwrap(),
                    block.data.unwrap(),
                    block.time.unwrap(),
                )
                .fetch_one(tx.as_mut())
                .await?.id;

                for transaction in block
                    .transactions
                    .unwrap_or(V0TransactionsArrRelInsertInput {
                        data: vec![],
                        on_conflict: None,
                    })
                    .data
                {
                    let transaction_id = sqlx::query!(
                        "INSERT INTO \"v0\".transactions (block_id, data, hash, index) VALUES ($1, $2, $3, $4) RETURNING id", 
                    block_id,
                    transaction.data.unwrap(),
                    transaction.hash.unwrap(),
                    transaction.index.unwrap() as i32).fetch_one(tx.as_mut()).await?.id;

                    for event in transaction.events.into_iter().flat_map(|e| e.data) {
                        sqlx::query!("INSERT INTO \"v0\".events (block_id, transaction_id, data, index, time, stage) VALUES ($1, $2, $3, $4, $5, $6)", block_id, transaction_id, event.data.unwrap(), event.index.unwrap() as i32, event.time.unwrap(), event.stage.unwrap() as i16).execute(tx.as_mut()).await?;
                    }
                }

                for event in block.events.into_iter().flat_map(|e| e.data) {
                    sqlx::query!("INSERT INTO \"v0\".events (block_id, data, index, time, stage) VALUES ($1, $2, $3, $4, $5)", block_id, event.data.unwrap(), event.index.unwrap() as i32, event.time.unwrap(), event.stage.unwrap() as i16).execute(tx.as_mut()).await?;
                }
            }

            Ok(Response {
                errors: None,
                extensions: None,
                data: Some(insert_blocks_many::ResponseData {
                    insert_v0_blocks: None,
                }),
            })
        }
    }
}
