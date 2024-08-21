use serde::{Deserialize, Serialize};
use sha2::Digest;
use sha3::Sha3_256;

use crate::{hash_value::HashValue, types::AccountAddress};

/// `TransactionInfo` is the object we store in the transaction accumulator. It consists of the
/// transaction as well as the execution result of this transaction.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum TransactionInfo {
    V0(TransactionInfoV0),
}

impl TransactionInfo {
    pub fn hash(&self) -> HashValue {
        let mut state = Sha3_256::new();
        state.update(
            Sha3_256::new()
                .chain_update("APTOS::TransactionInfo")
                .finalize(),
        );
        bcs::serialize_into(&mut state, &self).expect("expected to be able to serialize");
        HashValue(state.finalize().into())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TransactionInfoV0 {
    /// The amount of gas used.
    gas_used: u64,

    /// The vm status. If it is not `Executed`, this will provide the general error class. Execution
    /// failures and Move abort's receive more detailed information. But other errors are generally
    /// categorized with no status code or other information
    status: ExecutionStatus,

    /// The hash of this transaction.
    transaction_hash: HashValue,

    /// The root hash of Merkle Accumulator storing all events emitted during this transaction.
    event_root_hash: HashValue,

    /// The hash value summarizing all changes caused to the world state by this transaction.
    /// i.e. hash of the output write set.
    state_change_hash: HashValue,

    /// The root hash of the Sparse Merkle Tree describing the world state at the end of this
    /// transaction. Depending on the protocol configuration, this can be generated periodical
    /// only, like per block.
    state_checkpoint_hash: Option<HashValue>,

    /// Potentially summarizes all evicted items from state. Always `None` for now.
    state_cemetery_hash: Option<HashValue>,
}

impl TransactionInfoV0 {
    pub fn hash(&self) -> HashValue {
        let mut state = Sha3_256::new();
        state.update(
            Sha3_256::new()
                .chain_update("APTOS::TransactionInfoV0")
                .finalize(),
        );
        bcs::serialize_into(&mut state, &self).expect("expected to be able to serialize");
        HashValue(state.finalize().into())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Success,
    OutOfGas,
    MoveAbort {
        location: AbortLocation,
        code: u64,
        info: Option<AbortInfo>,
    },
    ExecutionFailure {
        location: AbortLocation,
        function: u16,
        code_offset: u16,
    },
    MiscellaneousError(Option<u64>),
}

/// An `AbortLocation` specifies where a Move program `abort` occurred, either in a function in
/// a module, or in a script
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum AbortLocation {
    /// Indicates `abort` occurred in the specified module
    Module(ModuleId),
    /// Indicates the `abort` occurred in a script
    Script,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct AbortInfo {
    pub reason_name: String,
    pub description: String,
}

/// Represents the initial key into global storage where we first index by the address, and then
/// the struct tag. The struct fields are public to support pattern matching.
#[derive(Serialize, Deserialize, Debug, PartialEq, Hash, Eq, Clone, PartialOrd, Ord)]
pub struct ModuleId {
    pub address: AccountAddress,
    pub name: String,
}
