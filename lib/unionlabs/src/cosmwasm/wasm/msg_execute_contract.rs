use macros::model;

use crate::cosmos::base::coin::Coin;

/// `MsgExecuteContract` submits the given message data to a smart contract
#[model(proto(raw(protos::cosmwasm::wasm::v1::MsgExecuteContract), from))]
pub struct MsgExecuteContract {
    /// Sender is the that actor that signed the messages
    // TODO: bech32 encoded address
    pub sender: String,
    /// Contract is the address of the smart contract
    // TODO: bech32 encoded address
    pub contract: String,
    /// Msg json encoded message to be passed to the contract
    pub msg: Vec<u8>,
    /// Funds coins that are transferred to the contract on execution
    pub funds: Vec<Coin>,
}

impl From<MsgExecuteContract> for protos::cosmwasm::wasm::v1::MsgExecuteContract {
    fn from(value: MsgExecuteContract) -> Self {
        Self {
            sender: value.sender,
            contract: value.contract,
            msg: value.msg,
            funds: value.funds.into_iter().map(Into::into).collect(),
        }
    }
}
