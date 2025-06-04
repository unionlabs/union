use ibc_union_spec::IbcUnion;
use voyager_sdk::plugin::StateModule;
use voyager_state_module_cosmos_sdk_union::Module;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    <Module as StateModule<IbcUnion>>::run().await;
}
