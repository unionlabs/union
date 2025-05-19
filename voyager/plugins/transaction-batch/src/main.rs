use voyager_message::Plugin;
use voyager_plugin_transaction_batch::Module;

#[tokio::main]
async fn main() {
    Module::run().await
}
