use sqlx::Postgres;

use crate::github_fetcher::{Attempt, Download, Subscription};

pub async fn get_subscriptions(
    tx: &mut sqlx::Transaction<'_, Postgres>,
) -> sqlx::Result<Vec<Subscription>> {
    Ok(sqlx::query!(
        r#"
            SELECT s.id, s.repo, s.path, s.branch, (SELECT data FROM internet.download WHERE subscription_id = s.id ORDER BY id DESC LIMIT 1) AS data
            FROM internet.subscription s
            WHERE enabled = true AND
            NOT EXISTS (SELECT 1 FROM internet.attempt a WHERE a.subscription_id = s.id AND a.timestamp > now() - make_interval(secs => s.interval_seconds))
            ORDER BY id
        "#,
    )
    .fetch_all(tx.as_mut())
    .await?
    .into_iter()
    .map(|record| Subscription {
        id: record.id,
        repo: record.repo,
        path: record.path,
        branch: record.branch,
        data: record.data,
    })
    .collect())
}

pub async fn insert_attempt(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    attempt: &Attempt,
) -> sqlx::Result<()> {
    sqlx::query!(
        "
        INSERT INTO internet.attempt(subscription_id, success, details)
        VALUES ($1, $2, $3)
        ",
        attempt.subscription_id,
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
        INSERT INTO internet.download(subscription_id, data, meta)
        VALUES ($1, $2, $3)
        ",
        download.subscription_id,
        download.data.0,
        download.meta,
    )
    .execute(tx.as_mut())
    .await?;

    Ok(())
}
