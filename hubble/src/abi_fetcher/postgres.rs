use sqlx::Postgres;

use crate::{
    abi_fetcher::{AbiDependency, Attempt, Download},
    github_client::GitCommitHash,
};

pub async fn get_missing_abi_dependencies(
    tx: &mut sqlx::Transaction<'_, Postgres>,
) -> sqlx::Result<Vec<AbiDependency>> {
    sqlx::query!(
        r#"
            SELECT d.commit
            FROM abi.dependency d
            WHERE NOT EXISTS (
                SELECT 1
                FROM abi.download d2
                WHERE d2.commit = d.commit
            )
            ORDER BY created_at ASC
        "#,
    )
    .fetch_all(tx.as_mut())
    .await?
    .into_iter()
    .map(|record| -> sqlx::Result<_> {
        Ok(AbiDependency {
            commit: GitCommitHash::from_slice(record.commit.as_slice())
                .map_err(|e| sqlx::Error::Decode(e.into()))?,
        })
    })
    .collect()
}

pub async fn insert_attempt(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    attempt: &Attempt,
) -> sqlx::Result<()> {
    sqlx::query!(
        "
        INSERT INTO abi.attempt(commit, success, details)
        VALUES ($1, $2, $3)
        ",
        attempt.commit.as_bytes(),
        attempt.success,
        attempt.details,
    )
    .execute(tx.as_mut())
    .await?;

    Ok(())
}

pub async fn insert_download(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    download: &Download<'_>,
) -> sqlx::Result<()> {
    sqlx::query!(
        "
        INSERT INTO abi.download(commit, data, meta)
        VALUES ($1, $2, $3)
        ",
        download.commit.as_bytes(),
        download.data,
        download.meta,
    )
    .execute(tx.as_mut())
    .await?;

    Ok(())
}
