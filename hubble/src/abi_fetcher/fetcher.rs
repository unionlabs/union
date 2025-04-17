use std::thread::sleep;

use sqlx::{Acquire, Postgres};
use tracing::{debug, error, info, warn};

use crate::{
    abi_fetcher::{
        self,
        client::{build_abis_with_commit_hash, AbiResult, BuildError},
        postgres::{get_missing_abi_dependencies, insert_download},
        AbiDependency, Attempt, Download,
    },
    github_client::commit_details::CommitDetailsError,
};

#[derive(Debug, thiserror::Error)]
pub enum FetchAbiError {
    #[error("database error creation transaction for dependency {0}: {1}")]
    CreateTransaction(AbiDependency, #[source] sqlx::Error),

    #[error("database error committing transaction for dependency {0}: {1}")]
    CommitTransaction(AbiDependency, #[source] sqlx::Error),

    #[error("database error inserting abis {0}: {1}")]
    InsertAbisError(AbiDependency, #[source] sqlx::Error),

    #[error("database error inserting attempt {0}: {1}")]
    InsertAttemptError(AbiDependency, #[source] sqlx::Error),

    #[error("commit details error {0}")]
    CannotFetchCommitDetailsError(#[from] CommitDetailsError),

    #[error("cannot build abi error {0}")]
    CannotBuildAbiError(#[from] BuildError),
}

pub async fn fetch_abis(db: &sqlx::PgPool) -> color_eyre::Result<()> {
    info!("Starting abi build process.");

    let abi_dependencies =
        get_missing_abi_dependencies(&mut db.acquire().await?.begin().await?).await?;

    for abi_dependency in abi_dependencies {
        debug!("process: {}", abi_dependency);

        match build_abi_dependency(db, &abi_dependency).await {
            Ok(_) => debug!("process: {abi_dependency} => success"),
            Err(error) => {
                warn!("process: {abi_dependency} => error: {error:?} (sleep for 5 minutes to prevent a loop that will trigger a github rate limit");
                sleep(std::time::Duration::from_secs(5 * 60));
            }
        }
    }

    info!("Finished abi build process.");
    Ok(())
}

pub async fn build_abi_dependency(
    db: &sqlx::PgPool,
    dependency: &AbiDependency,
) -> Result<(), FetchAbiError> {
    let mut tx = db
        .begin()
        .await
        .map_err(|error| FetchAbiError::CreateTransaction(dependency.clone(), error))?;

    let attempt = match build_dependency(&mut tx, dependency).await {
        Ok(abi_result) => &Attempt {
            commit: dependency.commit.clone(),
            success: true,
            details: serde_json::json!({
                "result": "OK",
                "details": abi_result.meta(),
            }),
        },
        Err(error) => &Attempt {
            commit: dependency.commit.clone(),
            success: false,
            details: serde_json::json!({
                "result:": "ERROR",
                "details": format!("{error:?}"),
            }),
        },
    };

    debug!("insert attempt: {dependency} => {attempt}");
    abi_fetcher::postgres::insert_attempt(&mut tx, attempt)
        .await
        .map_err(|error| FetchAbiError::InsertAttemptError(dependency.clone(), error))?;

    debug!("commit transaction: {dependency} => {attempt}");
    tx.commit()
        .await
        .map_err(|error| FetchAbiError::CommitTransaction(dependency.clone(), error))?;

    Ok(())
}

async fn build_dependency(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    dependency: &AbiDependency,
) -> Result<AbiResult, FetchAbiError> {
    let abi_result = build_abis_with_commit_hash(&dependency.commit).await?;

    insert_download(
        tx,
        &Download {
            commit: &dependency.commit,
            data: abi_result.data.clone(),
            meta: abi_result.meta(),
        },
    )
    .await
    .map_err(|error| FetchAbiError::InsertAbisError(dependency.clone(), error))?;
    Ok(abi_result)
}
