use std::{num::NonZero, sync::LazyLock};

use attested_light_client_types::{ClientState, ClientStateV1, ConsensusState, Header};
use cosmwasm_std::{
    Addr, Api, OwnedDeps,
    testing::{MockApi, MockQuerier, MockStorage, message_info, mock_dependencies, mock_env},
};
use ed25519_dalek::{SigningKey, ed25519::signature::SignerMut};
use frissitheto::UpgradeMsg;
use hex_literal::hex;
use ibc_union_light_client::spec::{Duration, Timestamp};
use unionlabs::{
    encoding::{Bincode, EncodeAs},
    primitives::{H256, H512},
};

use crate::{
    client::{verify_attestation, verify_header},
    contract::{execute, migrate},
    errors::Error,
    msg::{ExecuteMsg, InitMsg},
    types::{Attestation, AttestationValue},
};

// sha256(1)
static ATTESTOR_1: LazyLock<SigningKey> = LazyLock::new(|| {
    SigningKey::from_bytes(&hex!(
        "4bf5122f344554c53bde2ebb8cd2b7e3d1600ad631c385a5d7cce23c7785459a"
    ))
});
// sha256(2)
static ATTESTOR_2: LazyLock<SigningKey> = LazyLock::new(|| {
    SigningKey::from_bytes(&hex!(
        "dbc1b4c900ffe48d575b5da5c638040125f65db0fe3e24494b76ea986457d986"
    ))
});
// sha256(3)
static ATTESTOR_3: LazyLock<SigningKey> = LazyLock::new(|| {
    SigningKey::from_bytes(&hex!(
        "084fed08b978af4d7d196a7446a86b58009e636b611db16211b65a9aadff29c5"
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

fn setup() -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
    let mut deps = mock_dependencies();
    let env = mock_env();

    let ibc_host = deps
        .api
        .addr_humanize(&b"ibc-host".as_slice().into())
        .unwrap();

    migrate(
        deps.as_mut(),
        env,
        UpgradeMsg::Init(InitMsg {
            ibc_host,
            attestors: attestors().map(vk).collect(),
            quorum: const { <NonZero<u8>>::new(2).unwrap() },
        }),
    )
    .unwrap();

    deps
}

fn reach_quorum(deps: &mut OwnedDeps<MockStorage, MockApi, MockQuerier>, attestation: Attestation) {
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

    let res = execute(
        deps.as_mut(),
        mock_env(),
        message_info(&Addr::unchecked(""), &[]),
        ExecuteMsg::Attest {
            attestation: attestation.clone(),
            attestor: vk(&ATTESTOR_2),
            signature: sign(&ATTESTOR_2, &attestation),
        },
    )
    .unwrap();

    assert_eq!(res.events.len(), 2);
    assert_eq!(res.events[0].ty, "attestation_submitted");
    assert_eq!(res.events[1].ty, "quorum_reached");

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
    let mut deps = setup();

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
    let mut deps = setup();

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

    reach_quorum(&mut deps, attestation.clone());

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

    let res = verify_header(
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

    assert_eq!(res.height, 2);
    assert_eq!(
        res.client_state,
        Some(ClientState::V1(ClientStateV1 {
            chain_id: "999".to_owned(),
            latest_height: 2,
        })),
    );
    assert_eq!(
        res.consensus_state,
        ConsensusState {
            timestamp: attestation.timestamp
        }
    );
    assert!(res.storage_writes.is_empty());
}

#[test]
fn quorum() {
    let mut deps = setup();

    let attestation = Attestation {
        height: 1,
        timestamp: Timestamp::from_nanos(100),
        key: b"key-1".into(),
        value: AttestationValue::Existence(b"value-1".into()),
    };

    reach_quorum(&mut deps, attestation.clone());

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

    let attestation = Attestation {
        height: 2,
        timestamp: Timestamp::from_nanos(100),
        key: b"key-1".into(),
        value: AttestationValue::NonExistence,
    };

    reach_quorum(&mut deps, attestation.clone());

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
}

#[test]
fn invalid_signature() {
    let mut deps = setup();

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
    let mut deps = setup();

    let mut attestation = Attestation {
        height: 1,
        timestamp: Timestamp::from_nanos(100),
        key: b"key-1".into(),
        value: AttestationValue::Existence(b"value-1".into()),
    };

    // reach quorum first
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
    .unwrap();

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
