use sqlx::Postgres;

use crate::abi_fetcher::{AbiDependency, Attempt, Download};

pub async fn get_missing_abi_dependencies(
    tx: &mut sqlx::Transaction<'_, Postgres>,
) -> sqlx::Result<Vec<AbiDependency>> {
    Ok(sqlx::query!(
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
    .map(|record| AbiDependency {
        commit: record.commit,
    })
    .collect())
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
        attempt.commit,
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
        download.commit,
        download.data,
        download.meta,
    )
    .execute(tx.as_mut())
    .await?;

    Ok(())
}
