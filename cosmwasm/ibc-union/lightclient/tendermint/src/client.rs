use cometbft_types::{
    crypto::public_key::PublicKey,
    types::{commit::Commit, signed_header::SignedHeader, validator_set::ValidatorSet},
};
use cosmwasm_std::{Addr, Empty};
use ibc_union_light_client::{
    ClientCreationResult, IbcClient, IbcClientCtx, IbcClientError, StateUpdate, spec::Status,
};
use ibc_union_spec::{Timestamp, path::IBC_UNION_COSMWASM_COMMITMENT_PREFIX};
use ics23::ibc_api::SDK_SPECS;
use tendermint_light_client_types::{ClientState, ConsensusState, Header};
use tendermint_verifier::types::Verification;
use unionlabs::{
    bounded::BoundedI64,
    encoding::Bincode,
    ibc::core::{
        client::height::Height,
        commitment::{merkle_proof::MerkleProof, merkle_root::MerkleRoot},
    },
    primitives::{H256, encoding::HexUnprefixed},
};

use crate::{
    errors::{
        Error, IbcHeightTooLargeForTendermintHeight, InvalidChainId, InvalidHeaderError,
        MathOverflow, RevisionNumberMismatch, TrustedValidatorsMismatch,
    },
    verifier::Ed25519Verifier,
};

pub struct TendermintLightClient;

impl IbcClient for TendermintLightClient {
    type Error = Error;

    type CustomQuery = Empty;

    type Header = Header;

    // TODO(aeryz): Change this to appropriate misbehavior type when it is implemented
    type Misbehaviour = Header;

    type ClientState = ClientState;

    type ConsensusState = ConsensusState;

    type StorageProof = MerkleProof;

    type Encoding = Bincode;

    fn verify_membership(
        ctx: IbcClientCtx<Self>,
        height: u64,
        key: Vec<u8>,
        storage_proof: Self::StorageProof,
        value: Vec<u8>,
    ) -> Result<(), IbcClientError<Self>> {
        let client_state = ctx.read_self_client_state()?;
        let consensus_state = ctx.read_self_consensus_state(height)?;

        verify_membership(
            &client_state.contract_address,
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
        storage_proof: Self::StorageProof,
    ) -> Result<(), IbcClientError<Self>> {
        let client_state = ctx.read_self_client_state()?;
        let consensus_state = ctx.read_self_consensus_state(height)?;

        verify_non_membership(
            &client_state.contract_address,
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
        match header.validator_set.validators.first().map(|v| &v.pub_key) {
            #[cfg(feature = "bls")]
            Some(PublicKey::Bls12_381(_)) => Ok(verify_header(
                client_state,
                consensus_state,
                header,
                ctx.env.block.time,
                &SignatureVerifier::new(crate::verifier::bls::Bls12Verifier::new(ctx.deps)),
            )?),
            Some(PublicKey::Ed25519(_)) => Ok(verify_header(
                client_state,
                consensus_state,
                header,
                ctx.env.block.time,
                Ed25519Verifier::new(ctx.deps),
            )?),
            _ => Err(Error::InvalidValidatorSet.into()),
        }
    }

    fn misbehaviour(
        _ctx: IbcClientCtx<Self>,
        _caller: Addr,
        _misbehaviour: Self::Misbehaviour,
        _relayer: Addr,
    ) -> Result<Self::ClientState, IbcClientError<Self>> {
        Err(Error::Unimplemented.into())
    }

    fn status(ctx: IbcClientCtx<Self>, client_state: &Self::ClientState) -> Status {
        let _ = ctx;

        // FIXME: read latest consensus to verify if client expired
        // if is_client_expired(
        //     &consensus_state.timestamp,
        //     client_state.trusting_period,
        //     env.block
        //         .time
        //         .try_into()
        //         .map_err(|_| Error::from(InvalidHostTimestamp(env.block.time)))?,
        // ) {
        //     return Ok(Status::Expired);
        // }
        if client_state.frozen_height.unwrap_or_default().height() != 0 {
            Status::Frozen
        } else {
            Status::Active
        }
    }

    fn get_timestamp(consensus_state: &Self::ConsensusState) -> Timestamp {
        Timestamp::from_nanos(consensus_state.timestamp.as_unix_nanos())
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

pub fn verify_header<V: Verification>(
    mut client_state: ClientState,
    consensus_state: ConsensusState,
    mut header: Header,
    block_timestamp: cosmwasm_std::Timestamp,
    mut signature_verifier: V,
) -> Result<StateUpdate<TendermintLightClient>, Error> {
    set_total_voting_power(&mut header.validator_set).map_err(Error::from)?;
    set_total_voting_power(&mut header.trusted_validators).map_err(Error::from)?;

    check_trusted_header(&header, consensus_state.next_validators_hash.as_encoding())
        .map_err(Error::from)?;

    let revision_number = parse_revision_number(&header.signed_header.header.chain_id).ok_or(
        Error::from(InvalidChainId(header.signed_header.header.chain_id.clone())),
    )?;

    if revision_number != header.trusted_height.revision() {
        return Err(Error::from(RevisionNumberMismatch {
            trusted_revision_number: revision_number,
            header_revision_number: header.trusted_height.revision(),
        }));
    }

    let signed_height = header
        .signed_header
        .header
        .height
        .inner()
        .try_into()
        .expect("value is bounded >= 0; qed;");

    if signed_height <= header.trusted_height.height() {
        return Err(InvalidHeaderError::SignedHeaderHeightMustBeMoreRecent {
            signed_height,
            trusted_height: header.trusted_height.height(),
        }
        .into());
    }

    // FIXME: unionlabs is tied to cosmwasm <2, the TryFrom impl can't be used
    let block_timestamp_proto = unionlabs::google::protobuf::timestamp::Timestamp {
        seconds: i64::try_from(block_timestamp.seconds())
            .expect("impossible")
            .try_into()
            .expect("impossible"),
        nanos: i32::try_from(block_timestamp.subsec_nanos())
            .expect("impossible")
            .try_into()
            .expect("impossible"),
    };

    tendermint_verifier::verify::verify(
        &construct_partial_header(
            client_state.chain_id.clone(),
            i64::try_from(header.trusted_height.height())
                .map_err(|_| {
                    Error::from(IbcHeightTooLargeForTendermintHeight(
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
        block_timestamp_proto,
        client_state.max_clock_drift,
        &client_state.trust_level,
        &mut signature_verifier,
    )
    .map_err(Error::TendermintVerify)?;

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
                hash: (*header.signed_header.header.app_hash.get()).into(),
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

pub fn set_total_voting_power(validator_set: &mut ValidatorSet) -> Result<(), MathOverflow> {
    validator_set.total_voting_power =
        validator_set
            .validators
            .iter()
            .try_fold(0_i64, |acc, val| {
                acc.checked_add(val.voting_power.inner())
                    .ok_or(MathOverflow)
            })?;
    Ok(())
}

pub fn construct_partial_header(
    chain_id: String,
    height: BoundedI64<0, { i64::MAX }>,
    time: unionlabs::google::protobuf::timestamp::Timestamp,
    next_validators_hash: H256<HexUnprefixed>,
) -> SignedHeader {
    SignedHeader {
        header: cometbft_types::types::header::Header {
            chain_id,
            time,
            next_validators_hash,
            height,
            version: Default::default(),
            last_block_id: Default::default(),
            last_commit_hash: Default::default(),
            data_hash: Default::default(),
            validators_hash: Default::default(),
            consensus_hash: Default::default(),
            app_hash: Default::default(),
            last_results_hash: Default::default(),
            evidence_hash: Default::default(),
            proposer_address: Default::default(),
        },
        commit: Commit {
            height,
            round: 0.try_into().expect("impossible"),
            block_id: Default::default(),
            signatures: Default::default(),
        },
    }
}

pub fn is_client_expired(
    consensus_state_timestamp: &unionlabs::google::protobuf::timestamp::Timestamp,
    trusting_period: unionlabs::google::protobuf::duration::Duration,
    current_block_time: unionlabs::google::protobuf::timestamp::Timestamp,
) -> bool {
    if let Some(sum) = consensus_state_timestamp.checked_add(trusting_period) {
        sum < current_block_time
    } else {
        true
    }
}

/// Returns the height from the update data
///
/// `header.signed_header.header.height` is `u64` and it does not contain the
/// revision height. This function is a utility to generate a `Height` type out
/// of the update data.
pub fn height_from_header(header: &Header) -> Height {
    Height::new_with_revision(
        header.trusted_height.revision(),
        // SAFETY: height's bounds are [0..i64::MAX]
        header.signed_header.header.height.inner() as u64,
    )
}

pub fn check_trusted_header(
    header: &Header,
    next_validators_hash: &H256,
) -> Result<(), TrustedValidatorsMismatch> {
    let val_hash = tendermint_verifier::utils::validators_hash(&header.trusted_validators);

    if &val_hash != next_validators_hash {
        Err(TrustedValidatorsMismatch(val_hash, *next_validators_hash))
    } else {
        Ok(())
    }
}

pub fn parse_revision_number(chain_id: &str) -> Option<u64> {
    chain_id
        .rsplit('-')
        .next()
        .map(|height_str| height_str.parse().ok())?
}

pub fn verify_membership(
    contract_address: &H256,
    root: &MerkleRoot,
    key: Vec<u8>,
    storage_proof: MerkleProof,
    value: Vec<u8>,
) -> Result<(), Error> {
    ics23::ibc_api::verify_membership(
        &storage_proof,
        &SDK_SPECS,
        root,
        &[
            b"wasm".to_vec(),
            [0x03]
                .into_iter()
                .chain(*contract_address)
                .chain(IBC_UNION_COSMWASM_COMMITMENT_PREFIX)
                .chain(key)
                .collect::<Vec<_>>(),
        ],
        value,
    )
    .map_err(Error::VerifyMembership)
}

pub fn verify_non_membership(
    contract_address: &H256,
    root: &MerkleRoot,
    key: Vec<u8>,
    storage_proof: MerkleProof,
) -> Result<(), Error> {
    ics23::ibc_api::verify_non_membership(
        &storage_proof,
        &SDK_SPECS,
        root,
        &[
            b"wasm".to_vec(),
            [0x03]
                .into_iter()
                .chain(*contract_address)
                .chain(IBC_UNION_COSMWASM_COMMITMENT_PREFIX)
                .chain(key)
                .collect::<Vec<_>>(),
        ],
    )
    .map_err(Error::VerifyMembership)
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{Timestamp, testing::mock_dependencies};

    use super::*;

    #[test]
    fn update_works() {
        let mut client_state: ClientState = serde_json::from_str(r#"{"chain_id":"bbn-1","contract_address":"0xbcf923a74d8b8914e0235d28c6b59e62b547af5ce366c6aafcb006bce7bb3ba4","frozen_height":null,"latest_height":"1-1633807","max_clock_drift":"600s","proof_specs":[{"inner_spec":{"child_order":[0,1],"child_size":33,"empty_child":"0x","hash":"sha256","max_prefix_length":12,"min_prefix_length":4},"leaf_spec":{"hash":"sha256","length":"var_proto","prefix":"0x00","prehash_key":"no_hash","prehash_value":"sha256"},"max_depth":null,"min_depth":null,"prehash_key_before_comparison":false},{"inner_spec":{"child_order":[0,1],"child_size":32,"empty_child":"0x","hash":"sha256","max_prefix_length":1,"min_prefix_length":1},"leaf_spec":{"hash":"sha256","length":"var_proto","prefix":"0x00","prehash_key":"no_hash","prehash_value":"sha256"},"max_depth":null,"min_depth":null,"prehash_key_before_comparison":false}],"trust_level":{"denominator":3,"numerator":1},"trusting_period":"153000s","unbonding_period":"180000s","upgrade_path":["upgrade","upgradedIBCState"]}"#).unwrap();
        let consensus_state: ConsensusState = serde_json::from_str(
            r#"{"next_validators_hash":"c9b4f641bc23183daae525375c4cafac8ec92cedce0a0ce10db5c17504772548","root":{"hash":"Pp8BdDrdm2ZXjanHtr4Ywfo9eXzcZS5qXLoMyNE3WjA="},"timestamp":"2025-10-14T13:12:47.949379130Z"}"#,
        ).unwrap();
        let header: Header = serde_json::from_str(r#"{"signed_header":{"header":{"version":{"block":"11","app":"0"},"chain_id":"bbn-1","height":"1633942","time":"2025-10-14T13:35:06.754262528Z","last_block_id":{"hash":"b381e6826691148b254d80251b6d213c1733345d956b6361423e0289c8bc84f8","parts":{"total":2,"hash":"5dbd3fccc2b5314f37669e769a62876b095a251e881d04421ba32eb5cd25a178"}},"last_commit_hash":"13adaf0ed14d1b16a9691ae8bee92ce6628a686038eba7fd0523c4653fd8ed03","data_hash":"6f94eea24ff10576c878a1dca9edba1c3a3e7c72ecd3b362eb2bb19ad104bebf","validators_hash":"c9b4f641bc23183daae525375c4cafac8ec92cedce0a0ce10db5c17504772548","next_validators_hash":"c9b4f641bc23183daae525375c4cafac8ec92cedce0a0ce10db5c17504772548","consensus_hash":"22e3fa2d1695ae7db62e55677bf0c914b1ec88d64cd8d280cf2e29b2e06d0965","app_hash":"5df6d08ac2d45a92a47c0f363e0092f9742c1fd76842d663de7d85603bb5db90","last_results_hash":"331814b9a6ebc05b1abb5dd9c36f35e019d3fe17f0656a69105036d479c1ac9b","evidence_hash":"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855","proposer_address":"77be23139b650d69ede7985b31fd7b1cee9ca7d3"},"commit":{"height":"1633942","round":0,"block_id":{"hash":"6f22ca17c5acf577b941ef277127a3404e9aa8892c1443f5509781e38b15201c","parts":{"total":2,"hash":"3772dbc24457a603c8cae7c86670eb3e83d7df02bba68e52db15df39140a615f"}},"signatures":[{"block_id_flag":2,"validator_address":"3c70d8336dee739bdf2d1c5a0fca0a88b87b9a6d","timestamp":"2025-10-14T13:35:16.804163532Z","signature":"GefUXSOxiaCIWrhd66QlF1oSRPxQTGldiOrVvpmKX8UoB0LdxJW8kTf2Uk4DYvVovvZaGMpRrXvgLO9a6dKSDQ=="},{"block_id_flag":2,"validator_address":"ec033669fe506006f870bd8b12cb524a3abc3693","timestamp":"2025-10-14T13:35:16.867200809Z","signature":"TaUHYm6l/zbKQJ6RdyAbsiGR1eFAV986zxjy9Xi7zf8+SOTl2xFUsW/y/3ipNRshYOSlS/fTn2cFOqmeCY7qAw=="},{"block_id_flag":2,"validator_address":"33ecd5899a91105bb4e51066eb138189bc3825ad","timestamp":"2025-10-14T13:35:16.833609848Z","signature":"mVc6yLH6eLhy2BjJpM66fj+MI/6Jgvbw2Atowqc6AkZJ/XJIOBA7LlXB8Iv9v4SCoiHyXl/JfIu26yglak1zDQ=="},{"block_id_flag":2,"validator_address":"44c395a4a96c6d1a450ed33b5a8ddb359cefed36","timestamp":"2025-10-14T13:35:16.798332784Z","signature":"b904ZAs5cnG1owtZcrW9e/3/c1AqdMLYoXzhmnkTdg4Lm4hlYttSuq/5mWpK3sK+JwWiYEh807so5jWe6Ga5Dg=="},{"block_id_flag":2,"validator_address":"c27b627c2fd11ee1e3e850738720692be2e3f4c6","timestamp":"2025-10-14T13:35:16.816085472Z","signature":"yCiZsCIhIG5gxtBSbBZm0H2be849ViO5T+yG7fO22/PwbvJ1uISpnSYtH8Ep2Za7+oAflB4dUFzQrc9f0oHyAw=="},{"block_id_flag":2,"validator_address":"64cdacb04fb9c3b290579644f9edbd05cb00bb7d","timestamp":"2025-10-14T13:35:16.813152025Z","signature":"u7dDFYj1LZ5AqzYswGfngeCIFGZdFRsHxi7gam3k1D7oEqxRCcrjO/GCNptWwfi1arKxJKFmg13mG7y87J1zBg=="},{"block_id_flag":2,"validator_address":"5295664886565f43a3074140551a755872bf6ffd","timestamp":"2025-10-14T13:35:16.886834062Z","signature":"32CIGKELVhqrukC0vA/s8UsO4yBBlfkxTwaxVAdlMzTW5A6tMUB8AaQjz8zO15+tCANVS9VWrGAOL2TCsRllCg=="},{"block_id_flag":2,"validator_address":"2a3d40c3e7ce8d6b55077335cab141c2e11ac254","timestamp":"2025-10-14T13:35:16.828611352Z","signature":"gUFAPtiz9nDvjNqfepdyCA1oRhr39J9vDn1WMKc5v+6oEWfg1w0g8kHbVfNMn9uzRLCSs433rdPmFTLA2J8QCQ=="},{"block_id_flag":2,"validator_address":"f2e6bb25d8d269c9737bd3789b11dc76c4fc56e7","timestamp":"2025-10-14T13:35:16.842935918Z","signature":"HZkPxUmcYBmwE5ec1sXS0TSk8T7tFI2L1asHAS8E8FXv856bXGvtsMOpwLndlYfWQiQhOFQJKa852mm6P99XDA=="},{"block_id_flag":2,"validator_address":"bb386aa0e7b37eb33516aebb58cae3574af85d1f","timestamp":"2025-10-14T13:35:16.822050330Z","signature":"kSzcwHsEAXsEIV6Vo/c9rKXnrDiREg2uwQy0m6tleCaaKusOR/Y5llkw3qaioPKOiO4QCfm0g4Kp6goJOqaiAw=="},{"block_id_flag":2,"validator_address":"43804074334ac6111233037efe20105c516b0e9f","timestamp":"2025-10-14T13:35:16.836502518Z","signature":"RslNh01lORp7eAOqjpm/xCTGVZjwGC8bmv42i3sj1F5mvfFkMwnybKn8q2bHFsLXK1k8kTW/1sUuX08aKndsAw=="},{"block_id_flag":2,"validator_address":"da37d2935eae0a92de55e86db2e887b4f18575f0","timestamp":"2025-10-14T13:35:16.822154654Z","signature":"odqMZ2cFJEOk67m7nR1+r+Muzgfj32iXcvrEx2jWLxyFuKLfg5DDhhe2nvlqKIftIqjGVZ8MtoYtMHqjiQMEDQ=="},{"block_id_flag":2,"validator_address":"9c4cc8543dc684948490681118957942b29a9200","timestamp":"2025-10-14T13:35:16.808194913Z","signature":"+49qNcxkAhPwQ2aojC/Ev4u0VsNu59IsgcEqgFj03Ettctf/MGlFHETjYvinhMX9IdAigadiwTVaaYQ7T+2mBw=="},{"block_id_flag":2,"validator_address":"01b2c0f03ba9b2f352a39bc45ce37aa2710b5af4","timestamp":"2025-10-14T13:35:16.945498341Z","signature":"z1ZjlgSx3RW61AzbDczjmDmYF+VZ1coT9Btz8xS7LVwThyHuHnHNipsEH2L27O7/9ULsc9ltII/P/9Ry4VyFBA=="},{"block_id_flag":2,"validator_address":"410c1ebaf54c090f0a9132dfa9f79e0f480b2c06","timestamp":"2025-10-14T13:35:16.860619759Z","signature":"xvEW4m6sfChUnG42RtDMqyaGC4FjjshgtwHNYH4NwbM7hZY5p/cpeERSGvsKHbS/Bq27Lm7mDa8bLsFZnYyIAA=="},{"block_id_flag":2,"validator_address":"c2989ed51e73b742e4991cf915d6339419657ad5","timestamp":"2025-10-14T13:35:16.882512004Z","signature":"kqQcxLoBxidthsnizB/j+iVE9ys65ESaSTRBqh6BGp/DxSKAs8XoNfs23dSYJGEvO4Q6dUB/qx3Gdl3WSbj4Cg=="},{"block_id_flag":2,"validator_address":"f6866f1f291d5a6740480c6f0774aba45177b68d","timestamp":"2025-10-14T13:35:16.893733348Z","signature":"5h2czTahrsK5i2C1XBRrCcdqCPDsUi097NOPLHETQBLiighK76n2/nMBY+kmqLbZMhO9lf4nH8z3WJ+aGUnBAg=="},{"block_id_flag":2,"validator_address":"37b7e284915654acc06e394fb423dcae6a4fd94f","timestamp":"2025-10-14T13:35:16.939955970Z","signature":"MbAXe4/bIj4MmgCjEXtE84U/aqSWuKTdjIYiKwxbaoiJjhUl8wpl23Ae4HkRibeyU/gh8b4mPhn74hGdBGgOAA=="},{"block_id_flag":2,"validator_address":"1a4b644c16a4a1904618f8d3c0f6a67223038390","timestamp":"2025-10-14T13:35:16.922435962Z","signature":"MOKNp3LrKgypKQqFiVUJyQey9zDMg6CgBcX10NSw4fvJLQU+f2tylF4pfBI7XIamX+HbtZmWOC+K776TAeS5BA=="},{"block_id_flag":2,"validator_address":"53406de83381dd804632447f1baed00cf28048e0","timestamp":"2025-10-14T13:35:16.848275858Z","signature":"aH2UO6RisI1g8cs91p/AttFNcdHvSmgWIqk1ORdErPbGX5VVyiAbKPSvJsqfiW+3i7pd3R368GEqF0pG5e7uAQ=="},{"block_id_flag":2,"validator_address":"b3f0e466cf435fc603117e6cd516ab7e6337e212","timestamp":"2025-10-14T13:35:16.931329173Z","signature":"5HBkPoF3kj3HGOlRTFyC7hZQVdp7hb9KX57ba4jAO8NCvOkXEnf/AQSfiLjnS7ia7guCV4k1/ZHhZ7X0VUDpCQ=="},{"block_id_flag":2,"validator_address":"300636fc6b27a7016f3aef03c231aa63163eeb5d","timestamp":"2025-10-14T13:35:16.912739619Z","signature":"Ua2EMxsdAYmO8Rb/Cq0/Kt0/uflLS0WwJko2dPYtXq4+XOEQXXtynQBBd+2S9MZu+5fYDTi9HTCDWzAcZgSNBQ=="},{"block_id_flag":2,"validator_address":"b032b8177b81d9ed1bce8bf0ff14c45d17e821f1","timestamp":"2025-10-14T13:35:16.801900250Z","signature":"TW3bfhs0v8agHG/vzGVPZPbJrlUNMaRJkAw3NAb8r+8FJwjDNg9vHwq9YJqXJjnJqE4KP6zyU9SOyHoVVuMHAw=="},{"block_id_flag":2,"validator_address":"6cc0a1531ff58d61dc9061c5b89bc0efc07911fc","timestamp":"2025-10-14T13:35:16.817201350Z","signature":"JstuikU4GYb00TDnEK7bwKEug69PyyHLJFYCaoa/nm9A/09dU6R6stjF1JBJkYrsiEPFGG/OtX5sT20oowaHBQ=="},{"block_id_flag":2,"validator_address":"3965c223e13ac66f8175c86e657d396c711964b7","timestamp":"2025-10-14T13:35:16.859458177Z","signature":"XgPgJuww7X9iWtyQ9XTjFxbfKbi5GXDDMBRX0vgktmimTGE/HpTA4CSfkXh601NXt5b9sRyWk/V3klppMPKEBA=="},{"block_id_flag":2,"validator_address":"76f6dd3f07bf2e9ff5b0a188bc08d5a4efcf40d9","timestamp":"2025-10-14T13:35:16.812586719Z","signature":"JUOjNCya0zVwSvE9HiQwQwvSRY5HtUxURvS82lz2F9GiJT2xhwUSKgnxANnryUShy7f3zLZhjb4+iJyzDMW5Dw=="},{"block_id_flag":2,"validator_address":"1b21f9bf3b9a650607cced8c0c2fbeca3211791b","timestamp":"2025-10-14T13:35:16.822749327Z","signature":"ofOe2EX2VvISkZdKaUeS5VAoTdNFcSs3RuR3cAqaZGfl6aoCsVzKb2koLHO6ntVe+TLtyWS6W2S1N45E//2cAw=="},{"block_id_flag":2,"validator_address":"1b03ab3c744545b281d224127708a9ec7487e58c","timestamp":"2025-10-14T13:35:16.864646325Z","signature":"E/THg7ezDHFAtTJ/jZIUjW1AS54i6IAeVEzSdALv1icjBdxnZmDnwCRBsSSNWMxeTFFx8/IQJUGI03rW/Gk6Dw=="},{"block_id_flag":2,"validator_address":"e2fc4df9e7870fc8238136faca07a120cc078a0c","timestamp":"2025-10-14T13:35:16.876367328Z","signature":"D5J15oOCLe2/uLPqZ67L8OdzR2n/EuiIcuJdzJ7KBiPuYDuxZf/Pvxqe98CAN788Mnh6PH503UmkLGcwDP26Aw=="},{"block_id_flag":2,"validator_address":"671d816776e8c7fae18dc122fb6dc231956c6d48","timestamp":"2025-10-14T13:35:16.810209699Z","signature":"JtmLv/ErUTI3hh+NrA2NckTb9LzR7ZgOLepvOk9JBqYbkI6LJl7hYdd7wcvpir68V4BQKkDiyExkro/vcx11BA=="},{"block_id_flag":2,"validator_address":"6bdc45074c28d7467d37f875091fa5036ac62eed","timestamp":"2025-10-14T13:35:16.819497787Z","signature":"QtBKSvbBypI97JfjMmmbQoOvncAiVs+VqqqurOGam+ftHrORa4u3wz6AokDLylkSVEbAKDKNUg2psb2uEw4xBA=="},{"block_id_flag":2,"validator_address":"718160e3538e0b0ed83d3b3dc4ece83809fcd107","timestamp":"2025-10-14T13:35:16.885512141Z","signature":"94TX2nwyVpzIa4IOMtDHkoFlMTQbj3HzJ1WOO4A3dm0vTH6RagyJNogTUlXI99Nfh9OpiytDEzzqF3X/BLRYDg=="},{"block_id_flag":2,"validator_address":"b03f0cb997f3428ef3968833bca2f2a4f613a8eb","timestamp":"2025-10-14T13:35:16.816376437Z","signature":"Qja5jT/alwISK+QIuGaYhRIWwj9VVejtJQe/ylewe954fRoYRCNwkP1x5CbuT28+Qa4vwKPAnAAO3tZOQoKDCg=="},{"block_id_flag":2,"validator_address":"77be23139b650d69ede7985b31fd7b1cee9ca7d3","timestamp":"2025-10-14T13:35:16.873506591Z","signature":"ek+fWLJ/zL/m06hxE+UDIbxAZHCIIQwUG6aqRV1qEmN6dNK1FNB5FuKkD82rFh8lojtDYmwqzPVmHjpFGM0aDg=="},{"block_id_flag":2,"validator_address":"5c78a0762f13b0b2a793e82d1a178eb45642234d","timestamp":"2025-10-14T13:35:16.862321548Z","signature":"KVx/kYEM3k6B0+3upINPVMiyH1JpUYBFAut1Eh2B6sbKy8GiOUufcTV+AyUrw+3Ul05n42PcHEkOM7v4aZMvBA=="},{"block_id_flag":2,"validator_address":"70dac78498afc6d766d860c433265a274d48b7d8","timestamp":"2025-10-14T13:35:16.879429442Z","signature":"Hi/bLFJawoh5rIBYECo3jLKQXrgq2vRcNLzUiyhvx/Tz5eAoCvZzwUzaOYgPoXWnusNF96nbjOgb9s3sWEhjAg=="},{"block_id_flag":2,"validator_address":"b41ef50272a7a2398ba617d3c963986ebebc5b2d","timestamp":"2025-10-14T13:35:17.037470044Z","signature":"oBMw6nnl8cS5H8J20635pdQUXfRugeF0YKrvX5eJiZX2mWOOTym+oSJd/mm9sur88gEqsALV6iMpI1ndVNwPDw=="},{"block_id_flag":2,"validator_address":"521d2a344e322b8ba76401c833a1ec74ea2bd075","timestamp":"2025-10-14T13:35:16.781733245Z","signature":"/2PYlDb4ooJ6CHN2CqXr0vqf1J5M2//Z3kqDd+8F2UlXqf1r83P7Wblm5e+9ZmZAyYiw+KOW27gAZcXt4BxAAw=="},{"block_id_flag":2,"validator_address":"35f6c0359084c37ef6eb4061b694386910093b99","timestamp":"2025-10-14T13:35:16.931324525Z","signature":"mshV/HWNWb8fof/2YxQ3oiJMriRoDRkkuzEiltegv/5WuhNwYF6NTMHbQh6wFImYqWt3N7Cj+64BTtL0EmJ3Dw=="},{"block_id_flag":2,"validator_address":"2a1b3fee6b29ea1828d427f0e5c3887cb7e64487","timestamp":"2025-10-14T13:35:16.805681903Z","signature":"YqhsB8zMjecLYFa61LHh4rIxRlwA853KJqcBhoU3cLWUN7vuvizyfUyc+fgOAQLkHWjoEVhhf0Zxu4H4d6YEAg=="},{"block_id_flag":2,"validator_address":"ccb2909dcd316fc5e738dcdb1f14360f04b44320","timestamp":"2025-10-14T13:35:16.826109205Z","signature":"oiymf8t9KD7540tNEaCtMpPx1lvOwr5/aBtcqr62rGCTUCJ9bGp2UsmwY63VU20j2xxSkV/KyOH3zR2MMKPdDg=="},{"block_id_flag":2,"validator_address":"616a45478336b92583cfe29e8ee942802de69eae","timestamp":"2025-10-14T13:35:16.861447244Z","signature":"UE0wuH8KctrAIfv2N6kZaKhTRMapirWEyTFpFksc8SMRQitSlusUrVwoeDrxR3n79JAHdKWeJnsu5hJmCj3LAA=="},{"block_id_flag":2,"validator_address":"fbc2a3f682b507f1a3a5d4c44c8cbb6af0860296","timestamp":"2025-10-14T13:35:16.871591263Z","signature":"IkX1JqACkSALcP9cbUqxJ7CThcWsou5uTu2yF3sl3afKQn76OIFfVxoQgsWLad54AxpfITA4uQxEV6OWto7wDw=="},{"block_id_flag":2,"validator_address":"2c4e108c3f09cf9b8b1515d3b26d50e332d86e3b","timestamp":"2025-10-14T13:35:16.807003387Z","signature":"QWwMVQ/WNk7FdZJGBDgiU6ZkWY3nAzEw7IKEw1tkVar0tmGIaqxfewAktacLghL+aLizDWlVBXvlJGjFaNABAg=="},{"block_id_flag":2,"validator_address":"f42c7e49df4f4d0993237959518415e473792392","timestamp":"2025-10-14T13:35:16.800848813Z","signature":"aNSxYejnEG8YM0B5QFaBWAb9YChLCf7Fyn0fUmZdadrzEoawpO7YMCVxCKrwKms8/QhKJs0IKyFXluMeQUr9DA=="},{"block_id_flag":2,"validator_address":"c268fe001a533b6f0860167381384363a790fd0d","timestamp":"2025-10-14T13:35:16.801419520Z","signature":"DxyslXbnXYcXtAQ9G0dygo3KhQCjEMsZRpOVLLEc+PipzzrKSrb5o6mDN4tbGUCmmNgUJfIU7MHUE4FAosWsAw=="},{"block_id_flag":2,"validator_address":"9a323ff30c24f59e160aaf02a57c63e11c41495e","timestamp":"2025-10-14T13:35:16.830117467Z","signature":"oW6zcz+H5hgDPmSdNXPzfhmk86K/LmdbvtIanpan7P400xMjxqpB07Q/3twWx1fjgI0X0b7sP6Z6LHzLm+GJDQ=="},{"block_id_flag":2,"validator_address":"a8f38fcb39b580c4b856dcaed882d504a216ee00","timestamp":"2025-10-14T13:35:16.855479281Z","signature":"hJszBNAr08+oWPmFccxRE0owaQC6xSfBUZJ5XNCfD7dsD6OCe/vAsajNgu7UBXqEhE5FUdWd5fssphmn1pjnCg=="},{"block_id_flag":2,"validator_address":"7ea986d8723587f5e653c0df785fcfce39238785","timestamp":"2025-10-14T13:35:16.802610216Z","signature":"WT8qEZ2iSrUZJUWtrMLhK3pw0OPj65LNmrEgI9WtHawVXF+C3dIVRmtZCjf6OQgL8kkrw+sWe5WCR6T+oz51Dw=="},{"block_id_flag":2,"validator_address":"aea26001de9b15d4dad32808712864e61ae52a17","timestamp":"2025-10-14T13:35:16.949727080Z","signature":"f3DpJnBWNF5Oe5MF+35GiUnMYdQVBoLquamFSKqlKuZj+IpgxO+ASnjEz4GaMXx/65i63TV1/fbv/9gQRXF+AA=="},{"block_id_flag":2,"validator_address":"2972b0df619f73d1a5c8933aff22ad8c9258f370","timestamp":"2025-10-14T13:35:16.838260718Z","signature":"++et0qiMVBU9QhOYyEC/U1F+aNldDQkwkNGkZDNOpk2uD0ZyVy/kHDQVSkS4jlG2GszkfjKcA5FJy7kwRAz0Bw=="},{"block_id_flag":2,"validator_address":"a8a2bf2709ca05755f881ebd939991aea9f88cf1","timestamp":"2025-10-14T13:35:16.866347399Z","signature":"pO/cYRiM/O+VZlP1ToOx3nTA35P2qr0y4j638buvxQLk8QR+XqROSjICfSvGPP6rloykVeowbe6DyNsdysQdAA=="},{"block_id_flag":2,"validator_address":"23df4a8418762cea4d24b7227ad125df76d2baf6","timestamp":"2025-10-14T13:35:16.871447600Z","signature":"wjvC06pGCaBA1Q3LBCckDoMLkJeQf4a/maGBmxaTbCgkWpmD/UQg2cSSPtQFs0uOWa3g0Bxvts1fVgRjANLiAA=="},{"block_id_flag":2,"validator_address":"9407fa25941c932473f8e2b1ecc5481e3ae15a71","timestamp":"2025-10-14T13:35:16.871585034Z","signature":"d5fN0/UnE1dcBq6Cr/N1Ad6uDrlMe+1BX3nf6gSQPp9DkJ8czpJs/QblBz1ez2qetcWlU4+I8Mp+tX+kpffhDA=="},{"block_id_flag":2,"validator_address":"65c584027f5fd8349027fa5bf1d4ed3828c29058","timestamp":"2025-10-14T13:35:16.894428661Z","signature":"jXZYuyuEvzE73joxhBnJabtFVEKMh2VBlIWizsAAVQU6Q2ma3mGjGWdvR1lwyKiSlVkxYo4WecP9KEMs4I1pDQ=="},{"block_id_flag":2,"validator_address":"466d1236b8c77925b25219e4ffe34dd069351847","timestamp":"2025-10-14T13:35:16.887589197Z","signature":"iksPB7Y1D9rkXTsmzLEljYSlX6CDLi15VC6jREl/B8huBntBSehaR+txrmJL0xvysvyUUpLKid0nuL+b8ZcVCQ=="},{"block_id_flag":2,"validator_address":"b8f9204a348e0dbc85abe5bc98936d6b84c01e45","timestamp":"2025-10-14T13:35:16.805082455Z","signature":"V9i6zangjpzXqGmatf3XCFJy9iPhU1K2i9xd4Oo6Sw6Cl24VcLHQ+Brh5F59+M2B0BAWKD5uc3jCBJyybWT4Ag=="},{"block_id_flag":2,"validator_address":"2ab301f354a74f45afb347b2c00cdceb09cc719d","timestamp":"2025-10-14T13:35:16.827079668Z","signature":"DuzyqZoACKrq4ZK17kwuOTReUVVxhH47Y1VOHAH+oKIMmfU1K5ani2ARuPwonPP7APkXRkjyQmAJyvCfAhaPBg=="},{"block_id_flag":2,"validator_address":"9215b6b8ec64cb9e7b461c3945e6ce00da86f8f2","timestamp":"2025-10-14T13:35:16.819978877Z","signature":"Czkz6GbWplQKAS5gbwWOxgr1DExBzwnuyGbSbC9UE7MzjW1XNJBhtR7MaWkNf96sGsHEGKS3ZksNF9Bg/7pJDA=="},{"block_id_flag":2,"validator_address":"a9c7a884bd7d6e799ef5311bf9168e0105bce416","timestamp":"2025-10-14T13:35:16.875444674Z","signature":"nTDhLLq4O5LD/Bz22HbSb6Iwaak1bqEPnbTqL2SUyILu2wP+5NbNCXEoiY9b0l5rSxqH13zIsIDpz5SqjEiNDg=="},{"block_id_flag":2,"validator_address":"f1961eee75e5a1c8a11a5bf72fb368b990bdd289","timestamp":"2025-10-14T13:35:16.982329948Z","signature":"WUkp7IYHb0q0BUDm1qWG+XEvbBkg00QWTjoDw2UUnEnO2myHyx8MZO3jLc3SMC53TA0NjgM387ABT/ECOzBODA=="},{"block_id_flag":2,"validator_address":"9fee83fbf0bd5a4d71519efc16d0d438c203dd5a","timestamp":"2025-10-14T13:35:16.818733534Z","signature":"RftK39YIljeH6XuWpL6SB7uO7/K4ZVB+gr3WQQGILS8aSnH7N8zJZBGo9ap4pgsvACQxqzyoBFHrJ6F/jWRrBg=="},{"block_id_flag":2,"validator_address":"b44fc7d70708095fbc30af164410749483f96439","timestamp":"2025-10-14T13:35:16.854712016Z","signature":"pKpT9fLVqMRbliZIOXh+aWAsEJAZK2MqoiTU9qDo1UHDvhVTcx5dnDsPgOHGqWcvjk9s9cUmU8haC37z29XfCQ=="},{"block_id_flag":2,"validator_address":"c4ef42d72d69f4b58ffe2a58db1335269842ac43","timestamp":"2025-10-14T13:35:16.915874586Z","signature":"CP37xuiRa2GmIesBXVVHKvcTnM8m12pp2GNUm7z0Nq0h+zNfY70dEVBDaBann4WrsxRt7hLeuXDG/qHErSEIAQ=="},{"block_id_flag":2,"validator_address":"60c51d643048d0a426171bd0deabd1689a89690f","timestamp":"2025-10-14T13:35:16.881817719Z","signature":"GfCMFKozcJOEZ1Fwp7mbOfbhTSiJqBINE19ut81dB0CT7PmVo6lMCmUTXWiVcOMwOM/J+2iXwlCA2QTyu8IkBg=="},{"block_id_flag":2,"validator_address":"5d673308ca2b45e5222303836d833e0f70b1f9b0","timestamp":"2025-10-14T13:35:16.829328259Z","signature":"ZwZvymCeKngGiTNQNmdwCqLx8gvJBSZyK1AfgDQzB6fn1dFVY2P0gdMd1QbrXHg6IigtJvkjVg7iltLv/1kyCg=="},{"block_id_flag":2,"validator_address":"06be03f1b334fc3f7947d0d031f7dfa568005bd4","timestamp":"2025-10-14T13:35:16.803792607Z","signature":"kw8fJXu82byqZkKTSnyufnAekuA6ctCssQSulleFadAYQEMdZrcSC9J5JNqKntaquZmZ3ru9zHP+vXnSD9gkCA=="},{"block_id_flag":2,"validator_address":"d7076d16c2e91e0eb16753575ac3cee3057781c0","timestamp":"2025-10-14T13:35:16.804300322Z","signature":"HDPX+tmT+yM2dos0IfOHKjm8UP33xu57eT9FH9fH3Rvk3gwzeEKwf044cU3qcKYUbETU4wCOOFQVLIJvWEjlCA=="},{"block_id_flag":2,"validator_address":"913834357f92e577cdc02b1e682c65e86d016ae5","timestamp":"2025-10-14T13:35:16.884055010Z","signature":"02HCdbgvYP+FUHLJCcI0WQnq5/BCF2NukWqct3mYZVqhLGe0t/4+qMu4TC1Gk2ocTn5917Y3qranWdXzCqGHAg=="},{"block_id_flag":2,"validator_address":"3ce85b466ff74142307b4b03ddacf17a52d46373","timestamp":"2025-10-14T13:35:16.793427896Z","signature":"RT2mfPidMNYnzu9AFLJH+JBdZoWA+Ff1jSj2Udd8mQc/WiZml2eMy5qT75aN5BAV3fak5Mt1OE65DysHGqg9AA=="},{"block_id_flag":2,"validator_address":"dc0802ad3262ecd784ce339d87073c836e31c69e","timestamp":"2025-10-14T13:35:16.816058121Z","signature":"n1z1Q6V273giUJYincMzVsU7aNoHryEKsYcu4nIQSWjy5hhqfPi16LtTQ8SBAwkzB+KnA5wt8gh9ORgyk6K4AA=="},{"block_id_flag":2,"validator_address":"31e623ab0ff5dc408c052072f96df3d4506718ce","timestamp":"2025-10-14T13:35:16.878369234Z","signature":"rWAexlCT52ut5QMs7u7vaPVLSJJz+yUIyZD0acCoobxmNF+4pvxHWAw76HWYmBHje7SHbWt4DPM19+m2N++HBw=="},{"block_id_flag":2,"validator_address":"02d49d26c87ed7dc1e38d7bfe5b7fcdf5a3a3164","timestamp":"2025-10-14T13:35:16.894743814Z","signature":"vN2WnXfVcX7pqtoZO0rpBhc/Xe0UMt0E5+/7cgcyFELdoG5F/dsoCbLo5aHNFNaYH20VVOdn9yiqBt5d9TNuDg=="},{"block_id_flag":2,"validator_address":"646d5c6384c84c2e660bf268d0a4ebc1ffbceea6","timestamp":"2025-10-14T13:35:16.993396133Z","signature":"KxbNgFkLWRghwMD65IR3LsA3yqshu6JcJVw6mtmNC7HW1FM7UXPuzfj4TisGCOqwh+eYWNQtpLZBvNt0HxCcCQ=="},{"block_id_flag":2,"validator_address":"e6f8df77989972328c927e5493db6908936a4ea0","timestamp":"2025-10-14T13:35:16.792102156Z","signature":"XKBBC2daX+NjMW3gwUr2UE8SZGxYLAE3krH2aWzDsdr6QqhwdgtiI5FG5RYOGIkiZxcEUYHE8yoHd2EEmYueCg=="},{"block_id_flag":2,"validator_address":"0aabc33bff6e288ea151463ca133e38051bada85","timestamp":"2025-10-14T13:35:17.051971620Z","signature":"voP6fAKJvxU30T8OCyydIIbwNQrz4wY1oZW7hnHryHbxllyAiFjjRJAYpnD8Ys2qzn8eGUBH5CkRxXvhU/moAQ=="},{"block_id_flag":2,"validator_address":"e71b23c378e4c465bfe96ad48e7f93f237e939dc","timestamp":"2025-10-14T13:35:16.925969654Z","signature":"swpI/CiccP3BZZIxqZE3YAf7gfXLlCN4fquLMicYk/XM7l/iMJpoPVsN22+R8VahRFSFoEE51FfQdG2eWRmKDA=="},{"block_id_flag":2,"validator_address":"bfee4eba8e3307bcb074ad575ab9fa4524af2a25","timestamp":"2025-10-14T13:35:16.876593376Z","signature":"gyagLS63qXLuUC0XSvhaw+tp4dS1v/Qd8W97s1MwPR3/wSH5G3+O4Go2oxLHWAQBiWT+WNaWNKMu0P7rFYSSCg=="},{"block_id_flag":2,"validator_address":"fbe1e3ed324fa17379403acfc3dd28fa882c6e65","timestamp":"2025-10-14T13:35:16.856966914Z","signature":"FbvGqSofqDOKV2nqvki9GE6lt3xqRtPVLJUKP9yG5V+qa+zBqMg1Dsjr+ZxYx9cKZ7opTtN1lJwiUQsSRpLXBw=="},{"block_id_flag":2,"validator_address":"f8260be47a1c7d42bbd9a656bdf8e3d3f28cbab2","timestamp":"2025-10-14T13:35:16.793990564Z","signature":"yx7+pZAXJjTBS2gvjqtFtKhi6Un5QrA1HpWnNjmcNcE6IhMpYwsQdfc+WiOFHkX8NkDxVvhGlMHb7Z9m62b2Dw=="},{"block_id_flag":2,"validator_address":"7c4b90fe7584bad71c5c6ab0877b8c39cd289187","timestamp":"2025-10-14T13:35:16.957532122Z","signature":"QYL1TweLVRKQuhp3eD2VHtJPqu8o53T+L/r6p9dO6W+SwUPBRLkl1xHFtu9JX0U7DJUMQFjw5XHEKZ4NZjDbAg=="},{"block_id_flag":2,"validator_address":"3b2a7b69569e7835324f35f7f2264d008e850c12","timestamp":"2025-10-14T13:35:16.886441939Z","signature":"/E1cuu7SRPBji/gnMk6xbBtJEHuJEfVhbJ1pyuj+5Bkcqi8jrAUKO+9vXyejTyKgjKGb0WJXJvjABofu1zxRCw=="},{"block_id_flag":2,"validator_address":"edd5716db0738cb0eecd0a04aeb186c4a46720e9","timestamp":"2025-10-14T13:35:16.891830587Z","signature":"HGm8jj7bZVlSKTFLcDZxjPk9ahprRD1v0KqAYEgOV/mM6f1LBcZd61HXw2s7Kkmr8vkX33rBqfJ3MFuJWlT6Bg=="},{"block_id_flag":2,"validator_address":"f868d1eb91e32d7f21ef78462feac2892d09eb56","timestamp":"2025-10-14T13:35:16.869372664Z","signature":"XNEVLGt4I9msTE3mT4Mqd1bZFEqMWxTmyqsjoA8HTdvll8l5gRhzRbXzxMWNhe7VrRfK/jOyQpmUe/RwLsCRDg=="},{"block_id_flag":2,"validator_address":"59c09cd227541c5ee7d911084ada87352c899851","timestamp":"2025-10-14T13:35:16.823000638Z","signature":"ZBPYDKGsd6kiygujQTC0iHHuyelBoHHMNUk4XROzWxP9GtT9TPbaPqCLXSe1xgfv4ymZ6VBmsww+pIpZFivQBQ=="},{"block_id_flag":2,"validator_address":"478a66d6d2fcb936ba235daeb5a35ba454c048a7","timestamp":"2025-10-14T13:35:16.966472299Z","signature":"+BvY1KMOT2A8Sme5ExzmW8inBaUfCXKZ2sSnfc6rpa8CiWYO/Tu17AixQucuVI7DBqHpLn3PDON4QOWFFJ/fDA=="},{"block_id_flag":2,"validator_address":"15839d4135d8c8fa603f533ea3ba80a687fe527a","timestamp":"2025-10-14T13:35:16.880533325Z","signature":"x1LD55RFOW0mqq6YpluqVN/S7EmjplFCj1aYQSxNNlqXKnCNw4vepF3zCOQb+hW91N6zNtmCfMD6/12L9zAWBQ=="},{"block_id_flag":2,"validator_address":"13247270303a9d66ea8ffa02614a8b5369adda53","timestamp":"2025-10-14T13:35:16.834612945Z","signature":"2I+mPzUwhqI5hJnFZiTcy/S7J8l7WRDHyJifCwK/DZiZt6O5mHftHFzwtfdmYbEX9g+Yw+LZGuNZ6JLgPhPwDA=="},{"block_id_flag":2,"validator_address":"409d07de01ea2a63c779c0246562aa278a95611e","timestamp":"2025-10-14T13:35:16.900042462Z","signature":"KZoh5KTnomROCVotffDEUS2DRIZBLoyT6RyK9+XRyjXRHLqH4vihJzxij6z9bjiPanCOFhplD7HlHDbvoKE1Ag=="},{"block_id_flag":2,"validator_address":"4e30f6a73766baa5fe6db7181b7fd08329c103e0","timestamp":"2025-10-14T13:35:16.828609004Z","signature":"KIk7Mn4P27CS1QHRxAkHejg+tIz25BKLSiK2IwyWyFBL10DZduUHRrNVqLFdReAM3t3WUxcQywnQfVCd1/c2AQ=="},{"block_id_flag":2,"validator_address":"f70543a091a45a8c0de04e5ce9e40d9c16d980e1","timestamp":"2025-10-14T13:35:16.867745369Z","signature":"BLMQedIW/GtFJY7aityVzgZ8DJO1TDIoKYJta4EaFWXr/qzE8AXKamks6l5pXtFFhhl95JiXAPygNMMEnvk1AQ=="},{"block_id_flag":2,"validator_address":"e9b8e658e2f1aea81d57b78c8bdd2aa93a1bfc01","timestamp":"2025-10-14T13:35:16.819072960Z","signature":"hc83gJLpN9A2I812OdemzeU9+1AhY97csIRSE/nIL0CwPhb4NexvPs1qd0bVXlcRqeEF2GHxhKvaE5/RCb5CBw=="},{"block_id_flag":2,"validator_address":"3daf658a7b8dc1a5186f7c1547ab96ca35686575","timestamp":"2025-10-14T13:35:16.810616889Z","signature":"XlIWDkSPvUdKuL78c3n09xcfgsSOKSjIjxStqljGti02q09BzMcgpKUs00+bHemD/BgxjuV8IioDSl+x3x5WDg=="},{"block_id_flag":2,"validator_address":"02c375d9b259cdbb7b8a97d71b2a0a0dbed890e0","timestamp":"2025-10-14T13:35:16.804405242Z","signature":"tbDyx8A7IJ57TBh+mGqZffrub2Zjt6HZTOWGgTmMJbDyjxKqDWZktIM8GcI2gWW+lMAQ3wEMiKACYszsbiIGAQ=="},{"block_id_flag":2,"validator_address":"556f693ffba4b4c6de7af822852bd6e8a1a1d410","timestamp":"2025-10-14T13:35:16.885491263Z","signature":"hblJZBfZhNQdIFjLwgv4kWqqjMYl+PQCHs8QSFTLEgHVtame1B8oQg5wHYnn85Hj6++fwXstduRBtjPQEa8oAw=="},{"block_id_flag":2,"validator_address":"b31b8ebf6857480b1281e756f0fd67243ac7a202","timestamp":"2025-10-14T13:35:16.978626705Z","signature":"paNvmRWEqShvucbXgtEiaS96XFMstiArhv7iHY3DMfGoSBUZisCBWMa4E9mEPpgXytq3bUGpbXC4BZd7DosLAQ=="},{"block_id_flag":2,"validator_address":"8dabad1a71629e4938de8c44d0bd2f2ce1eabd08","timestamp":"2025-10-14T13:35:16.885644059Z","signature":"QPq+wwjw/nWaunFRXlG9CNj2I6mhBLD4fZkHyHT7E2QY4ZS6TYNM4XpP/Tl5Dkif+NT7ZJxqofRDrUCMHfwXBQ=="},{"block_id_flag":2,"validator_address":"615e9b48cd48fda3a754f28e3f42f77c2e49b0d7","timestamp":"2025-10-14T13:35:16.864513724Z","signature":"YczJKLkD9fwxa2oXMG6Y7pmkQ3K+S/+KBlgNTryxYaHYOlJpCocwsUwgFI0mEnmCFI2qgu/IL0MiKGe5TBzCBg=="},{"block_id_flag":2,"validator_address":"43965a09c5be34d027a599bd6f450f5e6a13ddf7","timestamp":"2025-10-14T13:35:16.805451178Z","signature":"kEcNJKkU4ZotGmmRXOomM9jP2Y1LDIxLlJG5daZ+QJ4D3zq4r2NrMbrujXtI0Uv4qOLLFiyfDFlKTOvKifuIAQ=="},{"block_id_flag":2,"validator_address":"25764380a6df1c9cad29a842a049f690ab0f08bc","timestamp":"2025-10-14T13:35:16.849808976Z","signature":"ksFgcObgQ5GLox5YBQJrDlRB37THtWA0Vjv7nMrUW51G9s2sAN8eZ8Psz6ytpe+3cd/ip3LNKButMhl6UWJxDg=="}]}},"validator_set":{"validators":[{"address":"3c70d8336dee739bdf2d1c5a0fca0a88b87b9a6d","pub_key":{"type":"tendermint/PubKeyEd25519","value":"hg4sm5gNYb9T4dWC94nDW+30olLECT6Tkrp1FtnVBRE="},"voting_power":"110885211","proposer_priority":"826892073"},{"address":"ec033669fe506006f870bd8b12cb524a3abc3693","pub_key":{"type":"tendermint/PubKeyEd25519","value":"ADyMDDsnw93NdBSqXRlRdeWMNnicitm1ikJ14J240Jk="},"voting_power":"70917400","proposer_priority":"-343660815"},{"address":"33ecd5899a91105bb4e51066eb138189bc3825ad","pub_key":{"type":"tendermint/PubKeyEd25519","value":"dWFxEYRPzIyQpPwSHyw30ZZDZn6GgZJpDyMUvxRSbtQ="},"voting_power":"66631704","proposer_priority":"460858499"},{"address":"44c395a4a96c6d1a450ed33b5a8ddb359cefed36","pub_key":{"type":"tendermint/PubKeyEd25519","value":"q6lQj+wYNZ+QQu9zgWoWfSXcwnQNEzb2a7HnyWyBQf0="},"voting_power":"66396058","proposer_priority":"-502578383"},{"address":"c27b627c2fd11ee1e3e850738720692be2e3f4c6","pub_key":{"type":"tendermint/PubKeyEd25519","value":"vuHjnsOVyXFYot1RaV+Wx+ck2hOOvb9/QXwXuzeWcm4="},"voting_power":"60834013","proposer_priority":"178486029"},{"address":"64cdacb04fb9c3b290579644f9edbd05cb00bb7d","pub_key":{"type":"tendermint/PubKeyEd25519","value":"ydZBYn1uztV039ywjiQ8rRycnXHc62hK10+4ZalaQVY="},"voting_power":"60576465","proposer_priority":"54070763"},{"address":"5295664886565f43a3074140551a755872bf6ffd","pub_key":{"type":"tendermint/PubKeyEd25519","value":"b3y28iu4R+IBmSIPwJA90k6yqb5kJPgvDVCmcnpSJck="},"voting_power":"60019185","proposer_priority":"-490243251"},{"address":"2a3d40c3e7ce8d6b55077335cab141c2e11ac254","pub_key":{"type":"tendermint/PubKeyEd25519","value":"kUQmCZtP4I3q5FkKaPVTvXnr3UMD7GwlhnGdkXqsbbg="},"voting_power":"56084140","proposer_priority":"-780542481"},{"address":"f2e6bb25d8d269c9737bd3789b11dc76c4fc56e7","pub_key":{"type":"tendermint/PubKeyEd25519","value":"AQbty1lMHkV1ZHEcW8C6AYK8QhMfxaAhBdp2w+Ykp28="},"voting_power":"56060558","proposer_priority":"316571682"},{"address":"bb386aa0e7b37eb33516aebb58cae3574af85d1f","pub_key":{"type":"tendermint/PubKeyEd25519","value":"cDPnCL7aSojDMLgu23KG3wIVb6cfpTFS3DBimDhtQro="},"voting_power":"50167947","proposer_priority":"-851639503"},{"address":"43804074334ac6111233037efe20105c516b0e9f","pub_key":{"type":"tendermint/PubKeyEd25519","value":"4HpLoBBK/GzdceBjoD4ZXBCeMyv14WPDeoCtZ1BqYv0="},"voting_power":"50022601","proposer_priority":"49527168"},{"address":"da37d2935eae0a92de55e86db2e887b4f18575f0","pub_key":{"type":"tendermint/PubKeyEd25519","value":"x7i3trbMJFfCcb9SAfZbL8s2M1D0OHQ4piGP+DmkJ0k="},"voting_power":"50004013","proposer_priority":"672238745"},{"address":"9c4cc8543dc684948490681118957942b29a9200","pub_key":{"type":"tendermint/PubKeyEd25519","value":"mQhc5iQg6aagqCU3i8bgNFr5jk7b733se67UQ7XizGM="},"voting_power":"49884162","proposer_priority":"513415451"},{"address":"01b2c0f03ba9b2f352a39bc45ce37aa2710b5af4","pub_key":{"type":"tendermint/PubKeyEd25519","value":"m2EQh1+c953MguY2I+O0+arfUVgEp57S7R7v8UOsLlM="},"voting_power":"45539042","proposer_priority":"-29276205"},{"address":"410c1ebaf54c090f0a9132dfa9f79e0f480b2c06","pub_key":{"type":"tendermint/PubKeyEd25519","value":"b7/i2ANiYV7Cta01WwjBIuN5mJT1qMgcE69gkJ4bBX8="},"voting_power":"40598279","proposer_priority":"716030867"},{"address":"c2989ed51e73b742e4991cf915d6339419657ad5","pub_key":{"type":"tendermint/PubKeyEd25519","value":"wFN94JzYFuZ9YiyRCoT7r4rnfoVaPiKDVmMYhsKn77o="},"voting_power":"40427821","proposer_priority":"707236170"},{"address":"f6866f1f291d5a6740480c6f0774aba45177b68d","pub_key":{"type":"tendermint/PubKeyEd25519","value":"SlYCRkhYoEI/4dsRq81rc7vN6eArQOD375W1oexAOPs="},"voting_power":"40022790","proposer_priority":"200098693"},{"address":"37b7e284915654acc06e394fb423dcae6a4fd94f","pub_key":{"type":"tendermint/PubKeyEd25519","value":"8YnR9Xa/B9WP201LE4tDTtMKYnzW5OTpRzYv9PtvEgk="},"voting_power":"40010006","proposer_priority":"68466812"},{"address":"1a4b644c16a4a1904618f8d3c0f6a67223038390","pub_key":{"type":"tendermint/PubKeyEd25519","value":"ifVDlvP2zSzTP+4pcCMgpUm2aROakV8bjFtq6FGKRZM="},"voting_power":"40002650","proposer_priority":"104011443"},{"address":"53406de83381dd804632447f1baed00cf28048e0","pub_key":{"type":"tendermint/PubKeyEd25519","value":"UfW0oW3DujAhga+d4Jf1kalUa5dUsrHLhtLZgN9M6sA="},"voting_power":"30562456","proposer_priority":"1556052"},{"address":"b3f0e466cf435fc603117e6cd516ab7e6337e212","pub_key":{"type":"tendermint/PubKeyEd25519","value":"kuNvI/L/PpQSIGgn0DNFuB7lqt8jIdFUbTtAmm6Q+ZU="},"voting_power":"30037649","proposer_priority":"877877493"},{"address":"300636fc6b27a7016f3aef03c231aa63163eeb5d","pub_key":{"type":"tendermint/PubKeyEd25519","value":"gruCRPAggDzkufHhdJ+HjaO2Tp/+UhtLs1uJwSS+fak="},"voting_power":"30010819","proposer_priority":"-267967571"},{"address":"b032b8177b81d9ed1bce8bf0ff14c45d17e821f1","pub_key":{"type":"tendermint/PubKeyEd25519","value":"wEH/QHM8PzhiheI9SPCLVHfBsE4f68lL42rTLox8wgc="},"voting_power":"30000004","proposer_priority":"-550538340"},{"address":"6cc0a1531ff58d61dc9061c5b89bc0efc07911fc","pub_key":{"type":"tendermint/PubKeyEd25519","value":"fu9i2vgnxjTxA2yISFpQdRmyJsfLt5zyA+Z7NijDK34="},"voting_power":"29875457","proposer_priority":"374194604"},{"address":"3965c223e13ac66f8175c86e657d396c711964b7","pub_key":{"type":"tendermint/PubKeyEd25519","value":"I43SS5LrmBqDQURAVVyisYxX3Li+mYK7P7Sij6+yYIE="},"voting_power":"28798994","proposer_priority":"548217620"},{"address":"76f6dd3f07bf2e9ff5b0a188bc08d5a4efcf40d9","pub_key":{"type":"tendermint/PubKeyEd25519","value":"UcCQ3UtXPjyF5k4+KMonaWnQ22vtlZq6DfygnmaTp9Y="},"voting_power":"26528649","proposer_priority":"704303420"},{"address":"1b21f9bf3b9a650607cced8c0c2fbeca3211791b","pub_key":{"type":"tendermint/PubKeyEd25519","value":"YjIyQkXPkU4HTUXD74DGwTXk1apkue35R0AispufH0g="},"voting_power":"25573773","proposer_priority":"-709916713"},{"address":"1b03ab3c744545b281d224127708a9ec7487e58c","pub_key":{"type":"tendermint/PubKeyEd25519","value":"YhgNr8OiPRzSI1MybqgRZf3yWYPGMy7vNPxSzsfRFrE="},"voting_power":"24542545","proposer_priority":"971045626"},{"address":"e2fc4df9e7870fc8238136faca07a120cc078a0c","pub_key":{"type":"tendermint/PubKeyEd25519","value":"1TSNDP080xciccCHFOrXhVB1XBDPSVwMK85lq/H0dC8="},"voting_power":"24515904","proposer_priority":"835593963"},{"address":"671d816776e8c7fae18dc122fb6dc231956c6d48","pub_key":{"type":"tendermint/PubKeyEd25519","value":"xafI5gPZEFu7ifGpLfsS6VbkLvE6PLsEXEnQw1TQAn0="},"voting_power":"24515270","proposer_priority":"744946864"},{"address":"6bdc45074c28d7467d37f875091fa5036ac62eed","pub_key":{"type":"tendermint/PubKeyEd25519","value":"+sad+WQif7UkeXMjgbtA2w+qNh/um0saLp5ICSujp/M="},"voting_power":"24000005","proposer_priority":"-473006639"},{"address":"718160e3538e0b0ed83d3b3dc4ece83809fcd107","pub_key":{"type":"tendermint/PubKeyEd25519","value":"uxrtntmwVXujeUpI8VJD9MjGubF2xptYnHFmT4lklvk="},"voting_power":"22876803","proposer_priority":"-665879227"},{"address":"b03f0cb997f3428ef3968833bca2f2a4f613a8eb","pub_key":{"type":"tendermint/PubKeyEd25519","value":"afEmBb9L2G5I/aE4ZJ24dGFEZJDvrdBH93of8jYGv20="},"voting_power":"20279407","proposer_priority":"-187193283"},{"address":"77be23139b650d69ede7985b31fd7b1cee9ca7d3","pub_key":{"type":"tendermint/PubKeyEd25519","value":"7LqCm1ivwRYq/ZFrdBVV0jaitBm1ub/7A5sPsoNTF5g="},"voting_power":"20219782","proposer_priority":"-1056392398"},{"address":"5c78a0762f13b0b2a793e82d1a178eb45642234d","pub_key":{"type":"tendermint/PubKeyEd25519","value":"tFyF8SVRJ81z9O2q2wzZMHUit2vgDWiEnv2aK/CSCoI="},"voting_power":"20216309","proposer_priority":"-297860650"},{"address":"70dac78498afc6d766d860c433265a274d48b7d8","pub_key":{"type":"tendermint/PubKeyEd25519","value":"VDk6q4U643fvFagHQjWNoKUqnr6lU3NKBo9ERAS5kDQ="},"voting_power":"20134471","proposer_priority":"919789418"},{"address":"b41ef50272a7a2398ba617d3c963986ebebc5b2d","pub_key":{"type":"tendermint/PubKeyEd25519","value":"GprPmRax5CnFvFsS9f6mGstf7zRXEvJdEoBuvENp/cI="},"voting_power":"20101846","proposer_priority":"-1039644225"},{"address":"521d2a344e322b8ba76401c833a1ec74ea2bd075","pub_key":{"type":"tendermint/PubKeyEd25519","value":"BzddkwvM57jNIsA7TxUyS8wLLSTbDpa8IBqiGy+LaxE="},"voting_power":"20020216","proposer_priority":"-187954646"},{"address":"35f6c0359084c37ef6eb4061b694386910093b99","pub_key":{"type":"tendermint/PubKeyEd25519","value":"AatuXTZOwAWhufDJiRXkrDce9jeVvbcZOhsuw6e+gNM="},"voting_power":"20002859","proposer_priority":"-105595836"},{"address":"2a1b3fee6b29ea1828d427f0e5c3887cb7e64487","pub_key":{"type":"tendermint/PubKeyEd25519","value":"WGS64GhNVcsRoSuQ0g1+iVaBp7CeQaITSPIqD+xcUAM="},"voting_power":"20002754","proposer_priority":"-712955740"},{"address":"ccb2909dcd316fc5e738dcdb1f14360f04b44320","pub_key":{"type":"tendermint/PubKeyEd25519","value":"R+agpHoODVGSoS3i/1cM3kgD6lbZKmJWYLcPVVmUn8Q="},"voting_power":"20000945","proposer_priority":"363179210"},{"address":"616a45478336b92583cfe29e8ee942802de69eae","pub_key":{"type":"tendermint/PubKeyEd25519","value":"XVwl5MdQw44yq0kvTQ2W1OKtFnO5/Z9snTk+2TI0PhQ="},"voting_power":"19589722","proposer_priority":"-456274238"},{"address":"fbc2a3f682b507f1a3a5d4c44c8cbb6af0860296","pub_key":{"type":"tendermint/PubKeyEd25519","value":"LWOEtc00ggBtocjJaKbNQ2ArZJMD5IaYQ/a/agRacPE="},"voting_power":"19525184","proposer_priority":"-653020167"},{"address":"2c4e108c3f09cf9b8b1515d3b26d50e332d86e3b","pub_key":{"type":"tendermint/PubKeyEd25519","value":"WIfDZZv31OnJDUv5ZR78/HLozeQZESoDYKcbJscoxdc="},"voting_power":"18048625","proposer_priority":"113025989"},{"address":"f42c7e49df4f4d0993237959518415e473792392","pub_key":{"type":"tendermint/PubKeyEd25519","value":"HS7PneXHx4p3Pw4/Tf3k8dqqq3zA85tgmtD2Za2zlfA="},"voting_power":"17858362","proposer_priority":"571033906"},{"address":"c268fe001a533b6f0860167381384363a790fd0d","pub_key":{"type":"tendermint/PubKeyEd25519","value":"I8TBe0JOeuJssNbOLGCyer0iMDlz6Ij/OcAxmCEpS/E="},"voting_power":"16755781","proposer_priority":"211144280"},{"address":"9a323ff30c24f59e160aaf02a57c63e11c41495e","pub_key":{"type":"tendermint/PubKeyEd25519","value":"YeVzZ/HjpKl0i0ddoaeekIdQdyvincMYD6YjPsSQpw4="},"voting_power":"13200031","proposer_priority":"-2754042"},{"address":"a8f38fcb39b580c4b856dcaed882d504a216ee00","pub_key":{"type":"tendermint/PubKeyEd25519","value":"Kda/tpSMpgzH0ImTXQR75NTU66y9D14hVCucT2y+9KA="},"voting_power":"12539120","proposer_priority":"588906164"},{"address":"7ea986d8723587f5e653c0df785fcfce39238785","pub_key":{"type":"tendermint/PubKeyEd25519","value":"ABSElT/KHsxFF5u5b5hS8LhDS30VsSee3V1uZAS6J8w="},"voting_power":"12480381","proposer_priority":"573995641"},{"address":"aea26001de9b15d4dad32808712864e61ae52a17","pub_key":{"type":"tendermint/PubKeyEd25519","value":"RtPXsKSt++T/v0Xqr04rraOXAdQcZCLpz5Q474EmtRw="},"voting_power":"12241388","proposer_priority":"-749753904"},{"address":"2972b0df619f73d1a5c8933aff22ad8c9258f370","pub_key":{"type":"tendermint/PubKeyEd25519","value":"NhA92zy67zrx+Sy1A7i6EMIvELJIB7swGRIeoQaf0Xo="},"voting_power":"12179809","proposer_priority":"837533236"},{"address":"a8a2bf2709ca05755f881ebd939991aea9f88cf1","pub_key":{"type":"tendermint/PubKeyEd25519","value":"srcpua9x2zK2xbHtXb1E/L6lJRTn7ehJggnlcBiCEPw="},"voting_power":"12119999","proposer_priority":"-938606509"},{"address":"23df4a8418762cea4d24b7227ad125df76d2baf6","pub_key":{"type":"tendermint/PubKeyEd25519","value":"2e/NDjibT4BFoyfT5myyzMIGpDPAiHjeLCNXSj6/Ow0="},"voting_power":"12048703","proposer_priority":"-145673241"},{"address":"9407fa25941c932473f8e2b1ecc5481e3ae15a71","pub_key":{"type":"tendermint/PubKeyEd25519","value":"uRLdogR9YCZFCUXggDpKqPJnRd1Tn52+twiAnkvC3zk="},"voting_power":"12041246","proposer_priority":"-558376975"},{"address":"65c584027f5fd8349027fa5bf1d4ed3828c29058","pub_key":{"type":"tendermint/PubKeyEd25519","value":"cxBI4Ef7MfASd1COwn2eqgVooP0Qp2eDQls/pv7dMa8="},"voting_power":"12031194","proposer_priority":"480976436"},{"address":"466d1236b8c77925b25219e4ffe34dd069351847","pub_key":{"type":"tendermint/PubKeyEd25519","value":"9pe9aXHTo4AckQ3eR2mHixLqZLjAw34OnzQLZtikS4c="},"voting_power":"12024095","proposer_priority":"267836605"},{"address":"b8f9204a348e0dbc85abe5bc98936d6b84c01e45","pub_key":{"type":"tendermint/PubKeyEd25519","value":"L2YmP55MMBPcbucOqkWZ8O5Xmwt/nQR9KABn5WRNC7I="},"voting_power":"12013392","proposer_priority":"894437732"},{"address":"2ab301f354a74f45afb347b2c00cdceb09cc719d","pub_key":{"type":"tendermint/PubKeyEd25519","value":"qFSnvrYxWgRHPWz424XRZOiClZXjHoDQBv1CMQDS7/o="},"voting_power":"12006092","proposer_priority":"-251313246"},{"address":"9215b6b8ec64cb9e7b461c3945e6ce00da86f8f2","pub_key":{"type":"tendermint/PubKeyEd25519","value":"Qng5pYzBWTouQK1EnLQxVfftKSo8DlOYydIuo8Am9ZA="},"voting_power":"12005923","proposer_priority":"-322140379"},{"address":"a9c7a884bd7d6e799ef5311bf9168e0105bce416","pub_key":{"type":"tendermint/PubKeyEd25519","value":"FdztpZioZVj+YQ0d5JCHZWBehgSFuWrjG8DKmRgwNRE="},"voting_power":"12004941","proposer_priority":"-838613255"},{"address":"f1961eee75e5a1c8a11a5bf72fb368b990bdd289","pub_key":{"type":"tendermint/PubKeyEd25519","value":"JrRT1aj11c5mrpfsh//PeKA+XCR6wlaDgAsB+fegNAs="},"voting_power":"12004031","proposer_priority":"141472735"},{"address":"9fee83fbf0bd5a4d71519efc16d0d438c203dd5a","pub_key":{"type":"tendermint/PubKeyEd25519","value":"sIJcUpa8J1/4qoXaZ0tbYNEkjSW25AJ31LsuiEYSivU="},"voting_power":"12002406","proposer_priority":"632883344"},{"address":"b44fc7d70708095fbc30af164410749483f96439","pub_key":{"type":"tendermint/PubKeyEd25519","value":"6nmVExrET0kDePGFZDjbNirkLOw0SKuYgHsvn+w32DY="},"voting_power":"12001462","proposer_priority":"-441697099"},{"address":"c4ef42d72d69f4b58ffe2a58db1335269842ac43","pub_key":{"type":"tendermint/PubKeyEd25519","value":"OwyWdapIzAWvKBByvLv2a5gl6yD6gK67eFzqf/SrlZE="},"voting_power":"12001234","proposer_priority":"-786662877"},{"address":"60c51d643048d0a426171bd0deabd1689a89690f","pub_key":{"type":"tendermint/PubKeyEd25519","value":"UlAJPv+2h+ChlG5uYdajVs3Eq+vp5/1Y83VoRk7tR7s="},"voting_power":"12001194","proposer_priority":"130231728"},{"address":"5d673308ca2b45e5222303836d833e0f70b1f9b0","pub_key":{"type":"tendermint/PubKeyEd25519","value":"9wI5MjjfeZnz5otZED8KVlqXREJovA8DxYrBWIj7AEQ="},"voting_power":"12000513","proposer_priority":"576467062"},{"address":"06be03f1b334fc3f7947d0d031f7dfa568005bd4","pub_key":{"type":"tendermint/PubKeyEd25519","value":"RneQYjxZQGtzfYRO38NExiVtgjdagu1cirb3XQKLD4c="},"voting_power":"12000393","proposer_priority":"966074639"},{"address":"d7076d16c2e91e0eb16753575ac3cee3057781c0","pub_key":{"type":"tendermint/PubKeyEd25519","value":"pwHw+O9X178B8Tmak4BelOCz0VRb2FkGHZzWSrS/ozs="},"voting_power":"12000372","proposer_priority":"-895101422"},{"address":"913834357f92e577cdc02b1e682c65e86d016ae5","pub_key":{"type":"tendermint/PubKeyEd25519","value":"GT1pq3LZ8xz8gNdWed8d2AJIWAeQ+qhxHBCOld24k0U="},"voting_power":"12000205","proposer_priority":"685373803"},{"address":"3ce85b466ff74142307b4b03ddacf17a52d46373","pub_key":{"type":"tendermint/PubKeyEd25519","value":"OPbuQy+x6/viqrL7Pc21FLRNquvAUeegJSop3m9yEhc="},"voting_power":"12000185","proposer_priority":"-960247617"},{"address":"dc0802ad3262ecd784ce339d87073c836e31c69e","pub_key":{"type":"tendermint/PubKeyEd25519","value":"3md1MTXk0zmxbKsPNAi1xpUvt5LJu4yUjGDtXaD7dKk="},"voting_power":"12000030","proposer_priority":"-855112185"},{"address":"31e623ab0ff5dc408c052072f96df3d4506718ce","pub_key":{"type":"tendermint/PubKeyEd25519","value":"KZACGMrjgoapUYJoZnXLubvTj7Lyvi8jPs2FZCDrdgU="},"voting_power":"12000008","proposer_priority":"476095106"},{"address":"02d49d26c87ed7dc1e38d7bfe5b7fcdf5a3a3164","pub_key":{"type":"tendermint/PubKeyEd25519","value":"s4FTOQAIHzWzpl4G/wsgj5cYgJDZAmcAimOud5VcMcQ="},"voting_power":"12000006","proposer_priority":"777903824"},{"address":"646d5c6384c84c2e660bf268d0a4ebc1ffbceea6","pub_key":{"type":"tendermint/PubKeyEd25519","value":"fN+f+HYif75SNXJTVJrQGOR19797ur001Hz/VUs3QgQ="},"voting_power":"12000005","proposer_priority":"-219372390"},{"address":"e6f8df77989972328c927e5493db6908936a4ea0","pub_key":{"type":"tendermint/PubKeyEd25519","value":"YZet9tH+pLca8oDQjPBqJdvCoSqfd+I7AAk4cwR4Lek="},"voting_power":"7826655","proposer_priority":"-922193094"},{"address":"0aabc33bff6e288ea151463ca133e38051bada85","pub_key":{"type":"tendermint/PubKeyEd25519","value":"8lf54CEx2NRPvLKy/wxlB8+XAGac4MHbHzKXHWwgHgM="},"voting_power":"2458471","proposer_priority":"-840428682"},{"address":"e71b23c378e4c465bfe96ad48e7f93f237e939dc","pub_key":{"type":"tendermint/PubKeyEd25519","value":"AATtUDupHksDYD1M0De5GdvoFcULQDol0XWSZoOt3lQ="},"voting_power":"2119422","proposer_priority":"220946237"},{"address":"bfee4eba8e3307bcb074ad575ab9fa4524af2a25","pub_key":{"type":"tendermint/PubKeyEd25519","value":"7dsEUHdM6F+9oLMSl3xF2hRe2FqRcLOI9HNT43tDbR0="},"voting_power":"1629711","proposer_priority":"923769646"},{"address":"fbe1e3ed324fa17379403acfc3dd28fa882c6e65","pub_key":{"type":"tendermint/PubKeyEd25519","value":"gPUH+r4GyzzyALi19pL5YiHdbZxwVNFNq/aeqQMPp4E="},"voting_power":"1502555","proposer_priority":"-747171267"},{"address":"f8260be47a1c7d42bbd9a656bdf8e3d3f28cbab2","pub_key":{"type":"tendermint/PubKeyEd25519","value":"tPfAAuLM1NQUvi6onz2bpiEFQ0nS0G+HH+tiTUD74GQ="},"voting_power":"1057984","proposer_priority":"682268587"},{"address":"7c4b90fe7584bad71c5c6ab0877b8c39cd289187","pub_key":{"type":"tendermint/PubKeyEd25519","value":"rUPhoUYxrAZ8nNVmDzYBG8hKeq4DgFubS9csT5HPU/s="},"voting_power":"898826","proposer_priority":"-744711484"},{"address":"3b2a7b69569e7835324f35f7f2264d008e850c12","pub_key":{"type":"tendermint/PubKeyEd25519","value":"OPrm7Vg+qi+G712UapKeeDBUbEeNbgHSRweHrI596h8="},"voting_power":"650165","proposer_priority":"932709426"},{"address":"edd5716db0738cb0eecd0a04aeb186c4a46720e9","pub_key":{"type":"tendermint/PubKeyEd25519","value":"crYIcbR36Gb3cv2i6j6iuDozHSmg5dYW/e7dOt0xRgw="},"voting_power":"398850","proposer_priority":"-419352069"},{"address":"f868d1eb91e32d7f21ef78462feac2892d09eb56","pub_key":{"type":"tendermint/PubKeyEd25519","value":"t6m29TlSMiMcl6l+KBJ11lX67QK2tC8P5LsF0dbWafk="},"voting_power":"217502","proposer_priority":"453775185"},{"address":"59c09cd227541c5ee7d911084ada87352c899851","pub_key":{"type":"tendermint/PubKeyEd25519","value":"qCUSWl0pwiYSJwbknU4Fc6sZlFJINRCICBtz5xFYo8E="},"voting_power":"191753","proposer_priority":"239487984"},{"address":"478a66d6d2fcb936ba235daeb5a35ba454c048a7","pub_key":{"type":"tendermint/PubKeyEd25519","value":"S5aSb0vlow3s2cR9n/PfrvEQpFdmnmFM4KaTVu2eO9M="},"voting_power":"176183","proposer_priority":"-281451715"},{"address":"15839d4135d8c8fa603f533ea3ba80a687fe527a","pub_key":{"type":"tendermint/PubKeyEd25519","value":"eXyi7xIpmOFoZ3LyW89SRog5/JvWOLGldYVCGOTeqOY="},"voting_power":"175897","proposer_priority":"-701319536"},{"address":"13247270303a9d66ea8ffa02614a8b5369adda53","pub_key":{"type":"tendermint/PubKeyEd25519","value":"uE6n04V4cBXf9pOiIET69qDKtIrxqHdJXqyrHWt4WXE="},"voting_power":"174527","proposer_priority":"-828126470"},{"address":"409d07de01ea2a63c779c0246562aa278a95611e","pub_key":{"type":"tendermint/PubKeyEd25519","value":"6QAY+9wbHL6x2JRX67y8+O6iYVaD40AaAL3s6w0WyE4="},"voting_power":"158846","proposer_priority":"400010513"},{"address":"4e30f6a73766baa5fe6db7181b7fd08329c103e0","pub_key":{"type":"tendermint/PubKeyEd25519","value":"qE97rQR9bxg3PoQQNzrW7jjiDfhURt212Ljh4NthQsE="},"voting_power":"129730","proposer_priority":"312908621"},{"address":"f70543a091a45a8c0de04e5ce9e40d9c16d980e1","pub_key":{"type":"tendermint/PubKeyEd25519","value":"rkXcWaND0QwZ02iCoTek8THzVbBU2ck0oVi0VuhK9E8="},"voting_power":"119288","proposer_priority":"-555870672"},{"address":"e9b8e658e2f1aea81d57b78c8bdd2aa93a1bfc01","pub_key":{"type":"tendermint/PubKeyEd25519","value":"lMfcJzktWf2G2VKv2EkAnKlhIO21BVN1TUhlDUxhlXg="},"voting_power":"119093","proposer_priority":"724237762"},{"address":"3daf658a7b8dc1a5186f7c1547ab96ca35686575","pub_key":{"type":"tendermint/PubKeyEd25519","value":"yL6jB1QoQGd4jgWnLZEg+YT2gsp2FiLglItPbC4UCuo="},"voting_power":"116590","proposer_priority":"-615195010"},{"address":"02c375d9b259cdbb7b8a97d71b2a0a0dbed890e0","pub_key":{"type":"tendermint/PubKeyEd25519","value":"W+0vG8CMeptG2Jc6rgBrVJAd0Ap0slNLKOmH2VDzA0E="},"voting_power":"107749","proposer_priority":"-71114881"},{"address":"556f693ffba4b4c6de7af822852bd6e8a1a1d410","pub_key":{"type":"tendermint/PubKeyEd25519","value":"kQRSDLmt8qpSDpPuxGQj0QDVToWhXF0uHKtnoZgoDx4="},"voting_power":"107326","proposer_priority":"-725928420"},{"address":"b31b8ebf6857480b1281e756f0fd67243ac7a202","pub_key":{"type":"tendermint/PubKeyEd25519","value":"7jdjjvLZhZLmlwwEceQu/9PU3/D4lLmeYqg7ohiQrvM="},"voting_power":"106665","proposer_priority":"417900465"},{"address":"8dabad1a71629e4938de8c44d0bd2f2ce1eabd08","pub_key":{"type":"tendermint/PubKeyEd25519","value":"SySq3SuLFsjmhqTOxtNQwBuHd/xlWBipkHCK/nvHcTU="},"voting_power":"106170","proposer_priority":"-1076373229"},{"address":"615e9b48cd48fda3a754f28e3f42f77c2e49b0d7","pub_key":{"type":"tendermint/PubKeyEd25519","value":"xGdHKqlekTKkSPiItkpINJl+CUc4TF0oQ3nh9KYE+vI="},"voting_power":"104964","proposer_priority":"260792826"},{"address":"43965a09c5be34d027a599bd6f450f5e6a13ddf7","pub_key":{"type":"tendermint/PubKeyEd25519","value":"6g6hfqPKqus5RBJ5nDfS5ZxHCNZQEl9KuCf+hbFAoJU="},"voting_power":"96973","proposer_priority":"399494349"},{"address":"25764380a6df1c9cad29a842a049f690ab0f08bc","pub_key":{"type":"tendermint/PubKeyEd25519","value":"sRm+BMdyAFN5ifMXlKUn7VtpkGTqCSNkA9/wDQPy+S8="},"voting_power":"63683","proposer_priority":"754499561"}],"proposer":{"address":"77be23139b650d69ede7985b31fd7b1cee9ca7d3","pub_key":{"type":"tendermint/PubKeyEd25519","value":"7LqCm1ivwRYq/ZFrdBVV0jaitBm1ub/7A5sPsoNTF5g="},"voting_power":"20219782","proposer_priority":"-1056392398"},"total_voting_power":2039442572},"trusted_height":"1-1633807","trusted_validators":{"validators":[{"address":"3c70d8336dee739bdf2d1c5a0fca0a88b87b9a6d","pub_key":{"type":"tendermint/PubKeyEd25519","value":"hg4sm5gNYb9T4dWC94nDW+30olLECT6Tkrp1FtnVBRE="},"voting_power":"110885211","proposer_priority":"244371803"},{"address":"ec033669fe506006f870bd8b12cb524a3abc3693","pub_key":{"type":"tendermint/PubKeyEd25519","value":"ADyMDDsnw93NdBSqXRlRdeWMNnicitm1ikJ14J240Jk="},"voting_power":"70917400","proposer_priority":"350620445"},{"address":"33ecd5899a91105bb4e51066eb138189bc3825ad","pub_key":{"type":"tendermint/PubKeyEd25519","value":"dWFxEYRPzIyQpPwSHyw30ZZDZn6GgZJpDyMUvxRSbtQ="},"voting_power":"66631704","proposer_priority":"-310019549"},{"address":"44c395a4a96c6d1a450ed33b5a8ddb359cefed36","pub_key":{"type":"tendermint/PubKeyEd25519","value":"q6lQj+wYNZ+QQu9zgWoWfSXcwnQNEzb2a7HnyWyBQf0="},"voting_power":"66396058","proposer_priority":"797562705"},{"address":"c27b627c2fd11ee1e3e850738720692be2e3f4c6","pub_key":{"type":"tendermint/PubKeyEd25519","value":"vuHjnsOVyXFYot1RaV+Wx+ck2hOOvb9/QXwXuzeWcm4="},"voting_power":"60834013","proposer_priority":"184498575"},{"address":"64cdacb04fb9c3b290579644f9edbd05cb00bb7d","pub_key":{"type":"tendermint/PubKeyEd25519","value":"ydZBYn1uztV039ywjiQ8rRycnXHc62hK10+4ZalaQVY="},"voting_power":"60576465","proposer_priority":"94594741"},{"address":"5295664886565f43a3074140551a755872bf6ffd","pub_key":{"type":"tendermint/PubKeyEd25519","value":"b3y28iu4R+IBmSIPwJA90k6yqb5kJPgvDVCmcnpSJck="},"voting_power":"60019185","proposer_priority":"-375043753"},{"address":"2a3d40c3e7ce8d6b55077335cab141c2e11ac254","pub_key":{"type":"tendermint/PubKeyEd25519","value":"kUQmCZtP4I3q5FkKaPVTvXnr3UMD7GwlhnGdkXqsbbg="},"voting_power":"56084140","proposer_priority":"-138046953"},{"address":"f2e6bb25d8d269c9737bd3789b11dc76c4fc56e7","pub_key":{"type":"tendermint/PubKeyEd25519","value":"AQbty1lMHkV1ZHEcW8C6AYK8QhMfxaAhBdp2w+Ykp28="},"voting_power":"56060558","proposer_priority":"962227198"},{"address":"bb386aa0e7b37eb33516aebb58cae3574af85d1f","pub_key":{"type":"tendermint/PubKeyEd25519","value":"cDPnCL7aSojDMLgu23KG3wIVb6cfpTFS3DBimDhtQro="},"voting_power":"50167947","proposer_priority":"583625887"},{"address":"43804074334ac6111233037efe20105c516b0e9f","pub_key":{"type":"tendermint/PubKeyEd25519","value":"4HpLoBBK/GzdceBjoD4ZXBCeMyv14WPDeoCtZ1BqYv0="},"voting_power":"50022601","proposer_priority":"-535173650"},{"address":"da37d2935eae0a92de55e86db2e887b4f18575f0","pub_key":{"type":"tendermint/PubKeyEd25519","value":"x7i3trbMJFfCcb9SAfZbL8s2M1D0OHQ4piGP+DmkJ0k="},"voting_power":"50004013","proposer_priority":"90028719"},{"address":"9c4cc8543dc684948490681118957942b29a9200","pub_key":{"type":"tendermint/PubKeyEd25519","value":"mQhc5iQg6aagqCU3i8bgNFr5jk7b733se67UQ7XizGM="},"voting_power":"49884162","proposer_priority":"-52734541"},{"address":"01b2c0f03ba9b2f352a39bc45ce37aa2710b5af4","pub_key":{"type":"tendermint/PubKeyEd25519","value":"m2EQh1+c953MguY2I+O0+arfUVgEp57S7R7v8UOsLlM="},"voting_power":"45539042","proposer_priority":"-13180117"},{"address":"410c1ebaf54c090f0a9132dfa9f79e0f480b2c06","pub_key":{"type":"tendermint/PubKeyEd25519","value":"b7/i2ANiYV7Cta01WwjBIuN5mJT1qMgcE69gkJ4bBX8="},"voting_power":"40598279","proposer_priority":"-645253375"},{"address":"c2989ed51e73b742e4991cf915d6339419657ad5","pub_key":{"type":"tendermint/PubKeyEd25519","value":"wFN94JzYFuZ9YiyRCoT7r4rnfoVaPiKDVmMYhsKn77o="},"voting_power":"40427821","proposer_priority":"-631206700"},{"address":"f6866f1f291d5a6740480c6f0774aba45177b68d","pub_key":{"type":"tendermint/PubKeyEd25519","value":"SlYCRkhYoEI/4dsRq81rc7vN6eArQOD375W1oexAOPs="},"voting_power":"40022790","proposer_priority":"955372549"},{"address":"37b7e284915654acc06e394fb423dcae6a4fd94f","pub_key":{"type":"tendermint/PubKeyEd25519","value":"8YnR9Xa/B9WP201LE4tDTtMKYnzW5OTpRzYv9PtvEgk="},"voting_power":"40010006","proposer_priority":"825453724"},{"address":"1a4b644c16a4a1904618f8d3c0f6a67223038390","pub_key":{"type":"tendermint/PubKeyEd25519","value":"ifVDlvP2zSzTP+4pcCMgpUm2aROakV8bjFtq6FGKRZM="},"voting_power":"40002650","proposer_priority":"861984059"},{"address":"53406de83381dd804632447f1baed00cf28048e0","pub_key":{"type":"tendermint/PubKeyEd25519","value":"UfW0oW3DujAhga+d4Jf1kalUa5dUsrHLhtLZgN9M6sA="},"voting_power":"30562456","proposer_priority":"-14927908"},{"address":"b3f0e466cf435fc603117e6cd516ab7e6337e212","pub_key":{"type":"tendermint/PubKeyEd25519","value":"kuNvI/L/PpQSIGgn0DNFuB7lqt8jIdFUbTtAmm6Q+ZU="},"voting_power":"30037649","proposer_priority":"931717671"},{"address":"300636fc6b27a7016f3aef03c231aa63163eeb5d","pub_key":{"type":"tendermint/PubKeyEd25519","value":"gruCRPAggDzkufHhdJ+HjaO2Tp/+UhtLs1uJwSS+fak="},"voting_power":"30010819","proposer_priority":"-210532173"},{"address":"b032b8177b81d9ed1bce8bf0ff14c45d17e821f1","pub_key":{"type":"tendermint/PubKeyEd25519","value":"wEH/QHM8PzhiheI9SPCLVHfBsE4f68lL42rTLox8wgc="},"voting_power":"30000004","proposer_priority":"-491653732"},{"address":"6cc0a1531ff58d61dc9061c5b89bc0efc07911fc","pub_key":{"type":"tendermint/PubKeyEd25519","value":"fu9i2vgnxjTxA2yISFpQdRmyJsfLt5zyA+Z7NijDK34="},"voting_power":"29875457","proposer_priority":"449768510"},{"address":"3965c223e13ac66f8175c86e657d396c711964b7","pub_key":{"type":"tendermint/PubKeyEd25519","value":"I43SS5LrmBqDQURAVVyisYxX3Li+mYK7P7Sij6+yYIE="},"voting_power":"28798994","proposer_priority":"768037568"},{"address":"76f6dd3f07bf2e9ff5b0a188bc08d5a4efcf40d9","pub_key":{"type":"tendermint/PubKeyEd25519","value":"UcCQ3UtXPjyF5k4+KMonaWnQ22vtlZq6DfygnmaTp9Y="},"voting_power":"26528649","proposer_priority":"-811092974"},{"address":"1b21f9bf3b9a650607cced8c0c2fbeca3211791b","pub_key":{"type":"tendermint/PubKeyEd25519","value":"YjIyQkXPkU4HTUXD74DGwTXk1apkue35R0AispufH0g="},"voting_power":"25573773","proposer_priority":"-57917151"},{"address":"1b03ab3c744545b281d224127708a9ec7487e58c","pub_key":{"type":"tendermint/PubKeyEd25519","value":"YhgNr8OiPRzSI1MybqgRZf3yWYPGMy7vNPxSzsfRFrE="},"voting_power":"24542545","proposer_priority":"-278212832"},{"address":"e2fc4df9e7870fc8238136faca07a120cc078a0c","pub_key":{"type":"tendermint/PubKeyEd25519","value":"1TSNDP080xciccCHFOrXhVB1XBDPSVwMK85lq/H0dC8="},"voting_power":"24515904","proposer_priority":"-410094601"},{"address":"671d816776e8c7fae18dc122fb6dc231956c6d48","pub_key":{"type":"tendermint/PubKeyEd25519","value":"xafI5gPZEFu7ifGpLfsS6VbkLvE6PLsEXEnQw1TQAn0="},"voting_power":"24515270","proposer_priority":"-500656744"},{"address":"6bdc45074c28d7467d37f875091fa5036ac62eed","pub_key":{"type":"tendermint/PubKeyEd25519","value":"+sad+WQif7UkeXMjgbtA2w+qNh/um0saLp5ICSujp/M="},"voting_power":"24000005","proposer_priority":"389877835"},{"address":"718160e3538e0b0ed83d3b3dc4ece83809fcd107","pub_key":{"type":"tendermint/PubKeyEd25519","value":"uxrtntmwVXujeUpI8VJD9MjGubF2xptYnHFmT4lklvk="},"voting_power":"22876803","proposer_priority":"347514315"},{"address":"b03f0cb997f3428ef3968833bca2f2a4f613a8eb","pub_key":{"type":"tendermint/PubKeyEd25519","value":"afEmBb9L2G5I/aE4ZJ24dGFEZJDvrdBH93of8jYGv20="},"voting_power":"20279407","proposer_priority":"-865191249"},{"address":"77be23139b650d69ede7985b31fd7b1cee9ca7d3","pub_key":{"type":"tendermint/PubKeyEd25519","value":"7LqCm1ivwRYq/ZFrdBVV0jaitBm1ub/7A5sPsoNTF5g="},"voting_power":"20219782","proposer_priority":"313041958"},{"address":"5c78a0762f13b0b2a793e82d1a178eb45642234d","pub_key":{"type":"tendermint/PubKeyEd25519","value":"tFyF8SVRJ81z9O2q2wzZMHUit2vgDWiEnv2aK/CSCoI="},"voting_power":"20216309","proposer_priority":"-967403484"},{"address":"70dac78498afc6d766d860c433265a274d48b7d8","pub_key":{"type":"tendermint/PubKeyEd25519","value":"VDk6q4U643fvFagHQjWNoKUqnr6lU3NKBo9ERAS5kDQ="},"voting_power":"20134471","proposer_priority":"261212876"},{"address":"b41ef50272a7a2398ba617d3c963986ebebc5b2d","pub_key":{"type":"tendermint/PubKeyEd25519","value":"GprPmRax5CnFvFsS9f6mGstf7zRXEvJdEoBuvENp/cI="},"voting_power":"20101846","proposer_priority":"345593555"},{"address":"521d2a344e322b8ba76401c833a1ec74ea2bd075","pub_key":{"type":"tendermint/PubKeyEd25519","value":"BzddkwvM57jNIsA7TxUyS8wLLSTbDpa8IBqiGy+LaxE="},"voting_power":"20020216","proposer_priority":"-831221018"},{"address":"35f6c0359084c37ef6eb4061b694386910093b99","pub_key":{"type":"tendermint/PubKeyEd25519","value":"AatuXTZOwAWhufDJiRXkrDce9jeVvbcZOhsuw6e+gNM="},"voting_power":"20002859","proposer_priority":"-746536370"},{"address":"2a1b3fee6b29ea1828d427f0e5c3887cb7e64487","pub_key":{"type":"tendermint/PubKeyEd25519","value":"WGS64GhNVcsRoSuQ0g1+iVaBp7CeQaITSPIqD+xcUAM="},"voting_power":"20002754","proposer_priority":"685560368"},{"address":"ccb2909dcd316fc5e738dcdb1f14360f04b44320","pub_key":{"type":"tendermint/PubKeyEd25519","value":"R+agpHoODVGSoS3i/1cM3kgD6lbZKmJWYLcPVVmUn8Q="},"voting_power":"20000945","proposer_priority":"-277504848"},{"address":"616a45478336b92583cfe29e8ee942802de69eae","pub_key":{"type":"tendermint/PubKeyEd25519","value":"XVwl5MdQw44yq0kvTQ2W1OKtFnO5/Z9snTk+2TI0PhQ="},"voting_power":"19589722","proposer_priority":"997588158"},{"address":"fbc2a3f682b507f1a3a5d4c44c8cbb6af0860296","pub_key":{"type":"tendermint/PubKeyEd25519","value":"LWOEtc00ggBtocjJaKbNQ2ArZJMD5IaYQ/a/agRacPE="},"voting_power":"19525184","proposer_priority":"809490321"},{"address":"2c4e108c3f09cf9b8b1515d3b26d50e332d86e3b","pub_key":{"type":"tendermint/PubKeyEd25519","value":"WIfDZZv31OnJDUv5ZR78/HLozeQZESoDYKcbJscoxdc="},"voting_power":"18048625","proposer_priority":"-266047189"},{"address":"f42c7e49df4f4d0993237959518415e473792392","pub_key":{"type":"tendermint/PubKeyEd25519","value":"HS7PneXHx4p3Pw4/Tf3k8dqqq3zA85tgmtD2Za2zlfA="},"voting_power":"17858362","proposer_priority":"217455970"},{"address":"c268fe001a533b6f0860167381384363a790fd0d","pub_key":{"type":"tendermint/PubKeyEd25519","value":"I8TBe0JOeuJssNbOLGCyer0iMDlz6Ij/OcAxmCEpS/E="},"voting_power":"16755781","proposer_priority":"5312198"},{"address":"9a323ff30c24f59e160aaf02a57c63e11c41495e","pub_key":{"type":"tendermint/PubKeyEd25519","value":"YeVzZ/HjpKl0i0ddoaeekIdQdyvincMYD6YjPsSQpw4="},"voting_power":"13200031","proposer_priority":"267884376"},{"address":"a8f38fcb39b580c4b856dcaed882d504a216ee00","pub_key":{"type":"tendermint/PubKeyEd25519","value":"Kda/tpSMpgzH0ImTXQR75NTU66y9D14hVCucT2y+9KA="},"voting_power":"12539120","proposer_priority":"948106656"},{"address":"7ea986d8723587f5e653c0df785fcfce39238785","pub_key":{"type":"tendermint/PubKeyEd25519","value":"ABSElT/KHsxFF5u5b5hS8LhDS30VsSee3V1uZAS6J8w="},"voting_power":"12480381","proposer_priority":"941067159"},{"address":"aea26001de9b15d4dad32808712864e61ae52a17","pub_key":{"type":"tendermint/PubKeyEd25519","value":"RtPXsKSt++T/v0Xqr04rraOXAdQcZCLpz5Q474EmtRw="},"voting_power":"12241388","proposer_priority":"-350657324"},{"address":"2972b0df619f73d1a5c8933aff22ad8c9258f370","pub_key":{"type":"tendermint/PubKeyEd25519","value":"NhA92zy67zrx+Sy1A7i6EMIvELJIB7swGRIeoQaf0Xo="},"voting_power":"12179809","proposer_priority":"-794561170"},{"address":"a8a2bf2709ca05755f881ebd939991aea9f88cf1","pub_key":{"type":"tendermint/PubKeyEd25519","value":"srcpua9x2zK2xbHtXb1E/L6lJRTn7ehJggnlcBiCEPw="},"voting_power":"12119999","proposer_priority":"-523243803"},{"address":"23df4a8418762cea4d24b7227ad125df76d2baf6","pub_key":{"type":"tendermint/PubKeyEd25519","value":"2e/NDjibT4BFoyfT5myyzMIGpDPAiHjeLCNXSj6/Ow0="},"voting_power":"12048703","proposer_priority":"279243129"},{"address":"9407fa25941c932473f8e2b1ecc5481e3ae15a71","pub_key":{"type":"tendermint/PubKeyEd25519","value":"uRLdogR9YCZFCUXggDpKqPJnRd1Tn52+twiAnkvC3zk="},"voting_power":"12041246","proposer_priority":"-132461367"},{"address":"65c584027f5fd8349027fa5bf1d4ed3828c29058","pub_key":{"type":"tendermint/PubKeyEd25519","value":"cxBI4Ef7MfASd1COwn2eqgVooP0Qp2eDQls/pv7dMa8="},"voting_power":"12031194","proposer_priority":"908239012"},{"address":"466d1236b8c77925b25219e4ffe34dd069351847","pub_key":{"type":"tendermint/PubKeyEd25519","value":"9pe9aXHTo4AckQ3eR2mHixLqZLjAw34OnzQLZtikS4c="},"voting_power":"12024095","proposer_priority":"696050447"},{"address":"b8f9204a348e0dbc85abe5bc98936d6b84c01e45","pub_key":{"type":"tendermint/PubKeyEd25519","value":"L2YmP55MMBPcbucOqkWZ8O5Xmwt/nQR9KABn5WRNC7I="},"voting_power":"12013392","proposer_priority":"-715356796"},{"address":"2ab301f354a74f45afb347b2c00cdceb09cc719d","pub_key":{"type":"tendermint/PubKeyEd25519","value":"qFSnvrYxWgRHPWz424XRZOiClZXjHoDQBv1CMQDS7/o="},"voting_power":"12006092","proposer_priority":"179312998"},{"address":"9215b6b8ec64cb9e7b461c3945e6ce00da86f8f2","pub_key":{"type":"tendermint/PubKeyEd25519","value":"Qng5pYzBWTouQK1EnLQxVfftKSo8DlOYydIuo8Am9ZA="},"voting_power":"12005923","proposer_priority":"108508511"},{"address":"a9c7a884bd7d6e799ef5311bf9168e0105bce416","pub_key":{"type":"tendermint/PubKeyEd25519","value":"FdztpZioZVj+YQ0d5JCHZWBehgSFuWrjG8DKmRgwNRE="},"voting_power":"12004941","proposer_priority":"-407832777"},{"address":"f1961eee75e5a1c8a11a5bf72fb368b990bdd289","pub_key":{"type":"tendermint/PubKeyEd25519","value":"JrRT1aj11c5mrpfsh//PeKA+XCR6wlaDgAsB+fegNAs="},"voting_power":"12004031","proposer_priority":"572375153"},{"address":"9fee83fbf0bd5a4d71519efc16d0d438c203dd5a","pub_key":{"type":"tendermint/PubKeyEd25519","value":"sIJcUpa8J1/4qoXaZ0tbYNEkjSW25AJ31LsuiEYSivU="},"voting_power":"12002406","proposer_priority":"-975439060"},{"address":"b44fc7d70708095fbc30af164410749483f96439","pub_key":{"type":"tendermint/PubKeyEd25519","value":"6nmVExrET0kDePGFZDjbNirkLOw0SKuYgHsvn+w32DY="},"voting_power":"12001462","proposer_priority":"-10450435"},{"address":"c4ef42d72d69f4b58ffe2a58db1335269842ac43","pub_key":{"type":"tendermint/PubKeyEd25519","value":"OwyWdapIzAWvKBByvLv2a5gl6yD6gK67eFzqf/SrlZE="},"voting_power":"12001234","proposer_priority":"-355385661"},{"address":"60c51d643048d0a426171bd0deabd1689a89690f","pub_key":{"type":"tendermint/PubKeyEd25519","value":"UlAJPv+2h+ChlG5uYdajVs3Eq+vp5/1Y83VoRk7tR7s="},"voting_power":"12001194","proposer_priority":"561514304"},{"address":"5d673308ca2b45e5222303836d833e0f70b1f9b0","pub_key":{"type":"tendermint/PubKeyEd25519","value":"9wI5MjjfeZnz5otZED8KVlqXREJovA8DxYrBWIj7AEQ="},"voting_power":"12000513","proposer_priority":"-1031601680"},{"address":"06be03f1b334fc3f7947d0d031f7dfa568005bd4","pub_key":{"type":"tendermint/PubKeyEd25519","value":"RneQYjxZQGtzfYRO38NExiVtgjdagu1cirb3XQKLD4c="},"voting_power":"12000393","proposer_priority":"-641978023"},{"address":"d7076d16c2e91e0eb16753575ac3cee3057781c0","pub_key":{"type":"tendermint/PubKeyEd25519","value":"pwHw+O9X178B8Tmak4BelOCz0VRb2FkGHZzWSrS/ozs="},"voting_power":"12000372","proposer_priority":"-463708698"},{"address":"913834357f92e577cdc02b1e682c65e86d016ae5","pub_key":{"type":"tendermint/PubKeyEd25519","value":"GT1pq3LZ8xz8gNdWed8d2AJIWAeQ+qhxHBCOld24k0U="},"voting_power":"12000205","proposer_priority":"-922653667"},{"address":"3ce85b466ff74142307b4b03ddacf17a52d46373","pub_key":{"type":"tendermint/PubKeyEd25519","value":"OPbuQy+x6/viqrL7Pc21FLRNquvAUeegJSop3m9yEhc="},"voting_power":"12000185","proposer_priority":"-528829835"},{"address":"dc0802ad3262ecd784ce339d87073c836e31c69e","pub_key":{"type":"tendermint/PubKeyEd25519","value":"3md1MTXk0zmxbKsPNAi1xpUvt5LJu4yUjGDtXaD7dKk="},"voting_power":"12000030","proposer_priority":"-423673633"},{"address":"31e623ab0ff5dc408c052072f96df3d4506718ce","pub_key":{"type":"tendermint/PubKeyEd25519","value":"KZACGMrjgoapUYJoZnXLubvTj7Lyvi8jPs2FZCDrdgU="},"voting_power":"12000008","proposer_priority":"907536606"},{"address":"02d49d26c87ed7dc1e38d7bfe5b7fcdf5a3a3164","pub_key":{"type":"tendermint/PubKeyEd25519","value":"s4FTOQAIHzWzpl4G/wsgj5cYgJDZAmcAimOud5VcMcQ="},"voting_power":"12000006","proposer_priority":"-830096980"},{"address":"646d5c6384c84c2e660bf268d0a4ebc1ffbceea6","pub_key":{"type":"tendermint/PubKeyEd25519","value":"fN+f+HYif75SNXJTVJrQGOR19797ur001Hz/VUs3QgQ="},"voting_power":"12000005","proposer_priority":"212069512"},{"address":"e6f8df77989972328c927e5493db6908936a4ea0","pub_key":{"type":"tendermint/PubKeyEd25519","value":"YZet9tH+pLca8oDQjPBqJdvCoSqfd+I7AAk4cwR4Lek="},"voting_power":"7826655","proposer_priority":"68477708"},{"address":"0aabc33bff6e288ea151463ca133e38051bada85","pub_key":{"type":"tendermint/PubKeyEd25519","value":"8lf54CEx2NRPvLKy/wxlB8+XAGac4MHbHzKXHWwgHgM="},"voting_power":"2458471","proposer_priority":"869578776"},{"address":"e71b23c378e4c465bfe96ad48e7f93f237e939dc","pub_key":{"type":"tendermint/PubKeyEd25519","value":"AATtUDupHksDYD1M0De5GdvoFcULQDol0XWSZoOt3lQ="},"voting_power":"2119422","proposer_priority":"-63056311"},{"address":"bfee4eba8e3307bcb074ad575ab9fa4524af2a25","pub_key":{"type":"tendermint/PubKeyEd25519","value":"7dsEUHdM6F+9oLMSl3xF2hRe2FqRcLOI9HNT43tDbR0="},"voting_power":"1629711","proposer_priority":"705388372"},{"address":"fbe1e3ed324fa17379403acfc3dd28fa882c6e65","pub_key":{"type":"tendermint/PubKeyEd25519","value":"gPUH+r4GyzzyALi19pL5YiHdbZxwVNFNq/aeqQMPp4E="},"voting_power":"1502555","proposer_priority":"-948513637"},{"address":"f8260be47a1c7d42bbd9a656bdf8e3d3f28cbab2","pub_key":{"type":"tendermint/PubKeyEd25519","value":"tPfAAuLM1NQUvi6onz2bpiEFQ0nS0G+HH+tiTUD74GQ="},"voting_power":"1057984","proposer_priority":"540498731"},{"address":"7c4b90fe7584bad71c5c6ab0877b8c39cd289187","pub_key":{"type":"tendermint/PubKeyEd25519","value":"rUPhoUYxrAZ8nNVmDzYBG8hKeq4DgFubS9csT5HPU/s="},"voting_power":"898826","proposer_priority":"-865154168"},{"address":"3b2a7b69569e7835324f35f7f2264d008e850c12","pub_key":{"type":"tendermint/PubKeyEd25519","value":"OPrm7Vg+qi+G712UapKeeDBUbEeNbgHSRweHrI596h8="},"voting_power":"650165","proposer_priority":"845587316"},{"address":"edd5716db0738cb0eecd0a04aeb186c4a46720e9","pub_key":{"type":"tendermint/PubKeyEd25519","value":"crYIcbR36Gb3cv2i6j6iuDozHSmg5dYW/e7dOt0xRgw="},"voting_power":"398850","proposer_priority":"-472797969"},{"address":"f868d1eb91e32d7f21ef78462feac2892d09eb56","pub_key":{"type":"tendermint/PubKeyEd25519","value":"t6m29TlSMiMcl6l+KBJ11lX67QK2tC8P5LsF0dbWafk="},"voting_power":"217502","proposer_priority":"424629917"},{"address":"59c09cd227541c5ee7d911084ada87352c899851","pub_key":{"type":"tendermint/PubKeyEd25519","value":"qCUSWl0pwiYSJwbknU4Fc6sZlFJINRCICBtz5xFYo8E="},"voting_power":"191753","proposer_priority":"213793082"},{"address":"478a66d6d2fcb936ba235daeb5a35ba454c048a7","pub_key":{"type":"tendermint/PubKeyEd25519","value":"S5aSb0vlow3s2cR9n/PfrvEQpFdmnmFM4KaTVu2eO9M="},"voting_power":"176183","proposer_priority":"-305060237"},{"address":"15839d4135d8c8fa603f533ea3ba80a687fe527a","pub_key":{"type":"tendermint/PubKeyEd25519","value":"eXyi7xIpmOFoZ3LyW89SRog5/JvWOLGldYVCGOTeqOY="},"voting_power":"175897","proposer_priority":"-724889734"},{"address":"13247270303a9d66ea8ffa02614a8b5369adda53","pub_key":{"type":"tendermint/PubKeyEd25519","value":"uE6n04V4cBXf9pOiIET69qDKtIrxqHdJXqyrHWt4WXE="},"voting_power":"174527","proposer_priority":"-851513088"},{"address":"409d07de01ea2a63c779c0246562aa278a95611e","pub_key":{"type":"tendermint/PubKeyEd25519","value":"6QAY+9wbHL6x2JRX67y8+O6iYVaD40AaAL3s6w0WyE4="},"voting_power":"158846","proposer_priority":"378725149"},{"address":"4e30f6a73766baa5fe6db7181b7fd08329c103e0","pub_key":{"type":"tendermint/PubKeyEd25519","value":"qE97rQR9bxg3PoQQNzrW7jjiDfhURt212Ljh4NthQsE="},"voting_power":"129730","proposer_priority":"295524801"},{"address":"f70543a091a45a8c0de04e5ce9e40d9c16d980e1","pub_key":{"type":"tendermint/PubKeyEd25519","value":"rkXcWaND0QwZ02iCoTek8THzVbBU2ck0oVi0VuhK9E8="},"voting_power":"119288","proposer_priority":"-571855264"},{"address":"e9b8e658e2f1aea81d57b78c8bdd2aa93a1bfc01","pub_key":{"type":"tendermint/PubKeyEd25519","value":"lMfcJzktWf2G2VKv2EkAnKlhIO21BVN1TUhlDUxhlXg="},"voting_power":"119093","proposer_priority":"708279300"},{"address":"3daf658a7b8dc1a5186f7c1547ab96ca35686575","pub_key":{"type":"tendermint/PubKeyEd25519","value":"yL6jB1QoQGd4jgWnLZEg+YT2gsp2FiLglItPbC4UCuo="},"voting_power":"116590","proposer_priority":"-630818070"},{"address":"02c375d9b259cdbb7b8a97d71b2a0a0dbed890e0","pub_key":{"type":"tendermint/PubKeyEd25519","value":"W+0vG8CMeptG2Jc6rgBrVJAd0Ap0slNLKOmH2VDzA0E="},"voting_power":"107749","proposer_priority":"-85553247"},{"address":"556f693ffba4b4c6de7af822852bd6e8a1a1d410","pub_key":{"type":"tendermint/PubKeyEd25519","value":"kQRSDLmt8qpSDpPuxGQj0QDVToWhXF0uHKtnoZgoDx4="},"voting_power":"107326","proposer_priority":"-740310104"},{"address":"b31b8ebf6857480b1281e756f0fd67243ac7a202","pub_key":{"type":"tendermint/PubKeyEd25519","value":"7jdjjvLZhZLmlwwEceQu/9PU3/D4lLmeYqg7ohiQrvM="},"voting_power":"106665","proposer_priority":"403607355"},{"address":"8dabad1a71629e4938de8c44d0bd2f2ce1eabd08","pub_key":{"type":"tendermint/PubKeyEd25519","value":"SySq3SuLFsjmhqTOxtNQwBuHd/xlWBipkHCK/nvHcTU="},"voting_power":"106170","proposer_priority":"-1090600009"},{"address":"615e9b48cd48fda3a754f28e3f42f77c2e49b0d7","pub_key":{"type":"tendermint/PubKeyEd25519","value":"xGdHKqlekTKkSPiItkpINJl+CUc4TF0oQ3nh9KYE+vI="},"voting_power":"104964","proposer_priority":"246727650"},{"address":"43965a09c5be34d027a599bd6f450f5e6a13ddf7","pub_key":{"type":"tendermint/PubKeyEd25519","value":"6g6hfqPKqus5RBJ5nDfS5ZxHCNZQEl9KuCf+hbFAoJU="},"voting_power":"96973","proposer_priority":"386499967"},{"address":"25764380a6df1c9cad29a842a049f690ab0f08bc","pub_key":{"type":"tendermint/PubKeyEd25519","value":"sRm+BMdyAFN5ifMXlKUn7VtpkGTqCSNkA9/wDQPy+S8="},"voting_power":"63683","proposer_priority":"745966039"}],"proposer":{"address":"5d673308ca2b45e5222303836d833e0f70b1f9b0","pub_key":{"type":"tendermint/PubKeyEd25519","value":"9wI5MjjfeZnz5otZED8KVlqXREJovA8DxYrBWIj7AEQ="},"voting_power":"12000513","proposer_priority":"-1031601680"},"total_voting_power":2039442572}}"#).unwrap();

        let timestamp = Timestamp::from_nanos(header.signed_header.header.time.as_unix_nanos());

        let state_update = verify_header(
            client_state.clone(),
            consensus_state,
            header.clone(),
            timestamp,
            Ed25519Verifier::new(mock_dependencies().as_ref()),
        )
        .unwrap();

        client_state.latest_height =
            Height::new_with_revision(1, header.signed_header.header.height.inner() as u64);

        assert_eq!(state_update.client_state.unwrap(), client_state);
        assert_eq!(
            state_update.consensus_state,
            ConsensusState {
                timestamp: header.signed_header.header.time,
                root: MerkleRoot {
                    hash: header.signed_header.header.app_hash.into_encoding()
                },
                next_validators_hash: header.signed_header.header.next_validators_hash
            }
        );
    }
}
