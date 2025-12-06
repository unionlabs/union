use access_managed::Restricted;
use cosmwasm_std::{Addr, Binary, CosmosMsg, Uint64, Uint256};
use ibc_union_spec::{ChannelId, Packet, Timestamp};
use pausable::{
    WhenNotPaused,
    msg::{Pausable, PausableQuery},
};
use serde::{Deserialize, Serialize};
use ucs03_zkgm_token_minter_api::TokenMinterInitMsg;
use unionlabs_primitives::{Bytes, H256};
use upgradable::msg::Upgradable;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct InitMsg {
    pub config: Config,
    pub minter_init_params: TokenMinterInitParams,
    pub access_managed_init_msg: access_managed::InitMsg,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct MigrateMsg {
    pub access_managed_init_msg: access_managed::InitMsg,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub struct Config {
    /// The address to set as the owner of the minter.
    pub admin: Addr,
    /// The address of the `ibc-union` contract running on this chain.
    pub ibc_host: Addr,
    /// The code id of the `ucs03-zkgm-token-minter-api` implementor. This will be instantiated by `ucs03-zkgm` and used to mint and burn tokens.
    pub token_minter_code_id: u64,
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
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
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

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Handle IBC module messages from the IBC host.
    /// Used by the IBC host to notify the contract of IBC events.
    IbcUnionMsg(WhenNotPaused<ibc_union_msg::module::IbcUnionMsg>),
    /// Execute a zkgm packet.
    /// Can only be called by the contract itself during packet handling.
    InternalExecutePacket {
        caller: Addr,
        packet: Packet,
        relayer: Addr,
        relayer_msg: Bytes,
        intent: bool,
    },
    /// Write an acknowledgement for a zkgm packet.
    /// Can only be called by the contract itself after packet execution.
    InternalWriteAck {
        ack: Bytes,
    },
    InternalBatch {
        messages: Vec<CosmosMsg>,
    },
    #[serde(untagged)]
    Send(WhenNotPaused<SendMsg>),
    #[serde(untagged)]
    AccessManaged(access_managed::ExecuteMsg),
    #[serde(untagged)]
    Restricted(Restricted<RestrictedExecuteMsg>),
}

// NOTE: Defined as a separate enum for easier downstream consumption.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum SendMsg {
    /// Send a custom instruction across chains.
    /// Allows sending any zkgm instruction (forward, multiplex, batch, etc)
    /// with custom timeout and salt parameters.
    Send {
        channel_id: ChannelId,
        // keeping this field for easier migration for integrators, it is ignored
        timeout_height: Uint64,
        #[serde(with = "serde_utils::string")]
        #[cfg_attr(feature = "schemars", schemars(with = "Timestamp"))]
        timeout_timestamp: Timestamp,
        salt: H256,
        instruction: Bytes,
    },
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum RestrictedExecuteMsg {
    /// Migrate V1 to V2.
    /// Can only be called by admin.
    MigrateV1ToV2 {
        balance_migrations: Vec<V1ToV2Migration>,
        wrapped_migrations: Vec<V1ToV2WrappedMigration>,
    },
    /// Update a token bucket for rate limiting.
    SetBucketConfig {
        denom: String,
        capacity: Uint256,
        refill_rate: Uint256,
        /// If true, reset the currently available amount to the new maximum capacity.
        reset: bool,
    },
    MigrateMinter {
        // code id of the new token minter
        new_code_id: u64,
        // migrate message json that will directly be passed to migrate call
        // it will be the same as the `to_json_binary(&msg)`'s output
        msg: Binary,
    },
    SetRateLimitDisabled {
        // Whether to enable or disable rate limiting.
        rate_limit_disabled: bool,
    },
    UpdateDummyCodeId {
        dummy_code_id: u64,
    },
    UpdateCwAccountCodeId {
        cw_account_code_id: u64,
    },
    #[serde(untagged)]
    Pausable(Pausable),
    #[serde(untagged)]
    Upgradable(Upgradable),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct V1ToV2Migration {
    pub path: Uint256,
    pub channel_id: ChannelId,
    pub base_token: String,
    pub quote_token: Bytes,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct V1ToV2WrappedMigration {
    pub path: Uint256,
    pub channel_id: ChannelId,
    pub base_token: Bytes,
    pub quote_token: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum QueryMsg {
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
    GetBurnAddress {},
    #[serde(untagged)]
    AccessManaged(access_managed::QueryMsg),
    #[serde(untagged)]
    Pausable(PausableQuery),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct PredictWrappedTokenResponse {
    pub wrapped_token: String,
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use super::*;
    use crate::msg::SendMsg;

    #[test]
    fn serde() {
        let json_str = r#"{"send":{"channel_id":9,"timeout_height":"0","timeout_timestamp":"1744248392563000000","salt":"0xfd0ff5488c14b15c03d8958b25261c22a42b049894aeb1f33926a1f7ccadaf98","instruction":"0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000003e00000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000000014000000000000000000000000000000000000000000000000000000000000001a000000000000000000000000000000000000000000000000000000000000001e0000000000000000000000000000000000000000000000000000000000012d450000000000000000000000000000000000000000000000000000000000000024000000000000000000000000000000000000000000000000000000000000002800000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002c0000000000000000000000000000000000000000000000000000000000012d450000000000000000000000000000000000000000000000000000000000000002a307832633664373366343061636535313263343330343032646362646339343839343134333035303438000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014bd1b743615f903a630393f78234b4500fbe5691a000000000000000000000000000000000000000000000000000000000000000000000000000000000000003e62626e316e7a6e6c377372676d7478396a7565766e72617372327372736c717a7a6533303833673330773875673936663936726a7065757167386636757700000000000000000000000000000000000000000000000000000000000000000007756e6942544364000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000006756e6942544300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014717dA936440d811EbBF98515EB4a8Db2443BeFf5000000000000000000000000"}}"#;

        let expected_msg = SendMsg::Send {
            channel_id: ChannelId!(9),
            timeout_height: 0_u64.into(),
            timeout_timestamp: Timestamp::from_nanos(1744248392563000000),
            salt: hex!("fd0ff5488c14b15c03d8958b25261c22a42b049894aeb1f33926a1f7ccadaf98").into(),
            instruction: "0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000003e00000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000000014000000000000000000000000000000000000000000000000000000000000001a000000000000000000000000000000000000000000000000000000000000001e0000000000000000000000000000000000000000000000000000000000012d450000000000000000000000000000000000000000000000000000000000000024000000000000000000000000000000000000000000000000000000000000002800000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002c0000000000000000000000000000000000000000000000000000000000012d450000000000000000000000000000000000000000000000000000000000000002a307832633664373366343061636535313263343330343032646362646339343839343134333035303438000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014bd1b743615f903a630393f78234b4500fbe5691a000000000000000000000000000000000000000000000000000000000000000000000000000000000000003e62626e316e7a6e6c377372676d7478396a7565766e72617372327372736c717a7a6533303833673330773875673936663936726a7065757167386636757700000000000000000000000000000000000000000000000000000000000000000007756e6942544364000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000006756e6942544300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014717dA936440d811EbBF98515EB4a8Db2443BeFf5000000000000000000000000".parse().unwrap()
        };

        let msg = serde_json_wasm::from_str::<SendMsg>(json_str).unwrap();

        assert_eq!(msg, expected_msg);

        let msg = serde_json_wasm::from_str::<ExecuteMsg>(json_str).unwrap();

        assert_eq!(msg, ExecuteMsg::Send(WhenNotPaused::wrap(expected_msg)));
    }
}
