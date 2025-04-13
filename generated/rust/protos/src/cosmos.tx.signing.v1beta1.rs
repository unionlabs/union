// @generated
/// SignatureDescriptors wraps multiple SignatureDescriptor's.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SignatureDescriptors {
    /// signatures are the signature descriptors
    #[prost(message, repeated, tag = "1")]
    pub signatures: ::prost::alloc::vec::Vec<SignatureDescriptor>,
}
impl ::prost::Name for SignatureDescriptors {
    const NAME: &'static str = "SignatureDescriptors";
    const PACKAGE: &'static str = "cosmos.tx.signing.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.tx.signing.v1beta1.{}", Self::NAME)
    }
}
/// SignatureDescriptor is a convenience type which represents the full data for
/// a signature including the public key of the signer, signing modes and the
/// signature itself. It is primarily used for coordinating signatures between
/// clients.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SignatureDescriptor {
    /// public_key is the public key of the signer
    #[prost(message, optional, tag = "1")]
    pub public_key: ::core::option::Option<super::super::super::super::google::protobuf::Any>,
    #[prost(message, optional, tag = "2")]
    pub data: ::core::option::Option<signature_descriptor::Data>,
    /// sequence is the sequence of the account, which describes the
    /// number of committed transactions signed by a given address. It is used to prevent
    /// replay attacks.
    #[prost(uint64, tag = "3")]
    pub sequence: u64,
}
/// Nested message and enum types in `SignatureDescriptor`.
pub mod signature_descriptor {
    /// Data represents signature data
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Data {
        /// sum is the oneof that specifies whether this represents single or multi-signature data
        #[prost(oneof = "data::Sum", tags = "1, 2")]
        pub sum: ::core::option::Option<data::Sum>,
    }
    /// Nested message and enum types in `Data`.
    pub mod data {
        /// Single is the signature data for a single signer
        #[allow(clippy::derive_partial_eq_without_eq)]
        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct Single {
            /// mode is the signing mode of the single signer
            #[prost(enumeration = "super::super::SignMode", tag = "1")]
            pub mode: i32,
            /// signature is the raw signature bytes
            #[prost(bytes = "vec", tag = "2")]
            pub signature: ::prost::alloc::vec::Vec<u8>,
        }
        impl ::prost::Name for Single {
            const NAME: &'static str = "Single";
            const PACKAGE: &'static str = "cosmos.tx.signing.v1beta1";
            fn full_name() -> ::prost::alloc::string::String {
                ::prost::alloc::format!(
                    "cosmos.tx.signing.v1beta1.SignatureDescriptor.Data.{}",
                    Self::NAME
                )
            }
        }
        /// Multi is the signature data for a multisig public key
        #[allow(clippy::derive_partial_eq_without_eq)]
        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct Multi {
            /// bitarray specifies which keys within the multisig are signing
            #[prost(message, optional, tag = "1")]
            pub bitarray: ::core::option::Option<
                super::super::super::super::super::crypto::multisig::v1beta1::CompactBitArray,
            >,
            /// signatures is the signatures of the multi-signature
            #[prost(message, repeated, tag = "2")]
            pub signatures: ::prost::alloc::vec::Vec<super::Data>,
        }
        impl ::prost::Name for Multi {
            const NAME: &'static str = "Multi";
            const PACKAGE: &'static str = "cosmos.tx.signing.v1beta1";
            fn full_name() -> ::prost::alloc::string::String {
                ::prost::alloc::format!(
                    "cosmos.tx.signing.v1beta1.SignatureDescriptor.Data.{}",
                    Self::NAME
                )
            }
        }
        /// sum is the oneof that specifies whether this represents single or multi-signature data
        #[allow(clippy::derive_partial_eq_without_eq)]
        #[derive(Clone, PartialEq, ::prost::Oneof)]
        pub enum Sum {
            /// single represents a single signer
            #[prost(message, tag = "1")]
            Single(Single),
            /// multi represents a multisig signer
            #[prost(message, tag = "2")]
            Multi(Multi),
        }
    }
    impl ::prost::Name for Data {
        const NAME: &'static str = "Data";
        const PACKAGE: &'static str = "cosmos.tx.signing.v1beta1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "cosmos.tx.signing.v1beta1.SignatureDescriptor.{}",
                Self::NAME
            )
        }
    }
}
impl ::prost::Name for SignatureDescriptor {
    const NAME: &'static str = "SignatureDescriptor";
    const PACKAGE: &'static str = "cosmos.tx.signing.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.tx.signing.v1beta1.{}", Self::NAME)
    }
}
/// SignMode represents a signing mode with its own security guarantees.
///
/// This enum should be considered a registry of all known sign modes
/// in the Cosmos ecosystem. Apps are not expected to support all known
/// sign modes. Apps that would like to support custom  sign modes are
/// encouraged to open a small PR against this file to add a new case
/// to this SignMode enum describing their sign mode so that different
/// apps have a consistent version of this enum.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum SignMode {
    /// SIGN_MODE_UNSPECIFIED specifies an unknown signing mode and will be
    /// rejected.
    Unspecified = 0,
    /// SIGN_MODE_DIRECT specifies a signing mode which uses SignDoc and is
    /// verified with raw bytes from Tx.
    Direct = 1,
    /// SIGN_MODE_TEXTUAL is a future signing mode that will verify some
    /// human-readable textual representation on top of the binary representation
    /// from SIGN_MODE_DIRECT.
    ///
    /// Since: cosmos-sdk 0.50
    Textual = 2,
    /// SIGN_MODE_DIRECT_AUX specifies a signing mode which uses
    /// SignDocDirectAux. As opposed to SIGN_MODE_DIRECT, this sign mode does not
    /// require signers signing over other signers' `signer_info`.
    ///
    /// Since: cosmos-sdk 0.46
    DirectAux = 3,
    /// SIGN_MODE_LEGACY_AMINO_JSON is a backwards compatibility mode which uses
    /// Amino JSON and will be removed in the future.
    LegacyAminoJson = 127,
    /// SIGN_MODE_EIP_191 specifies the sign mode for EIP 191 signing on the Cosmos
    /// SDK. Ref: <https://eips.ethereum.org/EIPS/eip-191>
    ///
    /// Currently, SIGN_MODE_EIP_191 is registered as a SignMode enum variant,
    /// but is not implemented on the SDK by default. To enable EIP-191, you need
    /// to pass a custom `TxConfig` that has an implementation of
    /// `SignModeHandler` for EIP-191. The SDK may decide to fully support
    /// EIP-191 in the future.
    ///
    /// Since: cosmos-sdk 0.45.2
    Eip191 = 191,
}
impl SignMode {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            SignMode::Unspecified => "SIGN_MODE_UNSPECIFIED",
            SignMode::Direct => "SIGN_MODE_DIRECT",
            SignMode::Textual => "SIGN_MODE_TEXTUAL",
            SignMode::DirectAux => "SIGN_MODE_DIRECT_AUX",
            SignMode::LegacyAminoJson => "SIGN_MODE_LEGACY_AMINO_JSON",
            SignMode::Eip191 => "SIGN_MODE_EIP_191",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "SIGN_MODE_UNSPECIFIED" => Some(Self::Unspecified),
            "SIGN_MODE_DIRECT" => Some(Self::Direct),
            "SIGN_MODE_TEXTUAL" => Some(Self::Textual),
            "SIGN_MODE_DIRECT_AUX" => Some(Self::DirectAux),
            "SIGN_MODE_LEGACY_AMINO_JSON" => Some(Self::LegacyAminoJson),
            "SIGN_MODE_EIP_191" => Some(Self::Eip191),
            _ => None,
        }
    }
}
// @@protoc_insertion_point(module)
