pub const IBC_UNION: &str = "union:ibc-union";

#[cw_orch::interface(
    ibc_union_msg::msg::InitMsg,
    ibc_union_msg::msg::ExecuteMsg,
    ibc_union_msg::query::QueryMsg,
    crate::contract::IbcUnionMigrateMsg,
    id = IBC_UNION
)]
pub struct IbcUnion<Chain>;

#[cfg(not(target_arch = "wasm32"))]
use cw_orch::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
impl<Chain: CwEnv> Uploadable for IbcUnion<Chain> {
    fn wasm(_chain_info: &ChainInfoOwned) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("ibc_union")
            .unwrap()
    }
    fn wrapper() -> <Mock as TxHandler>::ContractSource {
        Box::new(ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        ))
    }
}
