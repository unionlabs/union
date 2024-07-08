pub use multicall::*;
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
pub mod multicall {
    #[cfg(feature = "providers")]
    #[allow(deprecated)]
    #[cfg(feature = "providers")]
    fn __abi() -> ::ethers::core::abi::Abi {
        ::ethers::core::abi::ethabi::Contract {
            constructor: ::core::option::Option::None,
            functions: ::core::convert::From::from([(
                ::std::borrow::ToOwned::to_owned("multicall"),
                ::std::vec![::ethers::core::abi::ethabi::Function {
                    name: ::std::borrow::ToOwned::to_owned("multicall"),
                    inputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                        name: ::std::borrow::ToOwned::to_owned("calls"),
                        kind: ::ethers::core::abi::ethabi::ParamType::Array(
                            ::std::boxed::Box::new(::ethers::core::abi::ethabi::ParamType::Tuple(
                                ::std::vec![
                                    ::ethers::core::abi::ethabi::ParamType::Address,
                                    ::ethers::core::abi::ethabi::ParamType::Bool,
                                    ::ethers::core::abi::ethabi::ParamType::Bytes,
                                ],
                            ),),
                        ),
                        internal_type: ::core::option::Option::Some(
                            ::std::borrow::ToOwned::to_owned("struct Call3[]"),
                        ),
                    },],
                    outputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                        name: ::std::borrow::ToOwned::to_owned("returnData"),
                        kind: ::ethers::core::abi::ethabi::ParamType::Array(
                            ::std::boxed::Box::new(::ethers::core::abi::ethabi::ParamType::Tuple(
                                ::std::vec![
                                    ::ethers::core::abi::ethabi::ParamType::Bool,
                                    ::ethers::core::abi::ethabi::ParamType::Bytes,
                                ],
                            ),),
                        ),
                        internal_type: ::core::option::Option::Some(
                            ::std::borrow::ToOwned::to_owned("struct Result[]"),
                        ),
                    },],
                    constant: ::core::option::Option::None,
                    state_mutability: ::ethers::core::abi::ethabi::StateMutability::Payable,
                },],
            )]),
            events: ::core::convert::From::from([(
                ::std::borrow::ToOwned::to_owned("MulticallResult"),
                ::std::vec![::ethers::core::abi::ethabi::Event {
                    name: ::std::borrow::ToOwned::to_owned("MulticallResult"),
                    inputs: ::std::vec![::ethers::core::abi::ethabi::EventParam {
                        name: ::std::string::String::new(),
                        kind: ::ethers::core::abi::ethabi::ParamType::Array(
                            ::std::boxed::Box::new(::ethers::core::abi::ethabi::ParamType::Tuple(
                                ::std::vec![
                                    ::ethers::core::abi::ethabi::ParamType::Bool,
                                    ::ethers::core::abi::ethabi::ParamType::Bytes,
                                ],
                            ),),
                        ),
                        indexed: false,
                    },],
                    anonymous: false,
                },],
            )]),
            errors: ::std::collections::BTreeMap::new(),
            receive: false,
            fallback: false,
        }
    }
    ///The parsed JSON ABI of the contract.
    #[cfg(feature = "providers")]
    pub static MULTICALL_ABI: ::ethers::contract::Lazy<::ethers::core::abi::Abi> =
        ::ethers::contract::Lazy::new(__abi);
    #[rustfmt::skip]
    #[cfg(feature = "providers")]
    const __BYTECODE: &[u8] = b"`\x80\x80`@R4a\0\x16Wa\x05$\x90\x81a\0\x1C\x829\xF3[`\0\x80\xFD\xFE`@`\x80`@R`\x04\x806\x10\x15a\0\x15W`\0\x80\xFD[`\0\x90\x815`\xE0\x1Cc\xE8\xBB\xF5\xD7\x14a\0,W`\0\x80\xFD[` \x80\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFC6\x01\x12a\x01\xB3W`\x045\x92g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x91\x82\x85\x11a\x01\xB7W6`#\x86\x01\x12\x15a\x01\xB7W\x84`\x04\x015\x92\x83\x11a\x01\xB7W`$\x90`$\x86\x01\x95`$6\x91\x86`\x05\x1B\x01\x01\x11a\x01\xB3Wa\0\xA3\x84a\x037V[\x95\x83[\x85\x81\x10a\0\xF3Wa\0\xEF\x88\x7Fy\x8FY\xB5\xFB\xED\xBCk\x92\xC3f\xAE\xBB\xE4\xEF7\x89V\xA3\xA1\xB9\xFFJ\x1B\xA0v\x0F=\x07R\xA8\x83`@Q\x80a\0\xE0\x84\x82a\x01\xBBV[\x03\x90\xA1`@Q\x91\x82\x91\x82a\x01\xBBV[\x03\x90\xF3[\x88\x83a\0\xFF\x83\x8Ba\x03\xECV[Qa\x01N\x88\x80a\x01\x10\x87\x8D\x8Aa\x04\x05V[\x95a\x018\x82a\x01\x1E\x89a\x04JV[\x92a\x01+\x81\x8B\x01\x8Ba\x04kV[\x93\x90\x91Q\x80\x94\x81\x93a\x04\xBCV[\x03\x92Z\xF1a\x01Da\x04\xCAV[\x83\x85\x01R\x15\x15\x82RV[Q\x91\x015\x17\x15a\x01`W`\x01\x01a\0\xA6V[`d\x85`\x17\x86\x86\x8B\x7F\x08\xC3y\xA0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x85RRR\x7FMulticall3: call failed\0\0\0\0\0\0\0\0\0`DR\xFD[\x82\x80\xFD[P\x80\xFD[` \x80\x82\x01\x90\x80\x83R\x83Q\x80\x92R`@\x92`@\x81\x01\x82`@\x85`\x05\x1B\x84\x01\x01\x96\x01\x94`\0\x90`\0\x93[\x86\x85\x10a\x01\xF6WPPPPPPPP\x90V[\x90\x91\x92\x93\x94\x95\x96\x97\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xC0\x82\x82\x03\x01\x85R\x86\x80\x8AQ\x80Q\x15\x15\x84R\x01Q\x91\x84\x82\x82\x01R\x82Q\x92\x83\x86\x83\x01R\x86[\x84\x81\x10a\x02\x92WPP`\x01\x92\x82\x91\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xE0`\x1F``\x93\x8A\x85\x82\x86\x01\x01R\x01\x16\x01\x01\x9A\x01\x95\x01\x95\x01\x93\x96\x95\x94\x92\x91\x90a\x01\xE4V[\x81\x81\x01\x84\x01Q\x83\x82\x01``\x01R\x8A\x93\x01a\x02AV[\x7FNH{q\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`\0R`A`\x04R`$`\0\xFD[\x90\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xE0`\x1F`@Q\x93\x01\x16\x82\x01\x82\x81\x10g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x17a\x03\x1AW`@RV[a\x02\xA7V[g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x03\x1AW`\x05\x1B` \x01\x90V[\x90a\x03Ia\x03D\x83a\x03\x1FV[a\x02\xD6V[\x82\x81R\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xE0a\x03w\x82\x94a\x03\x1FV[\x01`\0\x80[\x82\x81\x10a\x03\x89WPPPPV[`@\x90\x81Q\x82\x81\x01\x81\x81\x10g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x17a\x03\x1AW` \x93R\x83\x81R\x82``\x81\x83\x01R\x82\x87\x01\x01R\x01a\x03|V[\x7FNH{q\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`\0R`2`\x04R`$`\0\xFD[\x80Q\x82\x10\x15a\x04\0W` \x91`\x05\x1B\x01\x01\x90V[a\x03\xBDV[\x91\x90\x81\x10\x15a\x04\0W`\x05\x1B\x81\x015\x90\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xA1\x816\x03\x01\x82\x12\x15a\x04EW\x01\x90V[`\0\x80\xFD[5s\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x16\x81\x03a\x04EW\x90V[\x905\x90\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xE1\x816\x03\x01\x82\x12\x15a\x04EW\x01\x805\x90g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11a\x04EW` \x01\x91\x816\x03\x83\x13a\x04EWV[\x90\x80\x92\x91\x827\x01`\0\x81R\x90V[=\x15a\x05\x1FW=\x90g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11a\x03\x1AWa\x05\x12` \x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xE0`\x1F\x85\x01\x16\x01a\x02\xD6V[\x91\x82R=`\0` \x84\x01>V[``\x90V";
    /// The bytecode of the contract.
    #[cfg(feature = "providers")]
    pub static MULTICALL_BYTECODE: ::ethers::core::types::Bytes =
        ::ethers::core::types::Bytes::from_static(__BYTECODE);
    #[rustfmt::skip]
    #[cfg(feature = "providers")]
    const __DEPLOYED_BYTECODE: &[u8] = b"`@`\x80`@R`\x04\x806\x10\x15a\0\x15W`\0\x80\xFD[`\0\x90\x815`\xE0\x1Cc\xE8\xBB\xF5\xD7\x14a\0,W`\0\x80\xFD[` \x80\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFC6\x01\x12a\x01\xB3W`\x045\x92g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x91\x82\x85\x11a\x01\xB7W6`#\x86\x01\x12\x15a\x01\xB7W\x84`\x04\x015\x92\x83\x11a\x01\xB7W`$\x90`$\x86\x01\x95`$6\x91\x86`\x05\x1B\x01\x01\x11a\x01\xB3Wa\0\xA3\x84a\x037V[\x95\x83[\x85\x81\x10a\0\xF3Wa\0\xEF\x88\x7Fy\x8FY\xB5\xFB\xED\xBCk\x92\xC3f\xAE\xBB\xE4\xEF7\x89V\xA3\xA1\xB9\xFFJ\x1B\xA0v\x0F=\x07R\xA8\x83`@Q\x80a\0\xE0\x84\x82a\x01\xBBV[\x03\x90\xA1`@Q\x91\x82\x91\x82a\x01\xBBV[\x03\x90\xF3[\x88\x83a\0\xFF\x83\x8Ba\x03\xECV[Qa\x01N\x88\x80a\x01\x10\x87\x8D\x8Aa\x04\x05V[\x95a\x018\x82a\x01\x1E\x89a\x04JV[\x92a\x01+\x81\x8B\x01\x8Ba\x04kV[\x93\x90\x91Q\x80\x94\x81\x93a\x04\xBCV[\x03\x92Z\xF1a\x01Da\x04\xCAV[\x83\x85\x01R\x15\x15\x82RV[Q\x91\x015\x17\x15a\x01`W`\x01\x01a\0\xA6V[`d\x85`\x17\x86\x86\x8B\x7F\x08\xC3y\xA0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x85RRR\x7FMulticall3: call failed\0\0\0\0\0\0\0\0\0`DR\xFD[\x82\x80\xFD[P\x80\xFD[` \x80\x82\x01\x90\x80\x83R\x83Q\x80\x92R`@\x92`@\x81\x01\x82`@\x85`\x05\x1B\x84\x01\x01\x96\x01\x94`\0\x90`\0\x93[\x86\x85\x10a\x01\xF6WPPPPPPPP\x90V[\x90\x91\x92\x93\x94\x95\x96\x97\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xC0\x82\x82\x03\x01\x85R\x86\x80\x8AQ\x80Q\x15\x15\x84R\x01Q\x91\x84\x82\x82\x01R\x82Q\x92\x83\x86\x83\x01R\x86[\x84\x81\x10a\x02\x92WPP`\x01\x92\x82\x91\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xE0`\x1F``\x93\x8A\x85\x82\x86\x01\x01R\x01\x16\x01\x01\x9A\x01\x95\x01\x95\x01\x93\x96\x95\x94\x92\x91\x90a\x01\xE4V[\x81\x81\x01\x84\x01Q\x83\x82\x01``\x01R\x8A\x93\x01a\x02AV[\x7FNH{q\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`\0R`A`\x04R`$`\0\xFD[\x90\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xE0`\x1F`@Q\x93\x01\x16\x82\x01\x82\x81\x10g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x17a\x03\x1AW`@RV[a\x02\xA7V[g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x03\x1AW`\x05\x1B` \x01\x90V[\x90a\x03Ia\x03D\x83a\x03\x1FV[a\x02\xD6V[\x82\x81R\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xE0a\x03w\x82\x94a\x03\x1FV[\x01`\0\x80[\x82\x81\x10a\x03\x89WPPPPV[`@\x90\x81Q\x82\x81\x01\x81\x81\x10g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x17a\x03\x1AW` \x93R\x83\x81R\x82``\x81\x83\x01R\x82\x87\x01\x01R\x01a\x03|V[\x7FNH{q\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`\0R`2`\x04R`$`\0\xFD[\x80Q\x82\x10\x15a\x04\0W` \x91`\x05\x1B\x01\x01\x90V[a\x03\xBDV[\x91\x90\x81\x10\x15a\x04\0W`\x05\x1B\x81\x015\x90\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xA1\x816\x03\x01\x82\x12\x15a\x04EW\x01\x90V[`\0\x80\xFD[5s\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x16\x81\x03a\x04EW\x90V[\x905\x90\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xE1\x816\x03\x01\x82\x12\x15a\x04EW\x01\x805\x90g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11a\x04EW` \x01\x91\x816\x03\x83\x13a\x04EWV[\x90\x80\x92\x91\x827\x01`\0\x81R\x90V[=\x15a\x05\x1FW=\x90g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11a\x03\x1AWa\x05\x12` \x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xE0`\x1F\x85\x01\x16\x01a\x02\xD6V[\x91\x82R=`\0` \x84\x01>V[``\x90V";
    /// The deployed bytecode of the contract.
    #[cfg(feature = "providers")]
    pub static MULTICALL_DEPLOYED_BYTECODE: ::ethers::core::types::Bytes =
        ::ethers::core::types::Bytes::from_static(__DEPLOYED_BYTECODE);
    #[cfg(feature = "providers")]
    pub struct Multicall<M>(::ethers::contract::Contract<M>);
    #[cfg(feature = "providers")]
    impl<M> ::core::clone::Clone for Multicall<M> {
        fn clone(&self) -> Self {
            Self(::core::clone::Clone::clone(&self.0))
        }
    }
    #[cfg(feature = "providers")]
    impl<M> ::core::ops::Deref for Multicall<M> {
        type Target = ::ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    #[cfg(feature = "providers")]
    impl<M> ::core::ops::DerefMut for Multicall<M> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    #[cfg(feature = "providers")]
    impl<M> ::core::fmt::Debug for Multicall<M> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple(::core::stringify!(Multicall))
                .field(&self.address())
                .finish()
        }
    }
    #[cfg(feature = "providers")]
    impl<M: ::ethers::providers::Middleware> Multicall<M> {
        /// Creates a new contract instance with the specified `ethers` client at
        /// `address`. The contract derefs to a `ethers::Contract` object.
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(::ethers::contract::Contract::new(
                address.into(),
                MULTICALL_ABI.clone(),
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
                MULTICALL_ABI.clone(),
                MULTICALL_BYTECODE.clone().into(),
                client,
            );
            let deployer = factory.deploy(constructor_args)?;
            let deployer = ::ethers::contract::ContractDeployer::new(deployer);
            Ok(deployer)
        }
        ///Calls the contract's `multicall` (0xe8bbf5d7) function
        pub fn multicall(
            &self,
            calls: ::std::vec::Vec<Call3>,
        ) -> ::ethers::contract::builders::ContractCall<M, ::std::vec::Vec<Result>> {
            self.0
                .method_hash([232, 187, 245, 215], calls)
                .expect("method not found (this should never happen)")
        }
        ///Gets the contract's `MulticallResult` event
        pub fn multicall_result_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<::std::sync::Arc<M>, M, MulticallResultFilter>
        {
            self.0.event()
        }
        /// Returns an `Event` builder for all the events of this contract.
        pub fn events(
            &self,
        ) -> ::ethers::contract::builders::Event<::std::sync::Arc<M>, M, MulticallResultFilter>
        {
            self.0
                .event_with_filter(::core::default::Default::default())
        }
    }
    #[cfg(feature = "providers")]
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>> for Multicall<M> {
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
        Hash,
    )]
    #[ethevent(name = "MulticallResult", abi = "MulticallResult((bool,bytes)[])")]
    pub struct MulticallResultFilter(pub ::std::vec::Vec<Result>);
    ///Container type for all input parameters for the `multicall` function with signature `multicall((address,bool,bytes)[])` and selector `0xe8bbf5d7`
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
    #[ethcall(name = "multicall", abi = "multicall((address,bool,bytes)[])")]
    pub struct MulticallCall {
        pub calls: ::std::vec::Vec<Call3>,
    }
    ///Container type for all return fields from the `multicall` function with signature `multicall((address,bool,bytes)[])` and selector `0xe8bbf5d7`
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
    pub struct MulticallReturn {
        pub return_data: ::std::vec::Vec<Result>,
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
