#![allow(clippy::result_large_err)] // don't feel like boxing everything

use cosmwasm_std::{Addr, Empty};
use gno_light_client_types::{ClientState, ConsensusState, Header};
use gno_types::{Commit, SignedHeader};
use gno_verifier::types::SignatureVerifier;
use ibc_union_light_client::{
    ClientCreationResult, IbcClient, IbcClientCtx, IbcClientError, StateUpdate, spec::Status,
};
use unionlabs::{
    bounded::BoundedI64,
    cosmos::ics23::proof_spec::ProofSpec,
    encoding::Bincode,
    google::protobuf::{
        duration::Duration,
        timestamp::{MAX_TIMESTAMP, Timestamp},
    },
    ibc::core::commitment::{merkle_proof::MerkleProof, merkle_root::MerkleRoot},
    primitives::{
        Bech32, Bytes, H160, H256,
        encoding::{Base64, HexUnprefixed},
    },
};

use crate::{
    errors::{Error, IbcHeightTooLargeForGnoHeight, InvalidHostTimestamp},
    verifier::CwVerifier,
};

pub struct GnoLightClient;

impl IbcClient for GnoLightClient {
    type Error = Error;

    type CustomQuery = Empty;

    type Header = Header;

    type Misbehaviour = Header;

    type ClientState = ClientState;

    type ConsensusState = ConsensusState;

    type StateProof = MerkleProof;

    type Encoding = Bincode;

    fn verify_membership(
        ctx: IbcClientCtx<Self>,
        height: u64,
        key: Vec<u8>,
        storage_proof: Self::StateProof,
        value: Vec<u8>,
    ) -> Result<(), IbcClientError<Self>> {
        let client_state = ctx.read_self_client_state()?;
        let consensus_state = ctx.read_self_consensus_state(height)?;

        verify_membership(
            &client_state.realm,
            &client_state.proof_specs,
            &consensus_state.root,
            key,
            storage_proof,
            value,
        )?;

        Ok(())
    }

    fn verify_non_membership(
        ctx: IbcClientCtx<Self>,
        height: u64,
        key: Vec<u8>,
        storage_proof: Self::StateProof,
    ) -> Result<(), IbcClientError<Self>> {
        let client_state = ctx.read_self_client_state()?;
        let consensus_state = ctx.read_self_consensus_state(height)?;

        verify_non_membership(
            &client_state.realm,
            &client_state.proof_specs,
            &consensus_state.root,
            key,
            storage_proof,
        )?;

        Ok(())
    }

    fn verify_header(
        ctx: IbcClientCtx<Self>,
        _caller: Addr,
        header: Self::Header,
        _relayer: Addr,
    ) -> Result<StateUpdate<Self>, IbcClientError<Self>> {
        let client_state = ctx.read_self_client_state()?;
        let consensus_state = ctx.read_self_consensus_state(header.trusted_height.height())?;

        let state_update = verify_header(
            client_state,
            consensus_state,
            header,
            ctx.env.block.time,
            &CwVerifier::new(ctx.deps),
        )?;

        Ok(state_update)
    }

    fn misbehaviour(
        _ctx: IbcClientCtx<Self>,
        _caller: Addr,
        _misbehaviour: Self::Misbehaviour,
        _relayer: Addr,
    ) -> Result<Self::ClientState, IbcClientError<Self>> {
        todo!()
    }

    fn status(ctx: IbcClientCtx<Self>, client_state: &Self::ClientState) -> Status {
        let Ok(consensus_state) =
            ctx.read_self_consensus_state(client_state.latest_height.height())
        else {
            return Status::Frozen;
        };

        if client_state.frozen_height.unwrap_or_default().height() != 0 {
            Status::Frozen
        } else if is_client_expired(
            &consensus_state.timestamp,
            client_state.trusting_period,
            ctx.env.block.time.try_into().unwrap_or(MAX_TIMESTAMP),
        ) {
            Status::Expired
        } else {
            Status::Active
        }
    }

    fn get_timestamp(consensus_state: &Self::ConsensusState) -> ibc_union_spec::Timestamp {
        ibc_union_spec::Timestamp::from_nanos(consensus_state.timestamp.as_unix_nanos())
    }

    fn get_latest_height(client_state: &Self::ClientState) -> u64 {
        client_state.latest_height.height()
    }

    fn get_counterparty_chain_id(client_state: &Self::ClientState) -> String {
        client_state.chain_id.clone()
    }

    fn verify_creation(
        _caller: Addr,
        _client_state: &Self::ClientState,
        _consensus_state: &Self::ConsensusState,
        _relayer: Addr,
    ) -> Result<ClientCreationResult<Self>, IbcClientError<Self>> {
        Ok(ClientCreationResult::new())
    }
}

pub fn verify_header<V: SignatureVerifier>(
    mut client_state: ClientState,
    consensus_state: ConsensusState,
    header: Header,
    block_timestamp: cosmwasm_std::Timestamp,
    signature_verifier: &V,
) -> Result<StateUpdate<GnoLightClient>, Error> {
    gno_verifier::verify::verify(
        &construct_partial_header(
            client_state.chain_id.clone(),
            i64::try_from(header.trusted_height.height())
                .map_err(|_| {
                    Error::from(IbcHeightTooLargeForGnoHeight(
                        header.trusted_height.height(),
                    ))
                })?
                .try_into()
                .expect(
                    "value is converted from u64, which is positive, \
                    and the expected bounded type is >= 0; qed;",
                ),
            consensus_state.timestamp,
            consensus_state.next_validators_hash,
        ),
        &header.trusted_validators,
        &header.signed_header,
        &header.validator_set,
        client_state.trusting_period,
        Timestamp::try_from(block_timestamp).map_err(InvalidHostTimestamp)?,
        client_state.max_clock_drift,
        &client_state.trust_level,
        signature_verifier,
    )
    .map_err(Error::GnoVerify)?;

    let update_height = header
        .signed_header
        .header
        .height
        .inner()
        .try_into()
        .expect("impossible");

    let state_update = StateUpdate::new(
        update_height,
        ConsensusState {
            timestamp: header.signed_header.header.time,
            root: MerkleRoot {
                hash: header.signed_header.header.app_hash.expect("must exist"),
            },
            next_validators_hash: header.signed_header.header.next_validators_hash,
        },
    );

    if client_state.latest_height.height() < update_height {
        *client_state.latest_height.height_mut() = update_height;
        Ok(state_update.overwrite_client_state(client_state))
    } else {
        Ok(state_update)
    }
}

pub fn construct_partial_header(
    chain_id: String,
    height: BoundedI64<0>,
    time: Timestamp,
    next_validators_hash: H256<Base64>,
) -> SignedHeader {
    SignedHeader {
        header: gno_types::Header {
            version: Default::default(),
            chain_id,
            height,
            time,
            num_txs: BoundedI64::new(0).unwrap(),
            total_txs: BoundedI64::new(0).unwrap(),
            app_version: "".to_owned(),
            last_block_id: Default::default(),
            last_commit_hash: Default::default(),
            data_hash: Default::default(),
            validators_hash: Default::default(),
            next_validators_hash,
            consensus_hash: Default::default(),
            app_hash: Default::default(),
            last_results_hash: Default::default(),
            proposer_address: Bech32::new("g".to_owned(), H160::default()),
        },
        commit: Commit {
            block_id: Default::default(),
            precommits: vec![],
        },
    }
}

pub fn is_client_expired(
    consensus_state_timestamp: &Timestamp,
    trusting_period: Duration,
    current_block_time: Timestamp,
) -> bool {
    if let Some(sum) = consensus_state_timestamp.checked_add(trusting_period) {
        sum < current_block_time
    } else {
        true
    }
}

pub fn verify_membership(
    realm: &str,
    proof_specs: &[ProofSpec],
    root: &MerkleRoot,
    key: Vec<u8>,
    storage_proof: MerkleProof,
    value: Vec<u8>,
) -> Result<(), Error> {
    ics23::ibc_api::verify_membership(
        &storage_proof,
        proof_specs,
        root,
        &[b"main".to_vec(), gnovm_store_key(realm, key)],
        value,
    )
    .map_err(Error::VerifyMembership)
}

pub fn verify_non_membership(
    realm: &str,
    proof_specs: &[ProofSpec],
    root: &MerkleRoot,
    key: Vec<u8>,
    storage_proof: MerkleProof,
) -> Result<(), Error> {
    ics23::ibc_api::verify_non_membership(
        &storage_proof,
        proof_specs,
        root,
        &[b"main".to_vec(), gnovm_store_key(realm, key)],
    )
    .map_err(Error::VerifyMembership)
}

fn gnovm_store_key(realm: &str, key: Vec<u8>) -> Vec<u8> {
    format!("/pv/vm:{realm}:{}", <Bytes<HexUnprefixed>>::new(key)).into_bytes()
}

// #[cfg(test)]
// mod tests {
//     use std::fs;

//     use cosmwasm_std::{
//         testing::{mock_dependencies, MockApi, MockQuerier, MockStorage},
//         OwnedDeps,
//     };
//     use ics008_wasm_client::{
//         storage_utils::{
//             consensus_db_key, read_subject_consensus_state, HOST_CLIENT_STATE_KEY,
//             SUBJECT_CLIENT_STORE_PREFIX, SUBSTITUTE_CLIENT_STORE_PREFIX,
//         },
//         FROZEN_HEIGHT,
//     };
//     use unionlabs::{encoding::EncodeAs, google::protobuf::any::Any};

//     use super::*;

//     const INITIAL_CONSENSUS_STATE_HEIGHT: Height = Height::new_with_revision(1, 10);

//     const INITIAL_SUBSTITUTE_CONSENSUS_STATE_HEIGHT: Height = Height::new_with_revision(1, 12);

//     fn save_states_to_migrate_store(
//         deps: DepsMut,
//         subject_client_state: &WasmClientStateOf<TendermintLightClient>,
//         substitute_client_state: &WasmClientStateOf<TendermintLightClient>,
//         subject_consensus_state: &WasmConsensusStateOf<TendermintLightClient>,
//         substitute_consensus_state: &WasmConsensusStateOf<TendermintLightClient>,
//     ) {
//         deps.storage.set(
//             format!("{SUBJECT_CLIENT_STORE_PREFIX}{HOST_CLIENT_STATE_KEY}").as_bytes(),
//             &Any(subject_client_state.clone()).encode_as::<Proto>(),
//         );
//         deps.storage.set(
//             format!(
//                 "{SUBJECT_CLIENT_STORE_PREFIX}{}",
//                 consensus_db_key(&INITIAL_CONSENSUS_STATE_HEIGHT)
//             )
//             .as_bytes(),
//             &Any(subject_consensus_state.clone()).encode_as::<Proto>(),
//         );
//         deps.storage.set(
//             format!("{SUBSTITUTE_CLIENT_STORE_PREFIX}{HOST_CLIENT_STATE_KEY}").as_bytes(),
//             &Any(substitute_client_state.clone()).encode_as::<Proto>(),
//         );
//         deps.storage.set(
//             format!(
//                 "{SUBSTITUTE_CLIENT_STORE_PREFIX}{}",
//                 consensus_db_key(&INITIAL_SUBSTITUTE_CONSENSUS_STATE_HEIGHT)
//             )
//             .as_bytes(),
//             &Any(substitute_consensus_state.clone()).encode_as::<Proto>(),
//         );
//     }

//     #[allow(clippy::type_complexity)] // it's fine bro
//     fn prepare_migrate_tests() -> (
//         OwnedDeps<MockStorage, MockApi, MockQuerier, Empty>,
//         WasmClientStateOf<TendermintLightClient>,
//         WasmConsensusStateOf<TendermintLightClient>,
//         WasmClientStateOf<TendermintLightClient>,
//         WasmConsensusStateOf<TendermintLightClient>,
//     ) {
//         (
//             mock_dependencies(),
//             serde_json::from_str(&fs::read_to_string("src/test/client_state.json").unwrap())
//                 .unwrap(),
//             serde_json::from_str(&fs::read_to_string("src/test/consensus_state.json").unwrap())
//                 .unwrap(),
//             serde_json::from_str(
//                 &fs::read_to_string("src/test/substitute_client_state.json").unwrap(),
//             )
//             .unwrap(),
//             serde_json::from_str(
//                 &fs::read_to_string("src/test/substitute_consensus_state.json").unwrap(),
//             )
//             .unwrap(),
//         )
//     }

//     #[test]
//     fn migrate_client_store_works() {
//         let (
//             mut deps,
//             mut wasm_client_state,
//             wasm_consensus_state,
//             substitute_wasm_client_state,
//             substitute_wasm_consensus_state,
//         ) = prepare_migrate_tests();

//         wasm_client_state.frozen_height = Some(FROZEN_HEIGHT);

//         save_states_to_migrate_store(
//             deps.as_mut(),
//             &wasm_client_state,
//             &substitute_wasm_client_state,
//             &wasm_consensus_state,
//             &substitute_wasm_consensus_state,
//         );

//         TendermintLightClient::migrate_client_store(deps.as_mut()).unwrap();

//         let wasm_client_state: WasmClientStateOf<TendermintLightClient> =
//             read_subject_client_state::<TendermintLightClient>(deps.as_ref()).unwrap();
//         // we didn't miss updating any fields
//         assert_eq!(wasm_client_state, substitute_wasm_client_state);
//         // client is unfrozen
//         assert_eq!(wasm_client_state.frozen_height, None);

//         // the new consensus state is saved under the correct height
//         assert_eq!(
//             read_subject_consensus_state::<TendermintLightClient>(
//                 deps.as_ref(),
//                 &INITIAL_SUBSTITUTE_CONSENSUS_STATE_HEIGHT
//             )
//             .unwrap()
//             .unwrap(),
//             substitute_wasm_consensus_state
//         );

//         // the new consensus state metadata is saved under substitute's latest height
//         assert_eq!(
//             get_current_or_next_consensus_state_meta(
//                 deps.as_ref(),
//                 INITIAL_SUBSTITUTE_CONSENSUS_STATE_HEIGHT
//             )
//             .unwrap()
//             .unwrap()
//             .0,
//             substitute_wasm_client_state.latest_height
//         );
//     }

//     #[test]
//     fn migrate_client_store_fails_when_invalid_change() {
//         let (
//             mut deps,
//             wasm_client_state,
//             wasm_consensus_state,
//             substitute_wasm_client_state,
//             substitute_wasm_consensus_state,
//         ) = prepare_migrate_tests();

//         macro_rules! modify_fns {
//             ($param:ident, $($m:expr), + $(,)?) => ([$(|$param: &mut ClientState| $m),+])
//         }

//         let modifications = modify_fns! { s,
//             s.trust_level.numerator ^= u64::MAX,
//             s.unbonding_period = Duration::new(s.unbonding_period.seconds().inner() + 1, 0).unwrap(),
//             s.max_clock_drift = Duration::new(s.max_clock_drift.seconds().inner() + 1, 0).unwrap(),
//             s.proof_specs.push(s.proof_specs[0].clone()),
//             s.upgrade_path.push(String::new()),

//         };

//         for m in modifications {
//             let mut state = substitute_wasm_client_state.clone();
//             m(&mut state.data);

//             save_states_to_migrate_store(
//                 deps.as_mut(),
//                 &wasm_client_state,
//                 &state,
//                 &wasm_consensus_state,
//                 &substitute_wasm_consensus_state,
//             );
//             assert_eq!(
//                 TendermintLightClient::migrate_client_store(deps.as_mut()),
//                 Err(
//                     Error::MigrateClientStore(MigrateClientStoreError::MigrateFieldsChanged).into()
//                 )
//             );
//         }
//     }

//     #[test]
//     fn migrate_client_store_fails_when_substitute_client_frozen() {
//         let (
//             mut deps,
//             wasm_client_state,
//             wasm_consensus_state,
//             mut substitute_wasm_client_state,
//             substitute_wasm_consensus_state,
//         ) = prepare_migrate_tests();

//         substitute_wasm_client_state.frozen_height = Some(FROZEN_HEIGHT);

//         save_states_to_migrate_store(
//             deps.as_mut(),
//             &wasm_client_state,
//             &substitute_wasm_client_state,
//             &wasm_consensus_state,
//             &substitute_wasm_consensus_state,
//         );

//         assert_eq!(
//             TendermintLightClient::migrate_client_store(deps.as_mut()),
//             Err(Error::MigrateClientStore(MigrateClientStoreError::SubstituteClientFrozen).into())
//         );
//     }
// }

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, num::NonZero};

    use cosmwasm_std::{
        ContractResult, SystemResult,
        testing::{mock_dependencies, mock_env},
    };
    use gno_light_client_types::Fraction;
    use hex_literal::hex;
    use ibc_union_spec::ClientId;
    use ics23::ibc_api::SDK_SPECS;
    use unionlabs::{
        encoding::{EncodeAs, EthAbi},
        google::protobuf,
        ibc::core::client::height::Height,
    };

    use super::*;

    #[test]
    fn status() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();

        let mut client_state = ClientState {
            chain_id: "osmosis-1".to_owned(),
            frozen_height: None,
            latest_height: Height::new_with_revision(1, 60157944),
            max_clock_drift: protobuf::duration::Duration::new(600, 0).unwrap(),
            proof_specs: vec![],
            trust_level: Fraction {
                numerator: 1,
                denominator: NonZero::new(3).unwrap(),
            },
            trusting_period: protobuf::duration::Duration::new(1028160, 0).unwrap(),
            unbonding_period: protobuf::duration::Duration::new(1209600, 0).unwrap(),
            upgrade_path: vec![],
            realm: "".to_owned(),
        };

        macro_rules! assert_status {
            ($Status:ident) => {
                assert_eq!(
                    GnoLightClient::status(
                        IbcClientCtx::new(
                            ClientId!(1),
                            Addr::unchecked("ibc_host"),
                            deps.as_ref(),
                            env.clone(),
                        ),
                        &client_state
                    ),
                    Status::$Status,
                );
            };
        }

        // 2026-05-01T17:42:59+00:00
        env.block.time = cosmwasm_std::Timestamp::from_seconds(1_777_657_379);

        let ts = RefCell::new(
            "2026-04-24T22:40:50.903065719Z"
                .parse::<Timestamp>()
                .unwrap(),
        );

        // frozen if the consensus state can't be read
        assert_status!(Frozen);

        deps.querier.update_wasm({
            let ts = ts.clone();
            move |wq| match wq {
                cosmwasm_std::WasmQuery::Raw { contract_addr, key }
                    if contract_addr == "ibc_host" =>
                {
                    SystemResult::Ok(ContractResult::Ok(cosmwasm_std::Binary::new(
                        ConsensusState {
                            timestamp: *ts.borrow(),
                            root: MerkleRoot {
                                hash: Default::default(),
                            },
                            next_validators_hash: Default::default(),
                        }
                        .encode_as::<EthAbi>(),
                    )))
                }
                _ => todo!(),
            }
        });

        // active within the trusting period
        assert_status!(Active);

        // frozen check works correctly
        client_state.frozen_height = Some(Height::new_with_revision(1, 1));
        assert_status!(Frozen);
        client_state.frozen_height = None;

        // not expired right at the trusting period
        env.block.time = cosmwasm_std::Timestamp::from_nanos(
            ts.borrow()
                .checked_add(client_state.trusting_period)
                .unwrap()
                .as_unix_nanos(),
        );
        assert_status!(Active);

        // expires right after the trusting period
        env.block.time = env.block.time.plus_nanos(1);
        assert_status!(Expired);

        // frozen takes priority over expired
        client_state.frozen_height = Some(Height::new_with_revision(1, 1));
        assert_status!(Frozen);
    }

    #[test]
    fn verify_proof() {
        // TODO: This is from a dev deployment, update this test with values from an actual testnet or mainnet deployment once there's one live

        let proof = r#"{"proofs":[{"@type":"exist","@value":{"key":"0x2f70762f766d3a676e6f2e6c616e642f722f636f72652f6962632f76312f636f72653a61366565663765333561626537303236373239363431313437663739313535373363376539376234376566613534366635663665333233303236336263623439","value":"0xfef470406bf3ca4daf4865ed047f1a4b9a49307e5724c21e508a8782f415889d","leaf":{"hash":"sha256","prehash_key":"no_hash","prehash_value":"sha256","length":"var_proto","prefix":"0x000206"},"path":[{"hash":"sha256","prefix":"0x020406206d81723c787f48cdb0fe48017bfeb8c7777c18d2ce768ff3bef8be989d732af920","suffix":"0x0"},{"hash":"sha256","prefix":"0x04060620424eb03f08942ba873f0ad61a3e532dd49f9ebe341c15820b91b2d6f59adb16520","suffix":"0x0"},{"hash":"sha256","prefix":"0x060a06207a6060551a75bb698b17d3eb5b11ef4b6c583d4dd65f074109c5d8ff352d874820","suffix":"0x0"},{"hash":"sha256","prefix":"0x08120620","suffix":"0x20aafb8c883b219159457c342081a41717e117622362092df6aecd69b2e823049b"},{"hash":"sha256","prefix":"0x0a2206207e05a6db10f558103a216d64fb77b010c74e9134995a502d303d1205cbe5ec2220","suffix":"0x0"},{"hash":"sha256","prefix":"0x0c420620f318b74164e6e7317df2eb259a7d3e6d0a81e9b7fc2b2b83d5407b37427a702e20","suffix":"0x0"},{"hash":"sha256","prefix":"0x0e86010620","suffix":"0x20508c3a794d8e22e86e7fa9ea4b1cf59fb581e726c4bec1a7bda9a238c830bb10"},{"hash":"sha256","prefix":"0x1086020620","suffix":"0x2028da987f98245d9cf15ae22917f42e5a2100d86cd7de7243a6a3a02519f2788a"},{"hash":"sha256","prefix":"0x1286040620","suffix":"0x2072afbfb3032a69b75dd4630d95f5916f34d5dd8ed017121f6f96a1e81d680011"},{"hash":"sha256","prefix":"0x14ca090620","suffix":"0x2042fe18a32b874de8f5c4cb122d9539e6b81b1453d991bf6013053975fe0bb975"}]}},{"@type":"exist","@value":{"key":"0x6d61696e","value":"0xf7e8b054c2e090fe8fb5ea96e815e62eb7b6ab710d211b9c9c1769f56d83a888","leaf":{"hash":"sha256","prehash_key":"no_hash","prehash_value":"sha256","length":"var_proto","prefix":"0x00"},"path":[{"hash":"sha256","prefix":"0x01ccb581a002a493db462c72bc97aac085192a8ffb6a45fa5ee3cf22ee89eb1574","suffix":"0x0"}]}}]}"#;

        verify_membership(
            "gno.land/r/core/ibc/v1/core",
            &SDK_SPECS,
            &MerkleRoot {
                hash: "UwihADwTPJO2lMK0aRr41qXcRJO6itbLR/zAKEN4bBo="
                    .parse()
                    .unwrap(),
            },
            hex!("a6eef7e35abe7026729641147f7915573c7e97b47efa546f5f6e3230263bcb49").to_vec(),
            serde_json::from_str(proof).unwrap(),
            hex!("fef470406bf3ca4daf4865ed047f1a4b9a49307e5724c21e508a8782f415889d").into(),
        )
        .unwrap();
    }
}
