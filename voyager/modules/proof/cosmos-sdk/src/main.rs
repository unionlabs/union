use voyager_proof_module_cosmos_sdk::Module;
use voyager_sdk::plugin::ProofModule;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await;
}
