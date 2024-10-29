use unionlabs::{
    aptos::{state_proof::StateProof, transaction_proof::TransactionInfoWithProof},
    ibc::{
        core::client::height::Height,
        lightclients::ethereum::{account_proof::AccountProof, storage_proof::StorageProof},
    },
};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Header {
    pub l1_height: Height,
    pub trusted_height: Height,
    pub state_proof: StateProof,
    pub tx_index: u64,
    pub tx_proof: TransactionInfoWithProof,
    /// Proof that the hash of the `StateProof` is committed to L1
    pub state_proof_hash_proof: StorageProof,
    /// Proof of state of the settlement contract on L1
    pub settlement_contract_proof: AccountProof,
    pub new_height: u64,
}

#[cfg(feature = "proto")]
pub mod proto {
    use unionlabs::{
        aptos::{
            state_proof::TryFromStateProofError,
            transaction_proof::TryFromTransactionInfoWithProofError,
        },
        errors::{required, MissingField},
        ibc::lightclients::ethereum::{
            account_proof::TryFromAccountProofError, storage_proof::TryFromStorageProofError,
        },
        impl_proto_via_try_from_into,
    };

    use crate::Header;

    impl_proto_via_try_from_into!(Header => protos::union::ibc::lightclients::movement::v1::Header);

    impl From<Header> for protos::union::ibc::lightclients::movement::v1::Header {
        fn from(value: Header) -> Self {
            Self {
                l1_height: Some(value.l1_height.into()),
                trusted_height: Some(value.trusted_height.into()),
                state_proof: Some(value.state_proof.into()),
                tx_index: value.tx_index,
                tx_proof: Some(value.tx_proof.into()),
                state_proof_hash_proof: Some(value.state_proof_hash_proof.into()),
                settlement_contract_proof: Some(value.settlement_contract_proof.into()),
                new_height: value.new_height,
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum Error {
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
        type Error = Error;

        fn try_from(
            value: protos::union::ibc::lightclients::movement::v1::Header,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                l1_height: required!(value.l1_height)?.into(),
                trusted_height: required!(value.trusted_height)?.into(),
                state_proof: required!(value.state_proof)?.try_into()?,
                tx_index: value.tx_index,
                tx_proof: required!(value.tx_proof)?.try_into()?,
                state_proof_hash_proof: required!(value.state_proof_hash_proof)?.try_into()?,
                settlement_contract_proof: required!(value.settlement_contract_proof)?
                    .try_into()?,
                new_height: value.new_height,
            })
        }
    }
}
