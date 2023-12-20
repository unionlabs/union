use core::fmt;
use derivative::Derivative;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::binary::Binary;
use crate::coin::Coin;
use crate::errors::StdResult;
#[cfg(feature = "stargate")]
use crate::ibc::IbcMsg;
use crate::serde::to_json_binary;
#[cfg(all(feature = "stargate", feature = "cosmwasm_1_2"))]
use crate::Decimal;

use super::Empty;

/// Like CustomQuery for better type clarity.
/// Also makes it shorter to use as a trait bound.
pub trait CustomMsg: Serialize + Clone + fmt::Debug + PartialEq + JsonSchema {}

impl CustomMsg for Empty {}

#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
// See https://github.com/serde-rs/serde/issues/1296 why we cannot add De-Serialize trait bounds to T
pub enum CosmosMsg<T = Empty> {
    Bank(BankMsg),
    // by default we use RawMsg, but a contract can override that
    // to call into more app-specific code (whatever they define)
    Custom(T),
    #[cfg(feature = "staking")]
    Staking(StakingMsg),
    #[cfg(feature = "staking")]
    Distribution(DistributionMsg),
    /// A Stargate message encoded the same way as a protobuf [Any](https://github.com/protocolbuffers/protobuf/blob/master/src/google/protobuf/any.proto).
    /// This is the same structure as messages in `TxBody` from [ADR-020](https://github.com/cosmos/cosmos-sdk/blob/master/docs/architecture/adr-020-protobuf-transaction-encoding.md)
    #[cfg(feature = "stargate")]
    Stargate {
        type_url: String,
        value: Binary,
    },
    #[cfg(feature = "stargate")]
    Ibc(IbcMsg),
    Wasm(WasmMsg),
    #[cfg(feature = "stargate")]
    Gov(GovMsg),
}

/// The message types of the bank module.
///
/// See https://github.com/cosmos/cosmos-sdk/blob/v0.40.0/proto/cosmos/bank/v1beta1/tx.proto
#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BankMsg {
    /// Sends native tokens from the contract to the given address.
    ///
    /// This is translated to a [MsgSend](https://github.com/cosmos/cosmos-sdk/blob/v0.40.0/proto/cosmos/bank/v1beta1/tx.proto#L19-L28).
    /// `from_address` is automatically filled with the current contract's address.
    Send {
        to_address: String,
        amount: Vec<Coin>,
    },
    /// This will burn the given coins from the contract's account.
    /// There is no Cosmos SDK message that performs this, but it can be done by calling the bank keeper.
    /// Important if a contract controls significant token supply that must be retired.
    Burn { amount: Vec<Coin> },
}

/// The message types of the staking module.
///
/// See https://github.com/cosmos/cosmos-sdk/blob/v0.40.0/proto/cosmos/staking/v1beta1/tx.proto
#[cfg(feature = "staking")]
#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum StakingMsg {
    /// This is translated to a [MsgDelegate](https://github.com/cosmos/cosmos-sdk/blob/v0.40.0/proto/cosmos/staking/v1beta1/tx.proto#L81-L90).
    /// `delegator_address` is automatically filled with the current contract's address.
    Delegate { validator: String, amount: Coin },
    /// This is translated to a [MsgUndelegate](https://github.com/cosmos/cosmos-sdk/blob/v0.40.0/proto/cosmos/staking/v1beta1/tx.proto#L112-L121).
    /// `delegator_address` is automatically filled with the current contract's address.
    Undelegate { validator: String, amount: Coin },
    /// This is translated to a [MsgBeginRedelegate](https://github.com/cosmos/cosmos-sdk/blob/v0.40.0/proto/cosmos/staking/v1beta1/tx.proto#L95-L105).
    /// `delegator_address` is automatically filled with the current contract's address.
    Redelegate {
        src_validator: String,
        dst_validator: String,
        amount: Coin,
    },
}

/// The message types of the distribution module.
///
/// See https://github.com/cosmos/cosmos-sdk/blob/v0.42.4/proto/cosmos/distribution/v1beta1/tx.proto
#[cfg(feature = "staking")]
#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DistributionMsg {
    /// This is translated to a [MsgSetWithdrawAddress](https://github.com/cosmos/cosmos-sdk/blob/v0.42.4/proto/cosmos/distribution/v1beta1/tx.proto#L29-L37).
    /// `delegator_address` is automatically filled with the current contract's address.
    SetWithdrawAddress {
        /// The `withdraw_address`
        address: String,
    },
    /// This is translated to a [[MsgWithdrawDelegatorReward](https://github.com/cosmos/cosmos-sdk/blob/v0.42.4/proto/cosmos/distribution/v1beta1/tx.proto#L42-L50).
    /// `delegator_address` is automatically filled with the current contract's address.
    WithdrawDelegatorReward {
        /// The `validator_address`
        validator: String,
    },
    /// This is translated to a [[MsgFundCommunityPool](https://github.com/cosmos/cosmos-sdk/blob/v0.42.4/proto/cosmos/distribution/v1beta1/tx.proto#LL69C1-L76C2).
    /// `depositor` is automatically filled with the current contract's address.
    #[cfg(feature = "cosmwasm_1_3")]
    FundCommunityPool {
        /// The amount to spend
        amount: Vec<Coin>,
    },
}

fn binary_to_string(data: &Binary, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
    match core::str::from_utf8(data.as_slice()) {
        Ok(s) => fmt.write_str(s),
        Err(_) => fmt::Debug::fmt(data, fmt),
    }
}

/// The message types of the wasm module.
///
/// See https://github.com/CosmWasm/wasmd/blob/v0.14.0/x/wasm/internal/types/tx.proto
#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, Derivative, PartialEq, Eq, JsonSchema)]
#[derivative(Debug)]
#[serde(rename_all = "snake_case")]
pub enum WasmMsg {
    /// Dispatches a call to another contract at a known address (with known ABI).
    ///
    /// This is translated to a [MsgExecuteContract](https://github.com/CosmWasm/wasmd/blob/v0.14.0/x/wasm/internal/types/tx.proto#L68-L78).
    /// `sender` is automatically filled with the current contract's address.
    Execute {
        contract_addr: String,
        /// msg is the json-encoded ExecuteMsg struct (as raw Binary)
        #[derivative(Debug(format_with = "binary_to_string"))]
        msg: Binary,
        funds: Vec<Coin>,
    },
    /// Instantiates a new contracts from previously uploaded Wasm code.
    ///
    /// The contract address is non-predictable. But it is guaranteed that
    /// when emitting the same Instantiate message multiple times,
    /// multiple instances on different addresses will be generated. See also
    /// Instantiate2.
    ///
    /// This is translated to a [MsgInstantiateContract](https://github.com/CosmWasm/wasmd/blob/v0.29.2/proto/cosmwasm/wasm/v1/tx.proto#L53-L71).
    /// `sender` is automatically filled with the current contract's address.
    Instantiate {
        admin: Option<String>,
        code_id: u64,
        /// msg is the JSON-encoded InstantiateMsg struct (as raw Binary)
        #[derivative(Debug(format_with = "binary_to_string"))]
        msg: Binary,
        funds: Vec<Coin>,
        /// A human-readable label for the contract.
        ///
        /// Valid values should:
        /// - not be empty
        /// - not be bigger than 128 bytes (or some chain-specific limit)
        /// - not start / end with whitespace
        label: String,
    },
    /// Instantiates a new contracts from previously uploaded Wasm code
    /// using a predictable address derivation algorithm implemented in
    /// [`cosmwasm_std::instantiate2_address`].
    ///
    /// This is translated to a [MsgInstantiateContract2](https://github.com/CosmWasm/wasmd/blob/v0.29.2/proto/cosmwasm/wasm/v1/tx.proto#L73-L96).
    /// `sender` is automatically filled with the current contract's address.
    /// `fix_msg` is automatically set to false.
    #[cfg(feature = "cosmwasm_1_2")]
    Instantiate2 {
        admin: Option<String>,
        code_id: u64,
        /// A human-readable label for the contract.
        ///
        /// Valid values should:
        /// - not be empty
        /// - not be bigger than 128 bytes (or some chain-specific limit)
        /// - not start / end with whitespace
        label: String,
        /// msg is the JSON-encoded InstantiateMsg struct (as raw Binary)
        #[derivative(Debug(format_with = "binary_to_string"))]
        msg: Binary,
        funds: Vec<Coin>,
        salt: Binary,
    },
    /// Migrates a given contracts to use new wasm code. Passes a MigrateMsg to allow us to
    /// customize behavior.
    ///
    /// Only the contract admin (as defined in wasmd), if any, is able to make this call.
    ///
    /// This is translated to a [MsgMigrateContract](https://github.com/CosmWasm/wasmd/blob/v0.14.0/x/wasm/internal/types/tx.proto#L86-L96).
    /// `sender` is automatically filled with the current contract's address.
    Migrate {
        contract_addr: String,
        /// the code_id of the new logic to place in the given contract
        new_code_id: u64,
        /// msg is the json-encoded MigrateMsg struct that will be passed to the new code
        #[derivative(Debug(format_with = "binary_to_string"))]
        msg: Binary,
    },
    /// Sets a new admin (for migrate) on the given contract.
    /// Fails if this contract is not currently admin of the target contract.
    UpdateAdmin {
        contract_addr: String,
        admin: String,
    },
    /// Clears the admin on the given contract, so no more migration possible.
    /// Fails if this contract is not currently admin of the target contract.
    ClearAdmin { contract_addr: String },
}

/// This message type allows the contract interact with the [x/gov] module in order
/// to cast votes.
///
/// [x/gov]: https://github.com/cosmos/cosmos-sdk/tree/v0.45.12/x/gov
///
/// ## Examples
///
/// Cast a simple vote:
///
/// ```
/// # use cosmwasm_std::{
/// #     HexBinary,
/// #     Storage, Api, Querier, DepsMut, Deps, entry_point, Env, StdError, MessageInfo,
/// #     Response, QueryResponse,
/// # };
/// # type ExecuteMsg = ();
/// use cosmwasm_std::{GovMsg, VoteOption};
///
/// #[entry_point]
/// pub fn execute(
///     deps: DepsMut,
///     env: Env,
///     info: MessageInfo,
///     msg: ExecuteMsg,
/// ) -> Result<Response, StdError> {
///     // ...
///     Ok(Response::new().add_message(GovMsg::Vote {
///         proposal_id: 4,
///         vote: VoteOption::Yes,
///     }))
/// }
/// ```
///
/// Cast a weighted vote:
///
/// ```
/// # use cosmwasm_std::{
/// #     HexBinary,
/// #     Storage, Api, Querier, DepsMut, Deps, entry_point, Env, StdError, MessageInfo,
/// #     Response, QueryResponse,
/// # };
/// # type ExecuteMsg = ();
/// # #[cfg(feature = "cosmwasm_1_2")]
/// use cosmwasm_std::{Decimal, GovMsg, VoteOption, WeightedVoteOption};
///
/// # #[cfg(feature = "cosmwasm_1_2")]
/// #[entry_point]
/// pub fn execute(
///     deps: DepsMut,
///     env: Env,
///     info: MessageInfo,
///     msg: ExecuteMsg,
/// ) -> Result<Response, StdError> {
///     // ...
///     Ok(Response::new().add_message(GovMsg::VoteWeighted {
///         proposal_id: 4,
///         options: vec![
///             WeightedVoteOption {
///                 option: VoteOption::Yes,
///                 weight: Decimal::percent(65),
///             },
///             WeightedVoteOption {
///                 option: VoteOption::Abstain,
///                 weight: Decimal::percent(35),
///             },
///         ],
///     }))
/// }
/// ```
#[cfg(feature = "stargate")]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GovMsg {
    /// This maps directly to [MsgVote](https://github.com/cosmos/cosmos-sdk/blob/v0.42.5/proto/cosmos/gov/v1beta1/tx.proto#L46-L56) in the Cosmos SDK with voter set to the contract address.
    Vote {
        proposal_id: u64,
        /// The vote option.
        ///
        /// This should be called "option" for consistency with Cosmos SDK. Sorry for that.
        /// See <https://github.com/CosmWasm/cosmwasm/issues/1571>.
        vote: VoteOption,
    },
    /// This maps directly to [MsgVoteWeighted](https://github.com/cosmos/cosmos-sdk/blob/v0.45.8/proto/cosmos/gov/v1beta1/tx.proto#L66-L78) in the Cosmos SDK with voter set to the contract address.
    #[cfg(feature = "cosmwasm_1_2")]
    VoteWeighted {
        proposal_id: u64,
        options: Vec<WeightedVoteOption>,
    },
}

#[cfg(feature = "stargate")]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VoteOption {
    Yes,
    No,
    Abstain,
    NoWithVeto,
}

#[cfg(all(feature = "stargate", feature = "cosmwasm_1_2"))]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct WeightedVoteOption {
    pub option: VoteOption,
    pub weight: Decimal,
}

/// Shortcut helper as the construction of WasmMsg::Instantiate can be quite verbose in contract code.
///
/// When using this, `admin` is always unset. If you need more flexibility, create the message directly.
pub fn wasm_instantiate(
    code_id: u64,
    msg: &impl Serialize,
    funds: Vec<Coin>,
    label: String,
) -> StdResult<WasmMsg> {
    let payload = to_json_binary(msg)?;
    Ok(WasmMsg::Instantiate {
        admin: None,
        code_id,
        msg: payload,
        funds,
        label,
    })
}

/// Shortcut helper as the construction of WasmMsg::Execute can be quite verbose in contract code
pub fn wasm_execute(
    contract_addr: impl Into<String>,
    msg: &impl Serialize,
    funds: Vec<Coin>,
) -> StdResult<WasmMsg> {
    let payload = to_json_binary(msg)?;
    Ok(WasmMsg::Execute {
        contract_addr: contract_addr.into(),
        msg: payload,
        funds,
    })
}

impl<T> From<BankMsg> for CosmosMsg<T> {
    fn from(msg: BankMsg) -> Self {
        CosmosMsg::Bank(msg)
    }
}

#[cfg(feature = "staking")]
impl<T> From<StakingMsg> for CosmosMsg<T> {
    fn from(msg: StakingMsg) -> Self {
        CosmosMsg::Staking(msg)
    }
}

#[cfg(feature = "staking")]
impl<T> From<DistributionMsg> for CosmosMsg<T> {
    fn from(msg: DistributionMsg) -> Self {
        CosmosMsg::Distribution(msg)
    }
}

impl<T> From<WasmMsg> for CosmosMsg<T> {
    fn from(msg: WasmMsg) -> Self {
        CosmosMsg::Wasm(msg)
    }
}

#[cfg(feature = "stargate")]
impl<T> From<IbcMsg> for CosmosMsg<T> {
    fn from(msg: IbcMsg) -> Self {
        CosmosMsg::Ibc(msg)
    }
}

#[cfg(feature = "stargate")]
impl<T> From<GovMsg> for CosmosMsg<T> {
    fn from(msg: GovMsg) -> Self {
        CosmosMsg::Gov(msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{coin, coins};

    #[test]
    fn from_bank_msg_works() {
        let to_address = String::from("you");
        let amount = coins(1015, "earth");
        let bank = BankMsg::Send { to_address, amount };
        let msg: CosmosMsg = bank.clone().into();
        match msg {
            CosmosMsg::Bank(msg) => assert_eq!(bank, msg),
            _ => panic!("must encode in Bank variant"),
        }
    }

    #[test]
    fn wasm_msg_serializes_to_correct_json() {
        // Instantiate with admin
        let msg = WasmMsg::Instantiate {
            admin: Some("king".to_string()),
            code_id: 7897,
            msg: br#"{"claim":{}}"#.into(),
            funds: vec![],
            label: "my instance".to_string(),
        };
        let json = to_json_binary(&msg).unwrap();
        assert_eq!(
            String::from_utf8_lossy(&json),
            r#"{"instantiate":{"admin":"king","code_id":7897,"msg":"eyJjbGFpbSI6e319","funds":[],"label":"my instance"}}"#,
        );

        // Instantiate without admin
        let msg = WasmMsg::Instantiate {
            admin: None,
            code_id: 7897,
            msg: br#"{"claim":{}}"#.into(),
            funds: vec![],
            label: "my instance".to_string(),
        };
        let json = to_json_binary(&msg).unwrap();
        assert_eq!(
            String::from_utf8_lossy(&json),
            r#"{"instantiate":{"admin":null,"code_id":7897,"msg":"eyJjbGFpbSI6e319","funds":[],"label":"my instance"}}"#,
        );

        // Instantiate with funds
        let msg = WasmMsg::Instantiate {
            admin: None,
            code_id: 7897,
            msg: br#"{"claim":{}}"#.into(),
            funds: vec![coin(321, "stones")],
            label: "my instance".to_string(),
        };
        let json = to_json_binary(&msg).unwrap();
        assert_eq!(
            String::from_utf8_lossy(&json),
            r#"{"instantiate":{"admin":null,"code_id":7897,"msg":"eyJjbGFpbSI6e319","funds":[{"denom":"stones","amount":"321"}],"label":"my instance"}}"#,
        );

        // Instantiate2
        #[cfg(feature = "cosmwasm_1_2")]
        {
            let msg = WasmMsg::Instantiate2 {
                admin: None,
                code_id: 7897,
                label: "my instance".to_string(),
                msg: br#"{"claim":{}}"#.into(),
                funds: vec![coin(321, "stones")],
                salt: Binary::from_base64("UkOVazhiwoo=").unwrap(),
            };
            let json = to_json_binary(&msg).unwrap();
            assert_eq!(
                String::from_utf8_lossy(&json),
                r#"{"instantiate2":{"admin":null,"code_id":7897,"label":"my instance","msg":"eyJjbGFpbSI6e319","funds":[{"denom":"stones","amount":"321"}],"salt":"UkOVazhiwoo="}}"#,
            );
        }
    }

    #[test]
    #[cfg(feature = "cosmwasm_1_3")]
    fn msg_distribution_serializes_to_correct_json() {
        // FundCommunityPool
        let fund_coins = vec![coin(200, "feathers"), coin(200, "stones")];
        let fund_msg = DistributionMsg::FundCommunityPool { amount: fund_coins };
        let fund_json = to_json_binary(&fund_msg).unwrap();
        assert_eq!(
            String::from_utf8_lossy(&fund_json),
            r#"{"fund_community_pool":{"amount":[{"denom":"feathers","amount":"200"},{"denom":"stones","amount":"200"}]}}"#,
        );

        // SetWithdrawAddress
        let set_msg = DistributionMsg::SetWithdrawAddress {
            address: String::from("withdrawer"),
        };
        let set_json = to_json_binary(&set_msg).unwrap();
        assert_eq!(
            String::from_utf8_lossy(&set_json),
            r#"{"set_withdraw_address":{"address":"withdrawer"}}"#,
        );

        // WithdrawDelegatorRewards
        let withdraw_msg = DistributionMsg::WithdrawDelegatorReward {
            validator: String::from("fancyoperator"),
        };
        let withdraw_json = to_json_binary(&withdraw_msg).unwrap();
        assert_eq!(
            String::from_utf8_lossy(&withdraw_json),
            r#"{"withdraw_delegator_reward":{"validator":"fancyoperator"}}"#
        );
    }

    #[test]
    fn wasm_msg_debug_decodes_binary_string_when_possible() {
        #[cosmwasm_schema::cw_serde]
        enum ExecuteMsg {
            Mint { coin: Coin },
        }

        let msg = WasmMsg::Execute {
            contract_addr: "joe".to_string(),
            msg: to_json_binary(&ExecuteMsg::Mint {
                coin: coin(10, "BTC"),
            })
            .unwrap(),
            funds: vec![],
        };

        assert_eq!(
            format!("{msg:?}"),
            "Execute { contract_addr: \"joe\", msg: {\"mint\":{\"coin\":{\"denom\":\"BTC\",\"amount\":\"10\"}}}, funds: [] }"
        );
    }

    #[test]
    fn wasm_msg_debug_dumps_binary_when_not_utf8() {
        let msg = WasmMsg::Execute {
            contract_addr: "joe".to_string(),
            msg: Binary::from([0, 159, 146, 150]),
            funds: vec![],
        };

        assert_eq!(
            format!("{msg:?}"),
            "Execute { contract_addr: \"joe\", msg: Binary(009f9296), funds: [] }"
        );
    }

    #[test]
    #[cfg(feature = "stargate")]
    fn gov_msg_serializes_to_correct_json() {
        // Vote
        let msg = GovMsg::Vote {
            proposal_id: 4,
            vote: VoteOption::NoWithVeto,
        };
        let json = to_json_binary(&msg).unwrap();
        assert_eq!(
            String::from_utf8_lossy(&json),
            r#"{"vote":{"proposal_id":4,"vote":"no_with_veto"}}"#,
        );

        // VoteWeighted
        #[cfg(feature = "cosmwasm_1_2")]
        {
            let msg = GovMsg::VoteWeighted {
                proposal_id: 25,
                options: vec![
                    WeightedVoteOption {
                        weight: Decimal::percent(25),
                        option: VoteOption::Yes,
                    },
                    WeightedVoteOption {
                        weight: Decimal::percent(25),
                        option: VoteOption::No,
                    },
                    WeightedVoteOption {
                        weight: Decimal::percent(50),
                        option: VoteOption::Abstain,
                    },
                ],
            };

            let json = to_json_binary(&msg).unwrap();
            assert_eq!(
                String::from_utf8_lossy(&json),
                r#"{"vote_weighted":{"proposal_id":25,"options":[{"option":"yes","weight":"0.25"},{"option":"no","weight":"0.25"},{"option":"abstain","weight":"0.5"}]}}"#,
            );
        }
    }
}
