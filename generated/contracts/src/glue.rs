pub use glue::*;
/// This module was auto-generated with ethers-rs Abigen.
/// More information at: <https://github.com/gakonst/ethers-rs>
#[allow(
    clippy::enum_variant_names,
    clippy::too_many_arguments,
    clippy::upper_case_acronyms,
    clippy::type_complexity,
    dead_code,
    non_camel_case_types
)]
pub mod glue {
    pub use super::super::shared_types::*;
    #[cfg(feature = "providers")]
    #[allow(deprecated)]
    #[cfg(feature = "providers")]
    fn __abi() -> ::ethers::core::abi::Abi {
        ::ethers::core::abi::ethabi::Contract {
            constructor: ::core::option::Option::None,
            functions: ::core::convert::From::from([
                (
                    ::std::borrow::ToOwned::to_owned("typesTelescope"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("typesTelescope"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                ],
                                            ),
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned(
                                            "struct UnionIbcLightclientsCometblsV1ClientState.Data",
                                        ),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![::ethers::core::abi::ethabi::ParamType::Bytes],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned(
                                            "struct UnionIbcLightclientsCometblsV1ConsensusState.Data",
                                        ),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                        ::std::vec![
                                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                                ::std::vec![
                                                                    ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                                    ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                                ],
                                                            ),
                                                            ::ethers::core::abi::ethabi::ParamType::String,
                                                            ::ethers::core::abi::ethabi::ParamType::Int(64usize),
                                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                                ::std::vec![
                                                                    ::ethers::core::abi::ethabi::ParamType::Int(64usize),
                                                                    ::ethers::core::abi::ethabi::ParamType::Int(64usize),
                                                                ],
                                                            ),
                                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                                ::std::vec![
                                                                    ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                                        ::std::vec![
                                                                            ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                                        ],
                                                                    ),
                                                                ],
                                                            ),
                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                        ],
                                                    ),
                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                        ::std::vec![
                                                            ::ethers::core::abi::ethabi::ParamType::Int(64usize),
                                                            ::ethers::core::abi::ethabi::ParamType::Int(32usize),
                                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                                ::std::vec![
                                                                    ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                                        ::std::vec![
                                                                            ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                                        ],
                                                                    ),
                                                                ],
                                                            ),
                                                            ::ethers::core::abi::ethabi::ParamType::Array(
                                                                ::std::boxed::Box::new(
                                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                                        ::std::vec![
                                                                            ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                                                ::std::vec![
                                                                                    ::ethers::core::abi::ethabi::ParamType::Int(64usize),
                                                                                    ::ethers::core::abi::ethabi::ParamType::Int(64usize),
                                                                                ],
                                                                            ),
                                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                                        ],
                                                                    ),
                                                                ),
                                                            ),
                                                        ],
                                                    ),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned(
                                            "struct UnionIbcLightclientsCometblsV1Header.Data",
                                        ),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                            ::ethers::core::abi::ethabi::ParamType::Int(64usize),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Int(64usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Int(64usize),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                        ::std::vec![
                                                            ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                        ],
                                                    ),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned(
                                            "struct TendermintTypesHeader.Data",
                                        ),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Int(64usize),
                                            ::ethers::core::abi::ethabi::ParamType::Int(32usize),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                        ::std::vec![
                                                            ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                        ],
                                                    ),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Array(
                                                ::std::boxed::Box::new(
                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                        ::std::vec![
                                                            ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                                ::std::vec![
                                                                    ::ethers::core::abi::ethabi::ParamType::Int(64usize),
                                                                    ::ethers::core::abi::ethabi::ParamType::Int(64usize),
                                                                ],
                                                            ),
                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                        ],
                                                    ),
                                                ),
                                            ),
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned(
                                            "struct TendermintTypesCommit.Data",
                                        ),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned(
                                            "struct IbcCoreClientV1Height.Data",
                                        ),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                            ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize),
                                            ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize),
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned(
                                            "struct OptimizedConsensusState",
                                        ),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Uint(128usize),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(128usize),
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("struct ProcessedMoment"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                            ::ethers::core::abi::ethabi::ParamType::Int(64usize),
                                            ::ethers::core::abi::ethabi::ParamType::Int(64usize),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                        ::std::vec![
                                                            ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                        ],
                                                    ),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned(
                                            "struct TendermintTypesCanonicalVote.Data",
                                        ),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Int(64usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Int(32usize),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Int(64usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Int(32usize),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Int(64usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Int(32usize),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Array(
                                                ::std::boxed::Box::new(
                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                        ::std::vec![
                                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                                ::std::vec![
                                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                                    ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                                ],
                                                            ),
                                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                                ::std::vec![
                                                                    ::ethers::core::abi::ethabi::ParamType::Array(
                                                                        ::std::boxed::Box::new(
                                                                            ::ethers::core::abi::ethabi::ParamType::Int(32usize),
                                                                        ),
                                                                    ),
                                                                    ::ethers::core::abi::ethabi::ParamType::Int(32usize),
                                                                    ::ethers::core::abi::ethabi::ParamType::Int(32usize),
                                                                    ::ethers::core::abi::ethabi::ParamType::Int(32usize),
                                                                    ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                                ],
                                                            ),
                                                            ::ethers::core::abi::ethabi::ParamType::Int(32usize),
                                                            ::ethers::core::abi::ethabi::ParamType::Int(32usize),
                                                        ],
                                                    ),
                                                ),
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Array(
                                                ::std::boxed::Box::new(
                                                    ::ethers::core::abi::ethabi::ParamType::String,
                                                ),
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Bool,
                                            ::ethers::core::abi::ethabi::ParamType::Bool,
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned(
                                            "struct IbcLightclientsTendermintV1ClientState.Data",
                                        ),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Int(64usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Int(64usize),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![::ethers::core::abi::ethabi::ParamType::Bytes],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned(
                                            "struct IbcLightclientsTendermintV1ConsensusState.Data",
                                        ),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                        ::std::vec![
                                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                                ::std::vec![
                                                                    ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                                    ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                                ],
                                                            ),
                                                            ::ethers::core::abi::ethabi::ParamType::String,
                                                            ::ethers::core::abi::ethabi::ParamType::Int(64usize),
                                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                                ::std::vec![
                                                                    ::ethers::core::abi::ethabi::ParamType::Int(64usize),
                                                                    ::ethers::core::abi::ethabi::ParamType::Int(64usize),
                                                                ],
                                                            ),
                                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                                ::std::vec![
                                                                    ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                                        ::std::vec![
                                                                            ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                                        ],
                                                                    ),
                                                                ],
                                                            ),
                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                        ],
                                                    ),
                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                        ::std::vec![
                                                            ::ethers::core::abi::ethabi::ParamType::Int(64usize),
                                                            ::ethers::core::abi::ethabi::ParamType::Int(32usize),
                                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                                ::std::vec![
                                                                    ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                                        ::std::vec![
                                                                            ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                                        ],
                                                                    ),
                                                                ],
                                                            ),
                                                            ::ethers::core::abi::ethabi::ParamType::Array(
                                                                ::std::boxed::Box::new(
                                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                                        ::std::vec![
                                                                            ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                                                ::std::vec![
                                                                                    ::ethers::core::abi::ethabi::ParamType::Int(64usize),
                                                                                    ::ethers::core::abi::ethabi::ParamType::Int(64usize),
                                                                                ],
                                                                            ),
                                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                                        ],
                                                                    ),
                                                                ),
                                                            ),
                                                        ],
                                                    ),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Array(
                                                        ::std::boxed::Box::new(
                                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                                ::std::vec![
                                                                    ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                                        ::std::vec![
                                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                                        ],
                                                                    ),
                                                                    ::ethers::core::abi::ethabi::ParamType::Int(64usize),
                                                                    ::ethers::core::abi::ethabi::ParamType::Int(64usize),
                                                                ],
                                                            ),
                                                        ),
                                                    ),
                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                        ::std::vec![
                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                                ::std::vec![
                                                                    ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                                    ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                                    ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                                ],
                                                            ),
                                                            ::ethers::core::abi::ethabi::ParamType::Int(64usize),
                                                            ::ethers::core::abi::ethabi::ParamType::Int(64usize),
                                                        ],
                                                    ),
                                                    ::ethers::core::abi::ethabi::ParamType::Int(64usize),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Array(
                                                        ::std::boxed::Box::new(
                                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                                ::std::vec![
                                                                    ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                                        ::std::vec![
                                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                                        ],
                                                                    ),
                                                                    ::ethers::core::abi::ethabi::ParamType::Int(64usize),
                                                                    ::ethers::core::abi::ethabi::ParamType::Int(64usize),
                                                                ],
                                                            ),
                                                        ),
                                                    ),
                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                        ::std::vec![
                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                                ::std::vec![
                                                                    ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                                    ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                                    ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                                ],
                                                            ),
                                                            ::ethers::core::abi::ethabi::ParamType::Int(64usize),
                                                            ::ethers::core::abi::ethabi::ParamType::Int(64usize),
                                                        ],
                                                    ),
                                                    ::ethers::core::abi::ethabi::ParamType::Int(64usize),
                                                ],
                                            ),
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned(
                                            "struct IbcLightclientsTendermintV1Header.Data",
                                        ),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::Pure,
                        },
                    ],
                ),
            ]),
            events: ::std::collections::BTreeMap::new(),
            errors: ::std::collections::BTreeMap::new(),
            receive: false,
            fallback: false,
        }
    }
    ///The parsed JSON ABI of the contract.
    #[cfg(feature = "providers")]
    pub static GLUE_ABI: ::ethers::contract::Lazy<::ethers::core::abi::Abi> =
        ::ethers::contract::Lazy::new(__abi);
    #[rustfmt::skip]
    #[cfg(feature = "providers")]
    const __BYTECODE: &[u8] = b"`\x80\x80`@R4a\0\x19W`@Qa\x18\x16\x90\x81a\0g\x829\xF3[bF\x1B\xCD`\xE5\x1B\x81R` `\x04\x82\x01R`\"`$\x82\x01R\x7FEther sent to non-payable functi`D\x82\x01Ra7\xB7`\xF1\x1B`d\x82\x01R`\x84\x90\xFD\xFE`\x80\x80`@R`\x046\x10\x15a\0\x93W[`\x84`@Q\x7F\x08\xC3y\xA0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R` `\x04\x82\x01R`5`$\x82\x01R\x7FContract does not have fallback `D\x82\x01R\x7Fnor receive functions\0\0\0\0\0\0\0\0\0\0\0`d\x82\x01R\xFD[`\x005`\xE0\x1Cct\x88\xEF\x01\x03a\0\x0FW4a\x0CZWa\x02\0`\x03\x196\x01\x12a\x0B\xD6W`\x045g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x08\x07Wa\x01\0`\x03\x19\x826\x03\x01\x12a\x08\x02Wa\0\xDEa\r\xE4V[\x90\x80`\x04\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x07\xFDW`\xA0\x91a\x01\ta\x01Z\x92`\x046\x91\x84\x01\x01a\x0F\xD0V[\x84Ra\x01\x17`$\x82\x01a\x10\xC4V[` \x85\x01Ra\x01(`D\x82\x01a\x10\xC4V[`@\x85\x01Ra\x019`d\x82\x01a\x10\xC4V[``\x85\x01Ra\x01K6`\x84\x83\x01a\x10\xD9V[`\x80\x85\x01R`\xC46\x91\x01a\x10\xD9V[\x91\x01R`$5g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x08\x07W```\x03\x19\x826\x03\x01\x12a\x08\x02Wa\x01\x85a\x0E\x04V[\x90a\x01\x92\x81`\x04\x01a\x10\xC4V[\x82R`$\x81\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x07\xFDWa\x01\xB8\x90`\x046\x91\x84\x01\x01a\x11\x0BV[` \x83\x01R`D\x81\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x07\xFDW`@\x91`\x04a\x01\xE3\x926\x92\x01\x01a\x0F\xD0V[\x91\x01Rg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF`D5\x11a\x08\x07W`\x80`\x03\x19`D56\x03\x01\x12a\x08\x02Wa\x02\x0Fa\x0E\x04V[g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF`D5`\x04\x015\x11a\x07\xFDWa\x02:6`\x04`D5\x81\x015`D5\x01\x01a\x15\xCAV[\x81Ra\x02K6`$`D5\x01a\x10\xD9V[` \x82\x01R`d`D5\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x07\xFDWa\x02z`@\x91`\x046\x91`D5\x01\x01a\x0F\xD0V[\x91\x01R`d5g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x08\x07Wa\x02\x9D\x906\x90`\x04\x01a\x12\x17V[P`\x845g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x08\x07Wa\x02\xBE\x906\x90`\x04\x01a\x14lV[P`@\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\\6\x01\x12a\x08\x02Wa\x02\xF1a\x0E$V[g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF`\xA45\x16`\xA45\x03a\x0B\x8AW`\xA45\x81R`\xC45\x90g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x16\x82\x03a\x0B\x8AW` \x01R``\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x1C6\x01\x12a\x08\x02Wa\x03Va\x0E\x04V[`\xE45\x81Ra\x01\x045` \x82\x01Ra\x01$5\x90g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x16\x82\x03a\x0B\x8AW`@\x01R`@\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFE\xBC6\x01\x12a\x08\x02Wa\x03\xB0a\x0E$V[o\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFFa\x01D5\x81\x81\x16\x81\x03a\x0B\x8AW\x82Ra\x01d5\x16a\x01d5\x03a\x0B\x8AW` a\x01d5\x91\x01Ra\x01\x845g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x08\x07W`\xA0`\x03\x19\x826\x03\x01\x12a\x08\x02Wa\x04\x10a\x0EDV[\x90\x80`\x04\x015`\x04\x81\x10\x15a\x0B\x8AW\x82Ra\x04-`$\x82\x01a\x11RV[` \x83\x01Ra\x04>`D\x82\x01a\x11RV[`@\x83\x01R`d\x81\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x07\xFDWa\x04g\x90`\x046\x91\x84\x01\x01a\x11\x8BV[``\x83\x01R`\x84\x81\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x07\xFDW`\x80\x91`\x04a\x04\x92\x926\x92\x01\x01a\x0F\xD0V[\x91\x01Rg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFFa\x01\xA45\x11a\x08\x07Wa\x02 `\x03\x19a\x01\xA456\x03\x01\x12a\x08\x02W`@Q\x80a\x01`\x81\x01\x10g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFFa\x01`\x83\x01\x11\x17a\x0B\xA7Wa\x01`\x81\x01`@Rg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFFa\x01\xA45`\x04\x015\x11a\x07\xFDWa\x05\r6`\x04a\x01\xA45\x81\x015a\x01\xA45\x01\x01a\x0F\xD0V[\x81Ra\x05\x1F6`$a\x01\xA45\x01a\x10\xD9V[` \x82\x01Ra\x0546`da\x01\xA45\x01a\x16\x16V[`@\x82\x01Ra\x05I6`\xA4a\x01\xA45\x01a\x16\x16V[``\x82\x01Ra\x05^6`\xE4a\x01\xA45\x01a\x16\x16V[`\x80\x82\x01Ra\x05t6a\x01$a\x01\xA45\x01a\x10\xD9V[`\xA0\x82\x01Ra\x05\x8A6a\x01da\x01\xA45\x01a\x10\xD9V[`\xC0\x82\x01Ra\x01\xA4\x805\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x07\xFDW6`#\x82a\x01\xA45\x01\x01\x12\x15a\x08CW`\x04\x81a\x01\xA45\x01\x015\x90a\x05\xD1a\x05\xCC\x83a\x13\xD0V[a\x0E\x84V[\x91` \x83\x82\x81R\x01\x916`$\x83`\x05\x1B\x83a\x01\xA45\x01\x01\x01\x11a\x08HW`$\x81a\x01\xA45\x01\x01\x92[`$\x83`\x05\x1B\x83a\x01\xA45\x01\x01\x01\x84\x10a\x08MW\x85\x85`\xE0\x82\x01Ra\x01\xC4a\x01\xA45\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x07\xFDW6`#\x82a\x01\xA45\x01\x01\x12\x15a\x08CW`\x04\x81a\x01\xA45\x01\x015\x90a\x06Sa\x05\xCC\x83a\x13\xD0V[\x91` \x83\x82\x81R\x01\x916`$\x83`\x05\x1B\x83a\x01\xA45\x01\x01\x01\x11a\x08HW`$\x81a\x01\xA45\x01\x01\x92[`$\x83`\x05\x1B\x83a\x01\xA45\x01\x01\x01\x84\x10a\x08\x0CW\x85\x85a\x01\0\x82\x01Ra\x01\xE4\x90a\x06\xA9\x82a\x01\xA45\x01a\x16AV[a\x01 \x82\x01Ra\x01@a\x06\xC2a\x02\x04a\x01\xA45\x01a\x16AV[\x91\x01Ra\x01\xC45g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x08\x07W`\x80`\x03\x19\x826\x03\x01\x12a\x08\x02Wa\x06\xEEa\x0E\x04V[\x90a\x06\xFC6\x82`\x04\x01a\x11`V[\x82R`D\x81\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x07\xFDWa\x07\"\x90`\x046\x91\x84\x01\x01a\x11\x0BV[` \x83\x01R`d\x81\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x07\xFDW`@\x91`\x04a\x07M\x926\x92\x01\x01a\x0F\xD0V[\x91\x01R5g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x08\x07W`\xA0`\x03\x19\x826\x03\x01\x12a\x08\x02Wa\x07va\x0EdV[\x81`\x04\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x07\xFDWa\x07\x9A\x90`\x046\x91\x85\x01\x01a\x15\xCAV[\x81R`$\x82\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x07\xFDWa\x07\xC0\x90`\x046\x91\x85\x01\x01a\x17\x1EV[` \x82\x01R`@a\x07\xD46`D\x85\x01a\x10\xD9V[\x91\x01R`\x84\x81\x015\x90g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11a\x07\xFDW`\x04a\x07\xFB\x926\x92\x01\x01a\x17\x1EV[\0[a\x0E\xC8V[a\r`V[a\x0C\xDCV[g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x845\x11a\x08CW` \x80`$\x92a\x0866\x85\x895\x88a\x01\xA45\x01\x01\x01a\x0F\xD0V[\x81R\x01\x94\x01\x93\x90Pa\x06{V[a\x0FLV[a\x13\xE8V[g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x845\x11a\x08CW`\x80\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xDC\x855\x84a\x01\xA45\x01\x016\x03\x01\x12a\x08\x02Wa\x08\x99a\x0EdV[\x90`$\x855\x84a\x01\xA45\x01\x01\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x07\xFDW`\xA0\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xDC\x82\x885\x87a\x01\xA45\x01\x01\x016\x03\x01\x12a\x08\x02Wa\x08\xF4a\x0EDV[\x90`\x07`$\x82\x895\x88a\x01\xA45\x01\x01\x01\x015\x10\x15a\x0B\x8AWa\x01\xA45\x85\x01\x875\x01\x81\x01`$\x81\x015\x83R`\x07`D\x90\x91\x015\x10\x15a\x0B\x8AWa\x01\xA45\x85\x01\x875\x01\x81\x01`D\x81\x015` \x84\x01R`\x07`d\x90\x91\x015\x10\x15a\x0B\x8AWa\x01\xA45\x85\x01\x875\x01\x81\x01`d\x81\x015`@\x84\x01R`\t`\x84\x90\x91\x015\x10\x15a\x0B\x8AWa\x01\xA45\x85\x01\x875\x01\x81\x01`\x84\x81\x015``\x84\x01R`\xA4\x015\x90g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11a\x07\xFDW`$a\t\xB4\x926\x92\x8A5\x89a\x01\xA45\x01\x01\x01\x01\x01a\x0F\xD0V[`\x80\x82\x01R\x82R`D\x855\x84a\x01\xA45\x01\x01\x015\x90g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11a\x07\xFDW`\xC0\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xDC\x83\x885\x87a\x01\xA45\x01\x01\x016\x03\x01\x12a\x08\x02Wa\n\x16a\r\xE4V[\x91`$\x81\x885\x87a\x01\xA45\x01\x01\x01\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x07\xFDW\x81\x885\x87a\x01\xA45\x01\x01\x01\x016`C\x82\x01\x12\x15a\x08CW`$\x81\x015a\n]a\x05\xCC\x82a\x13\xD0V[\x91` \x83\x83\x81R\x01\x906`D\x84`\x05\x1B\x83\x01\x01\x11a\x08HW`D\x81\x01\x91[`D\x84`\x05\x1B\x83\x01\x01\x83\x10a\x0B\x8FWPPPP\x83Ra\n\xA6`D\x82\x895\x88a\x01\xA45\x01\x01\x01\x01a\x13\xC2V[` \x84\x01Ra\n\xC1`d\x82\x895\x88a\x01\xA45\x01\x01\x01\x01a\x13\xC2V[`@\x84\x01Ra\n\xDC`\x84\x82\x895\x88a\x01\xA45\x01\x01\x01\x01a\x13\xC2V[``\x84\x01Rg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF`\xA4\x82\x895\x88a\x01\xA45\x01\x01\x01\x015\x11a\x07\xFDW`\xC4\x90a\x0B\x1E6a\x01\xA45\x88\x01\x8A5\x01\x83\x01`\xA4\x81\x015\x01`$\x01a\x0F\xD0V[`\x80\x85\x01R\x875\x86a\x01\xA45\x01\x01\x01\x015\x90`\x07\x82\x10\x15a\x0B\x8AW`$\x93\x83` \x94\x93`\xA0\x86\x95\x01R\x83\x82\x01Ra\x0B_`d\x895\x88a\x01\xA45\x01\x01\x01a\x13\xC2V[`@\x82\x01Ra\x0Bx`\x84\x895\x88a\x01\xA45\x01\x01\x01a\x13\xC2V[``\x82\x01R\x81R\x01\x94\x01\x93\x90Pa\x05\xF9V[`\0\x80\xFD[` \x80\x91a\x0B\x9C\x85a\x13\xC2V[\x81R\x01\x92\x01\x91a\n{V[\x7FNH{q\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`\0R`A`\x04R`$`\0\xFD[`\x84`@Q\x7F\x08\xC3y\xA0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R` `\x04\x82\x01R`\"`$\x82\x01R\x7FABI decoding: tuple data too sho`D\x82\x01R\x7Frt\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`d\x82\x01R\xFD[\x80\x7F\x08\xC3y\xA0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`\x84\x92R` `\x04\x82\x01R`\"`$\x82\x01R\x7FEther sent to non-payable functi`D\x82\x01R\x7Fon\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`d\x82\x01R\xFD[`\x84`@Q\x7F\x08\xC3y\xA0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R` `\x04\x82\x01R`\"`$\x82\x01R\x7FABI decoding: invalid tuple offs`D\x82\x01R\x7Fet\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`d\x82\x01R\xFD[`\x84`@Q\x7F\x08\xC3y\xA0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R` `\x04\x82\x01R`#`$\x82\x01R\x7FABI decoding: struct data too sh`D\x82\x01R\x7Fort\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`d\x82\x01R\xFD[`@Q\x90`\xC0\x82\x01\x82\x81\x10g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x17a\x0B\xA7W`@RV[`@Q\x90``\x82\x01\x82\x81\x10g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x17a\x0B\xA7W`@RV[`@Q\x90`@\x82\x01\x82\x81\x10g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x17a\x0B\xA7W`@RV[`@Q\x90`\xA0\x82\x01\x82\x81\x10g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x17a\x0B\xA7W`@RV[`@Q\x90`\x80\x82\x01\x82\x81\x10g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x17a\x0B\xA7W`@RV[\x90\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xE0`\x1F`@Q\x93\x01\x16\x82\x01\x82\x81\x10g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x17a\x0B\xA7W`@RV[`\x84`@Q\x7F\x08\xC3y\xA0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R` `\x04\x82\x01R`#`$\x82\x01R\x7FABI decoding: invalid struct off`D\x82\x01R\x7Fset\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`d\x82\x01R\xFD[`\x84`@Q\x7F\x08\xC3y\xA0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R` `\x04\x82\x01R`+`$\x82\x01R\x7FABI decoding: invalid calldata a`D\x82\x01R\x7Frray offset\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`d\x82\x01R\xFD[\x81`\x1F\x82\x01\x12\x15a\x08CW\x805` \x91g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11a\x0B\xA7Wa\x10\x1F\x83\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xE0`\x1F\x85\x01\x16\x01a\x0E\x84V[\x93\x82\x85R\x83\x83\x83\x01\x01\x11a\x10@W\x90\x80\x83`\0\x94\x93\x01\x83\x86\x017\x83\x01\x01R\x90V[`\x84\x83`@Q\x90\x7F\x08\xC3y\xA0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x82R`\x04\x82\x01R`'`$\x82\x01R\x7FABI decoding: invalid byte array`D\x82\x01R\x7F length\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`d\x82\x01R\xFD[5\x90g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x16\x82\x03a\x0B\x8AWV[\x91\x90\x82`@\x91\x03\x12a\x08\x02Wa\x11\x04` a\x10\xF2a\x0E$V[\x93a\x10\xFC\x81a\x10\xC4V[\x85R\x01a\x10\xC4V[` \x83\x01RV[\x91\x90\x91` \x81\x84\x03\x12a\x08\x02W`@Q\x90` \x82\x01\x93g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x94\x83\x81\x10\x86\x82\x11\x17a\x0B\xA7W`@R\x82\x94\x825\x90\x81\x11a\x07\xFDWa\x11N\x92\x01a\x0F\xD0V[\x90RV[5\x90\x81`\x07\x0B\x82\x03a\x0B\x8AWV[\x91\x90\x82`@\x91\x03\x12a\x08\x02Wa\x11\x04` a\x11ya\x0E$V[\x93a\x11\x83\x81a\x11RV[\x85R\x01a\x11RV[\x91\x90\x91`@\x81\x84\x03\x12a\x08\x02Wa\x11\xA0a\x0E$V[\x92g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x91\x805\x83\x81\x11a\x07\xFDW\x82a\x11\xC0\x91\x83\x01a\x0F\xD0V[\x85R` \x81\x015\x90\x83\x82\x11a\x07\xFDW\x01\x90`@\x82\x82\x03\x12a\x08\x02Wa\x11\xE3a\x0E$V[\x92\x825c\xFF\xFF\xFF\xFF\x81\x16\x81\x03a\x0B\x8AW\x84R` \x83\x015\x90\x81\x11a\x07\xFDWa\x12\x0B\x92\x01a\x0F\xD0V[` \x82\x01R` \x83\x01RV[\x91\x90\x91a\x02\0\x81\x84\x03\x12a\x08\x02W`@Q\x90a\x01\xC0\x90\x81\x83\x01\x94g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x95\x84\x81\x10\x87\x82\x11\x17a\x0B\xA7W`@R\x83\x95a\x12U\x82\x84a\x10\xD9V[\x85R`@\x83\x015\x81\x81\x11a\x07\xFDW\x82a\x12o\x91\x85\x01a\x0F\xD0V[` \x86\x01Ra\x12\x80``\x84\x01a\x11RV[`@\x86\x01Ra\x12\x92\x82`\x80\x85\x01a\x11`V[``\x86\x01R`\xC0\x83\x015\x81\x81\x11a\x07\xFDW\x82a\x12\xAF\x91\x85\x01a\x11\x8BV[`\x80\x86\x01R`\xE0\x83\x015\x81\x81\x11a\x07\xFDW\x82a\x12\xCC\x91\x85\x01a\x0F\xD0V[`\xA0\x86\x01Ra\x01\0\x80\x84\x015\x82\x81\x11a\x07\xFDW\x83a\x12\xEB\x91\x86\x01a\x0F\xD0V[`\xC0\x87\x01Ra\x01 \x94\x85\x85\x015\x83\x81\x11a\x07\xFDW\x84a\x13\x0B\x91\x87\x01a\x0F\xD0V[`\xE0\x88\x01Ra\x01@\x91\x82\x86\x015\x84\x81\x11a\x07\xFDW\x85a\x13+\x91\x88\x01a\x0F\xD0V[\x90\x88\x01Ra\x01`\x95\x86\x86\x015\x84\x81\x11a\x07\xFDW\x85a\x13J\x91\x88\x01a\x0F\xD0V[\x90\x88\x01Ra\x01\x80\x91\x82\x86\x015\x84\x81\x11a\x07\xFDW\x85a\x13i\x91\x88\x01a\x0F\xD0V[\x90\x88\x01Ra\x01\xA0\x95\x86\x86\x015\x84\x81\x11a\x07\xFDW\x85a\x13\x88\x91\x88\x01a\x0F\xD0V[\x90\x88\x01R\x84\x015\x82\x81\x11a\x07\xFDW\x83a\x13\xA2\x91\x86\x01a\x0F\xD0V[\x90\x86\x01Ra\x01\xE0\x83\x015\x90\x81\x11a\x07\xFDWa\x13\xBD\x92\x01a\x0F\xD0V[\x91\x01RV[5\x90\x81`\x03\x0B\x82\x03a\x0B\x8AWV[g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x0B\xA7W`\x05\x1B` \x01\x90V[`\x84`@Q\x7F\x08\xC3y\xA0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R` `\x04\x82\x01R`+`$\x82\x01R\x7FABI decoding: invalid calldata a`D\x82\x01R\x7Frray stride\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`d\x82\x01R\xFD[\x91\x90`\x80\x83\x82\x03\x12a\x08\x02Wa\x14\x80a\x0EdV[\x92a\x14\x8A\x81a\x11RV[\x84R` a\x14\x99\x81\x83\x01a\x13\xC2V[\x81\x86\x01R`@\x80\x83\x015\x93g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x94\x85\x81\x11a\x07\xFDW\x81a\x14\xC1\x91\x86\x01a\x11\x8BV[\x82\x88\x01R``\x93\x84\x81\x015\x90\x86\x82\x11a\x07\xFDW\x01\x91\x81`\x1F\x84\x01\x12\x15a\x08CW\x825\x90a\x14\xF0a\x05\xCC\x83a\x13\xD0V[\x96\x85\x80\x89\x85\x81R\x01\x93`\x05\x1B\x86\x01\x01\x94\x84\x86\x11a\x08HW\x86\x81\x01\x93[\x86\x85\x10a\x15 WPPPPPPPP\x83\x01RV[\x845\x83\x81\x11a\x08CW\x82\x01\x90`\xA0\x90\x81\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xE0\x84\x8A\x03\x01\x12a\x08\x02Wa\x15ba\x0EdV[\x91\x8A\x84\x015`\x04\x81\x10\x15a\x0B\x8AW\x83R\x86\x84\x015\x86\x81\x11a\x07\xFDW\x89\x8Ca\x15\x8B\x92\x87\x01\x01a\x0F\xD0V[\x8B\x84\x01Ra\x15\x9B\x89\x8D\x86\x01a\x11`V[\x87\x84\x01R\x83\x015\x91\x85\x83\x11a\x07\xFDWa\x15\xBB\x89\x8C\x80\x96\x95\x81\x96\x01\x01a\x0F\xD0V[\x8C\x82\x01R\x81R\x01\x94\x01\x93a\x15\x0CV[\x91\x90\x91`@\x81\x84\x03\x12a\x08\x02Wa\x15\xDFa\x0E$V[\x92g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x825\x81\x81\x11a\x07\xFDW\x82a\x15\xFE\x91\x85\x01a\x12\x17V[\x85R` \x83\x015\x90\x81\x11a\x07\xFDWa\x11\x04\x92\x01a\x14lV[\x91\x90\x82`@\x91\x03\x12a\x08\x02Wa\x11\x04` a\x16/a\x0E$V[\x93a\x169\x81a\x11RV[\x85R\x01a\x13\xC2V[5\x90\x81\x15\x15\x82\x03a\x0B\x8AWV[\x91\x90`\x80\x83\x82\x03\x12a\x08\x02Wa\x16ba\x0EdV[\x92g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x90\x805\x82\x81\x11a\x07\xFDW\x83a\x16\x82\x91\x83\x01a\x0F\xD0V[\x85R` \x81\x015\x82\x81\x11a\x07\xFDW\x81\x01``\x81\x85\x03\x12a\x08\x02Wa\x16\xA4a\x0E\x04V[\x90\x805\x84\x81\x11a\x07\xFDW\x85a\x16\xBA\x91\x83\x01a\x0F\xD0V[\x82R` \x81\x015\x84\x81\x11a\x07\xFDW\x85a\x16\xD4\x91\x83\x01a\x0F\xD0V[` \x83\x01R`@\x81\x015\x93\x84\x11a\x07\xFDWa\x17\x17\x94``\x94a\x16\xF6\x92\x01a\x0F\xD0V[`@\x82\x01R` \x86\x01Ra\x17\x0C`@\x82\x01a\x11RV[`@\x86\x01R\x01a\x11RV[``\x83\x01RV[\x91\x90\x91``\x81\x84\x03\x12a\x08\x02Wa\x173a\x0E\x04V[\x92g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x91\x805\x83\x81\x11a\x07\xFDW\x81\x01\x82`\x1F\x82\x01\x12\x15a\x08CW` \x90\x805a\x17ea\x05\xCC\x82a\x13\xD0V[\x91\x83\x80\x84\x84\x81R\x01\x92`\x05\x1B\x82\x01\x01\x91\x86\x83\x11a\x08HW\x84\x82\x01\x90[\x83\x82\x10a\x17\xBBWPPPP\x86R\x80\x82\x015\x93\x84\x11a\x07\xFDWa\x17\xAA`@\x93a\x17\xB4\x95\x84\x01a\x16NV[\x90\x86\x01R\x01a\x11RV[`@\x83\x01RV[\x815\x89\x81\x11a\x08CW\x86\x91a\x17\xD5\x8A\x84\x80\x94\x88\x01\x01a\x16NV[\x81R\x01\x91\x01\x90a\x17\x81V\xFE\xA2dipfsX\"\x12 \xC9x\xA3\xA6\xB3\x83?F\x1F\x04\xFAiU\xC3/\xAE;k\xB2\xFE\x91i\xB3F\xD1\xD2\xBBZH\xA5.;dsolcC\0\x08\x15\x003";
    /// The bytecode of the contract.
    #[cfg(feature = "providers")]
    pub static GLUE_BYTECODE: ::ethers::core::types::Bytes =
        ::ethers::core::types::Bytes::from_static(__BYTECODE);
    #[rustfmt::skip]
    #[cfg(feature = "providers")]
    const __DEPLOYED_BYTECODE: &[u8] = b"`\x80\x80`@R`\x046\x10\x15a\0\x93W[`\x84`@Q\x7F\x08\xC3y\xA0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R` `\x04\x82\x01R`5`$\x82\x01R\x7FContract does not have fallback `D\x82\x01R\x7Fnor receive functions\0\0\0\0\0\0\0\0\0\0\0`d\x82\x01R\xFD[`\x005`\xE0\x1Cct\x88\xEF\x01\x03a\0\x0FW4a\x0CZWa\x02\0`\x03\x196\x01\x12a\x0B\xD6W`\x045g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x08\x07Wa\x01\0`\x03\x19\x826\x03\x01\x12a\x08\x02Wa\0\xDEa\r\xE4V[\x90\x80`\x04\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x07\xFDW`\xA0\x91a\x01\ta\x01Z\x92`\x046\x91\x84\x01\x01a\x0F\xD0V[\x84Ra\x01\x17`$\x82\x01a\x10\xC4V[` \x85\x01Ra\x01(`D\x82\x01a\x10\xC4V[`@\x85\x01Ra\x019`d\x82\x01a\x10\xC4V[``\x85\x01Ra\x01K6`\x84\x83\x01a\x10\xD9V[`\x80\x85\x01R`\xC46\x91\x01a\x10\xD9V[\x91\x01R`$5g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x08\x07W```\x03\x19\x826\x03\x01\x12a\x08\x02Wa\x01\x85a\x0E\x04V[\x90a\x01\x92\x81`\x04\x01a\x10\xC4V[\x82R`$\x81\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x07\xFDWa\x01\xB8\x90`\x046\x91\x84\x01\x01a\x11\x0BV[` \x83\x01R`D\x81\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x07\xFDW`@\x91`\x04a\x01\xE3\x926\x92\x01\x01a\x0F\xD0V[\x91\x01Rg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF`D5\x11a\x08\x07W`\x80`\x03\x19`D56\x03\x01\x12a\x08\x02Wa\x02\x0Fa\x0E\x04V[g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF`D5`\x04\x015\x11a\x07\xFDWa\x02:6`\x04`D5\x81\x015`D5\x01\x01a\x15\xCAV[\x81Ra\x02K6`$`D5\x01a\x10\xD9V[` \x82\x01R`d`D5\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x07\xFDWa\x02z`@\x91`\x046\x91`D5\x01\x01a\x0F\xD0V[\x91\x01R`d5g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x08\x07Wa\x02\x9D\x906\x90`\x04\x01a\x12\x17V[P`\x845g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x08\x07Wa\x02\xBE\x906\x90`\x04\x01a\x14lV[P`@\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\\6\x01\x12a\x08\x02Wa\x02\xF1a\x0E$V[g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF`\xA45\x16`\xA45\x03a\x0B\x8AW`\xA45\x81R`\xC45\x90g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x16\x82\x03a\x0B\x8AW` \x01R``\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x1C6\x01\x12a\x08\x02Wa\x03Va\x0E\x04V[`\xE45\x81Ra\x01\x045` \x82\x01Ra\x01$5\x90g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x16\x82\x03a\x0B\x8AW`@\x01R`@\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFE\xBC6\x01\x12a\x08\x02Wa\x03\xB0a\x0E$V[o\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFFa\x01D5\x81\x81\x16\x81\x03a\x0B\x8AW\x82Ra\x01d5\x16a\x01d5\x03a\x0B\x8AW` a\x01d5\x91\x01Ra\x01\x845g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x08\x07W`\xA0`\x03\x19\x826\x03\x01\x12a\x08\x02Wa\x04\x10a\x0EDV[\x90\x80`\x04\x015`\x04\x81\x10\x15a\x0B\x8AW\x82Ra\x04-`$\x82\x01a\x11RV[` \x83\x01Ra\x04>`D\x82\x01a\x11RV[`@\x83\x01R`d\x81\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x07\xFDWa\x04g\x90`\x046\x91\x84\x01\x01a\x11\x8BV[``\x83\x01R`\x84\x81\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x07\xFDW`\x80\x91`\x04a\x04\x92\x926\x92\x01\x01a\x0F\xD0V[\x91\x01Rg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFFa\x01\xA45\x11a\x08\x07Wa\x02 `\x03\x19a\x01\xA456\x03\x01\x12a\x08\x02W`@Q\x80a\x01`\x81\x01\x10g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFFa\x01`\x83\x01\x11\x17a\x0B\xA7Wa\x01`\x81\x01`@Rg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFFa\x01\xA45`\x04\x015\x11a\x07\xFDWa\x05\r6`\x04a\x01\xA45\x81\x015a\x01\xA45\x01\x01a\x0F\xD0V[\x81Ra\x05\x1F6`$a\x01\xA45\x01a\x10\xD9V[` \x82\x01Ra\x0546`da\x01\xA45\x01a\x16\x16V[`@\x82\x01Ra\x05I6`\xA4a\x01\xA45\x01a\x16\x16V[``\x82\x01Ra\x05^6`\xE4a\x01\xA45\x01a\x16\x16V[`\x80\x82\x01Ra\x05t6a\x01$a\x01\xA45\x01a\x10\xD9V[`\xA0\x82\x01Ra\x05\x8A6a\x01da\x01\xA45\x01a\x10\xD9V[`\xC0\x82\x01Ra\x01\xA4\x805\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x07\xFDW6`#\x82a\x01\xA45\x01\x01\x12\x15a\x08CW`\x04\x81a\x01\xA45\x01\x015\x90a\x05\xD1a\x05\xCC\x83a\x13\xD0V[a\x0E\x84V[\x91` \x83\x82\x81R\x01\x916`$\x83`\x05\x1B\x83a\x01\xA45\x01\x01\x01\x11a\x08HW`$\x81a\x01\xA45\x01\x01\x92[`$\x83`\x05\x1B\x83a\x01\xA45\x01\x01\x01\x84\x10a\x08MW\x85\x85`\xE0\x82\x01Ra\x01\xC4a\x01\xA45\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x07\xFDW6`#\x82a\x01\xA45\x01\x01\x12\x15a\x08CW`\x04\x81a\x01\xA45\x01\x015\x90a\x06Sa\x05\xCC\x83a\x13\xD0V[\x91` \x83\x82\x81R\x01\x916`$\x83`\x05\x1B\x83a\x01\xA45\x01\x01\x01\x11a\x08HW`$\x81a\x01\xA45\x01\x01\x92[`$\x83`\x05\x1B\x83a\x01\xA45\x01\x01\x01\x84\x10a\x08\x0CW\x85\x85a\x01\0\x82\x01Ra\x01\xE4\x90a\x06\xA9\x82a\x01\xA45\x01a\x16AV[a\x01 \x82\x01Ra\x01@a\x06\xC2a\x02\x04a\x01\xA45\x01a\x16AV[\x91\x01Ra\x01\xC45g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x08\x07W`\x80`\x03\x19\x826\x03\x01\x12a\x08\x02Wa\x06\xEEa\x0E\x04V[\x90a\x06\xFC6\x82`\x04\x01a\x11`V[\x82R`D\x81\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x07\xFDWa\x07\"\x90`\x046\x91\x84\x01\x01a\x11\x0BV[` \x83\x01R`d\x81\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x07\xFDW`@\x91`\x04a\x07M\x926\x92\x01\x01a\x0F\xD0V[\x91\x01R5g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x08\x07W`\xA0`\x03\x19\x826\x03\x01\x12a\x08\x02Wa\x07va\x0EdV[\x81`\x04\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x07\xFDWa\x07\x9A\x90`\x046\x91\x85\x01\x01a\x15\xCAV[\x81R`$\x82\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x07\xFDWa\x07\xC0\x90`\x046\x91\x85\x01\x01a\x17\x1EV[` \x82\x01R`@a\x07\xD46`D\x85\x01a\x10\xD9V[\x91\x01R`\x84\x81\x015\x90g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11a\x07\xFDW`\x04a\x07\xFB\x926\x92\x01\x01a\x17\x1EV[\0[a\x0E\xC8V[a\r`V[a\x0C\xDCV[g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x845\x11a\x08CW` \x80`$\x92a\x0866\x85\x895\x88a\x01\xA45\x01\x01\x01a\x0F\xD0V[\x81R\x01\x94\x01\x93\x90Pa\x06{V[a\x0FLV[a\x13\xE8V[g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x845\x11a\x08CW`\x80\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xDC\x855\x84a\x01\xA45\x01\x016\x03\x01\x12a\x08\x02Wa\x08\x99a\x0EdV[\x90`$\x855\x84a\x01\xA45\x01\x01\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x07\xFDW`\xA0\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xDC\x82\x885\x87a\x01\xA45\x01\x01\x016\x03\x01\x12a\x08\x02Wa\x08\xF4a\x0EDV[\x90`\x07`$\x82\x895\x88a\x01\xA45\x01\x01\x01\x015\x10\x15a\x0B\x8AWa\x01\xA45\x85\x01\x875\x01\x81\x01`$\x81\x015\x83R`\x07`D\x90\x91\x015\x10\x15a\x0B\x8AWa\x01\xA45\x85\x01\x875\x01\x81\x01`D\x81\x015` \x84\x01R`\x07`d\x90\x91\x015\x10\x15a\x0B\x8AWa\x01\xA45\x85\x01\x875\x01\x81\x01`d\x81\x015`@\x84\x01R`\t`\x84\x90\x91\x015\x10\x15a\x0B\x8AWa\x01\xA45\x85\x01\x875\x01\x81\x01`\x84\x81\x015``\x84\x01R`\xA4\x015\x90g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11a\x07\xFDW`$a\t\xB4\x926\x92\x8A5\x89a\x01\xA45\x01\x01\x01\x01\x01a\x0F\xD0V[`\x80\x82\x01R\x82R`D\x855\x84a\x01\xA45\x01\x01\x015\x90g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11a\x07\xFDW`\xC0\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xDC\x83\x885\x87a\x01\xA45\x01\x01\x016\x03\x01\x12a\x08\x02Wa\n\x16a\r\xE4V[\x91`$\x81\x885\x87a\x01\xA45\x01\x01\x01\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x07\xFDW\x81\x885\x87a\x01\xA45\x01\x01\x01\x016`C\x82\x01\x12\x15a\x08CW`$\x81\x015a\n]a\x05\xCC\x82a\x13\xD0V[\x91` \x83\x83\x81R\x01\x906`D\x84`\x05\x1B\x83\x01\x01\x11a\x08HW`D\x81\x01\x91[`D\x84`\x05\x1B\x83\x01\x01\x83\x10a\x0B\x8FWPPPP\x83Ra\n\xA6`D\x82\x895\x88a\x01\xA45\x01\x01\x01\x01a\x13\xC2V[` \x84\x01Ra\n\xC1`d\x82\x895\x88a\x01\xA45\x01\x01\x01\x01a\x13\xC2V[`@\x84\x01Ra\n\xDC`\x84\x82\x895\x88a\x01\xA45\x01\x01\x01\x01a\x13\xC2V[``\x84\x01Rg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF`\xA4\x82\x895\x88a\x01\xA45\x01\x01\x01\x015\x11a\x07\xFDW`\xC4\x90a\x0B\x1E6a\x01\xA45\x88\x01\x8A5\x01\x83\x01`\xA4\x81\x015\x01`$\x01a\x0F\xD0V[`\x80\x85\x01R\x875\x86a\x01\xA45\x01\x01\x01\x015\x90`\x07\x82\x10\x15a\x0B\x8AW`$\x93\x83` \x94\x93`\xA0\x86\x95\x01R\x83\x82\x01Ra\x0B_`d\x895\x88a\x01\xA45\x01\x01\x01a\x13\xC2V[`@\x82\x01Ra\x0Bx`\x84\x895\x88a\x01\xA45\x01\x01\x01a\x13\xC2V[``\x82\x01R\x81R\x01\x94\x01\x93\x90Pa\x05\xF9V[`\0\x80\xFD[` \x80\x91a\x0B\x9C\x85a\x13\xC2V[\x81R\x01\x92\x01\x91a\n{V[\x7FNH{q\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`\0R`A`\x04R`$`\0\xFD[`\x84`@Q\x7F\x08\xC3y\xA0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R` `\x04\x82\x01R`\"`$\x82\x01R\x7FABI decoding: tuple data too sho`D\x82\x01R\x7Frt\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`d\x82\x01R\xFD[\x80\x7F\x08\xC3y\xA0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`\x84\x92R` `\x04\x82\x01R`\"`$\x82\x01R\x7FEther sent to non-payable functi`D\x82\x01R\x7Fon\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`d\x82\x01R\xFD[`\x84`@Q\x7F\x08\xC3y\xA0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R` `\x04\x82\x01R`\"`$\x82\x01R\x7FABI decoding: invalid tuple offs`D\x82\x01R\x7Fet\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`d\x82\x01R\xFD[`\x84`@Q\x7F\x08\xC3y\xA0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R` `\x04\x82\x01R`#`$\x82\x01R\x7FABI decoding: struct data too sh`D\x82\x01R\x7Fort\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`d\x82\x01R\xFD[`@Q\x90`\xC0\x82\x01\x82\x81\x10g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x17a\x0B\xA7W`@RV[`@Q\x90``\x82\x01\x82\x81\x10g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x17a\x0B\xA7W`@RV[`@Q\x90`@\x82\x01\x82\x81\x10g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x17a\x0B\xA7W`@RV[`@Q\x90`\xA0\x82\x01\x82\x81\x10g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x17a\x0B\xA7W`@RV[`@Q\x90`\x80\x82\x01\x82\x81\x10g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x17a\x0B\xA7W`@RV[\x90\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xE0`\x1F`@Q\x93\x01\x16\x82\x01\x82\x81\x10g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x17a\x0B\xA7W`@RV[`\x84`@Q\x7F\x08\xC3y\xA0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R` `\x04\x82\x01R`#`$\x82\x01R\x7FABI decoding: invalid struct off`D\x82\x01R\x7Fset\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`d\x82\x01R\xFD[`\x84`@Q\x7F\x08\xC3y\xA0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R` `\x04\x82\x01R`+`$\x82\x01R\x7FABI decoding: invalid calldata a`D\x82\x01R\x7Frray offset\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`d\x82\x01R\xFD[\x81`\x1F\x82\x01\x12\x15a\x08CW\x805` \x91g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11a\x0B\xA7Wa\x10\x1F\x83\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xE0`\x1F\x85\x01\x16\x01a\x0E\x84V[\x93\x82\x85R\x83\x83\x83\x01\x01\x11a\x10@W\x90\x80\x83`\0\x94\x93\x01\x83\x86\x017\x83\x01\x01R\x90V[`\x84\x83`@Q\x90\x7F\x08\xC3y\xA0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x82R`\x04\x82\x01R`'`$\x82\x01R\x7FABI decoding: invalid byte array`D\x82\x01R\x7F length\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`d\x82\x01R\xFD[5\x90g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x16\x82\x03a\x0B\x8AWV[\x91\x90\x82`@\x91\x03\x12a\x08\x02Wa\x11\x04` a\x10\xF2a\x0E$V[\x93a\x10\xFC\x81a\x10\xC4V[\x85R\x01a\x10\xC4V[` \x83\x01RV[\x91\x90\x91` \x81\x84\x03\x12a\x08\x02W`@Q\x90` \x82\x01\x93g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x94\x83\x81\x10\x86\x82\x11\x17a\x0B\xA7W`@R\x82\x94\x825\x90\x81\x11a\x07\xFDWa\x11N\x92\x01a\x0F\xD0V[\x90RV[5\x90\x81`\x07\x0B\x82\x03a\x0B\x8AWV[\x91\x90\x82`@\x91\x03\x12a\x08\x02Wa\x11\x04` a\x11ya\x0E$V[\x93a\x11\x83\x81a\x11RV[\x85R\x01a\x11RV[\x91\x90\x91`@\x81\x84\x03\x12a\x08\x02Wa\x11\xA0a\x0E$V[\x92g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x91\x805\x83\x81\x11a\x07\xFDW\x82a\x11\xC0\x91\x83\x01a\x0F\xD0V[\x85R` \x81\x015\x90\x83\x82\x11a\x07\xFDW\x01\x90`@\x82\x82\x03\x12a\x08\x02Wa\x11\xE3a\x0E$V[\x92\x825c\xFF\xFF\xFF\xFF\x81\x16\x81\x03a\x0B\x8AW\x84R` \x83\x015\x90\x81\x11a\x07\xFDWa\x12\x0B\x92\x01a\x0F\xD0V[` \x82\x01R` \x83\x01RV[\x91\x90\x91a\x02\0\x81\x84\x03\x12a\x08\x02W`@Q\x90a\x01\xC0\x90\x81\x83\x01\x94g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x95\x84\x81\x10\x87\x82\x11\x17a\x0B\xA7W`@R\x83\x95a\x12U\x82\x84a\x10\xD9V[\x85R`@\x83\x015\x81\x81\x11a\x07\xFDW\x82a\x12o\x91\x85\x01a\x0F\xD0V[` \x86\x01Ra\x12\x80``\x84\x01a\x11RV[`@\x86\x01Ra\x12\x92\x82`\x80\x85\x01a\x11`V[``\x86\x01R`\xC0\x83\x015\x81\x81\x11a\x07\xFDW\x82a\x12\xAF\x91\x85\x01a\x11\x8BV[`\x80\x86\x01R`\xE0\x83\x015\x81\x81\x11a\x07\xFDW\x82a\x12\xCC\x91\x85\x01a\x0F\xD0V[`\xA0\x86\x01Ra\x01\0\x80\x84\x015\x82\x81\x11a\x07\xFDW\x83a\x12\xEB\x91\x86\x01a\x0F\xD0V[`\xC0\x87\x01Ra\x01 \x94\x85\x85\x015\x83\x81\x11a\x07\xFDW\x84a\x13\x0B\x91\x87\x01a\x0F\xD0V[`\xE0\x88\x01Ra\x01@\x91\x82\x86\x015\x84\x81\x11a\x07\xFDW\x85a\x13+\x91\x88\x01a\x0F\xD0V[\x90\x88\x01Ra\x01`\x95\x86\x86\x015\x84\x81\x11a\x07\xFDW\x85a\x13J\x91\x88\x01a\x0F\xD0V[\x90\x88\x01Ra\x01\x80\x91\x82\x86\x015\x84\x81\x11a\x07\xFDW\x85a\x13i\x91\x88\x01a\x0F\xD0V[\x90\x88\x01Ra\x01\xA0\x95\x86\x86\x015\x84\x81\x11a\x07\xFDW\x85a\x13\x88\x91\x88\x01a\x0F\xD0V[\x90\x88\x01R\x84\x015\x82\x81\x11a\x07\xFDW\x83a\x13\xA2\x91\x86\x01a\x0F\xD0V[\x90\x86\x01Ra\x01\xE0\x83\x015\x90\x81\x11a\x07\xFDWa\x13\xBD\x92\x01a\x0F\xD0V[\x91\x01RV[5\x90\x81`\x03\x0B\x82\x03a\x0B\x8AWV[g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x0B\xA7W`\x05\x1B` \x01\x90V[`\x84`@Q\x7F\x08\xC3y\xA0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R` `\x04\x82\x01R`+`$\x82\x01R\x7FABI decoding: invalid calldata a`D\x82\x01R\x7Frray stride\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`d\x82\x01R\xFD[\x91\x90`\x80\x83\x82\x03\x12a\x08\x02Wa\x14\x80a\x0EdV[\x92a\x14\x8A\x81a\x11RV[\x84R` a\x14\x99\x81\x83\x01a\x13\xC2V[\x81\x86\x01R`@\x80\x83\x015\x93g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x94\x85\x81\x11a\x07\xFDW\x81a\x14\xC1\x91\x86\x01a\x11\x8BV[\x82\x88\x01R``\x93\x84\x81\x015\x90\x86\x82\x11a\x07\xFDW\x01\x91\x81`\x1F\x84\x01\x12\x15a\x08CW\x825\x90a\x14\xF0a\x05\xCC\x83a\x13\xD0V[\x96\x85\x80\x89\x85\x81R\x01\x93`\x05\x1B\x86\x01\x01\x94\x84\x86\x11a\x08HW\x86\x81\x01\x93[\x86\x85\x10a\x15 WPPPPPPPP\x83\x01RV[\x845\x83\x81\x11a\x08CW\x82\x01\x90`\xA0\x90\x81\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xE0\x84\x8A\x03\x01\x12a\x08\x02Wa\x15ba\x0EdV[\x91\x8A\x84\x015`\x04\x81\x10\x15a\x0B\x8AW\x83R\x86\x84\x015\x86\x81\x11a\x07\xFDW\x89\x8Ca\x15\x8B\x92\x87\x01\x01a\x0F\xD0V[\x8B\x84\x01Ra\x15\x9B\x89\x8D\x86\x01a\x11`V[\x87\x84\x01R\x83\x015\x91\x85\x83\x11a\x07\xFDWa\x15\xBB\x89\x8C\x80\x96\x95\x81\x96\x01\x01a\x0F\xD0V[\x8C\x82\x01R\x81R\x01\x94\x01\x93a\x15\x0CV[\x91\x90\x91`@\x81\x84\x03\x12a\x08\x02Wa\x15\xDFa\x0E$V[\x92g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x825\x81\x81\x11a\x07\xFDW\x82a\x15\xFE\x91\x85\x01a\x12\x17V[\x85R` \x83\x015\x90\x81\x11a\x07\xFDWa\x11\x04\x92\x01a\x14lV[\x91\x90\x82`@\x91\x03\x12a\x08\x02Wa\x11\x04` a\x16/a\x0E$V[\x93a\x169\x81a\x11RV[\x85R\x01a\x13\xC2V[5\x90\x81\x15\x15\x82\x03a\x0B\x8AWV[\x91\x90`\x80\x83\x82\x03\x12a\x08\x02Wa\x16ba\x0EdV[\x92g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x90\x805\x82\x81\x11a\x07\xFDW\x83a\x16\x82\x91\x83\x01a\x0F\xD0V[\x85R` \x81\x015\x82\x81\x11a\x07\xFDW\x81\x01``\x81\x85\x03\x12a\x08\x02Wa\x16\xA4a\x0E\x04V[\x90\x805\x84\x81\x11a\x07\xFDW\x85a\x16\xBA\x91\x83\x01a\x0F\xD0V[\x82R` \x81\x015\x84\x81\x11a\x07\xFDW\x85a\x16\xD4\x91\x83\x01a\x0F\xD0V[` \x83\x01R`@\x81\x015\x93\x84\x11a\x07\xFDWa\x17\x17\x94``\x94a\x16\xF6\x92\x01a\x0F\xD0V[`@\x82\x01R` \x86\x01Ra\x17\x0C`@\x82\x01a\x11RV[`@\x86\x01R\x01a\x11RV[``\x83\x01RV[\x91\x90\x91``\x81\x84\x03\x12a\x08\x02Wa\x173a\x0E\x04V[\x92g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x91\x805\x83\x81\x11a\x07\xFDW\x81\x01\x82`\x1F\x82\x01\x12\x15a\x08CW` \x90\x805a\x17ea\x05\xCC\x82a\x13\xD0V[\x91\x83\x80\x84\x84\x81R\x01\x92`\x05\x1B\x82\x01\x01\x91\x86\x83\x11a\x08HW\x84\x82\x01\x90[\x83\x82\x10a\x17\xBBWPPPP\x86R\x80\x82\x015\x93\x84\x11a\x07\xFDWa\x17\xAA`@\x93a\x17\xB4\x95\x84\x01a\x16NV[\x90\x86\x01R\x01a\x11RV[`@\x83\x01RV[\x815\x89\x81\x11a\x08CW\x86\x91a\x17\xD5\x8A\x84\x80\x94\x88\x01\x01a\x16NV[\x81R\x01\x91\x01\x90a\x17\x81V\xFE\xA2dipfsX\"\x12 \xC9x\xA3\xA6\xB3\x83?F\x1F\x04\xFAiU\xC3/\xAE;k\xB2\xFE\x91i\xB3F\xD1\xD2\xBBZH\xA5.;dsolcC\0\x08\x15\x003";
    /// The deployed bytecode of the contract.
    #[cfg(feature = "providers")]
    pub static GLUE_DEPLOYED_BYTECODE: ::ethers::core::types::Bytes =
        ::ethers::core::types::Bytes::from_static(__DEPLOYED_BYTECODE);
    #[cfg(feature = "providers")]
    pub struct Glue<M>(::ethers::contract::Contract<M>);
    #[cfg(feature = "providers")]
    impl<M> ::core::clone::Clone for Glue<M> {
        fn clone(&self) -> Self {
            Self(::core::clone::Clone::clone(&self.0))
        }
    }
    #[cfg(feature = "providers")]
    impl<M> ::core::ops::Deref for Glue<M> {
        type Target = ::ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    #[cfg(feature = "providers")]
    impl<M> ::core::ops::DerefMut for Glue<M> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    #[cfg(feature = "providers")]
    impl<M> ::core::fmt::Debug for Glue<M> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple(::core::stringify!(Glue))
                .field(&self.address())
                .finish()
        }
    }
    #[cfg(feature = "providers")]
    impl<M: ::ethers::providers::Middleware> Glue<M> {
        /// Creates a new contract instance with the specified `ethers` client at
        /// `address`. The contract derefs to a `ethers::Contract` object.
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(::ethers::contract::Contract::new(
                address.into(),
                GLUE_ABI.clone(),
                client,
            ))
        }
        /// Constructs the general purpose `Deployer` instance based on the provided constructor arguments and sends it.
        /// Returns a new instance of a deployer that returns an instance of this contract after sending the transaction
        ///
        /// Notes:
        /// - If there are no constructor arguments, you should pass `()` as the argument.
        /// - The default poll duration is 7 seconds.
        /// - The default number of confirmations is 1 block.
        ///
        ///
        /// # Example
        ///
        /// Generate contract bindings with `abigen!` and deploy a new contract instance.
        ///
        /// *Note*: this requires a `bytecode` and `abi` object in the `greeter.json` artifact.
        ///
        /// ```ignore
        /// # async fn deploy<M: ethers::providers::Middleware>(client: ::std::sync::Arc<M>) {
        ///     abigen!(Greeter, "../greeter.json");
        ///
        ///    let greeter_contract = Greeter::deploy(client, "Hello world!".to_string()).unwrap().send().await.unwrap();
        ///    let msg = greeter_contract.greet().call().await.unwrap();
        /// # }
        /// ```
        pub fn deploy<T: ::ethers::core::abi::Tokenize>(
            client: ::std::sync::Arc<M>,
            constructor_args: T,
        ) -> ::core::result::Result<
            ::ethers::contract::builders::ContractDeployer<M, Self>,
            ::ethers::contract::ContractError<M>,
        > {
            let factory = ::ethers::contract::ContractFactory::new(
                GLUE_ABI.clone(),
                GLUE_BYTECODE.clone().into(),
                client,
            );
            let deployer = factory.deploy(constructor_args)?;
            let deployer = ::ethers::contract::ContractDeployer::new(deployer);
            Ok(deployer)
        }
        ///Calls the contract's `typesTelescope` (0x7488ef01) function
        pub fn types_telescope(
            &self,
            p0: IbcLightclientsTendermintV1HeaderData,
            p1: IbcLightclientsTendermintV1HeaderData,
            p2: IbcLightclientsTendermintV1HeaderData,
            p3: IbcLightclientsTendermintV1HeaderData,
            p4: IbcLightclientsTendermintV1HeaderData,
            p5: IbcLightclientsTendermintV1HeaderData,
            p6: IbcLightclientsTendermintV1HeaderData,
            p7: IbcLightclientsTendermintV1HeaderData,
            p8: IbcLightclientsTendermintV1HeaderData,
            p9: IbcLightclientsTendermintV1HeaderData,
            p10: IbcLightclientsTendermintV1HeaderData,
            p11: IbcLightclientsTendermintV1HeaderData,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash(
                    [116, 136, 239, 1],
                    (p0, p1, p2, p3, p4, p5, p6, p7, p8, p9, p10, p11),
                )
                .expect("method not found (this should never happen)")
        }
    }
    #[cfg(feature = "providers")]
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>> for Glue<M> {
        fn from(contract: ::ethers::contract::Contract<M>) -> Self {
            Self::new(contract.address(), contract.client())
        }
    }
    ///Container type for all input parameters for the `typesTelescope` function with signature `typesTelescope((string,uint64,uint64,uint64,(uint64,uint64),(uint64,uint64)),(uint64,(bytes),bytes),((((uint64,uint64),string,int64,(int64,int64),(bytes,(uint32,bytes)),bytes,bytes,bytes,bytes,bytes,bytes,bytes,bytes,bytes),(int64,int32,(bytes,(uint32,bytes)),(uint8,bytes,(int64,int64),bytes)[])),(uint64,uint64),bytes),((uint64,uint64),string,int64,(int64,int64),(bytes,(uint32,bytes)),bytes,bytes,bytes,bytes,bytes,bytes,bytes,bytes,bytes),(int64,int32,(bytes,(uint32,bytes)),(uint8,bytes,(int64,int64),bytes)[]),(uint64,uint64),(bytes32,bytes32,uint64),(uint128,uint128),(uint8,int64,int64,(bytes,(uint32,bytes)),string),(string,(uint64,uint64),(int64,int32),(int64,int32),(int64,int32),(uint64,uint64),(uint64,uint64),((uint8,uint8,uint8,uint8,bytes),(int32[],int32,int32,int32,bytes,uint8),int32,int32)[],string[],bool,bool),((int64,int64),(bytes),bytes),((((uint64,uint64),string,int64,(int64,int64),(bytes,(uint32,bytes)),bytes,bytes,bytes,bytes,bytes,bytes,bytes,bytes,bytes),(int64,int32,(bytes,(uint32,bytes)),(uint8,bytes,(int64,int64),bytes)[])),((bytes,(bytes,bytes,bytes),int64,int64)[],(bytes,(bytes,bytes,bytes),int64,int64),int64),(uint64,uint64),((bytes,(bytes,bytes,bytes),int64,int64)[],(bytes,(bytes,bytes,bytes),int64,int64),int64)))` and selector `0x7488ef01`
    #[derive(Clone, ::ethers::contract::EthCall, ::ethers::contract::EthDisplay)]
    #[ethcall(
        name = "typesTelescope",
        abi = "typesTelescope((string,uint64,uint64,uint64,(uint64,uint64),(uint64,uint64)),(uint64,(bytes),bytes),((((uint64,uint64),string,int64,(int64,int64),(bytes,(uint32,bytes)),bytes,bytes,bytes,bytes,bytes,bytes,bytes,bytes,bytes),(int64,int32,(bytes,(uint32,bytes)),(uint8,bytes,(int64,int64),bytes)[])),(uint64,uint64),bytes),((uint64,uint64),string,int64,(int64,int64),(bytes,(uint32,bytes)),bytes,bytes,bytes,bytes,bytes,bytes,bytes,bytes,bytes),(int64,int32,(bytes,(uint32,bytes)),(uint8,bytes,(int64,int64),bytes)[]),(uint64,uint64),(bytes32,bytes32,uint64),(uint128,uint128),(uint8,int64,int64,(bytes,(uint32,bytes)),string),(string,(uint64,uint64),(int64,int32),(int64,int32),(int64,int32),(uint64,uint64),(uint64,uint64),((uint8,uint8,uint8,uint8,bytes),(int32[],int32,int32,int32,bytes,uint8),int32,int32)[],string[],bool,bool),((int64,int64),(bytes),bytes),((((uint64,uint64),string,int64,(int64,int64),(bytes,(uint32,bytes)),bytes,bytes,bytes,bytes,bytes,bytes,bytes,bytes,bytes),(int64,int32,(bytes,(uint32,bytes)),(uint8,bytes,(int64,int64),bytes)[])),((bytes,(bytes,bytes,bytes),int64,int64)[],(bytes,(bytes,bytes,bytes),int64,int64),int64),(uint64,uint64),((bytes,(bytes,bytes,bytes),int64,int64)[],(bytes,(bytes,bytes,bytes),int64,int64),int64)))"
    )]
    pub struct TypesTelescopeCall(
        pub IbcLightclientsTendermintV1HeaderData,
        pub IbcLightclientsTendermintV1HeaderData,
        pub IbcLightclientsTendermintV1HeaderData,
        pub IbcLightclientsTendermintV1HeaderData,
        pub IbcLightclientsTendermintV1HeaderData,
        pub IbcLightclientsTendermintV1HeaderData,
        pub IbcLightclientsTendermintV1HeaderData,
        pub IbcLightclientsTendermintV1HeaderData,
        pub IbcLightclientsTendermintV1HeaderData,
        pub IbcLightclientsTendermintV1HeaderData,
        pub IbcLightclientsTendermintV1HeaderData,
        pub IbcLightclientsTendermintV1HeaderData,
    );
    ///`CosmosIcs23V1InnerSpecData(int32[],int32,int32,int32,bytes,uint8)`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    pub struct CosmosIcs23V1InnerSpecData {
        pub child_order: ::std::vec::Vec<i32>,
        pub child_size: i32,
        pub min_prefix_length: i32,
        pub max_prefix_length: i32,
        pub empty_child: ::ethers::core::types::Bytes,
        pub hash: u8,
    }
    ///`CosmosIcs23V1LeafOpData(uint8,uint8,uint8,uint8,bytes)`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    pub struct CosmosIcs23V1LeafOpData {
        pub hash: u8,
        pub prehash_key: u8,
        pub prehash_value: u8,
        pub length: u8,
        pub prefix: ::ethers::core::types::Bytes,
    }
    ///`CosmosIcs23V1ProofSpecData((uint8,uint8,uint8,uint8,bytes),(int32[],int32,int32,int32,bytes,uint8),int32,int32)`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    pub struct CosmosIcs23V1ProofSpecData {
        pub leaf_spec: CosmosIcs23V1LeafOpData,
        pub inner_spec: CosmosIcs23V1InnerSpecData,
        pub max_depth: i32,
        pub min_depth: i32,
    }
    ///`GoogleProtobufDurationData(int64,int32)`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    pub struct GoogleProtobufDurationData {
        pub seconds: i64,
        pub nanos: i32,
    }
    ///`GoogleProtobufTimestampData(int64,int64)`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    pub struct GoogleProtobufTimestampData {
        pub secs: i64,
        pub nanos: i64,
    }
    ///`IbcCoreCommitmentV1MerkleRootData(bytes)`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    pub struct IbcCoreCommitmentV1MerkleRootData {
        pub hash: ::ethers::core::types::Bytes,
    }
    ///`IbcLightclientsTendermintV1ClientStateData(string,(uint64,uint64),(int64,int32),(int64,int32),(int64,int32),(uint64,uint64),(uint64,uint64),((uint8,uint8,uint8,uint8,bytes),(int32[],int32,int32,int32,bytes,uint8),int32,int32)[],string[],bool,bool)`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    pub struct IbcLightclientsTendermintV1ClientStateData {
        pub chain_id: ::std::string::String,
        pub trust_level: IbcLightclientsTendermintV1FractionData,
        pub trusting_period: GoogleProtobufDurationData,
        pub unbonding_period: GoogleProtobufDurationData,
        pub max_clock_drift: GoogleProtobufDurationData,
        pub frozen_height: IbcCoreClientV1HeightData,
        pub latest_height: IbcCoreClientV1HeightData,
        pub proof_specs: ::std::vec::Vec<CosmosIcs23V1ProofSpecData>,
        pub upgrade_path: ::std::vec::Vec<::std::string::String>,
        pub allow_update_after_expiry: bool,
        pub allow_update_after_misbehaviour: bool,
    }
    ///`IbcLightclientsTendermintV1ConsensusStateData((int64,int64),(bytes),bytes)`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    pub struct IbcLightclientsTendermintV1ConsensusStateData {
        pub timestamp: GoogleProtobufTimestampData,
        pub root: IbcCoreCommitmentV1MerkleRootData,
        pub next_validators_hash: ::ethers::core::types::Bytes,
    }
    ///`IbcLightclientsTendermintV1FractionData(uint64,uint64)`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    pub struct IbcLightclientsTendermintV1FractionData {
        pub numerator: u64,
        pub denominator: u64,
    }
    ///`IbcLightclientsTendermintV1HeaderData((((uint64,uint64),string,int64,(int64,int64),(bytes,(uint32,bytes)),bytes,bytes,bytes,bytes,bytes,bytes,bytes,bytes,bytes),(int64,int32,(bytes,(uint32,bytes)),(uint8,bytes,(int64,int64),bytes)[])),((bytes,(bytes,bytes,bytes),int64,int64)[],(bytes,(bytes,bytes,bytes),int64,int64),int64),(uint64,uint64),((bytes,(bytes,bytes,bytes),int64,int64)[],(bytes,(bytes,bytes,bytes),int64,int64),int64))`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    pub struct IbcLightclientsTendermintV1HeaderData {
        pub signed_header: TendermintTypesSignedHeaderData,
        pub validator_set: Data,
        pub trusted_height: IbcCoreClientV1HeightData,
        pub trusted_validators: Data,
    }
    ///`OptimizedConsensusState(uint64,bytes32,bytes32)`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    pub struct OptimizedConsensusState {
        pub timestamp: u64,
        pub root: [u8; 32],
        pub next_validators_hash: [u8; 32],
    }
    ///`ProcessedMoment(uint128,uint128)`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    pub struct ProcessedMoment {
        pub timestamp: u128,
        pub height: u128,
    }
    ///`TendermintCryptoPublicKeyData(bytes,bytes,bytes)`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    pub struct TendermintCryptoPublicKeyData {
        pub ed_25519: ::ethers::core::types::Bytes,
        pub secp_25_6k_1: ::ethers::core::types::Bytes,
        pub bn_254: ::ethers::core::types::Bytes,
    }
    ///`TendermintTypesBlockIDData(bytes,(uint32,bytes))`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    pub struct TendermintTypesBlockIDData {
        pub hash: ::ethers::core::types::Bytes,
        pub part_set_header: TendermintTypesPartSetHeaderData,
    }
    ///`TendermintTypesCanonicalBlockIDData(bytes,(uint32,bytes))`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    pub struct TendermintTypesCanonicalBlockIDData {
        pub hash: ::ethers::core::types::Bytes,
        pub part_set_header: TendermintTypesCanonicalPartSetHeaderData,
    }
    ///`TendermintTypesCanonicalPartSetHeaderData(uint32,bytes)`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    pub struct TendermintTypesCanonicalPartSetHeaderData {
        pub total: u32,
        pub hash: ::ethers::core::types::Bytes,
    }
    ///`TendermintTypesCanonicalVoteData(uint8,int64,int64,(bytes,(uint32,bytes)),string)`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    pub struct TendermintTypesCanonicalVoteData {
        pub type_: u8,
        pub height: i64,
        pub round: i64,
        pub block_id: TendermintTypesCanonicalBlockIDData,
        pub chain_id: ::std::string::String,
    }
    ///`TendermintTypesCommitData(int64,int32,(bytes,(uint32,bytes)),(uint8,bytes,(int64,int64),bytes)[])`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    pub struct TendermintTypesCommitData {
        pub height: i64,
        pub round: i32,
        pub block_id: TendermintTypesBlockIDData,
        pub signatures: ::std::vec::Vec<TendermintTypesCommitSigData>,
    }
    ///`TendermintTypesCommitSigData(uint8,bytes,(int64,int64),bytes)`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    pub struct TendermintTypesCommitSigData {
        pub block_id_flag: u8,
        pub validator_address: ::ethers::core::types::Bytes,
        pub timestamp: GoogleProtobufTimestampData,
        pub signature: ::ethers::core::types::Bytes,
    }
    ///`TendermintTypesHeaderData((uint64,uint64),string,int64,(int64,int64),(bytes,(uint32,bytes)),bytes,bytes,bytes,bytes,bytes,bytes,bytes,bytes,bytes)`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    pub struct TendermintTypesHeaderData {
        pub version: TendermintVersionConsensusData,
        pub chain_id: ::std::string::String,
        pub height: i64,
        pub time: GoogleProtobufTimestampData,
        pub last_block_id: TendermintTypesBlockIDData,
        pub last_commit_hash: ::ethers::core::types::Bytes,
        pub data_hash: ::ethers::core::types::Bytes,
        pub validators_hash: ::ethers::core::types::Bytes,
        pub next_validators_hash: ::ethers::core::types::Bytes,
        pub consensus_hash: ::ethers::core::types::Bytes,
        pub app_hash: ::ethers::core::types::Bytes,
        pub last_results_hash: ::ethers::core::types::Bytes,
        pub evidence_hash: ::ethers::core::types::Bytes,
        pub proposer_address: ::ethers::core::types::Bytes,
    }
    ///`TendermintTypesPartSetHeaderData(uint32,bytes)`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    pub struct TendermintTypesPartSetHeaderData {
        pub total: u32,
        pub hash: ::ethers::core::types::Bytes,
    }
    ///`TendermintTypesSignedHeaderData(((uint64,uint64),string,int64,(int64,int64),(bytes,(uint32,bytes)),bytes,bytes,bytes,bytes,bytes,bytes,bytes,bytes,bytes),(int64,int32,(bytes,(uint32,bytes)),(uint8,bytes,(int64,int64),bytes)[]))`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    pub struct TendermintTypesSignedHeaderData {
        pub header: TendermintTypesHeaderData,
        pub commit: TendermintTypesCommitData,
    }
    ///`TendermintTypesValidatorData(bytes,(bytes,bytes,bytes),int64,int64)`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    pub struct TendermintTypesValidatorData {
        pub addr: ::ethers::core::types::Bytes,
        pub pub_key: TendermintCryptoPublicKeyData,
        pub voting_power: i64,
        pub proposer_priority: i64,
    }
    ///`Data((bytes,(bytes,bytes,bytes),int64,int64)[],(bytes,(bytes,bytes,bytes),int64,int64),int64)`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    pub struct Data {
        pub validators: ::std::vec::Vec<TendermintTypesValidatorData>,
        pub proposer: TendermintTypesValidatorData,
        pub total_voting_power: i64,
    }
    ///`TendermintVersionConsensusData(uint64,uint64)`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    pub struct TendermintVersionConsensusData {
        pub block: u64,
        pub app: u64,
    }
    ///`UnionIbcLightclientsCometblsV1ClientStateData(string,uint64,uint64,uint64,(uint64,uint64),(uint64,uint64))`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    pub struct UnionIbcLightclientsCometblsV1ClientStateData {
        pub chain_id: ::std::string::String,
        pub trusting_period: u64,
        pub unbonding_period: u64,
        pub max_clock_drift: u64,
        pub frozen_height: IbcCoreClientV1HeightData,
        pub latest_height: IbcCoreClientV1HeightData,
    }
    ///`UnionIbcLightclientsCometblsV1ConsensusStateData(uint64,(bytes),bytes)`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    pub struct UnionIbcLightclientsCometblsV1ConsensusStateData {
        pub timestamp: u64,
        pub root: IbcCoreCommitmentV1MerkleRootData,
        pub next_validators_hash: ::ethers::core::types::Bytes,
    }
    ///`UnionIbcLightclientsCometblsV1HeaderData((((uint64,uint64),string,int64,(int64,int64),(bytes,(uint32,bytes)),bytes,bytes,bytes,bytes,bytes,bytes,bytes,bytes,bytes),(int64,int32,(bytes,(uint32,bytes)),(uint8,bytes,(int64,int64),bytes)[])),(uint64,uint64),bytes)`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    pub struct UnionIbcLightclientsCometblsV1HeaderData {
        pub signed_header: TendermintTypesSignedHeaderData,
        pub trusted_height: IbcCoreClientV1HeightData,
        pub zero_knowledge_proof: ::ethers::core::types::Bytes,
    }
}
