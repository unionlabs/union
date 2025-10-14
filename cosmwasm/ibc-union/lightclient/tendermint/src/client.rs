use cometbft_types::{
    crypto::public_key::PublicKey,
    types::{commit::Commit, signed_header::SignedHeader, validator_set::ValidatorSet},
};
use cosmwasm_std::{Addr, Empty};
use ibc_union_light_client::{
    ClientCreationResult, IbcClient, IbcClientCtx, IbcClientError, StateUpdate, spec::Status,
};
use ibc_union_msg::lightclient::Status;
use ibc_union_spec::{Timestamp, path::IBC_UNION_COSMWASM_COMMITMENT_PREFIX};
use ics23::ibc_api::SDK_SPECS;
use tendermint_light_client_types::{ClientState, ConsensusState, Header};
use tendermint_verifier::types::{HostFns, SignatureVerifier};
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
                &SignatureVerifier::new(Ed25519Verifier::new(ctx.deps)),
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

pub fn verify_header<V: HostFns>(
    mut client_state: ClientState,
    consensus_state: ConsensusState,
    mut header: Header,
    block_timestamp: cosmwasm_std::Timestamp,
    signature_verifier: &SignatureVerifier<V>,
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
        signature_verifier,
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
    use cosmwasm_std::{Deps, Timestamp, testing::mock_dependencies};
    use unionlabs::encoding::DecodeAs;

    use super::*;

    struct EdVerifier<'a> {
        deps: Deps<'a>,
    }

    impl<'a> HostFns for EdVerifier<'a> {
        fn verify_signature(&self, pubkey: &PublicKey, msg: &[u8], sig: &[u8]) -> bool {
            let PublicKey::Ed25519(pubkey) = pubkey else {
                panic!("invalid pubkey");
            };

            self.deps.api.ed25519_verify(msg, sig, &pubkey).unwrap()
        }

        fn verify_batch_signature(
            &self,
            _pubkeys: &[PublicKey],
            _msgs: &[&[u8]],
            _sigs: &[&[u8]],
        ) -> bool {
            true
        }
    }

    #[test]
    fn update_works() {
        let mut client_state: ClientState= serde_json::from_str(r#"{"chain_id":"bbn-1","contract_address":"0xbcf923a74d8b8914e0235d28c6b59e62b547af5ce366c6aafcb006bce7bb3ba4","frozen_height":null,"latest_height":"1-81661","max_clock_drift":"600s","proof_specs":[{"inner_spec":{"child_order":[0,1],"child_size":33,"empty_child":"0x","hash":"sha256","max_prefix_length":12,"min_prefix_length":4},"leaf_spec":{"hash":"sha256","length":"var_proto","prefix":"0x00","prehash_key":"no_hash","prehash_value":"sha256"},"max_depth":null,"min_depth":null,"prehash_key_before_comparison":false},{"inner_spec":{"child_order":[0,1],"child_size":32,"empty_child":"0x","hash":"sha256","max_prefix_length":1,"min_prefix_length":1},"leaf_spec":{"hash":"sha256","length":"var_proto","prefix":"0x00","prehash_key":"no_hash","prehash_value":"sha256"},"max_depth":null,"min_depth":null,"prehash_key_before_comparison":false}],"trust_level":{"denominator":3,"numerator":1},"trusting_period":"153000s","unbonding_period":"180000s","upgrade_path":["upgrade","upgradedIBCState"]}"#).unwrap();
        let consensus_state: ConsensusState = serde_json::from_str(
            r#"{"next_validators_hash":"94a730fd0e0a74e6e257d7748ff05ac1c2d893deb1649413566e08c4f6d6d8a1","root":{"hash":"RdTx8lb7Vkhy8bPDIn4uDCHW1qdoIO2WolWqk5LZyFQ="},"timestamp":"2025-04-17T17:45:37.818582138Z"}"#,
        ).unwrap();
        let header= Header::decode_as::<Bincode>(hex_literal::hex!("0b000000000000000000000000000000050000000000000062626e2d31183f0100000000004c3f016800000000442d77390116a684964f8de7af84a595843dc2363c1f011da1696ac090f5e94bf5ab75185c0100000001f6897c9f0f02cec01c705db55614d90fb8393240ae2080d9419841628b085b17819abaf0566a9cfe93db309a1a707a3219e727c51267c9bfd1e721638da1ae1e7beebece5e18e53a696424fc7495bec4daae68de9a641c63c593dee6e3c65ae894a730fd0e0a74e6e257d7748ff05ac1c2d893deb1649413566e08c4f6d6d8a194a730fd0e0a74e6e257d7748ff05ac1c2d893deb1649413566e08c4f6d6d8a122e3fa2d1695ae7db62e55677bf0c914b1ec88d64cd8d280cf2e29b2e06d0965c38fde7880a77cf99320bbcf3a1183e3c8793b5f4e05a50fdfbee789ab691bed4296233d1e5a978b174526e13513d59d6cd8225f1685bf4a9615f31ddf337a0ce3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855da37d2935eae0a92de55e86db2e887b4f18575f0183f01000000000000000000011353cbd7c13acd37c8aea26ad6b8905af433b152ec3abd7fec38bc8a45ecde9c010000000101bef4303b2f397e5416f0b6781c57dd5f29a88e5fed285c96be74a8673317c06400000000000000010000003c70d8336dee739bdf2d1c5a0fca0a88b87b9a6d573f01680000000098cbc0184000000000000000cf13e629b185ca7b11ad7ffa8e392711f9d3845c9f131a2e5a3cdefff8c7c5201e8aa804bcbf763c87928c50568cca989aa33de1ee726d238a7c2e97f8affa030100000033ecd5899a91105bb4e51066eb138189bc3825ad573f016800000000167a82194000000000000000b792a9cfb7772dbd943b3fa407b4f60d10d6dc7a61e7fd855222926a8e64ba41a9aea4734609f5fc4a606d27921f891f44a8322ff580454fce8790bf7867f60b01000000f2e6bb25d8d269c9737bd3789b11dc76c4fc56e7573f016800000000a884501840000000000000008bd977bef9b0156ec303cfb057d81da70ed6b6ab914334a286a5f83a8a5fde86d547cb07b8ec53cb2d38bc9b2f7fa48b75b5ad59ce8a7b29653a71155694f10f01000000c27b627c2fd11ee1e3e850738720692be2e3f4c6573f016800000000a1009116400000000000000076eb2fe8374f059ea2460e7000bb4664b66ef246f3b6555a06ff365862f0221fb7842a2caf74b82916376edfd1b32150125bf1b7043acfd43f9895b38ea5240e01000000bb386aa0e7b37eb33516aebb58cae3574af85d1f573f016800000000bd7d301840000000000000004d6da366025c3ad7f51eea0347b6a38041b6e6ad56c7d6d01eb4934ea85cc1fbc79f7e5e890d41906bcd04f89b280bccf6ab9c561fd88e0e03e63fcb11b6ba0e010000002a3d40c3e7ce8d6b55077335cab141c2e11ac254573f016800000000377c081c40000000000000007595f86269754a05c7596d9db6701e7a25d302052672e3cb4492911b2f64a1d0bff755abce48360a66e398fb40319c082ad282e8607ffd7e8ea2e74d601d130d0100000044c395a4a96c6d1a450ed33b5a8ddb359cefed36573f0168000000006e63a8164000000000000000a9dd35fec6edb7d3631c43ccbd5f10cfd230e5223b7bbe3853e73490997e4244543978be2c4889336522692dd374713963096eaa587f7ab848f30a8b81bdeb040100000043804074334ac6111233037efe20105c516b0e9f573f016800000000aa36fa1a4000000000000000e3618d8d43d07c46b9350727b1fc6f677e110da47cb185471445a1a679884fac855f1d33fb7e95e9066983372968a0eb805e2e32c824842d8160b7cc78d87a0601000000ec033669fe506006f870bd8b12cb524a3abc3693573f01680000000030b2fc194000000000000000c1de4d43e20d16fb9498ca260819815382231ba2ff2c6a762b5f6b5c20267d7e737c7919307cf11429b569656130c091f8d76342415e53fee4ae5c5d7f68c90d0100000064cdacb04fb9c3b290579644f9edbd05cb00bb7d573f01680000000059dd551740000000000000007f21b1225d146054f9ddc879e6a0f49ac776364b76efa87919d4d043309aa41ee6af40ea64a12bcad01c8886e8ea1028f040a89c7dee432d748134f3e5b0da0a010000005295664886565f43a3074140551a755872bf6ffd573f016800000000cd17061a4000000000000000c43e2288134dbada3c8a2d1eaca5a6f2e7baaf08da2d57718dcd3bf2190c4c8d675d2e8e9aba78b8375046d91c4bd2bce4abf974619abacc2f6932599a44f10e01000000da37d2935eae0a92de55e86db2e887b4f18575f0573f01680000000006b130174000000000000000eb51fad0a1c36966e838f509faed65d4196cb5e66f94eae890b2f0a973b01a29f843ae5a438d49afabad9f9eb581ad59271ac83e20257aad50d338a17907f60701000000718160e3538e0b0ed83d3b3dc4ece83809fcd107573f016800000000289d9b184000000000000000f2fdd679983ec9e1b334fc6267356180b4023ea72e4f848d88b03bd80a284249acddb39ec3b60f2ad6e44a9e342ce83a30a668286a4f0dbb68a45c93355b910f01000000b03f0cb997f3428ef3968833bca2f2a4f613a8eb573f016800000000199933184000000000000000a3d8c0cb0a2753ea2285b50462a3d87609aad0a1d524d8066fdb56cbb89250d653d8d9e124522e94c8e44ab8ddd1be01ed736bb5f5a8295f48d3fafcb19e110d010000003965c223e13ac66f8175c86e657d396c711964b7573f016800000000534c451a4000000000000000e1e1680f15134f3433061c430cb52e81693426f46c682a605e6c60f619dc791c49dcd83b1415597ead87de454b68ddd1d03237d7d867b31729638e5f83d96602010000005c78a0762f13b0b2a793e82d1a178eb45642234d573f016800000000a329f3174000000000000000e1d69e4a4ecf1560d0883ab060e3e6bd237fec0e2baac0cf7e4ca0e733281ede1d9c2e2633bf71b4b678ab3f6d80a7ace952d0c6a399e61a39214237f1a8a80f01000000410c1ebaf54c090f0a9132dfa9f79e0f480b2c06573f016800000000491b821a400000000000000076a91ca4104f15ef1b4ce336431c7f14a67c28f54181a1204bbc9f3152bdb4574bac14cb6446096c3567a888e9eebfdb66a8dca9cab6a13e44d797b7579db40b01000000b41ef50272a7a2398ba617d3c963986ebebc5b2d573f016800000000c7160d2340000000000000007428ed4492852740930668bee43e3b7621618b0c4efad4b193aafaae0d099473d1b50ee8c09f8481b360bb4627d809b48eff69c1c04847d130eb0a835aa04c04010000006cc0a1531ff58d61dc9061c5b89bc0efc07911fc573f0168000000001688be164000000000000000efc575767b1a60b962e8cb62116e75437a8ca699bd724f07f7f7550e84c446ee75519629e8ad9905b54227a256da30592ec29df844a709d0b2f2dffa6f8486050100000053406de83381dd804632447f1baed00cf28048e0573f0168000000003be0161740000000000000003f46955d38017871c21a32c4c79a7c86ea4d914b90a113ef4686b340ce143d41eeaba82203fb7d7e9b16acf9fb3fb46c14df9f9e1fa88c2700da1356f829a4050100000070dac78498afc6d766d860c433265a274d48b7d8573f0168000000006ef1021c400000000000000076d3feeb28257b34c159e8cfb7b9323b81ee3e512e8a6940401f1ff03fefaccb9ad4a8aff96063bcc8a5c53a6b988da45b2faa98badde49d62783e2cdc497e0101000000521d2a344e322b8ba76401c833a1ec74ea2bd075573f016800000000ed6a7f15400000000000000002321f7f42384d38514aa662854a31b4601498e7b0273e36dc08ffb163d02c5bad89fe76768dace9eec516df01db0d1b8f37d49444a167eb50ba942fd384e00101000000671d816776e8c7fae18dc122fb6dc231956c6d48573f016800000000ffcb7a1a4000000000000000a2226bc35ebd304d0f07697ba3fad587110c68074649cb58945e1955c462b42f0efe79b35932dbcd6c3bba8c7eefcbf2ca9e0baa0875461da4cad1d81244de0601000000e2fc4df9e7870fc8238136faca07a120cc078a0c573f01680000000063f9f71940000000000000002311e66d55510ccb587c3dc4cbae9171127526849770b37b0e4d7a4eabaddea5f7a59b3721ee2828c4cdab74a3b61045c19049e65071c5405b7add577a112100010000001b03ab3c744545b281d224127708a9ec7487e58c573f01680000000027d7ae174000000000000000c6bc2e923b2f7d2413c44f138b90383b40f88dbe417db887a5b5298fb69552965e0688812744976f88c0d1d908bc3c25a1d34ffa471f72a1f721945bbe7acc0c010000002a1b3fee6b29ea1828d427f0e5c3887cb7e64487573f01680000000001c0e3164000000000000000ff70b631b42f1c197d38d1de36072b48eb59a22cd478fb203cbabce22232dd1d72f30a7b771dbf48fa00119ac84431168202f0d089e8304f914380d12ed806030100000035f6c0359084c37ef6eb4061b694386910093b99573f016800000000e6f5601f4000000000000000fbf4f2034b087d25d3a948368e535a079cbb5fa7a2bea8c08b0c02f306c771d519c99bcbc2dbe4369d5e00bd26efe4f1783370208397c5507ddf7904c4a9b50401000000188f984567f5c6426386127b6fba27cb59684dc2573f0168000000005283a41640000000000000009c284a4bca332babc8b43bdf7b8ea36e921f616f456f0deb02dc5c58ce9e0bffbebf926f3293b4aae28d9c585ac8ed0833b5888264bb2103c474e06fb2489a0c01000000ccb2909dcd316fc5e738dcdb1f14360f04b44320573f016800000000c794e11a4000000000000000d32629d8c9786c4fd576366a9fbac62976ef544b025be2e6c200547efdc02ea7855e7384fdd6c02f62e27fd3c0c35c946c3dc442f4374973dcec5074afdee102010000001a4b644c16a4a1904618f8d3c0f6a67223038390573f0168000000006f840c2040000000000000007c841924f145b9b542170426525db566ce4f858b1fe475fe23cb175a77c1327ffa2341fb899ce7fa07731418859360993b852158b9968327f7f1530aa39ac1080100000077be23139b650d69ede7985b31fd7b1cee9ca7d3573f016800000000b708c81940000000000000001dde2b887bd4955ce932704e14afea5609dee338b61bdaaf98c5ce02e3138bb9ffe4b99723993f51e727a04314e092b6d769d5618747278ba41d34980681de0101000000c268fe001a533b6f0860167381384363a790fd0d573f016800000000d5415a1a40000000000000002e0e79c9238b70d4bfad7b6449346b25e67ba824db6cfaffc1d073c4c6531ab676431b7947e4b14c79877a038e8bcd0925e2a39612a03fd9e615213cefdee301010000007ea986d8723587f5e653c0df785fcfce39238785573f016800000000c7022b154000000000000000581f76733c330b65b9eb29226371ee2933a634877b84212f09895bc603c9b1100fec298cb707ba000ff2ef2b8b3159d0c9dda1613a33b10a62fe18da5a31240201000000fbc2a3f682b507f1a3a5d4c44c8cbb6af0860296573f016800000000b1275e1a4000000000000000925b5b1ff0ca0dadaf470386279f40929e5923298e46fd647670641b671378cd84f9173c7e16f9216ae5da88b27763c676a2c7580c71d2d7f40cbef6af19ec0801000000616a45478336b92583cfe29e8ee942802de69eae573f0168000000005925501640000000000000006f6d27f04e4ceee920d8328fc63b2cb7cdfaaee6c3939b9fa93eecf1a261ab9484c5f27e741b2295a78415c6bcd9a52c865ebe3dfa1e456d0f1186d3a24adf0001000000f42c7e49df4f4d0993237959518415e473792392573f016800000000921740174000000000000000295f8fb8f209cbed80996daf14b861fbde53ebd111452cd74d336e79367506c028077d3800441468ffc5b1ffe113e0672c9b0ebdd3b0f6f0064d5abb9cbb140e010000009407fa25941c932473f8e2b1ecc5481e3ae15a71573f016800000000896fcd184000000000000000670ca3635ab26245b242d9a9176a908466b2b58ff6b3d47652a13d07415c83726dd80866bacb7185e189faaac67d14e31550f767fbf7cdd9f5a1d778420026000000000001000000aea26001de9b15d4dad32808712864e61ae52a17573f016800000000504c84224000000000000000fd8bdd040e37b926a00274baa3f5871a4ff465e9947d2c7c5ccd53357bcbb302b523dbae29714dee0aceb2935fcc9cbd91ea70d61b0c621c26084c3e2a75da0b010000002888677e39e38bf4ee64081bd43776fb4d507fcb573f0168000000006185ec1d40000000000000007fcc84bd170c87d7e66a8829afab57cfc36508a27f440e3b5b280eff2fb3f1d85fe472710d4e100d34bcc8a8f97e2db843d4e7324b4376eaa126f30e95e5db040100000076f6dd3f07bf2e9ff5b0a188bc08d5a4efcf40d9573f0168000000001926f4164000000000000000c524dee40ccb01743bb31a2f8e5b4b7a41514511ed0b556ad4ccaa2753229ccb1014fc624cb23f72b626e9d42c1362f5ccb08c5f8dac81820094232f543795000100000023df4a8418762cea4d24b7227ad125df76d2baf6573f0168000000007c076417400000000000000050320803e021ee5321dbb80996f5c9751e96a3c65a9ae6bc8cccf2d94c829c834aca5de3bf7fcef8a0df783db312cbc9f68c2ad70a0c75f0de3e39955241b00e01000000a9c7a884bd7d6e799ef5311bf9168e0105bce416573f016800000000f7a4dc1640000000000000000cbd3ffb783b6f50d4bedcc22d17b9ade47682a45e6da7f32f8beb523e510a4c59ddcf3fc72402f20396b7ff76490c2ad91c47b968b4c8b09269e2a0f3b0e20f01000000b8f9204a348e0dbc85abe5bc98936d6b84c01e45573f01680000000026f40a1a400000000000000005cf7f8bb1703025f7a816431f6a54b81c215564a724db1dc423b465c6867f395808f96394eeb34a1da26782ecf42dab927d5216bb382afbd49846242387c50001000000a8f38fcb39b580c4b856dcaed882d504a216ee00573f016800000000c0e1e1164000000000000000fb850fce9a8572258236f6e68dd0b4be5c8d20dd7a2b0c213da9acefa29c4637875e74b02c16ce995fbd1d9d1316e46312c77404840360a587935e724a725102010000009fee83fbf0bd5a4d71519efc16d0d438c203dd5a573f01680000000021434c174000000000000000c17cbc28592932786f18b317d8d25ce910bddd7be24ffd85683c33210c190530a1cdfe4bcd5ce52d598ed178fd7faaee85701d2ffa877bd718af3fbdc9a38b0801000000b44fc7d70708095fbc30af164410749483f96439573f016800000000a5d4182140000000000000003374cd4832b145a0315659ddfef7aa2f225afd0a67e4be0dd0c169d0a709df4e79345038c27bc37f4443cb301f475c41e2b5e85a46b566b86c0ebb0f815c550d010000002ab301f354a74f45afb347b2c00cdceb09cc719d573f016800000000660dec144000000000000000f922633361ced5c58b0c5e7d0f381d68b9f35819dd13a1d42a51d2999176d79f93f7984571d276763d7fdd1064c0bd3be1e853ee1536b162a373cb08e63ee100010000002972b0df619f73d1a5c8933aff22ad8c9258f370573f01680000000050bbf81b4000000000000000fcc4daf52c6805cf2be30e6cc040782ac61d42e3558cbae7768280477ab54c16be162266de194cf6d91b21a651be6ee52481b2e094c54226d1fb0c97743ad50301000000dc0802ad3262ecd784ce339d87073c836e31c69e573f01680000000091abcf1a4000000000000000dad4c5be45bd0d790271c7fee31086a98c46ffa893a56751d459695d9b15f7fbefc636f8329dfb04541a92d4d037bb31b12f018605579a391eaa5cfc081901080100000006be03f1b334fc3f7947d0d031f7dfa568005bd4573f0168000000004c1c82194000000000000000f03d193eafcc60df21ac6fbfadfdc1b9fabf2d26329934b7f2881f233f853b613e499230ecd65b8a9001f3e75a20cc5fad006b33362549eea9c1b2b42df03d030100000060c51d643048d0a426171bd0deabd1689a89690f573f01680000000061864b1840000000000000002eb485ce6470a6965d6ed297c8bca36f0be0b725fe1fea4148533b6c9b772eaca632720d753e79c3703a8b67ad0b016776bb58c93ca63b7a2dfc1e50bf65dd0f010000009c4cc8543dc684948490681118957942b29a9200573f016800000000bb3ef516400000000000000037efc8026655a1cb1f251a0478c83c55ce6aa026594aa606afc9eeea3deef2fd52e7cba523497c176e745745dc0235e3327a450a046015de0a51839f5d74e409010000005d673308ca2b45e5222303836d833e0f70b1f9b0573f016800000000b5a9db1940000000000000006fd9ea012a76d26aabe17bd0af099e5f44599e8e761c6a5b50ab2c1ed5cd48cb687280437b528463d78fc266bf9be301f0a675fa6f69be6f4c9c42dda1bfc90a010000003ce85b466ff74142307b4b03ddacf17a52d46373573f0168000000001ba6e61940000000000000006c8cfb1005e5cd1acb175cee6d942b932e72bf27ece2806f0a63d874341997f2e0db916d90a139e7a73c39b63fe17e6d730fde5909a861c3e08b85a48dc6450401000000466d1236b8c77925b25219e4ffe34dd069351847573f016800000000e3f95a1b400000000000000063a3cdf267fdeeb5cc357a40577b8cc87ab521d2566a7f3415402b983e55761c59824b51f9faaf2121bed2e2ff34777dd3a8d36c26ae49ff5f3b21f1a115740001000000646d5c6384c84c2e660bf268d0a4ebc1ffbceea6573f016800000000aaa7bb1e400000000000000023a78b1adf912ac06e4d611238f402c76c8e0f48370147849ae4a0bd2f90308c311e2ae0f88c4a1731ad0f3c915515a865ecbe83cdc4bd56196a07f025bb65060100000002d49d26c87ed7dc1e38d7bfe5b7fcdf5a3a3164573f016800000000a4b8111d400000000000000028a0e8da8436914d1a7ce6571e6621936a81d18298bc683c40a99a4aaf6cf91f229a8973b9129c1e42fb4e6ffde81d5c90f2c89ab7d12144dc27dff20b31a10d01000000f1961eee75e5a1c8a11a5bf72fb368b990bdd289573f01680000000054c58220400000000000000090f1660fba823f2e8a51a81df74a043f93c48e7c88ec5866adb8000d39d67bc285c273588833c35c1b20161154513343b9fc3e52e392df4f3d51f485f4c48a01010000002c4e108c3f09cf9b8b1515d3b26d50e332d86e3b573f01680000000082d9961a4000000000000000c7b44d9e33bdedfb6e1d7794408e6d19019c09afd9e9aa9a24361fa3ec0785d77568521abdb552b694bda3228ff6cfa461e4acaf9d0e390699636ea8daccd40601000000913834357f92e577cdc02b1e682c65e86d016ae5573f0168000000006b8f6a1740000000000000008c1a6f236784d9750052fe3a4d5be8b7d9ed44f60a4f671aa84489717992df7afe6fe655420a72dc8c804403b8f3b4862095d13d6b463aab6fb6891b6aa4600301000000c4ef42d72d69f4b58ffe2a58db1335269842ac43573f0168000000003f61411c40000000000000009f05247709059bf29e7286f27e2aa7b5c243505f617f5fd4faafd17fe1616dff87d17e945f8352cf46e5a237821dd4bf143baaff336bdc0e36cea98866b7e90901000000d7076d16c2e91e0eb16753575ac3cee3057781c0573f0168000000000d1ad6174000000000000000099694ba03968a2a9ed12d8c7d4cdf3c13b02953f1cbbfaaf246b8fbc2c0d05ff09bdb5b2eff76c2e9506628b3518e30e137907099dfad313e966c51ee4c11020100000001b2c0f03ba9b2f352a39bc45ce37aa2710b5af4573f01680000000087330f214000000000000000f8845f6ddff720e08435e7b6fb868c6af78da6d06b7c6301e0fbb5e7f2a843ccdfa6b7404500b2ab1d6ca432c5ed2d6f9dbe0933d392e211b7a284bd9646a30401000000fbe1e3ed324fa17379403acfc3dd28fa882c6e65573f016800000000102734164000000000000000efe330ea3aafd08b0e6552ab3b17cee4af58810713fa750778f8b79ce74d1a015df2c7b68f7fc04e68af53be235ef98cb50f6f22ebdd508426bd59e9b3aef50d01000000edd5716db0738cb0eecd0a04aeb186c4a46720e9573f016800000000b14ac5174000000000000000f9bcad8e25178d091efc38f8851c06c397c7f63a1c815c5e5c2aa9c64ad9ce8cbad5561090747e6accf4e6531a484af06b47439fff788e512444341b26014a0f01000000e6f8df77989972328c927e5493db6908936a4ea0573f0168000000007d861f164000000000000000e451071d1b12130967baf1bdb5bd4b7c55b9f1a66729c9185a71ce0bb39d095447c79ea4da323e4d0f51cb278f9bfb529d7d14cb56ba3a7ada72e365c209430101000000f8260be47a1c7d42bbd9a656bdf8e3d3f28cbab2573f016800000000827ed3174000000000000000906e2be0fdf365ee882d531ee97792b965f145f76c53cb3bc3bffc4a8883f0b90d4f8112830b3dc03af1110fc0c3494dd170fe85c2f7dd4b314dbc014e0ad1090100000025764380a6df1c9cad29a842a049f690ab0f08bc573f016800000000a841ff1840000000000000006d39598d5dd7964c0a6116411e822c0216fe0dd32ab29fd5995defb3c2b564d6ab7632e53f6e3c8231bfec414db91e888e6bb781e9362cca42a4d7dfbceb450c0100000051b78fd2e37a2142b313f68a4edb31b23fe1afd1573f016800000000f3017f1a4000000000000000236c1d6640b600bf7c154a6dcc794e6ce80ca492867d6ed691246262075f716a24074e3850fd1d09347d06e943b4a94766715fdfabe859e435db7d9e651d0202010000008dabad1a71629e4938de8c44d0bd2f2ce1eabd08573f01680000000053c5871a400000000000000019e5075fe28c32c531396578169c53d6f1a60c910bdd635b7cdb93cea83ed58b008603da37219f3ec34c92c6d89a78d17a7ccc2330d72a3abb04575994bd3805010000004e30f6a73766baa5fe6db7181b7fd08329c103e0573f016800000000a8ea411a40000000000000004e5bb55fd148cdd4016de166bae1248ca53f61c534291994640526f8e189416f1515510e33f4deb6d9014a135e28c1a4370e28c4a86bc11245c6cacad50bb4040100000066439667823cf1cdefaf47d5ada806d462a6e05f573f016800000000ffa6731c400000000000000075bf7dff374543356be775e7976b0da9b5af9f3bb682b853965a6d54658ef31b3c58ab1a75fe046174dad9e2d4be0dc018659424134b934c8387e3d93be54c0901000000f70543a091a45a8c0de04e5ce9e40d9c16d980e1573f0168000000002080361c400000000000000068446c6f2a74a2400feb9bbadcec1d425698c47cce4f6a943cc6a4771a6032830bc652c5288029dbb72d9122537750ff591da0ce70c3d8cc2f6c85f722ce9a0a010000006967de8bfc42a68d7c5eac521dd0099cea1fe230573f016800000000f22c67194000000000000000c0ccd399558d985503c763bd434e20c318387a7e3954af5c6c1af7a5b006a6502cb667d84417eb17b5f2c6a22cb4b98f7102d163ad3e020d6e02e7a7744bb2020100000043965a09c5be34d027a599bd6f450f5e6a13ddf7573f016800000000aee2351a40000000000000008443238eeef4319be2bedddf227b6b0e1eea0ffdc15e21acc1cc39eeed2380be026469934b35ac7e53a2fd8aa9d5eaf0826a59c82f4ba7b168ac5de9b564d10701000000afa7739d36004030202c7ec8cda8ad979c1891b9573f01680000000092d8071840000000000000003f3fd89d20c289df972c7c07f1b7a5efead107c2477446302c7dd4bde592034c736d6807b55034566835d64f054c33b05f48fc19e0b47c556531c984b292050101000000437f4b77f9c87a4d4b0dba30bd742bcaf4db48c8573f01680000000061f10c1840000000000000005e0decd07fbb2a1445a39d0220d4154e255e2c8d330bd2eff92f5c001339c213b305b3200ea1a5287fa50f0b3e4cfbd7c0d2ac2538000db7dd60c4730bfe360501000000b860c8e1ad41462d6352406b85260c9713418ffa573f016800000000a21d6a1a40000000000000005b99243d1345e9f27177e4fd77bae9b9cbca9a2c66b1da6665ae14bd4c8d1aaf8ed1f9a636f61bb8abd536dc98e11b98e7878ffef2aa7e09fb7a1a4cfdd622050100000002c375d9b259cdbb7b8a97d71b2a0a0dbed890e0573f0168000000004dbe0e174000000000000000a0723261000b9b44b0b26c383f1b9606fbe4fa027764b59f6ea2992d2de040c79af1f414e5d1fa7bb954b957ce7ba7402843a72f96f71a01ffc3691a5565cb02010000000d1df149a8579909fedf2111513c57897118b09e573f01680000000000bafe1940000000000000002275f45ac763a505da4b4665523655b76954743bf0a85d70c6a4abe4a6cce63274effb715cf7904562d2b2e4ab25549cd39ac93f827570737164332475cf6208010000005ade83d0ec62ab3eb6b023ff54c976eaf32e8655573f0168000000003670cb184000000000000000aaa2057086857bf28f38ef05ebd77f71182ce0c94b7a00703d30ae7e6f7b23618ef0419039f507acc6c04dde0265cfcf4f0455445969c1538b925282a59e9f0301000000a93fccc20939a8a0fecc48a9737402486bc41d53573f016800000000a5b2ec1c4000000000000000ff1dc7596d16e2802ddd69b3326945c3e58f13523547c914624ab988b44be3d8392cf8267ff249b83cb5f6a0ceee3ccf1827c3164a2d8e3ba15fd47e1d6ca70a01000000c2f0ea5b136b23831fc601ff36ad43303702f3ec573f01680000000062b4b71a4000000000000000deb4c28d5408341f42e6978e5c060bee57b2d0a2ba22f9b8d8fe335cc300782096576f1df76207510db209e9c58fc77186132ebaa6702229be6eb1f8cfc3f60201000000bfffeb9cee54c0e110a1761f9a646e4589ee6fc0573f01680000000020f0a11b40000000000000007955bd06a4e2e0aa0c737d2fad04745cc84b92b61a0951050a1960a13d0ca15a2e9f36e0522e4d3ead6fb68c5ef1d08daadb42f2c6cf70bfc1904db1fbc7bc0401000000409d07de01ea2a63c779c0246562aa278a95611e573f016800000000150c9c1840000000000000006e179116ae174bb9c76ed9055e980b393750735a22c73c683f9658170526b98d4fa2cafd608d96322efc231bcf3a4c945b2574426f3238f8d6b23507633dde0e010000009215b6b8ec64cb9e7b461c3945e6ce00da86f8f2573f01680000000077ecf81d40000000000000009ec71509aeb0b9522a250706b1dbe95a531dc4187ed23968684ad0831cc022d5c5e13c91557a6609d2bbb2ad904eaf7b25f2ffbd4be73aee81eaf1e4dd8bbe0001000000cea4cde95b67ff91fd25ea77c3075944764754b6573f016800000000fb1bb01940000000000000002fdb082b435642e6bfcb1c0e84226661a8bdac3c6421fcf715d6f56949b85952e6fa658ae64d8425ba96efc5d326e219e4ee403ff92dd2eb28a15c6810cc560d0100000060131b8768de4f8a40aaa34a9f0d40be902807ae573f01680000000089ace41a4000000000000000362fd2c587d51322f39562204c014406495d5e2c2d00ff83b740205f8f141ad2067093c8e6fab7f7d5b84673069fab1bc63ee68554b84566d787edf6c3444e0a0100000013247270303a9d66ea8ffa02614a8b5369adda53573f016800000000dbd0b5194000000000000000ad271a9968ef57dcfda2257ebebacb431f54a89ada54d3896392f8f7b2e3da461c10ca75ceb2237d1b1b767152537db912a245b23167bb25304f316fd407010701000000349f27787689a635ffbe00e0dd66e848690a73f2573f016800000000e15a461d40000000000000005eab075124df313c8d7929cb445f8a89156d3046507dd89f1efc442e89148905cecc5a1152f852422b2467a235b3b2dbd3b35fbf5eeca14e45801b7a37d4150a01000000e71b23c378e4c465bfe96ad48e7f93f237e939dc573f016800000000a4ec9d1d4000000000000000fb4a82d17b035f5d5a8d303978ab53d6a3c856790ff0765dbf3169bc5867f7db58ccbe66c18bb73ad48dc3d595dd77ec50da6841cc5f6b9f42038474b38e010601000000c2989ed51e73b742e4991cf915d6339419657ad5573f016800000000e941eb1940000000000000007c8000bb9be19cc661b975ff9f4fb81a2e7d817ef42dc2f8aad8c3bf9178f4fd2a7315c0bde948386bfc5bdacd8febd56289e5053cc48866ed3fe67fbe1d6f090100000080f78edcdb53aa69dea3f482472027327f1acf0e573f016800000000f7839d184000000000000000f4f44c00a1571b07d98b31f64021a84b1a38456171945847fcbdc87c6a272cf27b8177dfa1c4490033b1da21b0bef08c77d2bde1f8bd98f0ba1bd8989ce771010100000069a300bbe8cba17536cdb4346c148d1dd7475c6d573f01680000000008ab9f184000000000000000ff559c8fa5ad7fc0fd7f0dffdd60af10bf8edc160c6585e9ff7831cc47ff2d32e93178b7fcd5d65fd1fc5ef64fbd0d17f9b31b7e957e5c16c59d0aa058de270001000000875a5ed5d1c185a47a9c2c96ec3491dea11d073b573f0168000000006b56f61f400000000000000094359578a67312b257060bb10bf62f0f963ef6de3f7cf7afeeebf22f52e3768e426cb4025fb958297de9e3e33c9bdeb26c4b4faae1254c539dbada4a1e227200010000001b21f9bf3b9a650607cced8c0c2fbeca3211791b573f0168000000006ce43a19400000000000000077ee04c6744995b77e1fd353b054fb1172b349fa22623ef5f1081ad5df3a411e02120f1d212fa69d96192c391cb567d6851307e7b6a772b8e802e64d7ad73e0c010000002be016839732d17fe16983d34f3b180873269443573f0168000000007d5e14314000000000000000c2c194a1c45e5bef78ce140d233ab9e570d9777e548298fe8c6aa0cf8b19ad2d81ed802f1ebf302ebc108e7827acb4dd38f3056af0bc2c7c2918b700fd76a10901000000300636fc6b27a7016f3aef03c231aa63163eeb5d573f0168000000000719831c4000000000000000051d846428edde706bebac0bc310662d1110dcf7bd567693dc52293070271e7c6562b1d41a374443b6c3f0b843882b733a0c473d71af79a62ce69c97286bb7060100000065c584027f5fd8349027fa5bf1d4ed3828c29058573f0168000000008439101b40000000000000003d34b73f34afa2c77e0cfcbd3bf2a1128e28b0910ee46a70432e78845ff1c5e6799d4f9bd33b9031d3d2b3d4252a3b39399c500ab9e3009e3ebd3492c5c64f0c64000000000000003c70d8336dee739bdf2d1c5a0fca0a88b87b9a6d000000002000000000000000860e2c9b980d61bf53e1d582f789c35bedf4a252c4093e9392ba7516d9d50511d2036103000000000d38401b0000000033ecd5899a91105bb4e51066eb138189bc3825ad00000000200000000000000075617111844fcc8c90a4fc121f2c37d19643667e868192690f2314bf14526ed405f206030000000067ceaa0d00000000f2e6bb25d8d269c9737bd3789b11dc76c4fc56e70000000020000000000000000106edcb594c1e457564711c5bc0ba0182bc42131fc5a02105da76c3e624a76f5f01030300000000b27cbf1400000000c27b627c2fd11ee1e3e850738720692be2e3f4c6000000002000000000000000bee1e39ec395c97158a2dd51695f96c7e724da138ebdbf7f417c17bb3796726eb779ff020000000025f50a1b00000000bb386aa0e7b37eb33516aebb58cae3574af85d1f0000000020000000000000007033e708beda4a88c330b82edb7286df02156fa71fa53152dc306298386d42bad90bfc020000000055339121000000002a3d40c3e7ce8d6b55077335cab141c2e11ac254000000002000000000000000914426099b4fe08deae4590a68f553bd79ebdd4303ec6c2586719d917aac6db8d393fb02000000004bbcbd150000000044c395a4a96c6d1a450ed33b5a8ddb359cefed36000000002000000000000000aba9508fec18359f9042ef73816a167d25dcc2740d1336f66bb1e7c96c8141fd2482fb02000000004bde13210000000043804074334ac6111233037efe20105c516b0e9f000000002000000000000000e07a4ba0104afc6cdd71e063a03e195c109e332bf5e163c37a80ad67506a62fdc21bfb0200000000830ad70b00000000ec033669fe506006f870bd8b12cb524a3abc3693000000002000000000000000003c8c0c3b27c3ddcd7414aa5d195175e58c36789c8ad9b58a4275e09db8d099f20afb02000000002477df1a0000000064cdacb04fb9c3b290579644f9edbd05cb00bb7d000000002000000000000000c9d641627d6eced574dfdcb08e243cad1c9c9d71dceb684ad74fb865a95a41564c02fb0200000000460b1708000000005295664886565f43a3074140551a755872bf6ffd0000000020000000000000006f7cb6f22bb847e20199220fc0903dd24eb2a9be6424f82f0d50a6727a5225c947fbfa02000000004272ee0a00000000da37d2935eae0a92de55e86db2e887b4f18575f0000000002000000000000000c7b8b7b6b6cc2457c271bf5201f65b2fcb363350f4387438a6218ff839a4274909f3fa0200000000a0868cd9ffffffff718160e3538e0b0ed83d3b3dc4ece83809fcd107000000002000000000000000bb1aed9ed9b0557ba3794a48f15243f4c8c6b9b176c69b589c71664f896496f9c7cd3b0100000000adff261000000000b03f0cb997f3428ef3968833bca2f2a4f613a8eb00000000200000000000000069f12605bf4bd86e48fda138649db87461446490efadd047f77a1ff23606bf6d5f28330100000000669240f6ffffffff3965c223e13ac66f8175c86e657d396c711964b7000000002000000000000000238dd24b92eb981a83414440555ca2b18c57dcb8be9982bb3fb4a28fafb26081e02232010000000032d2e6e8ffffffff5c78a0762f13b0b2a793e82d1a178eb45642234d000000002000000000000000b45c85f1255127cd73f4edaadb0cd9307522b76be00d68849efd9a2bf0920a8274ff310100000000de8830fbffffffff410c1ebaf54c090f0a9132dfa9f79e0f480b2c060000000020000000000000006fbfe2d80362615ec2b5ad355b08c122e3799894f5a8c81c13af60909e1b057fe4d23101000000009023f9e9ffffffffb41ef50272a7a2398ba617d3c963986ebebc5b2d0000000020000000000000001a9acf9916b1e429c5bc5b12f5fea61acb5fef345712f25d12806ebc4369fdc22dd231010000000041210ee1ffffffff6cc0a1531ff58d61dc9061c5b89bc0efc07911fc0000000020000000000000007eef62daf827c634f1036c88485a507519b226c7cbb79cf203e67b3628c32b7e49cf3101000000002cc483e9ffffffff53406de83381dd804632447f1baed00cf28048e000000000200000000000000051f5b4a16dc3ba302181af9de097f591a9546b9754b2b1cb86d2d980df4ceac08cc231010000000000d296e7ffffffff70dac78498afc6d766d860c433265a274d48b7d800000000200000000000000054393aab853ae377ef15a80742358da0a52a9ebea553734a068f444404b9903436c23101000000000d6f840200000000521d2a344e322b8ba76401c833a1ec74ea2bd07500000000200000000000000007375d930bcce7b8cd22c03b4f15324bcc0b2d24db0e96bc201aa21b2f8b6b11444e310100000000ed7ad8f0ffffffff671d816776e8c7fae18dc122fb6dc231956c6d48000000002000000000000000c5a7c8e603d9105bbb89f1a92dfb12e956e42ef13a3cbb045c49d0c354d0027d083631010000000019238c1f00000000e2fc4df9e7870fc8238136faca07a120cc078a0c000000002000000000000000d5348d0cfd3cd3172271c08714ead78550755c10cf495c0c2bce65abf1f4742f2834310100000000d29d9423000000001b03ab3c744545b281d224127708a9ec7487e58c00000000200000000000000062180dafc3a23d1cd22353326ea81165fdf25983c6332eef34fc52cec7d116b10534310100000000eddfb522000000002a1b3fee6b29ea1828d427f0e5c3887cb7e644870000000020000000000000005864bae0684d55cb11a12b90d20d7e895681a7b09e41a21348f22a0fec5c5003a133310100000000f2c5c6240000000035f6c0359084c37ef6eb4061b694386910093b9900000000200000000000000001ab6e5d364ec005a1b9f0c98915e4ac371ef63795bdb7193a1b2ec3a7be80d30131310100000000aded602100000000188f984567f5c6426386127b6fba27cb59684dc200000000200000000000000069d38fcce468d11594da7b380ba73cfb74d834d0315e9dd1e9672fe5db773652be2f31010000000033df33e6ffffffffccb2909dcd316fc5e738dcdb1f14360f04b4432000000000200000000000000047e6a0a47a0e0d5192a12de2ff570cde4803ea56d92a625660b70f5559949fc47e2f31010000000003db1021000000001a4b644c16a4a1904618f8d3c0f6a6722303839000000000200000000000000089f54396f3f6cd2cd33fee29702320a549b669139a915f1b8c5b6ae8518a4593492f310100000000b5a1901e0000000077be23139b650d69ede7985b31fd7b1cee9ca7d3000000002000000000000000ecba829b58afc1162afd916b741555d236a2b419b5b9bffb039b0fb283531798f82d31010000000005cf752000000000c268fe001a533b6f0860167381384363a790fd0d00000000200000000000000023c4c17b424e7ae26cb0d6ce2c60b27abd22303973e888ff39c0319821294bf1fb39a00000000000d81598fdffffffff7ea986d8723587f5e653c0df785fcfce39238785000000002000000000000000001484953fca1ecc45179bb96f9852f0b8434b7d15b1279edd5d6e6404ba27ccc2138200000000000f1890e5fffffffffbc2a3f682b507f1a3a5d4c44c8cbb6af08602960000000020000000000000002d6384b5cd3482006da1c8c968a6cd43602b649303e4869843f6bf6a045a70f116658100000000004a4abb1b00000000616a45478336b92583cfe29e8ee942802de69eae0000000020000000000000005d5c25e4c750c38e32ab492f4d0d96d4e2ad1673b9fd9f6c9d393ed932343e14295a8100000000004e232a0e00000000f42c7e49df4f4d0993237959518415e4737923920000000020000000000000001d2ecf9de5c7c78a773f0e3f4dfde4f1daaaab7cc0f39b609ad0f665adb395f0004a8100000000000f54fedeffffffff9407fa25941c932473f8e2b1ecc5481e3ae15a71000000002000000000000000b912dda2047d6026450945e0803a4aa8f26745dd539f9dbeb708809e4bc2df39673381000000000009a44cdfffffffffa8a2bf2709ca05755f881ebd939991aea9f88cf1000000002000000000000000b2b729b9af71db32b6c5b1ed5dbd44fcbea52514e7ede8498209e570188210fcca1f810000000000a451e11800000000aea26001de9b15d4dad32808712864e61ae52a1700000000200000000000000046d3d7b0a4adfbe4ffbf45eaaf4e2bada39701d41c6422e9cf9438ef8126b51cb00c810000000000e20f02d9ffffffff2888677e39e38bf4ee64081bd43776fb4d507fcb000000002000000000000000061caf848751b3590ab6f2c82d0ae3041f2831c8920dd62c4f556bd8d7a25e7f0be18000000000000cfd54100000000076f6dd3f07bf2e9ff5b0a188bc08d5a4efcf40d900000000200000000000000051c090dd4b573e3c85e64e3e28ca276969d0db6bed959aba0dfca09e6693a7d60bde800000000000a570ad070000000023df4a8418762cea4d24b7227ad125df76d2baf6000000002000000000000000d9efcd0e389b4f8045a327d3e66cb2ccc206a433c08878de2c23574a3ebf3b0d09dc80000000000042b2110e00000000a9c7a884bd7d6e799ef5311bf9168e0105bce41600000000200000000000000015dceda598a86558fe610d1de4908765605e860485b96ae31bc0ca99183035110bdb8000000000003a81e10900000000b8f9204a348e0dbc85abe5bc98936d6b84c01e450000000020000000000000002f66263f9e4c3013dc6ee70eaa4599f0ee579b0b7f9d047d280067e5644d0bb26dce8000000000001ccdab0900000000a8f38fcb39b580c4b856dcaed882d504a216ee0000000000200000000000000029d6bfb6948ca60cc7d089935d047be4d4d4ebacbd0f5e21542b9c4f6cbef4a018cb800000000000ae951e05000000009fee83fbf0bd5a4d71519efc16d0d438c203dd5a000000002000000000000000b0825c5296bc275ff8aa85da674b5b60d1248d25b6e40277d4bb2e8846128af541c680000000000051cc4b0600000000b44fc7d70708095fbc30af164410749483f96439000000002000000000000000ea7995131ac44f490378f1856438db362ae42cec3448ab98807b2f9fec37d8365dc3800000000000d864da04000000002ab301f354a74f45afb347b2c00cdceb09cc719d000000002000000000000000a854a7beb6315a04473d6cf8db85d164e8829595e31e80d006fd423100d2effa45c380000000000047e4e304000000002972b0df619f73d1a5c8933aff22ad8c9258f37000000000200000000000000036103ddb3cbaef3af1f92cb503b8ba10c22f10b24807bb3019121ea1069fd17a3bc28000000000005b461d0400000000dc0802ad3262ecd784ce339d87073c836e31c69e000000002000000000000000de67753135e4d339b16cab0f3408b5c6952fb792c9bb8c948c60ed5da0fb74a99cc0800000000000e8afe5030000000006be03f1b334fc3f7947d0d031f7dfa568005bd4000000002000000000000000467790623c59406b737d844edfc344c6256d82375a82ed5c8ab6f75d028b0f87dbbf800000000000d4d71c030000000060c51d643048d0a426171bd0deabd1689a89690f0000000020000000000000005250093effb687e0a1946e6e61d6a356cdc4abebe9e7fd58f37568464eed47bbc7bf800000000000ebfecd03000000009c4cc8543dc684948490681118957942b29a920000000000200000000000000099085ce62420e9a6a0a825378bc6e0345af98e4edbef7dec7baed443b5e2cc63c6bf800000000000c123c402000000005d673308ca2b45e5222303836d833e0f70b1f9b0000000002000000000000000f702393238df7999f3e68b59103f0a565a97444268bc0f03c58ac15888fb0044c5bf800000000000bad81e03000000003ce85b466ff74142307b4b03ddacf17a52d4637300000000200000000000000038f6ee432fb1ebfbe2aab2fb3dcdb514b44daaebc051e7a0252a29de6f72121780bf800000000000c96cb70200000000466d1236b8c77925b25219e4ffe34dd069351847000000002000000000000000f697bd6971d3a3801c910dde4769878b12ea64b8c0c37e0e9f340b66d8a44b8758bf80000000000093515b0300000000646d5c6384c84c2e660bf268d0a4ebc1ffbceea60000000020000000000000007cdf9ff876227fbe52357253549ad018e475f7bf7bbabd34d47cff554b3742044dbf800000000000c5b874030000000002d49d26c87ed7dc1e38d7bfe5b7fcdf5a3a3164000000002000000000000000b381533900081f35b3a65e06ff0b208f97188090d90267008a63ae77955c31c43dbf800000000000fdb8680300000000f1961eee75e5a1c8a11a5bf72fb368b990bdd28900000000200000000000000026b453d5a8f5d5ce66ae97ec87ffcf78a03e5c247ac25683800b01f9f7a0340bfebe800000000000d3bf5c03000000002c4e108c3f09cf9b8b1515d3b26d50e332d86e3b0000000020000000000000005887c3659bf7d4e9c90d4bf9651efcfc72e8cde419112a0360a71b26c728c5d7fdbe8000000000000e44370b00000000913834357f92e577cdc02b1e682c65e86d016ae5000000002000000000000000193d69ab72d9f31cfc80d75679df1dd80248580790faa8711c108e95ddb89345fdbe80000000000040e1a10200000000c4ef42d72d69f4b58ffe2a58db1335269842ac430000000020000000000000003b0c9675aa48cc05af281072bcbbf66b9825eb20fa80aebb785cea7ff4ab9591fdbe80000000000076fba30200000000d7076d16c2e91e0eb16753575ac3cee3057781c0000000002000000000000000a701f0f8ef57d7bf01f1399a93805e94e0b3d1545bd859061d9cd64ab4bfa33bfdbe80000000000072525c030000000001b2c0f03ba9b2f352a39bc45ce37aa2710b5af40000000020000000000000009b6110875f9cf79dcc82e63623e3b4f9aadf515804a79ed2ed1eeff143ac2e534dfb3800000000005075f9f6fffffffffbe1e3ed324fa17379403acfc3dd28fa882c6e6500000000200000000000000080f507fabe06cb3cf200b8b5f692f96221dd6d9c7054d14dabf69ea9030fa781267c0a0000000000909a91f2ffffffffedd5716db0738cb0eecd0a04aeb186c4a46720e900000000200000000000000072b60871b477e866f772fda2ea3ea2b83a331d29a0e5d616fdeedd3add31460c4ceb010000000000cd2e92e7ffffffffe6f8df77989972328c927e5493db6908936a4ea00000000020000000000000006197adf6d1fea4b71af280d08cf06a25dbc2a12a9f77e23b0009387304782de9937801000000000043e458d9fffffffff8260be47a1c7d42bbd9a656bdf8e3d3f28cbab2000000002000000000000000b4f7c002e2ccd4d414be2ea89f3d9ba621054349d2d06f871feb624d40fbe06453eb0000000000009de54a0a0000000025764380a6df1c9cad29a842a049f690ab0f08bc000000002000000000000000b119be04c77200537989f31794a527ed5b699064ea09236403dff00d03f2f92f5a7b00000000000033a51b0c0000000051b78fd2e37a2142b313f68a4edb31b23fe1afd100000000200000000000000046a6a52ea5dd0f7214a29e7c92ca575da79e26f861fe53b5ae84db2cf420c2548e5c0000000000009dfc0404000000008dabad1a71629e4938de8c44d0bd2f2ce1eabd080000000020000000000000004b24aadd2b8b16c8e686a4cec6d350c01b8777fc655818a990708afe7bc771351c3e000000000000229457faffffffff4e30f6a73766baa5fe6db7181b7fd08329c103e0000000002000000000000000a84f7bad047d6f18373e8410373ad6ee38e20df85446ddb5d8b8e1e0db6142c1463300000000000025c58defffffffff66439667823cf1cdefaf47d5ada806d462a6e05f00000000200000000000000075b96c2fd3bf61720acf96ef8d9a3aee11d4cee7e51d7793cd68bf66295c60527c2e000000000000021a6aeafffffffff70543a091a45a8c0de04e5ce9e40d9c16d980e1000000002000000000000000ae45dc59a343d10c19d36882a137a4f131f355b054d9c934a158b456e84af44f142e000000000000d7121cefffffffff6967de8bfc42a68d7c5eac521dd0099cea1fe230000000002000000000000000516be1ae9b9994b43b98613f3b0c24c3761d051aa27a0c9dc923fd60f3e61732891b000000000000160a45040000000043965a09c5be34d027a599bd6f450f5e6a13ddf7000000002000000000000000ea0ea17ea3caaaeb394412799c37d2e59c4708d650125f4ab827fe85b140a095ec180000000000003a884e1000000000afa7739d36004030202c7ec8cda8ad979c1891b90000000020000000000000004bd6cc488dd8f822c71e12c1f15a8b63c9460f0452990eb13cc19d3d408cfe02af17000000000000deb72ddcffffffff437f4b77f9c87a4d4b0dba30bd742bcaf4db48c800000000200000000000000042ea8ae95d794ebde340bb49bc02b25256071fb13970a53a8fdd2078520da3e8280a0000000000000bd41dd8ffffffffb860c8e1ad41462d6352406b85260c9713418ffa000000002000000000000000a43051727b37efe109f58c37356e78cd4a6c8659b8cac549998a8630f0df84300a0900000000000081f9a2080000000002c375d9b259cdbb7b8a97d71b2a0a0dbed890e00000000020000000000000005bed2f1bc08c7a9b46d8973aae006b54901dd00a74b2534b28e987d950f30341aa05000000000000a5bf1cdaffffffff0d1df149a8579909fedf2111513c57897118b09e00000000200000000000000079d419d4cdd176f50d166cd016c3d7ff910ef6c9db51194bc9c8a6e54a584af501040000000000002c36d1daffffffff5ade83d0ec62ab3eb6b023ff54c976eaf32e865500000000200000000000000047a58162ddcb35c8fd68f1f36f8b8fcd86ef8eb36013d86bd25912048b7c79246003000000000000c2e2db0600000000a93fccc20939a8a0fecc48a9737402486bc41d53000000002000000000000000ef67a6569b62e1f189982471ebd2ed120a18e15707dbe0a9257085e2bd144f1d5503000000000000857f0e0600000000c2f0ea5b136b23831fc601ff36ad43303702f3ec000000002000000000000000cb0d82f58047807fe4a7c44fb8e3591a4c399fc3f09770768e19fb63a5979060f501000000000000507048d7ffffffffbfffeb9cee54c0e110a1761f9a646e4589ee6fc0000000002000000000000000c073ae562642e4be9f84ead8cc50c0c7dc920b618e5bad629babbf98ffd4ae1df1010000000000009681990300000000409d07de01ea2a63c779c0246562aa278a95611e000000002000000000000000e90018fbdc1b1cbeb1d89457ebbcbcf8eea2615683e3401a00bdeceb0d16c84edc01000000000000fa977105000000009215b6b8ec64cb9e7b461c3945e6ce00da86f8f2000000002000000000000000427839a58cc1593a2e40ad449cb43155f7ed292a3c0e5398c9d22ea3c026f59090010000000000002845160600000000cea4cde95b67ff91fd25ea77c3075944764754b6000000002000000000000000d9d7fdfbee4ba7e247253aeb6faab2fd6e46b349ee7502c37f93588e43f2a4f9fe000000000000002e891bdaffffffff60131b8768de4f8a40aaa34a9f0d40be902807ae000000002000000000000000abeb97145ce1a283bdcc667645304de4ed529348748eb1d6e3c61a5e7a19b89d81000000000000008f6311fbffffffff13247270303a9d66ea8ffa02614a8b5369adda53000000002000000000000000b84ea7d385787015dff693a22044faf6a0cab48af1a877495eacab1d6b7859717200000000000000f9044e0400000000349f27787689a635ffbe00e0dd66e848690a73f2000000002000000000000000c74711e51b16a97d2b16139604ac68f9c241f5393a53cbcf757d9b142fb4da4a3a000000000000008c90850500000000e71b23c378e4c465bfe96ad48e7f93f237e939dc0000000020000000000000000004ed503ba91e4b03603d4cd037b919dbe815c50b403a25d175926683adde54210000000000000073fc69dbffffffffc2989ed51e73b742e4991cf915d6339419657ad5000000002000000000000000c0537de09cd816e67d622c910a84fbaf8ae77e855a3e228356631886c2a7efba1b00000000000000205578050000000080f78edcdb53aa69dea3f482472027327f1acf0e000000002000000000000000670a0d979a69fc78d59425442aba3548f2886fa68b4c783cab77a03405f3dbae0b000000000000008f6281040000000069a300bbe8cba17536cdb4346c148d1dd7475c6d0000000020000000000000006dafaf75f255ac071b403605b4fecde07eb28300943f29c7f04acc720561a30d020000000000000070a96d0500000000875a5ed5d1c185a47a9c2c96ec3491dea11d073b000000002000000000000000e82e177723fc67fb0c96eecb896c8a3c912b5e577129950a04e5cd51c1ef306d0200000000000000a67d16d8ffffffff1b21f9bf3b9a650607cced8c0c2fbeca3211791b0000000020000000000000006232324245cf914e074d45c3ef80c6c135e4d5aa64b9edf9474022b29b9f1f480100000000000000093c6d05000000002be016839732d17fe16983d34f3b1808732694430000000020000000000000007af2267433419b2aa9f7214d6ee66405b14585017a8591e6321841a452ed190e01000000000000007e5819d6ffffffff300636fc6b27a7016f3aef03c231aa63163eeb5d00000000200000000000000082bb8244f020803ce4b9f1e1749f878da3b64e9ffe521b4bb35b89c124be7da901000000000000002b3c6d050000000065c584027f5fd8349027fa5bf1d4ed3828c29058000000002000000000000000731048e047fb31f01277508ec27d9eaa0568a0fd10a76783425b3fa6fedd31af0100000000000000e0296d0500000000da37d2935eae0a92de55e86db2e887b4f18575f0000000002000000000000000c7b8b7b6b6cc2457c271bf5201f65b2fcb363350f4387438a6218ff839a4274909f3fa0200000000a0868cd9ffffffff03fd824b00000000010100000000000000fd3e01000000000064000000000000003c70d8336dee739bdf2d1c5a0fca0a88b87b9a6d000000002000000000000000860e2c9b980d61bf53e1d582f789c35bedf4a252c4093e9392ba7516d9d50511d203610300000000bcd1e80e0000000033ecd5899a91105bb4e51066eb138189bc3825ad00000000200000000000000075617111844fcc8c90a4fc121f2c37d19643667e868192690f2314bf14526ed405f2060300000000e836790a00000000f2e6bb25d8d269c9737bd3789b11dc76c4fc56e70000000020000000000000000106edcb594c1e457564711c5bc0ba0182bc42131fc5a02105da76c3e624a76f5f010303000000000f56f41100000000c27b627c2fd11ee1e3e850738720692be2e3f4c6000000002000000000000000bee1e39ec395c97158a2dd51695f96c7e724da138ebdbf7f417c17bb3796726eb779ff020000000092959b1800000000bb386aa0e7b37eb33516aebb58cae3574af85d1f0000000020000000000000007033e708beda4a88c330b82edb7286df02156fa71fa53152dc306298386d42bad90bfc02000000004efc7a1f000000002a3d40c3e7ce8d6b55077335cab141c2e11ac254000000002000000000000000914426099b4fe08deae4590a68f553bd79ebdd4303ec6c2586719d917aac6db8d393fb0200000000e0b5b3130000000044c395a4a96c6d1a450ed33b5a8ddb359cefed36000000002000000000000000aba9508fec18359f9042ef73816a167d25dcc2740d1336f66bb1e7c96c8141fd2482fb0200000000a6a30b1f0000000043804074334ac6111233037efe20105c516b0e9f000000002000000000000000e07a4ba0104afc6cdd71e063a03e195c109e332bf5e163c37a80ad67506a62fdc21bfb0200000000d235d90900000000ec033669fe506006f870bd8b12cb524a3abc3693000000002000000000000000003c8c0c3b27c3ddcd7414aa5d195175e58c36789c8ad9b58a4275e09db8d099f20afb02000000009357e3180000000064cdacb04fb9c3b290579644f9edbd05cb00bb7d000000002000000000000000c9d641627d6eced574dfdcb08e243cad1c9c9d71dceb684ad74fb865a95a41564c02fb020000000091cc1b06000000005295664886565f43a3074140551a755872bf6ffd0000000020000000000000006f7cb6f22bb847e20199220fc0903dd24eb2a9be6424f82f0d50a6727a5225c947fbfa02000000000feaf30800000000da37d2935eae0a92de55e86db2e887b4f18575f0000000002000000000000000c7b8b7b6b6cc2457c271bf5201f65b2fcb363350f4387438a6218ff839a4274909f3fa0200000000bcd1152300000000718160e3538e0b0ed83d3b3dc4ece83809fcd107000000002000000000000000bb1aed9ed9b0557ba3794a48f15243f4c8c6b9b176c69b589c71664f896496f9c7cd3b0100000000771914f0ffffffffb03f0cb997f3428ef3968833bca2f2a4f613a8eb00000000200000000000000069f12605bf4bd86e48fda138649db87461446490efadd047f77a1ff23606bf6d5f28330100000000c3759122000000003965c223e13ac66f8175c86e657d396c711964b7000000002000000000000000238dd24b92eb981a83414440555ca2b18c57dcb8be9982bb3fb4a28fafb26081e02232010000000075445215000000005c78a0762f13b0b2a793e82d1a178eb45642234d000000002000000000000000b45c85f1255127cd73f4edaadb0cd9307522b76be00d68849efd9a2bf0920a8274ff31010000000019949f2700000000410c1ebaf54c090f0a9132dfa9f79e0f480b2c060000000020000000000000006fbfe2d80362615ec2b5ad355b08c122e3799894f5a8c81c13af60909e1b057fe4d23101000000006bb56c1600000000b41ef50272a7a2398ba617d3c963986ebebc5b2d0000000020000000000000001a9acf9916b1e429c5bc5b12f5fea61acb5fef345712f25d12806ebc4369fdc22dd2310100000000b2c5810d000000006cc0a1531ff58d61dc9061c5b89bc0efc07911fc0000000020000000000000007eef62daf827c634f1036c88485a507519b226c7cbb79cf203e67b3628c32b7e49cf310100000000c5b3f7150000000053406de83381dd804632447f1baed00cf28048e000000000200000000000000051f5b4a16dc3ba302181af9de097f591a9546b9754b2b1cb86d2d980df4ceac08cc2310100000000cb0c0c140000000070dac78498afc6d766d860c433265a274d48b7d800000000200000000000000054393aab853ae377ef15a80742358da0a52a9ebea553734a068f444404b9903436c231010000000091b576e3ffffffff521d2a344e322b8ba76401c833a1ec74ea2bd07500000000200000000000000007375d930bcce7b8cd22c03b4f15324bcc0b2d24db0e96bc201aa21b2f8b6b11444e3101000000000885591d00000000671d816776e8c7fae18dc122fb6dc231956c6d48000000002000000000000000c5a7c8e603d9105bbb89f1a92dfb12e956e42ef13a3cbb045c49d0c354d0027d083631010000000049a68c0000000000e2fc4df9e7870fc8238136faca07a120cc078a0c000000002000000000000000d5348d0cfd3cd3172271c08714ead78550755c10cf495c0c2bce65abf1f4742f2834310100000000c2519504000000001b03ab3c744545b281d224127708a9ec7487e58c00000000200000000000000062180dafc3a23d1cd22353326ea81165fdf25983c6332eef34fc52cec7d116b105343101000000006b97b603000000002a1b3fee6b29ea1828d427f0e5c3887cb7e644870000000020000000000000005864bae0684d55cb11a12b90d20d7e895681a7b09e41a21348f22a0fec5c5003a1333101000000009887c7050000000035f6c0359084c37ef6eb4061b694386910093b9900000000200000000000000001ab6e5d364ec005a1b9f0c98915e4ac371ef63795bdb7193a1b2ec3a7be80d3013131010000000093f3610200000000188f984567f5c6426386127b6fba27cb59684dc200000000200000000000000069d38fcce468d11594da7b380ba73cfb74d834d0315e9dd1e9672fe5db773652be2f310100000000ea02b81200000000ccb2909dcd316fc5e738dcdb1f14360f04b4432000000000200000000000000047e6a0a47a0e0d5192a12de2ff570cde4803ea56d92a625660b70f5559949fc47e2f31010000000037081202000000001a4b644c16a4a1904618f8d3c0f6a6722303839000000000200000000000000089f54396f3f6cd2cd33fee29702320a549b669139a915f1b8c5b6ae8518a4593492f3101000000004bd491ffffffffff77be23139b650d69ede7985b31fd7b1cee9ca7d3000000002000000000000000ecba829b58afc1162afd916b741555d236a2b419b5b9bffb039b0fb283531798f82d310100000000d523770100000000c268fe001a533b6f0860167381384363a790fd0d00000000200000000000000023c4c17b424e7ae26cb0d6ce2c60b27abd22303973e888ff39c0319821294bf1fb39a000000000005a3252edffffffff7ea986d8723587f5e653c0df785fcfce39238785000000002000000000000000001484953fca1ecc45179bb96f9852f0b8434b7d15b1279edd5d6e6404ba27ccc2138200000000005e13dd2300000000fbc2a3f682b507f1a3a5d4c44c8cbb6af08602960000000020000000000000002d6384b5cd3482006da1c8c968a6cd43602b649303e4869843f6bf6a045a70f116658100000000000e06970e00000000616a45478336b92583cfe29e8ee942802de69eae0000000020000000000000005d5c25e4c750c38e32ab492f4d0d96d4e2ad1673b9fd9f6c9d393ed932343e14295a81000000000024fb060100000000f42c7e49df4f4d0993237959518415e4737923920000000020000000000000001d2ecf9de5c7c78a773f0e3f4dfde4f1daaaab7cc0f39b609ad0f665adb395f0004a81000000000012cd5f1d000000009407fa25941c932473f8e2b1ecc5481e3ae15a71000000002000000000000000b912dda2047d6026450945e0803a4aa8f26745dd539f9dbeb708809e4bc2df3967338100000000009668b01d00000000a8a2bf2709ca05755f881ebd939991aea9f88cf1000000002000000000000000b2b729b9af71db32b6c5b1ed5dbd44fcbea52514e7ede8498209e570188210fcca1f8100000000002017c40b00000000aea26001de9b15d4dad32808712864e61ae52a1700000000200000000000000046d3d7b0a4adfbe4ffbf45eaaf4e2bada39701d41c6422e9cf9438ef8126b51cb00c81000000000005c36917000000002888677e39e38bf4ee64081bd43776fb4d507fcb000000002000000000000000061caf848751b3590ab6f2c82d0ae3041f2831c8920dd62c4f556bd8d7a25e7f0be1800000000000ee213e030000000076f6dd3f07bf2e9ff5b0a188bc08d5a4efcf40d900000000200000000000000051c090dd4b573e3c85e64e3e28ca276969d0db6bed959aba0dfca09e6693a7d60bde80000000000087e396faffffffff23df4a8418762cea4d24b7227ad125df76d2baf6000000002000000000000000d9efcd0e389b4f8045a327d3e66cb2ccc206a433c08878de2c23574a3ebf3b0d09dc8000000000005859fb0000000000a9c7a884bd7d6e799ef5311bf9168e0105bce41600000000200000000000000015dceda598a86558fe610d1de4908765605e860485b96ae31bc0ca99183035110bdb8000000000001c42cbfcffffffffb8f9204a348e0dbc85abe5bc98936d6b84c01e450000000020000000000000002f66263f9e4c3013dc6ee70eaa4599f0ee579b0b7f9d047d280067e5644d0bb26dce8000000000000ad696fcffffffffa8f38fcb39b580c4b856dcaed882d504a216ee0000000000200000000000000029d6bfb6948ca60cc7d089935d047be4d4d4ebacbd0f5e21542b9c4f6cbef4a018cb8000000000003ef509f8ffffffff9fee83fbf0bd5a4d71519efc16d0d438c203dd5a000000002000000000000000b0825c5296bc275ff8aa85da674b5b60d1248d25b6e40277d4bb2e8846128af541c6800000000000b7a937f9ffffffffb44fc7d70708095fbc30af164410749483f96439000000002000000000000000ea7995131ac44f490378f1856438db362ae42cec3448ab98807b2f9fec37d8365dc3800000000000668dc6f7ffffffff2ab301f354a74f45afb347b2c00cdceb09cc719d000000002000000000000000a854a7beb6315a04473d6cf8db85d164e8829595e31e80d006fd423100d2effa45c3800000000000450fd0f7ffffffff2972b0df619f73d1a5c8933aff22ad8c9258f37000000000200000000000000036103ddb3cbaef3af1f92cb503b8ba10c22f10b24807bb3019121ea1069fd17a3bc28000000000005d8c09f7ffffffffdc0802ad3262ecd784ce339d87073c836e31c69e000000002000000000000000de67753135e4d339b16cab0f3408b5c6952fb792c9bb8c948c60ed5da0fb74a99cc08000000000001020d2f6ffffffff06be03f1b334fc3f7947d0d031f7dfa568005bd4000000002000000000000000467790623c59406b737d844edfc344c6256d82375a82ed5c8ab6f75d028b0f87dbbf800000000000965b09f6ffffffff60c51d643048d0a426171bd0deabd1689a89690f0000000020000000000000005250093effb687e0a1946e6e61d6a356cdc4abebe9e7fd58f37568464eed47bbc7bf800000000000b584baf6ffffffff9c4cc8543dc684948490681118957942b29a920000000000200000000000000099085ce62420e9a6a0a825378bc6e0345af98e4edbef7dec7baed443b5e2cc63c6bf800000000000a5a9b0f5ffffffff5d673308ca2b45e5222303836d833e0f70b1f9b0000000002000000000000000f702393238df7999f3e68b59103f0a565a97444268bc0f03c58ac15888fb0044c5bf800000000000b85e0bf6ffffffff3ce85b466ff74142307b4b03ddacf17a52d4637300000000200000000000000038f6ee432fb1ebfbe2aab2fb3dcdb514b44daaebc051e7a0252a29de6f72121780bf800000000000c9f9a3f5ffffffff466d1236b8c77925b25219e4ffe34dd069351847000000002000000000000000f697bd6971d3a3801c910dde4769878b12ea64b8c0c37e0e9f340b66d8a44b8758bf800000000000a3e247f6ffffffff646d5c6384c84c2e660bf268d0a4ebc1ffbceea60000000020000000000000007cdf9ff876227fbe52357253549ad018e475f7bf7bbabd34d47cff554b3742044dbf800000000000f34a61f6ffffffff02d49d26c87ed7dc1e38d7bfe5b7fcdf5a3a3164000000002000000000000000b381533900081f35b3a65e06ff0b208f97188090d90267008a63ae77955c31c43dbf800000000000cb4c55f6fffffffff1961eee75e5a1c8a11a5bf72fb368b990bdd28900000000200000000000000026b453d5a8f5d5ce66ae97ec87ffcf78a03e5c247ac25683800b01f9f7a0340bfebe800000000000075a49f6ffffffff2c4e108c3f09cf9b8b1515d3b26d50e332d86e3b0000000020000000000000005887c3659bf7d4e9c90d4bf9651efcfc72e8cde419112a0360a71b26c728c5d7fdbe8000000000005cde23feffffffff913834357f92e577cdc02b1e682c65e86d016ae5000000002000000000000000193d69ab72d9f31cfc80d75679df1dd80248580790faa8711c108e95ddb89345fdbe8000000000008e7b8ef5ffffffffc4ef42d72d69f4b58ffe2a58db1335269842ac430000000020000000000000003b0c9675aa48cc05af281072bcbbf66b9825eb20fa80aebb785cea7ff4ab9591fdbe800000000000c49590f5ffffffffd7076d16c2e91e0eb16753575ac3cee3057781c0000000002000000000000000a701f0f8ef57d7bf01f1399a93805e94e0b3d1545bd859061d9cd64ab4bfa33bfdbe800000000000c0ec48f6ffffffff01b2c0f03ba9b2f352a39bc45ce37aa2710b5af40000000020000000000000009b6110875f9cf79dcc82e63623e3b4f9aadf515804a79ed2ed1eeff143ac2e534dfb3800000000007eef2ff1fffffffffbe1e3ed324fa17379403acfc3dd28fa882c6e6500000000200000000000000080f507fabe06cb3cf200b8b5f692f96221dd6d9c7054d14dabf69ea9030fa781267c0a0000000000b4fe80f1ffffffffedd5716db0738cb0eecd0a04aeb186c4a46720e900000000200000000000000072b60871b477e866f772fda2ea3ea2b83a331d29a0e5d616fdeedd3add31460c4ceb010000000000154960e7ffffffffe6f8df77989972328c927e5493db6908936a4ea00000000020000000000000006197adf6d1fea4b71af280d08cf06a25dbc2a12a9f77e23b0009387304782de9937801000000000055a532d9fffffffff8260be47a1c7d42bbd9a656bdf8e3d3f28cbab2000000002000000000000000b4f7c002e2ccd4d414be2ea89f3d9ba621054349d2d06f871feb624d40fbe06453eb0000000000002fff320a0000000025764380a6df1c9cad29a842a049f690ab0f08bc000000002000000000000000b119be04c77200537989f31794a527ed5b699064ea09236403dff00d03f2f92f5a7b0000000000000f1e0f0c0000000051b78fd2e37a2142b313f68a4edb31b23fe1afd100000000200000000000000046a6a52ea5dd0f7214a29e7c92ca575da79e26f861fe53b5ae84db2cf420c2548e5c0000000000003196fb03000000008dabad1a71629e4938de8c44d0bd2f2ce1eabd080000000020000000000000004b24aadd2b8b16c8e686a4cec6d350c01b8777fc655818a990708afe7bc771351c3e0000000000004a4551faffffffff4e30f6a73766baa5fe6db7181b7fd08329c103e0000000002000000000000000a84f7bad047d6f18373e8410373ad6ee38e20df85446ddb5d8b8e1e0db6142c14633000000000000099088efffffffff66439667823cf1cdefaf47d5ada806d462a6e05f00000000200000000000000075b96c2fd3bf61720acf96ef8d9a3aee11d4cee7e51d7793cd68bf66295c60527c2e0000000000006a6165eafffffffff70543a091a45a8c0de04e5ce9e40d9c16d980e1000000002000000000000000ae45dc59a343d10c19d36882a137a4f131f355b054d9c934a158b456e84af44f142e000000000000cf6417efffffffff6967de8bfc42a68d7c5eac521dd0099cea1fe230000000002000000000000000516be1ae9b9994b43b98613f3b0c24c3761d051aa27a0c9dc923fd60f3e61732891b0000000000002c3e42040000000043965a09c5be34d027a599bd6f450f5e6a13ddf7000000002000000000000000ea0ea17ea3caaaeb394412799c37d2e59c4708d650125f4ab827fe85b140a095ec1800000000000042004c1000000000afa7739d36004030202c7ec8cda8ad979c1891b90000000020000000000000004bd6cc488dd8f822c71e12c1f15a8b63c9460f0452990eb13cc19d3d408cfe02af1700000000000018502bdcffffffff437f4b77f9c87a4d4b0dba30bd742bcaf4db48c800000000200000000000000042ea8ae95d794ebde340bb49bc02b25256071fb13970a53a8fdd2078520da3e8280a000000000000fbcb1cd8ffffffffb860c8e1ad41462d6352406b85260c9713418ffa000000002000000000000000a43051727b37efe109f58c37356e78cd4a6c8659b8cac549998a8630f0df84300a090000000000007d0ea2080000000002c375d9b259cdbb7b8a97d71b2a0a0dbed890e00000000020000000000000005bed2f1bc08c7a9b46d8973aae006b54901dd00a74b2534b28e987d950f30341aa05000000000000612c1cdaffffffff0d1df149a8579909fedf2111513c57897118b09e00000000200000000000000079d419d4cdd176f50d166cd016c3d7ff910ef6c9db51194bc9c8a6e54a584af5010400000000000012ced0daffffffff5ade83d0ec62ab3eb6b023ff54c976eaf32e865500000000200000000000000047a58162ddcb35c8fd68f1f36f8b8fcd86ef8eb36013d86bd25912048b7c79246003000000000000028bdb0600000000a93fccc20939a8a0fecc48a9737402486bc41d53000000002000000000000000ef67a6569b62e1f189982471ebd2ed120a18e15707dbe0a9257085e2bd144f1d5503000000000000e3280e0600000000c2f0ea5b136b23831fc601ff36ad43303702f3ec000000002000000000000000cb0d82f58047807fe4a7c44fb8e3591a4c399fc3f09770768e19fb63a5979060f5010000000000006e3d48d7ffffffffbfffeb9cee54c0e110a1761f9a646e4589ee6fc0000000002000000000000000c073ae562642e4be9f84ead8cc50c0c7dc920b618e5bad629babbf98ffd4ae1df1010000000000001c4f990300000000409d07de01ea2a63c779c0246562aa278a95611e000000002000000000000000e90018fbdc1b1cbeb1d89457ebbcbcf8eea2615683e3401a00bdeceb0d16c84edc01000000000000a2677105000000009215b6b8ec64cb9e7b461c3945e6ce00da86f8f2000000002000000000000000427839a58cc1593a2e40ad449cb43155f7ed292a3c0e5398c9d22ea3c026f5909001000000000000881c160600000000cea4cde95b67ff91fd25ea77c3075944764754b6000000002000000000000000d9d7fdfbee4ba7e247253aeb6faab2fd6e46b349ee7502c37f93588e43f2a4f9fe00000000000000626f1bdaffffffff60131b8768de4f8a40aaa34a9f0d40be902807ae000000002000000000000000abeb97145ce1a283bdcc667645304de4ed529348748eb1d6e3c61a5e7a19b89d8100000000000000755611fbffffffff13247270303a9d66ea8ffa02614a8b5369adda53000000002000000000000000b84ea7d385787015dff693a22044faf6a0cab48af1a877495eacab1d6b785971720000000000000065f94d0400000000349f27787689a635ffbe00e0dd66e848690a73f2000000002000000000000000c74711e51b16a97d2b16139604ac68f9c241f5393a53cbcf757d9b142fb4da4a3a00000000000000a88a850500000000e71b23c378e4c465bfe96ad48e7f93f237e939dc0000000020000000000000000004ed503ba91e4b03603d4cd037b919dbe815c50b403a25d175926683adde54210000000000000019f969dbffffffffc2989ed51e73b742e4991cf915d6339419657ad5000000002000000000000000c0537de09cd816e67d622c910a84fbaf8ae77e855a3e228356631886c2a7efba1b00000000000000625278050000000080f78edcdb53aa69dea3f482472027327f1acf0e000000002000000000000000670a0d979a69fc78d59425442aba3548f2886fa68b4c783cab77a03405f3dbae0b00000000000000716181040000000069a300bbe8cba17536cdb4346c148d1dd7475c6d0000000020000000000000006dafaf75f255ac071b403605b4fecde07eb28300943f29c7f04acc720561a30d02000000000000003ca96d0500000000875a5ed5d1c185a47a9c2c96ec3491dea11d073b000000002000000000000000e82e177723fc67fb0c96eecb896c8a3c912b5e577129950a04e5cd51c1ef306d0200000000000000727d16d8ffffffff1b21f9bf3b9a650607cced8c0c2fbeca3211791b0000000020000000000000006232324245cf914e074d45c3ef80c6c135e4d5aa64b9edf9474022b29b9f1f480100000000000000ef3b6d05000000002be016839732d17fe16983d34f3b1808732694430000000020000000000000007af2267433419b2aa9f7214d6ee66405b14585017a8591e6321841a452ed190e0100000000000000645819d6ffffffff300636fc6b27a7016f3aef03c231aa63163eeb5d00000000200000000000000082bb8244f020803ce4b9f1e1749f878da3b64e9ffe521b4bb35b89c124be7da90100000000000000113c6d050000000065c584027f5fd8349027fa5bf1d4ed3828c29058000000002000000000000000731048e047fb31f01277508ec27d9eaa0568a0fd10a76783425b3fa6fedd31af0100000000000000c6296d050000000070dac78498afc6d766d860c433265a274d48b7d800000000200000000000000054393aab853ae377ef15a80742358da0a52a9ebea553734a068f444404b9903436c231010000000091b576e3ffffffff03fd824b00000000").as_slice()).unwrap();

        let timestamp = Timestamp::from_nanos(header.signed_header.header.time.as_unix_nanos());

        let state_update = verify_header(
            client_state.clone(),
            consensus_state,
            header.clone(),
            timestamp,
            &SignatureVerifier {
                verifier: EdVerifier {
                    deps: mock_dependencies().as_ref(),
                },
            },
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
