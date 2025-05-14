use blake2::{Blake2b, Digest as _};
use cosmwasm_std::{Deps, Empty, HashFunction, StdError, BLS12_381_G2_GENERATOR};
use depolama::{KeyCodec, Prefix, Store, ValueCodec};
use ibc_union_light_client::{ClientCreationResult, IbcClient, IbcClientError, StateUpdate};
use ibc_union_msg::lightclient::Status;
use serde::Serialize;
use sui_light_client_types::{
    checkpoint_summary::CheckpointSummary,
    client_state::ClientState,
    committee::Committee,
    consensus_state::ConsensusState,
    crypto::{AuthorityStrongQuorumSignInfo, BLS_DST},
    header::Header,
    object::TypeTag,
    storage_proof::StorageProof,
    AppId, Intent, IntentMessage, IntentScope, IntentVersion, U64,
};
use unionlabs::encoding::{Bincode, DecodeAs, EncodeAs};

use crate::error::Error;

pub enum SuiLightClient {}

impl IbcClient for SuiLightClient {
    type Error = Error;

    type CustomQuery = Empty;

    type Header = Header;

    type Misbehaviour = Header;

    type ClientState = ClientState;

    type ConsensusState = ConsensusState;

    type StorageProof = StorageProof;

    type Encoding = Bincode;

    fn verify_membership(
        ctx: ibc_union_light_client::IbcClientCtx<Self>,
        height: u64,
        key: Vec<u8>,
        storage_proof: Self::StorageProof,
        value: Vec<u8>,
    ) -> Result<(), ibc_union_light_client::IbcClientError<Self>> {
        let client_state = ctx.read_self_client_state()?;

        let consensus_state = ctx.read_self_consensus_state(height)?;

        sui_verifier::verify_membership(
            client_state.ibc_commitments_object_id,
            key.into(),
            value.into(),
            storage_proof.object,
            storage_proof.transaction_effects,
            storage_proof.checkpoint_contents,
            consensus_state.contents_digest,
        )
        .map_err(Into::<Error>::into);

        Ok(())
    }

    fn verify_non_membership(
        _ctx: ibc_union_light_client::IbcClientCtx<Self>,
        _height: u64,
        _key: Vec<u8>,
        _storage_proof: Self::StorageProof,
    ) -> Result<(), ibc_union_light_client::IbcClientError<Self>> {
        Ok(())
    }

    fn get_timestamp(consensus_state: &Self::ConsensusState) -> u64 {
        consensus_state.timestamp
    }

    fn get_latest_height(client_state: &Self::ClientState) -> u64 {
        let ClientState::V1(client_state) = client_state;
        client_state.latest_checkpoint
    }

    fn get_counterparty_chain_id(client_state: &Self::ClientState) -> String {
        let ClientState::V1(cs) = client_state;
        cs.chain_id.clone()
    }

    fn status(
        _ctx: ibc_union_light_client::IbcClientCtx<Self>,
        _client_state: &Self::ClientState,
    ) -> Status {
        Status::Active
    }

    fn verify_creation(
        _caller: cosmwasm_std::Addr,
        client_state: &ClientState,
        _consensus_state: &ConsensusState,
        _relayer: cosmwasm_std::Addr,
    ) -> Result<ClientCreationResult<Self>, IbcClientError<Self>> {
        let ClientState::V1(client_state) = client_state;

        let Some(initial_committee) = &client_state.initial_committee else {
            return Err(Error::NoInitialCommittee.into());
        };

        let mut client_state = client_state.clone();
        client_state.initial_committee = None;

        Ok(ClientCreationResult::new()
            .overwrite_client_state(ClientState::V1(client_state))
            .add_storage_write::<CommitteeStore>(
                initial_committee.epoch.0,
                initial_committee.clone(),
            ))
    }

    fn verify_header(
        ctx: ibc_union_light_client::IbcClientCtx<Self>,
        _caller: cosmwasm_std::Addr,
        header: Header,
        _relayer: cosmwasm_std::Addr,
    ) -> Result<StateUpdate<Self>, IbcClientError<Self>> {
        let ClientState::V1(mut client_state) = ctx.read_self_client_state()?;

        let committee = ctx.read_self_storage::<CommitteeStore>(header.checkpoint_summary.epoch)?;

        verify_signature(
            ctx.deps,
            &committee,
            &header.checkpoint_summary,
            header.sign_info,
        );

        let consensus_state = ConsensusState {
            timestamp: header.checkpoint_summary.timestamp_ms * 1_000_000,
            content_digest: header.checkpoint_summary.content_digest,
        };

        let mut state_update =
            StateUpdate::new(header.checkpoint_summary.sequence_number, consensus_state);

        if let Some(epoch_ending) = header.checkpoint_summary.end_of_epoch_data {
            state_update = state_update.add_storage_write::<CommitteeStore>(
                header.checkpoint_summary.epoch + 1,
                Committee {
                    epoch: U64(header.checkpoint_summary.epoch + 1),
                    voting_rights: epoch_ending.next_epoch_committee,
                },
            );
        }

        if client_state.latest_checkpoint > header.checkpoint_summary.sequence_number {
            client_state.latest_checkpoint = header.checkpoint_summary.sequence_number;
            state_update = state_update.overwrite_client_state(ClientState::V1(client_state));
        }

        Ok(state_update)
    }

    fn misbehaviour(
        _ctx: ibc_union_light_client::IbcClientCtx<Self>,
        _caller: cosmwasm_std::Addr,
        _misbehaviour: Self::Misbehaviour,
        _relayer: cosmwasm_std::Addr,
    ) -> Result<Self::ClientState, ibc_union_light_client::IbcClientError<Self>> {
        todo!()
    }
}

fn verify_signature(
    deps: Deps,
    committee: &Committee,
    checkpoint: &CheckpointSummary,
    sign_info: AuthorityStrongQuorumSignInfo,
) {
    // TODO(aeryz): VERIFY QUORUM
    let pubkeys = selected_public_keys
        .into_iter()
        .flat_map(|x| x.0)
        .collect::<Vec<u8>>();

    let aggregate_pubkey = deps.api.bls12_381_aggregate_g2(&pubkeys).unwrap();

    let hashed_msg = deps
        .api
        .bls12_381_hash_to_g1(HashFunction::Sha256, &intent_msg_bytes, BLS_DST)
        .unwrap();

    let valid = deps
        .api
        .bls12_381_pairing_equality(
            sign_info.signature.0.as_ref(),
            &BLS12_381_G2_GENERATOR,
            &hashed_msg,
            &aggregate_pubkey,
        )
        .unwrap();

    assert!(valid);
}

pub enum CommitteeStore {}
impl Store for CommitteeStore {
    const PREFIX: Prefix = Prefix::new(b"committee");

    type Key = u64;
    type Value = Committee;
}

impl KeyCodec<u64> for CommitteeStore {
    fn encode_key(key: &u64) -> depolama::Bytes {
        key.to_be_bytes().into()
    }

    fn decode_key(raw: &depolama::Bytes) -> cosmwasm_std::StdResult<u64> {
        raw.try_into()
            .map_err(|_| {
                StdError::generic_err(format!(
                    "invalid key: expected {N} bytes, found {}: {raw}",
                    raw.len(),
                    N = u64::BITS / 8,
                ))
            })
            .map(u64::from_be_bytes)
    }
}

impl ValueCodec<Committee> for CommitteeStore {
    fn encode_value(value: &Committee) -> depolama::Bytes {
        value.encode_as::<Bincode>().into()
    }

    fn decode_value(raw: &depolama::Bytes) -> cosmwasm_std::StdResult<Committee> {
        Committee::decode_as::<Bincode>(raw).map_err(|e| {
            StdError::generic_err(format!("unable to decode {}: {e}", stringify!($ty)))
        })
    }
}

/// Calculate the object_id of the dynamic field within the commitments mapping
fn calculate_dynamic_field_key(parent: [u8; 32], key_bytes: &[u8]) -> Vec<u8> {
    #[repr(u8)]
    enum HashingIntentScope {
        ChildObjectId = 0xf0,
        RegularObjectId = 0xf1,
    }

    // hash(parent || len(key) || key || key_type_tag)
    let mut hasher = Blake2b::<typenum::U32>::default();
    hasher.update([HashingIntentScope::ChildObjectId as u8]);
    hasher.update(parent);
    // +1 since `key_bytes` should be prefixed with its length (bcs encoding)
    hasher.update((key_bytes.len() + 1).to_le_bytes());
    // instead of calling bcs::serialize, we just prefix the bytes with the its length
    // since the table we are verifying uses `vector<u8>` keys
    hasher.update([key_bytes.len() as u8]);
    hasher.update(key_bytes);
    hasher.update(
        bcs::to_bytes(&TypeTag::Vector(Box::new(TypeTag::U8))).expect("bcs serialization works"),
    );
    let hash = hasher.finalize();

    hash.to_vec()
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use cosmwasm_std::testing::mock_dependencies;
    use hex_literal::hex;
    use serde::Deserialize;
    use sui_light_client_types::{
        checkpoint_summary::CheckpointContents,
        digest::Digest,
        object::{MoveObject, ObjectInner},
        AccountAddress, CertifiedCheckpointSummary, SuiAddress,
    };
    use unionlabs::{encoding::Decode as _, primitives::encoding::Base64};
    use unionlabs_primitives::{
        encoding::{Base58, HexPrefixed},
        Bytes, FixedBytes,
    };

    use super::*;

    #[test]
    fn update_client() {
        let committee = Committee::decode_as::<Bincode>(&Bytes::<Base64>::from_str("2gIAAAAAAABzAAAAAAAAAIASfHDDJ2wBovXbbf7U6KslzbJ8RzTKSnjDYzk6Wj1Xv5b9e7mAt5rE9OF1kfdq1AT/btmcb+TyCQuBojtTojGPZFW+/2xaeb/hDSC7KN7Fi82NlQv0JrxqD3VSUZg99jEBAAAAAAAAgRyd7ILql8CBVNtvEa0osVPVlALHFwhPzKold3cKaAoL0/jTqzrRo3W1oqzOy5BgARhTuBY/oQ1duI84eThZKllap3bsSwPc9LYNc4ofUB4/qjOefIBStjPgTw0Xzu1qJQAAAAAAAACBYm0D/HEzjW5ys/3r2RFQEksMBjF1T53PMSjYCOqzlvx8lqC2naHfjuRK08L1X0MN/bGClAitq0gaxDm5aASIk0nRL7M3Nsg8bfQuJJ1nR+kF15aN91I1HSlsNNKfnL0mAAAAAAAAAIHYwHlNOGs9qpymMXYfa+Dp+O8zerFPOmGjDjVoGCV9d1jfozxnyarA8OjQ91qwRwpV0Il9ot+GtXtA7y2DK+4Vv6j70pldcrS4KKJeHfDXDEtvvKOgjTE54lZkpRfZL1gAAAAAAAAAgqI/f7PigATa6E12BJOfbSsBCgP/HFVjmyAkiP1vzX2w8AFliV2YjfOUggyyRdZGD9pdcpQb5MkHlRfv1hXpOrLsKYqK06tO/hD33ewAURal7vhFpRaef/EFpJfXZTsJegAAAAAAAACDWLjA+LgyZ6wJ65w8/f4Z/lr7kApw2j/SYujV1ix7paajRrP5hta4wcATwHalxpcS3D17Y4Fl1Sp6MVHGPesH15ZEqLn27HN5mW8UelnojaPrTaEKLHS5fC+wcOCd6RYeAAAAAAAAAIQp3sSaK6K3VBuRQX0EqN6Wo7n4s6XwuRxfQMrKmH/ZPHzE6SKEYMLoF1liziYMIQewgl/Hp6ci1LZHq5H2ItLX+DEXxvzkZbTBEz/3Mohd7nCATqTtLyT2h9/LmKfrqDYAAAAAAAAAhN33X1uYqEN9IclBMs1/nIAGPQasZePB5ZGXlkfOR3uhLQY8Ou57u9u6vEnkRsq4Fsim0ZPeXBMzG4+lSawMTBTOTRR2RXhfzp0c5wrRWe0pyl1xjM+7wTh0FomjgmjPNgAAAAAAAACFx40KwTC4h/7fLK2a/u98vBoQdEtUt8KL2tr3TmzvlvB0ox7b9HGbahcyWtcUjpoRWpMXHp4CuT4R+ElnZI63L+pMfyOH+AGWaTBAci3rU64zJD4mokAf89e0cMOVJT56AAAAAAAAAIYOYRYAE+ONmEcIRyNi10SHUll4yasfzIWUaD6nqIopXj+fhej3F8Jcixh5HFRQgQvC6WGEoAXzTdEg4IOOCh+UMKCUsHqaSToI3bVCORT7j7LiGxMMh9I1mayVxh6ttJkAAAAAAAAAhhir3LhIy0cD8VOHbne085lpMoQeu08MaGxjA2xI4fvPnnk4X/GKUY11rHCej9a1FaBUVGJtwDmyAOCzm1JnvTaCP14uvouEQtL9U4jcTLFd/Xa7bZ1ZGCkqXJn7bmhtTQAAAAAAAACGi/sgJhAZbzZ1Curo7ZAMecfkpI7JlISr42vHweHZQzE/JBWnwgoGHeBmxFUrfxgHMjHASoTTUXU5+WXZSiF2O71tDZNprkjAx8qH+uqmyuu3acn8/pv1QaTnp7lXAAceAAAAAAAAAIanj8KMbOeg/ST+KIHA/xZCPpeNkL44FmyX+u8fjIIB1Pcc0ba1RuNyzAsESGot5wJRVY7p3SFnr7aIelt9Z8fsGMTdBCI6paOOMEHpYISWO51ofG8bezNIoZJTXCx0TCYAAAAAAAAAhsKTpuy3K7ii/mDgLjpDbxB90zyE2UsFBseYDTUPyXlFuKX5KzcHOqS5TGpIg/EBBtw3hmA6pGW+4uZ+yU7h4jh/MMSxC5AExfxAmGlC/rQ5nv7igWWMMd+hl3Toa3csmQAAAAAAAACHMcVdXsHEXwa5fqw7TsqaAwANKgBPwuJTKs97IFttXfe7uFJw4VGyLCcaWXd5F2sDwdY4e0go4io9MBgQp22693CKfA4jW9FB/gEdcMJ6/oNM3c51nO+0sBKx/uqlCx4mAAAAAAAAAIeN3GySwPI5aNtIjxcJ9ruHTWOlbiXR6y7LmqTAN8NmwvBS7aZwVyj/WNTlR3PXoAPBMe9wKITvcPkIqVr2bZQXXRB5AFz1aYWq+QbyBOgu4RLbhbzRjdW6cN9FPnYbVjkAAAAAAAAAiHN8bU+ek8GBukQM0ysOl0wB/faNdNAEj/vTK5HxBOzpvmEUVnSy3Upl4ILxH6vtCqhZaiLUpSxY0Xwu/IQN6a63eDv3q3BoyzhLqMmjiuaXEhgHGo5vfskLN6xCZAppWAAAAAAAAACIpx0wI+GMv7Ow+YluxrDKk2VqZIOuPDFUFB9a0onGsA6QF28U6VHdfYssUkQa47sVlRfn8CiFpUm1EcE0V+IepZeKnLAFLDVm4FAfpMxCd57keSDrdA7wcTBFhnuIU/oDAQAAAAAAAIjL6wDtToQPLEqUu64hxm3kumqz1W2jP1NHQ5trMq4aYerT+g1vbLWB/MM/KYeyVglvqPFJZq2EykjeP/NUo4GN/qQ7/28byTsY9aCivOFRSAHEvzbxJc5kXaT0CzR/cR4AAAAAAAAAiVVAJCIzaoLT0d+M2wfyWCvCSHPngXiUH0yqmhqyBs0jQHEJPg/KEA1tSUNh8H/QAVxJSKr9W89qOibWmt4yBVTfD9rLIPQuAAyYi9gWWJUxO3DHM5QWnNwgFMwp4j0pHgAAAAAAAACK7WOdiZF8gZBgcu9EzUEGQzz9P2J8YgMOlyug3xCScKOSbTFIaTQ0VRXtZiSUWBYIFH97Mm/7/moyQ5dCsdEoVPeP4KHfkaS0MRyqxrIBawBmdJLICb2+GshsXGN0HRseAAAAAAAAAIsP3GOjvwz7ksaLY+lxeJoqhSowwUNNChwDgD0+H0XiAoV8dtR7s/jWrIhw4jKjmRQCI6vm/+CgCvDHPsEeLerQbUdNnVb/55ZEH2SBtyttpPUPb7uG1emWFAvJpDpzazYAAAAAAAAAi5mbjTxR5/4pXx51a+yBVzbEX9aexB60R7OeDUTM5ECad0mxSEhBaQsl/CFjOkYQEF6YEfpXfmLSwyhosH/wHn1mpTfd0sraWlLKCPKuSAslinHwKzO4uT/KlImzoDhWJgAAAAAAAACMd3H++gFMOHfNsKolRvU7DM5zqvZzRQk6SU9W6aHYYrOXc+8JH9V+Y0mIdnlXXxcRme2pz/Q53a1hyXkA7g0hcnaOjJ31rAeZbA+aDXx9S2Jo7NYo5ZMKVExR+4t/ufUlAAAAAAAAAI1jwz0vzkAv0zV/OfTCJIVaNM8XVjcUti2ed5HqcXt/3C+y7CySh3c9u8XVXdlzrBciQ9DRsK4oeGoTO4kp4Neb9E0+q5JGjw0OIJXcZyZ5uGNTSAQ8OYVk/1c37Po/2JEAAAAAAAAAjnrPOaK6yBBMguFHGfSr9YanfRJL6Offg2N8MfOJwhmOafNxEF0H+hrLpQnuDsKLAPSKJE1U77rs2sPb99V1a+LaL+1JzyyVzn0kBiFuXNrxDPWu3Q7vOgUJuaJRhL1bKAEAAAAAAACOp++KiaoEyXUQnL3VzFqbsCyJYsYuWgw7xmVokTZReZeMfgC0dBd8/+72KxNTOkgKBjhD610qccJeKrhBPPTQiagaj9l9B8MguOc4y5Ol8rsCHPBslqQ8MPj4//8eMpgeAAAAAAAAAI7EvFmAIE4k2+aEEIOkQj1H0RzEGqyWjK/Po2MA8Yjwna1vjxzwK3c/iwfM/WG2mhCW16KaL/9VxEuMBKa7FoC6KVm/h5Dy3hthz0gYX6BjhNmSEoSoEGpRuWJ3UeZ0JCUAAAAAAAAAjzaTRuYjJQYOWu6iSaCvtVf/69ArN2EenlJthPdxHKZycM+jMDQnsUfg3+jxRNP8DaqXS8F/mPVMj2uZ9nnMVvvt/GQuvXMQ50LxQkAM4uUWtvl0BZfJvFOtcY2g5ZecegAAAAAAAACP9b3gqQ0iE+duyK+yWyqIcBXC5aHUF71Aw560ftCZZ94cZHT3pCqATLdvabXjgUAV/KUOMIy8c5SY8yp5THQD+TGs3W0m3lhG0SmJoyKJ/fuzUDcurj6WpFkqB6RO/JMrAAAAAAAAAJEqv9IMspxIy8vyEUOSEnmV6Fgcx3RyzrSXWnLGYz2gf6NWb17zJ5ccs1nflLuXwg9gMSBGg8rjEBdJXKKPYgyQaQU4YFoLZ+skwj0vgFcYzl9VgbJeoV4j35LnFtnkSjYAAAAAAAAAkSrb5TASIwS0LkW6HZQqn+pq14ahV6ZG6bLp5RYTzt+8OsiVuVdcqYWmbBXwhddNChFxM+khPy6qgJBiPl46MF4wUub/pqP3tabV3HWcC9vNfscE/oXYleLyqUOwpWyQLAAAAAAAAACRY0lza+yoELZ63xsgBukSdE0DbCd9CepsZj1CItCBr856PgpTXAQq1+7iNs0NST4A553sjQvbY5CJnEKEqB/P3ulW0q1HjoLVtc7ox8dDiDJlh9YE2LYvozEiwsB0CAweAAAAAAAAAJJRTnbk0IlzbsWbYljNMFYs0maQZ84u7UG2dqLQ8uqe4uhYKuRRGbr1K2tJGjKeAQSVulDWYbrvfXCF4srhYltYY1kTE6LLkQ9+WvGm3fxNs3phAdYKjvtbykNYgOUzKCsAAAAAAAAAkl76hkg5t8BopiXVqjiihTU8iDr3R7r8aI0z+hFaFwTtQMZD9PHXXpwcJ/iaVx2fCNKUgmDLk0zoggq/6NxjICJu+0aa4xZV07dMBBKsRGFmrm7BzKCfqg0KO7l7/j+regAAAAAAAACS22yETO8omHkURsoYqeO5QXvR4J7HJgOSE9SM8aznm8uaG4GvMSK6VW10EkBkiK4YmwkyeaTK5t25r9UxDrxp2wBEjAu1Kgj6TmK1DVbTMc3a1NhL26HdH9GnyuDJgziCAAAAAAAAAJQh6DFXz6W4hljrrid7G9zqPVMQNZM/io0gCkuG2ktlh0CwOqyhQNDsDySUOEBODxSOxRE9x6bQkkXR4t8tyIuVlq54lp1Vem84wLyyqpITUANI212rJx0zhB9br6e1gyYAAAAAAAAAlGnmFH0X+Q4CaSgfyvQSJxeCEAhhTzcjA/7k3I/S9hlfzGIIGloesjm8skqoIDeVCjWOXnPipr50Fvm0g+lNXLwHa1dSc+SJrAA4Z2hNwoIfJYzAdCm7AYlVD5pPMGa9NgAAAAAAAACUt3yZTqc+wrYuC0Bo9etyemOT8sBh4vUVG8rjbvm7j1NWqZxvmUp59rtxrT4g0N8IP7lGvUzJPm2/encjJZeHA8iIGcM5L0eUm4v0kZ4nwCEZHgpWJWXlkM3GIjNQvhseAAAAAAAAAJTStrN1q2tZ4BgBNTLYFUqtam4zFjEovoHu7KwrFxz0VXFWYWMvgYh1jfJt+zP8bAHfHPbw8xEjfANPMFOQUGocF955t9J4PgIV9+bngo9X0x3eVSR75S74RGcBDwuTI1oAAAAAAAAAlNMiVCuJciLJ28HX9fBX1mOYxxRsJdSMzLGvkZdBA5U1dRadM7ICqK6QFyxjgqE+BLEv2YQSpkSvIdZWAW9m1IORlH0gv7iabtnVsVyjPDPyuXhwF9jqUDA+qkbPUDBeegAAAAAAAACU6cpEoEOLIV3629bKmcOFupd7boPCRfCip1txGyOUf96Wm0f6bo3PANOXqquTWYYN5BS53wVNHt/znH6hxcZW6dEKejNgEBAG9Vqoo9leq0zxGhAiEXGrV3ug4bg1WvEmAAAAAAAAAJZePPt9ZcIKCU21vK2ukNC29ljl6NKSwvEW+2koMGkdhQsLbVJd1iBGTn6t7WucSAOi+6DrjfG5Z7NXgYMfcijGG+OPYYPKGFnvx+pxddWGjpAKP2+tiCnY26ZUsVhP6R4AAAAAAAAAmBxTdpbftXk1pPXPIwXT2JRS0cVB0JqLQ9YjrtTOSd7ejXx4B0IEXtVjjJYy56FXC2F6jTQPVcbHPM4Gx96mYA35W+gTrEAIMwdSyA1g9/9IbpTQcTqJFwOb3P26ksYwNgAAAAAAAACYal+HuPz1EM5NCZ+Y5kJ0+0oC1XljHDaGxWKShfKotyS1eOixS8QPX7P8sD4anD4SsU+zL6WoNz9/pjAslY1wk9hOzvivvHJAjBSn3bTqrjyzzuxXf6VUzieCZSErNiuaAAAAAAAAAJiE5OUBAzMGXySrabkI3Gl+euJ7vnXM7bcPYRRelRi/LmQvvNbMpGU+rAi2olqkXxGGJvZ0QCTytfLApCkU6jDzeGzltgkTpO+ZIRLvUQ/sloxYokZsWBnz8NHYN7HMwB4AAAAAAAAAmIo7dv8nr8yClkGH31E6LaAcQdOqkDM2oUsVwovfNKzae/e8uzsBlPuj8yyxom3nEl/s5dXjrd40uokwq0CmlcR97x6Idea+9DOtMlxGF2ZU/o6O/og9/EJ+3z6Yx8uXHgAAAAAAAACYlZ6je07p4413kNxhR5aQLm4OZNC+BdJCm0Eharkd79iu0xSr145HeN5GIETyvGIZwGyWE+Inzo40cm9ylbCfHIjECF6rPxRUNON95+w/BMeTt9gcOy+z8mY2Nr3N+4VYAAAAAAAAAJlE9Cw6EfHtDPLTKfmtRjtJam6jTdbJ0At7ZOaXE3F3+Th+c0MlmmSQkWn4CNzW5w/Vv/pf5KJeQzyuoXggbZZNZwQBzrQV+t8h3u4LH61cdzD/L7CNtK1lw/ay1R4mJyYAAAAAAAAAmbpO2r14Df+O6fJ4Ou2GrF4DbTh5TBAaZPmXm3KLrfHB1LrBuS972JDGqAd0dC6eFSulgytQzwbLf5gqKVVkGb2FtRtEHN8G6Z4t9O+s+k9M2zcAjVO5Bh/rdJ6IC615egAAAAAAAACZzq7f5hhpviLH+Rx8lFEsFu5qZvVOcZ07LoiKJmZhf8Kv1jj0894VqHT4oPaTFoYWclhl7+Lv1Ssdd7rYzI69GOGO/ALWiOFDOPm3zItQDvojmJoq1SdUsAA6GDut5zEeAAAAAAAAAKAlGsFHgqGsEmeL61GEST7R2BuU+Ijx74bagfDmiOTZi+DUZ4HnaYGtW9OA5acR3wGMbe0qKW38han0T6HaGhzlIjc9McLfBAR4mcgWPQ+pqgQCFlwFoGTo2gAIXAV9dSsAAAAAAAAAoWtN4zVptwEQ61XJiKV3jkk1Ijhytp4iyLpDD/eXGfuxlQ40dCL9NEf4CTKlTQJzF/dD6C3N33ov8BSw8JVuz3SVSh2VqgueW2W85+Ws9MUSDY2wlGUCHvSQMxgTE+KvHgAAAAAAAACiDrUDiX415CdbISrFgM1TpY5NPYqkjY8yKSm8K//NsAPfu1B07WDeJnCX+G6EiOAV+dvJZTIgAaiGOgdPVWnAiPrrEqZ2lUGGgBlEjC5ALHgTwEX6raUks7VxKKo3k0glAAAAAAAAAKJNfDFTX/PTIFBSVT95X/b2DZuZN9QyAceB6aSvwgYxSEWtdiuZwOgERRYzeoowzAPP9btwcGsYCSgh2BFGrsvWu0W4jV22EhAQiG0gHldv/DlLPNgQtML2VAEaUNHT1CYAAAAAAAAAok63guBaAttArNI1dW/dn6+m0IJzO/ZfVb+98qBEKqtLGTxIdO2tUevPL3StttrBFtvdN+YcETPnOhJEuZ4SaHQqmSZUuUQavKSLBAl5Xs5hmm0aI37YrJo+ECcdJEDAWAAAAAAAAACinasgJo0tTIu4FWZ+5OGd4XlGAjHJjBI35Szfe8FKBOGR361fRabqcc9BtyffPLMPorei539gHPIg0KF94xOWOFh/dTYD+2l0L2r5vE0PDWA+OpJFzyRlRGvGkCEctv0eAAAAAAAAAKLTv9DfR8E7Ct45czXuN8vjiFYSUCikzp6cNBvXe0QQi2p1atuZ5uOtm1qER707ZgaB1fNeGhGXL5xj8BiLaATh94aOiwjun/H04TYhdQDF7lRqcUxJ010DVSqcTGsleqAAAAAAAAAAouVgosDgvzI6RwS8/KN7wSuxXsepDWPa3/dBHkyihFckoJYz5YYeVRYlyoTm6m5aANg7cCGbc1BgmHrMx1kYt4yR4ugH2zTxOQAr2DcCcyo39+ePvHRUWDZCd0q9KyVSTQAAAAAAAACjc+9+1Nn2I/j6XdNxwZjGgFruFPnXWShDAcp/F+G2AoyhNraogmNKpOYNKcl89x4SjfQEsqQPHIxJR3IhLkZrqGB+oLxjLJ6KLpbCZHVugYuNlAVtmaFOGDN8+CVkuLEeAAAAAAAAAKPV33dNsb2Rr3S8AkpSYNtBILrPoGgPBaHjC3KZaUjDcXcTaJrQXxwWP8Necn2IPhJVv2jYlkEHSmGFIJol8yj+xwojJsuX5wo1uiZdbbAm+GIl65dj+Bf6c1Wzw4c3rysAAAAAAAAApE+rmfEblNDr3JmwhQ461GtiYmcgR8p/wJfnJz6wxJ5tuu1yOpd5W1cSqp9wTJXeE6ySbWh5KSk+E+mN8eFgbHfGc2Ld1LZVdWRrqbY7CASTTKXWHvlfQrG5oI3Cd9J9JgAAAAAAAACkftxl9YpYnXsieddSxKw4O9MC2xTFztKc3ZKMZgD48ybZFAWNx7BjQx8LPhpF3EMLdNxpgD+QReip3n8QVCLgr9FBDZ93bGI5P1b6bDZtyplTJrGI2YY+SWwj6N4gH0tZAAAAAAAAAKbyJRJntIxz46+Tv+EGFU9oLCift+Gmj3HBjR1Oc7WOqMNacXJRvNgXYElfX2QmPgxJS+0g5clUTBjUTPOXGgLEHpzCOhnTOWO5lEL1UJyp0lb8lbqLwR9/9HjH4px1WKAAAAAAAAAApvT38lHofLALG6PtRCrCMYpSCOR5aQW7PH5FJrjviQS1y6VCU8c2vnQLsVK1Ci4PGVHKMmdKU3bOZV4BuolbP80b2g5Kf3fbzSOrwTeMQDOMWQlhGqLYQiv+wgabDSp9JgAAAAAAAACnA08/D4a9kyxqa98zwV/79TPM7yqiHKn7Mg+dkwoTU/0tgoqlbg/teEUtYs0V2ukSwURFvDcbsQdRSnAWBwNOxM8OwGhtNeFXWpM9IhgsBYaHlm/CJsnf8k8E1uuc8UomAAAAAAAAAKclE6jIaxV91NRakuus2gVg1liJQk3mE3D/y9DZFPevmCimSKnx43DbZ3yB899ucBlhCAshBcMQpWYcecjlc1esxke7Z6E8mN9KTq3g8k96waKRD/tkYAM8aHYqX17mQh4AAAAAAAAAqDvyGef+xvFvAiqrQbNaMjWOw3AXpv3G1j5Mx0tqCgS2Gp0dIX9XE+AzjipOJ2f/GOm1PswbZchslPCyvHWg1nK+8Ohb6wsKr77R/ZWW8sYWe+VPgciGU7MJIgqbm/GLMQEAAAAAAACoXVrd8NtRqpmDRhzOk/qKn+WFEkw1YmGi6Fsg/qHa/mXTACqPvgsIHsL/Ts8qr3kFXOPvGpyb1c/BzKfRY9mSzD/UGCPC0LuPdjSDBj2qU0i9uek0HVmNtxpqTSnnPE7HAAAAAAAAAKhsFkzJP2szO/dBjnGVUcY5Q7+/MKy7Y4YqK8Dpsv3eHGEOgh2N/g/6oUR9rl8ytAVYeSiw+2TdQcghwrzfkboiCL62kFfjjtDeRL0P7Rv1pidP1x9IhfXS6zUOKBU/iSYAAAAAAAAAqIHbuO8kqhNerPgAB2qrx0UP1KvyyUpMVumqWeUjJp9tpcajch1UTIHe2wQArZMQBec2C5Tacl7mwe1FQzfUgQdVK5AzzdbjHn5llIR4v2bPKj/i6VzMhPvE+iC4HdyzKwAAAAAAAACokSc1rVhQT+VlJfGLsCOZxWX6ruedpFhVNkULtBe7kLmFdPayLJ1oQiJYStVtTlAWnm6Vn7EhTUbrsvP//DO8DztQ/wOGH7hG1Prf76jt/eVRGIV8dJvnFM7v6hYZg/IuAAAAAAAAAKjxPZd9Mp/xm4rQfE3Rf3VxTGwW7kMYu7mWZ6RC3G4WnnQI2HoVJp0LE0z9tDYfhwWPjrFiCJHrREKysvzwnqkJKju3hmmJjPSnkwyy8ZqIqPDLB80NjvqlaAODcBzUYTEBAAAAAAAAqWv7RfIKgF8O2Yx/JCLo3V2dfTzX2e5hlxRi42yxsdA6YVYn4QNgCJ8sEyqbrZVoFtAiEJSJyaelpKLyhHv+9V5S9LDO+DACZUXbw/hT+t6rP5XvVVSNm1zxQkX5kdRnoAAAAAAAAACp4F++BLEmjpvrrdk6R92jYfhObiP3wTZiJ0ZSoEu9LYXY5ldU8ViTadbe4MC//8cKi7K+c3aQCExATzVYfdGHCL6S2yFvUk9vb7DCC9aPnsDNUxzG13kRzgWPpXQUqiQeAAAAAAAAAKn0KdS3pS6yNGGJxH9sElzOV8M3btz41s9QLCKqfnyCqL0l1KxeV1VDvLKhEkci3gWIri+YwETq6tA7oktPdyzznzXm0IyIDddJi1Ln0gwYi5BvMpSCO6xA3ptmlqSLPiYAAAAAAAAAqj7zSdZ6Tu7Z4H+c7TaiOLLwHA8bJ8JY3ysQ02tqMjBrq7mCvzmUUwfdeCdqG6UsC8qIGicUSRZ0anc77xVAcGdCmhpW5COMPE3wqwK7EkK+8J5x3gu0/ljfVcSWV3CUTQAAAAAAAACq4LEt6m8EWDHST4XeD3/rJjvTtTPwf4+2jnUjSSlwD77m9O3S4QqSm1YIGNfb7MoBFs+CFT2yFr34L5Ke6BlcxPAiBd8fXD20nZ7JCYYn3Sj3mKYMCX45khMbCz/L9iQ2AAAAAAAAAKr0nd8eoLkalJwkqYvyPj7zX6/+i8dVcTsFWMu/JVWobQJ3cj/B9UnP888xdc4VEBZiHkedDkE9E58Znika1V7WymmfvY2+Pd49WHzn36tTPk1O/VKQOjRHt2dUcKMc5B4AAAAAAAAAq3gbWulgExigeqNXHjJ21LUADhYrE7MwyXtcrxeIc87aCIibwn3LgEHA6p78PCzHEF3tfvhiogGAx1rPdXDP/9Fwi3DDN67cSvjI/oyq0ZRT5TIEKiIk91PpIyRD4BThHgAAAAAAAACrfttuBkm0jnAvo04Fu7x6g0+Ae0B7VP/90ZJW1dSEXlU8f2OnYnJEEQIdVcrW0ZwS/LPR+kiXJKo4+DgQODw6e6I0sogSU0a+NwD7fnPOOZDYiTh8gTtovfqN8Qk7vTl6AAAAAAAAAKwgDXmXAbtEKnyj0QYTVOANQOl8nY2ID3OGNvkEp29VvQ0kIWAMJq3/VP2bwNjuwQD0zMEyxIuksF0l/nbAiknPfZA3cL0vJ1PyVDjyLRvf4pqDkD7cGgbm9eoR9Z4OrR4AAAAAAAAArMXswW2bhMKvUpZODkclJNWWiSJAETAupOntMJ2pIsod++FWXx+bgRHteVCHqegoGBDXSbDQ+zgEpLc1w5c93mBrJESyc+HK5V4rkewiwJfLrTlobJeo8227/XoBfGGeMQEAAAAAAACsxyCeCrX9MjLfA9aVye7g6hSAHdnIS60oMmN+Z7ycreXMf+oK4QYImkL2NjHz364A1tAlQyhCDHlqynjSFAsL/9RCgxb0y6e17gFBGkL+RcZl+9A7rZ4ST6ZARgN8C+ImAAAAAAAAAK0w9gMxCoQLrWlFPJlrSJOUy+4KmscQu1w2UQq8SNAdRE09RdMtJppR/DWGYULZKReENdSv/LaLlqe2GyQEUA+xScSwMplOk6C879TNTyoharo3ZmwR+5ZE0FePpeOgNx4AAAAAAAAArcCl60268UzS49eyOBUE8NvTDuQ9A6JyYYiYNZdb6TW9rKT79jk+avxBsA6mpQf3AAe/NS9BhkStzfRkEqZDcv7mgLUhvDRPULFC6VFlmfsBwt5/ytgHuLdHVyUXAcobMwIAAAAAAACuX3WUI7wBA4yPqgg6jPWDB9E1MxaVfmPFa2UxD8anj5zJsYivyQ0OBYtBIuXUoz0Utd3JOry1LBwK6MQ85TQ3E+Eym7s68oIQRSuCTRAkmmykJVlxcvGien40PeJWnL0eAAAAAAAAAK+OJUb685XuYN6XUzn3Tg3zDb44TSQK4I25RklLNjuQqMbUog3fDrDtTAkkjfaehxWjvKEfLB0XM/H+LDcIBtop/NtgXk2pSYVIWqQvMnxsPYu2JUVsB4pLQSYy6gChmSYAAAAAAAAAr5p73HJBpcc4joU9nkLLO4PHmlUnjakZoKIYtYa1YPCIYcm9Z2rhzFDbjl9pAWzbDqUIToQI5jdkLXeEEyF39NPnB6Vg6dYas0/f+eQCBScuQpM/4FSOSIwDWcOChxAQKwAAAAAAAACvwfzhxBfwCWiYaw/CDr/hOb6gfdnoeOcOXKo1H5PV8Y5uiHsr1ScS3V7rQmk+uMoL6Rca/KUfvRS7/EuZJqQSemfJu/EaRgvL+RERe5cuPkFaLJEYBMsSxDjEtOfW0BU2AAAAAAAAALBDEyX1EmjDT/VWHU6hqj1WENrsj1t9rpWplc+rS6HDvb082//I/QtyR6XRUE9USQLBnv5Ks2moB2I8U0X4dvopfwOKZDIFuR66DYxyKw5vf6ybWZ7A1fg4P3V6vJv83yUAAAAAAAAAsIukItlu5BIiHu/6DaV9gyDmWtXb1y/ovXyNE24g2EtIA7x1sOojOGtg8+978eDPFmgNSVzNQM77VfTxXMZ2Gz2PRYtQKp9ku4QvhCDSjG1nkc50Ps+QuCUKGTeiRG8FJgAAAAAAAACw99LG8ihZZxc7WqqT9fTdWicKFo90IB23LwlfNd02SQpSpWVwDFoq5G7CiUu7NfAQFeH6dKWfu8ZTvuJQXLTU7oQuImhNgwgHnCcV0dsH3rYeVgQsYUYsc1OsiHxKk9oTAQAAAAAAALHajPCViBMv9upnwrCZuecUWR0JKVzr6TIq80ltIYzperkwBa872lYvD8jJ5f/+YxnGc4dmFODLE59Il7FwD/eB3IeDqNEvT229CzeNRzxaLSYkgbYNeiRrViPclAHFGDYAAAAAAAAAsjGfgINgbhhhACOIo77SamixNQ0Qbf2cMYxISIkdherXTgDIfNfljr4GVAY4kFctFDC5jVPZ995IOmC9BkoU3aEB5itF1eOO+bkdjx6vy8IgwGqJMUUut8/5QTsb+apxigAAAAAAAACytkg3sTjqDMmUKDYrbFEoxN3EwFIhjvLif79oBIs2KDgDjyumRA0ZEDOF/yV1v0QYKEtWdmBnDrZvJCPDiw0a/Dq0gX1kHAUonLncGCOQlY5FAZOmgnC+5IuOwtiPUUSgAAAAAAAAALK7sHxR04wBQ9b10H6WpYoB2PilcltDJM4NxjUwzyYEbi7Tjc55L0BEQTZPqOd3cBkSb+YYxZzLcudWsE4oE8NCjutD7jMqh0ondDrQ3GHBtdm4c2FZRZ8hwTQWjwXqrSsAAAAAAAAAsscEuaOzFcVgENskNzJJ/0pg9glAWCiqhxo8A9keBqakXKh2BwhKGZLYihVgqtuAF9xoGMqeOvMM7Sj5H7598PRsE3zAaHWAVQ+ZmSh38eoFoSso2aetDXjj7M0Y7/TsKwAAAAAAAACzELwl2yfQKhyo6tf/j0Ps9FJdIShphdOVsCeilZdlU6WE4z4HZ6sqrGdKo9BCpYgTj97riwzKxw1T4+jKZR3H/A5EHbHucum2dq5Plvb3nSEc9nQb6GQDzfoMzhXASTR6AAAAAAAAALOO2M45q2rUqhmWB7BpvaHVAYxIs1gd5y2DYq7/5Nn+Ep8M/dxNEVZH4r5NdrIU/wYVpWdmVKV7RZTDl6q5PeYapmoSzV+886ZjxCgDIUeN3BCcrI+OJIxM1wTHE8zptyUAAAAAAAAAs4+93GqSZHNwfA3jVeMCvshnHchPD7906nF0nkv365U3JvqhO1JWTa+CNYWenkNCEKm9YJNSOCUi5ICa2eSz5SXXXXbQCkpOhjMm1ADYZ+eqCFHcPPtrZxvtsMvq3AxhHgAAAAAAAACzvVjpj1VkIu2SD7q9qO1eBSQDPdEQVfJM8HC15ruC43/IBfr3Qm77dWYwwjkkikUPkZBKKVW1B+759AEcxGi81BJyp1y/z9HEpK/bt/S151vJ+jVKXWR/eBfrFSNZbxBYAAAAAAAAALQlclbzolS+R296AQ48nOTrA5EtoHkyPlAn/lxv2kLLE+NHkx1UiQcfCejFS8jPdRfTdhAmDuQ7aFCVUTcllqLgvOwuRtCq0XFZLC80ImEJcxiKAmuG+biIWPlTEP2cux4AAAAAAAAAtENtApgmmeZmlPpmAbF3V7hQupnDsfq6Gp+53DPLxy3oPeV9ddjzrM/MGJGvJ23YCjp/lpd3n8yFpsC0lUPOk6h1iPU5YVX0qtHlSRY/d2A742rnh1Wg0y63fFs8ie3JKwAAAAAAAAC0cMy5QkC/OBCkeTN/TptAjpahL5UHLZYClThIJiQQKd7HmqOkV4XlP95iY90w8aMGuTFLj7sqUFqQ1S23LywmkqtSRifM3GJnOSTeR2tfBfrtEEISNGq/H+jdmRaAZAFYAAAAAAAAALU6E3Wzv27mvW2i22GZpgcQU2364Gd15bnaVuiL+XdGHlBvXaod2bUaiasaTZVWPgh1yofIC6kUmasjrwES/zmUdykroaMJrumhOeZz7Ek0mn1vKqNqgiJl4sx1kt4VIysAAAAAAAAAtTvz2W9YowsOs5OSlHgUmX//lv0oNY+0on9NfZVJcrLU29Qq5+gFcLCOZOZ79OYiCFLIPYUj9N/HpPbDbYkFpbY3uj/AMmjKf15mpoUbgxmY9ov6nSIBFXQh5SztbccsHgAAAAAAAAC1TQIXK8urJw+YHZ8gtG9uTJOyl0lpjUUll+XZwPP5myOx6EYZ/0EVHxOaLgspoZYG+AvXrhFyAyxcsHctaTF0PnJAlPEfd7UThNYtUpBFcBNXBbHlozCkP+5d4JOHqzIxAQAAAAAAALbIHkeHcS+Zk39+vNKfDCd3jB1jITXD+Dxn8bQdtJKPzqXR8YQ2cSD/Qq823SOHcw+OXFHi+oaHTgLdw8oCpHK9Kaikdn6giho6/kXt/CgvdiBcehKVfEs6T9XfklIQKLcAAAAAAAAAtvWtK/n2Zg/mr2GwbnYC5njIcLcSHkxMeECb6w8f4XHu+TPfxlGWVDi6wM90qg/+F8ReOLSwo6puIQZEI91nQLGy0jL3maiIxBW5dwgqPz4sfb0lyffhiMKucT7la2llHgAAAAAAAAC2+kGktq8NHBkXzizu5SQ6yduWk/GM6UimzRQnnJJDWbHF8KLP4d0wtCjG4714HBMZ0W3k+m0pTHdLHjBw4GJrEdFYJhLnAxb8D/0NTA1/Jm8faMWfZAvk+bKG4lSYTmEeAAAAAAAAALcesuvnoi/bmYw7CcN8rpxdk+we3GEkPXzdmDuqB0UjQpPlSlCj52WFd4Bsw152GAIDU/F19AMEr2A/81nlsO/7EIPgVFjfuqsVntkddJh/rv1DSoQJcAfeZ1DjPdaNRaAAAAAAAAAAt1onCjwVtnAzP0YqedmSD0Zvj5zNu4BxWIA7eCHz1F8d23kamUD9DyF7LoSaGGtQAEiLlMWBCPGhx5Kpl2jGL1D/V4AIf/G8fuiw32EEZ543o/vhMHseipSCnnGlFwP8HgAAAAAAAAC3dA6JDleZP/IcrtkyRk6jtuCBPnTTM4nH9/USxgNdOs6iKptjEZzm6csFNFhLlJkUum7RX0d1AB9ORcEwo9XcGK0ynmEduIEMLyoJWKFA4NjMb0xlYm86P2B7vTTEEDl6AAAAAAAAALhOfR+qj/fsyAmPcOVorIfJn/2YNTlI0itWLCQGIe1nzZqE0syChbsTWqzpBFCuhw0kUgBA0du6+ez7us268cyR8daAonMq+hcwPcmrYukXe0A9kSNHzpNZJwiOM0d1twIBAAAAAAAA").unwrap()).unwrap();
        //
        // let committee: Committee = serde_json::from_str(r#"{"epoch":"730","voting_rights":[["gBJ8cMMnbAGi9dtt/tToqyXNsnxHNMpKeMNjOTpaPVe/lv17uYC3msT04XWR92rUBP9u2Zxv5PIJC4GiO1OiMY9kVb7/bFp5v+ENILso3sWLzY2VC/QmvGoPdVJRmD32","305"],["gRyd7ILql8CBVNtvEa0osVPVlALHFwhPzKold3cKaAoL0/jTqzrRo3W1oqzOy5BgARhTuBY/oQ1duI84eThZKllap3bsSwPc9LYNc4ofUB4/qjOefIBStjPgTw0Xzu1q","37"],["gWJtA/xxM41ucrP969kRUBJLDAYxdU+dzzEo2Ajqs5b8fJagtp2h347kStPC9V9DDf2xgpQIratIGsQ5uWgEiJNJ0S+zNzbIPG30LiSdZ0fpBdeWjfdSNR0pbDTSn5y9","38"],["gdjAeU04az2qnKYxdh9r4On47zN6sU86YaMONWgYJX13WN+jPGfJqsDw6ND3WrBHClXQiX2i34a1e0DvLYMr7hW/qPvSmV1ytLgool4d8NcMS2+8o6CNMTniVmSlF9kv","88"],["gqI/f7PigATa6E12BJOfbSsBCgP/HFVjmyAkiP1vzX2w8AFliV2YjfOUggyyRdZGD9pdcpQb5MkHlRfv1hXpOrLsKYqK06tO/hD33ewAURal7vhFpRaef/EFpJfXZTsJ","122"],["g1i4wPi4MmesCeucPP3+Gf5a+5AKcNo/0mLo1dYse6Wmo0az+YbWuMHAE8B2pcaXEtw9e2OBZdUqejFRxj3rB9eWRKi59uxzeZlvFHpZ6I2j602hCix0uXwvsHDgnekW","30"],["hCnexJorordUG5FBfQSo3pajufizpfC5HF9AysqYf9k8fMTpIoRgwugXWWLOJgwhB7CCX8enpyLUtkerkfYi0tf4MRfG/ORltMETP/cyiF3ucIBOpO0vJPaH38uYp+uo","54"],["hN33X1uYqEN9IclBMs1/nIAGPQasZePB5ZGXlkfOR3uhLQY8Ou57u9u6vEnkRsq4Fsim0ZPeXBMzG4+lSawMTBTOTRR2RXhfzp0c5wrRWe0pyl1xjM+7wTh0FomjgmjP","54"],["hceNCsEwuIf+3yytmv7vfLwaEHRLVLfCi9ra905s75bwdKMe2/Rxm2oXMlrXFI6aEVqTFx6eArk+EfhJZ2SOty/qTH8jh/gBlmkwQHIt61OuMyQ+JqJAH/PXtHDDlSU+","122"],["hg5hFgAT442YRwhHI2LXRIdSWXjJqx/MhZRoPqeoiileP5+F6PcXwlyLGHkcVFCBC8LpYYSgBfNN0SDgg44KH5QwoJSweppJOgjdtUI5FPuPsuIbEwyH0jWZrJXGHq20","153"],["hhir3LhIy0cD8VOHbne085lpMoQeu08MaGxjA2xI4fvPnnk4X/GKUY11rHCej9a1FaBUVGJtwDmyAOCzm1JnvTaCP14uvouEQtL9U4jcTLFd/Xa7bZ1ZGCkqXJn7bmht","77"],["hov7ICYQGW82dQrq6O2QDHnH5KSOyZSEq+Nrx8Hh2UMxPyQVp8IKBh3gZsRVK38YBzIxwEqE01F1Ofll2Uohdju9bQ2Taa5IwMfKh/rqpsrrt2nJ/P6b9UGk56e5VwAH","30"],["hqePwoxs56D9JP4ogcD/FkI+l42QvjgWbJf67x+MggHU9xzRtrVG43LMCwRIai3nAlFVjundIWevtoh6W31nx+wYxN0EIjqlo44wQelghJY7nWh8bxt7M0ihklNcLHRM","38"],["hsKTpuy3K7ii/mDgLjpDbxB90zyE2UsFBseYDTUPyXlFuKX5KzcHOqS5TGpIg/EBBtw3hmA6pGW+4uZ+yU7h4jh/MMSxC5AExfxAmGlC/rQ5nv7igWWMMd+hl3Toa3cs","153"],["hzHFXV7BxF8GuX6sO07KmgMADSoAT8LiUyrPeyBbbV33u7hScOFRsiwnGll3eRdrA8HWOHtIKOIqPTAYEKdtuvdwinwOI1vRQf4BHXDCev6DTN3OdZzvtLASsf7qpQse","38"],["h43cbJLA8jlo20iPFwn2u4dNY6VuJdHrLsuapMA3w2bC8FLtpnBXKP9Y1OVHc9egA8Ex73AohO9w+QipWvZtlBddEHkAXPVphar5BvIE6C7hEtuFvNGN1bpw30U+dhtW","57"],["iHN8bU+ek8GBukQM0ysOl0wB/faNdNAEj/vTK5HxBOzpvmEUVnSy3Upl4ILxH6vtCqhZaiLUpSxY0Xwu/IQN6a63eDv3q3BoyzhLqMmjiuaXEhgHGo5vfskLN6xCZApp","88"],["iKcdMCPhjL+zsPmJbsawypNlamSDrjwxVBQfWtKJxrAOkBdvFOlR3X2LLFJEGuO7FZUX5/AohaVJtRHBNFfiHqWXipywBSw1ZuBQH6TMQnee5Hkg63QO8HEwRYZ7iFP6","259"],["iMvrAO1OhA8sSpS7riHGbeS6arPVbaM/U0dDm2syrhph6tP6DW9stYH8wz8ph7JWCW+o8UlmrYTKSN4/81SjgY3+pDv/bxvJOxj1oKK84VFIAcS/NvElzmRdpPQLNH9x","30"],["iVVAJCIzaoLT0d+M2wfyWCvCSHPngXiUH0yqmhqyBs0jQHEJPg/KEA1tSUNh8H/QAVxJSKr9W89qOibWmt4yBVTfD9rLIPQuAAyYi9gWWJUxO3DHM5QWnNwgFMwp4j0p","30"],["iu1jnYmRfIGQYHLvRM1BBkM8/T9ifGIDDpcroN8QknCjkm0xSGk0NFUV7WYklFgWCBR/ezJv+/5qMkOXQrHRKFT3j+Ch35GktDEcqsayAWsAZnSSyAm9vhrIbFxjdB0b","30"],["iw/cY6O/DPuSxotj6XF4miqFKjDBQ00KHAOAPT4fReIChXx21Huz+NasiHDiMqOZFAIjq+b/4KAK8Mc+wR4t6tBtR02dVv/nlkQfZIG3K22k9Q9vu4bV6ZYUC8mkOnNr","54"],["i5mbjTxR5/4pXx51a+yBVzbEX9aexB60R7OeDUTM5ECad0mxSEhBaQsl/CFjOkYQEF6YEfpXfmLSwyhosH/wHn1mpTfd0sraWlLKCPKuSAslinHwKzO4uT/KlImzoDhW","38"],["jHdx/voBTDh3zbCqJUb1OwzOc6r2c0UJOklPVumh2GKzl3PvCR/VfmNJiHZ5V18XEZntqc/0Od2tYcl5AO4NIXJ2joyd9awHmWwPmg18fUtiaOzWKOWTClRMUfuLf7n1","37"],["jWPDPS/OQC/TNX859MIkhVo0zxdWNxS2LZ53kepxe3/cL7LsLJKHdz27xdVd2XOsFyJD0NGwrih4ahM7iSng15v0TT6rkkaPDQ4gldxnJnm4Y1NIBDw5hWT/Vzfs+j/Y","145"],["jnrPOaK6yBBMguFHGfSr9YanfRJL6Offg2N8MfOJwhmOafNxEF0H+hrLpQnuDsKLAPSKJE1U77rs2sPb99V1a+LaL+1JzyyVzn0kBiFuXNrxDPWu3Q7vOgUJuaJRhL1b","296"],["jqfviomqBMl1EJy91cxam7AsiWLGLloMO8ZlaJE2UXmXjH4AtHQXfP/u9isTUzpICgY4Q+tdKnHCXiq4QTz00ImoGo/ZfQfDILjnOMuTpfK7AhzwbJakPDD4+P//HjKY","30"],["jsS8WYAgTiTb5oQQg6RCPUfRHMQarJaMr8+jYwDxiPCdrW+PHPArdz+LB8z9YbaaEJbXopov/1XES4wEprsWgLopWb+HkPLeG2HPSBhfoGOE2ZIShKgQalG5YndR5nQk","37"],["jzaTRuYjJQYOWu6iSaCvtVf/69ArN2EenlJthPdxHKZycM+jMDQnsUfg3+jxRNP8DaqXS8F/mPVMj2uZ9nnMVvvt/GQuvXMQ50LxQkAM4uUWtvl0BZfJvFOtcY2g5Zec","122"],["j/W94KkNIhPnbsivslsqiHAVwuWh1Be9QMOetH7QmWfeHGR096QqgEy3b2m144FAFfylDjCMvHOUmPMqeUx0A/kxrN1tJt5YRtEpiaMiif37s1A3Lq4+lqRZKgekTvyT","43"],["kSq/0gyynEjLy/IRQ5ISeZXoWBzHdHLOtJdacsZjPaB/o1ZvXvMnlxyzWd+Uu5fCD2AxIEaDyuMQF0lcoo9iDJBpBThgWgtn6yTCPS+AVxjOX1WBsl6hXiPfkucW2eRK","54"],["kSrb5TASIwS0LkW6HZQqn+pq14ahV6ZG6bLp5RYTzt+8OsiVuVdcqYWmbBXwhddNChFxM+khPy6qgJBiPl46MF4wUub/pqP3tabV3HWcC9vNfscE/oXYleLyqUOwpWyQ","44"],["kWNJc2vsqBC2et8bIAbpEnRNA2wnfQnqbGY9QiLQga/Oej4KU1wEKtfu4jbNDUk+AOed7I0L22OQiZxChKgfz97pVtKtR46C1bXO6MfHQ4gyZYfWBNi2L6MxIsLAdAgM","30"],["klFOduTQiXNuxZtiWM0wVizSZpBnzi7tQbZ2otDy6p7i6Fgq5FEZuvUra0kaMp4BBJW6UNZhuu99cIXiyuFiW1hjWRMTosuRD35a8abd/E2zemEB1gqO+1vKQ1iA5TMo","43"],["kl76hkg5t8BopiXVqjiihTU8iDr3R7r8aI0z+hFaFwTtQMZD9PHXXpwcJ/iaVx2fCNKUgmDLk0zoggq/6NxjICJu+0aa4xZV07dMBBKsRGFmrm7BzKCfqg0KO7l7/j+r","122"],["kttshEzvKJh5FEbKGKnjuUF70eCexyYDkhPUjPGs55vLmhuBrzEiulVtdBJAZIiuGJsJMnmkyubdua/VMQ68adsARIwLtSoI+k5itQ1W0zHN2tTYS9uh3R/Rp8rgyYM4","130"],["lCHoMVfPpbiGWOuuJ3sb3Oo9UxA1kz+KjSAKS4baS2WHQLA6rKFA0OwPJJQ4QE4PFI7FET3HptCSRdHi3y3Ii5WWrniWnVV6bzjAvLKqkhNQA0jbXasnHTOEH1uvp7WD","38"],["lGnmFH0X+Q4CaSgfyvQSJxeCEAhhTzcjA/7k3I/S9hlfzGIIGloesjm8skqoIDeVCjWOXnPipr50Fvm0g+lNXLwHa1dSc+SJrAA4Z2hNwoIfJYzAdCm7AYlVD5pPMGa9","54"],["lLd8mU6nPsK2LgtAaPXrcnpjk/LAYeL1FRvK4275u49TVqmcb5lKefa7ca0+INDfCD+5Rr1MyT5tv3p3IyWXhwPIiBnDOS9HlJuL9JGeJ8AhGR4KViVl5ZDNxiIzUL4b","30"],["lNK2s3Wra1ngGAE1MtgVSq1qbjMWMSi+ge7srCsXHPRVcVZhYy+BiHWN8m37M/xsAd8c9vDzESN8A08wU5BQahwX3nm30ng+AhX35ueCj1fTHd5VJHvlLvhEZwEPC5Mj","90"],["lNMiVCuJciLJ28HX9fBX1mOYxxRsJdSMzLGvkZdBA5U1dRadM7ICqK6QFyxjgqE+BLEv2YQSpkSvIdZWAW9m1IORlH0gv7iabtnVsVyjPDPyuXhwF9jqUDA+qkbPUDBe","122"],["lOnKRKBDiyFd+tvWypnDhbqXe26DwkXwoqdbcRsjlH/elptH+m6NzwDTl6qrk1mGDeQUud8FTR7f85x+ocXGVunRCnozYBAQBvVaqKPZXqtM8RoQIhFxq1d7oOG4NVrx","38"],["ll48+31lwgoJTbW8ra6Q0Lb2WOXo0pLC8Rb7aSgwaR2FCwttUl3WIEZOfq3ta5xIA6L7oOuN8blns1eBgx9yKMYb449hg8oYWe/H6nF11YaOkAo/b62IKdjbplSxWE/p","30"],["mBxTdpbftXk1pPXPIwXT2JRS0cVB0JqLQ9YjrtTOSd7ejXx4B0IEXtVjjJYy56FXC2F6jTQPVcbHPM4Gx96mYA35W+gTrEAIMwdSyA1g9/9IbpTQcTqJFwOb3P26ksYw","54"],["mGpfh7j89RDOTQmfmOZCdPtKAtV5Yxw2hsVikoXyqLcktXjosUvED1+z/LA+Gpw+ErFPsy+lqDc/f6YwLJWNcJPYTs74r7xyQIwUp9206q48s87sV3+lVM4ngmUhKzYr","154"],["mITk5QEDMwZfJKtpuQjcaX564nu+dczttw9hFF6VGL8uZC+81sykZT6sCLaiWqRfEYYm9nRAJPK18sCkKRTqMPN4bOW2CROk75khEu9RD+yWjFiiRmxYGfPw0dg3sczA","30"],["mIo7dv8nr8yClkGH31E6LaAcQdOqkDM2oUsVwovfNKzae/e8uzsBlPuj8yyxom3nEl/s5dXjrd40uokwq0CmlcR97x6Idea+9DOtMlxGF2ZU/o6O/og9/EJ+3z6Yx8uX","30"],["mJWeo3tO6eONd5DcYUeWkC5uDmTQvgXSQptBIWq5He/YrtMUq9eOR3jeRiBE8rxiGcBslhPiJ86ONHJvcpWwnxyIxAheqz8UVDTjfefsPwTHk7fYHDsvs/JmNja9zfuF","88"],["mUT0LDoR8e0M8tMp+a1GO0lqbqNN1snQC3tk5pcTcXf5OH5zQyWaZJCRafgI3NbnD9W/+l/kol5DPK6heCBtlk1nBAHOtBX63yHe7gsfrVx3MP8vsI20rWXD9rLVHiYn","38"],["mbpO2r14Df+O6fJ4Ou2GrF4DbTh5TBAaZPmXm3KLrfHB1LrBuS972JDGqAd0dC6eFSulgytQzwbLf5gqKVVkGb2FtRtEHN8G6Z4t9O+s+k9M2zcAjVO5Bh/rdJ6IC615","122"],["mc6u3+YYab4ix/kcfJRRLBbuamb1TnGdOy6IiiZmYX/Cr9Y49PPeFah0+KD2kxaGFnJYZe/i79UrHXe62MyOvRjhjvwC1ojhQzj5t8yLUA76I5iaKtUnVLAAOhg7recx","30"],["oCUawUeCoawSZ4vrUYRJPtHYG5T4iPHvhtqB8OaI5NmL4NRngedpga1b04DlpxHfAYxt7SopbfyFqfRPodoaHOUiNz0xwt8EBHiZyBY9D6mqBAIWXAWgZOjaAAhcBX11","43"],["oWtN4zVptwEQ61XJiKV3jkk1Ijhytp4iyLpDD/eXGfuxlQ40dCL9NEf4CTKlTQJzF/dD6C3N33ov8BSw8JVuz3SVSh2VqgueW2W85+Ws9MUSDY2wlGUCHvSQMxgTE+Kv","30"],["og61A4l+NeQnWyEqxYDNU6WOTT2KpI2PMikpvCv/zbAD37tQdO1g3iZwl/huhIjgFfnbyWUyIAGohjoHT1VpwIj66xKmdpVBhoAZRIwuQCx4E8BF+q2lJLO1cSiqN5NI","37"],["ok18MVNf89MgUFJVP3lf9vYNm5k31DIBx4HppK/CBjFIRa12K5nA6ARFFjN6ijDMA8/1u3BwaxgJKCHYEUauy9a7RbiNXbYSEBCIbSAeV2/8OUs82BC0wvZUARpQ0dPU","38"],["ok63guBaAttArNI1dW/dn6+m0IJzO/ZfVb+98qBEKqtLGTxIdO2tUevPL3StttrBFtvdN+YcETPnOhJEuZ4SaHQqmSZUuUQavKSLBAl5Xs5hmm0aI37YrJo+ECcdJEDA","88"],["op2rICaNLUyLuBVmfuThneF5RgIxyYwSN+Us33vBSgThkd+tX0Wm6nHPQbcn3zyzD6K3oud/YBzyINChfeMTljhYf3U2A/tpdC9q+bxNDw1gPjqSRc8kZURrxpAhHLb9","30"],["otO/0N9HwTsK3jlzNe43y+OIVhJQKKTOnpw0G9d7RBCLanVq25nm462bWoRHvTtmBoHV814aEZcvnGPwGItoBOH3ho6LCO6f8fThNiF1AMXuVGpxTEnTXQNVKpxMayV6","160"],["ouVgosDgvzI6RwS8/KN7wSuxXsepDWPa3/dBHkyihFckoJYz5YYeVRYlyoTm6m5aANg7cCGbc1BgmHrMx1kYt4yR4ugH2zTxOQAr2DcCcyo39+ePvHRUWDZCd0q9KyVS","77"],["o3PvftTZ9iP4+l3TccGYxoBa7hT511koQwHKfxfhtgKMoTa2qIJjSqTmDSnJfPceEo30BLKkDxyMSUdyIS5Ga6hgfqC8Yyyeii6WwmR1boGLjZQFbZmhThgzfPglZLix","30"],["o9Xfd02xvZGvdLwCSlJg20Egus+gaA8FoeMLcplpSMNxdxNomtBfHBY/w15yfYg+ElW/aNiWQQdKYYUgmiXzKP7HCiMmy5fnCjW6Jl1tsCb4YiXrl2P4F/pzVbPDhzev","43"],["pE+rmfEblNDr3JmwhQ461GtiYmcgR8p/wJfnJz6wxJ5tuu1yOpd5W1cSqp9wTJXeE6ySbWh5KSk+E+mN8eFgbHfGc2Ld1LZVdWRrqbY7CASTTKXWHvlfQrG5oI3Cd9J9","38"],["pH7cZfWKWJ17InnXUsSsODvTAtsUxc7SnN2SjGYA+PMm2RQFjcewY0MfCz4aRdxDC3TcaYA/kEXoqd5/EFQi4K/RQQ2fd2xiOT9W+mw2bcqZUyaxiNmGPklsI+jeIB9L","89"],["pvIlEme0jHPjr5O/4QYVT2gsKJ+34aaPccGNHU5ztY6ow1pxclG82BdgSV9fZCY+DElL7SDlyVRMGNRM85caAsQenMI6GdM5Y7mUQvVQnKnSVvyVuovBH3/0eMfinHVY","160"],["pvT38lHofLALG6PtRCrCMYpSCOR5aQW7PH5FJrjviQS1y6VCU8c2vnQLsVK1Ci4PGVHKMmdKU3bOZV4BuolbP80b2g5Kf3fbzSOrwTeMQDOMWQlhGqLYQiv+wgabDSp9","38"],["pwNPPw+GvZMsamvfM8Ff+/UzzO8qohyp+zIPnZMKE1P9LYKKpW4P7XhFLWLNFdrpEsFERbw3G7EHUUpwFgcDTsTPDsBobTXhV1qTPSIYLAWGh5ZvwibJ3/JPBNbrnPFK","38"],["pyUTqMhrFX3U1FqS66zaBWDWWIlCTeYTcP/L0NkU96+YKKZIqfHjcNtnfIHz325wGWEICyEFwxClZhx5yOVzV6zGR7tnoTyY30pOreDyT3rBopEP+2RgAzxodipfXuZC","30"],["qDvyGef+xvFvAiqrQbNaMjWOw3AXpv3G1j5Mx0tqCgS2Gp0dIX9XE+AzjipOJ2f/GOm1PswbZchslPCyvHWg1nK+8Ohb6wsKr77R/ZWW8sYWe+VPgciGU7MJIgqbm/GL","305"],["qF1a3fDbUaqZg0YczpP6ip/lhRJMNWJhouhbIP6h2v5l0wAqj74LCB7C/07PKq95BVzj7xqcm9XPwcyn0WPZksw/1BgjwtC7j3Y0gwY9qlNIvbnpNB1Zjbcaak0p5zxO","199"],["qGwWTMk/azM790GOcZVRxjlDv78wrLtjhiorwOmy/d4cYQ6CHY3+D/qhRH2uXzK0BVh5KLD7ZN1ByCHCvN+RuiIIvraQV+OO0N5EvQ/tG/WmJ0/XH0iF9dLrNQ4oFT+J","38"],["qIHbuO8kqhNerPgAB2qrx0UP1KvyyUpMVumqWeUjJp9tpcajch1UTIHe2wQArZMQBec2C5Tacl7mwe1FQzfUgQdVK5AzzdbjHn5llIR4v2bPKj/i6VzMhPvE+iC4Hdyz","43"],["qJEnNa1YUE/lZSXxi7AjmcVl+q7nnaRYVTZFC7QXu5C5hXT2siydaEIiWErVbU5QFp5ulZ+xIU1G67Lz//wzvA87UP8Dhh+4RtT63++o7f3lURiFfHSb5xTO7+oWGYPy","46"],["qPE9l30yn/GbitB8TdF/dXFMbBbuQxi7uZZnpELcbhaedAjYehUmnQsTTP20Nh+HBY+OsWIIketEQrKy/PCeqQkqO7eGaYmM9KeTDLLxmoio8MsHzQ2O+qVoA4NwHNRh","305"],["qWv7RfIKgF8O2Yx/JCLo3V2dfTzX2e5hlxRi42yxsdA6YVYn4QNgCJ8sEyqbrZVoFtAiEJSJyaelpKLyhHv+9V5S9LDO+DACZUXbw/hT+t6rP5XvVVSNm1zxQkX5kdRn","160"],["qeBfvgSxJo6b663ZOkfdo2H4Tm4j98E2YidGUqBLvS2F2OZXVPFYk2nW3uDAv//HCouyvnN2kAhMQE81WH3Rhwi+ktshb1JPb2+wwgvWj57AzVMcxtd5Ec4Fj6V0FKok","30"],["qfQp1LelLrI0YYnEf2wSXM5Xwzdu3PjWz1AsIqp+fIKovSXUrF5XVUO8sqESRyLeBYiuL5jAROrq0DuiS093LPOfNebQjIgN10mLUufSDBiLkG8ylII7rEDem2aWpIs+","38"],["qj7zSdZ6Tu7Z4H+c7TaiOLLwHA8bJ8JY3ysQ02tqMjBrq7mCvzmUUwfdeCdqG6UsC8qIGicUSRZ0anc77xVAcGdCmhpW5COMPE3wqwK7EkK+8J5x3gu0/ljfVcSWV3CU","77"],["quCxLepvBFgx0k+F3g9/6yY707Uz8H+Pto51I0kpcA++5vTt0uEKkptWCBjX2+zKARbPghU9sha9+C+SnugZXMTwIgXfH1w9tJ2eyQmGJ90o95imDAl+OZITGws/y/Yk","54"],["qvSd3x6guRqUnCSpi/I+PvNfr/6Lx1VxOwVYy78lVahtAndyP8H1Sc/zzzF1zhUQFmIeR50OQT0TnxmeKRrVXtbKaZ+9jb493j1YfOffq1M+TU79UpA6NEe3Z1Rwoxzk","30"],["q3gbWulgExigeqNXHjJ21LUADhYrE7MwyXtcrxeIc87aCIibwn3LgEHA6p78PCzHEF3tfvhiogGAx1rPdXDP/9Fwi3DDN67cSvjI/oyq0ZRT5TIEKiIk91PpIyRD4BTh","30"],["q37bbgZJtI5wL6NOBbu8eoNPgHtAe1T//dGSVtXUhF5VPH9jp2JyRBECHVXK1tGcEvyz0fpIlySqOPg4EDg8OnuiNLKIElNGvjcA+35zzjmQ2Ik4fIE7aL36jfEJO705","122"],["rCANeZcBu0QqfKPRBhNU4A1A6XydjYgPc4Y2+QSnb1W9DSQhYAwmrf9U/ZvA2O7BAPTMwTLEi6SwXSX+dsCKSc99kDdwvS8nU/JUOPItG9/imoOQPtwaBub16hH1ng6t","30"],["rMXswW2bhMKvUpZODkclJNWWiSJAETAupOntMJ2pIsod++FWXx+bgRHteVCHqegoGBDXSbDQ+zgEpLc1w5c93mBrJESyc+HK5V4rkewiwJfLrTlobJeo8227/XoBfGGe","305"],["rMcgngq1/TIy3wPWlcnu4OoUgB3ZyEutKDJjfme8nK3lzH/qCuEGCJpC9jYx89+uANbQJUMoQgx5asp40hQLC//UQoMW9Munte4BQRpC/kXGZfvQO62eEk+mQEYDfAvi","38"],["rTD2AzEKhAutaUU8mWtIk5TL7gqaxxC7XDZRCrxI0B1ETT1F0y0mmlH8NYZhQtkpF4Q11K/8touWp7YbJARQD7FJxLAymU6ToLzv1M1PKiFqujdmbBH7lkTQV4+l46A3","30"],["rcCl60268UzS49eyOBUE8NvTDuQ9A6JyYYiYNZdb6TW9rKT79jk+avxBsA6mpQf3AAe/NS9BhkStzfRkEqZDcv7mgLUhvDRPULFC6VFlmfsBwt5/ytgHuLdHVyUXAcob","563"],["rl91lCO8AQOMj6oIOoz1gwfRNTMWlX5jxWtlMQ/Gp4+cybGIr8kNDgWLQSLl1KM9FLXdyTq8tSwcCujEPOU0NxPhMpu7OvKCEEUrgk0QJJpspCVZcXLxonp+ND3iVpy9","30"],["r44lRvrzle5g3pdTOfdODfMNvjhNJArgjblGSUs2O5CoxtSiDd8OsO1MCSSN9p6HFaO8oR8sHRcz8f4sNwgG2in822BeTalJhUhapC8yfGw9i7YlRWwHiktBJjLqAKGZ","38"],["r5p73HJBpcc4joU9nkLLO4PHmlUnjakZoKIYtYa1YPCIYcm9Z2rhzFDbjl9pAWzbDqUIToQI5jdkLXeEEyF39NPnB6Vg6dYas0/f+eQCBScuQpM/4FSOSIwDWcOChxAQ","43"],["r8H84cQX8AlomGsPwg6/4Tm+oH3Z6HjnDlyqNR+T1fGOboh7K9UnEt1e60JpPrjKC+kXGvylH70Uu/xLmSakEnpnybvxGkYLy/kREXuXLj5BWiyRGATLEsQ4xLTn1tAV","54"],["sEMTJfUSaMNP9VYdTqGqPVYQ2uyPW32ulamVz6tLocO9vTzb/8j9C3JHpdFQT1RJAsGe/kqzaagHYjxTRfh2+il/A4pkMgW5HroNjHIrDm9/rJtZnsDV+Dg/dXq8m/zf","37"],["sIukItlu5BIiHu/6DaV9gyDmWtXb1y/ovXyNE24g2EtIA7x1sOojOGtg8+978eDPFmgNSVzNQM77VfTxXMZ2Gz2PRYtQKp9ku4QvhCDSjG1nkc50Ps+QuCUKGTeiRG8F","38"],["sPfSxvIoWWcXO1qqk/X03VonChaPdCAdty8JXzXdNkkKUqVlcAxaKuRuwolLuzXwEBXh+nSln7vGU77iUFy01O6ELiJoTYMIB5wnFdHbB962HlYELGFGLHNTrIh8SpPa","275"],["sdqM8JWIEy/26mfCsJm55xRZHQkpXOvpMirzSW0hjOl6uTAFrzvaVi8PyMnl//5jGcZzh2YU4MsTn0iXsXAP94Hch4Oo0S9Pbb0LN41HPFotJiSBtg16JGtWI9yUAcUY","54"],["sjGfgINgbhhhACOIo77SamixNQ0Qbf2cMYxISIkdherXTgDIfNfljr4GVAY4kFctFDC5jVPZ995IOmC9BkoU3aEB5itF1eOO+bkdjx6vy8IgwGqJMUUut8/5QTsb+apx","138"],["srZIN7E46gzJlCg2K2xRKMTdxMBSIY7y4n+/aASLNig4A48rpkQNGRAzhf8ldb9EGChLVnZgZw62byQjw4sNGvw6tIF9ZBwFKJy53BgjkJWORQGTpoJwvuSLjsLYj1FE","160"],["sruwfFHTjAFD1vXQfpaligHY+KVyW0Mkzg3GNTDPJgRuLtONznkvQERBNk+o53dwGRJv5hjFnMty51awTigTw0KO60PuMyqHSid0OtDcYcG12bhzYVlFnyHBNBaPBeqt","43"],["sscEuaOzFcVgENskNzJJ/0pg9glAWCiqhxo8A9keBqakXKh2BwhKGZLYihVgqtuAF9xoGMqeOvMM7Sj5H7598PRsE3zAaHWAVQ+ZmSh38eoFoSso2aetDXjj7M0Y7/Ts","43"],["sxC8Jdsn0CocqOrX/49D7PRSXSEoaYXTlbAnopWXZVOlhOM+B2erKqxnSqPQQqWIE4/e64sMyscNU+PoymUdx/wORB2x7nLptnauT5b2950hHPZ0G+hkA836DM4VwEk0","122"],["s47YzjmratSqGZYHsGm9odUBjEizWB3nLYNirv/k2f4Snwz93E0RVkfivk12shT/BhWlZ2ZUpXtFlMOXqrk95hqmahLNX7zzpmPEKAMhR43cEJysj44kjEzXBMcTzOm3","37"],["s4+93GqSZHNwfA3jVeMCvshnHchPD7906nF0nkv365U3JvqhO1JWTa+CNYWenkNCEKm9YJNSOCUi5ICa2eSz5SXXXXbQCkpOhjMm1ADYZ+eqCFHcPPtrZxvtsMvq3Axh","30"],["s71Y6Y9VZCLtkg+6vajtXgUkAz3REFXyTPBwtea7guN/yAX690Ju+3VmMMI5JIpFD5GQSilVtQfu+fQBHMRovNQScqdcv8/RxKSv27f0tedbyfo1Sl1kf3gX6xUjWW8Q","88"],["tCVyVvOiVL5Hb3oBDjyc5OsDkS2geTI+UCf+XG/aQssT40eTHVSJBx8J6MVLyM91F9N2ECYO5DtoUJVRNyWWouC87C5G0KrRcVksLzQiYQlzGIoCa4b5uIhY+VMQ/Zy7","30"],["tENtApgmmeZmlPpmAbF3V7hQupnDsfq6Gp+53DPLxy3oPeV9ddjzrM/MGJGvJ23YCjp/lpd3n8yFpsC0lUPOk6h1iPU5YVX0qtHlSRY/d2A742rnh1Wg0y63fFs8ie3J","43"],["tHDMuUJAvzgQpHkzf06bQI6WoS+VBy2WApU4SCYkECnex5qjpFeF5T/eYmPdMPGjBrkxS4+7KlBakNUtty8sJpKrUkYnzNxiZzkk3kdrXwX67RBCEjRqvx/o3ZkWgGQB","88"],["tToTdbO/bua9baLbYZmmBxBTbfrgZ3XludpW6Iv5d0YeUG9dqh3ZtRqJqxpNlVY+CHXKh8gLqRSZqyOvARL/OZR3KSuhowmu6aE55nPsSTSafW8qo2qCImXizHWS3hUj","43"],["tTvz2W9YowsOs5OSlHgUmX//lv0oNY+0on9NfZVJcrLU29Qq5+gFcLCOZOZ79OYiCFLIPYUj9N/HpPbDbYkFpbY3uj/AMmjKf15mpoUbgxmY9ov6nSIBFXQh5Sztbccs","30"],["tU0CFyvLqycPmB2fILRvbkyTspdJaY1FJZfl2cDz+ZsjsehGGf9BFR8Tmi4LKaGWBvgL164RcgMsXLB3LWkxdD5yQJTxH3e1E4TWLVKQRXATVwWx5aMwpD/uXeCTh6sy","305"],["tsgeR4dxL5mTf3680p8MJ3eMHWMhNcP4PGfxtB20ko/OpdHxhDZxIP9CrzbdI4dzD45cUeL6hodOAt3DygKkcr0pqKR2fqCKGjr+Re38KC92IFx6EpV8SzpP1d+SUhAo","183"],["tvWtK/n2Zg/mr2GwbnYC5njIcLcSHkxMeECb6w8f4XHu+TPfxlGWVDi6wM90qg/+F8ReOLSwo6puIQZEI91nQLGy0jL3maiIxBW5dwgqPz4sfb0lyffhiMKucT7la2ll","30"],["tvpBpLavDRwZF84s7uUkOsnblpPxjOlIps0UJ5ySQ1mxxfCiz+HdMLQoxuO9eBwTGdFt5PptKUx3Sx4wcOBiaxHRWCYS5wMW/A/9DUwNfyZvH2jFn2QL5PmyhuJUmE5h","30"],["tx6y6+eiL9uZjDsJw3yunF2T7B7cYSQ9fN2YO6oHRSNCk+VKUKPnZYV3gGzDXnYYAgNT8XX0AwSvYD/zWeWw7/sQg+BUWN+6qxWe2R10mH+u/UNKhAlwB95nUOM91o1F","160"],["t1onCjwVtnAzP0YqedmSD0Zvj5zNu4BxWIA7eCHz1F8d23kamUD9DyF7LoSaGGtQAEiLlMWBCPGhx5Kpl2jGL1D/V4AIf/G8fuiw32EEZ543o/vhMHseipSCnnGlFwP8","30"],["t3QOiQ5XmT/yHK7ZMkZOo7bggT500zOJx/f1EsYDXTrOoiqbYxGc5unLBTRYS5SZFLpu0V9HdQAfTkXBMKPV3BitMp5hHbiBDC8qCVihQODYzG9MZWJvOj9ge700xBA5","122"],["uE59H6qP9+zICY9w5Wish8mf/Zg1OUjSK1YsJAYh7WfNmoTSzIKFuxNarOkEUK6HDSRSAEDR27r57Pu6zbrxzJHx1oCicyr6FzA9yati6Rd7QD2RI0fOk1knCI4zR3W3","258"]]}"#).unwrap();

        let header: Header = serde_json::from_str(r#"{"trusted_height":194261436,"checkpoint_summary":{"epoch":730,"sequence_number":194264160,"network_total_transactions":2500210826,"content_digest":"Ezy6ea9Fsx24qHwS5p4AYnxX8gJEq8kWv6P32iigzEkW","previous_digest":"8CoS5ceG6Z19RGatCYqpx9P6mZ6uxGVchShz9PFQmHrs","epoch_rolling_gas_cost_summary":{"computationCost":"377332000000","storageCost":"2117993247200","storageRebate":"1676773630224","nonRefundableStorageFee":"16937107376"},"timestamp_ms":1746868665898,"checkpoint_commitments":[],"end_of_epoch_data":null,"version_specific_data":[0,0]},"sign_info":{"epoch":730,"signature":"gDmf7wiw3548xXMzf7IBJ2TlOmuGDFcIo73J+iZqPKCxS5zAaWrAuw/SPoIkPg09","signers_map":[58,48,0,0,1,0,0,0,0,0,66,0,16,0,0,0,0,0,1,0,2,0,4,0,7,0,8,0,9,0,11,0,15,0,16,0,17,0,21,0,22,0,23,0,24,0,25,0,26,0,27,0,28,0,29,0,31,0,32,0,35,0,36,0,38,0,39,0,40,0,42,0,44,0,45,0,46,0,51,0,57,0,58,0,59,0,61,0,63,0,66,0,68,0,69,0,70,0,71,0,72,0,73,0,74,0,75,0,79,0,82,0,84,0,85,0,87,0,89,0,90,0,93,0,94,0,97,0,100,0,102,0,104,0,106,0,107,0,108,0,110,0,111,0,112,0,113,0,114,0]},"transactions":[{"transaction":"48EuQUd8vTG8kxqnbTAybqE6rJ298FQqNHrUQoWqhWUm","effects":"A7H8RR6VdzKMp6K9Hy5hsWZWGKjMzkDTkpK3m3cqQ3ht"},{"transaction":"CnW1TcGQpiWJbA8b8yBii51GP6edeeZbA5czxGGCvnv5","effects":"3e587WkPt5ihaDEvTiUze2iuwFW6gypSySzUfke6gib3"},{"transaction":"3kTwbbjTCoqhxqJ5WeogUncgEfkgxyvXqacagis1kxCr","effects":"NXrLcPfMkayRYycrMGc6wxRnZyq81patVZD59XntMRe"}]}"#).unwrap();

        verify_signature(
            mock_dependencies().as_ref(),
            &committee,
            &header.checkpoint_summary,
            header.sign_info,
        );
    }

    #[derive(Serialize, Deserialize)]
    pub enum Owner {
        AddrOwner,
        ObjectOwner(SuiAddress),
    }

    #[test]
    fn object_digest() {
        let object = ObjectInner {
            data: Data::Move(MoveObject {
                type_: MoveObjectType::Other(StructTag {
                    address: hex!(
                        "0000000000000000000000000000000000000000000000000000000000000002"
                    )
                    .into(),
                    module: "dynamic_field".into(),
                    name: "Field".into(),
                    type_params: vec![
                        TypeTag::Vector(Box::new(TypeTag::U8)),
                        TypeTag::Vector(Box::new(TypeTag::U8)),
                    ],
                }),
                has_public_transfer: false,
                version: 349179418,
                contents: hex!("0ed5840c06ba5b53ed19307535a6d16ea7d8766532d3e1fc3f700aa50ffa18866000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000020290decd9548b62a8d60345a988386fc84ba6bc95484008f6362f93160ef3e563").as_slice().into(),
            }),
            owner: Owner::ObjectOwner(
                hex!("78f2ce59629d065e94d2cd7a457d972937ba428ae186413cc2a08c8431f9e804").into(),
            ),
            previous_transaction: Digest(
                hex!("90d9f071c0ddcac62b8ac32cd7645c895821a0f5a1af602aa0db8cfe6b30d4f5").into(),
            ),
            storage_rebate: 2348400,
        };

        let digest = object.digest();

        panic!("digest: {}", digest);
    }

    #[test]
    fn calculate_hash() {
        let parent_address =
            hex!("78f2ce59629d065e94d2cd7a457d972937ba428ae186413cc2a08c8431f9e804");
        let this_address = hex!("e809eb6e1c3b1077f08e3a1c8228a8e0d3c3480895a315e7911cc206886bdaed");

        let key_bytes = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 3,
        ];

        // let key_bytes = [[key_bytes.len() as u8].as_slice(), &key_bytes].concat();

        let hash = calculate_dynamic_field_key(parent_address, &key_bytes);

        panic!(
            "hash: {}, this: {}",
            Bytes::<HexPrefixed>::new(hash),
            Bytes::<HexPrefixed>::new(this_address.to_vec())
        );
    }
}
