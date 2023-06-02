pub use cometbls_client::*;
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
pub mod cometbls_client {
    pub use super::super::shared_types::*;
    #[rustfmt::skip]
    const __ABI: &str = "[{\"inputs\":[{\"internalType\":\"address\",\"name\":\"ibcHandler_\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"contract IZKVerifier\",\"name\":\"verifier_\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"constructor\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"clientId\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"clientStateBytes\",\"type\":\"bytes\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"consensusStateBytes\",\"type\":\"bytes\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"createClient\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"clientStateCommitment\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"struct ConsensusStateUpdate\",\"name\":\"update\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"bytes32\",\"name\":\"consensusStateCommitment\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"struct IbcCoreClientV1Height.Data\",\"name\":\"height\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"revision_number\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"revision_height\",\"type\":\"uint64\",\"components\":[]}]}]},{\"internalType\":\"bool\",\"name\":\"ok\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"clientId\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"getClientState\",\"outputs\":[{\"internalType\":\"bytes\",\"name\":\"\",\"type\":\"bytes\",\"components\":[]},{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"clientId\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"struct IbcCoreClientV1Height.Data\",\"name\":\"height\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"revision_number\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"revision_height\",\"type\":\"uint64\",\"components\":[]}]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"getConsensusState\",\"outputs\":[{\"internalType\":\"bytes\",\"name\":\"\",\"type\":\"bytes\",\"components\":[]},{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"clientId\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"getLatestHeight\",\"outputs\":[{\"internalType\":\"struct IbcCoreClientV1Height.Data\",\"name\":\"\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"revision_number\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"revision_height\",\"type\":\"uint64\",\"components\":[]}]},{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"clientId\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"struct IbcCoreClientV1Height.Data\",\"name\":\"height\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"revision_number\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"revision_height\",\"type\":\"uint64\",\"components\":[]}]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"getTimestampAtHeight\",\"outputs\":[{\"internalType\":\"uint64\",\"name\":\"\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"clientId\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"clientMessageBytes\",\"type\":\"bytes\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"updateClient\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"struct ConsensusStateUpdate[]\",\"name\":\"\",\"type\":\"tuple[]\",\"components\":[{\"internalType\":\"bytes32\",\"name\":\"consensusStateCommitment\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"struct IbcCoreClientV1Height.Data\",\"name\":\"height\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"revision_number\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"revision_height\",\"type\":\"uint64\",\"components\":[]}]}]},{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"clientId\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"struct IbcCoreClientV1Height.Data\",\"name\":\"height\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"revision_number\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"revision_height\",\"type\":\"uint64\",\"components\":[]}]},{\"internalType\":\"uint64\",\"name\":\"delayTimePeriod\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"delayBlockPeriod\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"proof\",\"type\":\"bytes\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"prefix\",\"type\":\"bytes\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"path\",\"type\":\"bytes\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"value\",\"type\":\"bytes\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"verifyMembership\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"clientId\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"struct IbcCoreClientV1Height.Data\",\"name\":\"height\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"revision_number\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"revision_height\",\"type\":\"uint64\",\"components\":[]}]},{\"internalType\":\"uint64\",\"name\":\"delayTimePeriod\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"delayBlockPeriod\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"proof\",\"type\":\"bytes\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"prefix\",\"type\":\"bytes\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"path\",\"type\":\"bytes\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"verifyNonMembership\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]}]";
    ///The parsed JSON ABI of the contract.
    pub static COMETBLSCLIENT_ABI: ::ethers::contract::Lazy<::ethers::core::abi::Abi> =
        ::ethers::contract::Lazy::new(|| {
            ::ethers::core::utils::__serde_json::from_str(__ABI).expect("ABI is always valid")
        });
    pub struct CometblsClient<M>(::ethers::contract::Contract<M>);
    impl<M> ::core::clone::Clone for CometblsClient<M> {
        fn clone(&self) -> Self {
            Self(::core::clone::Clone::clone(&self.0))
        }
    }
    impl<M> ::core::ops::Deref for CometblsClient<M> {
        type Target = ::ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> ::core::ops::DerefMut for CometblsClient<M> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<M> ::core::fmt::Debug for CometblsClient<M> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple(stringify!(CometblsClient))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ::ethers::providers::Middleware> CometblsClient<M> {
        /// Creates a new contract instance with the specified `ethers` client at
        /// `address`. The contract derefs to a `ethers::Contract` object.
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(::ethers::contract::Contract::new(
                address.into(),
                COMETBLSCLIENT_ABI.clone(),
                client,
            ))
        }
        ///Calls the contract's `createClient` (0x2629636b) function
        pub fn create_client(
            &self,
            client_id: ::std::string::String,
            client_state_bytes: ::ethers::core::types::Bytes,
            consensus_state_bytes: ::ethers::core::types::Bytes,
        ) -> ::ethers::contract::builders::ContractCall<M, ([u8; 32], ConsensusStateUpdate, bool)>
        {
            self.0
                .method_hash(
                    [38, 41, 99, 107],
                    (client_id, client_state_bytes, consensus_state_bytes),
                )
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getClientState` (0x76c81c42) function
        pub fn get_client_state(
            &self,
            client_id: ::std::string::String,
        ) -> ::ethers::contract::builders::ContractCall<M, (::ethers::core::types::Bytes, bool)>
        {
            self.0
                .method_hash([118, 200, 28, 66], client_id)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getConsensusState` (0x6cf44bf4) function
        pub fn get_consensus_state(
            &self,
            client_id: ::std::string::String,
            height: Data,
        ) -> ::ethers::contract::builders::ContractCall<M, (::ethers::core::types::Bytes, bool)>
        {
            self.0
                .method_hash([108, 244, 75, 244], (client_id, height))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getLatestHeight` (0x329681d0) function
        pub fn get_latest_height(
            &self,
            client_id: ::std::string::String,
        ) -> ::ethers::contract::builders::ContractCall<M, (Data, bool)> {
            self.0
                .method_hash([50, 150, 129, 208], client_id)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getTimestampAtHeight` (0x4b0bbdc4) function
        pub fn get_timestamp_at_height(
            &self,
            client_id: ::std::string::String,
            height: Data,
        ) -> ::ethers::contract::builders::ContractCall<M, (u64, bool)> {
            self.0
                .method_hash([75, 11, 189, 196], (client_id, height))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `updateClient` (0x6fbf8079) function
        pub fn update_client(
            &self,
            client_id: ::std::string::String,
            client_message_bytes: ::ethers::core::types::Bytes,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ([u8; 32], ::std::vec::Vec<ConsensusStateUpdate>, bool),
        > {
            self.0
                .method_hash([111, 191, 128, 121], (client_id, client_message_bytes))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `verifyMembership` (0xf9bb5a51) function
        pub fn verify_membership(
            &self,
            client_id: ::std::string::String,
            height: Data,
            delay_time_period: u64,
            delay_block_period: u64,
            proof: ::ethers::core::types::Bytes,
            prefix: ::ethers::core::types::Bytes,
            path: ::ethers::core::types::Bytes,
            value: ::ethers::core::types::Bytes,
        ) -> ::ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash(
                    [249, 187, 90, 81],
                    (
                        client_id,
                        height,
                        delay_time_period,
                        delay_block_period,
                        proof,
                        prefix,
                        path,
                        value,
                    ),
                )
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `verifyNonMembership` (0x999fbbb3) function
        pub fn verify_non_membership(
            &self,
            client_id: ::std::string::String,
            height: Data,
            delay_time_period: u64,
            delay_block_period: u64,
            proof: ::ethers::core::types::Bytes,
            prefix: ::ethers::core::types::Bytes,
            path: ::ethers::core::types::Bytes,
        ) -> ::ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash(
                    [153, 159, 187, 179],
                    (
                        client_id,
                        height,
                        delay_time_period,
                        delay_block_period,
                        proof,
                        prefix,
                        path,
                    ),
                )
                .expect("method not found (this should never happen)")
        }
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>>
        for CometblsClient<M>
    {
        fn from(contract: ::ethers::contract::Contract<M>) -> Self {
            Self::new(contract.address(), contract.client())
        }
    }
    ///Container type for all input parameters for the `createClient` function with signature `createClient(string,bytes,bytes)` and selector `0x2629636b`
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
    #[ethcall(name = "createClient", abi = "createClient(string,bytes,bytes)")]
    pub struct CreateClientCall {
        pub client_id: ::std::string::String,
        pub client_state_bytes: ::ethers::core::types::Bytes,
        pub consensus_state_bytes: ::ethers::core::types::Bytes,
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
        Hash,
    )]
    #[ethcall(name = "getClientState", abi = "getClientState(string)")]
    pub struct GetClientStateCall {
        pub client_id: ::std::string::String,
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
        Hash,
    )]
    #[ethcall(
        name = "getConsensusState",
        abi = "getConsensusState(string,(uint64,uint64))"
    )]
    pub struct GetConsensusStateCall {
        pub client_id: ::std::string::String,
        pub height: Data,
    }
    ///Container type for all input parameters for the `getLatestHeight` function with signature `getLatestHeight(string)` and selector `0x329681d0`
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
    #[ethcall(name = "getLatestHeight", abi = "getLatestHeight(string)")]
    pub struct GetLatestHeightCall {
        pub client_id: ::std::string::String,
    }
    ///Container type for all input parameters for the `getTimestampAtHeight` function with signature `getTimestampAtHeight(string,(uint64,uint64))` and selector `0x4b0bbdc4`
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
        name = "getTimestampAtHeight",
        abi = "getTimestampAtHeight(string,(uint64,uint64))"
    )]
    pub struct GetTimestampAtHeightCall {
        pub client_id: ::std::string::String,
        pub height: Data,
    }
    ///Container type for all input parameters for the `updateClient` function with signature `updateClient(string,bytes)` and selector `0x6fbf8079`
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
    #[ethcall(name = "updateClient", abi = "updateClient(string,bytes)")]
    pub struct UpdateClientCall {
        pub client_id: ::std::string::String,
        pub client_message_bytes: ::ethers::core::types::Bytes,
    }
    ///Container type for all input parameters for the `verifyMembership` function with signature `verifyMembership(string,(uint64,uint64),uint64,uint64,bytes,bytes,bytes,bytes)` and selector `0xf9bb5a51`
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
        name = "verifyMembership",
        abi = "verifyMembership(string,(uint64,uint64),uint64,uint64,bytes,bytes,bytes,bytes)"
    )]
    pub struct VerifyMembershipCall {
        pub client_id: ::std::string::String,
        pub height: Data,
        pub delay_time_period: u64,
        pub delay_block_period: u64,
        pub proof: ::ethers::core::types::Bytes,
        pub prefix: ::ethers::core::types::Bytes,
        pub path: ::ethers::core::types::Bytes,
        pub value: ::ethers::core::types::Bytes,
    }
    ///Container type for all input parameters for the `verifyNonMembership` function with signature `verifyNonMembership(string,(uint64,uint64),uint64,uint64,bytes,bytes,bytes)` and selector `0x999fbbb3`
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
        name = "verifyNonMembership",
        abi = "verifyNonMembership(string,(uint64,uint64),uint64,uint64,bytes,bytes,bytes)"
    )]
    pub struct VerifyNonMembershipCall {
        pub client_id: ::std::string::String,
        pub height: Data,
        pub delay_time_period: u64,
        pub delay_block_period: u64,
        pub proof: ::ethers::core::types::Bytes,
        pub prefix: ::ethers::core::types::Bytes,
        pub path: ::ethers::core::types::Bytes,
    }
    ///Container type for all of the contract's call
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum CometblsClientCalls {
        CreateClient(CreateClientCall),
        GetClientState(GetClientStateCall),
        GetConsensusState(GetConsensusStateCall),
        GetLatestHeight(GetLatestHeightCall),
        GetTimestampAtHeight(GetTimestampAtHeightCall),
        UpdateClient(UpdateClientCall),
        VerifyMembership(VerifyMembershipCall),
        VerifyNonMembership(VerifyNonMembershipCall),
    }
    impl ::ethers::core::abi::AbiDecode for CometblsClientCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded) = <CreateClientCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::CreateClient(decoded));
            }
            if let Ok(decoded) =
                <GetClientStateCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::GetClientState(decoded));
            }
            if let Ok(decoded) =
                <GetConsensusStateCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::GetConsensusState(decoded));
            }
            if let Ok(decoded) =
                <GetLatestHeightCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::GetLatestHeight(decoded));
            }
            if let Ok(decoded) =
                <GetTimestampAtHeightCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::GetTimestampAtHeight(decoded));
            }
            if let Ok(decoded) = <UpdateClientCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::UpdateClient(decoded));
            }
            if let Ok(decoded) =
                <VerifyMembershipCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::VerifyMembership(decoded));
            }
            if let Ok(decoded) =
                <VerifyNonMembershipCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::VerifyNonMembership(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for CometblsClientCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                Self::CreateClient(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::GetClientState(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::GetConsensusState(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::GetLatestHeight(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::GetTimestampAtHeight(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::UpdateClient(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::VerifyMembership(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::VerifyNonMembership(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
            }
        }
    }
    impl ::core::fmt::Display for CometblsClientCalls {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::CreateClient(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetClientState(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetConsensusState(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetLatestHeight(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetTimestampAtHeight(element) => ::core::fmt::Display::fmt(element, f),
                Self::UpdateClient(element) => ::core::fmt::Display::fmt(element, f),
                Self::VerifyMembership(element) => ::core::fmt::Display::fmt(element, f),
                Self::VerifyNonMembership(element) => ::core::fmt::Display::fmt(element, f),
            }
        }
    }
    impl ::core::convert::From<CreateClientCall> for CometblsClientCalls {
        fn from(value: CreateClientCall) -> Self {
            Self::CreateClient(value)
        }
    }
    impl ::core::convert::From<GetClientStateCall> for CometblsClientCalls {
        fn from(value: GetClientStateCall) -> Self {
            Self::GetClientState(value)
        }
    }
    impl ::core::convert::From<GetConsensusStateCall> for CometblsClientCalls {
        fn from(value: GetConsensusStateCall) -> Self {
            Self::GetConsensusState(value)
        }
    }
    impl ::core::convert::From<GetLatestHeightCall> for CometblsClientCalls {
        fn from(value: GetLatestHeightCall) -> Self {
            Self::GetLatestHeight(value)
        }
    }
    impl ::core::convert::From<GetTimestampAtHeightCall> for CometblsClientCalls {
        fn from(value: GetTimestampAtHeightCall) -> Self {
            Self::GetTimestampAtHeight(value)
        }
    }
    impl ::core::convert::From<UpdateClientCall> for CometblsClientCalls {
        fn from(value: UpdateClientCall) -> Self {
            Self::UpdateClient(value)
        }
    }
    impl ::core::convert::From<VerifyMembershipCall> for CometblsClientCalls {
        fn from(value: VerifyMembershipCall) -> Self {
            Self::VerifyMembership(value)
        }
    }
    impl ::core::convert::From<VerifyNonMembershipCall> for CometblsClientCalls {
        fn from(value: VerifyNonMembershipCall) -> Self {
            Self::VerifyNonMembership(value)
        }
    }
    ///Container type for all return fields from the `createClient` function with signature `createClient(string,bytes,bytes)` and selector `0x2629636b`
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
        pub client_state_commitment: [u8; 32],
        pub update: ConsensusStateUpdate,
        pub ok: bool,
    }
    ///Container type for all return fields from the `getClientState` function with signature `getClientState(string)` and selector `0x76c81c42`
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
    pub struct GetClientStateReturn(pub ::ethers::core::types::Bytes, pub bool);
    ///Container type for all return fields from the `getConsensusState` function with signature `getConsensusState(string,(uint64,uint64))` and selector `0x6cf44bf4`
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
    pub struct GetConsensusStateReturn(pub ::ethers::core::types::Bytes, pub bool);
    ///Container type for all return fields from the `getLatestHeight` function with signature `getLatestHeight(string)` and selector `0x329681d0`
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
    pub struct GetLatestHeightReturn(pub Data, pub bool);
    ///Container type for all return fields from the `getTimestampAtHeight` function with signature `getTimestampAtHeight(string,(uint64,uint64))` and selector `0x4b0bbdc4`
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
    pub struct GetTimestampAtHeightReturn(pub u64, pub bool);
    ///Container type for all return fields from the `updateClient` function with signature `updateClient(string,bytes)` and selector `0x6fbf8079`
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
    pub struct UpdateClientReturn(
        pub [u8; 32],
        pub ::std::vec::Vec<ConsensusStateUpdate>,
        pub bool,
    );
    ///Container type for all return fields from the `verifyMembership` function with signature `verifyMembership(string,(uint64,uint64),uint64,uint64,bytes,bytes,bytes,bytes)` and selector `0xf9bb5a51`
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
    pub struct VerifyMembershipReturn(pub bool);
    ///Container type for all return fields from the `verifyNonMembership` function with signature `verifyNonMembership(string,(uint64,uint64),uint64,uint64,bytes,bytes,bytes)` and selector `0x999fbbb3`
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
    pub struct VerifyNonMembershipReturn(pub bool);
    ///`ConsensusStateUpdate(bytes32,(uint64,uint64))`
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
    pub struct ConsensusStateUpdate {
        pub consensus_state_commitment: [u8; 32],
        pub height: Data,
    }
}
