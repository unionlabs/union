// @generated
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Proof {
    #[prost(int64, tag = "1")]
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub total: i64,
    #[prost(int64, tag = "2")]
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub index: i64,
    #[prost(bytes = "vec", tag = "3")]
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::base64"))]
    pub leaf_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", repeated, tag = "4")]
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::inner_base64"))]
    pub aunts: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
impl ::prost::Name for Proof {
    const NAME: &'static str = "Proof";
    const PACKAGE: &'static str = "tendermint.crypto";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.crypto.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValueOp {
    /// Encoded in ProofOp.Key.
    #[prost(bytes = "vec", tag = "1")]
    pub key: ::prost::alloc::vec::Vec<u8>,
    /// To encode in ProofOp.Data
    #[prost(message, optional, tag = "2")]
    pub proof: ::core::option::Option<Proof>,
}
impl ::prost::Name for ValueOp {
    const NAME: &'static str = "ValueOp";
    const PACKAGE: &'static str = "tendermint.crypto";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.crypto.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DominoOp {
    #[prost(string, tag = "1")]
    pub key: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub input: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub output: ::prost::alloc::string::String,
}
impl ::prost::Name for DominoOp {
    const NAME: &'static str = "DominoOp";
    const PACKAGE: &'static str = "tendermint.crypto";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.crypto.{}", Self::NAME)
    }
}
/// ProofOp defines an operation used for calculating Merkle root
/// The data could be arbitrary format, providing nessecary data
/// for example neighbouring node hash
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProofOp {
    #[prost(string, tag = "1")]
    pub r#type: ::prost::alloc::string::String,
    #[prost(bytes = "vec", tag = "2")]
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::base64"))]
    pub key: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "3")]
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::base64"))]
    pub data: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for ProofOp {
    const NAME: &'static str = "ProofOp";
    const PACKAGE: &'static str = "tendermint.crypto";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.crypto.{}", Self::NAME)
    }
}
/// ProofOps is Merkle proof defined by the list of ProofOps
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProofOps {
    #[prost(message, repeated, tag = "1")]
    pub ops: ::prost::alloc::vec::Vec<ProofOp>,
}
impl ::prost::Name for ProofOps {
    const NAME: &'static str = "ProofOps";
    const PACKAGE: &'static str = "tendermint.crypto";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.crypto.{}", Self::NAME)
    }
}
/// PublicKey defines the keys available for use with Validators
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PublicKey {
    #[prost(oneof = "public_key::Sum", tags = "1, 2, 3, 4")]
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub sum: ::core::option::Option<public_key::Sum>,
}
/// Nested message and enum types in `PublicKey`.
pub mod public_key {
    #[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    #[cfg_attr(feature = "serde", serde(tag = "type", content = "value"))]
    pub enum Sum {
        #[prost(bytes, tag = "1")]
        #[serde(rename = "tendermint/PubKeyEd25519")]
        Ed25519(#[serde(with = "::serde_utils::base64")] ::prost::alloc::vec::Vec<u8>),
        #[prost(bytes, tag = "2")]
        #[serde(rename = "tendermint/PubKeySecp256k1")]
        Secp256k1(#[serde(with = "::serde_utils::base64")] ::prost::alloc::vec::Vec<u8>),
        #[prost(bytes, tag = "3")]
        #[serde(rename = "tendermint/PubKeyBn254")]
        Bn254(#[serde(with = "::serde_utils::base64")] ::prost::alloc::vec::Vec<u8>),
        #[prost(bytes, tag = "4")]
        #[serde(rename = "cometbft/PubKeyBls12_381")]
        Bls12_381(#[serde(with = "::serde_utils::base64")] ::prost::alloc::vec::Vec<u8>),
    }
}
impl ::prost::Name for PublicKey {
    const NAME: &'static str = "PublicKey";
    const PACKAGE: &'static str = "tendermint.crypto";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.crypto.{}", Self::NAME)
    }
}
// @@protoc_insertion_point(module)
