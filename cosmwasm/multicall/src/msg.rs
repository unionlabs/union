use cosmwasm_std::Coin;
use unionlabs_primitives::Bytes;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct InitMsg {}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Call {
    pub target: String,
    pub allow_failure: bool,
    pub calldata: Bytes,
    pub funds: Vec<Coin>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub enum ExecuteMsg {
    Multicall { calls: Vec<Call> },
}
