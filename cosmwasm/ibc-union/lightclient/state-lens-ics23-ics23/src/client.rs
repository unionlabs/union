use cometbls_light_client::client::CometblsLightClient;
use cosmwasm_std::{Addr, Empty};
use ibc_union_light_client::{
    ClientCreationResult, IbcClient, IbcClientCtx, IbcClientError, StateUpdate,
};
use ibc_union_msg::lightclient::VerifyCreationResponseEvent;
use ibc_union_spec::{Status, Timestamp, path::ConsensusStatePath};
use ics23::ibc_api::SDK_SPECS;
use state_lens_ics23_ics23_light_client_types::{ClientState, ConsensusState, client_state::Extra};
use state_lens_light_client_types::Header;
use unionlabs::{
    encoding::{Bincode, DecodeAs, EthAbi},
    ethereum::{ibc_commitment_key, keccak256},
    ibc::core::commitment::{merkle_proof::MerkleProof, merkle_root::MerkleRoot},
    primitives::H256,
};

use crate::errors::Error;

pub struct StateLensIcs23Ics23LightClient;

impl IbcClient for StateLensIcs23Ics23LightClient {
    type Error = Error;

    type CustomQuery = Empty;

    type Header = Header;

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
        let consensus_state = ctx.read_self_consensus_state(height)?;
        let client_state = ctx.read_self_client_state()?;

        let (store_key, key_prefix_storage, evm_commitment) = match client_state.extra {
            Extra::V1(extra_v1) => (extra_v1.store_key, extra_v1.key_prefix_storage, false),
            Extra::V2(extra_v2) => (extra_v2.store_key, extra_v2.key_prefix_storage, true),
        };

        verify_membership(
            &key,
            &MerkleRoot {
                hash: consensus_state.app_hash.into_encoding(),
            },
            &storage_proof,
            &value,
            &store_key,
            &key_prefix_storage,
            evm_commitment,
        )?;

        Ok(())
    }

    fn verify_non_membership(
        ctx: IbcClientCtx<Self>,
        height: u64,
        key: Vec<u8>,
        storage_proof: Self::StorageProof,
    ) -> Result<(), IbcClientError<Self>> {
        let consensus_state = ctx.read_self_consensus_state(height)?;
        let client_state = ctx.read_self_client_state()?;

        let (store_key, key_prefix_storage, evm_commitment) = match client_state.extra {
            Extra::V1(extra_v1) => (extra_v1.store_key, extra_v1.key_prefix_storage, false),
            Extra::V2(extra_v2) => (extra_v2.store_key, extra_v2.key_prefix_storage, true),
        };

        verify_non_membership(
            &key,
            &MerkleRoot {
                hash: consensus_state.app_hash.into_encoding(),
            },
            &storage_proof,
            &store_key,
            &key_prefix_storage,
            evm_commitment,
        )?;

        Ok(())
    }

    fn get_timestamp(consensus_state: &Self::ConsensusState) -> Timestamp {
        consensus_state.timestamp
    }

    fn get_latest_height(client_state: &Self::ClientState) -> u64 {
        client_state.l2_latest_height
    }

    fn get_counterparty_chain_id(client_state: &Self::ClientState) -> String {
        client_state.l2_chain_id.clone()
    }

    fn status(ctx: IbcClientCtx<Self>, client_state: &Self::ClientState) -> Status {
        let _ = ctx;
        let _ = client_state;

        // FIXME: expose the ctx to this call to allow threading this call to L1
        // client. generally, we want to thread if a client is an L2 so always
        // provide the ctx?
        // let client_state: WasmClientState = read_client_state(deps)?;
        // let l1_client_state = query_client_state::<WasmL1ClientState>(
        //     deps,
        //     env,
        //     client_state.data.l1_client_id.clone(),
        // )
        // .map_err(Error::CustomQuery)?;

        // if l1_client_state.data.frozen_height != Height::default() {
        //     return Ok(Status::Frozen);
        // }

        // let Some(_) = read_consensus_state::<Self>(deps, &client_state.latest_height)? else {
        //     return Ok(Status::Expired);
        // };

        // Ok(Status::Active)
        Status::Active
    }

    fn verify_creation(
        _caller: Addr,
        client_state: &Self::ClientState,
        _consensus_state: &Self::ConsensusState,
        _relayer: Addr,
    ) -> Result<ClientCreationResult<Self>, IbcClientError<StateLensIcs23Ics23LightClient>> {
        Ok(
            ClientCreationResult::new().add_event(VerifyCreationResponseEvent::CreateLensClient {
                l1_client_id: client_state.l1_client_id,
                l2_client_id: client_state.l2_client_id,
                l2_chain_id: client_state.l2_chain_id.clone(),
            }),
        )
    }

    fn verify_header(
        ctx: IbcClientCtx<Self>,
        _caller: Addr,
        header: Self::Header,
        _relayer: Addr,
    ) -> Result<StateUpdate<Self>, ibc_union_light_client::IbcClientError<Self>> {
        let mut client_state = ctx.read_self_client_state()?;

        let storage_proof = MerkleProof::decode_as::<Bincode>(&header.l2_consensus_state_proof)
            .map_err(|_| Error::ProofDecode(header.l2_consensus_state_proof))?;

        let l2_consensus_state =
            tendermint_light_client_types::ConsensusState::decode_as::<EthAbi>(
                &header.l2_consensus_state,
            )
            .map_err(Error::InvalidL2ConsensusState)?;

        ctx.verify_membership::<CometblsLightClient>(
            client_state.l1_client_id,
            header.l1_height.height(),
            ConsensusStatePath {
                client_id: client_state.l2_client_id,
                height: header.l2_height.height(),
            }
            .key()
            .into_bytes(),
            storage_proof,
            keccak256(&header.l2_consensus_state).into(),
        )
        .map_err(Error::L1Error)?;

        let mut state_update = StateUpdate::new(
            header.l2_height.height(),
            ConsensusState {
                timestamp: Timestamp::from_nanos(l2_consensus_state.timestamp.as_unix_nanos()),
                app_hash: l2_consensus_state.root.hash.into_encoding(),
            },
        );

        if client_state.l2_latest_height < header.l2_height.height() {
            client_state.l2_latest_height = header.l2_height.height();
            state_update = state_update.overwrite_client_state(client_state);
        }

        Ok(state_update)
    }

    fn misbehaviour(
        _ctx: IbcClientCtx<Self>,
        _caller: Addr,
        _misbehaviour: Self::Misbehaviour,
        _relayer: Addr,
    ) -> Result<Self::ClientState, IbcClientError<Self>> {
        Err(Error::Unimplemented.into())
    }
}

/// Verify a membership proof.
///
/// The proof is verified with the standard [`SDK_SPECS`]. The iavl proof is verified against
/// `store_key`, and the tendermint proof is verified against `key_prefix_storage ++ key`, where
/// `key` is either `path` or the [`ibc_commitment_key`] of `path` if `evm_commitment` is true.
///
/// # Errors
///
/// This function will error if either the path or the value are invalid (i.e. not a 32 byte value),
/// or if the proof cannot be verified.
pub fn verify_membership(
    key: &[u8],
    root: &MerkleRoot,
    proof: &MerkleProof,
    value: &[u8],
    store_key: &[u8],
    key_prefix_storage: &[u8],
    evm_commitment: bool,
) -> Result<(), Error> {
    let path = H256::try_from(key).map_err(|_| Error::InvalidCommitmentKeyLength(key.into()))?;

    let value =
        <H256>::try_from(value).map_err(|_| Error::InvalidCommitmentValueLength(value.into()))?;

    let key = if evm_commitment {
        ibc_commitment_key(path).to_be_bytes().into()
    } else {
        path
    };

    ics23::ibc_api::verify_membership(
        proof,
        &SDK_SPECS,
        root,
        &[
            store_key.to_vec(),
            key_prefix_storage
                .iter()
                .chain(&key)
                .copied()
                .collect::<Vec<_>>(),
        ],
        value.into(),
    )
    .map_err(Error::VerifyMembership)
}

/// Verify a non-membership proof.
///
/// The proof is verified with the standard [`SDK_SPECS`]. The iavl proof is verified against
/// `store_key`, and the tendermint proof is verified against `key_prefix_storage ++ key`, where
/// `key` is either `path` or the [`ibc_commitment_key`] of `path` if `evm_commitment` is true.
///
/// # Errors
///
/// This function will error if the path is invalid (i.e. not a 32 byte value), or if the proof
/// cannot be verified.
pub fn verify_non_membership(
    key: &[u8],
    root: &MerkleRoot,
    proof: &MerkleProof,
    store_key: &[u8],
    key_prefix_storage: &[u8],
    evm_commitment: bool,
) -> Result<(), Error> {
    let path = H256::try_from(key).map_err(|_| Error::InvalidCommitmentKeyLength(key.into()))?;

    let key = if evm_commitment {
        ibc_commitment_key(path).to_be_bytes().into()
    } else {
        path
    };

    ics23::ibc_api::verify_non_membership(
        proof,
        &SDK_SPECS,
        root,
        &[
            store_key.to_vec(),
            key_prefix_storage
                .iter()
                .chain(&key)
                .copied()
                .collect::<Vec<_>>(),
        ],
    )
    .map_err(Error::VerifyNonMembership)
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use super::*;

    // NOTE: app_hash (root) can be fetched with: curl "$RPC_URL/commit?height=$HEIGHT" | jq .result.signed_header.header.app_hash
    // NOTE: for sei remove the .result from the jq expression

    #[test]
    fn verify_membership_v2_evm_commitment() {
        // voyager rpc ibc-proof 1328 '{"client_state":{"client_id":4}}' --height 173218977
        let proof_json = r#"{"proofs":[{"@type":"exist","@value":{"key":"0x03ee4ea8d358473f0fcebf0329feed95d56e8c04d7b5c1de0ab73c11497798f429ec187ad784c3261f62637108ef73835a93ab9614","leaf":{"hash":"sha256","length":"var_proto","prefix":"0x0002ea8f94a501","prehash_key":"no_hash","prehash_value":"sha256"},"path":[{"hash":"sha256","prefix":"0x0204ea8f94a50120","suffix":"0x207912be8970359113fbb865562835e76480fa28b5f35d346faae7fb6ad96bb377"},{"hash":"sha256","prefix":"0x0408ea8f94a501205a26dbccd29242a010a0922b2a02b03ce7e8ea8f426185ea94ecdb1c75ba3a4420","suffix":"0x0"},{"hash":"sha256","prefix":"0x060cea8f94a50120","suffix":"0x2058f4602044ae86d9f9e9fce2e04fb9af9d869169089f01157e71319f6567a35c"},{"hash":"sha256","prefix":"0x081aea8f94a50120","suffix":"0x207afacadec7f780ef84faed570a47c6e90593b3e5648224477ac21a90c4de6c0c"},{"hash":"sha256","prefix":"0x0a2aea8f94a50120","suffix":"0x20a807233f4d3c83321ab80ee8ed08a2ac703793b61a612351dcfc92939a164afb"},{"hash":"sha256","prefix":"0x0c46ea8f94a501206a66a24db521b718e40e22df3afdabc4af0204dee2b8587b34c192bb3cd88ba320","suffix":"0x0"},{"hash":"sha256","prefix":"0x0e8801ea8f94a50120cab16c18f017be51a65f473d5e7bdbd1318cf81665b9878b0cfa3880b76accd820","suffix":"0x0"},{"hash":"sha256","prefix":"0x10c801ea8f94a50120","suffix":"0x202e7118befb4cc724ca9022aaf8cd29de1b065972387f5f4b0b95429545268f5d"},{"hash":"sha256","prefix":"0x12a602ea8f94a5012027808a27bf3490a5c9f4839ccc23f037c3119b0a502d858a4887bc4a69b61ceb20","suffix":"0x0"},{"hash":"sha256","prefix":"0x14bc04ea8f94a501203f5dc44e6e016ee2cf63117652a41e798da305e2ecafe56cd47ee8efb3d283b020","suffix":"0x0"},{"hash":"sha256","prefix":"0x168008ea8f94a50120a93bd924367779b7fd85f74b015aaa32c4fc36a11f253308454f615476e2aa7a20","suffix":"0x0"},{"hash":"sha256","prefix":"0x1ac418ea8f94a50120","suffix":"0x20d2c575d2bb0dbc8ea59ed1cf8dce802d8170f0514f06f0c1ecad3101016f4a46"},{"hash":"sha256","prefix":"0x1eac5aea8f94a50120a93fcfca145b9e58c455e590ccf7bc76d7826558dbbfa8adaf2eac32971b6da320","suffix":"0x0"},{"hash":"sha256","prefix":"0x2096a601ea8f94a50120389e517541f67acc55abf7ee1945f9e080a5fa7a8002616e472788af81b53a8720","suffix":"0x0"},{"hash":"sha256","prefix":"0x22b68702ea8f94a50120","suffix":"0x20cd6795fabe891156a6b60d70bff6b033dc2d35312327246b4b4a490691b1d2b2"},{"hash":"sha256","prefix":"0x24d88f03ea8f94a50120","suffix":"0x20377732534efcee7126a74ca30ab3643c363e6c5f9a0be74479f6866caebf26b1"},{"hash":"sha256","prefix":"0x26a8a606ea8f94a50120513587dd3eb217b8e231cc6d43822a046628e088a57e77c35d6215673d2af6d420","suffix":"0x0"},{"hash":"sha256","prefix":"0x28b6930cea8f94a501205e4f42b1b0cf57bb1b2b98772eae4714501fc1f735c1c5ce90fbad647e10427c20","suffix":"0x0"},{"hash":"sha256","prefix":"0x2ab2f112ea8f94a501208a504d1ba4611b1526146d14e613676d8bebf8564186676d327cc5a6ff5a5cfb20","suffix":"0x0"},{"hash":"sha256","prefix":"0x2cfcdb20ea8f94a50120f1749362b059ed5688a52f32b5f0bd835f74f5002994786e53f75d407384d7ea20","suffix":"0x0"},{"hash":"sha256","prefix":"0x2ea0e74af6f098a50120","suffix":"0x20253279b9158fea6749bb98ec21ce999e8cfaf0b0647886f02eaa397daf308b98"},{"hash":"sha256","prefix":"0x30f6b78e01c0f298a50120","suffix":"0x207bb1ce6f31e87674ccc92d2210ff321615395ab7e838c06e19d8d3109aa9c9e7"},{"hash":"sha256","prefix":"0x3280bab302c0f298a50120a97902da9180c12af3f5f65cad3921ccd5099a3242319abe335ee4777fc1181a20","suffix":"0x0"},{"hash":"sha256","prefix":"0x34c2938104c0f298a5012043bc803a052999e33a56459f26e0852beec39d7d29be134d12288030bff01e1520","suffix":"0x0"},{"hash":"sha256","prefix":"0x36b0908407c0f298a501209c2be38907dde07409e917e8aeadcd80bd7086ded9f2d7416c1f55710cfd92d920","suffix":"0x0"},{"hash":"sha256","prefix":"0x38b895db0bc0f298a501205999211d6315e7d3f3929e894fda0019ef9767c018b600682eb10885e8ee4b2520","suffix":"0x0"},{"hash":"sha256","prefix":"0x3a9cafb617c0f298a5012047f136e2ddcd0ffac9345d8f47c8af2f36a6866dac9315fb532dae42872913e320","suffix":"0x0"}],"value":"0x9a0d99845a85f92af893fcc4f90db7fb76d262dfc144459c6e90642c0e4bb437"}},{"@type":"exist","@value":{"key":"0x65766d","leaf":{"hash":"sha256","length":"var_proto","prefix":"0x00","prehash_key":"no_hash","prehash_value":"sha256"},"path":[{"hash":"sha256","prefix":"0x01337fb01fe848a98e4d7e4f97edd06baae1ab103533b6345917aa8a82d1646032","suffix":"0x0"},{"hash":"sha256","prefix":"0x01","suffix":"0xce1207acd784dfaf0b5007e567e4c925ed0b25b76ebeb4635e0beb937fbe9144"},{"hash":"sha256","prefix":"0x01","suffix":"0x4b666b98c59cf57436bcce3e618780d0d3898d55ba578f196ad058453b507880"},{"hash":"sha256","prefix":"0x01bb7c790383755ebb5c54aa138254059ecf09d6fd4f72bd54456a37946d4fab76","suffix":"0x0"},{"hash":"sha256","prefix":"0x01","suffix":"0x1b11d8cf47f215548b4bf10d7d8bf34cb0aca55faa54d3ded6f7d2675677bfc7"}],"value":"0x07250ac934a747a63f163bc4e64c0a4985e4e3297228c1082045fb3302d75b0d"}}]}"#;

        verify_membership(
            // u path client-state 4
            &hex!("17ef568e3e12ab5b9c7254a8d58478811de00f9e6eb34345acd53bf8fd09d3ec"),
            &MerkleRoot {
                hash: hex!("d0a81718cc318725c92e633106070caf16e51a447ded433f34c7c53df6bada71")
                    .into(),
            },
            &serde_json::from_str(proof_json).unwrap(),
            // voyager rpc client-state 1328 --height 173218977 | jq .state -r | u hex -d | cast keccak
            &hex!("9a0d99845a85f92af893fcc4f90db7fb76d262dfc144459c6e90642c0e4bb437"),
            b"evm",
            &hex!("03" "ee4ea8d358473f0fcebf0329feed95d56e8c04d7"),
            true,
        )
        .unwrap();
    }

    #[test]
    fn verify_non_membership_v2_evm_commitment() {
        // voyager rpc ibc-proof 1328 '{"client_state":{"client_id":5}}' --height 173218977
        // client 5 does not exist at this height
        let proof_json = r#"{"proofs":[{"@type":"nonexist","@value":{"key":"0x03ee4ea8d358473f0fcebf0329feed95d56e8c04d7cb1c2d00091ce3f9ab6ac80192f98042c72a89aa2ae548025e96800b1514cd1e","left":{"key":"0x03ee4ea8d358473f0fcebf0329feed95d56e8c04d7c7d3544292610d3ca89fdfd96140e78409e8fdfe2b997677676357195b032d4e","leaf":{"hash":"sha256","length":"var_proto","prefix":"0x0002c6a694a201","prehash_key":"no_hash","prehash_value":"sha256"},"path":[{"hash":"sha256","prefix":"0x0204ea8f94a50120","suffix":"0x20e917b661cfe4dd7ae97fae52f8ee7e190b94879c5c7c86b82c636ca821130197"},{"hash":"sha256","prefix":"0x0408ea8f94a50120","suffix":"0x20b70c5b0c267f1fa9e245af7dd065f9c0eaf466889665119ff62082b00f739193"},{"hash":"sha256","prefix":"0x060eea8f94a50120","suffix":"0x2005639b1ead1a1e9aae79b0d5452c43a6b1764203c22f7c1dfb6714841c2c8151"},{"hash":"sha256","prefix":"0x081aea8f94a501201332cc18faa250d45ecad10d1a64e3fce9ba31198ebdaa7bf16f383c7312e41620","suffix":"0x0"},{"hash":"sha256","prefix":"0x0a2aea8f94a50120","suffix":"0x20a807233f4d3c83321ab80ee8ed08a2ac703793b61a612351dcfc92939a164afb"},{"hash":"sha256","prefix":"0x0c46ea8f94a501206a66a24db521b718e40e22df3afdabc4af0204dee2b8587b34c192bb3cd88ba320","suffix":"0x0"},{"hash":"sha256","prefix":"0x0e8801ea8f94a50120cab16c18f017be51a65f473d5e7bdbd1318cf81665b9878b0cfa3880b76accd820","suffix":"0x0"},{"hash":"sha256","prefix":"0x10c801ea8f94a50120","suffix":"0x202e7118befb4cc724ca9022aaf8cd29de1b065972387f5f4b0b95429545268f5d"},{"hash":"sha256","prefix":"0x12a602ea8f94a5012027808a27bf3490a5c9f4839ccc23f037c3119b0a502d858a4887bc4a69b61ceb20","suffix":"0x0"},{"hash":"sha256","prefix":"0x14bc04ea8f94a501203f5dc44e6e016ee2cf63117652a41e798da305e2ecafe56cd47ee8efb3d283b020","suffix":"0x0"},{"hash":"sha256","prefix":"0x168008ea8f94a50120a93bd924367779b7fd85f74b015aaa32c4fc36a11f253308454f615476e2aa7a20","suffix":"0x0"},{"hash":"sha256","prefix":"0x1ac418ea8f94a50120","suffix":"0x20d2c575d2bb0dbc8ea59ed1cf8dce802d8170f0514f06f0c1ecad3101016f4a46"},{"hash":"sha256","prefix":"0x1eac5aea8f94a50120a93fcfca145b9e58c455e590ccf7bc76d7826558dbbfa8adaf2eac32971b6da320","suffix":"0x0"},{"hash":"sha256","prefix":"0x2096a601ea8f94a50120389e517541f67acc55abf7ee1945f9e080a5fa7a8002616e472788af81b53a8720","suffix":"0x0"},{"hash":"sha256","prefix":"0x22b68702ea8f94a50120","suffix":"0x20cd6795fabe891156a6b60d70bff6b033dc2d35312327246b4b4a490691b1d2b2"},{"hash":"sha256","prefix":"0x24d88f03ea8f94a50120","suffix":"0x20377732534efcee7126a74ca30ab3643c363e6c5f9a0be74479f6866caebf26b1"},{"hash":"sha256","prefix":"0x26a8a606ea8f94a50120513587dd3eb217b8e231cc6d43822a046628e088a57e77c35d6215673d2af6d420","suffix":"0x0"},{"hash":"sha256","prefix":"0x28b6930cea8f94a501205e4f42b1b0cf57bb1b2b98772eae4714501fc1f735c1c5ce90fbad647e10427c20","suffix":"0x0"},{"hash":"sha256","prefix":"0x2ab2f112ea8f94a501208a504d1ba4611b1526146d14e613676d8bebf8564186676d327cc5a6ff5a5cfb20","suffix":"0x0"},{"hash":"sha256","prefix":"0x2cfcdb20ea8f94a50120f1749362b059ed5688a52f32b5f0bd835f74f5002994786e53f75d407384d7ea20","suffix":"0x0"},{"hash":"sha256","prefix":"0x2ea0e74af6f098a50120","suffix":"0x20253279b9158fea6749bb98ec21ce999e8cfaf0b0647886f02eaa397daf308b98"},{"hash":"sha256","prefix":"0x30f6b78e01c0f298a50120","suffix":"0x207bb1ce6f31e87674ccc92d2210ff321615395ab7e838c06e19d8d3109aa9c9e7"},{"hash":"sha256","prefix":"0x3280bab302c0f298a50120a97902da9180c12af3f5f65cad3921ccd5099a3242319abe335ee4777fc1181a20","suffix":"0x0"},{"hash":"sha256","prefix":"0x34c2938104c0f298a5012043bc803a052999e33a56459f26e0852beec39d7d29be134d12288030bff01e1520","suffix":"0x0"},{"hash":"sha256","prefix":"0x36b0908407c0f298a501209c2be38907dde07409e917e8aeadcd80bd7086ded9f2d7416c1f55710cfd92d920","suffix":"0x0"},{"hash":"sha256","prefix":"0x38b895db0bc0f298a501205999211d6315e7d3f3929e894fda0019ef9767c018b600682eb10885e8ee4b2520","suffix":"0x0"},{"hash":"sha256","prefix":"0x3a9cafb617c0f298a5012047f136e2ddcd0ffac9345d8f47c8af2f36a6866dac9315fb532dae42872913e320","suffix":"0x0"}],"value":"0xe54cea436bde55599dbdc4c4a0ba576a24de7e9d49d5d73d54265c2d49f9717c"},"right":{"key":"0x03ee4ea8d358473f0fcebf0329feed95d56e8c04d7cbc4e5fb02c3d1de23a9f1e014b4d2ee5aeaea9505df5e855c9210bf472495af","leaf":{"hash":"sha256","length":"var_proto","prefix":"0x0002a09795a201","prehash_key":"no_hash","prehash_value":"sha256"},"path":[{"hash":"sha256","prefix":"0x0204ea8f94a501207a6af60154738618bb335c98500b898f3ebfc9a6ddb26e5906fbc28c9d9b3d5520","suffix":"0x0"},{"hash":"sha256","prefix":"0x0408ea8f94a50120","suffix":"0x20b70c5b0c267f1fa9e245af7dd065f9c0eaf466889665119ff62082b00f739193"},{"hash":"sha256","prefix":"0x060eea8f94a50120","suffix":"0x2005639b1ead1a1e9aae79b0d5452c43a6b1764203c22f7c1dfb6714841c2c8151"},{"hash":"sha256","prefix":"0x081aea8f94a501201332cc18faa250d45ecad10d1a64e3fce9ba31198ebdaa7bf16f383c7312e41620","suffix":"0x0"},{"hash":"sha256","prefix":"0x0a2aea8f94a50120","suffix":"0x20a807233f4d3c83321ab80ee8ed08a2ac703793b61a612351dcfc92939a164afb"},{"hash":"sha256","prefix":"0x0c46ea8f94a501206a66a24db521b718e40e22df3afdabc4af0204dee2b8587b34c192bb3cd88ba320","suffix":"0x0"},{"hash":"sha256","prefix":"0x0e8801ea8f94a50120cab16c18f017be51a65f473d5e7bdbd1318cf81665b9878b0cfa3880b76accd820","suffix":"0x0"},{"hash":"sha256","prefix":"0x10c801ea8f94a50120","suffix":"0x202e7118befb4cc724ca9022aaf8cd29de1b065972387f5f4b0b95429545268f5d"},{"hash":"sha256","prefix":"0x12a602ea8f94a5012027808a27bf3490a5c9f4839ccc23f037c3119b0a502d858a4887bc4a69b61ceb20","suffix":"0x0"},{"hash":"sha256","prefix":"0x14bc04ea8f94a501203f5dc44e6e016ee2cf63117652a41e798da305e2ecafe56cd47ee8efb3d283b020","suffix":"0x0"},{"hash":"sha256","prefix":"0x168008ea8f94a50120a93bd924367779b7fd85f74b015aaa32c4fc36a11f253308454f615476e2aa7a20","suffix":"0x0"},{"hash":"sha256","prefix":"0x1ac418ea8f94a50120","suffix":"0x20d2c575d2bb0dbc8ea59ed1cf8dce802d8170f0514f06f0c1ecad3101016f4a46"},{"hash":"sha256","prefix":"0x1eac5aea8f94a50120a93fcfca145b9e58c455e590ccf7bc76d7826558dbbfa8adaf2eac32971b6da320","suffix":"0x0"},{"hash":"sha256","prefix":"0x2096a601ea8f94a50120389e517541f67acc55abf7ee1945f9e080a5fa7a8002616e472788af81b53a8720","suffix":"0x0"},{"hash":"sha256","prefix":"0x22b68702ea8f94a50120","suffix":"0x20cd6795fabe891156a6b60d70bff6b033dc2d35312327246b4b4a490691b1d2b2"},{"hash":"sha256","prefix":"0x24d88f03ea8f94a50120","suffix":"0x20377732534efcee7126a74ca30ab3643c363e6c5f9a0be74479f6866caebf26b1"},{"hash":"sha256","prefix":"0x26a8a606ea8f94a50120513587dd3eb217b8e231cc6d43822a046628e088a57e77c35d6215673d2af6d420","suffix":"0x0"},{"hash":"sha256","prefix":"0x28b6930cea8f94a501205e4f42b1b0cf57bb1b2b98772eae4714501fc1f735c1c5ce90fbad647e10427c20","suffix":"0x0"},{"hash":"sha256","prefix":"0x2ab2f112ea8f94a501208a504d1ba4611b1526146d14e613676d8bebf8564186676d327cc5a6ff5a5cfb20","suffix":"0x0"},{"hash":"sha256","prefix":"0x2cfcdb20ea8f94a50120f1749362b059ed5688a52f32b5f0bd835f74f5002994786e53f75d407384d7ea20","suffix":"0x0"},{"hash":"sha256","prefix":"0x2ea0e74af6f098a50120","suffix":"0x20253279b9158fea6749bb98ec21ce999e8cfaf0b0647886f02eaa397daf308b98"},{"hash":"sha256","prefix":"0x30f6b78e01c0f298a50120","suffix":"0x207bb1ce6f31e87674ccc92d2210ff321615395ab7e838c06e19d8d3109aa9c9e7"},{"hash":"sha256","prefix":"0x3280bab302c0f298a50120a97902da9180c12af3f5f65cad3921ccd5099a3242319abe335ee4777fc1181a20","suffix":"0x0"},{"hash":"sha256","prefix":"0x34c2938104c0f298a5012043bc803a052999e33a56459f26e0852beec39d7d29be134d12288030bff01e1520","suffix":"0x0"},{"hash":"sha256","prefix":"0x36b0908407c0f298a501209c2be38907dde07409e917e8aeadcd80bd7086ded9f2d7416c1f55710cfd92d920","suffix":"0x0"},{"hash":"sha256","prefix":"0x38b895db0bc0f298a501205999211d6315e7d3f3929e894fda0019ef9767c018b600682eb10885e8ee4b2520","suffix":"0x0"},{"hash":"sha256","prefix":"0x3a9cafb617c0f298a5012047f136e2ddcd0ffac9345d8f47c8af2f36a6866dac9315fb532dae42872913e320","suffix":"0x0"}],"value":"0x0000000000000000000000007700ea295f62989d2c98721389a521c96fe3c4c0"}}},{"@type":"exist","@value":{"key":"0x65766d","leaf":{"hash":"sha256","length":"var_proto","prefix":"0x00","prehash_key":"no_hash","prehash_value":"sha256"},"path":[{"hash":"sha256","prefix":"0x01337fb01fe848a98e4d7e4f97edd06baae1ab103533b6345917aa8a82d1646032","suffix":"0x0"},{"hash":"sha256","prefix":"0x01","suffix":"0xce1207acd784dfaf0b5007e567e4c925ed0b25b76ebeb4635e0beb937fbe9144"},{"hash":"sha256","prefix":"0x01","suffix":"0x4b666b98c59cf57436bcce3e618780d0d3898d55ba578f196ad058453b507880"},{"hash":"sha256","prefix":"0x01bb7c790383755ebb5c54aa138254059ecf09d6fd4f72bd54456a37946d4fab76","suffix":"0x0"},{"hash":"sha256","prefix":"0x01","suffix":"0x1b11d8cf47f215548b4bf10d7d8bf34cb0aca55faa54d3ded6f7d2675677bfc7"}],"value":"0x07250ac934a747a63f163bc4e64c0a4985e4e3297228c1082045fb3302d75b0d"}}]}"#;

        verify_non_membership(
            // u path client-state 5
            &hex!("05b8ccbb9d4d8fb16ea74ce3c29a41f1b461fbdaff4714a0d9a8eb05499746bc"),
            &MerkleRoot {
                hash: hex!("d0a81718cc318725c92e633106070caf16e51a447ded433f34c7c53df6bada71")
                    .into(),
            },
            &serde_json::from_str(proof_json).unwrap(),
            b"evm",
            &hex!("03" "ee4ea8d358473f0fcebf0329feed95d56e8c04d7"),
            true,
        )
        .unwrap();
    }

    #[test]
    fn verify_membership_v1() {
        // voyager rpc ibc-proof union-testnet-10 '{"client_state":{"client_id":5}}' --height 1062249
        let proof_json = r#"{"proofs":[{"@type":"exist","@value":{"key":"0x03bcf923a74d8b8914e0235d28c6b59e62b547af5ce366c6aafcb006bce7bb3ba40005b8ccbb9d4d8fb16ea74ce3c29a41f1b461fbdaff4714a0d9a8eb05499746bc","leaf":{"hash":"sha256","length":"var_proto","prefix":"0x0002b8e874","prehash_key":"no_hash","prehash_value":"sha256"},"path":[{"hash":"sha256","prefix":"0x0204b8e87420","suffix":"0x2052da534a51a395ea8f9dcb027826f63f89e78c64b049f2d39938ca605d4a4fba"},{"hash":"sha256","prefix":"0x0408b8e87420","suffix":"0x206e4fda3f674cec409a3d036b568d93a0285a6b15ffbfe87a80ea056219c2385f"},{"hash":"sha256","prefix":"0x0812b8e87420","suffix":"0x20f9adbac60693ac583a7f6185832d85fd23bae73acae17da4bc0f7fde6cc095a5"},{"hash":"sha256","prefix":"0x0a26b8e87420","suffix":"0x204431cb50338c43627a09b1a4e6ecd24adffcb54a12137cf3f23fd479c409ea10"},{"hash":"sha256","prefix":"0x0c42b8e87420","suffix":"0x20a7469fd9e9b581c0d4a6ef1d4f242eaa28529399f3392a2ecf9ab90e70c16483"},{"hash":"sha256","prefix":"0x0e8e01b8e87420","suffix":"0x20410369d690f40c021d6af23303b6d98fe080ddd9481c6b93e0269fa51e5e5d5e"},{"hash":"sha256","prefix":"0x10f601b8e87420","suffix":"0x208f058b7391085b6227da0a5128c5eae606c41206b3e8a9cc38c67704132862a9"},{"hash":"sha256","prefix":"0x14f604b8e87420","suffix":"0x20585cd1f265d9d5552e1aa5857c1c156d137d96953eb4b6fa34da0d03bb4dd295"},{"hash":"sha256","prefix":"0x16880cb8e87420","suffix":"0x201654a4b1712201001aed430ddc935b39481c2d6a4c900e0149d6220fe69b142d"},{"hash":"sha256","prefix":"0x1acc1696c27d20","suffix":"0x2036dfca811534f8511fdc5527f8dcf5fefc3339fbf5fb808b17b09e196caae086"},{"hash":"sha256","prefix":"0x1ca22decc87d20","suffix":"0x20be0359ce545b0a7b3de04fec00182484847ab77f41aa5d5a25ff5e972fc04fc9"},{"hash":"sha256","prefix":"0x1eda70a09b7f20c3a71cccd9af6b7621f2fff3e4e3e6c317b4b1aeb3adc71a68898f9cbf28527120","suffix":"0x0"},{"hash":"sha256","prefix":"0x20fed501b8a77f20","suffix":"0x20aa9ae57917b8b93da13776c165b76c8a8200d5c5dc0c0b8d283deb002a706e4b"},{"hash":"sha256","prefix":"0x24facc03b8a77f20","suffix":"0x207e1eb8fa276e037cd586dc9628868e0fd641059ab36acc82935910844bee8d64"},{"hash":"sha256","prefix":"0x26c6cd05b8a77f20","suffix":"0x2024d392fc836b960fade6274552398f1eda9b1b3eda7d299d3d2789583c6e7308"},{"hash":"sha256","prefix":"0x288cc910e2b0810120","suffix":"0x20ff70cf8580f91efb85c059d8b8141924f628be72989cf23f275fce72364041b0"}],"value":"0x39a3a3ca84492f26081c5f63092415bdf673f68317b3a0cb0bc58d5010f901e1"}},{"@type":"exist","@value":{"key":"0x7761736d","leaf":{"hash":"sha256","length":"var_proto","prefix":"0x00","prehash_key":"no_hash","prehash_value":"sha256"},"path":[{"hash":"sha256","prefix":"0x016e2d008451c3b4adb79235593724d7e5899d468e964244c8e06ee3594272d363","suffix":"0x0"},{"hash":"sha256","prefix":"0x01dd4e0f6d93520f83eaaf1a78cc5a3fe9c038a1a8a511de5522a1d9f396059974","suffix":"0x0"},{"hash":"sha256","prefix":"0x01a78bcffeec901cb20425babe8a0606cd57a649c08e7d17da857631fdf832abf6","suffix":"0x0"}],"value":"0x5ca2ca57657312f1153d455577937d4b1b6c0dde8ebc47f5b96547a091d02217"}}]}"#;

        verify_membership(
            // u path client-state 5
            &hex!("05b8ccbb9d4d8fb16ea74ce3c29a41f1b461fbdaff4714a0d9a8eb05499746bc"),
            &MerkleRoot {
                hash: hex!("e1dafeac09a9557b30cedee432a089fbe8700b472e52731c0b958de0158fe58e")
                    .into(),
            },
            &serde_json::from_str(proof_json).unwrap(),
            // voyager rpc client-state union-testnet-10 5 --height 1062249 | jq .state -r | u hex -d | cast keccak
            &hex!("39a3a3ca84492f26081c5f63092415bdf673f68317b3a0cb0bc58d5010f901e1"),
            b"wasm",
            &hex!("03" "bcf923a74d8b8914e0235d28c6b59e62b547af5ce366c6aafcb006bce7bb3ba4" "00"),
            false,
        )
        .unwrap();
    }

    #[test]
    fn verify_non_membership_v1() {
        // voyager rpc ibc-proof union-testnet-10 '{"client_state":{"client_id":50}}' --height 1062249
        // client 50 does not exist at this height
        let proof_json = r#"{"proofs":[{"@type":"nonexist","@value":{"key":"0x03bcf923a74d8b8914e0235d28c6b59e62b547af5ce366c6aafcb006bce7bb3ba40000bcd6ff29ae71d399fb597d99792fa72d0863bd723b9ab11f79d0b8d8ac5bc8","left":{"key":"0x03bcf923a74d8b8914e0235d28c6b59e62b547af5ce366c6aafcb006bce7bb3ba40000bcc37fcf9a6359e224f5a98a6b7ad536bf16a0be17515318fab6e727e40fe6","leaf":{"hash":"sha256","length":"var_proto","prefix":"0x0002eca134","prehash_key":"no_hash","prehash_value":"sha256"},"path":[{"hash":"sha256","prefix":"0x02049ca26c20","suffix":"0x202eb34974e234275cb48c7d68fb95fd5ff1580b135dc87b208d21290eb436273a"},{"hash":"sha256","prefix":"0x04089ca26c20","suffix":"0x20a7ea1e51b1fc8fea396affd1aee5446b67aa46613942bc032e5fc8b5713daf11"},{"hash":"sha256","prefix":"0x08129ca26c20","suffix":"0x2011ba68069ae11abb1a192bb5cfab12627b70ce7c2ea81ba8c0ab6ce72b320b36"},{"hash":"sha256","prefix":"0x0a28f4fb7520","suffix":"0x20895099a9f6184f21ecc024e4d02ce5195d6ba8ddb4ea875885492324a264f264"},{"hash":"sha256","prefix":"0x0c50f4fb7520","suffix":"0x20048d228943e32b87c22be5a1cb48712956c0e7b281b299e3baf4d69b4581db94"},{"hash":"sha256","prefix":"0x10c401f4fb7520","suffix":"0x204f8682b634191b0025c43de1623f5eb7304df37114d6f53ce8a71ed82913a9a6"},{"hash":"sha256","prefix":"0x12c802f4fb7520611e2ae83fa0ba4bb5e8407a69a0394719a08b54ed4d89a748ecf53607043a9220","suffix":"0x0"},{"hash":"sha256","prefix":"0x149205f4fb75206428050fd9e740ffa6c1992fd0ec2607337f46d74db23c312f30e9baccb3682720","suffix":"0x0"},{"hash":"sha256","prefix":"0x169609a09b7f20bd1d34816b075dc0caf6cbfac5218641c1f29f9e4ad62a892cd5a4e57502d9b420","suffix":"0x0"},{"hash":"sha256","prefix":"0x18b011a09b7f20","suffix":"0x20655fd569f35193feb27781578067c57b7b58add8930bbb7cdc73ed4c79c164e7"},{"hash":"sha256","prefix":"0x1aa81ca09b7f20","suffix":"0x202e7bccf4d6ce3f11ac2111ab38772dac9f0aabf2df24f207d9937d9dd9ced84c"},{"hash":"sha256","prefix":"0x1cb843a09b7f20e4db405b3f563ff82d2cda0b7b055713ad71ec3e53b63b61783bde16e4dcd31520","suffix":"0x0"},{"hash":"sha256","prefix":"0x1eda70a09b7f20","suffix":"0x20bbd220f976dd41601fc8e5194e59d27a89e1307a6c7e6dc81339853a825d391f"},{"hash":"sha256","prefix":"0x20fed501b8a77f20","suffix":"0x20aa9ae57917b8b93da13776c165b76c8a8200d5c5dc0c0b8d283deb002a706e4b"},{"hash":"sha256","prefix":"0x24facc03b8a77f20","suffix":"0x207e1eb8fa276e037cd586dc9628868e0fd641059ab36acc82935910844bee8d64"},{"hash":"sha256","prefix":"0x26c6cd05b8a77f20","suffix":"0x2024d392fc836b960fade6274552398f1eda9b1b3eda7d299d3d2789583c6e7308"},{"hash":"sha256","prefix":"0x288cc910e2b0810120","suffix":"0x20ff70cf8580f91efb85c059d8b8141924f628be72989cf23f275fce72364041b0"}],"value":"0xb6df4390ce7d8fe413c63ef1b0c5aef8ee440e92dc0d4b6db954474561efa23d"},"right":{"key":"0x03bcf923a74d8b8914e0235d28c6b59e62b547af5ce366c6aafcb006bce7bb3ba40000bd518758fe343bf3358bb5f731ccea45a1fd8f206fd87459c99b8111ec28b9","leaf":{"hash":"sha256","length":"var_proto","prefix":"0x00029ca26c","prehash_key":"no_hash","prehash_value":"sha256"},"path":[{"hash":"sha256","prefix":"0x02049ca26c2069ceca6497dda3313ee48cc50e746fcbf0c8e752e90c2412b3451800180610c020","suffix":"0x0"},{"hash":"sha256","prefix":"0x04089ca26c20","suffix":"0x20a7ea1e51b1fc8fea396affd1aee5446b67aa46613942bc032e5fc8b5713daf11"},{"hash":"sha256","prefix":"0x08129ca26c20","suffix":"0x2011ba68069ae11abb1a192bb5cfab12627b70ce7c2ea81ba8c0ab6ce72b320b36"},{"hash":"sha256","prefix":"0x0a28f4fb7520","suffix":"0x20895099a9f6184f21ecc024e4d02ce5195d6ba8ddb4ea875885492324a264f264"},{"hash":"sha256","prefix":"0x0c50f4fb7520","suffix":"0x20048d228943e32b87c22be5a1cb48712956c0e7b281b299e3baf4d69b4581db94"},{"hash":"sha256","prefix":"0x10c401f4fb7520","suffix":"0x204f8682b634191b0025c43de1623f5eb7304df37114d6f53ce8a71ed82913a9a6"},{"hash":"sha256","prefix":"0x12c802f4fb7520611e2ae83fa0ba4bb5e8407a69a0394719a08b54ed4d89a748ecf53607043a9220","suffix":"0x0"},{"hash":"sha256","prefix":"0x149205f4fb75206428050fd9e740ffa6c1992fd0ec2607337f46d74db23c312f30e9baccb3682720","suffix":"0x0"},{"hash":"sha256","prefix":"0x169609a09b7f20bd1d34816b075dc0caf6cbfac5218641c1f29f9e4ad62a892cd5a4e57502d9b420","suffix":"0x0"},{"hash":"sha256","prefix":"0x18b011a09b7f20","suffix":"0x20655fd569f35193feb27781578067c57b7b58add8930bbb7cdc73ed4c79c164e7"},{"hash":"sha256","prefix":"0x1aa81ca09b7f20","suffix":"0x202e7bccf4d6ce3f11ac2111ab38772dac9f0aabf2df24f207d9937d9dd9ced84c"},{"hash":"sha256","prefix":"0x1cb843a09b7f20e4db405b3f563ff82d2cda0b7b055713ad71ec3e53b63b61783bde16e4dcd31520","suffix":"0x0"},{"hash":"sha256","prefix":"0x1eda70a09b7f20","suffix":"0x20bbd220f976dd41601fc8e5194e59d27a89e1307a6c7e6dc81339853a825d391f"},{"hash":"sha256","prefix":"0x20fed501b8a77f20","suffix":"0x20aa9ae57917b8b93da13776c165b76c8a8200d5c5dc0c0b8d283deb002a706e4b"},{"hash":"sha256","prefix":"0x24facc03b8a77f20","suffix":"0x207e1eb8fa276e037cd586dc9628868e0fd641059ab36acc82935910844bee8d64"},{"hash":"sha256","prefix":"0x26c6cd05b8a77f20","suffix":"0x2024d392fc836b960fade6274552398f1eda9b1b3eda7d299d3d2789583c6e7308"},{"hash":"sha256","prefix":"0x288cc910e2b0810120","suffix":"0x20ff70cf8580f91efb85c059d8b8141924f628be72989cf23f275fce72364041b0"}],"value":"0x88dbb543f1301c22f03bf140db2f69c9a29bbefcf98ed56e7ac6edae868179dc"}}},{"@type":"exist","@value":{"key":"0x7761736d","leaf":{"hash":"sha256","length":"var_proto","prefix":"0x00","prehash_key":"no_hash","prehash_value":"sha256"},"path":[{"hash":"sha256","prefix":"0x016e2d008451c3b4adb79235593724d7e5899d468e964244c8e06ee3594272d363","suffix":"0x0"},{"hash":"sha256","prefix":"0x01dd4e0f6d93520f83eaaf1a78cc5a3fe9c038a1a8a511de5522a1d9f396059974","suffix":"0x0"},{"hash":"sha256","prefix":"0x01a78bcffeec901cb20425babe8a0606cd57a649c08e7d17da857631fdf832abf6","suffix":"0x0"}],"value":"0x5ca2ca57657312f1153d455577937d4b1b6c0dde8ebc47f5b96547a091d02217"}}]}"#;

        verify_non_membership(
            // u path client-state 50
            &hex!("00bcd6ff29ae71d399fb597d99792fa72d0863bd723b9ab11f79d0b8d8ac5bc8"),
            &MerkleRoot {
                hash: hex!("e1dafeac09a9557b30cedee432a089fbe8700b472e52731c0b958de0158fe58e")
                    .into(),
            },
            &serde_json::from_str(proof_json).unwrap(),
            b"wasm",
            &hex!("03" "bcf923a74d8b8914e0235d28c6b59e62b547af5ce366c6aafcb006bce7bb3ba4" "00"),
            false,
        )
        .unwrap();
    }
}
