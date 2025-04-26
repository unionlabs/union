use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;

#[cw_serde]
pub enum ExecuteMsg {
    ChangeTokenOwner {
        denom: String,
        new_owner: Addr,
    },

    #[serde(untagged)]
    ZkgmExecuteMsg(ucs03_zkgm_token_minter_api::ExecuteMsg),
}
