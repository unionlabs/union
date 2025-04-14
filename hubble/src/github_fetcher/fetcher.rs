use std::thread::sleep;

use serde::Serialize;
use sqlx::{Acquire, Postgres};
use tracing::{debug, error, info, warn};

use crate::github_fetcher::{
    self,
    client::{fetch_commit_details, fetch_file_contents, CommitDetails, FileDownloadError},
    postgres::get_subscriptions,
    Attempt, Download, Subscription,
};

#[derive(Debug, thiserror::Error)]
pub enum UpdateSubscriptionError {
    #[error("database error creation transaction for subscription {0}: {1}")]
    CreateTransaction(Subscription, #[source] sqlx::Error),

    #[error("database error committing transaction for subscription {0}: {1}")]
    CommitTransaction(Subscription, #[source] sqlx::Error),

    #[error("database error inserting download {0}: {1}")]
    InsertDownloadError(Subscription, #[source] sqlx::Error),

    #[error("database error inserting attempt {0}: {1}")]
    InsertAttemptError(Subscription, #[source] sqlx::Error),

    #[error("file download error {0}")]
    CannotDownloadFileError(#[from] FileDownloadError),
}

pub async fn update_subscriptions(db: &sqlx::PgPool) -> color_eyre::Result<()> {
    info!("Starting github update process.");

    let subscriptions = get_subscriptions(&mut db.acquire().await?.begin().await?).await?;

    for subscription in subscriptions {
        debug!("process: {}", subscription);

        match update_subscription(db, &subscription).await {
            Ok(_) => debug!("process: {subscription} => success"),
            Err(error) => {
                warn!("process: {subscription} => error: {error:?} (sleep for 5 minutes to prevent a loop that will trigger a github rate limit");
                sleep(std::time::Duration::from_secs(5 * 60));
            }
        }
    }

    info!("Finished github update process.");
    Ok(())
}

pub async fn update_subscription(
    db: &sqlx::PgPool,
    subscription: &Subscription,
) -> Result<(), UpdateSubscriptionError> {
    let mut tx = db
        .begin()
        .await
        .map_err(|error| UpdateSubscriptionError::CreateTransaction(subscription.clone(), error))?;

    let attempt = match refresh_subscription(&mut tx, subscription).await {
        Ok((status, commit_details)) => &Attempt {
            subscription_id: subscription.id.clone(),
            success: true,
            details: serde_json::json!({
                "result": "OK",
                "status:": status,
                "details": commit_details,
            }),
        },
        Err(error) => &Attempt {
            subscription_id: subscription.id.clone(),
            success: false,
            details: serde_json::json!({
                "result:": "ERROR",
                "details": format!("{error:?}"),
            }),
        },
    };

    debug!("insert attempt: {subscription} => {attempt}");
    github_fetcher::postgres::insert_attempt(&mut tx, attempt)
        .await
        .map_err(|error| {
            UpdateSubscriptionError::InsertAttemptError(subscription.clone(), error)
        })?;

    debug!("commit transaction: {subscription} => {attempt}");
    tx.commit()
        .await
        .map_err(|error| UpdateSubscriptionError::CommitTransaction(subscription.clone(), error))?;

    Ok(())
}

async fn refresh_subscription(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    subscription: &Subscription,
) -> Result<(Status, CommitDetails), UpdateSubscriptionError> {
    let commit_details = fetch_commit_details(subscription).await?;
    let file_contents = fetch_file_contents(subscription, &commit_details).await?;

    let status = match subscription.data {
        Some(ref data) => match data == &file_contents.0 {
            true => Status::Unchanged,
            false => Status::Changed,
        },
        None => Status::New,
    };

    match &status {
        Status::New | Status::Changed => {
            debug!("process: {subscription} => new or changed ({commit_details}) => insert new download");
            github_fetcher::postgres::insert_download(
                tx,
                &Download {
                    subscription_id: subscription.id.clone(),
                    data: &file_contents,
                    meta: serde_json::to_value(&commit_details).expect("serializable"),
                },
            )
            .await
            .map_err(|error| {
                UpdateSubscriptionError::InsertDownloadError(subscription.clone(), error)
            })?;
        }
        Status::Unchanged => {
            debug!("process: {subscription} => unchanged ({commit_details})");
        }
    }

    Ok((status, commit_details))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum Status {
    New,
    Changed,
    Unchanged,
}
