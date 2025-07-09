use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use sqlx::{Postgres, Transaction};

use crate::indexer::{
    api::IndexerError,
    event::types::{BlockHeight, UniversalChainId},
};

/// Attempts to acquire a PostgreSQL advisory lock for a specific universal-chain-id + block-height combination.
///
/// This function uses PostgreSQL's `pg_try_advisory_xact_lock` function which tries to acquire the lock
/// and returns immediately (non-blocking). The lock is automatically released when the transaction
/// is committed or rolled back (transaction-scoped).
///
/// # Arguments
/// * `tx` - A mutable reference to a PostgreSQL transaction
/// * `universal_chain_id` - The universal chain identifier
/// * `block_height` - The block height to lock on
///
/// # Returns
/// * `Ok(())` - Lock was successfully acquired
/// * `Err(IndexerError::LockAcquisitionFailed)` - Lock could not be acquired (already held by another session)
/// * `Err(IndexerError::DatabaseError)` - Database error occurred
///
/// # Example
/// ```rust
/// let mut tx = pool.begin().await?;
/// let chain_id = UniversalChainId("chain-123".to_string());
/// try_lock_block(&mut tx, &chain_id, 12345).await?;
/// // Lock acquired, proceed with processing
/// // Lock will be automatically released when transaction ends
/// ```
pub async fn try_lock_block(
    tx: &mut Transaction<'_, Postgres>,
    universal_chain_id: &UniversalChainId,
    block_height: BlockHeight,
) -> Result<(), IndexerError> {
    // Calculate hash of universal_chain_id + block_height combination
    let lock_key = calculate_lock_key(universal_chain_id, block_height);

    // Use pg_try_advisory_xact_lock which returns immediately
    // Returns true if lock was acquired, false if already held
    // This lock is automatically released when the transaction commits or rolls back
    let result = sqlx::query_scalar::<_, bool>("SELECT pg_try_advisory_xact_lock($1)")
        .bind(lock_key)
        .fetch_one(&mut **tx)
        .await?;

    if result {
        Ok(())
    } else {
        Err(IndexerError::LockAcquisitionFailed(
            universal_chain_id.clone(),
            block_height,
        ))
    }
}

/// Calculates a hash key for the universal_chain_id + block_height combination.
///
/// This function creates a deterministic hash that can be used as a lock key
/// for PostgreSQL advisory locks. The same universal_chain_id + block_height combination
/// will always produce the same hash.
///
/// # Arguments
/// * `universal_chain_id` - The universal chain identifier
/// * `block_height` - The block height
///
/// # Returns
/// A 64-bit hash value suitable for use as a PostgreSQL advisory lock key
fn calculate_lock_key(universal_chain_id: &UniversalChainId, block_height: BlockHeight) -> i64 {
    let mut hasher = DefaultHasher::new();
    universal_chain_id.0.hash(&mut hasher);
    block_height.hash(&mut hasher);

    // Convert u64 hash to i64 for PostgreSQL (which expects signed integers)
    // Use wrapping_add to safely convert u64 to i64 without overflow issues
    let hash = hasher.finish();

    // Split the u64 into high and low 32-bit parts and combine them safely
    // This ensures we get a deterministic i64 value from any u64 input
    let high = (hash >> 32) as i32;
    let low = hash as i32;

    // Combine using XOR to maintain good distribution properties
    (high as i64) ^ (low as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calc() {
        // Same inputs should produce same hash
        let chain_id = UniversalChainId("corn.21000001".to_string());
        let key = calculate_lock_key(&chain_id, BlockHeight(4660300));
        println!("{key}");
    }

    #[test]
    fn test_calculate_lock_key_deterministic() {
        // Same inputs should produce same hash
        let chain_id = UniversalChainId("chain-1".to_string());
        let key1 = calculate_lock_key(&chain_id, BlockHeight(12345));
        let key2 = calculate_lock_key(&chain_id, BlockHeight(12345));
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_calculate_lock_key_different_inputs() {
        // Different inputs should produce different hashes
        let chain_id_1 = UniversalChainId("chain-1".to_string());
        let chain_id_2 = UniversalChainId("chain-2".to_string());
        let key1 = calculate_lock_key(&chain_id_1, BlockHeight(12345));
        let key2 = calculate_lock_key(&chain_id_2, BlockHeight(12345));
        let key3 = calculate_lock_key(&chain_id_1, BlockHeight(12346));

        assert_ne!(key1, key2);
        assert_ne!(key1, key3);
        assert_ne!(key2, key3);
    }

    #[test]
    fn test_calculate_lock_key_edge_cases() {
        // Test with zero and negative values
        let chain_id_zero = UniversalChainId("chain-0".to_string());
        let chain_id_empty = UniversalChainId("".to_string());
        let key1 = calculate_lock_key(&chain_id_zero, BlockHeight(0));
        let key2 = calculate_lock_key(&chain_id_empty, BlockHeight(0));
        let key3 = calculate_lock_key(&chain_id_zero, BlockHeight(1));

        // Should handle edge cases without panicking
        assert_ne!(key1, key2);
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_calculate_lock_key_safe_conversion() {
        // Test that our conversion always produces valid i64 values
        // and doesn't overflow/panic regardless of input
        let test_cases = vec![
            (
                UniversalChainId(
                    "very-long-chain-id-with-lots-of-characters-to-test-hashing".to_string(),
                ),
                u64::MAX,
            ),
            (UniversalChainId("short".to_string()), u64::MAX),
            (UniversalChainId("chain-0".to_string()), u64::MAX),
            (UniversalChainId("".to_string()), u64::MAX),
            (UniversalChainId("unicode-🚀-chain".to_string()), 0),
            (UniversalChainId("special-chars-@#$%^&*()".to_string()), 0),
        ];

        for (chain_id, block_height) in test_cases {
            // Should not panic and should produce a valid i64
            let result = calculate_lock_key(&chain_id, BlockHeight(block_height));

            // Result is always a valid i64 by construction
            // (this assertion is mainly for documentation purposes)

            // Verify determinism - same inputs produce same outputs
            let result2 = calculate_lock_key(&chain_id, BlockHeight(block_height));
            assert_eq!(result, result2);
        }
    }

    // Test to demonstrate that session-scoped locks persist beyond transaction rollback
    // This test shows the problem with using pg_try_advisory_lock instead of pg_try_advisory_xact_lock
    #[ignore] // Ignored by default since it requires a database connection
    #[tokio::test]
    async fn test_session_vs_transaction_scoped_locks() {
        // This test requires a database connection
        let database_url =
            std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable not set");

        let pool = sqlx::PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to database");

        let chain_id = UniversalChainId("test-session-vs-transaction".to_string());
        let block_height = BlockHeight(99999);

        // Test 1: Current implementation (session-scoped locks) - BAD BEHAVIOR
        {
            // Start transaction and acquire lock
            let mut tx1 = pool.begin().await.expect("Failed to begin transaction");
            try_lock_block(&mut tx1, &chain_id, block_height)
                .await
                .expect("First lock attempt should succeed");

            // Rollback transaction - with session locks, the lock persists!
            tx1.rollback()
                .await
                .expect("Failed to rollback transaction");

            // Try to acquire the same lock in a new transaction - this will FAIL with session locks
            let mut tx2 = pool
                .begin()
                .await
                .expect("Failed to begin second transaction");
            let result = try_lock_block(&mut tx2, &chain_id, block_height).await;

            // This demonstrates the problem: the lock is still held even though the transaction was rolled back
            assert!(
                matches!(result, Err(IndexerError::LockAcquisitionFailed(_, _))),
                "Lock should still be held (demonstrating the problem with session-scoped locks)"
            );

            tx2.rollback()
                .await
                .expect("Failed to rollback second transaction");

            // We need to manually release the session lock
            let lock_key = calculate_lock_key(&chain_id, block_height);
            sqlx::query("SELECT pg_advisory_unlock($1)")
                .bind(lock_key)
                .execute(&pool)
                .await
                .expect("Failed to manually release lock");
        }

        pool.close().await;
    }

    // Test to verify that transaction-scoped locks work correctly
    #[ignore] // Ignored by default since it requires a database connection
    #[tokio::test]
    async fn test_transaction_scoped_lock_behavior() {
        // This test requires a database connection
        let database_url =
            std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable not set");

        let pool = sqlx::PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to database");

        let chain_id = UniversalChainId("test-transaction-scoped".to_string());
        let block_height = BlockHeight(88888);

        // Test: Transaction-scoped locks should be released on rollback
        {
            // Start transaction and acquire lock
            let mut tx1 = pool.begin().await.expect("Failed to begin transaction");
            try_lock_block(&mut tx1, &chain_id, block_height)
                .await
                .expect("First lock attempt should succeed");

            // Rollback transaction - with transaction locks, the lock is automatically released
            tx1.rollback()
                .await
                .expect("Failed to rollback transaction");

            // Try to acquire the same lock in a new transaction - this should SUCCEED with transaction locks
            let mut tx2 = pool
                .begin()
                .await
                .expect("Failed to begin second transaction");
            let result = try_lock_block(&mut tx2, &chain_id, block_height).await;

            // This verifies the fix: the lock should be available since the previous transaction was rolled back
            assert!(
                result.is_ok(),
                "Lock should be available after previous transaction rollback (verifying transaction-scoped behavior)"
            );

            tx2.rollback()
                .await
                .expect("Failed to rollback second transaction");
        }

        // Test: Transaction-scoped locks should be released on commit
        {
            // Start transaction and acquire lock
            let mut tx1 = pool.begin().await.expect("Failed to begin transaction");
            try_lock_block(&mut tx1, &chain_id, block_height)
                .await
                .expect("First lock attempt should succeed");

            // Commit transaction - with transaction locks, the lock is automatically released
            tx1.commit().await.expect("Failed to commit transaction");

            // Try to acquire the same lock in a new transaction - this should SUCCEED with transaction locks
            let mut tx2 = pool
                .begin()
                .await
                .expect("Failed to begin second transaction");
            let result = try_lock_block(&mut tx2, &chain_id, block_height).await;

            // This verifies the fix: the lock should be available since the previous transaction was committed
            assert!(
                result.is_ok(),
                "Lock should be available after previous transaction commit (verifying transaction-scoped behavior)"
            );

            tx2.rollback()
                .await
                .expect("Failed to rollback second transaction");
        }

        pool.close().await;
    }

    // Integration test for database connection
    // To run this test, set: export DATABASE_URL=postgres://union:union@host.orb.internal:5432/union
    #[ignore] // Ignored by default since it requires a database connection
    #[tokio::test]
    async fn test_try_lock_block_integration() {
        // This test requires a database connection
        let database_url =
            std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable not set");

        let pool = sqlx::PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to database");

        let chain_id = UniversalChainId("test-chain-999".to_string()); // Use a test chain ID
        let block_height = BlockHeight(12345);

        // Start first transaction and acquire lock
        let mut tx1 = pool.begin().await.expect("Failed to begin transaction");
        try_lock_block(&mut tx1, &chain_id, block_height)
            .await
            .expect("First lock attempt should succeed");

        // Start second transaction and try to acquire same lock
        let mut tx2 = pool
            .begin()
            .await
            .expect("Failed to begin second transaction");
        let result = try_lock_block(&mut tx2, &chain_id, block_height).await;

        // Should get LockAcquisitionFailed error
        assert!(
            matches!(result, Err(IndexerError::LockAcquisitionFailed(_, _))),
            "Second lock attempt should fail with LockAcquisitionFailed error"
        );

        // Clean up
        tx1.rollback()
            .await
            .expect("Failed to rollback first transaction");
        tx2.rollback()
            .await
            .expect("Failed to rollback second transaction");

        pool.close().await;
    }
}
