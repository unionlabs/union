use cosmwasm_std::{Addr, CosmosMsg, Uint256, Uint64};
use ibc_union_spec::{ChannelId, Packet, Timestamp};
use serde::{Deserialize, Serialize};
use ucs03_zkgm_token_minter_api::TokenMinterInitMsg;
use unionlabs::primitives::{Bytes, H256};

use crate::com::CwTokenOrderV2;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct InitMsg {
    pub config: Config,
    pub minter_init_params: TokenMinterInitParams,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct Config {
    /// The address to set as the owner of the minter.
    pub admin: Addr,
    /// The address of the `ibc-union` contract running on this chain.
    pub ibc_host: Addr,
    /// The code id of the `ucs03-zkgm-token-minter-api` implementor. This will be instantiated by `ucs03-zkgm` and used to mint and burn tokens.
    pub token_minter_code_id: u64,
    /// The address that can update the rate limiters.
    pub rate_limit_admin: Addr,
    /// Addresses allowed to update token buckets for rate limiting.
    pub rate_limit_operators: Vec<Addr>,
    /// Enable or disable rate limiting.
    #[serde(default, skip_serializing_if = "core::ops::Not::not")]
    pub rate_limit_disabled: bool,
    /// The dummy code id for staking account deployment.
    #[serde(default)]
    pub dummy_code_id: u64,
    #[serde(default)]
    pub cw_account_code_id: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum TokenMinterInitParams {
    /// Instantiate `ucs03-zkgm` with a cw20 minter implementation.
    Cw20 {
        /// The code id of [`cw20-base`] to use for cw20 tokens. This will be threaded to the `cw20-token-minter` by `ucs03-zkgm`.
        ///
        /// [`cw20-base`]: https://github.com/CosmWasm/cw-plus/blob/main/packages/cw20/README.md#base
        // TODO: Should be NonZeroU64
        #[serde(alias = "cw20_base_code_id")]
        cw20_impl_code_id: u64,
        /// The code id of the dummy contract in order to get a contract address that does not depend on the code hash of `cw20_base`
        // TODO: Should be NonZeroU64
        dummy_code_id: u64,
    },
    /// Instantiate `ucs03-zkgm` with an osmosis token factory minter implementation.
    OsmosisTokenFactory {},
}

impl TokenMinterInitParams {
    /// Completes the init msg by using the runtime parameters
    pub fn into_msg(self, zkgm_admin: Addr) -> TokenMinterInitMsg {
        match self {
            TokenMinterInitParams::Cw20 {
                cw20_impl_code_id,
                dummy_code_id,
            } => TokenMinterInitMsg::Cw20 {
                cw20_impl_code_id,
                dummy_code_id,
                zkgm_admin,
            },
            TokenMinterInitParams::OsmosisTokenFactory {} => {
                TokenMinterInitMsg::OsmosisTokenFactory { zkgm_admin }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Update the set of rate limiters.
    SetRateLimitOperators {
        rate_limit_operators: Vec<Addr>,
    },
    /// Update a token bucket for rate limiting.
    SetBucketConfig {
        denom: String,
        capacity: Uint256,
        refill_rate: Uint256,
        // Indicates whether the currently available amount must be refilled to maximum capacity
        reset: bool,
    },
    /// Send a custom instruction across chains.
    /// Allows sending any zkgm instruction (forward, multiplex, batch, etc)
    /// with custom timeout and salt parameters.
    Send {
        channel_id: ChannelId,
        // keeping this field for easier migration for integrators, it is ignored
        timeout_height: Uint64,
        #[serde(with = "serde_utils::string")]
        timeout_timestamp: Timestamp,
        salt: H256,
        instruction: Bytes,
    },
    /// Handle IBC module messages from the IBC host.
    /// Used by the IBC host to notify the contract of IBC events.
    IbcUnionMsg(ibc_union_msg::module::IbcUnionMsg),
    /// Execute an Zkgm packet.
    /// Can only be called by the contract itself during packet handling.
    InternalExecutePacket {
        caller: Addr,
        packet: Packet,
        relayer: Addr,
        relayer_msg: Bytes,
        intent: bool,
    },
    /// Write an acknowledgement for an Zkgm packet.
    /// Can only be called by the contract itself after packet execution.
    InternalWriteAck {
        ack: Bytes,
    },
    InternalBatch {
        messages: Vec<CosmosMsg>,
    },
    /// Migrate V1 to V2 balances.
    /// Can only be called by admin.
    MigrateV1ToV2 {
        migrations: Vec<V1ToV2Migration>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum ZkgmMsg {
    OnZkgm {
        caller: Addr,
        path: Uint256,
        source_channel_id: ChannelId,
        destination_channel_id: ChannelId,
        sender: Bytes,
        message: Bytes,
        relayer: Addr,
        relayer_msg: Bytes,
    },
    OnIntentZkgm {
        caller: Addr,
        path: Uint256,
        source_channel_id: ChannelId,
        destination_channel_id: ChannelId,
        sender: Bytes,
        message: Bytes,
        market_maker: Addr,
        market_maker_msg: Bytes,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum SolverMsg {
    DoSolve {
        packet: Packet,
        order: CwTokenOrderV2,
        path: Uint256,
        caller: Addr,
        relayer: Addr,
        relayer_msg: Bytes,
        intent: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum SolverQuery {
    /// Returns unit if the contract is a solver.
    IsSolver,
    /// Whether the solver allows the relayer to fulfill the order on our behalf.
    AllowMarketMakers,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct MigrateMsg {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct V1ToV2Migration {
    pub channel_id: ChannelId,
    pub path: Uint256,
    pub base_token: String,
    pub quote_token: Bytes,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum QueryMsg {
    /// Calculate the stake account address
    PredictStakeAccount {
        /// The channel ID
        channel_id: ChannelId,
        /// The NFT ID
        token_id: Uint256,
    },
    /// Calculate the wrapped token denom
    PredictWrappedToken {
        path: String,
        /// Destination channel id
        channel_id: ChannelId,
        /// Base token denom
        token: Bytes,
    },
    /// Calculate the wrapped token denom using metadata image (V2)
    PredictWrappedTokenV2 {
        path: String,
        /// Destination channel id
        channel_id: ChannelId,
        /// Base token denom
        token: Bytes,
        /// Metadata image (hash)
        metadata_image: H256,
    },
    GetMinter {},
    GetTokenBucket {
        denom: String,
    },
    GetChannelBalance {
        channel_id: ChannelId,
        path: Uint256,
        denom: String,
    },
    GetChannelBalanceV2 {
        channel_id: ChannelId,
        path: Uint256,
        base_token: String,
        quote_token: Bytes,
    },
    GetConfig {},
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct PredictWrappedTokenResponse {
    pub wrapped_token: String,
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use super::*;

    #[test]
    fn serde() {
        let json_str = r#"{"send":{"channel_id":9,"timeout_height":"0","timeout_timestamp":"1744248392563000000","salt":"0xfd0ff5488c14b15c03d8958b25261c22a42b049894aeb1f33926a1f7ccadaf98","instruction":"0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000003e00000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000000014000000000000000000000000000000000000000000000000000000000000001a000000000000000000000000000000000000000000000000000000000000001e0000000000000000000000000000000000000000000000000000000000012d450000000000000000000000000000000000000000000000000000000000000024000000000000000000000000000000000000000000000000000000000000002800000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002c0000000000000000000000000000000000000000000000000000000000012d450000000000000000000000000000000000000000000000000000000000000002a307832633664373366343061636535313263343330343032646362646339343839343134333035303438000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014bd1b743615f903a630393f78234b4500fbe5691a000000000000000000000000000000000000000000000000000000000000000000000000000000000000003e62626e316e7a6e6c377372676d7478396a7565766e72617372327372736c717a7a6533303833673330773875673936663936726a7065757167386636757700000000000000000000000000000000000000000000000000000000000000000007756e6942544364000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000006756e6942544300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014717dA936440d811EbBF98515EB4a8Db2443BeFf5000000000000000000000000"}}"#;

        let msg = serde_json_wasm::from_str::<ExecuteMsg>(json_str).unwrap();

        assert_eq!(
            msg,
            ExecuteMsg::Send {
                channel_id: ChannelId!(9),
                timeout_height: 0_u64.into(),
                timeout_timestamp: Timestamp::from_nanos(1744248392563000000),
                salt: hex!("fd0ff5488c14b15c03d8958b25261c22a42b049894aeb1f33926a1f7ccadaf98").into(),
                instruction: "0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000003e00000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000000014000000000000000000000000000000000000000000000000000000000000001a000000000000000000000000000000000000000000000000000000000000001e0000000000000000000000000000000000000000000000000000000000012d450000000000000000000000000000000000000000000000000000000000000024000000000000000000000000000000000000000000000000000000000000002800000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002c0000000000000000000000000000000000000000000000000000000000012d450000000000000000000000000000000000000000000000000000000000000002a307832633664373366343061636535313263343330343032646362646339343839343134333035303438000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014bd1b743615f903a630393f78234b4500fbe5691a000000000000000000000000000000000000000000000000000000000000000000000000000000000000003e62626e316e7a6e6c377372676d7478396a7565766e72617372327372736c717a7a6533303833673330773875673936663936726a7065757167386636757700000000000000000000000000000000000000000000000000000000000000000007756e6942544364000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000006756e6942544300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014717dA936440d811EbBF98515EB4a8Db2443BeFf5000000000000000000000000".parse().unwrap()
            }
        );
    }
}
