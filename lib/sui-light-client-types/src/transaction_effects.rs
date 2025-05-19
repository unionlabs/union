use blake2::{Blake2b, Digest as _};
use serde::{Deserialize, Serialize};

use crate::{
    checkpoint_summary::GasCostSummary, digest::Digest, ObjectID, ObjectRef, Owner, SuiAddress,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub enum TransactionEffects {
    V1(TransactionEffectsV1),
    V2(TransactionEffectsV2),
}

/// The response from processing a transaction or a certified transaction
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct TransactionEffectsV2 {
    /// The status of the execution
    pub status: ExecutionStatus,
    /// The epoch when this transaction was executed.
    pub executed_epoch: u64,
    pub gas_used: GasCostSummary,
    /// The transaction digest
    pub transaction_digest: Digest,
    /// The updated gas object reference, as an index into the `changed_objects` vector.
    /// Having a dedicated field for convenient access.
    /// System transaction that don't require gas will leave this as None.
    pub gas_object_index: Option<u32>,
    /// The digest of the events emitted during execution,
    /// can be None if the transaction does not emit any event.
    pub events_digest: Option<Digest>,
    /// The set of transaction digests this transaction depends on.
    pub dependencies: Vec<Digest>,

    /// The version number of all the written Move objects by this transaction.
    pub lamport_version: u64,
    /// Objects whose state are changed in the object store.
    /// This field should not be exposed to the public API.
    /// Otherwise it will make it harder to use effects of different versions.
    pub changed_objects: Vec<(ObjectID, EffectsObjectChange)>,
    /// Shared objects that are not mutated in this transaction. Unlike owned objects,
    /// read-only shared objects' version are not committed in the transaction,
    /// and in order for a node to catch up and execute it without consensus sequencing,
    /// the version needs to be committed in the effects.
    pub unchanged_shared_objects: Vec<(ObjectID, UnchangedSharedKind)>,
    /// Auxiliary data that are not protocol-critical, generated as part of the effects but are stored separately.
    /// Storing it separately allows us to avoid bloating the effects with data that are not critical.
    /// It also provides more flexibility on the format and type of the data.
    pub aux_data_digest: Option<Digest>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct EffectsObjectChange {
    // input_state and output_state are the core fields that's required by
    // the protocol as it tells how an object changes on-chain.
    /// State of the object in the store prior to this transaction.
    pub input_state: ObjectIn,
    /// State of the object in the store after this transaction.
    pub output_state: ObjectOut,

    /// Whether this object ID is created or deleted in this transaction.
    /// This information isn't required by the protocol but is useful for providing more detailed
    /// semantics on object changes.
    pub id_operation: IDOperation,
}

pub type VersionDigest = (u64, Digest);

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub enum ObjectIn {
    NotExist,
    /// The old version, digest and owner.
    Exist((VersionDigest, Owner)),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub enum ObjectOut {
    /// Same definition as in ObjectIn.
    NotExist,
    /// Any written object, including all of mutated, created, unwrapped today.
    ObjectWrite((Digest, Owner)),
    /// Packages writes need to be tracked separately with version because
    /// we don't use lamport version for package publish and upgrades.
    PackageWrite(VersionDigest),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub enum UnchangedSharedKind {
    /// Read-only shared objects from the input. We don't really need ObjectDigest
    /// for protocol correctness, but it will make it easier to verify untrusted read.
    ReadOnlyRoot(VersionDigest),
    /// Objects with ended consensus streams that appear mutably/owned in the input.
    MutateConsensusStreamEnded(u64),
    /// Objects with ended consensus streams objects that appear as read-only in the input.
    ReadConsensusStreamEnded(u64),
    /// Shared objects in cancelled transaction. The sequence number embed cancellation reason.
    Cancelled(u64),
    /// Read of a per-epoch config object that should remain the same during an epoch.
    PerEpochConfig,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub enum IDOperation {
    None,
    Created,
    Deleted,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub enum ExecutionStatus {
    // We don't care about the failure case
    Success,
}

impl TransactionEffects {
    pub fn digest(&self) -> Digest {
        let mut hasher = Blake2b::<typenum::U32>::new();
        hasher.update("TransactionEffects::");
        bcs::serialize_into(&mut hasher, self).unwrap();
        Digest(hasher.finalize().into())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use blake2::{Blake2b, Digest as _};
    use hex_literal::hex;
    use unionlabs_primitives::{encoding::Base58, Bytes, FixedBytes};

    use super::*;
    use crate::U64;

    fn digest<T: Serialize>(effects: &T) -> Bytes<Base58> {
        let mut hasher = Blake2b::<typenum::U32>::new();
        hasher.update("TransactionEffects::");
        bcs::serialize_into(&mut hasher, effects).unwrap();
        Bytes::new(hasher.finalize().to_vec())
    }

    #[test]
    fn effects_digest() {
        let effect: TransactionEffectsV2 = bcs::from_bytes(&[
            0, 220, 2, 0, 0, 0, 0, 0, 0, 64, 66, 15, 0, 0, 0, 0, 0, 128, 1, 231, 0, 0, 0, 0, 0, 24,
            217, 167, 0, 0, 0, 0, 0, 8, 178, 1, 0, 0, 0, 0, 0, 32, 144, 217, 240, 113, 192, 221,
            202, 198, 43, 138, 195, 44, 215, 100, 92, 137, 88, 33, 160, 245, 161, 175, 96, 42, 160,
            219, 140, 254, 107, 48, 212, 245, 1, 2, 0, 0, 0, 1, 32, 133, 207, 55, 219, 159, 84,
            195, 241, 91, 250, 21, 133, 99, 82, 60, 233, 97, 193, 182, 8, 122, 141, 134, 88, 35, 3,
            17, 65, 178, 22, 82, 166, 2, 32, 42, 70, 130, 79, 215, 174, 173, 1, 63, 52, 130, 111,
            175, 183, 255, 195, 100, 105, 62, 178, 94, 47, 154, 112, 162, 18, 40, 56, 164, 157,
            130, 189, 32, 51, 156, 147, 167, 134, 208, 102, 124, 172, 0, 134, 78, 199, 178, 131,
            237, 245, 37, 26, 186, 166, 231, 224, 52, 3, 106, 11, 254, 113, 8, 199, 170, 26, 14,
            208, 20, 0, 0, 0, 0, 7, 14, 213, 132, 12, 6, 186, 91, 83, 237, 25, 48, 117, 53, 166,
            209, 110, 167, 216, 118, 101, 50, 211, 225, 252, 63, 112, 10, 165, 15, 250, 24, 134, 1,
            25, 14, 208, 20, 0, 0, 0, 0, 32, 16, 218, 129, 70, 217, 4, 60, 90, 26, 96, 184, 237,
            75, 72, 241, 55, 145, 44, 190, 103, 180, 196, 180, 146, 254, 218, 166, 60, 107, 82,
            196, 134, 1, 120, 242, 206, 89, 98, 157, 6, 94, 148, 210, 205, 122, 69, 125, 151, 41,
            55, 186, 66, 138, 225, 134, 65, 60, 194, 160, 140, 132, 49, 249, 232, 4, 1, 32, 74,
            138, 106, 49, 85, 141, 3, 209, 199, 219, 183, 95, 116, 93, 175, 250, 4, 157, 117, 222,
            144, 132, 69, 207, 83, 169, 201, 240, 151, 55, 232, 186, 1, 120, 242, 206, 89, 98, 157,
            6, 94, 148, 210, 205, 122, 69, 125, 151, 41, 55, 186, 66, 138, 225, 134, 65, 60, 194,
            160, 140, 132, 49, 249, 232, 4, 0, 19, 102, 88, 151, 255, 2, 98, 53, 8, 186, 19, 65,
            75, 20, 41, 191, 61, 54, 114, 54, 178, 209, 219, 108, 161, 122, 211, 220, 147, 184,
            172, 34, 1, 222, 13, 208, 20, 0, 0, 0, 0, 32, 227, 42, 103, 26, 45, 6, 131, 210, 165,
            200, 57, 109, 137, 251, 240, 225, 78, 162, 22, 69, 70, 56, 149, 74, 199, 155, 120, 219,
            161, 20, 234, 207, 1, 120, 242, 206, 89, 98, 157, 6, 94, 148, 210, 205, 122, 69, 125,
            151, 41, 55, 186, 66, 138, 225, 134, 65, 60, 194, 160, 140, 132, 49, 249, 232, 4, 1,
            32, 227, 110, 206, 235, 44, 176, 253, 218, 229, 42, 119, 124, 90, 120, 100, 149, 40,
            95, 207, 55, 10, 168, 243, 167, 99, 79, 97, 148, 178, 246, 36, 114, 1, 120, 242, 206,
            89, 98, 157, 6, 94, 148, 210, 205, 122, 69, 125, 151, 41, 55, 186, 66, 138, 225, 134,
            65, 60, 194, 160, 140, 132, 49, 249, 232, 4, 0, 84, 135, 98, 216, 125, 140, 205, 76,
            14, 62, 218, 32, 101, 162, 226, 203, 72, 58, 106, 110, 124, 25, 208, 130, 55, 135, 87,
            29, 79, 204, 62, 92, 1, 25, 14, 208, 20, 0, 0, 0, 0, 32, 135, 121, 101, 33, 219, 23,
            39, 111, 237, 233, 70, 187, 27, 14, 108, 84, 57, 97, 188, 147, 22, 221, 57, 173, 5,
            182, 173, 187, 171, 166, 142, 63, 0, 35, 42, 79, 126, 180, 194, 171, 245, 6, 19, 22,
            112, 67, 115, 253, 75, 255, 189, 41, 119, 41, 64, 109, 157, 4, 1, 41, 49, 64, 95, 89,
            11, 1, 32, 232, 236, 21, 78, 48, 1, 21, 0, 31, 127, 241, 252, 184, 254, 206, 102, 54,
            232, 135, 227, 119, 232, 79, 121, 136, 19, 178, 115, 118, 1, 50, 113, 0, 35, 42, 79,
            126, 180, 194, 171, 245, 6, 19, 22, 112, 67, 115, 253, 75, 255, 189, 41, 119, 41, 64,
            109, 157, 4, 1, 41, 49, 64, 95, 89, 11, 0, 113, 24, 228, 255, 245, 45, 70, 232, 217,
            153, 139, 146, 194, 249, 162, 186, 141, 245, 229, 24, 80, 123, 146, 37, 15, 34, 26,
            248, 98, 131, 115, 24, 0, 1, 32, 59, 144, 163, 202, 108, 230, 114, 252, 185, 28, 223,
            24, 91, 72, 175, 69, 220, 252, 109, 142, 78, 16, 204, 181, 198, 206, 73, 27, 212, 196,
            65, 36, 1, 255, 29, 114, 216, 241, 7, 253, 128, 187, 74, 30, 146, 90, 136, 108, 181,
            52, 232, 166, 136, 136, 54, 17, 209, 252, 142, 210, 122, 50, 205, 34, 100, 1, 170, 30,
            194, 226, 246, 61, 91, 145, 15, 135, 213, 1, 138, 247, 170, 188, 236, 180, 115, 101,
            16, 30, 29, 99, 253, 103, 188, 220, 58, 94, 202, 211, 1, 25, 14, 208, 20, 0, 0, 0, 0,
            32, 53, 21, 13, 174, 55, 237, 20, 64, 144, 242, 67, 42, 201, 9, 244, 128, 160, 26, 9,
            183, 200, 198, 237, 179, 131, 157, 125, 120, 218, 162, 251, 45, 2, 219, 13, 208, 20, 0,
            0, 0, 0, 1, 32, 17, 229, 209, 13, 164, 177, 110, 60, 155, 206, 147, 209, 129, 61, 253,
            148, 47, 23, 8, 109, 232, 124, 173, 14, 235, 230, 61, 51, 161, 217, 77, 247, 2, 219,
            13, 208, 20, 0, 0, 0, 0, 0, 213, 148, 211, 215, 171, 187, 149, 143, 72, 55, 79, 205,
            93, 98, 51, 86, 140, 110, 43, 55, 253, 33, 167, 103, 131, 35, 184, 99, 139, 133, 160,
            29, 1, 25, 14, 208, 20, 0, 0, 0, 0, 32, 252, 73, 219, 116, 56, 190, 0, 109, 76, 235,
            135, 201, 47, 29, 232, 177, 191, 128, 11, 18, 163, 231, 229, 109, 208, 12, 45, 108,
            147, 243, 255, 253, 1, 120, 242, 206, 89, 98, 157, 6, 94, 148, 210, 205, 122, 69, 125,
            151, 41, 55, 186, 66, 138, 225, 134, 65, 60, 194, 160, 140, 132, 49, 249, 232, 4, 1,
            32, 246, 190, 171, 176, 85, 209, 255, 97, 35, 19, 5, 16, 31, 170, 171, 94, 152, 25,
            174, 251, 250, 83, 86, 31, 68, 249, 83, 143, 174, 132, 175, 236, 1, 120, 242, 206, 89,
            98, 157, 6, 94, 148, 210, 205, 122, 69, 125, 151, 41, 55, 186, 66, 138, 225, 134, 65,
            60, 194, 160, 140, 132, 49, 249, 232, 4, 0, 232, 9, 235, 110, 28, 59, 16, 119, 240,
            142, 58, 28, 130, 40, 168, 224, 211, 195, 72, 8, 149, 163, 21, 231, 145, 28, 194, 6,
            136, 107, 218, 237, 0, 1, 32, 174, 171, 63, 71, 5, 23, 255, 8, 24, 231, 104, 182, 30,
            67, 94, 4, 11, 128, 225, 102, 58, 78, 43, 6, 15, 106, 240, 48, 192, 183, 32, 190, 1,
            120, 242, 206, 89, 98, 157, 6, 94, 148, 210, 205, 122, 69, 125, 151, 41, 55, 186, 66,
            138, 225, 134, 65, 60, 194, 160, 140, 132, 49, 249, 232, 4, 1, 0, 0,
        ])
        .unwrap();

        assert_eq!(
            Bytes::<Base58>::from_str("Fm3buiyN4vxDhnLiwvwDCJ1ZiohH3QCsVGiPCpC96RdR").unwrap(),
            digest(&TransactionEffects::V2(effect))
        );
    }
}
