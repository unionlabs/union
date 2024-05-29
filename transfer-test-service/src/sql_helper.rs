use serde::{ Deserialize, Serialize };
use sqlx::{ FromRow, PgPool, Postgres, QueryBuilder };
use crate::config::PacketStatus;
use std::time::{ SystemTime, UNIX_EPOCH };

pub async fn insert_or_update_packet_status(
    pool: &PgPool,
    packet_status: PacketStatus
) -> Result<(), sqlx::Error> {
    sqlx
        ::query(
            r#"
        INSERT INTO packet_statuses (source_chain_id, target_chain_id, sequence_number, send_packet, recv_packet, write_ack, acknowledge_packet, last_update)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (source_chain_id, target_chain_id, sequence_number)
        DO UPDATE SET
            send_packet = EXCLUDED.send_packet,
            recv_packet = EXCLUDED.recv_packet,
            write_ack = EXCLUDED.write_ack,
            acknowledge_packet = EXCLUDED.acknowledge_packet,
            last_update = EXCLUDED.last_update
        "#
        )
        .bind(packet_status.source_chain_id)
        .bind(packet_status.target_chain_id)
        .bind(packet_status.sequence_number as i64) // Convert u64 to i64
        .bind(serde_json::to_value(&packet_status.send_packet).unwrap())
        .bind(serde_json::to_value(&packet_status.recv_packet).unwrap())
        .bind(serde_json::to_value(&packet_status.write_ack).unwrap())
        .bind(serde_json::to_value(&packet_status.acknowledge_packet).unwrap())
        .bind(packet_status.last_update)
        .execute(pool).await?;

    Ok(())
}
// pub async fn get_packet_statuses(
//     pool: &PgPool,
//     source_chain_id: i32,
//     target_chain_id: i32
// ) -> Result<Vec<PacketStatus>, sqlx::Error> {
//     let statuses = sqlx
//         ::query_as(
//             PacketStatus,
//             r#"
//         SELECT source_chain_id, target_chain_id, sequence_number, send_packet, recv_packet, write_ack, acknowledge_packet, last_update
//         FROM packet_statuses
//         WHERE source_chain_id = $1 AND target_chain_id = $2
//         "#,
//             source_chain_id,
//             target_chain_id
//         )
//         .fetch_all(pool).await?;

//     Ok(statuses)
// }

pub async fn get_packet_statuses(
    pool: &PgPool,
    source_chain_id: i32,
    target_chain_id: i32
) -> Result<Vec<PacketStatus>, sqlx::Error> {
    let statuses = sqlx
        ::query_as::<_, PacketStatus>(
            r#"
        SELECT * FROM packet_statuses
        WHERE source_chain_id = $1 AND target_chain_id = $2
        "#
        )
        .bind(source_chain_id)
        .bind(target_chain_id)
        .fetch_all(pool).await?;

    Ok(statuses)
}

pub async fn delete_packet_status(
    pool: &PgPool,
    source_chain_id: i32,
    target_chain_id: i32,
    sequence_number: i64
) -> Result<(), sqlx::Error> {
    sqlx
        ::query(
            r#"
        DELETE FROM packet_statuses
        WHERE source_chain_id = $1 AND target_chain_id = $2 AND sequence_number = $3
        "#
        )
        .bind(source_chain_id)
        .bind(target_chain_id)
        .bind(sequence_number)
        .execute(pool).await?;

    Ok(())
}
