use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128, Uint256};
use ibc_union_spec::types::Packet;
use unionlabs::primitives::{Bytes, H256};

#[cw_serde]
pub struct InitMsg {
    pub config: Config,
    pub minter_init_msg: TokenMinterInitMsg,
}

#[cw_serde]
pub struct Config {
    /// The address to set as the owner of the minter.
    pub admin: Addr,
    /// The address of the `ibc-union` contract running on this chain.
    pub ibc_host: Addr,
    /// The code id of the `ucs03-zkgm-token-minter-api` implementor. This will be instantiated by `ucs03-zkgm` and used to mint and burn tokens.
    pub token_minter_code_id: u64,
}

#[cw_serde]
pub enum TokenMinterInitMsg {
    /// Instantiate `ucs03-zkgm` with a cw20 minter implementation.
    Cw20 {
        /// The code id of [`cw20-base`] to use for cw20 tokens. This will be threaded to the `cw20-token-minter` by `ucs03-zkgm`.
        ///
        /// [`cw20-base`]: https://github.com/CosmWasm/cw-plus/blob/main/packages/cw20/README.md#base
        cw20_base_code_id: u64,
        /// The code id of the dummy contract in order to get a contract address that does not depend on the code hash of `cw20_base`
        dummy_code_id: u64,
    },
    /// Instantiate `ucs03-zkgm` with a native tokenfactory minter implementation.
    Native,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Transfer tokens across chains with optional quote token and amount.
    /// Used for basic token transfers where the receiver gets quote_amount of quote_token
    /// in exchange for base_amount of base_token.
    Transfer {
        channel_id: u32,
        receiver: Bytes,
        base_token: String,
        base_amount: Uint128,
        quote_token: Bytes,
        quote_amount: Uint256,
        timeout_height: u64,
        timeout_timestamp: u64,
        salt: H256,
    },
    /// Send a custom instruction across chains.
    /// Allows sending any zkgm instruction (forward, multiplex, batch, etc)
    /// with custom timeout and salt parameters.
    Send {
        channel_id: u32,
        timeout_height: u64,
        timeout_timestamp: u64,
        salt: H256,
        instruction: Bytes,
    },
    /// Handle IBC module messages from the IBC host.
    /// Used by the IBC host to notify the contract of IBC events.
    IbcUnionMsg(ibc_union_msg::module::IbcUnionMsg),
    /// Execute an Zkgm packet.
    /// Can only be called by the contract itself during packet handling.
    InternalExecutePacket {
        packet: Packet,
        relayer: Addr,
        relayer_msg: Bytes,
    },
    /// Write an acknowledgement for an Zkgm packet.
    /// Can only be called by the contract itself after packet execution.
    InternalWriteAck { ack: Bytes },
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EurekaMsg {
    OnZkgm {
        path: Uint256,
        source_channel_id: u32,
        destination_channel_id: u32,
        sender: Bytes,
        message: Bytes,
    },
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub enum QueryMsg {
    /// Calculate the wrapped token denom
    PredictWrappedToken {
        path: String,
        /// Destination channel id
        channel: u32,
        /// Base token denom
        token: Bytes,
    },
}

#[cw_serde]
pub struct PredictWrappedTokenResponse {
    pub wrapped_token: Bytes,
}
