use cosmwasm_std::{
    Event,
    testing::{message_info, mock_dependencies, mock_env},
    to_json_binary,
};
use depolama::StorageExt;
use ibc_union_msg::{
    lightclient::{
        MisbehaviourQuery, MisbehaviourResponse, QueryMsg as LightClientQueryMsg, UpdateStateQuery,
        UpdateStateResponse, VerificationQueryMsg, VerifyCreationQuery, VerifyCreationResponse,
    },
    msg::{
        ExecuteMsg, InitMsg, MsgForceUpdateClient, MsgMisbehaviour, MsgUpdateClient,
        RestrictedExecuteMsg,
    },
};
use ibc_union_spec::path::{ClientStatePath, ConsensusStatePath};
use unionlabs::ethereum::keccak256;

use super::*;
use crate::{
    ContractError,
    contract::{execute, init},
    events::{ForceUpdateClient, Misbehaviour, RegisterClient},
    state::{
        ClientConsensusStates, ClientImpls, ClientRegistry, ClientStates, ClientTypes, Commitments,
    },
};

#[test]
fn register_client_ok() {
    let mut deps = mock_dependencies();

    init(
        deps.as_mut(),
        InitMsg {
            access_managed_init_msg: access_managed::InitMsg {
                initial_authority: mock_addr(MANAGER),
            },
        },
    )
    .unwrap();

    deps.querier.update_wasm(wasm_query_handler(|msg| {
        panic!("should not be called: {:?}", msg)
    }));

    let res = register_client(deps.as_mut()).unwrap();

    assert!(res.events.into_iter().any(|e| e
        == Event::from(RegisterClient {
            client_type: CLIENT_TYPE.to_owned(),
            client_address: mock_addr(CLIENT_ADDRESS),
        })));

    assert_eq!(
        deps.storage
            .read::<ClientRegistry>(&CLIENT_TYPE.to_string())
            .unwrap(),
        mock_addr(CLIENT_ADDRESS)
    );
}

#[test]
fn register_client_fails_when_duplicate() {
    let mut deps = mock_dependencies();

    init(
        deps.as_mut(),
        InitMsg {
            access_managed_init_msg: access_managed::InitMsg {
                initial_authority: mock_addr(MANAGER),
            },
        },
    )
    .unwrap();

    deps.querier.update_wasm(wasm_query_handler(|msg| {
        panic!("should not be called: {:?}", msg)
    }));

    register_client(deps.as_mut()).unwrap();

    assert_eq!(
        register_client(deps.as_mut()),
        Err(ContractError::ClientTypeAlreadyExists)
    );
}

#[test]
fn create_client_ok() {
    let mut deps = mock_dependencies();

    init(
        deps.as_mut(),
        InitMsg {
            access_managed_init_msg: access_managed::InitMsg {
                initial_authority: mock_addr(MANAGER),
            },
        },
    )
    .unwrap();
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::Verification(msg) => {
                match msg.ensure_not_paused(unpaused_deps()).unwrap() {
                    VerificationQueryMsg::VerifyCreation(VerifyCreationQuery { .. }) => {
                        to_json_binary(&VerifyCreationResponse {
                            counterparty_chain_id: "testchain".to_owned(),
                            events: vec![],
                            storage_writes: Default::default(),
                            client_state_bytes: None,
                        })
                    }
                    msg => panic!("should not be called: {:?}", msg),
                }
            }
            LightClientQueryMsg::GetLatestHeight { .. } => to_json_binary(&1),
            msg => panic!("should not be called: {:?}", msg),
        }));

    register_client(deps.as_mut()).expect("register client ok");
    assert!(create_client(deps.as_mut()).is_ok())
}

#[test]
fn create_client_commitments_saved() {
    let mut deps = mock_dependencies();

    init(
        deps.as_mut(),
        InitMsg {
            access_managed_init_msg: access_managed::InitMsg {
                initial_authority: mock_addr(MANAGER),
            },
        },
    )
    .unwrap();
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::Verification(msg) => {
                match msg.ensure_not_paused(unpaused_deps()).unwrap() {
                    VerificationQueryMsg::VerifyCreation(VerifyCreationQuery { .. }) => {
                        to_json_binary(&VerifyCreationResponse {
                            counterparty_chain_id: "testchain".to_owned(),
                            events: vec![],
                            storage_writes: Default::default(),
                            client_state_bytes: None,
                        })
                    }
                    msg => panic!("should not be called: {:?}", msg),
                }
            }
            LightClientQueryMsg::GetLatestHeight { .. } => to_json_binary(&1),
            msg => panic!("should not be called: {:?}", msg),
        }));

    register_client(deps.as_mut()).expect("register client ok");
    let res = create_client(deps.as_mut()).expect("create client ok");
    let client_id = res
        .events
        .iter()
        .find(|event| event.ty.eq("create_client"))
        .expect("create client event exists")
        .attributes
        .iter()
        .find(|attribute| attribute.key.eq("client_id"))
        .expect("client id attribute exists")
        .value
        .parse::<ClientId>()
        .expect("client id string is u32");

    assert_eq!(
        deps.storage.read::<ClientTypes>(&client_id).unwrap(),
        CLIENT_TYPE
    );
    assert_eq!(
        deps.storage.read::<ClientImpls>(&client_id).unwrap(),
        mock_addr(CLIENT_ADDRESS)
    );
    assert_eq!(
        deps.storage.read::<ClientStates>(&client_id).unwrap(),
        vec![1, 2, 3]
    );
    assert_eq!(
        deps.storage
            .read::<ClientConsensusStates>(&(client_id, 1))
            .unwrap(),
        vec![1, 2, 3]
    );
}

#[test]
fn update_client_ok() {
    let mut deps = mock_dependencies();

    init(
        deps.as_mut(),
        InitMsg {
            access_managed_init_msg: access_managed::InitMsg {
                initial_authority: mock_addr(MANAGER),
            },
        },
    )
    .unwrap();
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::Verification(msg) => {
                match msg.ensure_not_paused(unpaused_deps()).unwrap() {
                    VerificationQueryMsg::VerifyCreation(VerifyCreationQuery { .. }) => {
                        to_json_binary(&VerifyCreationResponse {
                            counterparty_chain_id: "testchain".to_owned(),
                            events: vec![],
                            storage_writes: Default::default(),
                            client_state_bytes: None,
                        })
                    }
                    VerificationQueryMsg::UpdateState(UpdateStateQuery { .. }) => {
                        to_json_binary(&UpdateStateResponse {
                            height: 2,
                            consensus_state_bytes: vec![3, 2, 1].into(),
                            client_state_bytes: Some(vec![3, 2, 1].into()),
                            storage_writes: Default::default(),
                        })
                    }
                    msg => panic!("should not be called: {:?}", msg),
                }
            }
            LightClientQueryMsg::GetStatus { .. } => to_json_binary(&Status::Active),
            LightClientQueryMsg::GetLatestHeight { .. } => to_json_binary(&1),
            msg => panic!("should not be called: {:?}", msg),
        }));

    register_client(deps.as_mut()).expect("register client ok");
    let res = create_client(deps.as_mut()).expect("create client ok");
    let client_id = res
        .events
        .iter()
        .find(|event| event.ty.eq("create_client"))
        .expect("create client event exists")
        .attributes
        .iter()
        .find(|attribute| attribute.key.eq("client_id"))
        .expect("client id attribute exists")
        .value
        .parse::<ClientId>()
        .expect("client id string is u32");

    assert!(
        execute(
            deps.as_mut(),
            mock_env(),
            message_info(&mock_addr(SENDER), &[]),
            ExecuteMsg::Restricted(Restricted::wrap(RestrictedExecuteMsg::UpdateClient(
                MsgUpdateClient {
                    client_id,
                    client_message: vec![3, 2, 1].into(),
                    relayer: mock_addr(RELAYER).into_string(),
                }
            )))
        )
        .is_ok()
    )
}

#[test]
fn update_client_ko() {
    let mut deps = mock_dependencies();

    init(
        deps.as_mut(),
        InitMsg {
            access_managed_init_msg: access_managed::InitMsg {
                initial_authority: mock_addr(MANAGER),
            },
        },
    )
    .unwrap();
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::Verification(msg) => {
                match msg.ensure_not_paused(unpaused_deps()).unwrap() {
                    VerificationQueryMsg::VerifyCreation(VerifyCreationQuery { .. }) => {
                        to_json_binary(&VerifyCreationResponse {
                            counterparty_chain_id: "testchain".to_owned(),
                            events: vec![],
                            storage_writes: Default::default(),
                            client_state_bytes: None,
                        })
                    }
                    VerificationQueryMsg::UpdateState { .. } => to_json_binary(&0),
                    msg => panic!("should not be called: {:?}", msg),
                }
            }
            LightClientQueryMsg::GetStatus { .. } => to_json_binary(&Status::Active),
            LightClientQueryMsg::GetLatestHeight { .. } => to_json_binary(&1),
            msg => panic!("should not be called: {:?}", msg),
        }));

    register_client(deps.as_mut()).expect("register client ok");
    let res = create_client(deps.as_mut()).expect("create client ok");
    let client_id = res
        .events
        .iter()
        .find(|event| event.ty.eq("create_client"))
        .expect("create client event exists")
        .attributes
        .iter()
        .find(|attribute| attribute.key.eq("client_id"))
        .expect("client id attribute exists")
        .value
        .parse::<ClientId>()
        .expect("client id string is u32");

    assert!(
        execute(
            deps.as_mut(),
            mock_env(),
            message_info(&mock_addr(SENDER), &[]),
            ExecuteMsg::Restricted(Restricted::wrap(RestrictedExecuteMsg::UpdateClient(
                MsgUpdateClient {
                    client_id,
                    client_message: vec![3, 2, 1].into(),
                    relayer: mock_addr(RELAYER).into_string(),
                }
            )))
        )
        .is_err()
    )
}

#[test]
fn force_update_client() {
    let mut deps = mock_dependencies();

    init(
        deps.as_mut(),
        InitMsg {
            access_managed_init_msg: access_managed::InitMsg {
                initial_authority: mock_addr(MANAGER),
            },
        },
    )
    .unwrap();

    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::Verification(msg) => {
                match msg.ensure_not_paused(unpaused_deps()).unwrap() {
                    VerificationQueryMsg::VerifyCreation(VerifyCreationQuery { .. }) => {
                        to_json_binary(&VerifyCreationResponse {
                            counterparty_chain_id: "testchain".to_owned(),
                            events: vec![],
                            storage_writes: Default::default(),
                            client_state_bytes: None,
                        })
                    }
                    msg => panic!("should not be called: {:?}", msg),
                }
            }
            LightClientQueryMsg::GetLatestHeight { .. } => to_json_binary(&1),
            msg => panic!("should not be called: {:?}", msg),
        }));

    register_client(deps.as_mut()).expect("register client ok");
    create_client(deps.as_mut()).expect("create client ok");

    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::Verification(msg) => {
                match msg.ensure_not_paused(unpaused_deps()).unwrap() {
                    VerificationQueryMsg::VerifyCreation(VerifyCreationQuery { .. }) => {
                        to_json_binary(&VerifyCreationResponse {
                            counterparty_chain_id: "testchain".to_owned(),
                            events: vec![],
                            storage_writes: Default::default(),
                            client_state_bytes: None,
                        })
                    }
                    msg => panic!("should not be called: {:?}", msg),
                }
            }
            LightClientQueryMsg::GetLatestHeight { .. } => to_json_binary(&2),
            msg => panic!("should not be called: {:?}", msg),
        }));

    let res = execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::Restricted(Restricted::wrap(RestrictedExecuteMsg::ForceUpdateClient(
            MsgForceUpdateClient {
                client_id: ClientId!(1),
                client_state_bytes: b"new_client_state".into(),
                consensus_state_bytes: b"new_consensus_state".into(),
            },
        ))),
    )
    .expect("force_update_client_ok");

    assert_eq!(
        res,
        Response::new().add_event(ForceUpdateClient {
            client_id: ClientId!(1),
            counterparty_height: 2,
        })
    );

    // states are saved correctly
    assert_eq!(
        deps.storage.read::<ClientStates>(&ClientId!(1)).unwrap(),
        b"new_client_state",
    );
    assert_eq!(
        deps.storage
            .read::<ClientConsensusStates>(&(ClientId!(1), 2))
            .unwrap(),
        b"new_consensus_state",
    );

    // commitments are saved correctly
    assert_eq!(
        deps.storage
            .read::<Commitments>(
                &ClientStatePath {
                    client_id: ClientId!(1)
                }
                .key()
            )
            .unwrap(),
        keccak256(b"new_client_state"),
    );
    assert_eq!(
        deps.storage
            .read::<Commitments>(
                &ConsensusStatePath {
                    client_id: ClientId!(1),
                    height: 2
                }
                .key()
            )
            .unwrap(),
        keccak256(b"new_consensus_state"),
    );
}

#[test]
fn update_client_commitments_saved() {
    let mut deps = mock_dependencies();

    init(
        deps.as_mut(),
        InitMsg {
            access_managed_init_msg: access_managed::InitMsg {
                initial_authority: mock_addr(MANAGER),
            },
        },
    )
    .unwrap();
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::Verification(msg) => {
                match msg.ensure_not_paused(unpaused_deps()).unwrap() {
                    VerificationQueryMsg::VerifyCreation(VerifyCreationQuery { .. }) => {
                        to_json_binary(&VerifyCreationResponse {
                            counterparty_chain_id: "testchain".to_owned(),
                            events: vec![],
                            storage_writes: Default::default(),
                            client_state_bytes: None,
                        })
                    }
                    VerificationQueryMsg::UpdateState(UpdateStateQuery { .. }) => {
                        to_json_binary(&UpdateStateResponse {
                            height: 2,
                            consensus_state_bytes: b"new_consensus_state".into(),
                            client_state_bytes: Some(b"new_client_state".into()),
                            storage_writes: Default::default(),
                        })
                    }
                    msg => panic!("should not be called: {:?}", msg),
                }
            }
            LightClientQueryMsg::GetStatus { .. } => to_json_binary(&Status::Active),
            LightClientQueryMsg::GetLatestHeight { .. } => to_json_binary(&1),
            msg => panic!("should not be called: {:?}", msg),
        }));

    register_client(deps.as_mut()).expect("register client ok");
    let res = create_client(deps.as_mut()).expect("create client ok");
    let client_id = res
        .events
        .iter()
        .find(|event| event.ty.eq("create_client"))
        .expect("create client event exists")
        .attributes
        .iter()
        .find(|attribute| attribute.key.eq("client_id"))
        .expect("client id attribute exists")
        .value
        .parse::<ClientId>()
        .expect("client id string is u32");

    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::Restricted(Restricted::wrap(RestrictedExecuteMsg::UpdateClient(
            MsgUpdateClient {
                client_id,
                client_message: vec![3, 2, 1].into(),
                relayer: mock_addr(RELAYER).into_string(),
            },
        ))),
    )
    .expect("update client ok");

    // states are saved correctly
    assert_eq!(
        deps.storage.read::<ClientStates>(&client_id).unwrap(),
        b"new_client_state",
    );
    assert_eq!(
        deps.storage
            .read::<ClientConsensusStates>(&(client_id, 2))
            .unwrap(),
        b"new_consensus_state",
    );

    // commitments are saved correctly
    assert_eq!(
        deps.storage
            .read::<Commitments>(&ClientStatePath { client_id }.key())
            .unwrap(),
        keccak256(b"new_client_state"),
    );
    assert_eq!(
        deps.storage
            .read::<Commitments>(
                &ConsensusStatePath {
                    client_id,
                    height: 2
                }
                .key()
            )
            .unwrap(),
        keccak256(b"new_consensus_state"),
    );
}

#[test]
fn misbehaviour() {
    let mut deps = mock_dependencies();

    init(
        deps.as_mut(),
        InitMsg {
            access_managed_init_msg: access_managed::InitMsg {
                initial_authority: mock_addr(MANAGER),
            },
        },
    )
    .unwrap();
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::Verification(msg) => {
                match msg.ensure_not_paused(unpaused_deps()).unwrap() {
                    VerificationQueryMsg::VerifyCreation(VerifyCreationQuery { .. }) => {
                        to_json_binary(&VerifyCreationResponse {
                            counterparty_chain_id: "testchain".to_owned(),
                            events: vec![],
                            storage_writes: Default::default(),
                            client_state_bytes: None,
                        })
                    }
                    VerificationQueryMsg::Misbehaviour(MisbehaviourQuery { .. }) => {
                        to_json_binary(&MisbehaviourResponse {
                            client_state_bytes: b"state".into(),
                        })
                    }
                    msg => panic!("should not be called: {:?}", msg),
                }
            }
            LightClientQueryMsg::GetStatus { .. } => to_json_binary(&Status::Active),
            LightClientQueryMsg::GetLatestHeight { .. } => to_json_binary(&1),
            msg => panic!("should not be called: {:?}", msg),
        }));

    register_client(deps.as_mut()).expect("register client ok");
    let res = create_client(deps.as_mut()).expect("create client ok");
    let client_id = res
        .events
        .iter()
        .find(|event| event.ty.eq("create_client"))
        .expect("create client event exists")
        .attributes
        .iter()
        .find(|attribute| attribute.key.eq("client_id"))
        .expect("client id attribute exists")
        .value
        .parse::<ClientId>()
        .expect("client id string is u32");

    let res = execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::Restricted(Restricted::wrap(RestrictedExecuteMsg::Misbehaviour(
            MsgMisbehaviour {
                client_id,
                client_message: b"state".into(),
                relayer: mock_addr(RELAYER).into_string(),
            },
        ))),
    )
    .expect("misbehaviour ok");

    assert_eq!(res, Response::new().add_event(Misbehaviour { client_id }));

    assert_eq!(
        deps.storage.read::<ClientStates>(&client_id).unwrap(),
        b"state",
    );

    assert_eq!(
        deps.storage
            .read::<Commitments>(&ClientStatePath { client_id }.key())
            .unwrap(),
        keccak256(b"state"),
    );

    // unchanged
    assert_eq!(
        deps.storage
            .read::<ClientConsensusStates>(&(client_id, 1))
            .unwrap(),
        [1, 2, 3],
    );
}
