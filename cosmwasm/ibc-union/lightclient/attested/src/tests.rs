use std::{collections::BTreeSet, fmt::Debug, num::NonZero, sync::LazyLock};

use attested_light_client_types::{ClientState, ClientStateV1, ConsensusState, Header};
use cosmwasm_std::{
    Addr, Api, Deps, Env, Event, OwnedDeps, Response, from_json,
    testing::{MockApi, MockQuerier, MockStorage, message_info, mock_dependencies, mock_env},
};
use ed25519_dalek::{SigningKey, ed25519::signature::SignerMut};
use frissitheto::UpgradeMsg;
use hex_literal::hex;
use ibc_union_light_client::{
    StateUpdate,
    msg::InitMsg,
    spec::{Duration, Timestamp},
};
use serde::de::DeserializeOwned;
use unionlabs::{
    encoding::{Bincode, EncodeAs},
    primitives::{H256, H512},
};

use crate::{
    client::{verify_attestation, verify_header},
    contract::{execute, migrate, query},
    errors::Error,
    msg::{ExecuteMsg, QueryMsg, RestrictedExecuteMsg},
    types::{Attestation, AttestationValue},
};

// sha256(0x01)
static ATTESTOR_1: LazyLock<SigningKey> = LazyLock::new(|| {
    SigningKey::from_bytes(&hex!(
        "4bf5122f344554c53bde2ebb8cd2b7e3d1600ad631c385a5d7cce23c7785459a"
    ))
});
// sha256(0x02)
static ATTESTOR_2: LazyLock<SigningKey> = LazyLock::new(|| {
    SigningKey::from_bytes(&hex!(
        "dbc1b4c900ffe48d575b5da5c638040125f65db0fe3e24494b76ea986457d986"
    ))
});
// sha256(0x03)
static ATTESTOR_3: LazyLock<SigningKey> = LazyLock::new(|| {
    SigningKey::from_bytes(&hex!(
        "084fed08b978af4d7d196a7446a86b58009e636b611db16211b65a9aadff29c5"
    ))
});
// sha256(0x04)
static ATTESTOR_4: LazyLock<SigningKey> = LazyLock::new(|| {
    SigningKey::from_bytes(&hex!(
        "e52d9c508c502347344d8c07ad91cbd6068afc75ff6292f062a09ca381c89e71"
    ))
});

fn attestors() -> impl Iterator<Item = &'static SigningKey> {
    [&ATTESTOR_1, &ATTESTOR_2, &ATTESTOR_3]
        .into_iter()
        .map(|sk| &**sk)
}

fn sign(sk: &SigningKey, attestation: &Attestation) -> H512 {
    sk.clone()
        .sign(&(attestation).encode_as::<Bincode>())
        .to_bytes()
        .into()
}

fn vk(sk: &SigningKey) -> H256 {
    sk.verifying_key().to_bytes().into()
}

#[track_caller]
pub(crate) fn assert_query_result<T: Debug + PartialEq + DeserializeOwned>(
    deps: Deps,
    env: &Env,
    msg: QueryMsg,
    expected: &T,
) {
    let res = query(deps, env.clone(), msg).unwrap();
    assert_eq!(&from_json::<T>(res).unwrap(), expected);
}

fn setup() -> (OwnedDeps<MockStorage, MockApi, MockQuerier>, Env) {
    let mut deps = mock_dependencies();
    let env = mock_env();

    let ibc_host = deps
        .api
        .addr_humanize(&b"ibc-host".as_slice().into())
        .unwrap();

    migrate(
        deps.as_mut(),
        env.clone(),
        UpgradeMsg::Init(InitMsg { ibc_host }),
    )
    .unwrap();

    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&Addr::unchecked(""), &[]),
            ExecuteMsg::Restricted(RestrictedExecuteMsg::SetQuorum {
                new_quorum: const { NonZero::new(2).unwrap() },
            }),
        )
        .unwrap(),
        Response::new().add_event(Event::new("quorum_updated").add_attribute("quorum", "2"))
    );

    assert_query_result(deps.as_ref(), &env, QueryMsg::Quorum {}, &2);

    for attestor in attestors() {
        assert_eq!(
            execute(
                deps.as_mut(),
                env.clone(),
                message_info(&Addr::unchecked(""), &[]),
                ExecuteMsg::Restricted(RestrictedExecuteMsg::AddAttestor {
                    new_attestor: vk(attestor)
                }),
            )
            .unwrap(),
            Response::new().add_event(
                Event::new("attestor_added").add_attribute("attestor", vk(attestor).to_string())
            )
        );
    }

    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::Attestors {},
        &attestors().map(vk).collect::<BTreeSet<_>>(),
    );

    (deps, env)
}

fn reach_quorum<'a>(
    deps: &mut OwnedDeps<MockStorage, MockApi, MockQuerier>,
    env: &Env,
    attestation: Attestation,
    attestors: impl IntoIterator<Item = &'a SigningKey>,
) {
    assert_eq!(
        verify_attestation(
            deps.as_ref(),
            attestation.height,
            attestation.key.clone(),
            attestation.value.clone(),
        )
        .unwrap_err(),
        Error::AttestationNotFound {
            height: attestation.height,
            key: attestation.key.clone(),
        },
    );

    let mut res = Response::new();
    for attestor in attestors {
        res = execute(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked(""), &[]),
            ExecuteMsg::Attest {
                attestation: attestation.clone(),
                attestor: vk(attestor),
                signature: sign(attestor, &attestation),
            },
        )
        .unwrap();

        assert_eq!(res.events[0].ty, "attestation_submitted");
    }

    assert_eq!(res.events[1].ty, "quorum_reached");

    assert_query_result(
        deps.as_ref(),
        env,
        QueryMsg::AttestedValue {
            height: attestation.height,
            key: attestation.key.clone(),
        },
        &attestation.value,
    );

    assert_query_result(
        deps.as_ref(),
        env,
        QueryMsg::TimestampAtHeight {
            height: attestation.height,
        },
        &attestation.timestamp,
    );

    // quorum reached, attestation should verify
    verify_attestation(
        deps.as_ref(),
        attestation.height,
        attestation.key.clone(),
        attestation.value.clone(),
    )
    .unwrap();

    // attesting to the same data should fail
    assert_eq!(
        execute(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked(""), &[]),
            ExecuteMsg::Attest {
                attestation: attestation.clone(),
                attestor: vk(&ATTESTOR_2),
                signature: sign(&ATTESTOR_2, &attestation),
            },
        )
        .unwrap_err(),
        Error::AlreadyAttested {
            height: attestation.height,
            timestamp: attestation.timestamp,
            key: attestation.key,
            value: attestation.value,
        }
    );
}

#[test]
fn attest() {
    let (mut deps, _) = setup();

    let attestation = Attestation {
        height: 1,
        timestamp: Timestamp::from_nanos(100),
        key: b"key-1".into(),
        value: AttestationValue::Existence(b"value-1".into()),
    };

    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&Addr::unchecked(""), &[]),
        ExecuteMsg::Attest {
            attestation: attestation.clone(),
            attestor: vk(&ATTESTOR_1),
            signature: sign(&ATTESTOR_1, &attestation),
        },
    )
    .unwrap();

    // attesting twice should fail
    assert_eq!(
        execute(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked(""), &[]),
            ExecuteMsg::Attest {
                attestation: attestation.clone(),
                attestor: vk(&ATTESTOR_1),
                signature: sign(&ATTESTOR_1, &attestation),
            },
        )
        .unwrap_err(),
        Error::AttestationAlreadyReceived,
    );
}

#[test]
fn verify_header_works() {
    let (mut deps, env) = setup();

    let attestation = Attestation {
        height: 2,
        timestamp: Timestamp::from_nanos(100),
        key: b"key-1".into(),
        value: AttestationValue::Existence(b"value-1".into()),
    };

    // can't update before quorum is reached for a height
    assert!(
        verify_header(
            deps.as_ref(),
            ClientState::V1(ClientStateV1 {
                chain_id: "999".to_owned(),
                latest_height: 1,
            }),
            Header {
                height: attestation.height,
                timestamp: Timestamp::from_nanos(100),
            },
        )
        .is_err()
    );

    reach_quorum(
        &mut deps,
        &env,
        attestation.clone(),
        [&*ATTESTOR_1, &*ATTESTOR_2],
    );

    // timestamp is checked
    assert_eq!(
        verify_header(
            deps.as_ref(),
            ClientState::V1(ClientStateV1 {
                chain_id: "999".to_owned(),
                latest_height: 1,
            }),
            Header {
                height: attestation.height,
                timestamp: Timestamp::from_nanos(101),
            },
        )
        .err()
        .unwrap(),
        Error::InvalidTimestamp {
            height: 2,
            attested_timestamp: Timestamp::from_nanos(100),
            timestamp: Timestamp::from_nanos(101)
        }
    );

    let StateUpdate {
        height,
        client_state,
        consensus_state,
        storage_writes,
    } = verify_header(
        deps.as_ref(),
        ClientState::V1(ClientStateV1 {
            chain_id: "999".to_owned(),
            latest_height: 1,
        }),
        Header {
            height: attestation.height,
            timestamp: attestation.timestamp,
        },
    )
    .unwrap();

    assert_eq!(height, 2);
    assert_eq!(
        client_state,
        Some(ClientState::V1(ClientStateV1 {
            chain_id: "999".to_owned(),
            latest_height: 2,
        })),
    );
    assert_eq!(
        consensus_state,
        ConsensusState {
            timestamp: attestation.timestamp
        }
    );
    assert!(storage_writes.is_empty());
}

#[test]
fn quorum() {
    let (mut deps, env) = setup();

    let attestation = Attestation {
        height: 1,
        timestamp: Timestamp::from_nanos(100),
        key: b"key-1".into(),
        value: AttestationValue::Existence(b"value-1".into()),
    };

    reach_quorum(
        &mut deps,
        &env,
        attestation.clone(),
        [&*ATTESTOR_1, &*ATTESTOR_2],
    );

    // membership, proof value is non-existence
    assert_eq!(
        verify_attestation(
            deps.as_ref(),
            attestation.height,
            attestation.key.clone(),
            AttestationValue::NonExistence,
        )
        .unwrap_err(),
        Error::InvalidAttestedValue {
            height: attestation.height,
            key: attestation.key.clone(),
            value: AttestationValue::NonExistence,
            attested: attestation.value.clone(),
        },
    );

    // membership, proof value is invalid
    assert_eq!(
        verify_attestation(
            deps.as_ref(),
            attestation.height,
            attestation.key.clone(),
            AttestationValue::Existence(b"invalid value".into()),
        )
        .unwrap_err(),
        Error::InvalidAttestedValue {
            height: attestation.height,
            key: attestation.key.clone(),
            value: AttestationValue::Existence(b"invalid value".into()),
            attested: attestation.value.clone(),
        },
    );

    verify_attestation(
        deps.as_ref(),
        attestation.height,
        attestation.key.clone(),
        AttestationValue::Existence(b"value-1".into()),
    )
    .unwrap();

    let attestation = Attestation {
        height: 2,
        timestamp: Timestamp::from_nanos(100),
        key: b"key-1".into(),
        value: AttestationValue::NonExistence,
    };

    reach_quorum(
        &mut deps,
        &env,
        attestation.clone(),
        [&*ATTESTOR_1, &*ATTESTOR_2],
    );

    // non-membership, proof value is existence
    assert_eq!(
        verify_attestation(
            deps.as_ref(),
            attestation.height,
            attestation.key.clone(),
            AttestationValue::Existence(b"unexpected existence".into()),
        )
        .unwrap_err(),
        Error::InvalidAttestedValue {
            height: attestation.height,
            key: attestation.key.clone(),
            value: AttestationValue::Existence(b"unexpected existence".into()),
            attested: attestation.value.clone(),
        },
    );

    verify_attestation(
        deps.as_ref(),
        attestation.height,
        attestation.key.clone(),
        AttestationValue::NonExistence,
    )
    .unwrap();
}

#[test]
fn invalid_signature() {
    let (mut deps, _) = setup();

    let attestation = Attestation {
        height: 1,
        timestamp: Timestamp::from_nanos(100),
        key: b"key-1".into(),
        value: AttestationValue::Existence(b"value-1".into()),
    };

    assert_eq!(
        execute(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked(""), &[]),
            ExecuteMsg::Attest {
                attestation: attestation.clone(),
                attestor: vk(&ATTESTOR_1),
                signature: (*b"this is an invalid attestation signature, it should not verify!!")
                    .into(),
            },
        )
        .unwrap_err(),
        Error::InvalidSignature
    );
}

#[test]
fn inconsistent_timestamp() {
    let (mut deps, env) = setup();

    let mut attestation = Attestation {
        height: 1,
        timestamp: Timestamp::from_nanos(100),
        key: b"key-1".into(),
        value: AttestationValue::Existence(b"value-1".into()),
    };

    // no attestations exist at this height yet
    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::TimestampAtHeight {
            height: attestation.height,
        },
        &None::<Timestamp>,
    );

    // reach quorum first
    reach_quorum(
        &mut deps,
        &env,
        attestation.clone(),
        [&*ATTESTOR_1, &*ATTESTOR_2],
    );

    // attesting to data at the same height but with a different timestamp should fail
    attestation.timestamp = attestation
        .timestamp
        .plus_duration(Duration::from_nanos(1))
        .unwrap();

    assert_eq!(
        execute(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked(""), &[]),
            ExecuteMsg::Attest {
                attestation: attestation.clone(),
                attestor: vk(&ATTESTOR_1),
                signature: sign(&ATTESTOR_1, &attestation),
            },
        )
        .unwrap_err(),
        Error::InconsistentTimestamp {
            height: 1,
            timestamp: Timestamp::from_nanos(101),
            previously_attested_timestamp: Timestamp::from_nanos(100),
        }
    );
}

#[test]
fn add_attestor() {
    let (mut deps, env) = setup();

    // can't add an attestor that's already in the attestation set
    assert_eq!(
        execute(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked(""), &[]),
            ExecuteMsg::Restricted(RestrictedExecuteMsg::AddAttestor {
                new_attestor: vk(&ATTESTOR_3)
            }),
        )
        .unwrap_err(),
        Error::AttestorAlreadyExists {
            attestor: vk(&ATTESTOR_3)
        }
    );

    assert_eq!(
        execute(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked(""), &[]),
            ExecuteMsg::Restricted(RestrictedExecuteMsg::AddAttestor {
                new_attestor: vk(&ATTESTOR_4)
            }),
        )
        .unwrap(),
        Response::new().add_event(
            Event::new("attestor_added").add_attribute("attestor", vk(&ATTESTOR_4).to_string())
        ),
    );

    // the new attestor can now attest
    reach_quorum(
        &mut deps,
        &env,
        Attestation {
            height: 1,
            timestamp: Timestamp::from_secs(1),
            key: b"key".into(),
            value: AttestationValue::NonExistence,
        },
        [&*ATTESTOR_4, &*ATTESTOR_3],
    );
}

#[test]
fn remove_attestor() {
    let (mut deps, env) = setup();

    // can't remove an attestor that isn't in the attestation set
    assert_eq!(
        execute(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked(""), &[]),
            ExecuteMsg::Restricted(RestrictedExecuteMsg::RemoveAttestor {
                old_attestor: vk(&ATTESTOR_4)
            }),
        )
        .unwrap_err(),
        Error::InvalidAttestor {
            attestor: vk(&ATTESTOR_4)
        }
    );

    let attestation = Attestation {
        height: 1,
        timestamp: Timestamp::from_secs(1),
        key: b"key".into(),
        value: AttestationValue::NonExistence,
    };

    // begin an attestation with a signature from the attestor that will be removed

    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&Addr::unchecked(""), &[]),
        ExecuteMsg::Attest {
            attestation: attestation.clone(),
            attestor: vk(&ATTESTOR_1),
            signature: sign(&ATTESTOR_1, &attestation),
        },
    )
    .unwrap();

    assert_eq!(
        execute(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked(""), &[]),
            ExecuteMsg::Restricted(RestrictedExecuteMsg::RemoveAttestor {
                old_attestor: vk(&ATTESTOR_1)
            }),
        )
        .unwrap(),
        Response::new().add_event(
            Event::new("attestor_removed").add_attribute("attestor", vk(&ATTESTOR_1).to_string())
        ),
    );

    // 2 signatures are required now, since the signature from attestor-1 is no longer valid
    reach_quorum(&mut deps, &env, attestation, [&*ATTESTOR_2, &*ATTESTOR_3]);

    let attestation = Attestation {
        height: 1,
        timestamp: Timestamp::from_secs(1),
        key: b"key2".into(),
        value: AttestationValue::NonExistence,
    };

    // removed attestor can't attest to any new attestations
    assert_eq!(
        execute(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked(""), &[]),
            ExecuteMsg::Attest {
                attestation: attestation.clone(),
                attestor: vk(&ATTESTOR_1),
                signature: sign(&ATTESTOR_1, &attestation)
            },
        )
        .unwrap_err(),
        Error::InvalidAttestor {
            attestor: vk(&ATTESTOR_1)
        }
    );
}

#[test]
fn confirm_attestation() {
    let (mut deps, _) = setup();

    let attestation = Attestation {
        height: 1,
        timestamp: Timestamp::from_secs(1),
        key: b"key2".into(),
        value: AttestationValue::NonExistence,
    };

    // can't confirm non-existent attestation
    assert_eq!(
        execute(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked(""), &[]),
            ExecuteMsg::ConfirmAttestation {
                attestation: attestation.clone()
            },
        )
        .unwrap_err(),
        Error::QuorumNotReached {
            quorum: const { NonZero::new(2).unwrap() },
            current: 0,
        }
    );

    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&Addr::unchecked(""), &[]),
        ExecuteMsg::Attest {
            attestation: attestation.clone(),
            attestor: vk(&ATTESTOR_1),
            signature: sign(&ATTESTOR_1, &attestation),
        },
    )
    .unwrap();

    // can't confirm attestation that hasn't reached quorum
    assert_eq!(
        execute(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked(""), &[]),
            ExecuteMsg::ConfirmAttestation {
                attestation: attestation.clone()
            },
        )
        .unwrap_err(),
        Error::QuorumNotReached {
            quorum: const { NonZero::new(2).unwrap() },
            current: 1,
        }
    );

    // lower the quorum to 1, making the current pending attestation valid
    assert_eq!(
        execute(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked(""), &[]),
            ExecuteMsg::Restricted(RestrictedExecuteMsg::SetQuorum {
                new_quorum: const { NonZero::new(1).unwrap() }
            }),
        )
        .unwrap(),
        Response::new().add_event(Event::new("quorum_updated").add_attribute("quorum", "1"))
    );

    // the attestation has hit the new quorum, so it can be confirmed
    assert_eq!(
        execute(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked(""), &[]),
            ExecuteMsg::ConfirmAttestation {
                attestation: attestation.clone()
            },
        )
        .unwrap()
        .events[0]
            .ty,
        "quorum_reached",
    );
}
