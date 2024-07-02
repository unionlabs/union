pub use multicall_3::*;
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
pub mod multicall_3 {
    #[cfg(feature = "providers")]
    #[allow(deprecated)]
    #[cfg(feature = "providers")]
    fn __abi() -> ::ethers::core::abi::Abi {
        ::ethers::core::abi::ethabi::Contract {
            constructor: ::core::option::Option::None,
            functions: ::core::convert::From::from([
                (
                    ::std::borrow::ToOwned::to_owned("aggregate"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("aggregate"),
                        inputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("calls"),
                            kind: ::ethers::core::abi::ethabi::ParamType::Array(
                                ::std::boxed::Box::new(
                                    ::ethers::core::abi::ethabi::ParamType::Tuple(::std::vec![
                                        ::ethers::core::abi::ethabi::ParamType::Address,
                                        ::ethers::core::abi::ethabi::ParamType::Bytes,
                                    ],),
                                ),
                            ),
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("struct Multicall3.Call[]"),
                            ),
                        },],
                        outputs: ::std::vec![
                            ::ethers::core::abi::ethabi::Param {
                                name: ::std::borrow::ToOwned::to_owned("blockNumber"),
                                kind: ::ethers::core::abi::ethabi::ParamType::Uint(256usize,),
                                internal_type: ::core::option::Option::Some(
                                    ::std::borrow::ToOwned::to_owned("uint256"),
                                ),
                            },
                            ::ethers::core::abi::ethabi::Param {
                                name: ::std::borrow::ToOwned::to_owned("returnData"),
                                kind: ::ethers::core::abi::ethabi::ParamType::Array(
                                    ::std::boxed::Box::new(
                                        ::ethers::core::abi::ethabi::ParamType::Bytes,
                                    ),
                                ),
                                internal_type: ::core::option::Option::Some(
                                    ::std::borrow::ToOwned::to_owned("bytes[]"),
                                ),
                            },
                        ],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::Payable,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("aggregate3"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("aggregate3"),
                        inputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("calls"),
                            kind: ::ethers::core::abi::ethabi::ParamType::Array(
                                ::std::boxed::Box::new(
                                    ::ethers::core::abi::ethabi::ParamType::Tuple(::std::vec![
                                        ::ethers::core::abi::ethabi::ParamType::Address,
                                        ::ethers::core::abi::ethabi::ParamType::Bool,
                                        ::ethers::core::abi::ethabi::ParamType::Bytes,
                                    ],),
                                ),
                            ),
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("struct Multicall3.Call3[]",),
                            ),
                        },],
                        outputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("returnData"),
                            kind: ::ethers::core::abi::ethabi::ParamType::Array(
                                ::std::boxed::Box::new(
                                    ::ethers::core::abi::ethabi::ParamType::Tuple(::std::vec![
                                        ::ethers::core::abi::ethabi::ParamType::Bool,
                                        ::ethers::core::abi::ethabi::ParamType::Bytes,
                                    ],),
                                ),
                            ),
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("struct Multicall3.Result[]",),
                            ),
                        },],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::Payable,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("aggregate3Value"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("aggregate3Value"),
                        inputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("calls"),
                            kind: ::ethers::core::abi::ethabi::ParamType::Array(
                                ::std::boxed::Box::new(
                                    ::ethers::core::abi::ethabi::ParamType::Tuple(::std::vec![
                                        ::ethers::core::abi::ethabi::ParamType::Address,
                                        ::ethers::core::abi::ethabi::ParamType::Bool,
                                        ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                        ::ethers::core::abi::ethabi::ParamType::Bytes,
                                    ],),
                                ),
                            ),
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("struct Multicall3.Call3Value[]",),
                            ),
                        },],
                        outputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("returnData"),
                            kind: ::ethers::core::abi::ethabi::ParamType::Array(
                                ::std::boxed::Box::new(
                                    ::ethers::core::abi::ethabi::ParamType::Tuple(::std::vec![
                                        ::ethers::core::abi::ethabi::ParamType::Bool,
                                        ::ethers::core::abi::ethabi::ParamType::Bytes,
                                    ],),
                                ),
                            ),
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("struct Multicall3.Result[]",),
                            ),
                        },],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::Payable,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("blockAndAggregate"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("blockAndAggregate"),
                        inputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("calls"),
                            kind: ::ethers::core::abi::ethabi::ParamType::Array(
                                ::std::boxed::Box::new(
                                    ::ethers::core::abi::ethabi::ParamType::Tuple(::std::vec![
                                        ::ethers::core::abi::ethabi::ParamType::Address,
                                        ::ethers::core::abi::ethabi::ParamType::Bytes,
                                    ],),
                                ),
                            ),
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("struct Multicall3.Call[]"),
                            ),
                        },],
                        outputs: ::std::vec![
                            ::ethers::core::abi::ethabi::Param {
                                name: ::std::borrow::ToOwned::to_owned("blockNumber"),
                                kind: ::ethers::core::abi::ethabi::ParamType::Uint(256usize,),
                                internal_type: ::core::option::Option::Some(
                                    ::std::borrow::ToOwned::to_owned("uint256"),
                                ),
                            },
                            ::ethers::core::abi::ethabi::Param {
                                name: ::std::borrow::ToOwned::to_owned("blockHash"),
                                kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize,),
                                internal_type: ::core::option::Option::Some(
                                    ::std::borrow::ToOwned::to_owned("bytes32"),
                                ),
                            },
                            ::ethers::core::abi::ethabi::Param {
                                name: ::std::borrow::ToOwned::to_owned("returnData"),
                                kind: ::ethers::core::abi::ethabi::ParamType::Array(
                                    ::std::boxed::Box::new(
                                        ::ethers::core::abi::ethabi::ParamType::Tuple(::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Bool,
                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                        ],),
                                    ),
                                ),
                                internal_type: ::core::option::Option::Some(
                                    ::std::borrow::ToOwned::to_owned("struct Multicall3.Result[]",),
                                ),
                            },
                        ],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::Payable,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getBasefee"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("getBasefee"),
                        inputs: ::std::vec![],
                        outputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("basefee"),
                            kind: ::ethers::core::abi::ethabi::ParamType::Uint(256usize,),
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("uint256"),
                            ),
                        },],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getBlockHash"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("getBlockHash"),
                        inputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("blockNumber"),
                            kind: ::ethers::core::abi::ethabi::ParamType::Uint(256usize,),
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("uint256"),
                            ),
                        },],
                        outputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("blockHash"),
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
                    ::std::borrow::ToOwned::to_owned("getBlockNumber"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("getBlockNumber"),
                        inputs: ::std::vec![],
                        outputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("blockNumber"),
                            kind: ::ethers::core::abi::ethabi::ParamType::Uint(256usize,),
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("uint256"),
                            ),
                        },],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getChainId"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("getChainId"),
                        inputs: ::std::vec![],
                        outputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("chainid"),
                            kind: ::ethers::core::abi::ethabi::ParamType::Uint(256usize,),
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("uint256"),
                            ),
                        },],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getCurrentBlockCoinbase"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("getCurrentBlockCoinbase",),
                        inputs: ::std::vec![],
                        outputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("coinbase"),
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
                    ::std::borrow::ToOwned::to_owned("getCurrentBlockDifficulty"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("getCurrentBlockDifficulty",),
                        inputs: ::std::vec![],
                        outputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("difficulty"),
                            kind: ::ethers::core::abi::ethabi::ParamType::Uint(256usize,),
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("uint256"),
                            ),
                        },],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getCurrentBlockGasLimit"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("getCurrentBlockGasLimit",),
                        inputs: ::std::vec![],
                        outputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("gaslimit"),
                            kind: ::ethers::core::abi::ethabi::ParamType::Uint(256usize,),
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("uint256"),
                            ),
                        },],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getCurrentBlockTimestamp"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("getCurrentBlockTimestamp",),
                        inputs: ::std::vec![],
                        outputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("timestamp"),
                            kind: ::ethers::core::abi::ethabi::ParamType::Uint(256usize,),
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("uint256"),
                            ),
                        },],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getEthBalance"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("getEthBalance"),
                        inputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("addr"),
                            kind: ::ethers::core::abi::ethabi::ParamType::Address,
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("address"),
                            ),
                        },],
                        outputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("balance"),
                            kind: ::ethers::core::abi::ethabi::ParamType::Uint(256usize,),
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("uint256"),
                            ),
                        },],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getLastBlockHash"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("getLastBlockHash"),
                        inputs: ::std::vec![],
                        outputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("blockHash"),
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
                    ::std::borrow::ToOwned::to_owned("tryAggregate"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("tryAggregate"),
                        inputs: ::std::vec![
                            ::ethers::core::abi::ethabi::Param {
                                name: ::std::borrow::ToOwned::to_owned("requireSuccess"),
                                kind: ::ethers::core::abi::ethabi::ParamType::Bool,
                                internal_type: ::core::option::Option::Some(
                                    ::std::borrow::ToOwned::to_owned("bool"),
                                ),
                            },
                            ::ethers::core::abi::ethabi::Param {
                                name: ::std::borrow::ToOwned::to_owned("calls"),
                                kind: ::ethers::core::abi::ethabi::ParamType::Array(
                                    ::std::boxed::Box::new(
                                        ::ethers::core::abi::ethabi::ParamType::Tuple(::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Address,
                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                        ],),
                                    ),
                                ),
                                internal_type: ::core::option::Option::Some(
                                    ::std::borrow::ToOwned::to_owned("struct Multicall3.Call[]"),
                                ),
                            },
                        ],
                        outputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("returnData"),
                            kind: ::ethers::core::abi::ethabi::ParamType::Array(
                                ::std::boxed::Box::new(
                                    ::ethers::core::abi::ethabi::ParamType::Tuple(::std::vec![
                                        ::ethers::core::abi::ethabi::ParamType::Bool,
                                        ::ethers::core::abi::ethabi::ParamType::Bytes,
                                    ],),
                                ),
                            ),
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("struct Multicall3.Result[]",),
                            ),
                        },],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::Payable,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("tryBlockAndAggregate"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("tryBlockAndAggregate",),
                        inputs: ::std::vec![
                            ::ethers::core::abi::ethabi::Param {
                                name: ::std::borrow::ToOwned::to_owned("requireSuccess"),
                                kind: ::ethers::core::abi::ethabi::ParamType::Bool,
                                internal_type: ::core::option::Option::Some(
                                    ::std::borrow::ToOwned::to_owned("bool"),
                                ),
                            },
                            ::ethers::core::abi::ethabi::Param {
                                name: ::std::borrow::ToOwned::to_owned("calls"),
                                kind: ::ethers::core::abi::ethabi::ParamType::Array(
                                    ::std::boxed::Box::new(
                                        ::ethers::core::abi::ethabi::ParamType::Tuple(::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Address,
                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                        ],),
                                    ),
                                ),
                                internal_type: ::core::option::Option::Some(
                                    ::std::borrow::ToOwned::to_owned("struct Multicall3.Call[]"),
                                ),
                            },
                        ],
                        outputs: ::std::vec![
                            ::ethers::core::abi::ethabi::Param {
                                name: ::std::borrow::ToOwned::to_owned("blockNumber"),
                                kind: ::ethers::core::abi::ethabi::ParamType::Uint(256usize,),
                                internal_type: ::core::option::Option::Some(
                                    ::std::borrow::ToOwned::to_owned("uint256"),
                                ),
                            },
                            ::ethers::core::abi::ethabi::Param {
                                name: ::std::borrow::ToOwned::to_owned("blockHash"),
                                kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize,),
                                internal_type: ::core::option::Option::Some(
                                    ::std::borrow::ToOwned::to_owned("bytes32"),
                                ),
                            },
                            ::ethers::core::abi::ethabi::Param {
                                name: ::std::borrow::ToOwned::to_owned("returnData"),
                                kind: ::ethers::core::abi::ethabi::ParamType::Array(
                                    ::std::boxed::Box::new(
                                        ::ethers::core::abi::ethabi::ParamType::Tuple(::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Bool,
                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                        ],),
                                    ),
                                ),
                                internal_type: ::core::option::Option::Some(
                                    ::std::borrow::ToOwned::to_owned("struct Multicall3.Result[]",),
                                ),
                            },
                        ],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::Payable,
                    },],
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
    pub static MULTICALL3_ABI: ::ethers::contract::Lazy<::ethers::core::abi::Abi> =
        ::ethers::contract::Lazy::new(__abi);
    #[cfg(feature = "providers")]
    pub struct Multicall3<M>(::ethers::contract::Contract<M>);
    #[cfg(feature = "providers")]
    impl<M> ::core::clone::Clone for Multicall3<M> {
        fn clone(&self) -> Self {
            Self(::core::clone::Clone::clone(&self.0))
        }
    }
    #[cfg(feature = "providers")]
    impl<M> ::core::ops::Deref for Multicall3<M> {
        type Target = ::ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    #[cfg(feature = "providers")]
    impl<M> ::core::ops::DerefMut for Multicall3<M> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    #[cfg(feature = "providers")]
    impl<M> ::core::fmt::Debug for Multicall3<M> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple(::core::stringify!(Multicall3))
                .field(&self.address())
                .finish()
        }
    }
    #[cfg(feature = "providers")]
    impl<M: ::ethers::providers::Middleware> Multicall3<M> {
        /// Creates a new contract instance with the specified `ethers` client at
        /// `address`. The contract derefs to a `ethers::Contract` object.
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(::ethers::contract::Contract::new(
                address.into(),
                MULTICALL3_ABI.clone(),
                client,
            ))
        }
        ///Calls the contract's `aggregate` (0x252dba42) function
        pub fn aggregate(
            &self,
            calls: ::std::vec::Vec<Call>,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            (
                ::ethers::core::types::U256,
                ::std::vec::Vec<::ethers::core::types::Bytes>,
            ),
        > {
            self.0
                .method_hash([37, 45, 186, 66], calls)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `aggregate3` (0x82ad56cb) function
        pub fn aggregate_3(
            &self,
            calls: ::std::vec::Vec<Call3>,
        ) -> ::ethers::contract::builders::ContractCall<M, ::std::vec::Vec<Result>> {
            self.0
                .method_hash([130, 173, 86, 203], calls)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `aggregate3Value` (0x174dea71) function
        pub fn aggregate_3_value(
            &self,
            calls: ::std::vec::Vec<Call3Value>,
        ) -> ::ethers::contract::builders::ContractCall<M, ::std::vec::Vec<Result>> {
            self.0
                .method_hash([23, 77, 234, 113], calls)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `blockAndAggregate` (0xc3077fa9) function
        pub fn block_and_aggregate(
            &self,
            calls: ::std::vec::Vec<Call>,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            (
                ::ethers::core::types::U256,
                [u8; 32],
                ::std::vec::Vec<Result>,
            ),
        > {
            self.0
                .method_hash([195, 7, 127, 169], calls)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getBasefee` (0x3e64a696) function
        pub fn get_basefee(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::U256> {
            self.0
                .method_hash([62, 100, 166, 150], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getBlockHash` (0xee82ac5e) function
        pub fn get_block_hash(
            &self,
            block_number: ::ethers::core::types::U256,
        ) -> ::ethers::contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([238, 130, 172, 94], block_number)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getBlockNumber` (0x42cbb15c) function
        pub fn get_block_number(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::U256> {
            self.0
                .method_hash([66, 203, 177, 92], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getChainId` (0x3408e470) function
        pub fn get_chain_id(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::U256> {
            self.0
                .method_hash([52, 8, 228, 112], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getCurrentBlockCoinbase` (0xa8b0574e) function
        pub fn get_current_block_coinbase(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::Address> {
            self.0
                .method_hash([168, 176, 87, 78], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getCurrentBlockDifficulty` (0x72425d9d) function
        pub fn get_current_block_difficulty(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::U256> {
            self.0
                .method_hash([114, 66, 93, 157], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getCurrentBlockGasLimit` (0x86d516e8) function
        pub fn get_current_block_gas_limit(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::U256> {
            self.0
                .method_hash([134, 213, 22, 232], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getCurrentBlockTimestamp` (0x0f28c97d) function
        pub fn get_current_block_timestamp(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::U256> {
            self.0
                .method_hash([15, 40, 201, 125], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getEthBalance` (0x4d2301cc) function
        pub fn get_eth_balance(
            &self,
            addr: ::ethers::core::types::Address,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::U256> {
            self.0
                .method_hash([77, 35, 1, 204], addr)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getLastBlockHash` (0x27e86d6e) function
        pub fn get_last_block_hash(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([39, 232, 109, 110], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `tryAggregate` (0xbce38bd7) function
        pub fn try_aggregate(
            &self,
            require_success: bool,
            calls: ::std::vec::Vec<Call>,
        ) -> ::ethers::contract::builders::ContractCall<M, ::std::vec::Vec<Result>> {
            self.0
                .method_hash([188, 227, 139, 215], (require_success, calls))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `tryBlockAndAggregate` (0x399542e9) function
        pub fn try_block_and_aggregate(
            &self,
            require_success: bool,
            calls: ::std::vec::Vec<Call>,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            (
                ::ethers::core::types::U256,
                [u8; 32],
                ::std::vec::Vec<Result>,
            ),
        > {
            self.0
                .method_hash([57, 149, 66, 233], (require_success, calls))
                .expect("method not found (this should never happen)")
        }
    }
    #[cfg(feature = "providers")]
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>> for Multicall3<M> {
        fn from(contract: ::ethers::contract::Contract<M>) -> Self {
            Self::new(contract.address(), contract.client())
        }
    }
    ///Container type for all input parameters for the `aggregate` function with signature `aggregate((address,bytes)[])` and selector `0x252dba42`
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
    #[ethcall(name = "aggregate", abi = "aggregate((address,bytes)[])")]
    pub struct AggregateCall {
        pub calls: ::std::vec::Vec<Call>,
    }
    ///Container type for all input parameters for the `aggregate3` function with signature `aggregate3((address,bool,bytes)[])` and selector `0x82ad56cb`
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
    #[ethcall(name = "aggregate3", abi = "aggregate3((address,bool,bytes)[])")]
    pub struct Aggregate3Call {
        pub calls: ::std::vec::Vec<Call3>,
    }
    ///Container type for all input parameters for the `aggregate3Value` function with signature `aggregate3Value((address,bool,uint256,bytes)[])` and selector `0x174dea71`
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
    #[ethcall(
        name = "aggregate3Value",
        abi = "aggregate3Value((address,bool,uint256,bytes)[])"
    )]
    pub struct Aggregate3ValueCall {
        pub calls: ::std::vec::Vec<Call3Value>,
    }
    ///Container type for all input parameters for the `blockAndAggregate` function with signature `blockAndAggregate((address,bytes)[])` and selector `0xc3077fa9`
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
    #[ethcall(
        name = "blockAndAggregate",
        abi = "blockAndAggregate((address,bytes)[])"
    )]
    pub struct BlockAndAggregateCall {
        pub calls: ::std::vec::Vec<Call>,
    }
    ///Container type for all input parameters for the `getBasefee` function with signature `getBasefee()` and selector `0x3e64a696`
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
    #[ethcall(name = "getBasefee", abi = "getBasefee()")]
    pub struct GetBasefeeCall;
    ///Container type for all input parameters for the `getBlockHash` function with signature `getBlockHash(uint256)` and selector `0xee82ac5e`
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
    #[ethcall(name = "getBlockHash", abi = "getBlockHash(uint256)")]
    pub struct GetBlockHashCall {
        pub block_number: ::ethers::core::types::U256,
    }
    ///Container type for all input parameters for the `getBlockNumber` function with signature `getBlockNumber()` and selector `0x42cbb15c`
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
    #[ethcall(name = "getBlockNumber", abi = "getBlockNumber()")]
    pub struct GetBlockNumberCall;
    ///Container type for all input parameters for the `getChainId` function with signature `getChainId()` and selector `0x3408e470`
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
    #[ethcall(name = "getChainId", abi = "getChainId()")]
    pub struct GetChainIdCall;
    ///Container type for all input parameters for the `getCurrentBlockCoinbase` function with signature `getCurrentBlockCoinbase()` and selector `0xa8b0574e`
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
    #[ethcall(name = "getCurrentBlockCoinbase", abi = "getCurrentBlockCoinbase()")]
    pub struct GetCurrentBlockCoinbaseCall;
    ///Container type for all input parameters for the `getCurrentBlockDifficulty` function with signature `getCurrentBlockDifficulty()` and selector `0x72425d9d`
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
    #[ethcall(
        name = "getCurrentBlockDifficulty",
        abi = "getCurrentBlockDifficulty()"
    )]
    pub struct GetCurrentBlockDifficultyCall;
    ///Container type for all input parameters for the `getCurrentBlockGasLimit` function with signature `getCurrentBlockGasLimit()` and selector `0x86d516e8`
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
    #[ethcall(name = "getCurrentBlockGasLimit", abi = "getCurrentBlockGasLimit()")]
    pub struct GetCurrentBlockGasLimitCall;
    ///Container type for all input parameters for the `getCurrentBlockTimestamp` function with signature `getCurrentBlockTimestamp()` and selector `0x0f28c97d`
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
    #[ethcall(name = "getCurrentBlockTimestamp", abi = "getCurrentBlockTimestamp()")]
    pub struct GetCurrentBlockTimestampCall;
    ///Container type for all input parameters for the `getEthBalance` function with signature `getEthBalance(address)` and selector `0x4d2301cc`
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
    #[ethcall(name = "getEthBalance", abi = "getEthBalance(address)")]
    pub struct GetEthBalanceCall {
        pub addr: ::ethers::core::types::Address,
    }
    ///Container type for all input parameters for the `getLastBlockHash` function with signature `getLastBlockHash()` and selector `0x27e86d6e`
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
    #[ethcall(name = "getLastBlockHash", abi = "getLastBlockHash()")]
    pub struct GetLastBlockHashCall;
    ///Container type for all input parameters for the `tryAggregate` function with signature `tryAggregate(bool,(address,bytes)[])` and selector `0xbce38bd7`
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
    #[ethcall(name = "tryAggregate", abi = "tryAggregate(bool,(address,bytes)[])")]
    pub struct TryAggregateCall {
        pub require_success: bool,
        pub calls: ::std::vec::Vec<Call>,
    }
    ///Container type for all input parameters for the `tryBlockAndAggregate` function with signature `tryBlockAndAggregate(bool,(address,bytes)[])` and selector `0x399542e9`
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
    #[ethcall(
        name = "tryBlockAndAggregate",
        abi = "tryBlockAndAggregate(bool,(address,bytes)[])"
    )]
    pub struct TryBlockAndAggregateCall {
        pub require_success: bool,
        pub calls: ::std::vec::Vec<Call>,
    }
    ///Container type for all of the contract's call
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum Multicall3Calls {
        Aggregate(AggregateCall),
        Aggregate3(Aggregate3Call),
        Aggregate3Value(Aggregate3ValueCall),
        BlockAndAggregate(BlockAndAggregateCall),
        GetBasefee(GetBasefeeCall),
        GetBlockHash(GetBlockHashCall),
        GetBlockNumber(GetBlockNumberCall),
        GetChainId(GetChainIdCall),
        GetCurrentBlockCoinbase(GetCurrentBlockCoinbaseCall),
        GetCurrentBlockDifficulty(GetCurrentBlockDifficultyCall),
        GetCurrentBlockGasLimit(GetCurrentBlockGasLimitCall),
        GetCurrentBlockTimestamp(GetCurrentBlockTimestampCall),
        GetEthBalance(GetEthBalanceCall),
        GetLastBlockHash(GetLastBlockHashCall),
        TryAggregate(TryAggregateCall),
        TryBlockAndAggregate(TryBlockAndAggregateCall),
    }
    impl ::ethers::core::abi::AbiDecode for Multicall3Calls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded) = <AggregateCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Aggregate(decoded));
            }
            if let Ok(decoded) = <Aggregate3Call as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Aggregate3(decoded));
            }
            if let Ok(decoded) =
                <Aggregate3ValueCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::Aggregate3Value(decoded));
            }
            if let Ok(decoded) =
                <BlockAndAggregateCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::BlockAndAggregate(decoded));
            }
            if let Ok(decoded) = <GetBasefeeCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::GetBasefee(decoded));
            }
            if let Ok(decoded) = <GetBlockHashCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::GetBlockHash(decoded));
            }
            if let Ok(decoded) =
                <GetBlockNumberCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::GetBlockNumber(decoded));
            }
            if let Ok(decoded) = <GetChainIdCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::GetChainId(decoded));
            }
            if let Ok(decoded) =
                <GetCurrentBlockCoinbaseCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::GetCurrentBlockCoinbase(decoded));
            }
            if let Ok(decoded) =
                <GetCurrentBlockDifficultyCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::GetCurrentBlockDifficulty(decoded));
            }
            if let Ok(decoded) =
                <GetCurrentBlockGasLimitCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::GetCurrentBlockGasLimit(decoded));
            }
            if let Ok(decoded) =
                <GetCurrentBlockTimestampCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::GetCurrentBlockTimestamp(decoded));
            }
            if let Ok(decoded) = <GetEthBalanceCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::GetEthBalance(decoded));
            }
            if let Ok(decoded) =
                <GetLastBlockHashCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::GetLastBlockHash(decoded));
            }
            if let Ok(decoded) = <TryAggregateCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::TryAggregate(decoded));
            }
            if let Ok(decoded) =
                <TryBlockAndAggregateCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::TryBlockAndAggregate(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for Multicall3Calls {
        fn encode(self) -> Vec<u8> {
            match self {
                Self::Aggregate(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::Aggregate3(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::Aggregate3Value(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::BlockAndAggregate(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::GetBasefee(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::GetBlockHash(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::GetBlockNumber(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::GetChainId(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::GetCurrentBlockCoinbase(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetCurrentBlockDifficulty(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetCurrentBlockGasLimit(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetCurrentBlockTimestamp(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetEthBalance(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::GetLastBlockHash(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::TryAggregate(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::TryBlockAndAggregate(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
            }
        }
    }
    impl ::core::fmt::Display for Multicall3Calls {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::Aggregate(element) => ::core::fmt::Display::fmt(element, f),
                Self::Aggregate3(element) => ::core::fmt::Display::fmt(element, f),
                Self::Aggregate3Value(element) => ::core::fmt::Display::fmt(element, f),
                Self::BlockAndAggregate(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetBasefee(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetBlockHash(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetBlockNumber(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetChainId(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetCurrentBlockCoinbase(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetCurrentBlockDifficulty(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetCurrentBlockGasLimit(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetCurrentBlockTimestamp(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetEthBalance(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetLastBlockHash(element) => ::core::fmt::Display::fmt(element, f),
                Self::TryAggregate(element) => ::core::fmt::Display::fmt(element, f),
                Self::TryBlockAndAggregate(element) => ::core::fmt::Display::fmt(element, f),
            }
        }
    }
    impl ::core::convert::From<AggregateCall> for Multicall3Calls {
        fn from(value: AggregateCall) -> Self {
            Self::Aggregate(value)
        }
    }
    impl ::core::convert::From<Aggregate3Call> for Multicall3Calls {
        fn from(value: Aggregate3Call) -> Self {
            Self::Aggregate3(value)
        }
    }
    impl ::core::convert::From<Aggregate3ValueCall> for Multicall3Calls {
        fn from(value: Aggregate3ValueCall) -> Self {
            Self::Aggregate3Value(value)
        }
    }
    impl ::core::convert::From<BlockAndAggregateCall> for Multicall3Calls {
        fn from(value: BlockAndAggregateCall) -> Self {
            Self::BlockAndAggregate(value)
        }
    }
    impl ::core::convert::From<GetBasefeeCall> for Multicall3Calls {
        fn from(value: GetBasefeeCall) -> Self {
            Self::GetBasefee(value)
        }
    }
    impl ::core::convert::From<GetBlockHashCall> for Multicall3Calls {
        fn from(value: GetBlockHashCall) -> Self {
            Self::GetBlockHash(value)
        }
    }
    impl ::core::convert::From<GetBlockNumberCall> for Multicall3Calls {
        fn from(value: GetBlockNumberCall) -> Self {
            Self::GetBlockNumber(value)
        }
    }
    impl ::core::convert::From<GetChainIdCall> for Multicall3Calls {
        fn from(value: GetChainIdCall) -> Self {
            Self::GetChainId(value)
        }
    }
    impl ::core::convert::From<GetCurrentBlockCoinbaseCall> for Multicall3Calls {
        fn from(value: GetCurrentBlockCoinbaseCall) -> Self {
            Self::GetCurrentBlockCoinbase(value)
        }
    }
    impl ::core::convert::From<GetCurrentBlockDifficultyCall> for Multicall3Calls {
        fn from(value: GetCurrentBlockDifficultyCall) -> Self {
            Self::GetCurrentBlockDifficulty(value)
        }
    }
    impl ::core::convert::From<GetCurrentBlockGasLimitCall> for Multicall3Calls {
        fn from(value: GetCurrentBlockGasLimitCall) -> Self {
            Self::GetCurrentBlockGasLimit(value)
        }
    }
    impl ::core::convert::From<GetCurrentBlockTimestampCall> for Multicall3Calls {
        fn from(value: GetCurrentBlockTimestampCall) -> Self {
            Self::GetCurrentBlockTimestamp(value)
        }
    }
    impl ::core::convert::From<GetEthBalanceCall> for Multicall3Calls {
        fn from(value: GetEthBalanceCall) -> Self {
            Self::GetEthBalance(value)
        }
    }
    impl ::core::convert::From<GetLastBlockHashCall> for Multicall3Calls {
        fn from(value: GetLastBlockHashCall) -> Self {
            Self::GetLastBlockHash(value)
        }
    }
    impl ::core::convert::From<TryAggregateCall> for Multicall3Calls {
        fn from(value: TryAggregateCall) -> Self {
            Self::TryAggregate(value)
        }
    }
    impl ::core::convert::From<TryBlockAndAggregateCall> for Multicall3Calls {
        fn from(value: TryBlockAndAggregateCall) -> Self {
            Self::TryBlockAndAggregate(value)
        }
    }
    ///Container type for all return fields from the `aggregate` function with signature `aggregate((address,bytes)[])` and selector `0x252dba42`
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
    pub struct AggregateReturn {
        pub block_number: ::ethers::core::types::U256,
        pub return_data: ::std::vec::Vec<::ethers::core::types::Bytes>,
    }
    ///Container type for all return fields from the `aggregate3` function with signature `aggregate3((address,bool,bytes)[])` and selector `0x82ad56cb`
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
    pub struct Aggregate3Return {
        pub return_data: ::std::vec::Vec<Result>,
    }
    ///Container type for all return fields from the `aggregate3Value` function with signature `aggregate3Value((address,bool,uint256,bytes)[])` and selector `0x174dea71`
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
    pub struct Aggregate3ValueReturn {
        pub return_data: ::std::vec::Vec<Result>,
    }
    ///Container type for all return fields from the `blockAndAggregate` function with signature `blockAndAggregate((address,bytes)[])` and selector `0xc3077fa9`
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
    pub struct BlockAndAggregateReturn {
        pub block_number: ::ethers::core::types::U256,
        pub block_hash: [u8; 32],
        pub return_data: ::std::vec::Vec<Result>,
    }
    ///Container type for all return fields from the `getBasefee` function with signature `getBasefee()` and selector `0x3e64a696`
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
    pub struct GetBasefeeReturn {
        pub basefee: ::ethers::core::types::U256,
    }
    ///Container type for all return fields from the `getBlockHash` function with signature `getBlockHash(uint256)` and selector `0xee82ac5e`
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
    pub struct GetBlockHashReturn {
        pub block_hash: [u8; 32],
    }
    ///Container type for all return fields from the `getBlockNumber` function with signature `getBlockNumber()` and selector `0x42cbb15c`
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
    pub struct GetBlockNumberReturn {
        pub block_number: ::ethers::core::types::U256,
    }
    ///Container type for all return fields from the `getChainId` function with signature `getChainId()` and selector `0x3408e470`
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
    pub struct GetChainIdReturn {
        pub chainid: ::ethers::core::types::U256,
    }
    ///Container type for all return fields from the `getCurrentBlockCoinbase` function with signature `getCurrentBlockCoinbase()` and selector `0xa8b0574e`
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
    pub struct GetCurrentBlockCoinbaseReturn {
        pub coinbase: ::ethers::core::types::Address,
    }
    ///Container type for all return fields from the `getCurrentBlockDifficulty` function with signature `getCurrentBlockDifficulty()` and selector `0x72425d9d`
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
    pub struct GetCurrentBlockDifficultyReturn {
        pub difficulty: ::ethers::core::types::U256,
    }
    ///Container type for all return fields from the `getCurrentBlockGasLimit` function with signature `getCurrentBlockGasLimit()` and selector `0x86d516e8`
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
    pub struct GetCurrentBlockGasLimitReturn {
        pub gaslimit: ::ethers::core::types::U256,
    }
    ///Container type for all return fields from the `getCurrentBlockTimestamp` function with signature `getCurrentBlockTimestamp()` and selector `0x0f28c97d`
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
    pub struct GetCurrentBlockTimestampReturn {
        pub timestamp: ::ethers::core::types::U256,
    }
    ///Container type for all return fields from the `getEthBalance` function with signature `getEthBalance(address)` and selector `0x4d2301cc`
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
    pub struct GetEthBalanceReturn {
        pub balance: ::ethers::core::types::U256,
    }
    ///Container type for all return fields from the `getLastBlockHash` function with signature `getLastBlockHash()` and selector `0x27e86d6e`
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
    pub struct GetLastBlockHashReturn {
        pub block_hash: [u8; 32],
    }
    ///Container type for all return fields from the `tryAggregate` function with signature `tryAggregate(bool,(address,bytes)[])` and selector `0xbce38bd7`
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
    pub struct TryAggregateReturn {
        pub return_data: ::std::vec::Vec<Result>,
    }
    ///Container type for all return fields from the `tryBlockAndAggregate` function with signature `tryBlockAndAggregate(bool,(address,bytes)[])` and selector `0x399542e9`
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
    pub struct TryBlockAndAggregateReturn {
        pub block_number: ::ethers::core::types::U256,
        pub block_hash: [u8; 32],
        pub return_data: ::std::vec::Vec<Result>,
    }
    ///`Call(address,bytes)`
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
    pub struct Call {
        pub target: ::ethers::core::types::Address,
        pub call_data: ::ethers::core::types::Bytes,
    }
    ///`Call3(address,bool,bytes)`
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
    pub struct Call3 {
        pub target: ::ethers::core::types::Address,
        pub allow_failure: bool,
        pub call_data: ::ethers::core::types::Bytes,
    }
    ///`Call3Value(address,bool,uint256,bytes)`
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
    pub struct Call3Value {
        pub target: ::ethers::core::types::Address,
        pub allow_failure: bool,
        pub value: ::ethers::core::types::U256,
        pub call_data: ::ethers::core::types::Bytes,
    }
    ///`Result(bool,bytes)`
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
    pub struct Result {
        pub success: bool,
        pub return_data: ::ethers::core::types::Bytes,
    }
}
