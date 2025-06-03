use voyager_client_module_bob::Module;
use voyager_sdk::plugin::ClientModule;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}
