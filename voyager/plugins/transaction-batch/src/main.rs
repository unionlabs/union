use voyager_message::Plugin;
use voyager_plugin_transaction_batch::Module;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}
