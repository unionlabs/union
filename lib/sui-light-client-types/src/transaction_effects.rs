use serde::{Deserialize, Serialize};

use crate::{checkpoint_summary::GasCostSummary, digest::Digest, ObjectID, ObjectRef, SuiAddress};

/// The response from processing a transaction or a certified transaction
#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct TransactionEffectsV1 {
    /// The status of the execution
    pub status: ExecutionStatus,
    /// The epoch when this transaction was executed.
    pub executed_epoch: u64,
    pub gas_used: GasCostSummary,
    /// The version that every modified (mutated or deleted) object had before it was modified by
    /// this transaction.
    pub modified_at_versions: Vec<(ObjectID, u64)>,
    /// The object references of the shared objects used in this transaction. Empty if no shared objects were used.
    pub shared_objects: Vec<ObjectRef>,
    /// The transaction digest
    pub transaction_digest: Digest,

    pub created: Vec<(ObjectRef, Owner)>,
    /// ObjectRef and owner of mutated objects, including gas object.
    pub mutated: Vec<(ObjectRef, Owner)>,
    /// ObjectRef and owner of objects that are unwrapped in this transaction.
    /// Unwrapped objects are objects that were wrapped into other objects in the past,
    /// and just got extracted out.
    pub unwrapped: Vec<(ObjectRef, Owner)>,
    /// Object Refs of objects now deleted (the new refs).
    pub deleted: Vec<ObjectRef>,
    /// Object refs of objects previously wrapped in other objects but now deleted.
    pub unwrapped_then_deleted: Vec<ObjectRef>,
    /// Object refs of objects now wrapped in other objects.
    pub wrapped: Vec<ObjectRef>,
    /// The updated gas object reference. Have a dedicated field for convenient access.
    /// It's also included in mutated.
    pub gas_object: (ObjectRef, Owner),
    /// The digest of the events emitted during execution,
    /// can be None if the transaction does not emit any event.
    pub events_digest: Option<Digest>,
    /// The set of transaction digests this transaction depends on.
    pub dependencies: Vec<Digest>,
}

#[derive(Eq, PartialEq, Debug, Clone, Deserialize, Serialize, Hash, Ord, PartialOrd)]
pub enum Owner {
    /// Object is exclusively owned by a single address, and is mutable.
    AddressOwner(SuiAddress),
    /// Object is exclusively owned by a single object, and is mutable.
    /// The object ID is converted to SuiAddress as SuiAddress is universal.
    ObjectOwner(SuiAddress),
    /// Object is shared, can be used by any address, and is mutable.
    Shared {
        /// The version at which the object became shared
        initial_shared_version: u64,
    },
    /// Object is immutable, and hence ownership doesn't matter.
    Immutable,
    /// Object is sequenced via consensus. Ownership is managed by the configured authenticator.
    ///
    /// Note: wondering what happened to `V1`? `Shared` above was the V1 of consensus objects.
    ConsensusV2 {
        /// The version at which the object most recently became a consensus object.
        /// This serves the same function as `initial_shared_version`, except it may change
        /// if the object's Owner type changes.
        start_version: u64,
        /// The authentication mode of the object
        authenticator: Box<Authenticator>,
    },
}

#[derive(Eq, PartialEq, Debug, Clone, Copy, Deserialize, Serialize, Hash, Ord, PartialOrd)]
pub enum Authenticator {
    /// The contained SuiAddress exclusively has all permissions: read, write, delete, transfer
    SingleOwner(SuiAddress),
}

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum ExecutionStatus {
    // We don't care about the failure case
    Success,
}
