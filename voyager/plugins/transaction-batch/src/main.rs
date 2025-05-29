use voyager_plugin_transaction_batch::Module;
use voyager_sdk::plugin::Plugin;

#[tokio::main]
async fn main() {
    Module::run().await
}
