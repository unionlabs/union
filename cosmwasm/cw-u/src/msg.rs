use cosmwasm_std::Addr;
use ibc_union_spec::{ChannelId, Packet};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use ucs03_zkgm::com::CwTokenOrderV2;
use unionlabs::primitives::{Bytes, H256, U256};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum Cw20InstantiateMsg {
    Cw20(cw20_base::msg::InstantiateMsg),
    Tokenfactory(cw20_wrapped_tokenfactory::msg::InitMsg),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct InitMsg {
    pub zkgm: Addr,
    pub admin: Addr,
    pub extra_minters: Vec<String>,
    pub cw20_init: Cw20InstantiateMsg,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum ExecuteMsg {
    WhitelistIntents {
        hashes_whitelist: Vec<(H256, bool)>,
    },
    SetFungibleCounterparty {
        path: U256,
        channel_id: ChannelId,
        base_token: Bytes,
        counterparty_beneficiary: Bytes,
    },
    DoSolve {
        packet: Packet,
        order: Box<CwTokenOrderV2>,
        path: U256,
        caller: Addr,
        relayer: Addr,
        relayer_msg: Bytes,
        intent: bool,
    },
    #[serde(untagged)]
    Cw20(Value),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum QueryMsg {
    AllMinters {},
    GetFungibleCounterparty {
        path: U256,
        channel_id: ChannelId,
        base_token: Bytes,
    },
    GetAllFungibleCounterparties {},
    // prevent from routing to the inner cw20 impl message
    Minter {},
    #[serde(untagged)]
    Cw20(Value),
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{from_json, to_json_string};
    use serde_json::from_value;

    #[test]
    fn test_deserialize_cw20_base() {
        let i = cw20_base::msg::ExecuteMsg::Cw20ExecuteMsg(cw20::Cw20ExecuteMsg::Burn {
            amount: 100u128.into(),
        });
        let msg_str = to_json_string(&i).unwrap();
        println!("{}", msg_str);
        let msg = from_json::<super::ExecuteMsg>(msg_str).unwrap();
        match msg {
            crate::msg::ExecuteMsg::Cw20(value) => {
                let o = from_value::<cw20_base::msg::ExecuteMsg>(value).unwrap();
                assert_eq!(i, o);
            }
            _ => panic!("impossible"),
        }
    }
}
