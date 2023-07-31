#![cfg(test)]

use cosmwasm_std::{
    testing::{mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage},
    DepsMut, IbcChannel, IbcChannelConnectMsg, IbcChannelOpenMsg, IbcEndpoint, OwnedDeps,
};

use crate::{
    contract::instantiate,
    ibc::{ibc_channel_connect, ibc_channel_open, ICS20_ORDERING, ICS20_VERSION},
    msg::{AllowMsg, InitMsg},
    state::ChannelInfo,
};

pub const DEFAULT_TIMEOUT: u64 = 3600; // 1 hour,
pub const CONTRACT_PORT: &str = "ibc:wasm1234567890abcdef";
pub const REMOTE_PORT: &str = "transfer";
pub const CONNECTION_ID: &str = "connection-2";

pub fn mock_channel(channel_id: &str) -> IbcChannel {
    IbcChannel::new(
        IbcEndpoint {
            port_id: CONTRACT_PORT.into(),
            channel_id: channel_id.into(),
        },
        IbcEndpoint {
            port_id: REMOTE_PORT.into(),
            channel_id: format!("{}5", channel_id),
        },
        ICS20_ORDERING,
        ICS20_VERSION,
        CONNECTION_ID,
    )
}

pub fn mock_channel_info(channel_id: &str) -> ChannelInfo {
    ChannelInfo {
        id: channel_id.to_string(),
        counterparty_endpoint: IbcEndpoint {
            port_id: REMOTE_PORT.into(),
            channel_id: format!("{}5", channel_id),
        },
        connection_id: CONNECTION_ID.into(),
    }
}

// we simulate instantiate and ack here
pub fn add_channel(mut deps: DepsMut, channel_id: &str) {
    let channel = mock_channel(channel_id);
    let open_msg = IbcChannelOpenMsg::new_init(channel.clone());
    ibc_channel_open(deps.branch(), mock_env(), open_msg).unwrap();
    let connect_msg = IbcChannelConnectMsg::new_ack(channel, ICS20_VERSION);
    ibc_channel_connect(deps.branch(), mock_env(), connect_msg).unwrap();
}

pub fn setup(
    channels: &[&str],
    allow: &[(&str, u64)],
) -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
    let mut deps = mock_dependencies();

    let allowlist = allow
        .iter()
        .map(|(contract, gas)| AllowMsg {
            contract: contract.to_string(),
            gas_limit: Some(*gas),
        })
        .collect();

    // instantiate an empty contract
    let instantiate_msg = InitMsg {
        default_gas_limit: None,
        default_timeout: DEFAULT_TIMEOUT,
        gov_contract: "gov".to_string(),
        allowlist,
        channel: None,
    };
    let info = mock_info(&String::from("anyone"), &[]);
    let res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();
    assert_eq!(0, res.messages.len());

    for channel in channels {
        add_channel(deps.as_mut(), channel);
    }
    deps
}
