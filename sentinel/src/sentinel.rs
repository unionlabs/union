use async_trait::async_trait;

use crate::{args::RunCmd, report::Report};
/// A sentinel is a test scenario that is run against different production-like environments,
/// such as a local devnet, testnet, or even mainnet.
///
/// Each Sentinel should require as little configuration as possible. If possible, fetch configurations
/// from the graphql API, secrets from a 1Password, as if this is a pure user interaction.
#[async_trait]
pub trait Sentinel: Send + Sync {
    fn name(&self) -> &str;

    fn description(&self) -> &str;

    /// Perform any necessary test set up. This may be used to fetch data or perform one-time setup. Note that
    /// as little IO as possible should be performed here. Querying for balances, RPC nodes through graphql.union.build,
    /// or other operations that the UI might normally perform should occur in `Sentinel::run`.
    async fn setup(&mut self) {}

    async fn run(&mut self);

    async fn teardown(&mut self) {}

    fn report(&mut self) -> Report;

    fn configure(&mut self, args: &RunCmd);
}
