//! The Ethereum indexing algorithm is relatively complicated compared to the Cosmos indexer
//! due to the fact that Ethereum does not have single-slot finality, hence we must handle
//! reorgs.
//!
//! # Algorithm
//!
//! 1. Index head eagerly.
//! 2. Index from the latest finalized block backwards to the last finalized block in the DB.
//! 3. Remove all data associated with uncle blocks.
//!
//! # Parsing
//!
//! Since EthAbi is not self-describing, we need a separate parsing step to transform the data
//! for consumption. This indexer does not handle parsing, just creating an up-to-date view of logs, transactions and blocks.
//!
use ethers_providers::{Http, Middleware, Provider};

pub struct EagerIndexer {
    provider: Provider<Http>,
}

impl EagerIndexer {
    fn new(provider: Provider<Http>) -> Self {
        Self { provider }
    }
}

impl EagerIndexer {
    async fn index_from(&self, number: u64) {
        let mut current = number;
        loop {
            let block = self.provider.get_block(current).await.unwrap();
            let receipts = self.provider.get_block_receipts(current);
            current += 1;
            dbg!(block);
            dbg!(receipts);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    fn test_works() {
        let provider = Provider::<Http>::try_from("https://eth.llamarpc.com").unwrap();
        let indexer = EagerIndexer::new(provider);
        indexer.index_from(0).await
    }
}
