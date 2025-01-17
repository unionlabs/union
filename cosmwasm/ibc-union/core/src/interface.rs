pub const UNION_IBC: &str = "union:ibc-union";

#[cw_orch::interface(
    crate::msg::InstantiateMsg,
    crate::msg::ExecuteMsg,
    crate::msg::QueryMsg,
    crate::msg::MigrateMsg,
    id = UNION_IBC
)]
pub struct UnionIbc<Chain>;

#[cfg(not(target_arch = "wasm32"))]
use cw_orch::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
impl<Chain: CwEnv> Uploadable for UnionIbc<Chain> {
    fn wasm(_chain_info: &ChainInfoOwned) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("ibc_union")
            .unwrap()
    }
    fn wrapper() -> <Mock as TxHandler>::ContractSource {
        Box::new(
            ContractWrapper::new(
                crate::contract::execute,
                crate::contract::instantiate,
                crate::contract::query,
            )
        )
    }
}
