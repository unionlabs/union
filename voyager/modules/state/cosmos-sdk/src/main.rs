use ibc_classic_spec::IbcClassic;
use voyager_sdk::plugin::StateModule;
use voyager_state_module_cosmos_sdk::Module;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    <Module as StateModule<IbcClassic>>::run().await;
}
