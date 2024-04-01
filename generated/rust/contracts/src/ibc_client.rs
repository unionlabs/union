pub use ibc_client::*;
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
pub mod ibc_client {
    pub use super::super::shared_types::*;
    #[cfg(feature = "providers")]
    #[allow(deprecated)]
    #[cfg(feature = "providers")]
    fn __abi() -> ::ethers::core::abi::Abi {
        ::ethers::core::abi::ethabi::Contract {
            constructor: ::core::option::Option::None,
            functions: ::core::convert::From::from([
                (
                    ::std::borrow::ToOwned::to_owned("COMMITMENT_PREFIX"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("COMMITMENT_PREFIX"),
                        inputs: ::std::vec![],
                        outputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::string::String::new(),
                            kind: ::ethers::core::abi::ethabi::ParamType::String,
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("string"),
                            ),
                        },],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("capabilities"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("capabilities"),
                        inputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::string::String::new(),
                            kind: ::ethers::core::abi::ethabi::ParamType::String,
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("string"),
                            ),
                        },],
                        outputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::string::String::new(),
                            kind: ::ethers::core::abi::ethabi::ParamType::Address,
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("address"),
                            ),
                        },],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("channels"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("channels"),
                        inputs: ::std::vec![
                            ::ethers::core::abi::ethabi::Param {
                                name: ::std::string::String::new(),
                                kind: ::ethers::core::abi::ethabi::ParamType::String,
                                internal_type: ::core::option::Option::Some(
                                    ::std::borrow::ToOwned::to_owned("string"),
                                ),
                            },
                            ::ethers::core::abi::ethabi::Param {
                                name: ::std::string::String::new(),
                                kind: ::ethers::core::abi::ethabi::ParamType::String,
                                internal_type: ::core::option::Option::Some(
                                    ::std::borrow::ToOwned::to_owned("string"),
                                ),
                            },
                        ],
                        outputs: ::std::vec![
                            ::ethers::core::abi::ethabi::Param {
                                name: ::std::borrow::ToOwned::to_owned("state"),
                                kind: ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                internal_type: ::core::option::Option::Some(
                                    ::std::borrow::ToOwned::to_owned(
                                        "enum IbcCoreChannelV1GlobalEnums.State",
                                    ),
                                ),
                            },
                            ::ethers::core::abi::ethabi::Param {
                                name: ::std::borrow::ToOwned::to_owned("ordering"),
                                kind: ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                internal_type: ::core::option::Option::Some(
                                    ::std::borrow::ToOwned::to_owned(
                                        "enum IbcCoreChannelV1GlobalEnums.Order",
                                    ),
                                ),
                            },
                            ::ethers::core::abi::ethabi::Param {
                                name: ::std::borrow::ToOwned::to_owned("counterparty"),
                                kind: ::ethers::core::abi::ethabi::ParamType::Tuple(::std::vec![
                                    ::ethers::core::abi::ethabi::ParamType::String,
                                    ::ethers::core::abi::ethabi::ParamType::String,
                                ],),
                                internal_type: ::core::option::Option::Some(
                                    ::std::borrow::ToOwned::to_owned(
                                        "struct IbcCoreChannelV1Counterparty.Data",
                                    ),
                                ),
                            },
                            ::ethers::core::abi::ethabi::Param {
                                name: ::std::borrow::ToOwned::to_owned("version"),
                                kind: ::ethers::core::abi::ethabi::ParamType::String,
                                internal_type: ::core::option::Option::Some(
                                    ::std::borrow::ToOwned::to_owned("string"),
                                ),
                            },
                        ],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("clientImpls"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("clientImpls"),
                        inputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::string::String::new(),
                            kind: ::ethers::core::abi::ethabi::ParamType::String,
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("string"),
                            ),
                        },],
                        outputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::string::String::new(),
                            kind: ::ethers::core::abi::ethabi::ParamType::Address,
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("address"),
                            ),
                        },],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("clientRegistry"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("clientRegistry"),
                        inputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::string::String::new(),
                            kind: ::ethers::core::abi::ethabi::ParamType::String,
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("string"),
                            ),
                        },],
                        outputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::string::String::new(),
                            kind: ::ethers::core::abi::ethabi::ParamType::Address,
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("address"),
                            ),
                        },],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("clientTypes"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("clientTypes"),
                        inputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::string::String::new(),
                            kind: ::ethers::core::abi::ethabi::ParamType::String,
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("string"),
                            ),
                        },],
                        outputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::string::String::new(),
                            kind: ::ethers::core::abi::ethabi::ParamType::String,
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("string"),
                            ),
                        },],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("commitments"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("commitments"),
                        inputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::string::String::new(),
                            kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize,),
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("bytes32"),
                            ),
                        },],
                        outputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::string::String::new(),
                            kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize,),
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("bytes32"),
                            ),
                        },],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("connections"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("connections"),
                        inputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::string::String::new(),
                            kind: ::ethers::core::abi::ethabi::ParamType::String,
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("string"),
                            ),
                        },],
                        outputs: ::std::vec![
                            ::ethers::core::abi::ethabi::Param {
                                name: ::std::borrow::ToOwned::to_owned("client_id"),
                                kind: ::ethers::core::abi::ethabi::ParamType::String,
                                internal_type: ::core::option::Option::Some(
                                    ::std::borrow::ToOwned::to_owned("string"),
                                ),
                            },
                            ::ethers::core::abi::ethabi::Param {
                                name: ::std::borrow::ToOwned::to_owned("state"),
                                kind: ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                internal_type: ::core::option::Option::Some(
                                    ::std::borrow::ToOwned::to_owned(
                                        "enum IbcCoreConnectionV1GlobalEnums.State",
                                    ),
                                ),
                            },
                            ::ethers::core::abi::ethabi::Param {
                                name: ::std::borrow::ToOwned::to_owned("counterparty"),
                                kind: ::ethers::core::abi::ethabi::ParamType::Tuple(::std::vec![
                                    ::ethers::core::abi::ethabi::ParamType::String,
                                    ::ethers::core::abi::ethabi::ParamType::String,
                                    ::ethers::core::abi::ethabi::ParamType::Tuple(::std::vec![
                                        ::ethers::core::abi::ethabi::ParamType::Bytes
                                    ],),
                                ],),
                                internal_type: ::core::option::Option::Some(
                                    ::std::borrow::ToOwned::to_owned(
                                        "struct IbcCoreConnectionV1Counterparty.Data",
                                    ),
                                ),
                            },
                            ::ethers::core::abi::ethabi::Param {
                                name: ::std::borrow::ToOwned::to_owned("delay_period"),
                                kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                internal_type: ::core::option::Option::Some(
                                    ::std::borrow::ToOwned::to_owned("uint64"),
                                ),
                            },
                        ],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("createClient"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("createClient"),
                        inputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("msg_"),
                            kind: ::ethers::core::abi::ethabi::ParamType::Tuple(::std::vec![
                                ::ethers::core::abi::ethabi::ParamType::String,
                                ::ethers::core::abi::ethabi::ParamType::Bytes,
                                ::ethers::core::abi::ethabi::ParamType::Bytes,
                            ],),
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("struct IBCMsgs.MsgCreateClient",),
                            ),
                        },],
                        outputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("clientId"),
                            kind: ::ethers::core::abi::ethabi::ParamType::String,
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("string"),
                            ),
                        },],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getClient"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("getClient"),
                        inputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("clientId"),
                            kind: ::ethers::core::abi::ethabi::ParamType::String,
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("string"),
                            ),
                        },],
                        outputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::string::String::new(),
                            kind: ::ethers::core::abi::ethabi::ParamType::Address,
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("contract ILightClient"),
                            ),
                        },],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("nextChannelSequence"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("nextChannelSequence",),
                        inputs: ::std::vec![],
                        outputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::string::String::new(),
                            kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("uint64"),
                            ),
                        },],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("nextClientSequence"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("nextClientSequence"),
                        inputs: ::std::vec![],
                        outputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::string::String::new(),
                            kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("uint64"),
                            ),
                        },],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("nextConnectionSequence"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("nextConnectionSequence",),
                        inputs: ::std::vec![],
                        outputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::string::String::new(),
                            kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("uint64"),
                            ),
                        },],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("nextSequenceAcks"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("nextSequenceAcks"),
                        inputs: ::std::vec![
                            ::ethers::core::abi::ethabi::Param {
                                name: ::std::string::String::new(),
                                kind: ::ethers::core::abi::ethabi::ParamType::String,
                                internal_type: ::core::option::Option::Some(
                                    ::std::borrow::ToOwned::to_owned("string"),
                                ),
                            },
                            ::ethers::core::abi::ethabi::Param {
                                name: ::std::string::String::new(),
                                kind: ::ethers::core::abi::ethabi::ParamType::String,
                                internal_type: ::core::option::Option::Some(
                                    ::std::borrow::ToOwned::to_owned("string"),
                                ),
                            },
                        ],
                        outputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::string::String::new(),
                            kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("uint64"),
                            ),
                        },],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("nextSequenceRecvs"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("nextSequenceRecvs"),
                        inputs: ::std::vec![
                            ::ethers::core::abi::ethabi::Param {
                                name: ::std::string::String::new(),
                                kind: ::ethers::core::abi::ethabi::ParamType::String,
                                internal_type: ::core::option::Option::Some(
                                    ::std::borrow::ToOwned::to_owned("string"),
                                ),
                            },
                            ::ethers::core::abi::ethabi::Param {
                                name: ::std::string::String::new(),
                                kind: ::ethers::core::abi::ethabi::ParamType::String,
                                internal_type: ::core::option::Option::Some(
                                    ::std::borrow::ToOwned::to_owned("string"),
                                ),
                            },
                        ],
                        outputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::string::String::new(),
                            kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("uint64"),
                            ),
                        },],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("nextSequenceSends"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("nextSequenceSends"),
                        inputs: ::std::vec![
                            ::ethers::core::abi::ethabi::Param {
                                name: ::std::string::String::new(),
                                kind: ::ethers::core::abi::ethabi::ParamType::String,
                                internal_type: ::core::option::Option::Some(
                                    ::std::borrow::ToOwned::to_owned("string"),
                                ),
                            },
                            ::ethers::core::abi::ethabi::Param {
                                name: ::std::string::String::new(),
                                kind: ::ethers::core::abi::ethabi::ParamType::String,
                                internal_type: ::core::option::Option::Some(
                                    ::std::borrow::ToOwned::to_owned("string"),
                                ),
                            },
                        ],
                        outputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::string::String::new(),
                            kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("uint64"),
                            ),
                        },],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("packetReceipts"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("packetReceipts"),
                        inputs: ::std::vec![
                            ::ethers::core::abi::ethabi::Param {
                                name: ::std::string::String::new(),
                                kind: ::ethers::core::abi::ethabi::ParamType::String,
                                internal_type: ::core::option::Option::Some(
                                    ::std::borrow::ToOwned::to_owned("string"),
                                ),
                            },
                            ::ethers::core::abi::ethabi::Param {
                                name: ::std::string::String::new(),
                                kind: ::ethers::core::abi::ethabi::ParamType::String,
                                internal_type: ::core::option::Option::Some(
                                    ::std::borrow::ToOwned::to_owned("string"),
                                ),
                            },
                            ::ethers::core::abi::ethabi::Param {
                                name: ::std::string::String::new(),
                                kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                internal_type: ::core::option::Option::Some(
                                    ::std::borrow::ToOwned::to_owned("uint64"),
                                ),
                            },
                        ],
                        outputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::string::String::new(),
                            kind: ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("uint8"),
                            ),
                        },],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("registerClient"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("registerClient"),
                        inputs: ::std::vec![
                            ::ethers::core::abi::ethabi::Param {
                                name: ::std::borrow::ToOwned::to_owned("clientType"),
                                kind: ::ethers::core::abi::ethabi::ParamType::String,
                                internal_type: ::core::option::Option::Some(
                                    ::std::borrow::ToOwned::to_owned("string"),
                                ),
                            },
                            ::ethers::core::abi::ethabi::Param {
                                name: ::std::borrow::ToOwned::to_owned("client"),
                                kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                internal_type: ::core::option::Option::Some(
                                    ::std::borrow::ToOwned::to_owned("contract ILightClient"),
                                ),
                            },
                        ],
                        outputs: ::std::vec![],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("updateClient"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("updateClient"),
                        inputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("msg_"),
                            kind: ::ethers::core::abi::ethabi::ParamType::Tuple(::std::vec![
                                ::ethers::core::abi::ethabi::ParamType::String,
                                ::ethers::core::abi::ethabi::ParamType::Bytes,
                            ],),
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("struct IBCMsgs.MsgUpdateClient",),
                            ),
                        },],
                        outputs: ::std::vec![],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                    },],
                ),
            ]),
            events: ::core::convert::From::from([
                (
                    ::std::borrow::ToOwned::to_owned("ClientCreated"),
                    ::std::vec![::ethers::core::abi::ethabi::Event {
                        name: ::std::borrow::ToOwned::to_owned("ClientCreated"),
                        inputs: ::std::vec![::ethers::core::abi::ethabi::EventParam {
                            name: ::std::string::String::new(),
                            kind: ::ethers::core::abi::ethabi::ParamType::String,
                            indexed: false,
                        },],
                        anonymous: false,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("ClientRegistered"),
                    ::std::vec![::ethers::core::abi::ethabi::Event {
                        name: ::std::borrow::ToOwned::to_owned("ClientRegistered"),
                        inputs: ::std::vec![
                            ::ethers::core::abi::ethabi::EventParam {
                                name: ::std::string::String::new(),
                                kind: ::ethers::core::abi::ethabi::ParamType::String,
                                indexed: false,
                            },
                            ::ethers::core::abi::ethabi::EventParam {
                                name: ::std::string::String::new(),
                                kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                indexed: false,
                            },
                        ],
                        anonymous: false,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("ClientUpdated"),
                    ::std::vec![::ethers::core::abi::ethabi::Event {
                        name: ::std::borrow::ToOwned::to_owned("ClientUpdated"),
                        inputs: ::std::vec![
                            ::ethers::core::abi::ethabi::EventParam {
                                name: ::std::string::String::new(),
                                kind: ::ethers::core::abi::ethabi::ParamType::String,
                                indexed: false,
                            },
                            ::ethers::core::abi::ethabi::EventParam {
                                name: ::std::string::String::new(),
                                kind: ::ethers::core::abi::ethabi::ParamType::Tuple(::std::vec![
                                    ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                ],),
                                indexed: false,
                            },
                        ],
                        anonymous: false,
                    },],
                ),
            ]),
            errors: ::core::convert::From::from([
                (
                    ::std::borrow::ToOwned::to_owned("ErrClientMustNotBeSelf"),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned("ErrClientMustNotBeSelf",),
                        inputs: ::std::vec![],
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("ErrClientNotFound"),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned("ErrClientNotFound"),
                        inputs: ::std::vec![],
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("ErrClientTypeAlreadyExists"),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned("ErrClientTypeAlreadyExists",),
                        inputs: ::std::vec![],
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("ErrClientTypeNotFound"),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned("ErrClientTypeNotFound",),
                        inputs: ::std::vec![],
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("ErrFailedToCreateClient"),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned("ErrFailedToCreateClient",),
                        inputs: ::std::vec![],
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("ErrFailedToUpdateClient"),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned("ErrFailedToUpdateClient",),
                        inputs: ::std::vec![],
                    },],
                ),
            ]),
            receive: false,
            fallback: false,
        }
    }
    ///The parsed JSON ABI of the contract.
    #[cfg(feature = "providers")]
    pub static IBCCLIENT_ABI: ::ethers::contract::Lazy<::ethers::core::abi::Abi> =
        ::ethers::contract::Lazy::new(__abi);
    #[rustfmt::skip]
    #[cfg(feature = "providers")]
    const __BYTECODE: &[u8] = b"`\x80\x80`@R4a\0\x16Wa\x1C\xEB\x90\x81a\0\x1C\x829\xF3[`\0\x80\xFD\xFE`\x80`@R`\x046\x10\x15a\0\x12W`\0\x80\xFD[`\x005`\xE0\x1C\x80c\x13\x90\xD2\x8D\x14a\x15\xE4W\x80c\x18\xC1\x98p\x14a\x14`W\x80c&\x07\x847\x14a\x13\xA5W\x80c1\x97?\0\x14a\x11\xE6W\x80cW\x17\xBC\xF5\x14a\x11hW\x80c[=\xE2`\x14a\x10#W\x80cy&\xB8\xA9\x14a\x0F\xDAW\x80c~\xB7\x892\x14a\x0FhW\x80c\x82\x1C\xB5\xD0\x14a\x0F8W\x80c\x83\x9D\xF9E\x14a\x0E\xEEW\x80c\x99\x04\x91\xA5\x14a\x0EpW\x80c\xA0I\xE6w\x14a\x0E'W\x80c\xA9U\r\xAC\x14a\r\xABW\x80c\xC28\x01\x05\x14a\r\x1DW\x80c\xC90\xB1\xB0\x14a\x0C\xCEW\x80c\xD1){\x8D\x14a\x0C=W\x80c\xD5\xA2D\x81\x14a\x04\x9CW\x80c\xDAl\xEAU\x14a\x018Wc\xE1\xB1{C\x14a\0\xEDW`\0\x80\xFD[4a\x013W`\0\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFC6\x01\x12a\x013W` g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF`\x0BT\x16`@Q\x90\x81R\xF3[`\0\x80\xFD[4a\x013W` \x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFC\x81\x816\x01\x12a\x013W`\x045\x91g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x80\x84\x11a\x013W\x83`\x04\x01\x93`@\x84\x826\x03\x01\x12a\x013Wa\x01\xAAa\x01\xA5a\x01\x9E\x87\x80a\x19wV[6\x91a\x16\xABV[a\x1A\xA4V[\x83\x81Q\x91\x01 `\0R`\0\x83R`@`\0 T\x15a\x04rWa\x02R`\0\x91s\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFFa\x01\xF2a\x01\xEDa\x01\x9E\x8A\x80a\x19wV[a\x194V[\x16\x90\x83`$a\x02aa\x02\x11a\x02\x07\x8C\x80a\x19wV[\x93\x90\x95\x01\x8Ca\x19wV[`@\x9B\x91\x9BQ\x9B\x8C\x98\x89\x97\x88\x96\x7Fo\xBF\x80y\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x88R`@`\x04\x89\x01R`D\x88\x01\x91a\x18\xF5V[\x92\x85\x84\x03\x01`$\x86\x01Ra\x18\xF5V[\x03\x92Z\xF1\x80\x15a\x04fW`\0\x90`\0\x94`\0\x91a\x03\x85W[P\x15a\x03[Wa\x02\x8Fa\x01\xA5a\x01\x9E\x87\x80a\x19wV[\x83\x81Q\x91\x01 `\0R`\0\x83R`@`\0 U`\0[\x83Q\x81\x10\x15a\x03\x14W\x80a\x02\xBB`\x01\x92\x86a\x1AaV[QQa\x03\x01a\x02\xFC\x87a\x02\xCE\x8A\x80a\x19wV[\x92\x90\x88\x8A\x80a\x02\xEE\x8A\x84\x83a\x02\xE3\x83\x8Ba\x1AaV[Q\x01QQ\x16\x97a\x1AaV[Q\x01Q\x01Q\x16\x936\x91a\x16\xABV[a\x1B\x1EV[`\0R`\0\x85R`@`\0 U\x01a\x02\xA5V[\x7FR\x9F\xC6\xFF\xF6\xD1\x8A\xD4\x1F\xA6\x02F\x9BF\x07jX\x15If\x0C\x1D\x18\xF7>\xF5\r\x9A\x18\x92\xD31a\x03V\x84a\x03C\x88\x80a\x19wV[\x92\x90`@Q\x93\x83\x85\x94\x85R\x84\x01\x91a\x18\xF5V[\x03\x90\xA1\0[`\x04`@Q\x7F\xF1i\x119\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R\xFD[\x94PPP=\x80`\0\x85>a\x03\x99\x81\x85a\x160V[\x83\x01``\x93``\x81\x83\x03\x12a\x013W\x80Q\x94\x84\x82\x01Q\x84\x81\x11a\x013W\x82\x01\x83`\x1F\x82\x01\x12\x15a\x013W\x80Q\x85\x81\x11a\x047W`@Q\x94a\x03\xDF\x88\x83`\x05\x1B\x01\x87a\x160V[\x81\x86R\x87``\x81\x88\x01\x93\x02\x84\x01\x01\x92\x81\x84\x11a\x013W\x88\x80\x91\x01\x92[\x84\x84\x10a\x04\x1DWPPPPPP`@a\x04\x14\x91\x01a\x1ATV[\x93\x90\x93\x86a\x02yV[\x90\x85\x91a\x04*\x84\x86a\x19\xDDV[\x81R\x01\x92\x01\x91\x88\x90a\x03\xFBV[\x7FNH{q\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`\0R`A`\x04R`$`\0\xFD[`@Q=`\0\x82>=\x90\xFD[`\x04`@Q\x7F\xB6\xC7\x1F}\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R\xFD[4a\x013W` \x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFC6\x01\x12a\x013Wg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF`\x045\x11a\x013W``\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFC`\x0456\x03\x01\x12a\x013Ws\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF` a\x05/`\x04\x805\x01\x80a\x19wV[\x91\x90\x82`@Q\x93\x84\x92\x837\x81\x01`\x01\x81R\x03\x01\x90 T\x16\x80\x15a\x0C\x13Wa\x05Z`\x04\x805\x01\x80a\x19wV[`\x0BT\x90\x91g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x16\x90`\0\x82\x80z\x18O\x03\xE9?\xF9\xF4\xDA\xA7\x97\xEDn8\xEDd\xBFj\x1F\x01\0\0\0\0\0\0\0\0\x80\x82\x10\x15a\x0C\x04W[P`\n\x90m\x04\xEE-mA[\x85\xAC\xEF\x81\0\0\0\0\x80\x82\x10\x15a\x0B\xF7W[Pf#\x86\xF2o\xC1\0\0\x80\x82\x10\x15a\x0B\xEAW[Pc\x05\xF5\xE1\0\x80\x82\x10\x15a\x0B\xDDW[Pa'\x10\x80\x82\x10\x15a\x0B\xD0W[P`d\x81\x10\x15a\x0B\xC2W[\x10\x15a\x0B\xB7W[`\n\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF`!`\x01\x85\x01\x94\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xE0a\x06]a\x06G\x88a\x16qV[\x97a\x06U`@Q\x99\x8Aa\x160V[\x80\x89Ra\x16qV[\x016` \x88\x017\x85\x01\x01[\x01\x91\x7F0123456789abcdef\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x82\x82\x06\x1A\x83S\x04\x90\x81\x15a\x06\xC3W`\n\x90\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x90a\x06hV[PPa\x07\"\x91`!\x91\x86`@Q\x97\x88\x93` \x85\x017\x82\x01\x7F-\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0` \x82\x01Ra\x07\x13\x82Q\x80\x93` \x87\x85\x01\x91\x01a\x17bV[\x01\x03`\x01\x81\x01\x86R\x01\x84a\x160V[g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x14a\x0B\x88Wg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF`\x01\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\x92\x01\x16\x91\x16\x17`\x0BUa\x07z`\x045`\x04\x01`\x045`\x04\x01a\x19wV[`@\x93\x91\x93Q` \x81\x85Qa\x07\x92\x81\x83\x85\x8A\x01a\x17bV[\x81\x01`\x02\x81R\x03\x01\x90 \x90g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x047Wa\x07\xB6\x82Ta\x17\xABV[`\x1F\x81\x11a\x0BDW[P`\0`\x1F\x82\x11`\x01\x14a\nvW\x91\x81`\xA0\x94\x92a\x08\xFD\x94a\t-\x98`\0\x92a\nkW[PP\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82`\x01\x1B\x92`\x03\x1B\x1C\x19\x16\x17\x90U[`@Q` \x81\x86Qa\x08*\x81\x83\x85\x8B\x01a\x17bV[\x81\x01`\x03\x81R\x03\x01\x90 \x81\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x82T\x16\x17\x90U`\0a\x08r`$`\x045\x01`\x045`\x04\x01a\x19wV[a\x08\x89`D`\x04\x94\x93\x945\x01`\x045`\x04\x01a\x19wV[\x98\x90`@Q\x99\x8A\x97\x88\x96\x87\x95\x7F&)ck\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x87R```\x04\x88\x01Ra\x08\xCD\x8D`d\x89\x01\x90a\x18\xB2V[\x91\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFC\x88\x84\x03\x01`$\x89\x01Ra\x18\xF5V[\x91\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFC\x85\x84\x03\x01`D\x86\x01Ra\x18\xF5V[\x03\x92Z\xF1\x91\x82\x15a\x04fW`\0\x92`\0\x91`\0\x91a\n\x1BW[P\x15a\t\xF1Wa\t\xED\x92a\tY\x83a\x1A\xA4V[` \x81Q\x91\x01 `\0R`\0` R`@`\0 Ua\t\x92` \x82Q\x92\x01Qg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF` \x81\x83Q\x16\x92\x01Q\x16\x90\x84a\x1B\x1EV[`\0R`\0` R`@`\0 U\x7F\xEB\x98\xDFG\r\x17&e8\xE4\xEE\x03IR f!\xFA\xD8\xD8l\xA3\x8B\t\x0E\x92\xF6E\x89\x10\x84\x82`@Q` \x81R\x80a\t\xD6` \x82\x01\x85a\x18\xB2V[\x03\x90\xA1`@Q\x91\x82\x91` \x83R` \x83\x01\x90a\x18\xB2V[\x03\x90\xF3[`\x04`@Q\x7F\x8B\x9F\x95\xB2\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R\xFD[\x93PPP`\xA0=`\xA0\x11a\ndW[a\n4\x81\x84a\x160V[\x82\x01`\xA0\x83\x82\x03\x12a\x013Wa\n[`\x80a\nT\x85Q\x93` \x87\x01a\x19\xDDV[\x94\x01a\x1ATV[\x90\x92\x90\x84a\tFV[P=a\n*V[\x015\x90P\x88\x80a\x07\xE3V[\x82`\0R` `\0 \x90`\0[\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xE0\x84\x16\x81\x10a\x0B,WP\x82a\x08\xFD\x94\x92a\t-\x98`\x01\x93\x83\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xE0`\xA0\x9A\x98\x16\x10a\n\xF4W[PPP\x81\x1B\x01\x90Ua\x08\x15V[\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF`\xF8\x85`\x03\x1B\x16\x1C\x19\x91\x015\x16\x90U\x88\x80\x80a\n\xE7V[\x90\x91` `\x01\x81\x92\x85\x8B\x015\x81U\x01\x93\x01\x91\x01a\n\x83V[\x82`\0R` `\0 `\x1F\x83\x01`\x05\x1C\x81\x01` \x84\x10a\x0B\x81W[`\x1F\x83\x01`\x05\x1C\x82\x01\x81\x10a\x0BuWPPa\x07\xBFV[`\0\x81U`\x01\x01a\x0B_V[P\x80a\x0B_V[\x7FNH{q\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`\0R`\x11`\x04R`$`\0\xFD[`\x01\x90\x91\x01\x90a\x05\xF0V[`d`\x02\x91\x04\x93\x01\x92a\x05\xE9V[`\x04\x91\x04\x93\x01\x92\x89a\x05\xDEV[`\x08\x91\x04\x93\x01\x92\x89a\x05\xD1V[`\x10\x91\x04\x93\x01\x92\x89a\x05\xC2V[` \x91\x04\x93\x01\x92\x89a\x05\xB0V[`@\x93P\x82\x04\x90P`\na\x05\x94V[`\x04`@Q\x7F\xAAG\x8A\xF9\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R\xFD[4a\x013W` \x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFC6\x01\x12a\x013W`\x045g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x013Ws\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFFa\x0C\xBA` a\x0C\xA7\x81\x946\x90`\x04\x01a\x16\xE2V[\x81`@Q\x93\x82\x85\x80\x94Q\x93\x84\x92\x01a\x17bV[\x81\x01`\x03\x81R\x03\x01\x90 T\x16`@Q\x90\x81R\xF3[4a\x013W` g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFFa\r\x13\x82a\r\x03a\x0C\xEE6a\x17\0V[\x92\x90\x81`@Q\x93\x82\x85\x80\x94Q\x93\x84\x92\x01a\x17bV[\x81\x01`\x07\x81R\x03\x01\x90 \x90a\x17\x85V[T\x16`@Q\x90\x81R\xF3[4a\x013W` \x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFC6\x01\x12a\x013W`\x045g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x013Wa\r\x90a\r\x97a\rz` a\x0C\xA7a\t\xED\x956\x90`\x04\x01a\x16\xE2V[\x81\x01`\x02\x81R\x03\x01\x90 `@Q\x92\x83\x80\x92a\x17\xFEV[\x03\x82a\x160V[`@Q\x91\x82\x91` \x83R` \x83\x01\x90a\x18\xB2V[4a\x013W`\0\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFC6\x01\x12a\x013Wa\t\xED`@Qa\r\xE9\x81a\x16\x14V[`\x03\x81R\x7Fibc\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0` \x82\x01R`@Q\x91\x82\x91` \x83R` \x83\x01\x90a\x18\xB2V[4a\x013W`\0\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFC6\x01\x12a\x013W` g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF`\x0BT`@\x1C\x16`@Q\x90\x81R\xF3[4a\x013W` \x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFC6\x01\x12a\x013W`\x045g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x013Ws\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFFa\x0E\xDA` a\x0C\xA7\x81\x946\x90`\x04\x01a\x16\xE2V[\x81\x01`\x01\x81R\x03\x01\x90 T\x16`@Q\x90\x81R\xF3[4a\x013W` \x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFC6\x01\x12a\x013W`\x045`\0R`\0` R` `@`\0 T`@Q\x90\x81R\xF3[4a\x013W` g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFFa\r\x13\x82a\x0FXa\x0C\xEE6a\x17\0V[\x81\x01`\x06\x81R\x03\x01\x90 \x90a\x17\x85V[4a\x013W` \x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFC6\x01\x12a\x013W`\x045g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x013Wa\x0F\xBCa\x01\xED` \x926\x90`\x04\x01a\x16\xE2V[s\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF`@Q\x91\x16\x81R\xF3[4a\x013W`\0\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFC6\x01\x12a\x013W` g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF`\x0BT`\x80\x1C\x16`@Q\x90\x81R\xF3[4a\x013Wa\x10I` a\x109a\x0C\xEE6a\x17\0V[\x81\x01`\x05\x81R\x03\x01\x90 \x90a\x17\x85V[\x80T\x90`\xFF\x80\x83\x16\x92`\x08\x1C\x16`\x04`@Qa\x10d\x81a\x16\x14V[`@Qa\x10x\x81a\r\x90\x81`\x01\x89\x01a\x17\xFEV[\x81Ra\x10\xB7`@Q\x94a\x10\x99\x86a\x10\x92\x81`\x02\x85\x01a\x17\xFEV[\x03\x87a\x160V[` \x83\x01\x95\x86Ra\x10\xB0`@Q\x80\x95\x81\x93\x01a\x17\xFEV[\x03\x83a\x160V[`@Q\x94`\x05\x81\x10\x15a\x119W\x85R`\x03\x83\x10\x15a\x119W\x84\x93a\x10\xFAa\x11+\x92a\t\xED\x95` \x88\x01R`\x80`@\x88\x01RQ`@`\x80\x88\x01R`\xC0\x87\x01\x90a\x18\xB2V[\x90Q\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x80\x86\x83\x03\x01`\xA0\x87\x01Ra\x18\xB2V[\x90\x83\x82\x03``\x85\x01Ra\x18\xB2V[\x7FNH{q\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`\0R`!`\x04R`$`\0\xFD[4a\x013W` \x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFC6\x01\x12a\x013W`\x045g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x013Ws\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFFa\x11\xD2` a\x0C\xA7\x81\x946\x90`\x04\x01a\x16\xE2V[\x81\x01`\n\x81R\x03\x01\x90 T\x16`@Q\x90\x81R\xF3[4a\x013W` \x80\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFC6\x01\x12a\x013Wg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF`\x045\x81\x81\x11a\x013W\x82a\x0C\xA7a\x12;\x926\x90`\x04\x01a\x16\xE2V[\x81\x01`\x04\x81R\x03\x01\x90 \x91`@Q\x92a\x12_\x84a\x12X\x81\x84a\x17\xFEV[\x03\x85a\x160V[`\xFF`\x02\x82\x01T\x16\x90`@Q\x92``\x84\x01\x84\x81\x10\x86\x82\x11\x17a\x047W\x80`@Ra\x12\xB9\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xA0\x86a\x12\xB1\x84`\x03\x88\x01a\x17\xFEV[\x03\x01\x82a\x160V[\x84R`@Q\x94a\x12\xD0\x86a\x10\x92\x81`\x04\x87\x01a\x17\xFEV[\x81\x85\x01\x95\x86R`@Q\x92\x82\x84\x01\x90\x84\x82\x10\x83\x83\x11\x17a\x047W\x81`\x06\x92`@Ra\x13\"\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xE0\x87a\x12\xB1\x84`\x05\x87\x01a\x17\xFEV[\x85R`@\x87\x01\x94\x85R\x01T\x16\x93a\x13D`@Q\x97`\x80\x89R`\x80\x89\x01\x90a\x18\xB2V[\x90`\x04\x85\x10\x15a\x119W\x87\x96a\x13va\x13\x84\x92a\x13\x9B\x97\x86\x8B\x01R\x89\x85\x03`@\x8B\x01RQ``\x85R``\x85\x01\x90a\x18\xB2V[\x90Q\x83\x82\x03\x85\x85\x01Ra\x18\xB2V[\x92Q\x90`@\x81\x85\x03\x91\x01RQ\x91\x81\x81R\x01\x90a\x18\xB2V[\x90``\x83\x01R\x03\x90\xF3[4a\x013W``\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFC6\x01\x12a\x013Wg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF`\x045\x81\x81\x11a\x013Wa\x13\xF5\x906\x90`\x04\x01a\x16\xE2V[`$5\x82\x81\x11a\x013Wa\x14\r\x906\x90`\x04\x01a\x16\xE2V[`D5\x92\x83\x16\x80\x93\x03a\x013Wa\x146` a\x14F\x93\x81`@Q\x93\x82\x85\x80\x94Q\x93\x84\x92\x01a\x17bV[\x81\x01`\t\x81R\x03\x01\x90 \x90a\x17\x85V[\x90`\0R` R` `\xFF`@`\0 T\x16`@Q\x90\x81R\xF3[4a\x013W`@\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFC6\x01\x12a\x013Wg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF`\x045\x81\x81\x11a\x013W6`#\x82\x01\x12\x15a\x013W\x80`\x04\x015\x91\x82\x11a\x013W`$\x81\x01\x90`$\x836\x92\x01\x01\x11a\x013W`$5\x91s\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x80\x84\x16\x80\x94\x03a\x013W`@Q\x82\x84\x827` \x81\x84\x81\x01`\x01\x81R\x03\x01\x90 T\x16a\x15\xBAW0\x83\x14a\x15\x90W\x7F\xF7\x80\x9E\xF0\xAEyO\xDAda;\xF1\xA5\xBF,\x8DF\x12\x0EKLo\xCC\x1D\xD9\xFBf\xA4V\xAC;A\x92`@Q\x82\x84\x827` \x81\x84\x81\x01`\x01\x81R\x03\x01\x90 \x81\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x82T\x16\x17\x90Ua\x15\x85`@Q\x93\x84\x93`@\x85R`@\x85\x01\x91a\x18\xF5V[\x90` \x83\x01R\x03\x90\xA1\0[`\x04`@Q\x7FF\x8E\xF7\x87\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R\xFD[`\x04`@Q\x7F\x0C|\xC9\xB9\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R\xFD[4a\x013W` g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFFa\r\x13\x82a\x16\x04a\x0C\xEE6a\x17\0V[\x81\x01`\x08\x81R\x03\x01\x90 \x90a\x17\x85V[`@\x81\x01\x90\x81\x10g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x17a\x047W`@RV[\x90`\x1F\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xE0\x91\x01\x16\x81\x01\x90\x81\x10g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x17a\x047W`@RV[g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x047W`\x1F\x01\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xE0\x16` \x01\x90V[\x92\x91\x92a\x16\xB7\x82a\x16qV[\x91a\x16\xC5`@Q\x93\x84a\x160V[\x82\x94\x81\x84R\x81\x83\x01\x11a\x013W\x82\x81` \x93\x84`\0\x96\x017\x01\x01RV[\x90\x80`\x1F\x83\x01\x12\x15a\x013W\x81` a\x16\xFD\x935\x91\x01a\x16\xABV[\x90V[\x90`@\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFC\x83\x01\x12a\x013Wg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF`\x045\x81\x81\x11a\x013W\x83a\x17K\x91`\x04\x01a\x16\xE2V[\x92`$5\x91\x82\x11a\x013Wa\x16\xFD\x91`\x04\x01a\x16\xE2V[`\0[\x83\x81\x10a\x17uWPP`\0\x91\x01RV[\x81\x81\x01Q\x83\x82\x01R` \x01a\x17eV[` \x90a\x17\x9F\x92\x82`@Q\x94\x83\x86\x80\x95Q\x93\x84\x92\x01a\x17bV[\x82\x01\x90\x81R\x03\x01\x90 \x90V[\x90`\x01\x82\x81\x1C\x92\x16\x80\x15a\x17\xF4W[` \x83\x10\x14a\x17\xC5WV[\x7FNH{q\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`\0R`\"`\x04R`$`\0\xFD[\x91`\x7F\x16\x91a\x17\xBAV[\x80T`\0\x93\x92a\x18\r\x82a\x17\xABV[\x91\x82\x82R` \x93`\x01\x91`\x01\x81\x16\x90\x81`\0\x14a\x18uWP`\x01\x14a\x184W[PPPPPV[\x90\x93\x94\x95P`\0\x92\x91\x92R\x83`\0 \x92\x84`\0\x94[\x83\x86\x10a\x18aWPPPP\x01\x01\x908\x80\x80\x80\x80a\x18-V[\x80T\x85\x87\x01\x83\x01R\x94\x01\x93\x85\x90\x82\x01a\x18IV[\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\0\x16\x86\x85\x01RPPP\x90\x15\x15`\x05\x1B\x01\x01\x91P8\x80\x80\x80\x80a\x18-V[\x90\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xE0`\x1F` \x93a\x18\xEE\x81Q\x80\x92\x81\x87R\x87\x80\x88\x01\x91\x01a\x17bV[\x01\x16\x01\x01\x90V[`\x1F\x82` \x94\x93\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xE0\x93\x81\x86R\x86\x86\x017`\0\x85\x82\x86\x01\x01R\x01\x16\x01\x01\x90V[a\x19b` s\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x92\x81`@Q\x93\x82\x85\x80\x94Q\x93\x84\x92\x01a\x17bV[\x81\x01`\x03\x81R\x03\x01\x90 T\x16\x80\x15a\x04rW\x90V[\x905\x90\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xE1\x816\x03\x01\x82\x12\x15a\x013W\x01\x805\x90g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11a\x013W` \x01\x91\x816\x03\x83\x13a\x013WV[Q\x90g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x16\x82\x03a\x013WV[\x80\x92\x91\x03\x91``\x83\x12a\x013W`@Qa\x19\xF6\x81a\x16\x14V[`@\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xE0\x82\x95\x84Q\x84R\x01\x12a\x013W` \x90a\x1AL`@\x80Q\x94a\x1A9\x86a\x16\x14V[a\x1AD\x85\x82\x01a\x19\xC8V[\x86R\x01a\x19\xC8V[\x82\x84\x01R\x01RV[Q\x90\x81\x15\x15\x82\x03a\x013WV[\x80Q\x82\x10\x15a\x1AuW` \x91`\x05\x1B\x01\x01\x90V[\x7FNH{q\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`\0R`2`\x04R`$`\0\xFD[a\x16\xFD`4`@Q\x80\x93\x7Fclients/\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0` \x83\x01Ra\x1A\xE8\x81Q\x80\x92` `(\x86\x01\x91\x01a\x17bV[\x81\x01\x7F/clientState\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`(\x82\x01R\x03`\x14\x81\x01\x84R\x01\x82a\x160V[\x91\x90`:a\x1BDa\x1C\x0C\x92a\x1B=g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x80\x92\x16a\x1C\x12V[\x94\x16a\x1C\x12V[\x92`@Q\x93\x84\x91` \x83\x01\x96\x7Fclients/\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x88Ra\x1B\x87\x81Q\x80\x92` `(\x88\x01\x91\x01a\x17bV[\x83\x01\x7F/consensusStates/\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`(\x82\x01Ra\x1B\xC3\x82Q\x80\x93` `9\x85\x01\x91\x01a\x17bV[\x01\x7F-\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`9\x82\x01Ra\x1B\xFD\x82Q\x80\x93` \x87\x85\x01\x91\x01a\x17bV[\x01\x03`\x1A\x81\x01\x84R\x01\x82a\x160V[Q\x90 \x90V[\x90`@Q\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF`\x80\x82\x01\x93`\xA0\x83\x01`@R`\0\x85R\x93[\x01\x92`\n\x90\x81\x81\x06`0\x01\x85S\x04\x92\x83\x15a\x1C\x85W\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x90a\x1CIV[\x92P`\x80\x83\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xE0\x92\x03\x01\x92\x01\x91\x82RV\xFE\xA2dipfsX\"\x12 \x13`\xEC\xC2\xF2\xDD/@\xAC\xFB\x92C\x99\x8C?\xD1\xE8\xFAXw\x8E&L\x1C\xF6\x946\xFB\x10\xEEf\x87dsolcC\0\x08\x17\x003";
    /// The bytecode of the contract.
    #[cfg(feature = "providers")]
    pub static IBCCLIENT_BYTECODE: ::ethers::core::types::Bytes =
        ::ethers::core::types::Bytes::from_static(__BYTECODE);
    #[rustfmt::skip]
    #[cfg(feature = "providers")]
    const __DEPLOYED_BYTECODE: &[u8] = b"`\x80`@R`\x046\x10\x15a\0\x12W`\0\x80\xFD[`\x005`\xE0\x1C\x80c\x13\x90\xD2\x8D\x14a\x15\xE4W\x80c\x18\xC1\x98p\x14a\x14`W\x80c&\x07\x847\x14a\x13\xA5W\x80c1\x97?\0\x14a\x11\xE6W\x80cW\x17\xBC\xF5\x14a\x11hW\x80c[=\xE2`\x14a\x10#W\x80cy&\xB8\xA9\x14a\x0F\xDAW\x80c~\xB7\x892\x14a\x0FhW\x80c\x82\x1C\xB5\xD0\x14a\x0F8W\x80c\x83\x9D\xF9E\x14a\x0E\xEEW\x80c\x99\x04\x91\xA5\x14a\x0EpW\x80c\xA0I\xE6w\x14a\x0E'W\x80c\xA9U\r\xAC\x14a\r\xABW\x80c\xC28\x01\x05\x14a\r\x1DW\x80c\xC90\xB1\xB0\x14a\x0C\xCEW\x80c\xD1){\x8D\x14a\x0C=W\x80c\xD5\xA2D\x81\x14a\x04\x9CW\x80c\xDAl\xEAU\x14a\x018Wc\xE1\xB1{C\x14a\0\xEDW`\0\x80\xFD[4a\x013W`\0\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFC6\x01\x12a\x013W` g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF`\x0BT\x16`@Q\x90\x81R\xF3[`\0\x80\xFD[4a\x013W` \x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFC\x81\x816\x01\x12a\x013W`\x045\x91g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x80\x84\x11a\x013W\x83`\x04\x01\x93`@\x84\x826\x03\x01\x12a\x013Wa\x01\xAAa\x01\xA5a\x01\x9E\x87\x80a\x19wV[6\x91a\x16\xABV[a\x1A\xA4V[\x83\x81Q\x91\x01 `\0R`\0\x83R`@`\0 T\x15a\x04rWa\x02R`\0\x91s\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFFa\x01\xF2a\x01\xEDa\x01\x9E\x8A\x80a\x19wV[a\x194V[\x16\x90\x83`$a\x02aa\x02\x11a\x02\x07\x8C\x80a\x19wV[\x93\x90\x95\x01\x8Ca\x19wV[`@\x9B\x91\x9BQ\x9B\x8C\x98\x89\x97\x88\x96\x7Fo\xBF\x80y\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x88R`@`\x04\x89\x01R`D\x88\x01\x91a\x18\xF5V[\x92\x85\x84\x03\x01`$\x86\x01Ra\x18\xF5V[\x03\x92Z\xF1\x80\x15a\x04fW`\0\x90`\0\x94`\0\x91a\x03\x85W[P\x15a\x03[Wa\x02\x8Fa\x01\xA5a\x01\x9E\x87\x80a\x19wV[\x83\x81Q\x91\x01 `\0R`\0\x83R`@`\0 U`\0[\x83Q\x81\x10\x15a\x03\x14W\x80a\x02\xBB`\x01\x92\x86a\x1AaV[QQa\x03\x01a\x02\xFC\x87a\x02\xCE\x8A\x80a\x19wV[\x92\x90\x88\x8A\x80a\x02\xEE\x8A\x84\x83a\x02\xE3\x83\x8Ba\x1AaV[Q\x01QQ\x16\x97a\x1AaV[Q\x01Q\x01Q\x16\x936\x91a\x16\xABV[a\x1B\x1EV[`\0R`\0\x85R`@`\0 U\x01a\x02\xA5V[\x7FR\x9F\xC6\xFF\xF6\xD1\x8A\xD4\x1F\xA6\x02F\x9BF\x07jX\x15If\x0C\x1D\x18\xF7>\xF5\r\x9A\x18\x92\xD31a\x03V\x84a\x03C\x88\x80a\x19wV[\x92\x90`@Q\x93\x83\x85\x94\x85R\x84\x01\x91a\x18\xF5V[\x03\x90\xA1\0[`\x04`@Q\x7F\xF1i\x119\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R\xFD[\x94PPP=\x80`\0\x85>a\x03\x99\x81\x85a\x160V[\x83\x01``\x93``\x81\x83\x03\x12a\x013W\x80Q\x94\x84\x82\x01Q\x84\x81\x11a\x013W\x82\x01\x83`\x1F\x82\x01\x12\x15a\x013W\x80Q\x85\x81\x11a\x047W`@Q\x94a\x03\xDF\x88\x83`\x05\x1B\x01\x87a\x160V[\x81\x86R\x87``\x81\x88\x01\x93\x02\x84\x01\x01\x92\x81\x84\x11a\x013W\x88\x80\x91\x01\x92[\x84\x84\x10a\x04\x1DWPPPPPP`@a\x04\x14\x91\x01a\x1ATV[\x93\x90\x93\x86a\x02yV[\x90\x85\x91a\x04*\x84\x86a\x19\xDDV[\x81R\x01\x92\x01\x91\x88\x90a\x03\xFBV[\x7FNH{q\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`\0R`A`\x04R`$`\0\xFD[`@Q=`\0\x82>=\x90\xFD[`\x04`@Q\x7F\xB6\xC7\x1F}\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R\xFD[4a\x013W` \x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFC6\x01\x12a\x013Wg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF`\x045\x11a\x013W``\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFC`\x0456\x03\x01\x12a\x013Ws\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF` a\x05/`\x04\x805\x01\x80a\x19wV[\x91\x90\x82`@Q\x93\x84\x92\x837\x81\x01`\x01\x81R\x03\x01\x90 T\x16\x80\x15a\x0C\x13Wa\x05Z`\x04\x805\x01\x80a\x19wV[`\x0BT\x90\x91g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x16\x90`\0\x82\x80z\x18O\x03\xE9?\xF9\xF4\xDA\xA7\x97\xEDn8\xEDd\xBFj\x1F\x01\0\0\0\0\0\0\0\0\x80\x82\x10\x15a\x0C\x04W[P`\n\x90m\x04\xEE-mA[\x85\xAC\xEF\x81\0\0\0\0\x80\x82\x10\x15a\x0B\xF7W[Pf#\x86\xF2o\xC1\0\0\x80\x82\x10\x15a\x0B\xEAW[Pc\x05\xF5\xE1\0\x80\x82\x10\x15a\x0B\xDDW[Pa'\x10\x80\x82\x10\x15a\x0B\xD0W[P`d\x81\x10\x15a\x0B\xC2W[\x10\x15a\x0B\xB7W[`\n\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF`!`\x01\x85\x01\x94\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xE0a\x06]a\x06G\x88a\x16qV[\x97a\x06U`@Q\x99\x8Aa\x160V[\x80\x89Ra\x16qV[\x016` \x88\x017\x85\x01\x01[\x01\x91\x7F0123456789abcdef\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x82\x82\x06\x1A\x83S\x04\x90\x81\x15a\x06\xC3W`\n\x90\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x90a\x06hV[PPa\x07\"\x91`!\x91\x86`@Q\x97\x88\x93` \x85\x017\x82\x01\x7F-\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0` \x82\x01Ra\x07\x13\x82Q\x80\x93` \x87\x85\x01\x91\x01a\x17bV[\x01\x03`\x01\x81\x01\x86R\x01\x84a\x160V[g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x14a\x0B\x88Wg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF`\x01\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\x92\x01\x16\x91\x16\x17`\x0BUa\x07z`\x045`\x04\x01`\x045`\x04\x01a\x19wV[`@\x93\x91\x93Q` \x81\x85Qa\x07\x92\x81\x83\x85\x8A\x01a\x17bV[\x81\x01`\x02\x81R\x03\x01\x90 \x90g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x047Wa\x07\xB6\x82Ta\x17\xABV[`\x1F\x81\x11a\x0BDW[P`\0`\x1F\x82\x11`\x01\x14a\nvW\x91\x81`\xA0\x94\x92a\x08\xFD\x94a\t-\x98`\0\x92a\nkW[PP\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82`\x01\x1B\x92`\x03\x1B\x1C\x19\x16\x17\x90U[`@Q` \x81\x86Qa\x08*\x81\x83\x85\x8B\x01a\x17bV[\x81\x01`\x03\x81R\x03\x01\x90 \x81\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x82T\x16\x17\x90U`\0a\x08r`$`\x045\x01`\x045`\x04\x01a\x19wV[a\x08\x89`D`\x04\x94\x93\x945\x01`\x045`\x04\x01a\x19wV[\x98\x90`@Q\x99\x8A\x97\x88\x96\x87\x95\x7F&)ck\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x87R```\x04\x88\x01Ra\x08\xCD\x8D`d\x89\x01\x90a\x18\xB2V[\x91\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFC\x88\x84\x03\x01`$\x89\x01Ra\x18\xF5V[\x91\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFC\x85\x84\x03\x01`D\x86\x01Ra\x18\xF5V[\x03\x92Z\xF1\x91\x82\x15a\x04fW`\0\x92`\0\x91`\0\x91a\n\x1BW[P\x15a\t\xF1Wa\t\xED\x92a\tY\x83a\x1A\xA4V[` \x81Q\x91\x01 `\0R`\0` R`@`\0 Ua\t\x92` \x82Q\x92\x01Qg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF` \x81\x83Q\x16\x92\x01Q\x16\x90\x84a\x1B\x1EV[`\0R`\0` R`@`\0 U\x7F\xEB\x98\xDFG\r\x17&e8\xE4\xEE\x03IR f!\xFA\xD8\xD8l\xA3\x8B\t\x0E\x92\xF6E\x89\x10\x84\x82`@Q` \x81R\x80a\t\xD6` \x82\x01\x85a\x18\xB2V[\x03\x90\xA1`@Q\x91\x82\x91` \x83R` \x83\x01\x90a\x18\xB2V[\x03\x90\xF3[`\x04`@Q\x7F\x8B\x9F\x95\xB2\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R\xFD[\x93PPP`\xA0=`\xA0\x11a\ndW[a\n4\x81\x84a\x160V[\x82\x01`\xA0\x83\x82\x03\x12a\x013Wa\n[`\x80a\nT\x85Q\x93` \x87\x01a\x19\xDDV[\x94\x01a\x1ATV[\x90\x92\x90\x84a\tFV[P=a\n*V[\x015\x90P\x88\x80a\x07\xE3V[\x82`\0R` `\0 \x90`\0[\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xE0\x84\x16\x81\x10a\x0B,WP\x82a\x08\xFD\x94\x92a\t-\x98`\x01\x93\x83\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xE0`\xA0\x9A\x98\x16\x10a\n\xF4W[PPP\x81\x1B\x01\x90Ua\x08\x15V[\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF`\xF8\x85`\x03\x1B\x16\x1C\x19\x91\x015\x16\x90U\x88\x80\x80a\n\xE7V[\x90\x91` `\x01\x81\x92\x85\x8B\x015\x81U\x01\x93\x01\x91\x01a\n\x83V[\x82`\0R` `\0 `\x1F\x83\x01`\x05\x1C\x81\x01` \x84\x10a\x0B\x81W[`\x1F\x83\x01`\x05\x1C\x82\x01\x81\x10a\x0BuWPPa\x07\xBFV[`\0\x81U`\x01\x01a\x0B_V[P\x80a\x0B_V[\x7FNH{q\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`\0R`\x11`\x04R`$`\0\xFD[`\x01\x90\x91\x01\x90a\x05\xF0V[`d`\x02\x91\x04\x93\x01\x92a\x05\xE9V[`\x04\x91\x04\x93\x01\x92\x89a\x05\xDEV[`\x08\x91\x04\x93\x01\x92\x89a\x05\xD1V[`\x10\x91\x04\x93\x01\x92\x89a\x05\xC2V[` \x91\x04\x93\x01\x92\x89a\x05\xB0V[`@\x93P\x82\x04\x90P`\na\x05\x94V[`\x04`@Q\x7F\xAAG\x8A\xF9\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R\xFD[4a\x013W` \x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFC6\x01\x12a\x013W`\x045g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x013Ws\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFFa\x0C\xBA` a\x0C\xA7\x81\x946\x90`\x04\x01a\x16\xE2V[\x81`@Q\x93\x82\x85\x80\x94Q\x93\x84\x92\x01a\x17bV[\x81\x01`\x03\x81R\x03\x01\x90 T\x16`@Q\x90\x81R\xF3[4a\x013W` g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFFa\r\x13\x82a\r\x03a\x0C\xEE6a\x17\0V[\x92\x90\x81`@Q\x93\x82\x85\x80\x94Q\x93\x84\x92\x01a\x17bV[\x81\x01`\x07\x81R\x03\x01\x90 \x90a\x17\x85V[T\x16`@Q\x90\x81R\xF3[4a\x013W` \x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFC6\x01\x12a\x013W`\x045g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x013Wa\r\x90a\r\x97a\rz` a\x0C\xA7a\t\xED\x956\x90`\x04\x01a\x16\xE2V[\x81\x01`\x02\x81R\x03\x01\x90 `@Q\x92\x83\x80\x92a\x17\xFEV[\x03\x82a\x160V[`@Q\x91\x82\x91` \x83R` \x83\x01\x90a\x18\xB2V[4a\x013W`\0\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFC6\x01\x12a\x013Wa\t\xED`@Qa\r\xE9\x81a\x16\x14V[`\x03\x81R\x7Fibc\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0` \x82\x01R`@Q\x91\x82\x91` \x83R` \x83\x01\x90a\x18\xB2V[4a\x013W`\0\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFC6\x01\x12a\x013W` g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF`\x0BT`@\x1C\x16`@Q\x90\x81R\xF3[4a\x013W` \x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFC6\x01\x12a\x013W`\x045g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x013Ws\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFFa\x0E\xDA` a\x0C\xA7\x81\x946\x90`\x04\x01a\x16\xE2V[\x81\x01`\x01\x81R\x03\x01\x90 T\x16`@Q\x90\x81R\xF3[4a\x013W` \x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFC6\x01\x12a\x013W`\x045`\0R`\0` R` `@`\0 T`@Q\x90\x81R\xF3[4a\x013W` g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFFa\r\x13\x82a\x0FXa\x0C\xEE6a\x17\0V[\x81\x01`\x06\x81R\x03\x01\x90 \x90a\x17\x85V[4a\x013W` \x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFC6\x01\x12a\x013W`\x045g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x013Wa\x0F\xBCa\x01\xED` \x926\x90`\x04\x01a\x16\xE2V[s\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF`@Q\x91\x16\x81R\xF3[4a\x013W`\0\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFC6\x01\x12a\x013W` g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF`\x0BT`\x80\x1C\x16`@Q\x90\x81R\xF3[4a\x013Wa\x10I` a\x109a\x0C\xEE6a\x17\0V[\x81\x01`\x05\x81R\x03\x01\x90 \x90a\x17\x85V[\x80T\x90`\xFF\x80\x83\x16\x92`\x08\x1C\x16`\x04`@Qa\x10d\x81a\x16\x14V[`@Qa\x10x\x81a\r\x90\x81`\x01\x89\x01a\x17\xFEV[\x81Ra\x10\xB7`@Q\x94a\x10\x99\x86a\x10\x92\x81`\x02\x85\x01a\x17\xFEV[\x03\x87a\x160V[` \x83\x01\x95\x86Ra\x10\xB0`@Q\x80\x95\x81\x93\x01a\x17\xFEV[\x03\x83a\x160V[`@Q\x94`\x05\x81\x10\x15a\x119W\x85R`\x03\x83\x10\x15a\x119W\x84\x93a\x10\xFAa\x11+\x92a\t\xED\x95` \x88\x01R`\x80`@\x88\x01RQ`@`\x80\x88\x01R`\xC0\x87\x01\x90a\x18\xB2V[\x90Q\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x80\x86\x83\x03\x01`\xA0\x87\x01Ra\x18\xB2V[\x90\x83\x82\x03``\x85\x01Ra\x18\xB2V[\x7FNH{q\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`\0R`!`\x04R`$`\0\xFD[4a\x013W` \x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFC6\x01\x12a\x013W`\x045g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x013Ws\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFFa\x11\xD2` a\x0C\xA7\x81\x946\x90`\x04\x01a\x16\xE2V[\x81\x01`\n\x81R\x03\x01\x90 T\x16`@Q\x90\x81R\xF3[4a\x013W` \x80\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFC6\x01\x12a\x013Wg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF`\x045\x81\x81\x11a\x013W\x82a\x0C\xA7a\x12;\x926\x90`\x04\x01a\x16\xE2V[\x81\x01`\x04\x81R\x03\x01\x90 \x91`@Q\x92a\x12_\x84a\x12X\x81\x84a\x17\xFEV[\x03\x85a\x160V[`\xFF`\x02\x82\x01T\x16\x90`@Q\x92``\x84\x01\x84\x81\x10\x86\x82\x11\x17a\x047W\x80`@Ra\x12\xB9\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xA0\x86a\x12\xB1\x84`\x03\x88\x01a\x17\xFEV[\x03\x01\x82a\x160V[\x84R`@Q\x94a\x12\xD0\x86a\x10\x92\x81`\x04\x87\x01a\x17\xFEV[\x81\x85\x01\x95\x86R`@Q\x92\x82\x84\x01\x90\x84\x82\x10\x83\x83\x11\x17a\x047W\x81`\x06\x92`@Ra\x13\"\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xE0\x87a\x12\xB1\x84`\x05\x87\x01a\x17\xFEV[\x85R`@\x87\x01\x94\x85R\x01T\x16\x93a\x13D`@Q\x97`\x80\x89R`\x80\x89\x01\x90a\x18\xB2V[\x90`\x04\x85\x10\x15a\x119W\x87\x96a\x13va\x13\x84\x92a\x13\x9B\x97\x86\x8B\x01R\x89\x85\x03`@\x8B\x01RQ``\x85R``\x85\x01\x90a\x18\xB2V[\x90Q\x83\x82\x03\x85\x85\x01Ra\x18\xB2V[\x92Q\x90`@\x81\x85\x03\x91\x01RQ\x91\x81\x81R\x01\x90a\x18\xB2V[\x90``\x83\x01R\x03\x90\xF3[4a\x013W``\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFC6\x01\x12a\x013Wg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF`\x045\x81\x81\x11a\x013Wa\x13\xF5\x906\x90`\x04\x01a\x16\xE2V[`$5\x82\x81\x11a\x013Wa\x14\r\x906\x90`\x04\x01a\x16\xE2V[`D5\x92\x83\x16\x80\x93\x03a\x013Wa\x146` a\x14F\x93\x81`@Q\x93\x82\x85\x80\x94Q\x93\x84\x92\x01a\x17bV[\x81\x01`\t\x81R\x03\x01\x90 \x90a\x17\x85V[\x90`\0R` R` `\xFF`@`\0 T\x16`@Q\x90\x81R\xF3[4a\x013W`@\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFC6\x01\x12a\x013Wg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF`\x045\x81\x81\x11a\x013W6`#\x82\x01\x12\x15a\x013W\x80`\x04\x015\x91\x82\x11a\x013W`$\x81\x01\x90`$\x836\x92\x01\x01\x11a\x013W`$5\x91s\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x80\x84\x16\x80\x94\x03a\x013W`@Q\x82\x84\x827` \x81\x84\x81\x01`\x01\x81R\x03\x01\x90 T\x16a\x15\xBAW0\x83\x14a\x15\x90W\x7F\xF7\x80\x9E\xF0\xAEyO\xDAda;\xF1\xA5\xBF,\x8DF\x12\x0EKLo\xCC\x1D\xD9\xFBf\xA4V\xAC;A\x92`@Q\x82\x84\x827` \x81\x84\x81\x01`\x01\x81R\x03\x01\x90 \x81\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x82T\x16\x17\x90Ua\x15\x85`@Q\x93\x84\x93`@\x85R`@\x85\x01\x91a\x18\xF5V[\x90` \x83\x01R\x03\x90\xA1\0[`\x04`@Q\x7FF\x8E\xF7\x87\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R\xFD[`\x04`@Q\x7F\x0C|\xC9\xB9\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R\xFD[4a\x013W` g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFFa\r\x13\x82a\x16\x04a\x0C\xEE6a\x17\0V[\x81\x01`\x08\x81R\x03\x01\x90 \x90a\x17\x85V[`@\x81\x01\x90\x81\x10g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x17a\x047W`@RV[\x90`\x1F\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xE0\x91\x01\x16\x81\x01\x90\x81\x10g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x17a\x047W`@RV[g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x047W`\x1F\x01\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xE0\x16` \x01\x90V[\x92\x91\x92a\x16\xB7\x82a\x16qV[\x91a\x16\xC5`@Q\x93\x84a\x160V[\x82\x94\x81\x84R\x81\x83\x01\x11a\x013W\x82\x81` \x93\x84`\0\x96\x017\x01\x01RV[\x90\x80`\x1F\x83\x01\x12\x15a\x013W\x81` a\x16\xFD\x935\x91\x01a\x16\xABV[\x90V[\x90`@\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFC\x83\x01\x12a\x013Wg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF`\x045\x81\x81\x11a\x013W\x83a\x17K\x91`\x04\x01a\x16\xE2V[\x92`$5\x91\x82\x11a\x013Wa\x16\xFD\x91`\x04\x01a\x16\xE2V[`\0[\x83\x81\x10a\x17uWPP`\0\x91\x01RV[\x81\x81\x01Q\x83\x82\x01R` \x01a\x17eV[` \x90a\x17\x9F\x92\x82`@Q\x94\x83\x86\x80\x95Q\x93\x84\x92\x01a\x17bV[\x82\x01\x90\x81R\x03\x01\x90 \x90V[\x90`\x01\x82\x81\x1C\x92\x16\x80\x15a\x17\xF4W[` \x83\x10\x14a\x17\xC5WV[\x7FNH{q\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`\0R`\"`\x04R`$`\0\xFD[\x91`\x7F\x16\x91a\x17\xBAV[\x80T`\0\x93\x92a\x18\r\x82a\x17\xABV[\x91\x82\x82R` \x93`\x01\x91`\x01\x81\x16\x90\x81`\0\x14a\x18uWP`\x01\x14a\x184W[PPPPPV[\x90\x93\x94\x95P`\0\x92\x91\x92R\x83`\0 \x92\x84`\0\x94[\x83\x86\x10a\x18aWPPPP\x01\x01\x908\x80\x80\x80\x80a\x18-V[\x80T\x85\x87\x01\x83\x01R\x94\x01\x93\x85\x90\x82\x01a\x18IV[\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\0\x16\x86\x85\x01RPPP\x90\x15\x15`\x05\x1B\x01\x01\x91P8\x80\x80\x80\x80a\x18-V[\x90\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xE0`\x1F` \x93a\x18\xEE\x81Q\x80\x92\x81\x87R\x87\x80\x88\x01\x91\x01a\x17bV[\x01\x16\x01\x01\x90V[`\x1F\x82` \x94\x93\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xE0\x93\x81\x86R\x86\x86\x017`\0\x85\x82\x86\x01\x01R\x01\x16\x01\x01\x90V[a\x19b` s\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x92\x81`@Q\x93\x82\x85\x80\x94Q\x93\x84\x92\x01a\x17bV[\x81\x01`\x03\x81R\x03\x01\x90 T\x16\x80\x15a\x04rW\x90V[\x905\x90\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xE1\x816\x03\x01\x82\x12\x15a\x013W\x01\x805\x90g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11a\x013W` \x01\x91\x816\x03\x83\x13a\x013WV[Q\x90g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x16\x82\x03a\x013WV[\x80\x92\x91\x03\x91``\x83\x12a\x013W`@Qa\x19\xF6\x81a\x16\x14V[`@\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xE0\x82\x95\x84Q\x84R\x01\x12a\x013W` \x90a\x1AL`@\x80Q\x94a\x1A9\x86a\x16\x14V[a\x1AD\x85\x82\x01a\x19\xC8V[\x86R\x01a\x19\xC8V[\x82\x84\x01R\x01RV[Q\x90\x81\x15\x15\x82\x03a\x013WV[\x80Q\x82\x10\x15a\x1AuW` \x91`\x05\x1B\x01\x01\x90V[\x7FNH{q\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`\0R`2`\x04R`$`\0\xFD[a\x16\xFD`4`@Q\x80\x93\x7Fclients/\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0` \x83\x01Ra\x1A\xE8\x81Q\x80\x92` `(\x86\x01\x91\x01a\x17bV[\x81\x01\x7F/clientState\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`(\x82\x01R\x03`\x14\x81\x01\x84R\x01\x82a\x160V[\x91\x90`:a\x1BDa\x1C\x0C\x92a\x1B=g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x80\x92\x16a\x1C\x12V[\x94\x16a\x1C\x12V[\x92`@Q\x93\x84\x91` \x83\x01\x96\x7Fclients/\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x88Ra\x1B\x87\x81Q\x80\x92` `(\x88\x01\x91\x01a\x17bV[\x83\x01\x7F/consensusStates/\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`(\x82\x01Ra\x1B\xC3\x82Q\x80\x93` `9\x85\x01\x91\x01a\x17bV[\x01\x7F-\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`9\x82\x01Ra\x1B\xFD\x82Q\x80\x93` \x87\x85\x01\x91\x01a\x17bV[\x01\x03`\x1A\x81\x01\x84R\x01\x82a\x160V[Q\x90 \x90V[\x90`@Q\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF`\x80\x82\x01\x93`\xA0\x83\x01`@R`\0\x85R\x93[\x01\x92`\n\x90\x81\x81\x06`0\x01\x85S\x04\x92\x83\x15a\x1C\x85W\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x90a\x1CIV[\x92P`\x80\x83\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xE0\x92\x03\x01\x92\x01\x91\x82RV\xFE\xA2dipfsX\"\x12 \x13`\xEC\xC2\xF2\xDD/@\xAC\xFB\x92C\x99\x8C?\xD1\xE8\xFAXw\x8E&L\x1C\xF6\x946\xFB\x10\xEEf\x87dsolcC\0\x08\x17\x003";
    /// The deployed bytecode of the contract.
    #[cfg(feature = "providers")]
    pub static IBCCLIENT_DEPLOYED_BYTECODE: ::ethers::core::types::Bytes =
        ::ethers::core::types::Bytes::from_static(__DEPLOYED_BYTECODE);
    #[cfg(feature = "providers")]
    pub struct IBCClient<M>(::ethers::contract::Contract<M>);
    #[cfg(feature = "providers")]
    impl<M> ::core::clone::Clone for IBCClient<M> {
        fn clone(&self) -> Self {
            Self(::core::clone::Clone::clone(&self.0))
        }
    }
    #[cfg(feature = "providers")]
    impl<M> ::core::ops::Deref for IBCClient<M> {
        type Target = ::ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    #[cfg(feature = "providers")]
    impl<M> ::core::ops::DerefMut for IBCClient<M> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    #[cfg(feature = "providers")]
    impl<M> ::core::fmt::Debug for IBCClient<M> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple(::core::stringify!(IBCClient))
                .field(&self.address())
                .finish()
        }
    }
    #[cfg(feature = "providers")]
    impl<M: ::ethers::providers::Middleware> IBCClient<M> {
        /// Creates a new contract instance with the specified `ethers` client at
        /// `address`. The contract derefs to a `ethers::Contract` object.
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(::ethers::contract::Contract::new(
                address.into(),
                IBCCLIENT_ABI.clone(),
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
                IBCCLIENT_ABI.clone(),
                IBCCLIENT_BYTECODE.clone().into(),
                client,
            );
            let deployer = factory.deploy(constructor_args)?;
            let deployer = ::ethers::contract::ContractDeployer::new(deployer);
            Ok(deployer)
        }
        ///Calls the contract's `COMMITMENT_PREFIX` (0xa9550dac) function
        pub fn commitment_prefix(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ::std::string::String> {
            self.0
                .method_hash([169, 85, 13, 172], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `capabilities` (0x5717bcf5) function
        pub fn capabilities(
            &self,
            p0: ::std::string::String,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::Address> {
            self.0
                .method_hash([87, 23, 188, 245], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `channels` (0x5b3de260) function
        pub fn channels(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            (
                u8,
                u8,
                IbcCoreChannelV1CounterpartyData,
                ::std::string::String,
            ),
        > {
            self.0
                .method_hash([91, 61, 226, 96], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `clientImpls` (0xd1297b8d) function
        pub fn client_impls(
            &self,
            p0: ::std::string::String,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::Address> {
            self.0
                .method_hash([209, 41, 123, 141], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `clientRegistry` (0x990491a5) function
        pub fn client_registry(
            &self,
            p0: ::std::string::String,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::Address> {
            self.0
                .method_hash([153, 4, 145, 165], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `clientTypes` (0xc2380105) function
        pub fn client_types(
            &self,
            p0: ::std::string::String,
        ) -> ::ethers::contract::builders::ContractCall<M, ::std::string::String> {
            self.0
                .method_hash([194, 56, 1, 5], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `commitments` (0x839df945) function
        pub fn commitments(
            &self,
            p0: [u8; 32],
        ) -> ::ethers::contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([131, 157, 249, 69], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `connections` (0x31973f00) function
        pub fn connections(
            &self,
            p0: ::std::string::String,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            (
                ::std::string::String,
                u8,
                IbcCoreConnectionV1CounterpartyData,
                u64,
            ),
        > {
            self.0
                .method_hash([49, 151, 63, 0], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `createClient` (0xd5a24481) function
        pub fn create_client(
            &self,
            msg: MsgCreateClient,
        ) -> ::ethers::contract::builders::ContractCall<M, ::std::string::String> {
            self.0
                .method_hash([213, 162, 68, 129], (msg,))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getClient` (0x7eb78932) function
        pub fn get_client(
            &self,
            client_id: ::std::string::String,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::Address> {
            self.0
                .method_hash([126, 183, 137, 50], client_id)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `nextChannelSequence` (0x7926b8a9) function
        pub fn next_channel_sequence(&self) -> ::ethers::contract::builders::ContractCall<M, u64> {
            self.0
                .method_hash([121, 38, 184, 169], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `nextClientSequence` (0xe1b17b43) function
        pub fn next_client_sequence(&self) -> ::ethers::contract::builders::ContractCall<M, u64> {
            self.0
                .method_hash([225, 177, 123, 67], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `nextConnectionSequence` (0xa049e677) function
        pub fn next_connection_sequence(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, u64> {
            self.0
                .method_hash([160, 73, 230, 119], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `nextSequenceAcks` (0x1390d28d) function
        pub fn next_sequence_acks(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
        ) -> ::ethers::contract::builders::ContractCall<M, u64> {
            self.0
                .method_hash([19, 144, 210, 141], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `nextSequenceRecvs` (0xc930b1b0) function
        pub fn next_sequence_recvs(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
        ) -> ::ethers::contract::builders::ContractCall<M, u64> {
            self.0
                .method_hash([201, 48, 177, 176], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `nextSequenceSends` (0x821cb5d0) function
        pub fn next_sequence_sends(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
        ) -> ::ethers::contract::builders::ContractCall<M, u64> {
            self.0
                .method_hash([130, 28, 181, 208], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `packetReceipts` (0x26078437) function
        pub fn packet_receipts(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
            p2: u64,
        ) -> ::ethers::contract::builders::ContractCall<M, u8> {
            self.0
                .method_hash([38, 7, 132, 55], (p0, p1, p2))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `registerClient` (0x18c19870) function
        pub fn register_client(
            &self,
            client_type: ::std::string::String,
            client: ::ethers::core::types::Address,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([24, 193, 152, 112], (client_type, client))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `updateClient` (0xda6cea55) function
        pub fn update_client(
            &self,
            msg: MsgUpdateClient,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([218, 108, 234, 85], (msg,))
                .expect("method not found (this should never happen)")
        }
        ///Gets the contract's `ClientCreated` event
        pub fn client_created_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<::std::sync::Arc<M>, M, ClientCreatedFilter>
        {
            self.0.event()
        }
        ///Gets the contract's `ClientRegistered` event
        pub fn client_registered_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<::std::sync::Arc<M>, M, ClientRegisteredFilter>
        {
            self.0.event()
        }
        ///Gets the contract's `ClientUpdated` event
        pub fn client_updated_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<::std::sync::Arc<M>, M, ClientUpdatedFilter>
        {
            self.0.event()
        }
        /// Returns an `Event` builder for all the events of this contract.
        pub fn events(
            &self,
        ) -> ::ethers::contract::builders::Event<::std::sync::Arc<M>, M, IBCClientEvents> {
            self.0
                .event_with_filter(::core::default::Default::default())
        }
    }
    #[cfg(feature = "providers")]
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>> for IBCClient<M> {
        fn from(contract: ::ethers::contract::Contract<M>) -> Self {
            Self::new(contract.address(), contract.client())
        }
    }
    ///Custom Error type `ErrClientMustNotBeSelf` with signature `ErrClientMustNotBeSelf()` and selector `0x468ef787`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[etherror(name = "ErrClientMustNotBeSelf", abi = "ErrClientMustNotBeSelf()")]
    pub struct ErrClientMustNotBeSelf;
    ///Custom Error type `ErrClientNotFound` with signature `ErrClientNotFound()` and selector `0xb6c71f7d`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[etherror(name = "ErrClientNotFound", abi = "ErrClientNotFound()")]
    pub struct ErrClientNotFound;
    ///Custom Error type `ErrClientTypeAlreadyExists` with signature `ErrClientTypeAlreadyExists()` and selector `0x0c7cc9b9`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[etherror(
        name = "ErrClientTypeAlreadyExists",
        abi = "ErrClientTypeAlreadyExists()"
    )]
    pub struct ErrClientTypeAlreadyExists;
    ///Custom Error type `ErrClientTypeNotFound` with signature `ErrClientTypeNotFound()` and selector `0xaa478af9`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[etherror(name = "ErrClientTypeNotFound", abi = "ErrClientTypeNotFound()")]
    pub struct ErrClientTypeNotFound;
    ///Custom Error type `ErrFailedToCreateClient` with signature `ErrFailedToCreateClient()` and selector `0x8b9f95b2`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[etherror(name = "ErrFailedToCreateClient", abi = "ErrFailedToCreateClient()")]
    pub struct ErrFailedToCreateClient;
    ///Custom Error type `ErrFailedToUpdateClient` with signature `ErrFailedToUpdateClient()` and selector `0xf1691139`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[etherror(name = "ErrFailedToUpdateClient", abi = "ErrFailedToUpdateClient()")]
    pub struct ErrFailedToUpdateClient;
    ///Container type for all of the contract's custom errors
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum IBCClientErrors {
        ErrClientMustNotBeSelf(ErrClientMustNotBeSelf),
        ErrClientNotFound(ErrClientNotFound),
        ErrClientTypeAlreadyExists(ErrClientTypeAlreadyExists),
        ErrClientTypeNotFound(ErrClientTypeNotFound),
        ErrFailedToCreateClient(ErrFailedToCreateClient),
        ErrFailedToUpdateClient(ErrFailedToUpdateClient),
        /// The standard solidity revert string, with selector
        /// Error(string) -- 0x08c379a0
        RevertString(::std::string::String),
    }
    impl ::ethers::core::abi::AbiDecode for IBCClientErrors {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded) =
                <::std::string::String as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::RevertString(decoded));
            }
            if let Ok(decoded) =
                <ErrClientMustNotBeSelf as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::ErrClientMustNotBeSelf(decoded));
            }
            if let Ok(decoded) = <ErrClientNotFound as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::ErrClientNotFound(decoded));
            }
            if let Ok(decoded) =
                <ErrClientTypeAlreadyExists as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::ErrClientTypeAlreadyExists(decoded));
            }
            if let Ok(decoded) =
                <ErrClientTypeNotFound as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::ErrClientTypeNotFound(decoded));
            }
            if let Ok(decoded) =
                <ErrFailedToCreateClient as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::ErrFailedToCreateClient(decoded));
            }
            if let Ok(decoded) =
                <ErrFailedToUpdateClient as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::ErrFailedToUpdateClient(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for IBCClientErrors {
        fn encode(self) -> ::std::vec::Vec<u8> {
            match self {
                Self::ErrClientMustNotBeSelf(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ErrClientNotFound(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::ErrClientTypeAlreadyExists(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ErrClientTypeNotFound(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ErrFailedToCreateClient(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ErrFailedToUpdateClient(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RevertString(s) => ::ethers::core::abi::AbiEncode::encode(s),
            }
        }
    }
    impl ::ethers::contract::ContractRevert for IBCClientErrors {
        fn valid_selector(selector: [u8; 4]) -> bool {
            match selector {
                [0x08, 0xc3, 0x79, 0xa0] => true,
                _ if selector
                    == <ErrClientMustNotBeSelf as ::ethers::contract::EthError>::selector() =>
                {
                    true
                }
                _ if selector
                    == <ErrClientNotFound as ::ethers::contract::EthError>::selector() =>
                {
                    true
                }
                _ if selector
                    == <ErrClientTypeAlreadyExists as ::ethers::contract::EthError>::selector() =>
                {
                    true
                }
                _ if selector
                    == <ErrClientTypeNotFound as ::ethers::contract::EthError>::selector() =>
                {
                    true
                }
                _ if selector
                    == <ErrFailedToCreateClient as ::ethers::contract::EthError>::selector() =>
                {
                    true
                }
                _ if selector
                    == <ErrFailedToUpdateClient as ::ethers::contract::EthError>::selector() =>
                {
                    true
                }
                _ => false,
            }
        }
    }
    impl ::core::fmt::Display for IBCClientErrors {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::ErrClientMustNotBeSelf(element) => ::core::fmt::Display::fmt(element, f),
                Self::ErrClientNotFound(element) => ::core::fmt::Display::fmt(element, f),
                Self::ErrClientTypeAlreadyExists(element) => ::core::fmt::Display::fmt(element, f),
                Self::ErrClientTypeNotFound(element) => ::core::fmt::Display::fmt(element, f),
                Self::ErrFailedToCreateClient(element) => ::core::fmt::Display::fmt(element, f),
                Self::ErrFailedToUpdateClient(element) => ::core::fmt::Display::fmt(element, f),
                Self::RevertString(s) => ::core::fmt::Display::fmt(s, f),
            }
        }
    }
    impl ::core::convert::From<::std::string::String> for IBCClientErrors {
        fn from(value: String) -> Self {
            Self::RevertString(value)
        }
    }
    impl ::core::convert::From<ErrClientMustNotBeSelf> for IBCClientErrors {
        fn from(value: ErrClientMustNotBeSelf) -> Self {
            Self::ErrClientMustNotBeSelf(value)
        }
    }
    impl ::core::convert::From<ErrClientNotFound> for IBCClientErrors {
        fn from(value: ErrClientNotFound) -> Self {
            Self::ErrClientNotFound(value)
        }
    }
    impl ::core::convert::From<ErrClientTypeAlreadyExists> for IBCClientErrors {
        fn from(value: ErrClientTypeAlreadyExists) -> Self {
            Self::ErrClientTypeAlreadyExists(value)
        }
    }
    impl ::core::convert::From<ErrClientTypeNotFound> for IBCClientErrors {
        fn from(value: ErrClientTypeNotFound) -> Self {
            Self::ErrClientTypeNotFound(value)
        }
    }
    impl ::core::convert::From<ErrFailedToCreateClient> for IBCClientErrors {
        fn from(value: ErrFailedToCreateClient) -> Self {
            Self::ErrFailedToCreateClient(value)
        }
    }
    impl ::core::convert::From<ErrFailedToUpdateClient> for IBCClientErrors {
        fn from(value: ErrFailedToUpdateClient) -> Self {
            Self::ErrFailedToUpdateClient(value)
        }
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[ethevent(name = "ClientCreated", abi = "ClientCreated(string)")]
    pub struct ClientCreatedFilter(pub ::std::string::String);
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[ethevent(name = "ClientRegistered", abi = "ClientRegistered(string,address)")]
    pub struct ClientRegisteredFilter(
        pub ::std::string::String,
        pub ::ethers::core::types::Address,
    );
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[ethevent(name = "ClientUpdated", abi = "ClientUpdated(string,(uint64,uint64))")]
    pub struct ClientUpdatedFilter(pub ::std::string::String, pub IbcCoreClientV1HeightData);
    ///Container type for all of the contract's events
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum IBCClientEvents {
        ClientCreatedFilter(ClientCreatedFilter),
        ClientRegisteredFilter(ClientRegisteredFilter),
        ClientUpdatedFilter(ClientUpdatedFilter),
    }
    impl ::ethers::contract::EthLogDecode for IBCClientEvents {
        fn decode_log(
            log: &::ethers::core::abi::RawLog,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::Error> {
            if let Ok(decoded) = ClientCreatedFilter::decode_log(log) {
                return Ok(IBCClientEvents::ClientCreatedFilter(decoded));
            }
            if let Ok(decoded) = ClientRegisteredFilter::decode_log(log) {
                return Ok(IBCClientEvents::ClientRegisteredFilter(decoded));
            }
            if let Ok(decoded) = ClientUpdatedFilter::decode_log(log) {
                return Ok(IBCClientEvents::ClientUpdatedFilter(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData)
        }
    }
    impl ::core::fmt::Display for IBCClientEvents {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::ClientCreatedFilter(element) => ::core::fmt::Display::fmt(element, f),
                Self::ClientRegisteredFilter(element) => ::core::fmt::Display::fmt(element, f),
                Self::ClientUpdatedFilter(element) => ::core::fmt::Display::fmt(element, f),
            }
        }
    }
    impl ::core::convert::From<ClientCreatedFilter> for IBCClientEvents {
        fn from(value: ClientCreatedFilter) -> Self {
            Self::ClientCreatedFilter(value)
        }
    }
    impl ::core::convert::From<ClientRegisteredFilter> for IBCClientEvents {
        fn from(value: ClientRegisteredFilter) -> Self {
            Self::ClientRegisteredFilter(value)
        }
    }
    impl ::core::convert::From<ClientUpdatedFilter> for IBCClientEvents {
        fn from(value: ClientUpdatedFilter) -> Self {
            Self::ClientUpdatedFilter(value)
        }
    }
    ///Container type for all input parameters for the `COMMITMENT_PREFIX` function with signature `COMMITMENT_PREFIX()` and selector `0xa9550dac`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[ethcall(name = "COMMITMENT_PREFIX", abi = "COMMITMENT_PREFIX()")]
    pub struct CommitmentPrefixCall;
    ///Container type for all input parameters for the `capabilities` function with signature `capabilities(string)` and selector `0x5717bcf5`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[ethcall(name = "capabilities", abi = "capabilities(string)")]
    pub struct CapabilitiesCall(pub ::std::string::String);
    ///Container type for all input parameters for the `channels` function with signature `channels(string,string)` and selector `0x5b3de260`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[ethcall(name = "channels", abi = "channels(string,string)")]
    pub struct ChannelsCall(pub ::std::string::String, pub ::std::string::String);
    ///Container type for all input parameters for the `clientImpls` function with signature `clientImpls(string)` and selector `0xd1297b8d`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[ethcall(name = "clientImpls", abi = "clientImpls(string)")]
    pub struct ClientImplsCall(pub ::std::string::String);
    ///Container type for all input parameters for the `clientRegistry` function with signature `clientRegistry(string)` and selector `0x990491a5`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[ethcall(name = "clientRegistry", abi = "clientRegistry(string)")]
    pub struct ClientRegistryCall(pub ::std::string::String);
    ///Container type for all input parameters for the `clientTypes` function with signature `clientTypes(string)` and selector `0xc2380105`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[ethcall(name = "clientTypes", abi = "clientTypes(string)")]
    pub struct ClientTypesCall(pub ::std::string::String);
    ///Container type for all input parameters for the `commitments` function with signature `commitments(bytes32)` and selector `0x839df945`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[ethcall(name = "commitments", abi = "commitments(bytes32)")]
    pub struct CommitmentsCall(pub [u8; 32]);
    ///Container type for all input parameters for the `connections` function with signature `connections(string)` and selector `0x31973f00`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[ethcall(name = "connections", abi = "connections(string)")]
    pub struct ConnectionsCall(pub ::std::string::String);
    ///Container type for all input parameters for the `createClient` function with signature `createClient((string,bytes,bytes))` and selector `0xd5a24481`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[ethcall(name = "createClient", abi = "createClient((string,bytes,bytes))")]
    pub struct CreateClientCall {
        pub msg: MsgCreateClient,
    }
    ///Container type for all input parameters for the `getClient` function with signature `getClient(string)` and selector `0x7eb78932`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[ethcall(name = "getClient", abi = "getClient(string)")]
    pub struct GetClientCall {
        pub client_id: ::std::string::String,
    }
    ///Container type for all input parameters for the `nextChannelSequence` function with signature `nextChannelSequence()` and selector `0x7926b8a9`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[ethcall(name = "nextChannelSequence", abi = "nextChannelSequence()")]
    pub struct NextChannelSequenceCall;
    ///Container type for all input parameters for the `nextClientSequence` function with signature `nextClientSequence()` and selector `0xe1b17b43`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[ethcall(name = "nextClientSequence", abi = "nextClientSequence()")]
    pub struct NextClientSequenceCall;
    ///Container type for all input parameters for the `nextConnectionSequence` function with signature `nextConnectionSequence()` and selector `0xa049e677`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[ethcall(name = "nextConnectionSequence", abi = "nextConnectionSequence()")]
    pub struct NextConnectionSequenceCall;
    ///Container type for all input parameters for the `nextSequenceAcks` function with signature `nextSequenceAcks(string,string)` and selector `0x1390d28d`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[ethcall(name = "nextSequenceAcks", abi = "nextSequenceAcks(string,string)")]
    pub struct NextSequenceAcksCall(pub ::std::string::String, pub ::std::string::String);
    ///Container type for all input parameters for the `nextSequenceRecvs` function with signature `nextSequenceRecvs(string,string)` and selector `0xc930b1b0`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[ethcall(name = "nextSequenceRecvs", abi = "nextSequenceRecvs(string,string)")]
    pub struct NextSequenceRecvsCall(pub ::std::string::String, pub ::std::string::String);
    ///Container type for all input parameters for the `nextSequenceSends` function with signature `nextSequenceSends(string,string)` and selector `0x821cb5d0`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[ethcall(name = "nextSequenceSends", abi = "nextSequenceSends(string,string)")]
    pub struct NextSequenceSendsCall(pub ::std::string::String, pub ::std::string::String);
    ///Container type for all input parameters for the `packetReceipts` function with signature `packetReceipts(string,string,uint64)` and selector `0x26078437`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[ethcall(name = "packetReceipts", abi = "packetReceipts(string,string,uint64)")]
    pub struct PacketReceiptsCall(
        pub ::std::string::String,
        pub ::std::string::String,
        pub u64,
    );
    ///Container type for all input parameters for the `registerClient` function with signature `registerClient(string,address)` and selector `0x18c19870`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[ethcall(name = "registerClient", abi = "registerClient(string,address)")]
    pub struct RegisterClientCall {
        pub client_type: ::std::string::String,
        pub client: ::ethers::core::types::Address,
    }
    ///Container type for all input parameters for the `updateClient` function with signature `updateClient((string,bytes))` and selector `0xda6cea55`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[ethcall(name = "updateClient", abi = "updateClient((string,bytes))")]
    pub struct UpdateClientCall {
        pub msg: MsgUpdateClient,
    }
    ///Container type for all of the contract's call
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum IBCClientCalls {
        CommitmentPrefix(CommitmentPrefixCall),
        Capabilities(CapabilitiesCall),
        Channels(ChannelsCall),
        ClientImpls(ClientImplsCall),
        ClientRegistry(ClientRegistryCall),
        ClientTypes(ClientTypesCall),
        Commitments(CommitmentsCall),
        Connections(ConnectionsCall),
        CreateClient(CreateClientCall),
        GetClient(GetClientCall),
        NextChannelSequence(NextChannelSequenceCall),
        NextClientSequence(NextClientSequenceCall),
        NextConnectionSequence(NextConnectionSequenceCall),
        NextSequenceAcks(NextSequenceAcksCall),
        NextSequenceRecvs(NextSequenceRecvsCall),
        NextSequenceSends(NextSequenceSendsCall),
        PacketReceipts(PacketReceiptsCall),
        RegisterClient(RegisterClientCall),
        UpdateClient(UpdateClientCall),
    }
    impl ::ethers::core::abi::AbiDecode for IBCClientCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded) =
                <CommitmentPrefixCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::CommitmentPrefix(decoded));
            }
            if let Ok(decoded) = <CapabilitiesCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::Capabilities(decoded));
            }
            if let Ok(decoded) = <ChannelsCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Channels(decoded));
            }
            if let Ok(decoded) = <ClientImplsCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ClientImpls(decoded));
            }
            if let Ok(decoded) =
                <ClientRegistryCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::ClientRegistry(decoded));
            }
            if let Ok(decoded) = <ClientTypesCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ClientTypes(decoded));
            }
            if let Ok(decoded) = <CommitmentsCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Commitments(decoded));
            }
            if let Ok(decoded) = <ConnectionsCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Connections(decoded));
            }
            if let Ok(decoded) = <CreateClientCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::CreateClient(decoded));
            }
            if let Ok(decoded) = <GetClientCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::GetClient(decoded));
            }
            if let Ok(decoded) =
                <NextChannelSequenceCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::NextChannelSequence(decoded));
            }
            if let Ok(decoded) =
                <NextClientSequenceCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::NextClientSequence(decoded));
            }
            if let Ok(decoded) =
                <NextConnectionSequenceCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::NextConnectionSequence(decoded));
            }
            if let Ok(decoded) =
                <NextSequenceAcksCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::NextSequenceAcks(decoded));
            }
            if let Ok(decoded) =
                <NextSequenceRecvsCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::NextSequenceRecvs(decoded));
            }
            if let Ok(decoded) =
                <NextSequenceSendsCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::NextSequenceSends(decoded));
            }
            if let Ok(decoded) =
                <PacketReceiptsCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::PacketReceipts(decoded));
            }
            if let Ok(decoded) =
                <RegisterClientCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::RegisterClient(decoded));
            }
            if let Ok(decoded) = <UpdateClientCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::UpdateClient(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for IBCClientCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                Self::CommitmentPrefix(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::Capabilities(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::Channels(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::ClientImpls(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::ClientRegistry(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::ClientTypes(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::Commitments(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::Connections(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::CreateClient(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::GetClient(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::NextChannelSequence(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NextClientSequence(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NextConnectionSequence(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NextSequenceAcks(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::NextSequenceRecvs(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::NextSequenceSends(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::PacketReceipts(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::RegisterClient(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::UpdateClient(element) => ::ethers::core::abi::AbiEncode::encode(element),
            }
        }
    }
    impl ::core::fmt::Display for IBCClientCalls {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::CommitmentPrefix(element) => ::core::fmt::Display::fmt(element, f),
                Self::Capabilities(element) => ::core::fmt::Display::fmt(element, f),
                Self::Channels(element) => ::core::fmt::Display::fmt(element, f),
                Self::ClientImpls(element) => ::core::fmt::Display::fmt(element, f),
                Self::ClientRegistry(element) => ::core::fmt::Display::fmt(element, f),
                Self::ClientTypes(element) => ::core::fmt::Display::fmt(element, f),
                Self::Commitments(element) => ::core::fmt::Display::fmt(element, f),
                Self::Connections(element) => ::core::fmt::Display::fmt(element, f),
                Self::CreateClient(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetClient(element) => ::core::fmt::Display::fmt(element, f),
                Self::NextChannelSequence(element) => ::core::fmt::Display::fmt(element, f),
                Self::NextClientSequence(element) => ::core::fmt::Display::fmt(element, f),
                Self::NextConnectionSequence(element) => ::core::fmt::Display::fmt(element, f),
                Self::NextSequenceAcks(element) => ::core::fmt::Display::fmt(element, f),
                Self::NextSequenceRecvs(element) => ::core::fmt::Display::fmt(element, f),
                Self::NextSequenceSends(element) => ::core::fmt::Display::fmt(element, f),
                Self::PacketReceipts(element) => ::core::fmt::Display::fmt(element, f),
                Self::RegisterClient(element) => ::core::fmt::Display::fmt(element, f),
                Self::UpdateClient(element) => ::core::fmt::Display::fmt(element, f),
            }
        }
    }
    impl ::core::convert::From<CommitmentPrefixCall> for IBCClientCalls {
        fn from(value: CommitmentPrefixCall) -> Self {
            Self::CommitmentPrefix(value)
        }
    }
    impl ::core::convert::From<CapabilitiesCall> for IBCClientCalls {
        fn from(value: CapabilitiesCall) -> Self {
            Self::Capabilities(value)
        }
    }
    impl ::core::convert::From<ChannelsCall> for IBCClientCalls {
        fn from(value: ChannelsCall) -> Self {
            Self::Channels(value)
        }
    }
    impl ::core::convert::From<ClientImplsCall> for IBCClientCalls {
        fn from(value: ClientImplsCall) -> Self {
            Self::ClientImpls(value)
        }
    }
    impl ::core::convert::From<ClientRegistryCall> for IBCClientCalls {
        fn from(value: ClientRegistryCall) -> Self {
            Self::ClientRegistry(value)
        }
    }
    impl ::core::convert::From<ClientTypesCall> for IBCClientCalls {
        fn from(value: ClientTypesCall) -> Self {
            Self::ClientTypes(value)
        }
    }
    impl ::core::convert::From<CommitmentsCall> for IBCClientCalls {
        fn from(value: CommitmentsCall) -> Self {
            Self::Commitments(value)
        }
    }
    impl ::core::convert::From<ConnectionsCall> for IBCClientCalls {
        fn from(value: ConnectionsCall) -> Self {
            Self::Connections(value)
        }
    }
    impl ::core::convert::From<CreateClientCall> for IBCClientCalls {
        fn from(value: CreateClientCall) -> Self {
            Self::CreateClient(value)
        }
    }
    impl ::core::convert::From<GetClientCall> for IBCClientCalls {
        fn from(value: GetClientCall) -> Self {
            Self::GetClient(value)
        }
    }
    impl ::core::convert::From<NextChannelSequenceCall> for IBCClientCalls {
        fn from(value: NextChannelSequenceCall) -> Self {
            Self::NextChannelSequence(value)
        }
    }
    impl ::core::convert::From<NextClientSequenceCall> for IBCClientCalls {
        fn from(value: NextClientSequenceCall) -> Self {
            Self::NextClientSequence(value)
        }
    }
    impl ::core::convert::From<NextConnectionSequenceCall> for IBCClientCalls {
        fn from(value: NextConnectionSequenceCall) -> Self {
            Self::NextConnectionSequence(value)
        }
    }
    impl ::core::convert::From<NextSequenceAcksCall> for IBCClientCalls {
        fn from(value: NextSequenceAcksCall) -> Self {
            Self::NextSequenceAcks(value)
        }
    }
    impl ::core::convert::From<NextSequenceRecvsCall> for IBCClientCalls {
        fn from(value: NextSequenceRecvsCall) -> Self {
            Self::NextSequenceRecvs(value)
        }
    }
    impl ::core::convert::From<NextSequenceSendsCall> for IBCClientCalls {
        fn from(value: NextSequenceSendsCall) -> Self {
            Self::NextSequenceSends(value)
        }
    }
    impl ::core::convert::From<PacketReceiptsCall> for IBCClientCalls {
        fn from(value: PacketReceiptsCall) -> Self {
            Self::PacketReceipts(value)
        }
    }
    impl ::core::convert::From<RegisterClientCall> for IBCClientCalls {
        fn from(value: RegisterClientCall) -> Self {
            Self::RegisterClient(value)
        }
    }
    impl ::core::convert::From<UpdateClientCall> for IBCClientCalls {
        fn from(value: UpdateClientCall) -> Self {
            Self::UpdateClient(value)
        }
    }
    ///Container type for all return fields from the `COMMITMENT_PREFIX` function with signature `COMMITMENT_PREFIX()` and selector `0xa9550dac`
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
    pub struct CommitmentPrefixReturn(pub ::std::string::String);
    ///Container type for all return fields from the `capabilities` function with signature `capabilities(string)` and selector `0x5717bcf5`
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
    pub struct CapabilitiesReturn(pub ::ethers::core::types::Address);
    ///Container type for all return fields from the `channels` function with signature `channels(string,string)` and selector `0x5b3de260`
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
    pub struct ChannelsReturn {
        pub state: u8,
        pub ordering: u8,
        pub counterparty: IbcCoreChannelV1CounterpartyData,
        pub version: ::std::string::String,
    }
    ///Container type for all return fields from the `clientImpls` function with signature `clientImpls(string)` and selector `0xd1297b8d`
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
    pub struct ClientImplsReturn(pub ::ethers::core::types::Address);
    ///Container type for all return fields from the `clientRegistry` function with signature `clientRegistry(string)` and selector `0x990491a5`
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
    pub struct ClientRegistryReturn(pub ::ethers::core::types::Address);
    ///Container type for all return fields from the `clientTypes` function with signature `clientTypes(string)` and selector `0xc2380105`
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
    pub struct ClientTypesReturn(pub ::std::string::String);
    ///Container type for all return fields from the `commitments` function with signature `commitments(bytes32)` and selector `0x839df945`
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
    pub struct CommitmentsReturn(pub [u8; 32]);
    ///Container type for all return fields from the `connections` function with signature `connections(string)` and selector `0x31973f00`
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
    pub struct ConnectionsReturn {
        pub client_id: ::std::string::String,
        pub state: u8,
        pub counterparty: IbcCoreConnectionV1CounterpartyData,
        pub delay_period: u64,
    }
    ///Container type for all return fields from the `createClient` function with signature `createClient((string,bytes,bytes))` and selector `0xd5a24481`
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
    pub struct CreateClientReturn {
        pub client_id: ::std::string::String,
    }
    ///Container type for all return fields from the `getClient` function with signature `getClient(string)` and selector `0x7eb78932`
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
    pub struct GetClientReturn(pub ::ethers::core::types::Address);
    ///Container type for all return fields from the `nextChannelSequence` function with signature `nextChannelSequence()` and selector `0x7926b8a9`
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
    pub struct NextChannelSequenceReturn(pub u64);
    ///Container type for all return fields from the `nextClientSequence` function with signature `nextClientSequence()` and selector `0xe1b17b43`
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
    pub struct NextClientSequenceReturn(pub u64);
    ///Container type for all return fields from the `nextConnectionSequence` function with signature `nextConnectionSequence()` and selector `0xa049e677`
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
    pub struct NextConnectionSequenceReturn(pub u64);
    ///Container type for all return fields from the `nextSequenceAcks` function with signature `nextSequenceAcks(string,string)` and selector `0x1390d28d`
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
    pub struct NextSequenceAcksReturn(pub u64);
    ///Container type for all return fields from the `nextSequenceRecvs` function with signature `nextSequenceRecvs(string,string)` and selector `0xc930b1b0`
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
    pub struct NextSequenceRecvsReturn(pub u64);
    ///Container type for all return fields from the `nextSequenceSends` function with signature `nextSequenceSends(string,string)` and selector `0x821cb5d0`
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
    pub struct NextSequenceSendsReturn(pub u64);
    ///Container type for all return fields from the `packetReceipts` function with signature `packetReceipts(string,string,uint64)` and selector `0x26078437`
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
    pub struct PacketReceiptsReturn(pub u8);
}
