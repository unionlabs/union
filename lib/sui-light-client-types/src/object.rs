use std::collections::BTreeMap;

use blake2::{Blake2b, Digest as _};
use serde::{Deserialize, Serialize};
use unionlabs_primitives::{encoding::Base58, Bytes, FixedBytes};

use crate::{digest::Digest, AccountAddress, ObjectID, Owner, SuiAddress};

#[derive(Serialize)]
pub struct ObjectInner {
    pub data: Data,
    pub owner: Owner,
    pub previous_transaction: Digest,
    pub storage_rebate: u64,
}

impl ObjectInner {
    pub fn digest(&self) -> Digest {
        let mut hasher = Blake2b::<typenum::U32>::new();
        hasher.update("Object::");
        bcs::serialize_into(&mut hasher, self).unwrap();
        Digest(FixedBytes::new(hasher.finalize().into()))
    }
}

#[derive(Serialize)]
pub enum Data {
    /// An object whose governing logic lives in a published Move module
    Move(MoveObject),
}

#[derive(Eq, PartialEq, Debug, Clone, Deserialize, Serialize, Hash)]
pub struct MoveObject {
    /// The type of this object. Immutable
    pub type_: MoveObjectType,
    /// DEPRECATED this field is no longer used to determine whether a tx can transfer this
    /// object. Instead, it is always calculated from the objects type when loaded in execution
    pub has_public_transfer: bool,
    /// Number that increases each time a tx takes this object as a mutable input
    /// This is a lamport timestamp, not a sequentially increasing version
    pub version: u64,
    /// BCS bytes of a Move struct value
    pub contents: Bytes,
}

#[derive(Eq, PartialEq, PartialOrd, Ord, Debug, Clone, Deserialize, Serialize, Hash)]
pub enum MoveObjectType {
    /// A type that is not `0x2::coin::Coin<T>`
    Other(StructTag),
    /// A SUI coin (i.e., `0x2::coin::Coin<0x2::sui::SUI>`)
    GasCoin,
    /// A record of a staked SUI coin (i.e., `0x3::staking_pool::StakedSui`)
    StakedSui,
    /// A non-SUI coin type (i.e., `0x2::coin::Coin<T> where T != 0x2::sui::SUI`)
    Coin(TypeTag),
    // NOTE: if adding a new type here, and there are existing on-chain objects of that
    // type with Other(_), that is ok, but you must hand-roll PartialEq/Eq/Ord/maybe Hash
    // to make sure the new type and Other(_) are interpreted consistently.
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Hash, Eq, Clone, PartialOrd, Ord)]
pub struct StructTag {
    pub address: AccountAddress,
    pub module: String,
    pub name: String,
    // alias for compatibility with old json serialized data.
    #[serde(rename = "type_args", alias = "type_params")]
    pub type_params: Vec<TypeTag>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Hash, Eq, Clone, PartialOrd, Ord)]
pub enum TypeTag {
    // alias for compatibility with old json serialized data.
    Bool,
    U8,
    U64,
    U128,
    Address,
    Signer,
    Vector(Box<TypeTag>),
    Struct(Box<StructTag>),
    U16,
    U32,
    U256,
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use unionlabs_primitives::encoding::{Base64, HexPrefixed};

    use super::*;
    #[test]
    fn object() {
        let bcs_bytes = Bytes::<Base64>::from_str("OEFp/jw2aT/lQdeNUyaKpj307NdfpvLogPgIZZvXeedAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAATsKdGhpcy1jaGFpbgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAADoAwAAAAAAAA==").unwrap();

        let decoded: (SuiAddress, Vec<u8>, Vec<u8>) = bcs::from_bytes(&bcs_bytes).unwrap();

        panic!("{decoded:?}");
    }
}
