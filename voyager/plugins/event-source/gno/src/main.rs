use voyager_event_source_plugin_gno::Module;
use voyager_sdk::plugin::Plugin;

#[tokio::main]
async fn main() {
    Module::run().await
}
