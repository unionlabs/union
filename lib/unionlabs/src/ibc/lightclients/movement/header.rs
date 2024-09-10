use macros::model;

use crate::{
    aptos::{
        state_proof::{StateProof, TryFromStateProofError},
        transaction_proof::{TransactionInfoWithProof, TryFromTransactionInfoWithProofError},
    },
    errors::{required, MissingField},
    ibc::{
        core::client::height::Height,
        lightclients::ethereum::{
            account_proof::{AccountProof, TryFromAccountProofError},
            storage_proof::{StorageProof, TryFromStorageProofError},
        },
    },
};

#[model(proto(
    raw(protos::union::ibc::lightclients::movement::v1::Header),
    into,
    from
))]
pub struct Header {
    pub l1_height: Height,
    pub trusted_height: Height,
    pub state_proof: StateProof,
    pub tx_proof: TransactionInfoWithProof,
    /// Proof that the hash of the `StateProof` is committed to L1
    pub state_proof_hash_proof: StorageProof,
    /// Proof of state of the settlement contract on L1
    pub settlement_contract_proof: AccountProof,
    pub new_height: u64,
}

impl From<Header> for protos::union::ibc::lightclients::movement::v1::Header {
    fn from(value: Header) -> Self {
        Self {
            l1_height: Some(value.l1_height.into()),
            trusted_height: Some(value.trusted_height.into()),
            state_proof: Some(value.state_proof.into()),
            tx_proof: Some(value.tx_proof.into()),
            state_proof_hash_proof: Some(value.state_proof_hash_proof.into()),
            settlement_contract_proof: Some(value.settlement_contract_proof.into()),
            new_height: value.new_height,
        }
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TryFromHeaderError {
    #[error(transparent)]
    MissingField(#[from] MissingField),
    #[error("invalid state proof")]
    StateProof(#[from] TryFromStateProofError),
    #[error("invalid tx proof")]
    TxProof(#[from] TryFromTransactionInfoWithProofError),
    #[error("invalid state proof hash proof")]
    StateProofHashProof(#[from] TryFromStorageProofError),
    #[error("invalid settlement contract proof")]
    SettlementContractProof(#[from] TryFromAccountProofError),
}

impl TryFrom<protos::union::ibc::lightclients::movement::v1::Header> for Header {
    type Error = TryFromHeaderError;

    fn try_from(
        value: protos::union::ibc::lightclients::movement::v1::Header,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            l1_height: required!(value.l1_height)?.into(),
            trusted_height: required!(value.trusted_height)?.into(),
            state_proof: required!(value.state_proof)?.try_into()?,
            tx_proof: required!(value.tx_proof)?.try_into()?,
            state_proof_hash_proof: required!(value.state_proof_hash_proof)?.try_into()?,
            settlement_contract_proof: required!(value.settlement_contract_proof)?.try_into()?,
            new_height: value.new_height,
        })
    }
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use super::*;
    use crate::{
        encoding::{DecodeAs, Proto},
        google::protobuf::any::Any,
        ibc::lightclients::wasm,
    };

    #[test]
    fn proto_decode() {
        let bz = hex!("0a0210011202100d1ae4010adf010ada010ab50108011a200bf2c3eeff4a624cd4ce97f18af615c2d94b4f007fd09b9c482b5366dd5dd2192220046e01adaf1c1bd7805d1ef7385e6399a913b8d2c5c1927f04c076b4160b016f288a0130d5d6d4d4eeb888033a610801125d0a5b0a20d1126ce48bd65fb72190dbd9a6eaa65ba973f1e1664ac0cfba4db1d071fd0c3612320a3086fb211f41a07c6399ccc6ab3a8fe568fb0f574ce1b811896c44c6da4f267d543c6cac9fb8f4e9b92a3b809eefb91cbd1880c2d72f122000000000000000000000000000000000000000000000000000000000000000001200120022bd030a20fe64accd475b2ed5e58d856d0765d3cae338e807d9636a0b19ef9457037380f00a2065c0370ceee1ef5adaa85f91ae33dd192aa049d734aafdfc0341bfe5e37ac00b0a20c6ae59a74870796c4331aaa80dc0554d15fcb7a324868654bd046f52a7cdb3890a207ff0d6738e83dd48d8b8d05ac8211c91883d318cedef72b16c8b17e3f4c57d320a20ee7e340551ccd5f30b2aecb382be10512ea8e05a70ad90bc1e7cfc9fe809200e0a20ae1eab32192ab3bacfcdb1529ee19ab4c27ffacae767af086ba777a8eacf1fde0a204a68f4c4f71c1f100b5419a5ff7b1505cee664255d51005faa8164b51ad89b780a20602cba26053f35e0de38d84c61914d6afbc3e2a08c34a651063c6cb3ef59714112aa011220d857831148f5e61c04f3e67ed93f04777c8615e555bdb5a8335e8c08042165da1a20414343554d554c41544f525f504c414345484f4c4445525f48415348000000002220afb6e14fe47d850fd0a7395bcfb997ffacf4715e0f895cc162c218e4a7564bc62a20e142d7896721539b32f737318f4565272c9d9f6eb05661e666c3c16754b23734322000000000000000000000000000000000000000000000000000000000000000002a440a2000000000000000000000000000000000000000000000000000000000000000001220000000000000000000000000000000000000000000000000000000000000000032220a2000000000000000000000000000000000000000000000000000000000000000003837");

        let header = Header::decode_as::<Proto>(bz.as_slice()).unwrap();

        dbg!(header);
    }
}
