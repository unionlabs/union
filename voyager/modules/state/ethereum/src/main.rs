use voyager_sdk::plugin::StateModule;
use voyager_state_module_ethereum::Module;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}
