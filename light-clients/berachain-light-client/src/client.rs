use cosmwasm_std::{Deps, DepsMut, Env};
use ics008_wasm_client::{
    storage_utils::{
        read_client_state, read_consensus_state, read_subject_client_state,
        read_substitute_client_state, read_substitute_consensus_state, save_client_state,
        save_consensus_state, save_subject_client_state, save_subject_consensus_state,
    },
    IbcClient, IbcClientError, Status, StorageState, WasmClientStateOf, WasmConsensusStateOf,
    ZERO_HEIGHT,
};
use tendermint_light_client::{
    client::{
        check_trusted_header, construct_partial_header, height_from_header, parse_revision_number,
        set_total_voting_power,
    },
    errors::{
        IbcHeightTooLargeForTendermintHeight, InvalidChainId, InvalidHeaderError,
        InvalidHostTimestamp, MigrateClientStoreError, RevisionNumberMismatch,
    },
    storage::{
        get_current_or_next_consensus_state_meta, get_current_or_prev_consensus_state_meta,
        save_consensus_state_metadata,
    },
};
use tendermint_verifier::types::SignatureVerifier;
use unionlabs::{
    berachain::LATEST_EXECUTION_PAYLOAD_HEADER_PREFIX,
    cosmwasm::wasm::union::custom_query::UnionCustomQuery,
    encoding::{DecodeAs, EncodeAs, Proto, Ssz},
    ensure,
    ibc::{
        core::{
            client::{genesis_metadata::GenesisMetadata, height::Height},
            commitment::{merkle_path::MerklePath, merkle_root::MerkleRoot},
        },
        lightclients::{
            berachain::{
                client_state::ClientState, consensus_state::ConsensusState, header::Header,
            },
            ethereum::storage_proof::StorageProof,
        },
    },
};

use crate::{errors::Error, verifier::Bls12_381Verifier};

pub struct BerachainLightClient;

impl IbcClient for BerachainLightClient {
    type Error = Error;

    type CustomQuery = UnionCustomQuery;

    type Header = Header;

    // TODO(aeryz): Change this to appropriate misbehavior type when it is implemented
    type Misbehaviour = Header;

    type ClientState = ClientState;

    type ConsensusState = ConsensusState;

    type Encoding = Proto;

    fn verify_membership(
        deps: Deps<Self::CustomQuery>,
        height: Height,
        _delay_time_period: u64,
        _delay_block_period: u64,
        proof: Vec<u8>,
        mut path: MerklePath,
        value: StorageState,
    ) -> Result<(), IbcClientError<Self>> {
        let consensus_state = read_consensus_state::<Self>(deps, &height)?
            .ok_or(IbcClientError::ConsensusStateNotFound(height))?;
        let client_state = read_client_state::<Self>(deps)?;

        let path = path.key_path.pop().ok_or(Error::EmptyIbcPath)?;

        // This storage root is verified during the header update, so we don't need to verify it again.
        let storage_root = consensus_state.data.eth_storage_root;

        let storage_proof =
            StorageProof::decode_as::<Proto>(&proof).map_err(Error::StorageProofDecode)?;

        match value {
            StorageState::Occupied(value) => ethereum_light_client::client::do_verify_membership(
                path,
                storage_root,
                client_state.data.ibc_commitment_slot,
                storage_proof,
                value,
            ),
            StorageState::Empty => ethereum_light_client::client::do_verify_non_membership(
                path,
                storage_root,
                client_state.data.ibc_commitment_slot,
                storage_proof,
            ),
        }
        .map_err(Error::from)?;

        Ok(())
    }

    fn verify_header(
        deps: Deps<Self::CustomQuery>,
        env: Env,
        mut header: Self::Header,
    ) -> Result<(), IbcClientError<Self>> {
        set_total_voting_power(&mut header.cometbft_header.validator_set).map_err(Error::from)?;
        set_total_voting_power(&mut header.cometbft_header.trusted_validators)
            .map_err(Error::from)?;

        let client_state = read_client_state::<Self>(deps)?;
        let consensus_state =
            read_consensus_state::<Self>(deps, &header.cometbft_header.trusted_height)?.ok_or(
                IbcClientError::ConsensusStateNotFound(header.cometbft_header.trusted_height),
            )?;

        check_trusted_header(
            &header.cometbft_header,
            &consensus_state.data.comet_next_validators_hash,
        )
        .map_err(Error::from)?;

        let revision_number = parse_revision_number(
            &header.cometbft_header.signed_header.header.chain_id,
        )
        .ok_or(Error::from(InvalidChainId(
            header.cometbft_header.signed_header.header.chain_id.clone(),
        )))?;

        if revision_number != header.cometbft_header.trusted_height.revision_number {
            return Err(Error::from(RevisionNumberMismatch {
                trusted_revision_number: revision_number,
                header_revision_number: header.cometbft_header.trusted_height.revision_number,
            })
            .into());
        }

        let signed_height = header
            .cometbft_header
            .signed_header
            .header
            .height
            .inner()
            .try_into()
            .expect("value is bounded >= 0; qed;");

        if signed_height <= header.cometbft_header.trusted_height.revision_height {
            return Err(InvalidHeaderError::SignedHeaderHeightMustBeMoreRecent {
                signed_height,
                trusted_height: header.cometbft_header.trusted_height.revision_height,
            }
            .into());
        }

        // verify the ibc account against the state root
        ethereum_verifier::verify::verify_account_storage_root(
            header.execution_header.state_root,
            &client_state.data.ibc_contract_address,
            &header.account_proof.proof,
            &header.account_proof.storage_root,
        )
        .map_err(Error::VerifyAccountStorageRoot)?;

        // verify that the execution header (and thus the state root) is actually stored in the beacon state
        ics23::ibc_api::verify_membership(
            &header.execution_header_proof,
            &client_state.data.proof_specs,
            &MerkleRoot {
                hash: header.cometbft_header.signed_header.header.app_hash,
            },
            &[
                b"beacon".to_vec(),
                [LATEST_EXECUTION_PAYLOAD_HEADER_PREFIX].to_vec(),
            ],
            header.execution_header.encode_as::<Ssz>(),
        )
        .map_err(Error::ExecutionHeaderVerify)?;

        tendermint_verifier::verify::verify(
            &construct_partial_header(
                client_state.data.consensus_chain_id,
                i64::try_from(header.cometbft_header.trusted_height.revision_height)
                    .map_err(|_| {
                        Error::from(IbcHeightTooLargeForTendermintHeight(
                            header.cometbft_header.trusted_height.revision_height,
                        ))
                    })?
                    .try_into()
                    .expect(
                        "value is converted from u64, which is positive, \
                        and the expected bounded type is >= 0; qed;",
                    ),
                consensus_state.data.comet_timestamp,
                consensus_state.data.comet_next_validators_hash,
            ),
            &header.cometbft_header.trusted_validators,
            &header.cometbft_header.signed_header,
            &header.cometbft_header.validator_set,
            client_state.data.trusting_period,
            env.block
                .time
                .try_into()
                .map_err(|_| Error::from(InvalidHostTimestamp(env.block.time)))?,
            client_state.data.max_clock_drift,
            &client_state.data.trust_level,
            &SignatureVerifier::new(Bls12_381Verifier::new(deps)),
        )
        .map_err(Error::TendermintVerify)?;

        Ok(())
    }

    fn verify_misbehaviour(
        _deps: Deps<Self::CustomQuery>,
        _env: Env,
        _misbehaviour: Self::Misbehaviour,
    ) -> Result<(), IbcClientError<Self>> {
        Err(Error::Unimplemented.into())
    }

    fn update_state(
        mut deps: DepsMut<Self::CustomQuery>,
        _env: Env,
        header: Self::Header,
    ) -> Result<Vec<Height>, IbcClientError<Self>> {
        let update_height = height_from_header(&header.cometbft_header);
        if read_consensus_state::<Self>(deps.as_ref(), &update_height)?.is_some() {
            return Ok(vec![update_height]);
        }

        // TODO(aeryz): prune oldest expired consensus state

        let mut client_state = read_client_state::<Self>(deps.as_ref())?;

        if update_height > client_state.latest_height {
            client_state.latest_height = update_height;
            client_state.data.latest_height = update_height;
        }

        save_client_state::<Self>(deps.branch(), client_state);
        save_consensus_state_metadata(
            deps.branch(),
            header.cometbft_header.signed_header.header.time,
            update_height,
        );
        save_consensus_state::<Self>(
            deps,
            WasmConsensusStateOf::<Self> {
                data: ConsensusState {
                    eth_timestamp: header.execution_header.timestamp,
                    comet_next_validators_hash: header
                        .cometbft_header
                        .signed_header
                        .header
                        .next_validators_hash,
                    eth_storage_root: header.account_proof.storage_root,
                    comet_timestamp: header.cometbft_header.signed_header.header.time,
                },
            },
            &update_height,
        );

        Ok(vec![update_height])
    }

    fn update_state_on_misbehaviour(
        _deps: DepsMut<Self::CustomQuery>,
        _env: Env,
        _client_message: Vec<u8>,
    ) -> Result<(), IbcClientError<Self>> {
        Err(Error::Unimplemented.into())
    }

    fn check_for_misbehaviour_on_header(
        deps: Deps<Self::CustomQuery>,
        header: Self::Header,
    ) -> Result<bool, IbcClientError<Self>> {
        let height = height_from_header(&header.cometbft_header);

        // If there is already a header at this height, it should be exactly the same as the header that
        // we saved previously. If this is not the case, either the client is broken or the chain is
        // broken. Because it should not be possible to have two distinct valid headers at a height.
        if let Some(WasmConsensusStateOf::<Self> {
            data:
                ConsensusState {
                    comet_timestamp,
                    comet_next_validators_hash,
                    ..
                },
        }) = read_consensus_state::<Self>(deps, &height)?
        {
            if comet_timestamp != header.cometbft_header.signed_header.header.time
                || comet_next_validators_hash
                    != header
                        .cometbft_header
                        .signed_header
                        .header
                        .next_validators_hash
            {
                return Ok(true);
            }

            // We don't need to check for previous or next consensus state since we know that we already
            // saved this header correctly previously.
            return Ok(false);
        }

        if let Ok(Some((_, next_consensus_state))) =
            get_current_or_next_consensus_state_meta(deps, height)
        {
            // next (in terms of height) consensus state must have a larger timestamp
            if next_consensus_state.timestamp <= header.cometbft_header.signed_header.header.time {
                return Ok(true);
            }
        }

        if let Ok(Some((_, prev_consensus_state))) =
            get_current_or_prev_consensus_state_meta(deps, height)
        {
            // previous (in terms of height) consensus state must have a smaller timestamp
            if prev_consensus_state.timestamp >= header.cometbft_header.signed_header.header.time {
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn check_for_misbehaviour_on_misbehaviour(
        _deps: Deps<Self::CustomQuery>,
        _misbehaviour: Self::Misbehaviour,
    ) -> Result<bool, IbcClientError<Self>> {
        Err(Error::Unimplemented.into())
    }

    fn verify_upgrade_and_update_state(
        _deps: DepsMut<Self::CustomQuery>,
        _upgrade_client_state: Self::ClientState,
        _upgrade_consensus_state: Self::ConsensusState,
        _proof_upgrade_client: Vec<u8>,
        _proof_upgrade_consensus_state: Vec<u8>,
    ) -> Result<(), IbcClientError<Self>> {
        Err(Error::Unimplemented.into())
    }

    fn migrate_client_store(
        mut deps: DepsMut<Self::CustomQuery>,
    ) -> Result<(), IbcClientError<Self>> {
        let subject_client_state = read_subject_client_state::<Self>(deps.as_ref())?;
        let substitute_client_state = read_substitute_client_state::<Self>(deps.as_ref())?;

        ensure(
            substitute_client_state
                .data
                .frozen_height
                .unwrap_or(ZERO_HEIGHT)
                == ZERO_HEIGHT,
            MigrateClientStoreError::SubstituteClientFrozen,
        )?;

        // TODO: Figure out a half-decent way to verify this
        // ensure(
        //     migrate_check_allowed_fields(&subject_client_state.data, &substitute_client_state.data),
        //     MigrateClientStoreError::MigrateFieldsChanged,
        // )?;

        let substitute_consensus_state: WasmConsensusStateOf<Self> =
            read_substitute_consensus_state(deps.as_ref(), &substitute_client_state.latest_height)?
                .ok_or(IbcClientError::ConsensusStateNotFound(
                    substitute_client_state.latest_height,
                ))?;

        save_consensus_state_metadata(
            deps.branch(),
            substitute_consensus_state.data.comet_timestamp,
            substitute_client_state.latest_height,
        );

        save_subject_consensus_state::<Self>(
            deps.branch(),
            substitute_consensus_state,
            &substitute_client_state.latest_height,
        );

        let scs = substitute_client_state.data;
        save_subject_client_state::<Self>(
            deps,
            WasmClientStateOf::<Self> {
                data: ClientState {
                    consensus_chain_id: scs.consensus_chain_id,
                    trusting_period: scs.trusting_period,
                    latest_height: scs.latest_height,
                    frozen_height: None,
                    ..subject_client_state.data
                },
                checksum: subject_client_state.checksum,
                latest_height: scs.latest_height,
            },
        );

        Ok(())
    }

    fn status(
        deps: Deps<Self::CustomQuery>,
        _env: &cosmwasm_std::Env,
    ) -> Result<Status, IbcClientError<Self>> {
        let client_state = read_client_state::<Self>(deps)?;

        // TODO(aeryz): when refactoring the tm client, we should consider making this non-optional
        // because otherwise we always have to check if the inner height is zero.
        if client_state.data.frozen_height.unwrap_or(ZERO_HEIGHT) != ZERO_HEIGHT {
            return Ok(Status::Frozen);
        }

        // TODO: Re-enable
        // let Some(consensus_state) =
        //     read_consensus_state::<Self>(deps, &client_state.latest_height)?
        // else {
        //     return Ok(Status::Expired);
        // };

        // if tendermint_light_client::client::is_client_expired(
        //     &consensus_state.data.comet_timestamp,
        //     client_state.data.trusting_period,
        //     env.block
        //         .time
        //         .try_into()
        //         .map_err(|_| Error::from(InvalidHostTimestamp(env.block.time)))?,
        // ) {
        //     return Ok(Status::Expired);
        // }

        Ok(Status::Active)
    }

    fn export_metadata(
        _deps: Deps<Self::CustomQuery>,
        _env: &cosmwasm_std::Env,
    ) -> Result<Vec<GenesisMetadata>, IbcClientError<Self>> {
        Ok(Vec::new())
    }

    fn timestamp_at_height(
        deps: Deps<Self::CustomQuery>,
        height: Height,
    ) -> Result<u64, IbcClientError<Self>> {
        Ok(read_consensus_state::<Self>(deps, &height)?
            .ok_or(IbcClientError::ConsensusStateNotFound(height))?
            .data
            .eth_timestamp)
    }
}

#[cfg(test)]
mod tests {
    use std::{marker::PhantomData, str::FromStr};

    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage},
        OwnedDeps,
    };
    use ethereum_light_client::client::test_utils::custom_query_handler;
    use unionlabs::{
        google::protobuf::timestamp::Timestamp,
        ibc::lightclients::{berachain, wasm},
    };

    use super::*;

    #[test]
    fn client_message_decode() {
        let raw = hex::decode("0ad2070af0040a92030a02080b120c626561636f6e642d3230363118b131220b08cec094b30610ccb2f5742a480a20c769fd1c7d0e5e1eafc1aba1704922d435f6c5f8f8c685e2062e4a6ae3e66ddf1224080112204ffb41666fbf636f14be2004d0e0f26e650c336be96fcd0958b4cfedd52f1b7c32209699d99529b297011661bf486a4967fa3f294495f3eeb267cc4dec56cf81614c3a20d08c972ee252a74d9fe7a0e5052a8d4e735f5bea0499cb668253f43898847c394220bc24b70832d067cf764dd53894fd03607a5714ecc9c4e02d5049f1f3086b5c904a20bc24b70832d067cf764dd53894fd03607a5714ecc9c4e02d5049f1f3086b5c90522068ecd6f333119ce43751ece583b981f23508aeaf4221ff582b1bb33be42bcefa5a2067d5dd67a274b98b05a64f990a96a9485deacb45adc59963cdbbf881b3d3f8fc6220bd9700632f7ec94359b209c966bebe67bada2e4fc855a20fb50b69c672fb6d956a20e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b85572140e6cc28d1e5dd03e5b8b20556cd74eefd09878fd12d80108b1311a480a20901ca8fc098583f4d286254da5218b88f9e2fd9aeffae2b82c3982a3fcdccae9122408011220108544c872fe1908bfbccf57b6d632ff5f267e5f7ea8b7da743cbbb3d31b1c9b228801080212140e6cc28d1e5dd03e5b8b20556cd74eefd09878fd1a0c08cfc094b30610ccddbe99012260ae44fb2beaaee80fe4f355318748795b41d670bad78ad96b9222a5aa4faa7187371c42278d31f90a824c4ddce70860f20d9c5def2639ae8a629b941e4527e9fb21deae1bc95d3b03da27a00c6949a2b5be0ea2f18f6878019eb1117697c8f16812aa010a500a140e6cc28d1e5dd03e5b8b20556cd74eefd09878fd12322230a94d1d36bdc1cf19fa275f9b784032b829b57fe35dd4fe675fe0325ba3838e6142861470e43c12db904872806dc6e571188080e59a7712500a140e6cc28d1e5dd03e5b8b20556cd74eefd09878fd12322230a94d1d36bdc1cf19fa275f9b784032b829b57fe35dd4fe675fe0325ba3838e6142861470e43c12db904872806dc6e571188080e59a77188080e59a771a0310a81922aa010a500a140e6cc28d1e5dd03e5b8b20556cd74eefd09878fd12322230a94d1d36bdc1cf19fa275f9b784032b829b57fe35dd4fe675fe0325ba3838e6142861470e43c12db904872806dc6e571188080e59a7712500a140e6cc28d1e5dd03e5b8b20556cd74eefd09878fd12322230a94d1d36bdc1cf19fa275f9b784032b829b57fe35dd4fe675fe0325ba3838e6142861470e43c12db904872806dc6e571188080e59a77188080e59a7712d2040a2008dbcd1e159769b93a75a70a902cd7fef7be0e8dee10cee2a74bae71b4acef12121400000000000000000000000000000000000000001a20eeb8d4d0877127afabfc5e518a8900d2110a3f6d85ba914a7d0240ea39651102222056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b4212a800200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000322030fac3726d7da65b3d9d42349208009ae7b3ea48255082d9f1157b4aec6b558538b131408087a70e50d1c094b3065a19d883010e06846765746888676f312e32322e34856c696e7578622000000000000000000000000000000000000000000000000000000000000000076a2028cafa78edb556a36c1dd447d9c72dee22ab454a818a0a0d78286fcc8d54417372207ffe241ea60187fdb0187bfa22de35d1f9bed7ab061d9401fd47e34a54fbede17a200d6bb8b66af67a2cd71e843cdafccc8f82f96ab8b9cce906841ce4e4105c3d261a96070ada060ad7060a011112e10408dbcd1e159769b93a75a70a902cd7fef7be0e8dee10cee2a74bae71b4acef120000000000000000000000000000000000000000eeb8d4d0877127afabfc5e518a8900d2110a3f6d85ba914a7d0240ea3965110256e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b4210000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000030fac3726d7da65b3d9d42349208009ae7b3ea48255082d9f1157b4aec6b5585b11800000000000080c3c901000000000000000000000000512065660000000048020000070000000000000000000000000000000000000000000000000000000000000028cafa78edb556a36c1dd447d9c72dee22ab454a818a0a0d78286fcc8d5441737ffe241ea60187fdb0187bfa22de35d1f9bed7ab061d9401fd47e34a54fbede10d6bb8b66af67a2cd71e843cdafccc8f82f96ab8b9cce906841ce4e4105c3d2600000000000000000000000000000000d883010e06846765746888676f312e32322e34856c696e75781a0c0801180120012a040002e262222c080112050204e262201a2120d8df974382b81c159f53ccc8c34c4f12efd9ca3090583acfb536194331b3c99b222c080112050408e262201a2120650b6146891216605836a22e6878a60a25a92d1ca4ccb2e806808839e7b6d39e222c080112050812e262201a21201056682056855919c038f92cccd12e437a7330b501fa3182ccf19db36b249e14222a080112260a32e26220c915b343bca24ec60cf2975ac8d1c4ddcdaf70d188e5505640df25854b47564220222a080112260c62e262207241328b874082bedad414ed9ec4f5910de83fd30dbc4e29e924ae35472e3ada200a370a350a06626561636f6e12205e0a44602e323088c04529f64c8df629dfa658a34371eab0fa49fbd87158f4441a090801180120012a010022bc060a20415be020d1d0fb2f9ef05f5f9c8da365f4832d0e555e4d0187e2e4e33e86cfe7129404f90211a08fb7125541be82928093fe7036ee16c81e9dc9d7f0973c476f91c253d452253da00f662c5b3199a45f2591320f7164b86d6a919428cc9be52acb64cb15687bb59ea0ef99a8b347085f2c5057344a3abb0fa6c73856157c66bcc8feb30e23d8a88840a0a9c41fdf5dfe7ce55b11b6dc0ff5cf1661be7661f81ecf9fbddf91ec606a379aa0a6e65ea772150053523c0e05ef50bc44e09ed43bfb71b38a0dbd8b4a81c1d81ba039f184a4993e8a7110f2b2fb6fa7d9fd38ec26c379e3a939c056895bb45de1f5a0cec268c663a0e22144e9eb5f0134e5d9f56081605ed62668d05be5a8c8099171a04df88d312fbd456db4238f936029dd7385cdaa98cb5ff5672be24edc30dce8b1a0b53ed40667bcaf565143e934acc4d6fb008c36b1e5be8ba62f7e49204fb766f2a026a5f81000b26d485520a5801225e7c58da5a2dcf39fd1233cce03b610fb42daa017f38830390e70ab06cf9e152f054329fd2ec46ceb975bce67edeb79873ad855a04816058a6a60b089e7b011d8eeb1fa55c84397fb263e938797d2be390efa9e06a09c571a0b1acbf273d9c6cadaf976c84918dc21f68145274f88757fd8624ab63ca0757acad8814c46ce9b291f8bfa1aabf878c162c6c3fa422be9b1bb0730cf9dc2a0e4083cad4ce79d5c932fbf47cf9ad95041b77043d2079e173ce92e9098064dc8a0ee4183a6caf662c3cad1e58acf63c145813f0e96c8ab8bb6754b0423c492eef080129301f891a02589b2a7f4dd4933b58c4f3e0bab8064f3e9d1197f7d1dea3452098f67514c7c8080a06ed73802a89aa9429cb614cb88eb0bd8c81e7ddf3e00a7cd1f400829f02f796880a0183a55d7327047a7191d3305410733446c8dabd6b08da6ade338bd5fbe0e3ad380808080a0f0242db40fb34b7b46fbdc1c2e1de70fccde86b3cd9b5d2a9f3f1ec8e1af6a2d808080808080126bf869a020d32623befacb9313eead27e141fdc3277aa60b482dc2212f03e278b726f51bb846f8440180a0415be020d1d0fb2f9ef05f5f9c8da365f4832d0e555e4d0187e2e4e33e86cfe7a0de128a53cd6aa844853c7789b39f5ae3d62300166c2034c6a4e95a5932ed854f").unwrap();

        let res = <BerachainLightClient as IbcClient>::Header::decode_as::<
            <BerachainLightClient as IbcClient>::Encoding,
        >(&raw)
        .unwrap();

        dbg!(res);
    }

    #[test]
    fn update() {
        let header = serde_json::from_str::<berachain::header::Header>(r#"{"account_proof":{"proof":["0xf90211a08fb7125541be82928093fe7036ee16c81e9dc9d7f0973c476f91c253d452253da00f662c5b3199a45f2591320f7164b86d6a919428cc9be52acb64cb15687bb59ea0ef99a8b347085f2c5057344a3abb0fa6c73856157c66bcc8feb30e23d8a88840a0b65e4c694a53697c1e68ede137e7e975a5ffbb6fabd0ab3748827bdfa11f9192a0a6e65ea772150053523c0e05ef50bc44e09ed43bfb71b38a0dbd8b4a81c1d81ba0aef9f6ea66cf81819eac5fb921014e0434b1e8941c9e357c10f3cec16ae0ba07a0cec268c663a0e22144e9eb5f0134e5d9f56081605ed62668d05be5a8c8099171a04df88d312fbd456db4238f936029dd7385cdaa98cb5ff5672be24edc30dce8b1a0f62800a2afacc25314545567a7f04a331d4c07461f1bb73227e1e24d99a88c01a026a5f81000b26d485520a5801225e7c58da5a2dcf39fd1233cce03b610fb42daa017f38830390e70ab06cf9e152f054329fd2ec46ceb975bce67edeb79873ad855a04816058a6a60b089e7b011d8eeb1fa55c84397fb263e938797d2be390efa9e06a09c571a0b1acbf273d9c6cadaf976c84918dc21f68145274f88757fd8624ab63ca0757acad8814c46ce9b291f8bfa1aabf878c162c6c3fa422be9b1bb0730cf9dc2a0f522ad04bfc2854c954c0a042f2d0b216b400bd024151e5f8b6e7c245c886d6da0ee4183a6caf662c3cad1e58acf63c145813f0e96c8ab8bb6754b0423c492eef080","0xf891a02589b2a7f4dd4933b58c4f3e0bab8064f3e9d1197f7d1dea3452098f67514c7c8080a016f1ba3f4e735d5a61e7ec5afaab8e0992b75ee647165a593e4a5b7b85c5c45680a0183a55d7327047a7191d3305410733446c8dabd6b08da6ade338bd5fbe0e3ad380808080a0f0242db40fb34b7b46fbdc1c2e1de70fccde86b3cd9b5d2a9f3f1ec8e1af6a2d808080808080","0xf869a020d32623befacb9313eead27e141fdc3277aa60b482dc2212f03e278b726f51bb846f8440180a06911ea1077a835f86fc220717278216c4816693113d47525746d7d1522544bada0de128a53cd6aa844853c7789b39f5ae3d62300166c2034c6a4e95a5932ed854f"],"storage_root":"0x6911ea1077a835f86fc220717278216c4816693113d47525746d7d1522544bad"},"cometbft_header":{"signed_header":{"commit":{"round":0,"height":128,"block_id":{"hash":"0x3d437b139c9e97a95612775548a62d608fe7e7c5d5d609ce382daaa6c219121d","part_set_header":{"hash":"0xe17c7b2d4fa6362bc59d4d9de2721a674f15a1f9c6d54d5585929e939fcb72fc","total":1}},"signatures":[{"@type":"commit","@value":{"signature":"0x87aaec647f47fd0ecac9ed13aa1bb106a4c67c77fec62a18c092abd718aaaf127886cf073f4c968d017d41bce6c79d7201d8f725c75dd03aa7ef83ad0b49dafa0e055b8bf7e993c0e381ca6e7831ee01d686a6e73839ea73648c90d0bb1d04c8","timestamp":"2024-06-09T07:07:58.946278555Z","validator_address":"0x4ac691fccbbf4992eb5f1365f3e49108ea2e9367"}}]},"header":{"time":"2024-06-09T07:07:57.877034196Z","height":"128","version":{"app":"0","block":"11"},"app_hash":"0x0730a257675d0a499fffec44d6c9826ca026181996f9f66c6c8ce2b7895ce616","chain_id":"beacond-2061","data_hash":"0xb81b0a62dbcb7b9771886bc7acae6e63f9691dd500e8a8e54479d619603d7423","evidence_hash":"0xe3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855","last_block_id":{"hash":"0x06c39f622be3a53a366e582d597b847b02f1816aaf8f458442001f97590fb92f","part_set_header":{"hash":"0x0c248d81b96e9a26922451d5aef3bf1ce455c648d0b66b6a71b03d665fe16ecb","total":1}},"consensus_hash":"0x68ecd6f333119ce43751ece583b981f23508aeaf4221ff582b1bb33be42bcefa","validators_hash":"0xf79cb1e942656d6189798a768e071bfdd8419a27d4461972cceab87fb8299341","last_commit_hash":"0xf8b4942e8fddbc5b00d27966cf3d978a28e99b00db4a790842c58943094da17c","proposer_address":"0x4ac691fccbbf4992eb5f1365f3e49108ea2e9367","last_results_hash":"0xbd9700632f7ec94359b209c966bebe67bada2e4fc855a20fb50b69c672fb6d95","next_validators_hash":"0xf79cb1e942656d6189798a768e071bfdd8419a27d4461972cceab87fb8299341"}},"validator_set":{"proposer":{"address":"0x4ac691fccbbf4992eb5f1365f3e49108ea2e9367","pub_key":{"@type":"bls12_381","@value":"0xab43d87000d094a8ab2d0af3c6e91d85c16e48e630e00c6774f1e0d670494ab8ed17ac0770d424de11c34369397469ad"},"voting_power":32000000000,"proposer_priority":0},"validators":[{"address":"0x4ac691fccbbf4992eb5f1365f3e49108ea2e9367","pub_key":{"@type":"bls12_381","@value":"0xab43d87000d094a8ab2d0af3c6e91d85c16e48e630e00c6774f1e0d670494ab8ed17ac0770d424de11c34369397469ad"},"voting_power":32000000000,"proposer_priority":0}],"total_voting_power":32000000000},"trusted_height":{"revision_height":91,"revision_number":2061},"trusted_validators":{"proposer":{"address":"0x4ac691fccbbf4992eb5f1365f3e49108ea2e9367","pub_key":{"@type":"bls12_381","@value":"0xab43d87000d094a8ab2d0af3c6e91d85c16e48e630e00c6774f1e0d670494ab8ed17ac0770d424de11c34369397469ad"},"voting_power":32000000000,"proposer_priority":0},"validators":[{"address":"0x4ac691fccbbf4992eb5f1365f3e49108ea2e9367","pub_key":{"@type":"bls12_381","@value":"0xab43d87000d094a8ab2d0af3c6e91d85c16e48e630e00c6774f1e0d670494ab8ed17ac0770d424de11c34369397469ad"},"voting_power":32000000000,"proposer_priority":0}],"total_voting_power":32000000000}},"execution_header":{"gas_used":"0","gas_limit":"30000000","timestamp":"1717916879","block_hash":"0x06e8e7e7d81b8755e98ce1a32dcf4e7fa6a450161a8bea4b60172737e835eb2f","extra_data":"0xd883010e06846765746888676f312e32322e34856c696e7578","logs_bloom":"0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000","state_root":"0x8072e7bd4c6f29da77938b7dc2ef25a5b4d7e75e3cd19fe72d97119d442bdf37","parent_hash":"0x2968cf50d857da85c9231591940dcdec66e5626a5d7a39eff74ac6c507c37f3f","prev_randao":"0x3d918d6c81b5a52d2e595c7beb3864842cd02a4789e2881a037a75610213b18b","block_number":"127","blob_gas_used":"0","fee_recipient":"0x0000000000000000000000000000000000000000","receipts_root":"0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421","excess_blob_gas":"0","base_fee_per_gas":"60","withdrawals_root":"0xe23c3e89d78dc874cd12c50c52c91575ce215ee6c4c473e1ad82abee9c0369c1","transactions_root":"0x7ffe241ea60187fdb0187bfa22de35d1f9bed7ab061d9401fd47e34a54fbede1"},"execution_header_proof":{"proofs":[{"@type":"exist","@value":{"key":"0x11","leaf":{"hash":"sha256","length":"var_proto","prefix":"0x0002fe01","prehash_key":"no_hash","prehash_value":"sha256"},"path":[{"hash":"sha256","prefix":"0x0204fe0120","suffix":"0x20e52a484769165b8807081724bfd0c52c3b7fef9b1f4626046034a64326dad3b6"},{"hash":"sha256","prefix":"0x0408fe0120","suffix":"0x2084a3f0ce31f51078b5f0fc0af29b9fb5f92122a53e14851c5c5aee5bedb4cd35"},{"hash":"sha256","prefix":"0x0812fe0120","suffix":"0x20a6b90031003a8562d5669f967391347f95d37fd9b6a5bc4dd4f7d0229a7d6f83"},{"hash":"sha256","prefix":"0x0a32fe01206203f6a4f5cd3f9b0b2a412fed2c85851844ecb1faa83791d65fee92a05951ff20","suffix":"0x0"},{"hash":"sha256","prefix":"0x0c58fe0120905f7ace9135feb8d6d237d7d6913b4d8a8fdd0fceabed36ad0fe6bcfe03392720","suffix":"0x0"}],"value":"0x2968cf50d857da85c9231591940dcdec66e5626a5d7a39eff74ac6c507c37f3f00000000000000000000000000000000000000008072e7bd4c6f29da77938b7dc2ef25a5b4d7e75e3cd19fe72d97119d442bdf3756e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003d918d6c81b5a52d2e595c7beb3864842cd02a4789e2881a037a75610213b18b7f0000000000000080c3c901000000000000000000000000cf54656600000000480200003c0000000000000000000000000000000000000000000000000000000000000006e8e7e7d81b8755e98ce1a32dcf4e7fa6a450161a8bea4b60172737e835eb2f7ffe241ea60187fdb0187bfa22de35d1f9bed7ab061d9401fd47e34a54fbede1e23c3e89d78dc874cd12c50c52c91575ce215ee6c4c473e1ad82abee9c0369c100000000000000000000000000000000d883010e06846765746888676f312e32322e34856c696e7578"}},{"@type":"exist","@value":{"key":"0x626561636f6e","leaf":{"hash":"sha256","length":"var_proto","prefix":"0x00","prehash_key":"no_hash","prehash_value":"sha256"},"path":[],"value":"0x0097a7c20b179c3072b9cc15d39785a8e2ab828cbbc2cf7e1c08640463e6c1a2"}}]}}"#).unwrap();

        // dbg!(&header);

        let client_state = serde_json::from_str::<wasm::client_state::ClientState<berachain::client_state::ClientState>>(r#"{"checksum":"0x9fb5b96177354ffad3fd1f0707f862dda0a651c933fe72d4a392ffab3934d8d8","data":{"consensus_chain_id":"beacond-2061","execution_chain_id":"80085","frozen_height":null,"ibc_commitment_slot":"0","ibc_contract_address":"0x9664d804589c1c9f573288edc2b9027661d624fa","latest_height":{"revision_height":91,"revision_number":2061},"max_clock_drift":"600s","proof_specs":[{"inner_spec":{"child_order":[0,1],"child_size":33,"empty_child":"0x0","hash":"sha256","max_prefix_length":12,"min_prefix_length":4},"leaf_spec":{"hash":"sha256","length":"var_proto","prefix":"0x00","prehash_key":"no_hash","prehash_value":"sha256"},"max_depth":null,"min_depth":null,"prehash_key_before_comparison":false},{"inner_spec":{"child_order":[0,1],"child_size":32,"empty_child":"0x0","hash":"sha256","max_prefix_length":1,"min_prefix_length":1},"leaf_spec":{"hash":"sha256","length":"var_proto","prefix":"0x00","prehash_key":"no_hash","prehash_value":"sha256"},"max_depth":null,"min_depth":null,"prehash_key_before_comparison":false}],"trust_level":{"denominator":3,"numerator":1},"trusting_period":"21420s","upgrade_path":["upgrade","upgradedIBCState"]},"latest_height":{"revision_height":91,"revision_number":2061}}"#).unwrap();

        let consensus_state = serde_json::from_str::<wasm::consensus_state::ConsensusState<berachain::consensus_state::ConsensusState>>(r#"{"data":{"comet_next_validators_hash":"0xf79cb1e942656d6189798a768e071bfdd8419a27d4461972cceab87fb8299341","comet_timestamp":"2024-06-09T07:07:18.253461685Z","eth_storage_root":"0xc0d6dc20fa4a581e2fb5ce8d5983c9790c618267583739c71b6e6e40a5698ddc","eth_timestamp":1717916840}}"#).unwrap();

        dbg!(&header);

        let mut deps = mk_deps();

        ics008_wasm_client::storage_utils::save_client_state::<BerachainLightClient>(
            deps.as_mut(),
            client_state,
        );

        ics008_wasm_client::storage_utils::save_consensus_state::<BerachainLightClient>(
            deps.as_mut(),
            consensus_state,
            &Height {
                revision_number: 2061,
                revision_height: 91,
            },
        );

        let mut env = mock_env();
        env.block.time = Timestamp::from_str("2024-06-09T07:07:57.877034196Z")
            .unwrap()
            .into();
        BerachainLightClient::verify_header(deps.as_ref(), env, header).unwrap();
    }

    fn mk_deps() -> OwnedDeps<MockStorage, MockApi, MockQuerier<UnionCustomQuery>, UnionCustomQuery>
    {
        let deps = mock_dependencies();

        OwnedDeps {
            storage: deps.storage,
            api: deps.api,
            querier: MockQuerier::<UnionCustomQuery>::new(&[])
                .with_custom_handler(custom_query_handler),
            custom_query_type: PhantomData,
        }
    }
}
