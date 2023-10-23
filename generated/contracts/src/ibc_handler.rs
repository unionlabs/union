pub use ibc_handler::*;
/// This module was auto-generated with ethers-rs Abigen.
/// More information at: <https://github.com/gakonst/ethers-rs>
#[allow(
    clippy::enum_variant_names,
    clippy::too_many_arguments,
    clippy::upper_case_acronyms,
    clippy::type_complexity,
    dead_code,
    non_camel_case_types,
)]
pub mod ibc_handler {
    pub use super::super::shared_types::*;
    #[allow(deprecated)]
    fn __abi() -> ::ethers::core::abi::Abi {
        ::ethers::core::abi::ethabi::Contract {
            constructor: ::core::option::Option::None,
            functions: ::core::convert::From::from([
                (
                    ::std::borrow::ToOwned::to_owned("acknowledgePacket"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("acknowledgePacket"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("msg_"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                    ::ethers::core::abi::ethabi::ParamType::String,
                                                    ::ethers::core::abi::ethabi::ParamType::String,
                                                    ::ethers::core::abi::ethabi::ParamType::String,
                                                    ::ethers::core::abi::ethabi::ParamType::String,
                                                    ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                        ::std::vec![
                                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                        ],
                                                    ),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
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
                                            "struct IBCMsgs.MsgPacketAcknowledgement",
                                        ),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("bindPort"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("bindPort"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("portId"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("moduleAddress"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("capabilities"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("capabilities"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bytes,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("channelCapabilityPath"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "channelCapabilityPath",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("portId"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("channelId"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bytes,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::Pure,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("channelCloseConfirm"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "channelCloseConfirm",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("msg_"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
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
                                            "struct IBCMsgs.MsgChannelCloseConfirm",
                                        ),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("channelCloseInit"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("channelCloseInit"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("msg_"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned(
                                            "struct IBCMsgs.MsgChannelCloseInit",
                                        ),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("channelOpenAck"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("channelOpenAck"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("msg_"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
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
                                            "struct IBCMsgs.MsgChannelOpenAck",
                                        ),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("channelOpenConfirm"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("channelOpenConfirm"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("msg_"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
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
                                            "struct IBCMsgs.MsgChannelOpenConfirm",
                                        ),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("channelOpenInit"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("channelOpenInit"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("msg_"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                        ::std::vec![
                                                            ::ethers::core::abi::ethabi::ParamType::String,
                                                            ::ethers::core::abi::ethabi::ParamType::String,
                                                        ],
                                                    ),
                                                    ::ethers::core::abi::ethabi::ParamType::Array(
                                                        ::std::boxed::Box::new(
                                                            ::ethers::core::abi::ethabi::ParamType::String,
                                                        ),
                                                    ),
                                                    ::ethers::core::abi::ethabi::ParamType::String,
                                                ],
                                            ),
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned(
                                            "struct IBCMsgs.MsgChannelOpenInit",
                                        ),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("channelId"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("channelOpenTry"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("channelOpenTry"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("msg_"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                        ::std::vec![
                                                            ::ethers::core::abi::ethabi::ParamType::String,
                                                            ::ethers::core::abi::ethabi::ParamType::String,
                                                        ],
                                                    ),
                                                    ::ethers::core::abi::ethabi::ParamType::Array(
                                                        ::std::boxed::Box::new(
                                                            ::ethers::core::abi::ethabi::ParamType::String,
                                                        ),
                                                    ),
                                                    ::ethers::core::abi::ethabi::ParamType::String,
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
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
                                            "struct IBCMsgs.MsgChannelOpenTry",
                                        ),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("channelId"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("channels"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
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
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                        ],
                                    ),
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
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("clientImpls"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("clientImpls"),
                            inputs: ::std::vec![
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
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("clientRegistry"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("clientRegistry"),
                            inputs: ::std::vec![
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
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("clientTypes"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("clientTypes"),
                            inputs: ::std::vec![
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
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("commitments"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("commitments"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes32"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes32"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("connectionOpenAck"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("connectionOpenAck"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("msg_"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::String,
                                                    ::ethers::core::abi::ethabi::ParamType::Array(
                                                        ::std::boxed::Box::new(
                                                            ::ethers::core::abi::ethabi::ParamType::String,
                                                        ),
                                                    ),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
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
                                            "struct IBCMsgs.MsgConnectionOpenAck",
                                        ),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("connectionOpenConfirm"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "connectionOpenConfirm",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("msg_"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
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
                                            "struct IBCMsgs.MsgConnectionOpenConfirm",
                                        ),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("connectionOpenInit"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("connectionOpenInit"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("msg_"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::String,
                                                    ::ethers::core::abi::ethabi::ParamType::String,
                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                        ::std::vec![::ethers::core::abi::ethabi::ParamType::Bytes],
                                                    ),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned(
                                            "struct IBCMsgs.MsgConnectionOpenInit",
                                        ),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("connectionId"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("connectionOpenTry"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("connectionOpenTry"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("msg_"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::String,
                                                    ::ethers::core::abi::ethabi::ParamType::String,
                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                        ::std::vec![::ethers::core::abi::ethabi::ParamType::Bytes],
                                                    ),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                            ::ethers::core::abi::ethabi::ParamType::Array(
                                                ::std::boxed::Box::new(
                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                        ::std::vec![
                                                            ::ethers::core::abi::ethabi::ParamType::String,
                                                            ::ethers::core::abi::ethabi::ParamType::Array(
                                                                ::std::boxed::Box::new(
                                                                    ::ethers::core::abi::ethabi::ParamType::String,
                                                                ),
                                                            ),
                                                        ],
                                                    ),
                                                ),
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
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
                                            "struct IBCMsgs.MsgConnectionOpenTry",
                                        ),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("connectionId"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("connections"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("connections"),
                            inputs: ::std::vec![
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
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![::ethers::core::abi::ethabi::ParamType::Bytes],
                                            ),
                                        ],
                                    ),
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
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("createClient"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("createClient"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("msg_"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned(
                                            "struct IBCMsgs.MsgCreateClient",
                                        ),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("clientId"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("expectedTimePerBlock"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "expectedTimePerBlock",
                            ),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getChannel"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("getChannel"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("portId"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("channelId"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::String,
                                                    ::ethers::core::abi::ethabi::ParamType::String,
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Array(
                                                ::std::boxed::Box::new(
                                                    ::ethers::core::abi::ethabi::ParamType::String,
                                                ),
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned(
                                            "struct IbcCoreChannelV1Channel.Data",
                                        ),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bool"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getClientState"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("getClientState"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("clientId"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bytes,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bool"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getConnection"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("getConnection"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("connectionId"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                            ::ethers::core::abi::ethabi::ParamType::Array(
                                                ::std::boxed::Box::new(
                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                        ::std::vec![
                                                            ::ethers::core::abi::ethabi::ParamType::String,
                                                            ::ethers::core::abi::ethabi::ParamType::Array(
                                                                ::std::boxed::Box::new(
                                                                    ::ethers::core::abi::ethabi::ParamType::String,
                                                                ),
                                                            ),
                                                        ],
                                                    ),
                                                ),
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::String,
                                                    ::ethers::core::abi::ethabi::ParamType::String,
                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                        ::std::vec![::ethers::core::abi::ethabi::ParamType::Bytes],
                                                    ),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned(
                                            "struct IbcCoreConnectionV1ConnectionEnd.Data",
                                        ),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bool"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getConsensusState"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("getConsensusState"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("clientId"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("height"),
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
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "consensusStateBytes",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bytes,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bool"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getExpectedTimePerBlock"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "getExpectedTimePerBlock",
                            ),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned(
                        "getHashedPacketAcknowledgementCommitment",
                    ),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "getHashedPacketAcknowledgementCommitment",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("portId"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("channelId"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("sequence"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes32"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bool"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getHashedPacketCommitment"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "getHashedPacketCommitment",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("portId"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("channelId"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("sequence"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes32"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bool"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getNextSequenceSend"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "getNextSequenceSend",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("portId"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("channelId"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("hasPacketReceipt"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("hasPacketReceipt"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("portId"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("channelId"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("sequence"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bool"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("nextSequenceAcks"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
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
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("nextSequenceRecvs"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
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
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("nextSequenceSends"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
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
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("packetReceipts"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
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
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint8"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("portCapabilityPath"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("portCapabilityPath"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("portId"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bytes,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::Pure,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("recvPacket"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("recvPacket"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("msg_"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                    ::ethers::core::abi::ethabi::ParamType::String,
                                                    ::ethers::core::abi::ethabi::ParamType::String,
                                                    ::ethers::core::abi::ethabi::ParamType::String,
                                                    ::ethers::core::abi::ethabi::ParamType::String,
                                                    ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                        ::std::vec![
                                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                        ],
                                                    ),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
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
                                            "struct IBCMsgs.MsgPacketRecv",
                                        ),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("registerClient"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
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
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("sendPacket"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("sendPacket"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("sourcePort"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("sourceChannel"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("timeoutHeight"),
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
                                    name: ::std::borrow::ToOwned::to_owned("timeoutTimestamp"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("data"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bytes,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("setExpectedTimePerBlock"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "setExpectedTimePerBlock",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "expectedTimePerBlock_",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("timeoutPacket"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("timeoutPacket"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("msg_"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                    ::ethers::core::abi::ethabi::ParamType::String,
                                                    ::ethers::core::abi::ethabi::ParamType::String,
                                                    ::ethers::core::abi::ethabi::ParamType::String,
                                                    ::ethers::core::abi::ethabi::ParamType::String,
                                                    ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                        ::std::vec![
                                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                        ],
                                                    ),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned(
                                            "struct IBCMsgs.MsgPacketTimeout",
                                        ),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("updateClient"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("updateClient"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("msg_"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned(
                                            "struct IBCMsgs.MsgUpdateClient",
                                        ),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("writeAcknowledgement"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "writeAcknowledgement",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("destinationPortId"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "destinationChannel",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("sequence"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("acknowledgement"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bytes,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
            ]),
            events: ::core::convert::From::from([
                (
                    ::std::borrow::ToOwned::to_owned("AcknowledgePacket"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned("AcknowledgePacket"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("packet"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                        ],
                                    ),
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("acknowledgement"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bytes,
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("ChannelCloseConfirm"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned(
                                "ChannelCloseConfirm",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("channelId"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("portId"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("ChannelCloseInit"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned("ChannelCloseInit"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("channelId"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("portId"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("ChannelOpenAck"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned("ChannelOpenAck"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("channelId"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("portId"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("ChannelOpenConfirm"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned("ChannelOpenConfirm"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("channelId"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("portId"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("ChannelOpenInit"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned("ChannelOpenInit"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("channelId"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("connectionId"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("portId"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "counterpartyPortId",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("ChannelOpenTry"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned("ChannelOpenTry"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("channelId"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("connectionId"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("portId"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "counterpartyPortId",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("version"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("ConnectionOpenAck"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned("ConnectionOpenAck"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("connectionId"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("ConnectionOpenConfirm"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned(
                                "ConnectionOpenConfirm",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("connectionId"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("ConnectionOpenInit"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned("ConnectionOpenInit"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("connectionId"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("ConnectionOpenTry"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned("ConnectionOpenTry"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("connectionId"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("GeneratedClientIdentifier"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned(
                                "GeneratedClientIdentifier",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("RecvPacket"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned("RecvPacket"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("packet"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                        ],
                                    ),
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("SendPacket"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned("SendPacket"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("sequence"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("sourcePort"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("sourceChannel"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("timeoutHeight"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                        ],
                                    ),
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("timeoutTimestamp"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("data"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bytes,
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("TimeoutPacket"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned("TimeoutPacket"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("packet"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                            ::ethers::core::abi::ethabi::ParamType::String,
                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                        ],
                                    ),
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("WriteAcknowledgement"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned(
                                "WriteAcknowledgement",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("destinationPortId"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "destinationChannel",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("sequence"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("acknowledgement"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bytes,
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
            ]),
            errors: ::std::collections::BTreeMap::new(),
            receive: false,
            fallback: false,
        }
    }
    ///The parsed JSON ABI of the contract.
    pub static IBCHANDLER_ABI: ::ethers::contract::Lazy<::ethers::core::abi::Abi> = ::ethers::contract::Lazy::new(
        __abi,
    );
    pub struct IBCHandler<M>(::ethers::contract::Contract<M>);
    impl<M> ::core::clone::Clone for IBCHandler<M> {
        fn clone(&self) -> Self {
            Self(::core::clone::Clone::clone(&self.0))
        }
    }
    impl<M> ::core::ops::Deref for IBCHandler<M> {
        type Target = ::ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> ::core::ops::DerefMut for IBCHandler<M> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<M> ::core::fmt::Debug for IBCHandler<M> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple(::core::stringify!(IBCHandler)).field(&self.address()).finish()
        }
    }
    impl<M: ::ethers::providers::Middleware> IBCHandler<M> {
        /// Creates a new contract instance with the specified `ethers` client at
        /// `address`. The contract derefs to a `ethers::Contract` object.
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(
                ::ethers::contract::Contract::new(
                    address.into(),
                    IBCHANDLER_ABI.clone(),
                    client,
                ),
            )
        }
        ///Calls the contract's `acknowledgePacket` (0x59f37976) function
        pub fn acknowledge_packet(
            &self,
            msg: MsgPacketAcknowledgement,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([89, 243, 121, 118], (msg,))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `bindPort` (0x117e886a) function
        pub fn bind_port(
            &self,
            port_id: ::std::string::String,
            module_address: ::ethers::core::types::Address,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([17, 126, 136, 106], (port_id, module_address))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `capabilities` (0xdd5b9f4d) function
        pub fn capabilities(
            &self,
            p0: ::ethers::core::types::Bytes,
            p1: ::ethers::core::types::U256,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::ethers::core::types::Address,
        > {
            self.0
                .method_hash([221, 91, 159, 77], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `channelCapabilityPath` (0x3bc3339f) function
        pub fn channel_capability_path(
            &self,
            port_id: ::std::string::String,
            channel_id: ::std::string::String,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::ethers::core::types::Bytes,
        > {
            self.0
                .method_hash([59, 195, 51, 159], (port_id, channel_id))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `channelCloseConfirm` (0x25cbc3a6) function
        pub fn channel_close_confirm(
            &self,
            msg: MsgChannelCloseConfirm,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([37, 203, 195, 166], (msg,))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `channelCloseInit` (0xa06cb3a2) function
        pub fn channel_close_init(
            &self,
            msg: MsgChannelCloseInit,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([160, 108, 179, 162], (msg,))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `channelOpenAck` (0x256c4199) function
        pub fn channel_open_ack(
            &self,
            msg: MsgChannelOpenAck,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([37, 108, 65, 153], (msg,))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `channelOpenConfirm` (0x5bd51b62) function
        pub fn channel_open_confirm(
            &self,
            msg: MsgChannelOpenConfirm,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([91, 213, 27, 98], (msg,))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `channelOpenInit` (0xdd3469fc) function
        pub fn channel_open_init(
            &self,
            msg: MsgChannelOpenInit,
        ) -> ::ethers::contract::builders::ContractCall<M, ::std::string::String> {
            self.0
                .method_hash([221, 52, 105, 252], (msg,))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `channelOpenTry` (0x11b88a15) function
        pub fn channel_open_try(
            &self,
            msg: MsgChannelOpenTry,
        ) -> ::ethers::contract::builders::ContractCall<M, ::std::string::String> {
            self.0
                .method_hash([17, 184, 138, 21], (msg,))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `channels` (0x5b3de260) function
        pub fn channels(
            &self,
            p0: ::std::string::String,
            p1: ::std::string::String,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            (u8, u8, IbcCoreChannelV1CounterpartyData, ::std::string::String),
        > {
            self.0
                .method_hash([91, 61, 226, 96], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `clientImpls` (0xd1297b8d) function
        pub fn client_impls(
            &self,
            p0: ::std::string::String,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::ethers::core::types::Address,
        > {
            self.0
                .method_hash([209, 41, 123, 141], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `clientRegistry` (0x990491a5) function
        pub fn client_registry(
            &self,
            p0: ::std::string::String,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::ethers::core::types::Address,
        > {
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
        ///Calls the contract's `connectionOpenAck` (0xb531861f) function
        pub fn connection_open_ack(
            &self,
            msg: MsgConnectionOpenAck,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([181, 49, 134, 31], (msg,))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `connectionOpenConfirm` (0x6a728f2c) function
        pub fn connection_open_confirm(
            &self,
            msg: MsgConnectionOpenConfirm,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([106, 114, 143, 44], (msg,))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `connectionOpenInit` (0x01c6400f) function
        pub fn connection_open_init(
            &self,
            msg: MsgConnectionOpenInit,
        ) -> ::ethers::contract::builders::ContractCall<M, ::std::string::String> {
            self.0
                .method_hash([1, 198, 64, 15], (msg,))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `connectionOpenTry` (0x04f68e5c) function
        pub fn connection_open_try(
            &self,
            msg: MsgConnectionOpenTry,
        ) -> ::ethers::contract::builders::ContractCall<M, ::std::string::String> {
            self.0
                .method_hash([4, 246, 142, 92], (msg,))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `connections` (0x31973f00) function
        pub fn connections(
            &self,
            p0: ::std::string::String,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            (::std::string::String, u8, IbcCoreConnectionV1CounterpartyData, u64),
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
        ///Calls the contract's `expectedTimePerBlock` (0xd31407fe) function
        pub fn expected_time_per_block(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, u64> {
            self.0
                .method_hash([211, 20, 7, 254], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getChannel` (0x3000217a) function
        pub fn get_channel(
            &self,
            port_id: ::std::string::String,
            channel_id: ::std::string::String,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            (IbcCoreChannelV1ChannelData, bool),
        > {
            self.0
                .method_hash([48, 0, 33, 122], (port_id, channel_id))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getClientState` (0x76c81c42) function
        pub fn get_client_state(
            &self,
            client_id: ::std::string::String,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            (::ethers::core::types::Bytes, bool),
        > {
            self.0
                .method_hash([118, 200, 28, 66], client_id)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getConnection` (0x27711a69) function
        pub fn get_connection(
            &self,
            connection_id: ::std::string::String,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            (IbcCoreConnectionV1ConnectionEndData, bool),
        > {
            self.0
                .method_hash([39, 113, 26, 105], connection_id)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getConsensusState` (0x6cf44bf4) function
        pub fn get_consensus_state(
            &self,
            client_id: ::std::string::String,
            height: IbcCoreClientV1HeightData,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            (::ethers::core::types::Bytes, bool),
        > {
            self.0
                .method_hash([108, 244, 75, 244], (client_id, height))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getExpectedTimePerBlock` (0xec75d829) function
        pub fn get_expected_time_per_block(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, u64> {
            self.0
                .method_hash([236, 117, 216, 41], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getHashedPacketAcknowledgementCommitment` (0x5be164ee) function
        pub fn get_hashed_packet_acknowledgement_commitment(
            &self,
            port_id: ::std::string::String,
            channel_id: ::std::string::String,
            sequence: u64,
        ) -> ::ethers::contract::builders::ContractCall<M, ([u8; 32], bool)> {
            self.0
                .method_hash([91, 225, 100, 238], (port_id, channel_id, sequence))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getHashedPacketCommitment` (0x23402a33) function
        pub fn get_hashed_packet_commitment(
            &self,
            port_id: ::std::string::String,
            channel_id: ::std::string::String,
            sequence: u64,
        ) -> ::ethers::contract::builders::ContractCall<M, ([u8; 32], bool)> {
            self.0
                .method_hash([35, 64, 42, 51], (port_id, channel_id, sequence))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getNextSequenceSend` (0x582418b6) function
        pub fn get_next_sequence_send(
            &self,
            port_id: ::std::string::String,
            channel_id: ::std::string::String,
        ) -> ::ethers::contract::builders::ContractCall<M, u64> {
            self.0
                .method_hash([88, 36, 24, 182], (port_id, channel_id))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `hasPacketReceipt` (0x5a9afac3) function
        pub fn has_packet_receipt(
            &self,
            port_id: ::std::string::String,
            channel_id: ::std::string::String,
            sequence: u64,
        ) -> ::ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([90, 154, 250, 195], (port_id, channel_id, sequence))
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
        ///Calls the contract's `portCapabilityPath` (0x2570dae0) function
        pub fn port_capability_path(
            &self,
            port_id: ::std::string::String,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::ethers::core::types::Bytes,
        > {
            self.0
                .method_hash([37, 112, 218, 224], port_id)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `recvPacket` (0x236ebd70) function
        pub fn recv_packet(
            &self,
            msg: MsgPacketRecv,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([35, 110, 189, 112], (msg,))
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
        ///Calls the contract's `sendPacket` (0xae4cd201) function
        pub fn send_packet(
            &self,
            source_port: ::std::string::String,
            source_channel: ::std::string::String,
            timeout_height: IbcCoreClientV1HeightData,
            timeout_timestamp: u64,
            data: ::ethers::core::types::Bytes,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash(
                    [174, 76, 210, 1],
                    (
                        source_port,
                        source_channel,
                        timeout_height,
                        timeout_timestamp,
                        data,
                    ),
                )
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `setExpectedTimePerBlock` (0x27184c13) function
        pub fn set_expected_time_per_block(
            &self,
            expected_time_per_block: u64,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([39, 24, 76, 19], expected_time_per_block)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `timeoutPacket` (0xaa18c8b1) function
        pub fn timeout_packet(
            &self,
            msg: MsgPacketTimeout,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([170, 24, 200, 177], (msg,))
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
        ///Calls the contract's `writeAcknowledgement` (0xb56e79de) function
        pub fn write_acknowledgement(
            &self,
            destination_port_id: ::std::string::String,
            destination_channel: ::std::string::String,
            sequence: u64,
            acknowledgement: ::ethers::core::types::Bytes,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash(
                    [181, 110, 121, 222],
                    (destination_port_id, destination_channel, sequence, acknowledgement),
                )
                .expect("method not found (this should never happen)")
        }
        ///Gets the contract's `AcknowledgePacket` event
        pub fn acknowledge_packet_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            AcknowledgePacketFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `ChannelCloseConfirm` event
        pub fn channel_close_confirm_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            ChannelCloseConfirmFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `ChannelCloseInit` event
        pub fn channel_close_init_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            ChannelCloseInitFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `ChannelOpenAck` event
        pub fn channel_open_ack_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            ChannelOpenAckFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `ChannelOpenConfirm` event
        pub fn channel_open_confirm_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            ChannelOpenConfirmFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `ChannelOpenInit` event
        pub fn channel_open_init_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            ChannelOpenInitFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `ChannelOpenTry` event
        pub fn channel_open_try_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            ChannelOpenTryFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `ConnectionOpenAck` event
        pub fn connection_open_ack_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            ConnectionOpenAckFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `ConnectionOpenConfirm` event
        pub fn connection_open_confirm_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            ConnectionOpenConfirmFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `ConnectionOpenInit` event
        pub fn connection_open_init_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            ConnectionOpenInitFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `ConnectionOpenTry` event
        pub fn connection_open_try_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            ConnectionOpenTryFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `GeneratedClientIdentifier` event
        pub fn generated_client_identifier_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            GeneratedClientIdentifierFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `RecvPacket` event
        pub fn recv_packet_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            RecvPacketFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `SendPacket` event
        pub fn send_packet_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            SendPacketFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `TimeoutPacket` event
        pub fn timeout_packet_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            TimeoutPacketFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `WriteAcknowledgement` event
        pub fn write_acknowledgement_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            WriteAcknowledgementFilter,
        > {
            self.0.event()
        }
        /// Returns an `Event` builder for all the events of this contract.
        pub fn events(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            IBCHandlerEvents,
        > {
            self.0.event_with_filter(::core::default::Default::default())
        }
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>>
    for IBCHandler<M> {
        fn from(contract: ::ethers::contract::Contract<M>) -> Self {
            Self::new(contract.address(), contract.client())
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
        Hash
    )]
    #[ethevent(
        name = "AcknowledgePacket",
        abi = "AcknowledgePacket((uint64,string,string,string,string,bytes,(uint64,uint64),uint64),bytes)"
    )]
    pub struct AcknowledgePacketFilter {
        pub packet: IbcCoreChannelV1PacketData,
        pub acknowledgement: ::ethers::core::types::Bytes,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(name = "ChannelCloseConfirm", abi = "ChannelCloseConfirm(string,string)")]
    pub struct ChannelCloseConfirmFilter {
        pub channel_id: ::std::string::String,
        pub port_id: ::std::string::String,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(name = "ChannelCloseInit", abi = "ChannelCloseInit(string,string)")]
    pub struct ChannelCloseInitFilter {
        pub channel_id: ::std::string::String,
        pub port_id: ::std::string::String,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(name = "ChannelOpenAck", abi = "ChannelOpenAck(string,string)")]
    pub struct ChannelOpenAckFilter {
        pub channel_id: ::std::string::String,
        pub port_id: ::std::string::String,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(name = "ChannelOpenConfirm", abi = "ChannelOpenConfirm(string,string)")]
    pub struct ChannelOpenConfirmFilter {
        pub channel_id: ::std::string::String,
        pub port_id: ::std::string::String,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(
        name = "ChannelOpenInit",
        abi = "ChannelOpenInit(string,string,string,string)"
    )]
    pub struct ChannelOpenInitFilter {
        pub channel_id: ::std::string::String,
        pub connection_id: ::std::string::String,
        pub port_id: ::std::string::String,
        pub counterparty_port_id: ::std::string::String,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(
        name = "ChannelOpenTry",
        abi = "ChannelOpenTry(string,string,string,string,string)"
    )]
    pub struct ChannelOpenTryFilter {
        pub channel_id: ::std::string::String,
        pub connection_id: ::std::string::String,
        pub port_id: ::std::string::String,
        pub counterparty_port_id: ::std::string::String,
        pub version: ::std::string::String,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(name = "ConnectionOpenAck", abi = "ConnectionOpenAck(string)")]
    pub struct ConnectionOpenAckFilter {
        pub connection_id: ::std::string::String,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(name = "ConnectionOpenConfirm", abi = "ConnectionOpenConfirm(string)")]
    pub struct ConnectionOpenConfirmFilter {
        pub connection_id: ::std::string::String,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(name = "ConnectionOpenInit", abi = "ConnectionOpenInit(string)")]
    pub struct ConnectionOpenInitFilter {
        pub connection_id: ::std::string::String,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(name = "ConnectionOpenTry", abi = "ConnectionOpenTry(string)")]
    pub struct ConnectionOpenTryFilter {
        pub connection_id: ::std::string::String,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(
        name = "GeneratedClientIdentifier",
        abi = "GeneratedClientIdentifier(string)"
    )]
    pub struct GeneratedClientIdentifierFilter(pub ::std::string::String);
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(
        name = "RecvPacket",
        abi = "RecvPacket((uint64,string,string,string,string,bytes,(uint64,uint64),uint64))"
    )]
    pub struct RecvPacketFilter {
        pub packet: IbcCoreChannelV1PacketData,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(
        name = "SendPacket",
        abi = "SendPacket(uint64,string,string,(uint64,uint64),uint64,bytes)"
    )]
    pub struct SendPacketFilter {
        pub sequence: u64,
        pub source_port: ::std::string::String,
        pub source_channel: ::std::string::String,
        pub timeout_height: IbcCoreClientV1HeightData,
        pub timeout_timestamp: u64,
        pub data: ::ethers::core::types::Bytes,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(
        name = "TimeoutPacket",
        abi = "TimeoutPacket((uint64,string,string,string,string,bytes,(uint64,uint64),uint64))"
    )]
    pub struct TimeoutPacketFilter {
        pub packet: IbcCoreChannelV1PacketData,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(
        name = "WriteAcknowledgement",
        abi = "WriteAcknowledgement(string,string,uint64,bytes)"
    )]
    pub struct WriteAcknowledgementFilter {
        pub destination_port_id: ::std::string::String,
        pub destination_channel: ::std::string::String,
        pub sequence: u64,
        pub acknowledgement: ::ethers::core::types::Bytes,
    }
    ///Container type for all of the contract's events
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum IBCHandlerEvents {
        AcknowledgePacketFilter(AcknowledgePacketFilter),
        ChannelCloseConfirmFilter(ChannelCloseConfirmFilter),
        ChannelCloseInitFilter(ChannelCloseInitFilter),
        ChannelOpenAckFilter(ChannelOpenAckFilter),
        ChannelOpenConfirmFilter(ChannelOpenConfirmFilter),
        ChannelOpenInitFilter(ChannelOpenInitFilter),
        ChannelOpenTryFilter(ChannelOpenTryFilter),
        ConnectionOpenAckFilter(ConnectionOpenAckFilter),
        ConnectionOpenConfirmFilter(ConnectionOpenConfirmFilter),
        ConnectionOpenInitFilter(ConnectionOpenInitFilter),
        ConnectionOpenTryFilter(ConnectionOpenTryFilter),
        GeneratedClientIdentifierFilter(GeneratedClientIdentifierFilter),
        RecvPacketFilter(RecvPacketFilter),
        SendPacketFilter(SendPacketFilter),
        TimeoutPacketFilter(TimeoutPacketFilter),
        WriteAcknowledgementFilter(WriteAcknowledgementFilter),
    }
    impl ::ethers::contract::EthLogDecode for IBCHandlerEvents {
        fn decode_log(
            log: &::ethers::core::abi::RawLog,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::Error> {
            if let Ok(decoded) = AcknowledgePacketFilter::decode_log(log) {
                return Ok(IBCHandlerEvents::AcknowledgePacketFilter(decoded));
            }
            if let Ok(decoded) = ChannelCloseConfirmFilter::decode_log(log) {
                return Ok(IBCHandlerEvents::ChannelCloseConfirmFilter(decoded));
            }
            if let Ok(decoded) = ChannelCloseInitFilter::decode_log(log) {
                return Ok(IBCHandlerEvents::ChannelCloseInitFilter(decoded));
            }
            if let Ok(decoded) = ChannelOpenAckFilter::decode_log(log) {
                return Ok(IBCHandlerEvents::ChannelOpenAckFilter(decoded));
            }
            if let Ok(decoded) = ChannelOpenConfirmFilter::decode_log(log) {
                return Ok(IBCHandlerEvents::ChannelOpenConfirmFilter(decoded));
            }
            if let Ok(decoded) = ChannelOpenInitFilter::decode_log(log) {
                return Ok(IBCHandlerEvents::ChannelOpenInitFilter(decoded));
            }
            if let Ok(decoded) = ChannelOpenTryFilter::decode_log(log) {
                return Ok(IBCHandlerEvents::ChannelOpenTryFilter(decoded));
            }
            if let Ok(decoded) = ConnectionOpenAckFilter::decode_log(log) {
                return Ok(IBCHandlerEvents::ConnectionOpenAckFilter(decoded));
            }
            if let Ok(decoded) = ConnectionOpenConfirmFilter::decode_log(log) {
                return Ok(IBCHandlerEvents::ConnectionOpenConfirmFilter(decoded));
            }
            if let Ok(decoded) = ConnectionOpenInitFilter::decode_log(log) {
                return Ok(IBCHandlerEvents::ConnectionOpenInitFilter(decoded));
            }
            if let Ok(decoded) = ConnectionOpenTryFilter::decode_log(log) {
                return Ok(IBCHandlerEvents::ConnectionOpenTryFilter(decoded));
            }
            if let Ok(decoded) = GeneratedClientIdentifierFilter::decode_log(log) {
                return Ok(IBCHandlerEvents::GeneratedClientIdentifierFilter(decoded));
            }
            if let Ok(decoded) = RecvPacketFilter::decode_log(log) {
                return Ok(IBCHandlerEvents::RecvPacketFilter(decoded));
            }
            if let Ok(decoded) = SendPacketFilter::decode_log(log) {
                return Ok(IBCHandlerEvents::SendPacketFilter(decoded));
            }
            if let Ok(decoded) = TimeoutPacketFilter::decode_log(log) {
                return Ok(IBCHandlerEvents::TimeoutPacketFilter(decoded));
            }
            if let Ok(decoded) = WriteAcknowledgementFilter::decode_log(log) {
                return Ok(IBCHandlerEvents::WriteAcknowledgementFilter(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData)
        }
    }
    impl ::core::fmt::Display for IBCHandlerEvents {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::AcknowledgePacketFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ChannelCloseConfirmFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ChannelCloseInitFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ChannelOpenAckFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ChannelOpenConfirmFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ChannelOpenInitFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ChannelOpenTryFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ConnectionOpenAckFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ConnectionOpenConfirmFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ConnectionOpenInitFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ConnectionOpenTryFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::GeneratedClientIdentifierFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::RecvPacketFilter(element) => ::core::fmt::Display::fmt(element, f),
                Self::SendPacketFilter(element) => ::core::fmt::Display::fmt(element, f),
                Self::TimeoutPacketFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::WriteAcknowledgementFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
            }
        }
    }
    impl ::core::convert::From<AcknowledgePacketFilter> for IBCHandlerEvents {
        fn from(value: AcknowledgePacketFilter) -> Self {
            Self::AcknowledgePacketFilter(value)
        }
    }
    impl ::core::convert::From<ChannelCloseConfirmFilter> for IBCHandlerEvents {
        fn from(value: ChannelCloseConfirmFilter) -> Self {
            Self::ChannelCloseConfirmFilter(value)
        }
    }
    impl ::core::convert::From<ChannelCloseInitFilter> for IBCHandlerEvents {
        fn from(value: ChannelCloseInitFilter) -> Self {
            Self::ChannelCloseInitFilter(value)
        }
    }
    impl ::core::convert::From<ChannelOpenAckFilter> for IBCHandlerEvents {
        fn from(value: ChannelOpenAckFilter) -> Self {
            Self::ChannelOpenAckFilter(value)
        }
    }
    impl ::core::convert::From<ChannelOpenConfirmFilter> for IBCHandlerEvents {
        fn from(value: ChannelOpenConfirmFilter) -> Self {
            Self::ChannelOpenConfirmFilter(value)
        }
    }
    impl ::core::convert::From<ChannelOpenInitFilter> for IBCHandlerEvents {
        fn from(value: ChannelOpenInitFilter) -> Self {
            Self::ChannelOpenInitFilter(value)
        }
    }
    impl ::core::convert::From<ChannelOpenTryFilter> for IBCHandlerEvents {
        fn from(value: ChannelOpenTryFilter) -> Self {
            Self::ChannelOpenTryFilter(value)
        }
    }
    impl ::core::convert::From<ConnectionOpenAckFilter> for IBCHandlerEvents {
        fn from(value: ConnectionOpenAckFilter) -> Self {
            Self::ConnectionOpenAckFilter(value)
        }
    }
    impl ::core::convert::From<ConnectionOpenConfirmFilter> for IBCHandlerEvents {
        fn from(value: ConnectionOpenConfirmFilter) -> Self {
            Self::ConnectionOpenConfirmFilter(value)
        }
    }
    impl ::core::convert::From<ConnectionOpenInitFilter> for IBCHandlerEvents {
        fn from(value: ConnectionOpenInitFilter) -> Self {
            Self::ConnectionOpenInitFilter(value)
        }
    }
    impl ::core::convert::From<ConnectionOpenTryFilter> for IBCHandlerEvents {
        fn from(value: ConnectionOpenTryFilter) -> Self {
            Self::ConnectionOpenTryFilter(value)
        }
    }
    impl ::core::convert::From<GeneratedClientIdentifierFilter> for IBCHandlerEvents {
        fn from(value: GeneratedClientIdentifierFilter) -> Self {
            Self::GeneratedClientIdentifierFilter(value)
        }
    }
    impl ::core::convert::From<RecvPacketFilter> for IBCHandlerEvents {
        fn from(value: RecvPacketFilter) -> Self {
            Self::RecvPacketFilter(value)
        }
    }
    impl ::core::convert::From<SendPacketFilter> for IBCHandlerEvents {
        fn from(value: SendPacketFilter) -> Self {
            Self::SendPacketFilter(value)
        }
    }
    impl ::core::convert::From<TimeoutPacketFilter> for IBCHandlerEvents {
        fn from(value: TimeoutPacketFilter) -> Self {
            Self::TimeoutPacketFilter(value)
        }
    }
    impl ::core::convert::From<WriteAcknowledgementFilter> for IBCHandlerEvents {
        fn from(value: WriteAcknowledgementFilter) -> Self {
            Self::WriteAcknowledgementFilter(value)
        }
    }
    ///Container type for all input parameters for the `acknowledgePacket` function with signature `acknowledgePacket(((uint64,string,string,string,string,bytes,(uint64,uint64),uint64),bytes,bytes,(uint64,uint64)))` and selector `0x59f37976`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "acknowledgePacket",
        abi = "acknowledgePacket(((uint64,string,string,string,string,bytes,(uint64,uint64),uint64),bytes,bytes,(uint64,uint64)))"
    )]
    pub struct AcknowledgePacketCall {
        pub msg: MsgPacketAcknowledgement,
    }
    ///Container type for all input parameters for the `bindPort` function with signature `bindPort(string,address)` and selector `0x117e886a`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "bindPort", abi = "bindPort(string,address)")]
    pub struct BindPortCall {
        pub port_id: ::std::string::String,
        pub module_address: ::ethers::core::types::Address,
    }
    ///Container type for all input parameters for the `capabilities` function with signature `capabilities(bytes,uint256)` and selector `0xdd5b9f4d`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "capabilities", abi = "capabilities(bytes,uint256)")]
    pub struct CapabilitiesCall(
        pub ::ethers::core::types::Bytes,
        pub ::ethers::core::types::U256,
    );
    ///Container type for all input parameters for the `channelCapabilityPath` function with signature `channelCapabilityPath(string,string)` and selector `0x3bc3339f`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "channelCapabilityPath",
        abi = "channelCapabilityPath(string,string)"
    )]
    pub struct ChannelCapabilityPathCall {
        pub port_id: ::std::string::String,
        pub channel_id: ::std::string::String,
    }
    ///Container type for all input parameters for the `channelCloseConfirm` function with signature `channelCloseConfirm((string,string,bytes,(uint64,uint64)))` and selector `0x25cbc3a6`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "channelCloseConfirm",
        abi = "channelCloseConfirm((string,string,bytes,(uint64,uint64)))"
    )]
    pub struct ChannelCloseConfirmCall {
        pub msg: MsgChannelCloseConfirm,
    }
    ///Container type for all input parameters for the `channelCloseInit` function with signature `channelCloseInit((string,string))` and selector `0xa06cb3a2`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "channelCloseInit", abi = "channelCloseInit((string,string))")]
    pub struct ChannelCloseInitCall {
        pub msg: MsgChannelCloseInit,
    }
    ///Container type for all input parameters for the `channelOpenAck` function with signature `channelOpenAck((string,string,string,string,bytes,(uint64,uint64)))` and selector `0x256c4199`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "channelOpenAck",
        abi = "channelOpenAck((string,string,string,string,bytes,(uint64,uint64)))"
    )]
    pub struct ChannelOpenAckCall {
        pub msg: MsgChannelOpenAck,
    }
    ///Container type for all input parameters for the `channelOpenConfirm` function with signature `channelOpenConfirm((string,string,bytes,(uint64,uint64)))` and selector `0x5bd51b62`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "channelOpenConfirm",
        abi = "channelOpenConfirm((string,string,bytes,(uint64,uint64)))"
    )]
    pub struct ChannelOpenConfirmCall {
        pub msg: MsgChannelOpenConfirm,
    }
    ///Container type for all input parameters for the `channelOpenInit` function with signature `channelOpenInit((string,(uint8,uint8,(string,string),string[],string)))` and selector `0xdd3469fc`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "channelOpenInit",
        abi = "channelOpenInit((string,(uint8,uint8,(string,string),string[],string)))"
    )]
    pub struct ChannelOpenInitCall {
        pub msg: MsgChannelOpenInit,
    }
    ///Container type for all input parameters for the `channelOpenTry` function with signature `channelOpenTry((string,(uint8,uint8,(string,string),string[],string),string,bytes,(uint64,uint64)))` and selector `0x11b88a15`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "channelOpenTry",
        abi = "channelOpenTry((string,(uint8,uint8,(string,string),string[],string),string,bytes,(uint64,uint64)))"
    )]
    pub struct ChannelOpenTryCall {
        pub msg: MsgChannelOpenTry,
    }
    ///Container type for all input parameters for the `channels` function with signature `channels(string,string)` and selector `0x5b3de260`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
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
        Hash
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
        Hash
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
        Hash
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
        Hash
    )]
    #[ethcall(name = "commitments", abi = "commitments(bytes32)")]
    pub struct CommitmentsCall(pub [u8; 32]);
    ///Container type for all input parameters for the `connectionOpenAck` function with signature `connectionOpenAck((string,bytes,(string,string[]),string,bytes,bytes,bytes,(uint64,uint64),(uint64,uint64)))` and selector `0xb531861f`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "connectionOpenAck",
        abi = "connectionOpenAck((string,bytes,(string,string[]),string,bytes,bytes,bytes,(uint64,uint64),(uint64,uint64)))"
    )]
    pub struct ConnectionOpenAckCall {
        pub msg: MsgConnectionOpenAck,
    }
    ///Container type for all input parameters for the `connectionOpenConfirm` function with signature `connectionOpenConfirm((string,bytes,(uint64,uint64)))` and selector `0x6a728f2c`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "connectionOpenConfirm",
        abi = "connectionOpenConfirm((string,bytes,(uint64,uint64)))"
    )]
    pub struct ConnectionOpenConfirmCall {
        pub msg: MsgConnectionOpenConfirm,
    }
    ///Container type for all input parameters for the `connectionOpenInit` function with signature `connectionOpenInit((string,(string,string,(bytes)),uint64))` and selector `0x01c6400f`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "connectionOpenInit",
        abi = "connectionOpenInit((string,(string,string,(bytes)),uint64))"
    )]
    pub struct ConnectionOpenInitCall {
        pub msg: MsgConnectionOpenInit,
    }
    ///Container type for all input parameters for the `connectionOpenTry` function with signature `connectionOpenTry(((string,string,(bytes)),uint64,string,bytes,(string,string[])[],bytes,bytes,bytes,(uint64,uint64),(uint64,uint64)))` and selector `0x04f68e5c`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "connectionOpenTry",
        abi = "connectionOpenTry(((string,string,(bytes)),uint64,string,bytes,(string,string[])[],bytes,bytes,bytes,(uint64,uint64),(uint64,uint64)))"
    )]
    pub struct ConnectionOpenTryCall {
        pub msg: MsgConnectionOpenTry,
    }
    ///Container type for all input parameters for the `connections` function with signature `connections(string)` and selector `0x31973f00`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
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
        Hash
    )]
    #[ethcall(name = "createClient", abi = "createClient((string,bytes,bytes))")]
    pub struct CreateClientCall {
        pub msg: MsgCreateClient,
    }
    ///Container type for all input parameters for the `expectedTimePerBlock` function with signature `expectedTimePerBlock()` and selector `0xd31407fe`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "expectedTimePerBlock", abi = "expectedTimePerBlock()")]
    pub struct ExpectedTimePerBlockCall;
    ///Container type for all input parameters for the `getChannel` function with signature `getChannel(string,string)` and selector `0x3000217a`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "getChannel", abi = "getChannel(string,string)")]
    pub struct GetChannelCall {
        pub port_id: ::std::string::String,
        pub channel_id: ::std::string::String,
    }
    ///Container type for all input parameters for the `getClientState` function with signature `getClientState(string)` and selector `0x76c81c42`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "getClientState", abi = "getClientState(string)")]
    pub struct GetClientStateCall {
        pub client_id: ::std::string::String,
    }
    ///Container type for all input parameters for the `getConnection` function with signature `getConnection(string)` and selector `0x27711a69`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "getConnection", abi = "getConnection(string)")]
    pub struct GetConnectionCall {
        pub connection_id: ::std::string::String,
    }
    ///Container type for all input parameters for the `getConsensusState` function with signature `getConsensusState(string,(uint64,uint64))` and selector `0x6cf44bf4`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "getConsensusState",
        abi = "getConsensusState(string,(uint64,uint64))"
    )]
    pub struct GetConsensusStateCall {
        pub client_id: ::std::string::String,
        pub height: IbcCoreClientV1HeightData,
    }
    ///Container type for all input parameters for the `getExpectedTimePerBlock` function with signature `getExpectedTimePerBlock()` and selector `0xec75d829`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "getExpectedTimePerBlock", abi = "getExpectedTimePerBlock()")]
    pub struct GetExpectedTimePerBlockCall;
    ///Container type for all input parameters for the `getHashedPacketAcknowledgementCommitment` function with signature `getHashedPacketAcknowledgementCommitment(string,string,uint64)` and selector `0x5be164ee`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "getHashedPacketAcknowledgementCommitment",
        abi = "getHashedPacketAcknowledgementCommitment(string,string,uint64)"
    )]
    pub struct GetHashedPacketAcknowledgementCommitmentCall {
        pub port_id: ::std::string::String,
        pub channel_id: ::std::string::String,
        pub sequence: u64,
    }
    ///Container type for all input parameters for the `getHashedPacketCommitment` function with signature `getHashedPacketCommitment(string,string,uint64)` and selector `0x23402a33`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "getHashedPacketCommitment",
        abi = "getHashedPacketCommitment(string,string,uint64)"
    )]
    pub struct GetHashedPacketCommitmentCall {
        pub port_id: ::std::string::String,
        pub channel_id: ::std::string::String,
        pub sequence: u64,
    }
    ///Container type for all input parameters for the `getNextSequenceSend` function with signature `getNextSequenceSend(string,string)` and selector `0x582418b6`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "getNextSequenceSend", abi = "getNextSequenceSend(string,string)")]
    pub struct GetNextSequenceSendCall {
        pub port_id: ::std::string::String,
        pub channel_id: ::std::string::String,
    }
    ///Container type for all input parameters for the `hasPacketReceipt` function with signature `hasPacketReceipt(string,string,uint64)` and selector `0x5a9afac3`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "hasPacketReceipt", abi = "hasPacketReceipt(string,string,uint64)")]
    pub struct HasPacketReceiptCall {
        pub port_id: ::std::string::String,
        pub channel_id: ::std::string::String,
        pub sequence: u64,
    }
    ///Container type for all input parameters for the `nextSequenceAcks` function with signature `nextSequenceAcks(string,string)` and selector `0x1390d28d`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "nextSequenceAcks", abi = "nextSequenceAcks(string,string)")]
    pub struct NextSequenceAcksCall(
        pub ::std::string::String,
        pub ::std::string::String,
    );
    ///Container type for all input parameters for the `nextSequenceRecvs` function with signature `nextSequenceRecvs(string,string)` and selector `0xc930b1b0`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "nextSequenceRecvs", abi = "nextSequenceRecvs(string,string)")]
    pub struct NextSequenceRecvsCall(
        pub ::std::string::String,
        pub ::std::string::String,
    );
    ///Container type for all input parameters for the `nextSequenceSends` function with signature `nextSequenceSends(string,string)` and selector `0x821cb5d0`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "nextSequenceSends", abi = "nextSequenceSends(string,string)")]
    pub struct NextSequenceSendsCall(
        pub ::std::string::String,
        pub ::std::string::String,
    );
    ///Container type for all input parameters for the `packetReceipts` function with signature `packetReceipts(string,string,uint64)` and selector `0x26078437`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "packetReceipts", abi = "packetReceipts(string,string,uint64)")]
    pub struct PacketReceiptsCall(
        pub ::std::string::String,
        pub ::std::string::String,
        pub u64,
    );
    ///Container type for all input parameters for the `portCapabilityPath` function with signature `portCapabilityPath(string)` and selector `0x2570dae0`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "portCapabilityPath", abi = "portCapabilityPath(string)")]
    pub struct PortCapabilityPathCall {
        pub port_id: ::std::string::String,
    }
    ///Container type for all input parameters for the `recvPacket` function with signature `recvPacket(((uint64,string,string,string,string,bytes,(uint64,uint64),uint64),bytes,(uint64,uint64)))` and selector `0x236ebd70`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "recvPacket",
        abi = "recvPacket(((uint64,string,string,string,string,bytes,(uint64,uint64),uint64),bytes,(uint64,uint64)))"
    )]
    pub struct RecvPacketCall {
        pub msg: MsgPacketRecv,
    }
    ///Container type for all input parameters for the `registerClient` function with signature `registerClient(string,address)` and selector `0x18c19870`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "registerClient", abi = "registerClient(string,address)")]
    pub struct RegisterClientCall {
        pub client_type: ::std::string::String,
        pub client: ::ethers::core::types::Address,
    }
    ///Container type for all input parameters for the `sendPacket` function with signature `sendPacket(string,string,(uint64,uint64),uint64,bytes)` and selector `0xae4cd201`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "sendPacket",
        abi = "sendPacket(string,string,(uint64,uint64),uint64,bytes)"
    )]
    pub struct SendPacketCall {
        pub source_port: ::std::string::String,
        pub source_channel: ::std::string::String,
        pub timeout_height: IbcCoreClientV1HeightData,
        pub timeout_timestamp: u64,
        pub data: ::ethers::core::types::Bytes,
    }
    ///Container type for all input parameters for the `setExpectedTimePerBlock` function with signature `setExpectedTimePerBlock(uint64)` and selector `0x27184c13`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "setExpectedTimePerBlock", abi = "setExpectedTimePerBlock(uint64)")]
    pub struct SetExpectedTimePerBlockCall {
        pub expected_time_per_block: u64,
    }
    ///Container type for all input parameters for the `timeoutPacket` function with signature `timeoutPacket(((uint64,string,string,string,string,bytes,(uint64,uint64),uint64),bytes,(uint64,uint64),uint64))` and selector `0xaa18c8b1`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "timeoutPacket",
        abi = "timeoutPacket(((uint64,string,string,string,string,bytes,(uint64,uint64),uint64),bytes,(uint64,uint64),uint64))"
    )]
    pub struct TimeoutPacketCall {
        pub msg: MsgPacketTimeout,
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
        Hash
    )]
    #[ethcall(name = "updateClient", abi = "updateClient((string,bytes))")]
    pub struct UpdateClientCall {
        pub msg: MsgUpdateClient,
    }
    ///Container type for all input parameters for the `writeAcknowledgement` function with signature `writeAcknowledgement(string,string,uint64,bytes)` and selector `0xb56e79de`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "writeAcknowledgement",
        abi = "writeAcknowledgement(string,string,uint64,bytes)"
    )]
    pub struct WriteAcknowledgementCall {
        pub destination_port_id: ::std::string::String,
        pub destination_channel: ::std::string::String,
        pub sequence: u64,
        pub acknowledgement: ::ethers::core::types::Bytes,
    }
    ///Container type for all of the contract's call
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum IBCHandlerCalls {
        AcknowledgePacket(AcknowledgePacketCall),
        BindPort(BindPortCall),
        Capabilities(CapabilitiesCall),
        ChannelCapabilityPath(ChannelCapabilityPathCall),
        ChannelCloseConfirm(ChannelCloseConfirmCall),
        ChannelCloseInit(ChannelCloseInitCall),
        ChannelOpenAck(ChannelOpenAckCall),
        ChannelOpenConfirm(ChannelOpenConfirmCall),
        ChannelOpenInit(ChannelOpenInitCall),
        ChannelOpenTry(ChannelOpenTryCall),
        Channels(ChannelsCall),
        ClientImpls(ClientImplsCall),
        ClientRegistry(ClientRegistryCall),
        ClientTypes(ClientTypesCall),
        Commitments(CommitmentsCall),
        ConnectionOpenAck(ConnectionOpenAckCall),
        ConnectionOpenConfirm(ConnectionOpenConfirmCall),
        ConnectionOpenInit(ConnectionOpenInitCall),
        ConnectionOpenTry(ConnectionOpenTryCall),
        Connections(ConnectionsCall),
        CreateClient(CreateClientCall),
        ExpectedTimePerBlock(ExpectedTimePerBlockCall),
        GetChannel(GetChannelCall),
        GetClientState(GetClientStateCall),
        GetConnection(GetConnectionCall),
        GetConsensusState(GetConsensusStateCall),
        GetExpectedTimePerBlock(GetExpectedTimePerBlockCall),
        GetHashedPacketAcknowledgementCommitment(
            GetHashedPacketAcknowledgementCommitmentCall,
        ),
        GetHashedPacketCommitment(GetHashedPacketCommitmentCall),
        GetNextSequenceSend(GetNextSequenceSendCall),
        HasPacketReceipt(HasPacketReceiptCall),
        NextSequenceAcks(NextSequenceAcksCall),
        NextSequenceRecvs(NextSequenceRecvsCall),
        NextSequenceSends(NextSequenceSendsCall),
        PacketReceipts(PacketReceiptsCall),
        PortCapabilityPath(PortCapabilityPathCall),
        RecvPacket(RecvPacketCall),
        RegisterClient(RegisterClientCall),
        SendPacket(SendPacketCall),
        SetExpectedTimePerBlock(SetExpectedTimePerBlockCall),
        TimeoutPacket(TimeoutPacketCall),
        UpdateClient(UpdateClientCall),
        WriteAcknowledgement(WriteAcknowledgementCall),
    }
    impl ::ethers::core::abi::AbiDecode for IBCHandlerCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded) = <AcknowledgePacketCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::AcknowledgePacket(decoded));
            }
            if let Ok(decoded) = <BindPortCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::BindPort(decoded));
            }
            if let Ok(decoded) = <CapabilitiesCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::Capabilities(decoded));
            }
            if let Ok(decoded) = <ChannelCapabilityPathCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::ChannelCapabilityPath(decoded));
            }
            if let Ok(decoded) = <ChannelCloseConfirmCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::ChannelCloseConfirm(decoded));
            }
            if let Ok(decoded) = <ChannelCloseInitCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::ChannelCloseInit(decoded));
            }
            if let Ok(decoded) = <ChannelOpenAckCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::ChannelOpenAck(decoded));
            }
            if let Ok(decoded) = <ChannelOpenConfirmCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::ChannelOpenConfirm(decoded));
            }
            if let Ok(decoded) = <ChannelOpenInitCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::ChannelOpenInit(decoded));
            }
            if let Ok(decoded) = <ChannelOpenTryCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::ChannelOpenTry(decoded));
            }
            if let Ok(decoded) = <ChannelsCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::Channels(decoded));
            }
            if let Ok(decoded) = <ClientImplsCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::ClientImpls(decoded));
            }
            if let Ok(decoded) = <ClientRegistryCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::ClientRegistry(decoded));
            }
            if let Ok(decoded) = <ClientTypesCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::ClientTypes(decoded));
            }
            if let Ok(decoded) = <CommitmentsCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::Commitments(decoded));
            }
            if let Ok(decoded) = <ConnectionOpenAckCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::ConnectionOpenAck(decoded));
            }
            if let Ok(decoded) = <ConnectionOpenConfirmCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::ConnectionOpenConfirm(decoded));
            }
            if let Ok(decoded) = <ConnectionOpenInitCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::ConnectionOpenInit(decoded));
            }
            if let Ok(decoded) = <ConnectionOpenTryCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::ConnectionOpenTry(decoded));
            }
            if let Ok(decoded) = <ConnectionsCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::Connections(decoded));
            }
            if let Ok(decoded) = <CreateClientCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::CreateClient(decoded));
            }
            if let Ok(decoded) = <ExpectedTimePerBlockCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::ExpectedTimePerBlock(decoded));
            }
            if let Ok(decoded) = <GetChannelCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::GetChannel(decoded));
            }
            if let Ok(decoded) = <GetClientStateCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::GetClientState(decoded));
            }
            if let Ok(decoded) = <GetConnectionCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::GetConnection(decoded));
            }
            if let Ok(decoded) = <GetConsensusStateCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::GetConsensusState(decoded));
            }
            if let Ok(decoded) = <GetExpectedTimePerBlockCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::GetExpectedTimePerBlock(decoded));
            }
            if let Ok(decoded) = <GetHashedPacketAcknowledgementCommitmentCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::GetHashedPacketAcknowledgementCommitment(decoded));
            }
            if let Ok(decoded) = <GetHashedPacketCommitmentCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::GetHashedPacketCommitment(decoded));
            }
            if let Ok(decoded) = <GetNextSequenceSendCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::GetNextSequenceSend(decoded));
            }
            if let Ok(decoded) = <HasPacketReceiptCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::HasPacketReceipt(decoded));
            }
            if let Ok(decoded) = <NextSequenceAcksCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::NextSequenceAcks(decoded));
            }
            if let Ok(decoded) = <NextSequenceRecvsCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::NextSequenceRecvs(decoded));
            }
            if let Ok(decoded) = <NextSequenceSendsCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::NextSequenceSends(decoded));
            }
            if let Ok(decoded) = <PacketReceiptsCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::PacketReceipts(decoded));
            }
            if let Ok(decoded) = <PortCapabilityPathCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::PortCapabilityPath(decoded));
            }
            if let Ok(decoded) = <RecvPacketCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::RecvPacket(decoded));
            }
            if let Ok(decoded) = <RegisterClientCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::RegisterClient(decoded));
            }
            if let Ok(decoded) = <SendPacketCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::SendPacket(decoded));
            }
            if let Ok(decoded) = <SetExpectedTimePerBlockCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::SetExpectedTimePerBlock(decoded));
            }
            if let Ok(decoded) = <TimeoutPacketCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::TimeoutPacket(decoded));
            }
            if let Ok(decoded) = <UpdateClientCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::UpdateClient(decoded));
            }
            if let Ok(decoded) = <WriteAcknowledgementCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::WriteAcknowledgement(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for IBCHandlerCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                Self::AcknowledgePacket(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::BindPort(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::Capabilities(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ChannelCapabilityPath(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ChannelCloseConfirm(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ChannelCloseInit(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ChannelOpenAck(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ChannelOpenConfirm(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ChannelOpenInit(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ChannelOpenTry(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::Channels(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ClientImpls(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ClientRegistry(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ClientTypes(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::Commitments(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ConnectionOpenAck(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ConnectionOpenConfirm(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ConnectionOpenInit(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ConnectionOpenTry(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::Connections(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::CreateClient(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ExpectedTimePerBlock(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetChannel(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetClientState(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetConnection(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetConsensusState(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetExpectedTimePerBlock(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetHashedPacketAcknowledgementCommitment(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetHashedPacketCommitment(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetNextSequenceSend(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::HasPacketReceipt(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NextSequenceAcks(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NextSequenceRecvs(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NextSequenceSends(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::PacketReceipts(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::PortCapabilityPath(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RecvPacket(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RegisterClient(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::SendPacket(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::SetExpectedTimePerBlock(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::TimeoutPacket(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::UpdateClient(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::WriteAcknowledgement(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
            }
        }
    }
    impl ::core::fmt::Display for IBCHandlerCalls {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::AcknowledgePacket(element) => ::core::fmt::Display::fmt(element, f),
                Self::BindPort(element) => ::core::fmt::Display::fmt(element, f),
                Self::Capabilities(element) => ::core::fmt::Display::fmt(element, f),
                Self::ChannelCapabilityPath(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ChannelCloseConfirm(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ChannelCloseInit(element) => ::core::fmt::Display::fmt(element, f),
                Self::ChannelOpenAck(element) => ::core::fmt::Display::fmt(element, f),
                Self::ChannelOpenConfirm(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ChannelOpenInit(element) => ::core::fmt::Display::fmt(element, f),
                Self::ChannelOpenTry(element) => ::core::fmt::Display::fmt(element, f),
                Self::Channels(element) => ::core::fmt::Display::fmt(element, f),
                Self::ClientImpls(element) => ::core::fmt::Display::fmt(element, f),
                Self::ClientRegistry(element) => ::core::fmt::Display::fmt(element, f),
                Self::ClientTypes(element) => ::core::fmt::Display::fmt(element, f),
                Self::Commitments(element) => ::core::fmt::Display::fmt(element, f),
                Self::ConnectionOpenAck(element) => ::core::fmt::Display::fmt(element, f),
                Self::ConnectionOpenConfirm(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ConnectionOpenInit(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ConnectionOpenTry(element) => ::core::fmt::Display::fmt(element, f),
                Self::Connections(element) => ::core::fmt::Display::fmt(element, f),
                Self::CreateClient(element) => ::core::fmt::Display::fmt(element, f),
                Self::ExpectedTimePerBlock(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::GetChannel(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetClientState(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetConnection(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetConsensusState(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetExpectedTimePerBlock(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::GetHashedPacketAcknowledgementCommitment(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::GetHashedPacketCommitment(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::GetNextSequenceSend(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::HasPacketReceipt(element) => ::core::fmt::Display::fmt(element, f),
                Self::NextSequenceAcks(element) => ::core::fmt::Display::fmt(element, f),
                Self::NextSequenceRecvs(element) => ::core::fmt::Display::fmt(element, f),
                Self::NextSequenceSends(element) => ::core::fmt::Display::fmt(element, f),
                Self::PacketReceipts(element) => ::core::fmt::Display::fmt(element, f),
                Self::PortCapabilityPath(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::RecvPacket(element) => ::core::fmt::Display::fmt(element, f),
                Self::RegisterClient(element) => ::core::fmt::Display::fmt(element, f),
                Self::SendPacket(element) => ::core::fmt::Display::fmt(element, f),
                Self::SetExpectedTimePerBlock(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::TimeoutPacket(element) => ::core::fmt::Display::fmt(element, f),
                Self::UpdateClient(element) => ::core::fmt::Display::fmt(element, f),
                Self::WriteAcknowledgement(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
            }
        }
    }
    impl ::core::convert::From<AcknowledgePacketCall> for IBCHandlerCalls {
        fn from(value: AcknowledgePacketCall) -> Self {
            Self::AcknowledgePacket(value)
        }
    }
    impl ::core::convert::From<BindPortCall> for IBCHandlerCalls {
        fn from(value: BindPortCall) -> Self {
            Self::BindPort(value)
        }
    }
    impl ::core::convert::From<CapabilitiesCall> for IBCHandlerCalls {
        fn from(value: CapabilitiesCall) -> Self {
            Self::Capabilities(value)
        }
    }
    impl ::core::convert::From<ChannelCapabilityPathCall> for IBCHandlerCalls {
        fn from(value: ChannelCapabilityPathCall) -> Self {
            Self::ChannelCapabilityPath(value)
        }
    }
    impl ::core::convert::From<ChannelCloseConfirmCall> for IBCHandlerCalls {
        fn from(value: ChannelCloseConfirmCall) -> Self {
            Self::ChannelCloseConfirm(value)
        }
    }
    impl ::core::convert::From<ChannelCloseInitCall> for IBCHandlerCalls {
        fn from(value: ChannelCloseInitCall) -> Self {
            Self::ChannelCloseInit(value)
        }
    }
    impl ::core::convert::From<ChannelOpenAckCall> for IBCHandlerCalls {
        fn from(value: ChannelOpenAckCall) -> Self {
            Self::ChannelOpenAck(value)
        }
    }
    impl ::core::convert::From<ChannelOpenConfirmCall> for IBCHandlerCalls {
        fn from(value: ChannelOpenConfirmCall) -> Self {
            Self::ChannelOpenConfirm(value)
        }
    }
    impl ::core::convert::From<ChannelOpenInitCall> for IBCHandlerCalls {
        fn from(value: ChannelOpenInitCall) -> Self {
            Self::ChannelOpenInit(value)
        }
    }
    impl ::core::convert::From<ChannelOpenTryCall> for IBCHandlerCalls {
        fn from(value: ChannelOpenTryCall) -> Self {
            Self::ChannelOpenTry(value)
        }
    }
    impl ::core::convert::From<ChannelsCall> for IBCHandlerCalls {
        fn from(value: ChannelsCall) -> Self {
            Self::Channels(value)
        }
    }
    impl ::core::convert::From<ClientImplsCall> for IBCHandlerCalls {
        fn from(value: ClientImplsCall) -> Self {
            Self::ClientImpls(value)
        }
    }
    impl ::core::convert::From<ClientRegistryCall> for IBCHandlerCalls {
        fn from(value: ClientRegistryCall) -> Self {
            Self::ClientRegistry(value)
        }
    }
    impl ::core::convert::From<ClientTypesCall> for IBCHandlerCalls {
        fn from(value: ClientTypesCall) -> Self {
            Self::ClientTypes(value)
        }
    }
    impl ::core::convert::From<CommitmentsCall> for IBCHandlerCalls {
        fn from(value: CommitmentsCall) -> Self {
            Self::Commitments(value)
        }
    }
    impl ::core::convert::From<ConnectionOpenAckCall> for IBCHandlerCalls {
        fn from(value: ConnectionOpenAckCall) -> Self {
            Self::ConnectionOpenAck(value)
        }
    }
    impl ::core::convert::From<ConnectionOpenConfirmCall> for IBCHandlerCalls {
        fn from(value: ConnectionOpenConfirmCall) -> Self {
            Self::ConnectionOpenConfirm(value)
        }
    }
    impl ::core::convert::From<ConnectionOpenInitCall> for IBCHandlerCalls {
        fn from(value: ConnectionOpenInitCall) -> Self {
            Self::ConnectionOpenInit(value)
        }
    }
    impl ::core::convert::From<ConnectionOpenTryCall> for IBCHandlerCalls {
        fn from(value: ConnectionOpenTryCall) -> Self {
            Self::ConnectionOpenTry(value)
        }
    }
    impl ::core::convert::From<ConnectionsCall> for IBCHandlerCalls {
        fn from(value: ConnectionsCall) -> Self {
            Self::Connections(value)
        }
    }
    impl ::core::convert::From<CreateClientCall> for IBCHandlerCalls {
        fn from(value: CreateClientCall) -> Self {
            Self::CreateClient(value)
        }
    }
    impl ::core::convert::From<ExpectedTimePerBlockCall> for IBCHandlerCalls {
        fn from(value: ExpectedTimePerBlockCall) -> Self {
            Self::ExpectedTimePerBlock(value)
        }
    }
    impl ::core::convert::From<GetChannelCall> for IBCHandlerCalls {
        fn from(value: GetChannelCall) -> Self {
            Self::GetChannel(value)
        }
    }
    impl ::core::convert::From<GetClientStateCall> for IBCHandlerCalls {
        fn from(value: GetClientStateCall) -> Self {
            Self::GetClientState(value)
        }
    }
    impl ::core::convert::From<GetConnectionCall> for IBCHandlerCalls {
        fn from(value: GetConnectionCall) -> Self {
            Self::GetConnection(value)
        }
    }
    impl ::core::convert::From<GetConsensusStateCall> for IBCHandlerCalls {
        fn from(value: GetConsensusStateCall) -> Self {
            Self::GetConsensusState(value)
        }
    }
    impl ::core::convert::From<GetExpectedTimePerBlockCall> for IBCHandlerCalls {
        fn from(value: GetExpectedTimePerBlockCall) -> Self {
            Self::GetExpectedTimePerBlock(value)
        }
    }
    impl ::core::convert::From<GetHashedPacketAcknowledgementCommitmentCall>
    for IBCHandlerCalls {
        fn from(value: GetHashedPacketAcknowledgementCommitmentCall) -> Self {
            Self::GetHashedPacketAcknowledgementCommitment(value)
        }
    }
    impl ::core::convert::From<GetHashedPacketCommitmentCall> for IBCHandlerCalls {
        fn from(value: GetHashedPacketCommitmentCall) -> Self {
            Self::GetHashedPacketCommitment(value)
        }
    }
    impl ::core::convert::From<GetNextSequenceSendCall> for IBCHandlerCalls {
        fn from(value: GetNextSequenceSendCall) -> Self {
            Self::GetNextSequenceSend(value)
        }
    }
    impl ::core::convert::From<HasPacketReceiptCall> for IBCHandlerCalls {
        fn from(value: HasPacketReceiptCall) -> Self {
            Self::HasPacketReceipt(value)
        }
    }
    impl ::core::convert::From<NextSequenceAcksCall> for IBCHandlerCalls {
        fn from(value: NextSequenceAcksCall) -> Self {
            Self::NextSequenceAcks(value)
        }
    }
    impl ::core::convert::From<NextSequenceRecvsCall> for IBCHandlerCalls {
        fn from(value: NextSequenceRecvsCall) -> Self {
            Self::NextSequenceRecvs(value)
        }
    }
    impl ::core::convert::From<NextSequenceSendsCall> for IBCHandlerCalls {
        fn from(value: NextSequenceSendsCall) -> Self {
            Self::NextSequenceSends(value)
        }
    }
    impl ::core::convert::From<PacketReceiptsCall> for IBCHandlerCalls {
        fn from(value: PacketReceiptsCall) -> Self {
            Self::PacketReceipts(value)
        }
    }
    impl ::core::convert::From<PortCapabilityPathCall> for IBCHandlerCalls {
        fn from(value: PortCapabilityPathCall) -> Self {
            Self::PortCapabilityPath(value)
        }
    }
    impl ::core::convert::From<RecvPacketCall> for IBCHandlerCalls {
        fn from(value: RecvPacketCall) -> Self {
            Self::RecvPacket(value)
        }
    }
    impl ::core::convert::From<RegisterClientCall> for IBCHandlerCalls {
        fn from(value: RegisterClientCall) -> Self {
            Self::RegisterClient(value)
        }
    }
    impl ::core::convert::From<SendPacketCall> for IBCHandlerCalls {
        fn from(value: SendPacketCall) -> Self {
            Self::SendPacket(value)
        }
    }
    impl ::core::convert::From<SetExpectedTimePerBlockCall> for IBCHandlerCalls {
        fn from(value: SetExpectedTimePerBlockCall) -> Self {
            Self::SetExpectedTimePerBlock(value)
        }
    }
    impl ::core::convert::From<TimeoutPacketCall> for IBCHandlerCalls {
        fn from(value: TimeoutPacketCall) -> Self {
            Self::TimeoutPacket(value)
        }
    }
    impl ::core::convert::From<UpdateClientCall> for IBCHandlerCalls {
        fn from(value: UpdateClientCall) -> Self {
            Self::UpdateClient(value)
        }
    }
    impl ::core::convert::From<WriteAcknowledgementCall> for IBCHandlerCalls {
        fn from(value: WriteAcknowledgementCall) -> Self {
            Self::WriteAcknowledgement(value)
        }
    }
    ///Container type for all return fields from the `capabilities` function with signature `capabilities(bytes,uint256)` and selector `0xdd5b9f4d`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct CapabilitiesReturn(pub ::ethers::core::types::Address);
    ///Container type for all return fields from the `channelCapabilityPath` function with signature `channelCapabilityPath(string,string)` and selector `0x3bc3339f`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct ChannelCapabilityPathReturn(pub ::ethers::core::types::Bytes);
    ///Container type for all return fields from the `channelOpenInit` function with signature `channelOpenInit((string,(uint8,uint8,(string,string),string[],string)))` and selector `0xdd3469fc`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct ChannelOpenInitReturn {
        pub channel_id: ::std::string::String,
    }
    ///Container type for all return fields from the `channelOpenTry` function with signature `channelOpenTry((string,(uint8,uint8,(string,string),string[],string),string,bytes,(uint64,uint64)))` and selector `0x11b88a15`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct ChannelOpenTryReturn {
        pub channel_id: ::std::string::String,
    }
    ///Container type for all return fields from the `channels` function with signature `channels(string,string)` and selector `0x5b3de260`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
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
        Hash
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
        Hash
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
        Hash
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
        Hash
    )]
    pub struct CommitmentsReturn(pub [u8; 32]);
    ///Container type for all return fields from the `connectionOpenInit` function with signature `connectionOpenInit((string,(string,string,(bytes)),uint64))` and selector `0x01c6400f`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct ConnectionOpenInitReturn {
        pub connection_id: ::std::string::String,
    }
    ///Container type for all return fields from the `connectionOpenTry` function with signature `connectionOpenTry(((string,string,(bytes)),uint64,string,bytes,(string,string[])[],bytes,bytes,bytes,(uint64,uint64),(uint64,uint64)))` and selector `0x04f68e5c`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct ConnectionOpenTryReturn {
        pub connection_id: ::std::string::String,
    }
    ///Container type for all return fields from the `connections` function with signature `connections(string)` and selector `0x31973f00`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
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
        Hash
    )]
    pub struct CreateClientReturn {
        pub client_id: ::std::string::String,
    }
    ///Container type for all return fields from the `expectedTimePerBlock` function with signature `expectedTimePerBlock()` and selector `0xd31407fe`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct ExpectedTimePerBlockReturn(pub u64);
    ///Container type for all return fields from the `getChannel` function with signature `getChannel(string,string)` and selector `0x3000217a`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct GetChannelReturn(pub IbcCoreChannelV1ChannelData, pub bool);
    ///Container type for all return fields from the `getClientState` function with signature `getClientState(string)` and selector `0x76c81c42`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct GetClientStateReturn(pub ::ethers::core::types::Bytes, pub bool);
    ///Container type for all return fields from the `getConnection` function with signature `getConnection(string)` and selector `0x27711a69`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct GetConnectionReturn(pub IbcCoreConnectionV1ConnectionEndData, pub bool);
    ///Container type for all return fields from the `getConsensusState` function with signature `getConsensusState(string,(uint64,uint64))` and selector `0x6cf44bf4`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct GetConsensusStateReturn {
        pub consensus_state_bytes: ::ethers::core::types::Bytes,
        pub p1: bool,
    }
    ///Container type for all return fields from the `getExpectedTimePerBlock` function with signature `getExpectedTimePerBlock()` and selector `0xec75d829`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct GetExpectedTimePerBlockReturn(pub u64);
    ///Container type for all return fields from the `getHashedPacketAcknowledgementCommitment` function with signature `getHashedPacketAcknowledgementCommitment(string,string,uint64)` and selector `0x5be164ee`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct GetHashedPacketAcknowledgementCommitmentReturn(pub [u8; 32], pub bool);
    ///Container type for all return fields from the `getHashedPacketCommitment` function with signature `getHashedPacketCommitment(string,string,uint64)` and selector `0x23402a33`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct GetHashedPacketCommitmentReturn(pub [u8; 32], pub bool);
    ///Container type for all return fields from the `getNextSequenceSend` function with signature `getNextSequenceSend(string,string)` and selector `0x582418b6`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct GetNextSequenceSendReturn(pub u64);
    ///Container type for all return fields from the `hasPacketReceipt` function with signature `hasPacketReceipt(string,string,uint64)` and selector `0x5a9afac3`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct HasPacketReceiptReturn(pub bool);
    ///Container type for all return fields from the `nextSequenceAcks` function with signature `nextSequenceAcks(string,string)` and selector `0x1390d28d`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
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
        Hash
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
        Hash
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
        Hash
    )]
    pub struct PacketReceiptsReturn(pub u8);
    ///Container type for all return fields from the `portCapabilityPath` function with signature `portCapabilityPath(string)` and selector `0x2570dae0`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct PortCapabilityPathReturn(pub ::ethers::core::types::Bytes);
}
