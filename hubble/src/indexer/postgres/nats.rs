use std::collections::HashMap;

use async_nats::HeaderMap;
use bytes::Bytes;
use sqlx::Postgres;

use crate::indexer::nats::Message;

pub async fn schedule(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    subject: &str,
    data: bytes::Bytes,
    headers: &HashMap<String, Vec<String>>,
) -> sqlx::Result<i64> {
    let record = sqlx::query!(
        "
        INSERT INTO hubble.out(subject, data, headers)
        VALUES ($1, $2, $3)
        RETURNING id
        ",
        subject,
        data.as_ref(),
        serde_json::to_value(headers).expect("headers should be json serializable"),
    )
    .fetch_one(tx.as_mut())
    .await?;

    Ok(record.id)
}

pub async fn next_to_publish(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    subject: &str,
    batch_size: usize,
) -> sqlx::Result<Vec<Message>> {
    let raw_rows = sqlx::query!(
        r#"
        WITH to_publish AS (
            SELECT id
            FROM hubble.out
            WHERE subject = $1
            ORDER BY id
            FOR UPDATE SKIP LOCKED
            LIMIT $2
        ),
        deleted AS (
            DELETE FROM hubble.out
            USING to_publish
            WHERE hubble.out.id = to_publish.id
            RETURNING hubble.out.id, hubble.out.subject, hubble.out.headers, hubble.out.data
        )
        SELECT id, subject, headers, data
        FROM deleted
        ORDER BY id;
        "#,
        subject,
        i64::try_from(batch_size).expect("batch-size < i64 max"),
    )
    .fetch_all(tx.as_mut())
    .await?;

    let result: Vec<Message> = raw_rows
        .into_iter()
        .map(|row| {
            let id: i64 = row.id;
            let subject: String = row.subject;
            let data: Bytes = row.data.into();

            let raw_headers: HashMap<String, Vec<String>> = serde_json::from_value(row.headers)
                .map_err(|e| sqlx::Error::ColumnDecode {
                    index: "headers".into(),
                    source: Box::new(e),
                })?;

            let mut headers = HeaderMap::new();
            for (key, values) in raw_headers {
                for value in values {
                    headers.insert(key.clone(), value);
                }
            }

            Ok(Message::new(id, subject, headers, data))
        })
        .collect::<Result<_, sqlx::Error>>()?;

    Ok(result)
}
