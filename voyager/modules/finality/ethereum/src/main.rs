use voyager_finality_module_ethereum::Module;
use voyager_sdk::plugin::FinalityModule;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}
