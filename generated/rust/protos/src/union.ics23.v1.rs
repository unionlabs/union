// @generated
/// *
/// ExistenceProof takes a key and a value and a set of steps to perform on it.
/// The result of peforming all these steps will provide a "root hash", which can
/// be compared to the value in a header.
///
/// Since it is computationally infeasible to produce a hash collission for any of the used
/// cryptographic hash functions, if someone can provide a series of operations to transform
/// a given key and value into a root hash that matches some trusted root, these key and values
/// must be in the referenced merkle tree.
///
/// The only possible issue is maliablity in LeafOp, such as providing extra prefix data,
/// which should be controlled by a spec. Eg. with lengthOp as NONE,
/// prefix = FOO, key = BAR, value = CHOICE
/// and
/// prefix = F, key = OOBAR, value = CHOICE
/// would produce the same value.
///
/// With LengthOp this is tricker but not impossible. Which is why the "leafPrefixEqual" field
/// in the ProofSpec is valuable to prevent this mutability. And why all trees should
/// length-prefix the data before hashing it.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ExistenceProof {
    #[prost(bytes = "vec", tag = "1")]
    pub key: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub value: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "3")]
    pub leaf_prefix: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, repeated, tag = "4")]
    pub path: ::prost::alloc::vec::Vec<InnerOp>,
}
impl ::prost::Name for ExistenceProof {
    const NAME: &'static str = "ExistenceProof";
    const PACKAGE: &'static str = "union.ics23.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ics23.v1.{}", Self::NAME)
    }
}
/// NonExistenceProof takes a proof of two neighbors, one 3left of the desired key,
/// one right of the desired key. If both proofs are valid AND they are neighbors,
/// then there is no valid proof for the given key.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NonExistenceProof {
    /// TODO: remove this as unnecessary??? we prove a range
    #[prost(bytes = "vec", tag = "1")]
    pub key: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "2")]
    pub left: ::core::option::Option<ExistenceProof>,
    #[prost(message, optional, tag = "3")]
    pub right: ::core::option::Option<ExistenceProof>,
}
impl ::prost::Name for NonExistenceProof {
    const NAME: &'static str = "NonExistenceProof";
    const PACKAGE: &'static str = "union.ics23.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ics23.v1.{}", Self::NAME)
    }
}
/// *
/// InnerOp represents a merkle-proof step that is not a leaf.
/// It represents concatenating two children and hashing them to provide the next result.
///
/// The result of the previous step is passed in, so the signature of this op is:
/// innerOp(child) -> output
///
/// The result of applying InnerOp should be:
/// output = op.hash(op.prefix || child || op.suffix)
///
/// where the || operator is concatenation of binary data,
/// and child is the result of hashing all the tree below this step.
///
/// Any special data, like prepending child with the length, or prepending the entire operation with
/// some value to differentiate from leaf nodes, should be included in prefix and suffix.
/// If either of prefix or suffix is empty, we just treat it as an empty string
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InnerOp {
    #[prost(bytes = "vec", tag = "1")]
    pub prefix: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub suffix: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for InnerOp {
    const NAME: &'static str = "InnerOp";
    const PACKAGE: &'static str = "union.ics23.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ics23.v1.{}", Self::NAME)
    }
}
// @@protoc_insertion_point(module)
