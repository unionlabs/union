use aptos_rest_client::error::RestError;
use color_eyre::eyre::Report;

use super::api::IndexerError;

mod block_handle;
pub mod config;
mod context;
mod create_client_tracker;
mod fetcher_client;
// mod postgres;
mod provider;

impl From<RestError> for IndexerError {
    fn from(error: RestError) -> Self {
        Self::ProviderError(Report::from(error))
    }
}
