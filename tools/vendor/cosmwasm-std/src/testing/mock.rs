use alloc::collections::BTreeMap;
use bech32::{encode, ToBase32, Variant};
use core::marker::PhantomData;
#[cfg(feature = "cosmwasm_1_3")]
use core::ops::Bound;
use serde::de::DeserializeOwned;
#[cfg(feature = "stargate")]
use serde::Serialize;
use sha2::{Digest, Sha256};
#[cfg(feature = "cosmwasm_1_3")]
use std::collections::BTreeSet;
use std::collections::HashMap;

use crate::addresses::{Addr, CanonicalAddr};
use crate::binary::Binary;
use crate::coin::Coin;
use crate::deps::OwnedDeps;
use crate::errors::{RecoverPubkeyError, StdError, StdResult, SystemError, VerificationError};
#[cfg(feature = "stargate")]
use crate::ibc::{
    IbcAcknowledgement, IbcChannel, IbcChannelCloseMsg, IbcChannelConnectMsg, IbcChannelOpenMsg,
    IbcEndpoint, IbcOrder, IbcPacket, IbcPacketAckMsg, IbcPacketReceiveMsg, IbcPacketTimeoutMsg,
    IbcTimeoutBlock,
};
use crate::math::Uint128;
#[cfg(feature = "cosmwasm_1_1")]
use crate::query::SupplyResponse;
use crate::query::{
    AllBalanceResponse, BalanceResponse, BankQuery, CustomQuery, QueryRequest, WasmQuery,
};
#[cfg(feature = "staking")]
use crate::query::{
    AllDelegationsResponse, AllValidatorsResponse, BondedDenomResponse, DelegationResponse,
    FullDelegation, StakingQuery, Validator, ValidatorResponse,
};
#[cfg(feature = "cosmwasm_1_3")]
use crate::query::{DelegatorWithdrawAddressResponse, DistributionQuery};
use crate::results::{ContractResult, Empty, SystemResult};
use crate::serde::{from_json, to_json_binary};
use crate::storage::MemoryStorage;
use crate::timestamp::Timestamp;
use crate::traits::{Api, Querier, QuerierResult};
use crate::types::{BlockInfo, ContractInfo, Env, MessageInfo, TransactionInfo};
#[cfg(feature = "cosmwasm_1_3")]
use crate::{
    query::{AllDenomMetadataResponse, DecCoin, DenomMetadataResponse},
    PageRequest,
};
use crate::{Attribute, DenomMetadata};
#[cfg(feature = "stargate")]
use crate::{ChannelResponse, IbcQuery, ListChannelsResponse, PortIdResponse};
#[cfg(feature = "cosmwasm_1_4")]
use crate::{Decimal256, DelegationRewardsResponse, DelegatorValidatorsResponse};

use super::riffle_shuffle;

pub const MOCK_CONTRACT_ADDR: &str = "cosmos2contract";

/// Creates all external requirements that can be injected for unit tests.
///
/// See also [`mock_dependencies_with_balance`] and [`mock_dependencies_with_balances`]
/// if you want to start with some initial balances.
pub fn mock_dependencies() -> OwnedDeps<MockStorage, MockApi, MockQuerier, Empty> {
    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: MockQuerier::default(),
        custom_query_type: PhantomData,
    }
}

/// Creates all external requirements that can be injected for unit tests.
///
/// It sets the given balance for the contract itself, nothing else.
pub fn mock_dependencies_with_balance(
    contract_balance: &[Coin],
) -> OwnedDeps<MockStorage, MockApi, MockQuerier, Empty> {
    mock_dependencies_with_balances(&[(MOCK_CONTRACT_ADDR, contract_balance)])
}

/// Initializes the querier along with the mock_dependencies.
/// Sets all balances provided (you must explicitly set contract balance if desired).
pub fn mock_dependencies_with_balances(
    balances: &[(&str, &[Coin])],
) -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: MockQuerier::new(balances),
        custom_query_type: PhantomData,
    }
}

// Use MemoryStorage implementation (which is valid in non-testcode)
// We can later make simplifications here if needed
pub type MockStorage = MemoryStorage;

/// Length of canonical addresses created with this API. Contracts should not make any assumptions
/// what this value is.
///
/// The mock API can only canonicalize and humanize addresses up to this length. So it must be
/// long enough to store common bech32 addresses.
///
/// The value here must be restorable with `SHUFFLES_ENCODE` + `SHUFFLES_DECODE` in-shuffles.
/// See <https://oeis.org/A002326/list> for a table of those values.
const CANONICAL_LENGTH: usize = 90; // n = 45

const SHUFFLES_ENCODE: usize = 10;
const SHUFFLES_DECODE: usize = 2;

/// Default prefix used when creating Bech32 encoded address.
const BECH32_PREFIX: &str = "cosmwasm";

// MockApi zero pads all human addresses to make them fit the canonical_length
// it trims off zeros for the reverse operation.
// not really smart, but allows us to see a difference (and consistent length for canonical addresses)
#[derive(Copy, Clone)]
pub struct MockApi {
    /// Length of canonical addresses created with this API. Contracts should not make any assumptions
    /// what this value is.
    canonical_length: usize,
    /// Prefix used for creating addresses in Bech32 encoding.
    bech32_prefix: &'static str,
}

impl Default for MockApi {
    fn default() -> Self {
        MockApi {
            canonical_length: CANONICAL_LENGTH,
            bech32_prefix: BECH32_PREFIX,
        }
    }
}

impl Api for MockApi {
    fn addr_validate(&self, input: &str) -> StdResult<Addr> {
        let canonical = self.addr_canonicalize(input)?;
        let normalized = self.addr_humanize(&canonical)?;
        if input != normalized {
            return Err(StdError::generic_err(
                "Invalid input: address not normalized",
            ));
        }

        Ok(Addr::unchecked(input))
    }

    fn addr_canonicalize(&self, input: &str) -> StdResult<CanonicalAddr> {
        // Dummy input validation. This is more sophisticated for formats like bech32, where format and checksum are validated.
        let min_length = 3;
        let max_length = self.canonical_length;
        if input.len() < min_length {
            return Err(StdError::generic_err(
                format!("Invalid input: human address too short for this mock implementation (must be >= {min_length})."),
            ));
        }
        if input.len() > max_length {
            return Err(StdError::generic_err(
                format!("Invalid input: human address too long for this mock implementation (must be <= {max_length})."),
            ));
        }

        // mimics formats like hex or bech32 where different casings are valid for one address
        let normalized = input.to_lowercase();

        let mut out = Vec::from(normalized);

        // pad to canonical length with NULL bytes
        out.resize(self.canonical_length, 0x00);
        // content-dependent rotate followed by shuffle to destroy
        // the most obvious structure (https://github.com/CosmWasm/cosmwasm/issues/552)
        let rotate_by = digit_sum(&out) % self.canonical_length;
        out.rotate_left(rotate_by);
        for _ in 0..SHUFFLES_ENCODE {
            out = riffle_shuffle(&out);
        }
        Ok(out.into())
    }

    fn addr_humanize(&self, canonical: &CanonicalAddr) -> StdResult<Addr> {
        if canonical.len() != self.canonical_length {
            return Err(StdError::generic_err(
                "Invalid input: canonical address length not correct",
            ));
        }

        let mut tmp: Vec<u8> = canonical.clone().into();
        // Shuffle two more times which restored the original value (24 elements are back to original after 20 rounds)
        for _ in 0..SHUFFLES_DECODE {
            tmp = riffle_shuffle(&tmp);
        }
        // Rotate back
        let rotate_by = digit_sum(&tmp) % self.canonical_length;
        tmp.rotate_right(rotate_by);
        // Remove NULL bytes (i.e. the padding)
        let trimmed = tmp.into_iter().filter(|&x| x != 0x00).collect();
        // decode UTF-8 bytes into string
        let human = String::from_utf8(trimmed)?;
        Ok(Addr::unchecked(human))
    }

    fn secp256k1_verify(
        &self,
        message_hash: &[u8],
        signature: &[u8],
        public_key: &[u8],
    ) -> Result<bool, VerificationError> {
        Ok(cosmwasm_crypto::secp256k1_verify(
            message_hash,
            signature,
            public_key,
        )?)
    }

    fn secp256k1_recover_pubkey(
        &self,
        message_hash: &[u8],
        signature: &[u8],
        recovery_param: u8,
    ) -> Result<Vec<u8>, RecoverPubkeyError> {
        let pubkey =
            cosmwasm_crypto::secp256k1_recover_pubkey(message_hash, signature, recovery_param)?;
        Ok(pubkey.to_vec())
    }

    fn ed25519_verify(
        &self,
        message: &[u8],
        signature: &[u8],
        public_key: &[u8],
    ) -> Result<bool, VerificationError> {
        Ok(cosmwasm_crypto::ed25519_verify(
            message, signature, public_key,
        )?)
    }

    fn ed25519_batch_verify(
        &self,
        messages: &[&[u8]],
        signatures: &[&[u8]],
        public_keys: &[&[u8]],
    ) -> Result<bool, VerificationError> {
        Ok(cosmwasm_crypto::ed25519_batch_verify(
            messages,
            signatures,
            public_keys,
        )?)
    }

    fn debug(&self, message: &str) {
        println!("{message}");
    }
}

impl MockApi {
    /// Returns [MockApi] with Bech32 prefix set to provided value.
    ///
    /// Bech32 prefix must not be empty.
    ///
    /// # Example
    ///
    /// ```
    /// # use cosmwasm_std::Addr;
    /// # use cosmwasm_std::testing::MockApi;
    /// #
    /// let mock_api = MockApi::default().with_prefix("juno");
    /// let addr = mock_api.addr_make("creator");
    ///
    /// assert_eq!(addr.to_string(), "juno1h34lmpywh4upnjdg90cjf4j70aee6z8qqfspugamjp42e4q28kqsksmtyp");
    /// ```
    pub fn with_prefix(mut self, prefix: &'static str) -> Self {
        self.bech32_prefix = prefix;
        self
    }

    /// Returns an address built from provided input string.
    ///
    /// # Example
    ///
    /// ```
    /// # use cosmwasm_std::Addr;
    /// # use cosmwasm_std::testing::MockApi;
    /// #
    /// let mock_api = MockApi::default();
    /// let addr = mock_api.addr_make("creator");
    ///
    /// assert_eq!(addr.to_string(), "cosmwasm1h34lmpywh4upnjdg90cjf4j70aee6z8qqfspugamjp42e4q28kqs8s7vcp");
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics when generating a valid address is not possible,
    /// especially when Bech32 prefix set in function [with_prefix](Self::with_prefix) is empty.
    ///
    pub fn addr_make(&self, input: &str) -> Addr {
        let digest = Sha256::digest(input).to_vec();
        match encode(self.bech32_prefix, digest.to_base32(), Variant::Bech32) {
            Ok(address) => Addr::unchecked(address),
            Err(reason) => panic!("Generating address failed with reason: {reason}"),
        }
    }
}

/// Returns a default enviroment with height, time, chain_id, and contract address
/// You can submit as is to most contracts, or modify height/time if you want to
/// test for expiration.
///
/// This is intended for use in test code only.
pub fn mock_env() -> Env {
    Env {
        block: BlockInfo {
            height: 12_345,
            time: Timestamp::from_nanos(1_571_797_419_879_305_533),
            chain_id: "cosmos-testnet-14002".to_string(),
        },
        transaction: Some(TransactionInfo { index: 3 }),
        contract: ContractInfo {
            address: Addr::unchecked(MOCK_CONTRACT_ADDR),
        },
    }
}

/// Just set sender and funds for the message.
/// This is intended for use in test code only.
pub fn mock_info(sender: &str, funds: &[Coin]) -> MessageInfo {
    MessageInfo {
        sender: Addr::unchecked(sender),
        funds: funds.to_vec(),
    }
}

/// Creates an IbcChannel for testing. You set a few key parameters for handshaking,
/// If you want to set more, use this as a default and mutate other fields
#[cfg(feature = "stargate")]
pub fn mock_ibc_channel(my_channel_id: &str, order: IbcOrder, version: &str) -> IbcChannel {
    IbcChannel {
        endpoint: IbcEndpoint {
            port_id: "my_port".to_string(),
            channel_id: my_channel_id.to_string(),
        },
        counterparty_endpoint: IbcEndpoint {
            port_id: "their_port".to_string(),
            channel_id: "channel-7".to_string(),
        },
        order,
        version: version.to_string(),
        connection_id: "connection-2".to_string(),
    }
}

/// Creates a IbcChannelOpenMsg::OpenInit for testing ibc_channel_open.
#[cfg(feature = "stargate")]
pub fn mock_ibc_channel_open_init(
    my_channel_id: &str,
    order: IbcOrder,
    version: &str,
) -> IbcChannelOpenMsg {
    IbcChannelOpenMsg::new_init(mock_ibc_channel(my_channel_id, order, version))
}

/// Creates a IbcChannelOpenMsg::OpenTry for testing ibc_channel_open.
#[cfg(feature = "stargate")]
pub fn mock_ibc_channel_open_try(
    my_channel_id: &str,
    order: IbcOrder,
    version: &str,
) -> IbcChannelOpenMsg {
    IbcChannelOpenMsg::new_try(mock_ibc_channel(my_channel_id, order, version), version)
}

/// Creates a IbcChannelConnectMsg::ConnectAck for testing ibc_channel_connect.
#[cfg(feature = "stargate")]
pub fn mock_ibc_channel_connect_ack(
    my_channel_id: &str,
    order: IbcOrder,
    version: &str,
) -> IbcChannelConnectMsg {
    IbcChannelConnectMsg::new_ack(mock_ibc_channel(my_channel_id, order, version), version)
}

/// Creates a IbcChannelConnectMsg::ConnectConfirm for testing ibc_channel_connect.
#[cfg(feature = "stargate")]
pub fn mock_ibc_channel_connect_confirm(
    my_channel_id: &str,
    order: IbcOrder,
    version: &str,
) -> IbcChannelConnectMsg {
    IbcChannelConnectMsg::new_confirm(mock_ibc_channel(my_channel_id, order, version))
}

/// Creates a IbcChannelCloseMsg::CloseInit for testing ibc_channel_close.
#[cfg(feature = "stargate")]
pub fn mock_ibc_channel_close_init(
    my_channel_id: &str,
    order: IbcOrder,
    version: &str,
) -> IbcChannelCloseMsg {
    IbcChannelCloseMsg::new_init(mock_ibc_channel(my_channel_id, order, version))
}

/// Creates a IbcChannelCloseMsg::CloseConfirm for testing ibc_channel_close.
#[cfg(feature = "stargate")]
pub fn mock_ibc_channel_close_confirm(
    my_channel_id: &str,
    order: IbcOrder,
    version: &str,
) -> IbcChannelCloseMsg {
    IbcChannelCloseMsg::new_confirm(mock_ibc_channel(my_channel_id, order, version))
}

/// Creates a IbcPacketReceiveMsg for testing ibc_packet_receive. You set a few key parameters that are
/// often parsed. If you want to set more, use this as a default and mutate other fields
#[cfg(feature = "stargate")]
pub fn mock_ibc_packet_recv(
    my_channel_id: &str,
    data: &impl Serialize,
) -> StdResult<IbcPacketReceiveMsg> {
    Ok(IbcPacketReceiveMsg::new(
        IbcPacket {
            data: to_json_binary(data)?,
            src: IbcEndpoint {
                port_id: "their-port".to_string(),
                channel_id: "channel-1234".to_string(),
            },
            dest: IbcEndpoint {
                port_id: "our-port".to_string(),
                channel_id: my_channel_id.into(),
            },
            sequence: 27,
            timeout: IbcTimeoutBlock {
                revision: 1,
                height: 12345678,
            }
            .into(),
        },
        #[cfg(feature = "ibc3")]
        Addr::unchecked("relayer"),
    ))
}

/// Creates a IbcPacket for testing ibc_packet_{ack,timeout}. You set a few key parameters that are
/// often parsed. If you want to set more, use this as a default and mutate other fields.
/// The difference from mock_ibc_packet_recv is if `my_channel_id` is src or dest.
#[cfg(feature = "stargate")]
fn mock_ibc_packet(my_channel_id: &str, data: &impl Serialize) -> StdResult<IbcPacket> {
    Ok(IbcPacket {
        data: to_json_binary(data)?,
        src: IbcEndpoint {
            port_id: "their-port".to_string(),
            channel_id: my_channel_id.into(),
        },
        dest: IbcEndpoint {
            port_id: "our-port".to_string(),
            channel_id: "channel-1234".to_string(),
        },
        sequence: 29,
        timeout: IbcTimeoutBlock {
            revision: 1,
            height: 432332552,
        }
        .into(),
    })
}

/// Creates a IbcPacketAckMsg for testing ibc_packet_ack. You set a few key parameters that are
/// often parsed. If you want to set more, use this as a default and mutate other fields.
/// The difference from mock_ibc_packet_recv is if `my_channel_id` is src or dest.
#[cfg(feature = "stargate")]
pub fn mock_ibc_packet_ack(
    my_channel_id: &str,
    data: &impl Serialize,
    ack: IbcAcknowledgement,
) -> StdResult<IbcPacketAckMsg> {
    let packet = mock_ibc_packet(my_channel_id, data)?;

    Ok(IbcPacketAckMsg::new(
        ack,
        packet,
        #[cfg(feature = "ibc3")]
        Addr::unchecked("relayer"),
    ))
}

/// Creates a IbcPacketTimeoutMsg for testing ibc_packet_timeout. You set a few key parameters that are
/// often parsed. If you want to set more, use this as a default and mutate other fields.
/// The difference from mock_ibc_packet_recv is if `my_channel_id` is src or dest./
#[cfg(feature = "stargate")]
pub fn mock_ibc_packet_timeout(
    my_channel_id: &str,
    data: &impl Serialize,
) -> StdResult<IbcPacketTimeoutMsg> {
    let packet = mock_ibc_packet(my_channel_id, data)?;
    Ok(IbcPacketTimeoutMsg::new(
        packet,
        #[cfg(feature = "ibc3")]
        Addr::unchecked("relayer"),
    ))
}

/// The same type as cosmwasm-std's QuerierResult, but easier to reuse in
/// cosmwasm-vm. It might diverge from QuerierResult at some point.
pub type MockQuerierCustomHandlerResult = SystemResult<ContractResult<Binary>>;

/// MockQuerier holds an immutable table of bank balances
/// and configurable handlers for Wasm queries and custom queries.
pub struct MockQuerier<C: DeserializeOwned = Empty> {
    bank: BankQuerier,
    #[cfg(feature = "staking")]
    staking: StakingQuerier,
    #[cfg(feature = "cosmwasm_1_3")]
    distribution: DistributionQuerier,
    wasm: WasmQuerier,
    #[cfg(feature = "stargate")]
    ibc: IbcQuerier,
    /// A handler to handle custom queries. This is set to a dummy handler that
    /// always errors by default. Update it via `with_custom_handler`.
    ///
    /// Use box to avoid the need of another generic type
    custom_handler: Box<dyn for<'a> Fn(&'a C) -> MockQuerierCustomHandlerResult>,
}

impl<C: DeserializeOwned> MockQuerier<C> {
    pub fn new(balances: &[(&str, &[Coin])]) -> Self {
        MockQuerier {
            bank: BankQuerier::new(balances),
            #[cfg(feature = "cosmwasm_1_3")]
            distribution: DistributionQuerier::default(),
            #[cfg(feature = "staking")]
            staking: StakingQuerier::default(),
            wasm: WasmQuerier::default(),
            #[cfg(feature = "stargate")]
            ibc: IbcQuerier::default(),
            // strange argument notation suggested as a workaround here: https://github.com/rust-lang/rust/issues/41078#issuecomment-294296365
            custom_handler: Box::from(|_: &_| -> MockQuerierCustomHandlerResult {
                SystemResult::Err(SystemError::UnsupportedRequest {
                    kind: "custom".to_string(),
                })
            }),
        }
    }

    // set a new balance for the given address and return the old balance
    pub fn update_balance(
        &mut self,
        addr: impl Into<String>,
        balance: Vec<Coin>,
    ) -> Option<Vec<Coin>> {
        self.bank.update_balance(addr, balance)
    }

    pub fn set_denom_metadata(&mut self, denom_metadata: &[DenomMetadata]) {
        self.bank.set_denom_metadata(denom_metadata);
    }

    #[cfg(feature = "cosmwasm_1_3")]
    pub fn set_withdraw_address(
        &mut self,
        delegator_address: impl Into<String>,
        withdraw_address: impl Into<String>,
    ) {
        self.distribution
            .set_withdraw_address(delegator_address, withdraw_address);
    }

    /// Sets multiple withdraw addresses.
    ///
    /// This allows passing multiple tuples of `(delegator_address, withdraw_address)`.
    /// It does not overwrite existing entries.
    #[cfg(feature = "cosmwasm_1_3")]
    pub fn set_withdraw_addresses(
        &mut self,
        withdraw_addresses: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) {
        self.distribution.set_withdraw_addresses(withdraw_addresses);
    }

    #[cfg(feature = "cosmwasm_1_3")]
    pub fn clear_withdraw_addresses(&mut self) {
        self.distribution.clear_withdraw_addresses();
    }

    #[cfg(feature = "staking")]
    pub fn update_staking(
        &mut self,
        denom: &str,
        validators: &[crate::query::Validator],
        delegations: &[crate::query::FullDelegation],
    ) {
        self.staking = StakingQuerier::new(denom, validators, delegations);
    }

    #[cfg(feature = "stargate")]
    pub fn update_ibc(&mut self, port_id: &str, channels: &[IbcChannel]) {
        self.ibc = IbcQuerier::new(port_id, channels);
    }

    pub fn update_wasm<WH: 'static>(&mut self, handler: WH)
    where
        WH: Fn(&WasmQuery) -> QuerierResult,
    {
        self.wasm.update_handler(handler)
    }

    pub fn with_custom_handler<CH: 'static>(mut self, handler: CH) -> Self
    where
        CH: Fn(&C) -> MockQuerierCustomHandlerResult,
    {
        self.custom_handler = Box::from(handler);
        self
    }
}

impl Default for MockQuerier {
    fn default() -> Self {
        MockQuerier::new(&[])
    }
}

impl<C: CustomQuery + DeserializeOwned> Querier for MockQuerier<C> {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        let request: QueryRequest<C> = match from_json(bin_request) {
            Ok(v) => v,
            Err(e) => {
                return SystemResult::Err(SystemError::InvalidRequest {
                    error: format!("Parsing query request: {e}"),
                    request: bin_request.into(),
                })
            }
        };
        self.handle_query(&request)
    }
}

impl<C: CustomQuery + DeserializeOwned> MockQuerier<C> {
    pub fn handle_query(&self, request: &QueryRequest<C>) -> QuerierResult {
        match &request {
            QueryRequest::Bank(bank_query) => self.bank.query(bank_query),
            QueryRequest::Custom(custom_query) => (*self.custom_handler)(custom_query),
            #[cfg(feature = "staking")]
            QueryRequest::Staking(staking_query) => self.staking.query(staking_query),
            #[cfg(feature = "cosmwasm_1_3")]
            QueryRequest::Distribution(distribution_query) => {
                self.distribution.query(distribution_query)
            }
            QueryRequest::Wasm(msg) => self.wasm.query(msg),
            #[cfg(feature = "stargate")]
            QueryRequest::Stargate { .. } => SystemResult::Err(SystemError::UnsupportedRequest {
                kind: "Stargate".to_string(),
            }),
            #[cfg(feature = "stargate")]
            QueryRequest::Ibc(msg) => self.ibc.query(msg),
        }
    }
}

struct WasmQuerier {
    /// A handler to handle Wasm queries. This is set to a dummy handler that
    /// always errors by default. Update it via `with_custom_handler`.
    ///
    /// Use box to avoid the need of generic type.
    handler: Box<dyn for<'a> Fn(&'a WasmQuery) -> QuerierResult>,
}

impl WasmQuerier {
    fn new(handler: Box<dyn for<'a> Fn(&'a WasmQuery) -> QuerierResult>) -> Self {
        Self { handler }
    }

    fn update_handler<WH: 'static>(&mut self, handler: WH)
    where
        WH: Fn(&WasmQuery) -> QuerierResult,
    {
        self.handler = Box::from(handler)
    }

    fn query(&self, request: &WasmQuery) -> QuerierResult {
        (*self.handler)(request)
    }
}

impl Default for WasmQuerier {
    fn default() -> Self {
        let handler = Box::from(|request: &WasmQuery| -> QuerierResult {
            let err = match request {
                WasmQuery::Smart { contract_addr, .. } => SystemError::NoSuchContract {
                    addr: contract_addr.clone(),
                },
                WasmQuery::Raw { contract_addr, .. } => SystemError::NoSuchContract {
                    addr: contract_addr.clone(),
                },
                WasmQuery::ContractInfo { contract_addr, .. } => SystemError::NoSuchContract {
                    addr: contract_addr.clone(),
                },
                #[cfg(feature = "cosmwasm_1_2")]
                WasmQuery::CodeInfo { code_id, .. } => {
                    SystemError::NoSuchCode { code_id: *code_id }
                }
            };
            SystemResult::Err(err)
        });
        Self::new(handler)
    }
}

#[derive(Clone, Default)]
pub struct BankQuerier {
    #[allow(dead_code)]
    /// HashMap<denom, amount>
    supplies: HashMap<String, Uint128>,
    /// HashMap<address, coins>
    balances: HashMap<String, Vec<Coin>>,
    /// Vec<Metadata>
    denom_metadata: BTreeMap<Vec<u8>, DenomMetadata>,
}

impl BankQuerier {
    pub fn new(balances: &[(&str, &[Coin])]) -> Self {
        let balances: HashMap<_, _> = balances
            .iter()
            .map(|(s, c)| (s.to_string(), c.to_vec()))
            .collect();

        BankQuerier {
            supplies: Self::calculate_supplies(&balances),
            balances,
            denom_metadata: BTreeMap::new(),
        }
    }

    pub fn update_balance(
        &mut self,
        addr: impl Into<String>,
        balance: Vec<Coin>,
    ) -> Option<Vec<Coin>> {
        let result = self.balances.insert(addr.into(), balance);
        self.supplies = Self::calculate_supplies(&self.balances);

        result
    }

    pub fn set_denom_metadata(&mut self, denom_metadata: &[DenomMetadata]) {
        self.denom_metadata = denom_metadata
            .iter()
            .map(|d| (d.base.as_bytes().to_vec(), d.clone()))
            .collect();
    }

    fn calculate_supplies(balances: &HashMap<String, Vec<Coin>>) -> HashMap<String, Uint128> {
        let mut supplies = HashMap::new();

        let all_coins = balances.iter().flat_map(|(_, coins)| coins);

        for coin in all_coins {
            *supplies
                .entry(coin.denom.clone())
                .or_insert_with(Uint128::zero) += coin.amount;
        }

        supplies
    }

    pub fn query(&self, request: &BankQuery) -> QuerierResult {
        let contract_result: ContractResult<Binary> = match request {
            #[cfg(feature = "cosmwasm_1_1")]
            BankQuery::Supply { denom } => {
                let amount = self
                    .supplies
                    .get(denom)
                    .cloned()
                    .unwrap_or_else(Uint128::zero);
                let bank_res = SupplyResponse {
                    amount: Coin {
                        amount,
                        denom: denom.to_string(),
                    },
                };
                to_json_binary(&bank_res).into()
            }
            BankQuery::Balance { address, denom } => {
                // proper error on not found, serialize result on found
                let amount = self
                    .balances
                    .get(address)
                    .and_then(|v| v.iter().find(|c| &c.denom == denom).map(|c| c.amount))
                    .unwrap_or_default();
                let bank_res = BalanceResponse {
                    amount: Coin {
                        amount,
                        denom: denom.to_string(),
                    },
                };
                to_json_binary(&bank_res).into()
            }
            BankQuery::AllBalances { address } => {
                // proper error on not found, serialize result on found
                let bank_res = AllBalanceResponse {
                    amount: self.balances.get(address).cloned().unwrap_or_default(),
                };
                to_json_binary(&bank_res).into()
            }
            #[cfg(feature = "cosmwasm_1_3")]
            BankQuery::DenomMetadata { denom } => {
                let denom_metadata = self.denom_metadata.get(denom.as_bytes());
                match denom_metadata {
                    Some(m) => {
                        let metadata_res = DenomMetadataResponse {
                            metadata: m.clone(),
                        };
                        to_json_binary(&metadata_res).into()
                    }
                    None => return SystemResult::Err(SystemError::Unknown {}),
                }
            }
            #[cfg(feature = "cosmwasm_1_3")]
            BankQuery::AllDenomMetadata { pagination } => {
                let default_pagination = PageRequest {
                    key: None,
                    limit: 100,
                    reverse: false,
                };
                let pagination = pagination.as_ref().unwrap_or(&default_pagination);

                // range of all denoms after the given key (or until the key for reverse)
                let range = match (pagination.reverse, &pagination.key) {
                    (_, None) => (Bound::Unbounded, Bound::Unbounded),
                    (true, Some(key)) => (Bound::Unbounded, Bound::Included(key.as_slice())),
                    (false, Some(key)) => (Bound::Included(key.as_slice()), Bound::Unbounded),
                };
                let iter = self.denom_metadata.range::<[u8], _>(range);
                // using dynamic dispatch here to reduce code duplication and since this is only testing code
                let iter: Box<dyn Iterator<Item = _>> = if pagination.reverse {
                    Box::new(iter.rev())
                } else {
                    Box::new(iter)
                };

                let mut metadata: Vec<_> = iter
                    // take the requested amount + 1 to get the next key
                    .take((pagination.limit.saturating_add(1)) as usize)
                    .map(|(_, m)| m.clone())
                    .collect();

                // if we took more than requested, remove the last element (the next key),
                // otherwise this is the last batch
                let next_key = if metadata.len() > pagination.limit as usize {
                    metadata.pop().map(|m| Binary::from(m.base.as_bytes()))
                } else {
                    None
                };

                let metadata_res = AllDenomMetadataResponse { metadata, next_key };
                to_json_binary(&metadata_res).into()
            }
        };
        // system result is always ok in the mock implementation
        SystemResult::Ok(contract_result)
    }
}

#[cfg(feature = "stargate")]
#[derive(Clone, Default)]
pub struct IbcQuerier {
    port_id: String,
    channels: Vec<IbcChannel>,
}

#[cfg(feature = "stargate")]
impl IbcQuerier {
    /// Create a mock querier where:
    /// - port_id is the port the "contract" is bound to
    /// - channels are a list of ibc channels
    pub fn new(port_id: &str, channels: &[IbcChannel]) -> Self {
        IbcQuerier {
            port_id: port_id.to_string(),
            channels: channels.to_vec(),
        }
    }

    pub fn query(&self, request: &IbcQuery) -> QuerierResult {
        let contract_result: ContractResult<Binary> = match request {
            IbcQuery::Channel {
                channel_id,
                port_id,
            } => {
                let channel = self
                    .channels
                    .iter()
                    .find(|c| match port_id {
                        Some(p) => c.endpoint.channel_id.eq(channel_id) && c.endpoint.port_id.eq(p),
                        None => {
                            c.endpoint.channel_id.eq(channel_id)
                                && c.endpoint.port_id == self.port_id
                        }
                    })
                    .cloned();
                let res = ChannelResponse { channel };
                to_json_binary(&res).into()
            }
            IbcQuery::ListChannels { port_id } => {
                let channels = self
                    .channels
                    .iter()
                    .filter(|c| match port_id {
                        Some(p) => c.endpoint.port_id.eq(p),
                        None => c.endpoint.port_id == self.port_id,
                    })
                    .cloned()
                    .collect();
                let res = ListChannelsResponse { channels };
                to_json_binary(&res).into()
            }
            IbcQuery::PortId {} => {
                let res = PortIdResponse {
                    port_id: self.port_id.clone(),
                };
                to_json_binary(&res).into()
            }
        };
        // system result is always ok in the mock implementation
        SystemResult::Ok(contract_result)
    }
}

#[cfg(feature = "staking")]
#[derive(Clone, Default)]
pub struct StakingQuerier {
    denom: String,
    validators: Vec<Validator>,
    delegations: Vec<FullDelegation>,
}

#[cfg(feature = "staking")]
impl StakingQuerier {
    pub fn new(denom: &str, validators: &[Validator], delegations: &[FullDelegation]) -> Self {
        StakingQuerier {
            denom: denom.to_string(),
            validators: validators.to_vec(),
            delegations: delegations.to_vec(),
        }
    }

    pub fn query(&self, request: &StakingQuery) -> QuerierResult {
        let contract_result: ContractResult<Binary> = match request {
            StakingQuery::BondedDenom {} => {
                let res = BondedDenomResponse {
                    denom: self.denom.clone(),
                };
                to_json_binary(&res).into()
            }
            StakingQuery::AllValidators {} => {
                let res = AllValidatorsResponse {
                    validators: self.validators.clone(),
                };
                to_json_binary(&res).into()
            }
            StakingQuery::Validator { address } => {
                let validator: Option<Validator> = self
                    .validators
                    .iter()
                    .find(|validator| validator.address == *address)
                    .cloned();
                let res = ValidatorResponse { validator };
                to_json_binary(&res).into()
            }
            StakingQuery::AllDelegations { delegator } => {
                let delegations: Vec<_> = self
                    .delegations
                    .iter()
                    .filter(|d| d.delegator.as_str() == delegator)
                    .cloned()
                    .map(|d| d.into())
                    .collect();
                let res = AllDelegationsResponse { delegations };
                to_json_binary(&res).into()
            }
            StakingQuery::Delegation {
                delegator,
                validator,
            } => {
                let delegation = self
                    .delegations
                    .iter()
                    .find(|d| d.delegator.as_str() == delegator && d.validator == *validator);
                let res = DelegationResponse {
                    delegation: delegation.cloned(),
                };
                to_json_binary(&res).into()
            }
        };
        // system result is always ok in the mock implementation
        SystemResult::Ok(contract_result)
    }
}

#[cfg(feature = "cosmwasm_1_3")]
#[derive(Clone, Default)]
pub struct DistributionQuerier {
    withdraw_addresses: HashMap<String, String>,
    /// Mock of accumulated rewards, indexed first by delegator and then validator address.
    rewards: BTreeMap<String, BTreeMap<String, Vec<DecCoin>>>,
    /// Mock of validators that a delegator has bonded to.
    validators: BTreeMap<String, BTreeSet<String>>,
}

#[cfg(feature = "cosmwasm_1_3")]
impl DistributionQuerier {
    pub fn new(withdraw_addresses: HashMap<String, String>) -> Self {
        DistributionQuerier {
            withdraw_addresses,
            ..Default::default()
        }
    }

    pub fn set_withdraw_address(
        &mut self,
        delegator_address: impl Into<String>,
        withdraw_address: impl Into<String>,
    ) {
        self.withdraw_addresses
            .insert(delegator_address.into(), withdraw_address.into());
    }

    /// Sets multiple withdraw addresses.
    ///
    /// This allows passing multiple tuples of `(delegator_address, withdraw_address)`.
    /// It does not overwrite existing entries.
    pub fn set_withdraw_addresses(
        &mut self,
        withdraw_addresses: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) {
        for (d, w) in withdraw_addresses {
            self.set_withdraw_address(d, w);
        }
    }

    pub fn clear_withdraw_addresses(&mut self) {
        self.withdraw_addresses.clear();
    }

    /// Sets accumulated rewards for a given validator and delegator pair.
    pub fn set_rewards(
        &mut self,
        validator: impl Into<String>,
        delegator: impl Into<String>,
        rewards: Vec<DecCoin>,
    ) {
        self.rewards
            .entry(delegator.into())
            .or_default()
            .insert(validator.into(), rewards);
    }

    /// Sets the validators a given delegator has bonded to.
    pub fn set_validators(
        &mut self,
        delegator: impl Into<String>,
        validators: impl IntoIterator<Item = impl Into<String>>,
    ) {
        self.validators.insert(
            delegator.into(),
            validators.into_iter().map(Into::into).collect(),
        );
    }

    pub fn query(&self, request: &DistributionQuery) -> QuerierResult {
        let contract_result: ContractResult<Binary> = match request {
            DistributionQuery::DelegatorWithdrawAddress { delegator_address } => {
                let res = DelegatorWithdrawAddressResponse {
                    withdraw_address: Addr::unchecked(
                        self.withdraw_addresses
                            .get(delegator_address)
                            .unwrap_or(delegator_address),
                    ),
                };
                to_json_binary(&res).into()
            }
            #[cfg(feature = "cosmwasm_1_4")]
            DistributionQuery::DelegationRewards {
                delegator_address,
                validator_address,
            } => {
                let res = DelegationRewardsResponse {
                    rewards: self
                        .rewards
                        .get(delegator_address)
                        .and_then(|v| v.get(validator_address))
                        .cloned()
                        .unwrap_or_default(),
                };
                to_json_binary(&res).into()
            }
            #[cfg(feature = "cosmwasm_1_4")]
            DistributionQuery::DelegationTotalRewards { delegator_address } => {
                let validator_rewards = self
                    .validator_rewards(delegator_address)
                    .unwrap_or_default();
                let res = crate::DelegationTotalRewardsResponse {
                    total: validator_rewards
                        .iter()
                        .fold(BTreeMap::<&str, DecCoin>::new(), |mut acc, rewards| {
                            for coin in &rewards.reward {
                                acc.entry(&coin.denom)
                                    .or_insert_with(|| DecCoin {
                                        denom: coin.denom.clone(),
                                        amount: Decimal256::zero(),
                                    })
                                    .amount += coin.amount;
                            }

                            acc
                        })
                        .into_values()
                        .collect(),
                    rewards: validator_rewards,
                };
                to_json_binary(&res).into()
            }
            #[cfg(feature = "cosmwasm_1_4")]
            DistributionQuery::DelegatorValidators { delegator_address } => {
                let res = DelegatorValidatorsResponse {
                    validators: self
                        .validators
                        .get(delegator_address)
                        .map(|set| set.iter().cloned().collect())
                        .unwrap_or_default(),
                };
                to_json_binary(&res).into()
            }
        };
        // system result is always ok in the mock implementation
        SystemResult::Ok(contract_result)
    }

    /// Helper method to get all rewards for a given delegator.
    #[cfg(feature = "cosmwasm_1_4")]
    fn validator_rewards(&self, delegator_address: &str) -> Option<Vec<crate::DelegatorReward>> {
        let validator_rewards = self.rewards.get(delegator_address)?;

        Some(
            validator_rewards
                .iter()
                .map(|(validator, rewards)| crate::DelegatorReward {
                    validator_address: validator.clone(),
                    reward: rewards.clone(),
                })
                .collect(),
        )
    }
}

pub fn digit_sum(input: &[u8]) -> usize {
    input.iter().fold(0, |sum, val| sum + (*val as usize))
}

/// Only for test code. This bypasses assertions in new, allowing us to create _*
/// Attributes to simulate responses from the blockchain
pub fn mock_wasmd_attr(key: impl Into<String>, value: impl Into<String>) -> Attribute {
    Attribute {
        key: key.into(),
        value: value.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "cosmwasm_1_3")]
    use crate::DenomUnit;
    use crate::{coin, coins, from_json, to_json_binary, ContractInfoResponse, Response};
    #[cfg(feature = "staking")]
    use crate::{Decimal, Delegation};
    use hex_literal::hex;
    use serde::Deserialize;

    const SECP256K1_MSG_HASH_HEX: &str =
        "5ae8317d34d1e595e3fa7247db80c0af4320cce1116de187f8f7e2e099c0d8d0";
    const SECP256K1_SIG_HEX: &str = "207082eb2c3dfa0b454e0906051270ba4074ac93760ba9e7110cd9471475111151eb0dbbc9920e72146fb564f99d039802bf6ef2561446eb126ef364d21ee9c4";
    const SECP256K1_PUBKEY_HEX: &str = "04051c1ee2190ecfb174bfe4f90763f2b4ff7517b70a2aec1876ebcfd644c4633fb03f3cfbd94b1f376e34592d9d41ccaf640bb751b00a1fadeb0c01157769eb73";

    const ED25519_MSG_HEX: &str = "72";
    const ED25519_SIG_HEX: &str = "92a009a9f0d4cab8720e820b5f642540a2b27b5416503f8fb3762223ebdb69da085ac1e43e15996e458f3613d0f11d8c387b2eaeb4302aeeb00d291612bb0c00";
    const ED25519_PUBKEY_HEX: &str =
        "3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c";

    #[test]
    fn mock_info_works() {
        let info = mock_info("my name", &coins(100, "atom"));
        assert_eq!(
            info,
            MessageInfo {
                sender: Addr::unchecked("my name"),
                funds: vec![Coin {
                    amount: 100u128.into(),
                    denom: "atom".into(),
                }]
            }
        );
    }

    #[test]
    fn addr_validate_works() {
        let api = MockApi::default();

        // valid
        let addr = api.addr_validate("foobar123").unwrap();
        assert_eq!(addr, "foobar123");

        // invalid: too short
        api.addr_validate("").unwrap_err();
        // invalid: not normalized
        api.addr_validate("Foobar123").unwrap_err();
        api.addr_validate("FOOBAR123").unwrap_err();
    }

    #[test]
    fn addr_canonicalize_works() {
        let api = MockApi::default();

        api.addr_canonicalize("foobar123").unwrap();

        // is case insensitive
        let data1 = api.addr_canonicalize("foo123").unwrap();
        let data2 = api.addr_canonicalize("FOO123").unwrap();
        assert_eq!(data1, data2);
    }

    #[test]
    fn canonicalize_and_humanize_restores_original() {
        let api = MockApi::default();

        // simple
        let original = String::from("shorty");
        let canonical = api.addr_canonicalize(&original).unwrap();
        let recovered = api.addr_humanize(&canonical).unwrap();
        assert_eq!(recovered, original);

        // normalizes input
        let original = String::from("CosmWasmChef");
        let canonical = api.addr_canonicalize(&original).unwrap();
        let recovered = api.addr_humanize(&canonical).unwrap();
        assert_eq!(recovered, "cosmwasmchef");

        // Long input (Juno contract address)
        let original =
            String::from("juno1v82su97skv6ucfqvuvswe0t5fph7pfsrtraxf0x33d8ylj5qnrysdvkc95");
        let canonical = api.addr_canonicalize(&original).unwrap();
        let recovered = api.addr_humanize(&canonical).unwrap();
        assert_eq!(recovered, original);
    }

    #[test]
    fn addr_canonicalize_min_input_length() {
        let api = MockApi::default();
        let human = String::from("1");
        let err = api.addr_canonicalize(&human).unwrap_err();
        assert!(err
            .to_string()
            .contains("human address too short for this mock implementation (must be >= 3)"));
    }

    #[test]
    fn addr_canonicalize_max_input_length() {
        let api = MockApi::default();
        let human =
            String::from("some-extremely-long-address-not-supported-by-this-api-longer-than-supported------------------------");
        let err = api.addr_canonicalize(&human).unwrap_err();
        assert!(err
            .to_string()
            .contains("human address too long for this mock implementation (must be <= 90)"));
    }

    #[test]
    #[should_panic(expected = "length not correct")]
    fn addr_humanize_input_length() {
        let api = MockApi::default();
        let input = CanonicalAddr::from(vec![61; 11]);
        api.addr_humanize(&input).unwrap();
    }

    // Basic "works" test. Exhaustive tests on VM's side (packages/vm/src/imports.rs)
    #[test]
    fn secp256k1_verify_works() {
        let api = MockApi::default();

        let hash = hex::decode(SECP256K1_MSG_HASH_HEX).unwrap();
        let signature = hex::decode(SECP256K1_SIG_HEX).unwrap();
        let public_key = hex::decode(SECP256K1_PUBKEY_HEX).unwrap();

        assert!(api
            .secp256k1_verify(&hash, &signature, &public_key)
            .unwrap());
    }

    // Basic "fails" test. Exhaustive tests on VM's side (packages/vm/src/imports.rs)
    #[test]
    fn secp256k1_verify_fails() {
        let api = MockApi::default();

        let mut hash = hex::decode(SECP256K1_MSG_HASH_HEX).unwrap();
        // alter hash
        hash[0] ^= 0x01;
        let signature = hex::decode(SECP256K1_SIG_HEX).unwrap();
        let public_key = hex::decode(SECP256K1_PUBKEY_HEX).unwrap();

        assert!(!api
            .secp256k1_verify(&hash, &signature, &public_key)
            .unwrap());
    }

    // Basic "errors" test. Exhaustive tests on VM's side (packages/vm/src/imports.rs)
    #[test]
    fn secp256k1_verify_errs() {
        let api = MockApi::default();

        let hash = hex::decode(SECP256K1_MSG_HASH_HEX).unwrap();
        let signature = hex::decode(SECP256K1_SIG_HEX).unwrap();
        let public_key = vec![];

        let res = api.secp256k1_verify(&hash, &signature, &public_key);
        assert_eq!(res.unwrap_err(), VerificationError::InvalidPubkeyFormat);
    }

    #[test]
    fn secp256k1_recover_pubkey_works() {
        let api = MockApi::default();

        // https://gist.github.com/webmaster128/130b628d83621a33579751846699ed15
        let hash = hex!("5ae8317d34d1e595e3fa7247db80c0af4320cce1116de187f8f7e2e099c0d8d0");
        let signature = hex!("45c0b7f8c09a9e1f1cea0c25785594427b6bf8f9f878a8af0b1abbb48e16d0920d8becd0c220f67c51217eecfd7184ef0732481c843857e6bc7fc095c4f6b788");
        let recovery_param = 1;
        let expected = hex!("044a071e8a6e10aada2b8cf39fa3b5fb3400b04e99ea8ae64ceea1a977dbeaf5d5f8c8fbd10b71ab14cd561f7df8eb6da50f8a8d81ba564342244d26d1d4211595");

        let pubkey = api
            .secp256k1_recover_pubkey(&hash, &signature, recovery_param)
            .unwrap();
        assert_eq!(pubkey, expected);
    }

    #[test]
    fn secp256k1_recover_pubkey_fails_for_wrong_recovery_param() {
        let api = MockApi::default();

        // https://gist.github.com/webmaster128/130b628d83621a33579751846699ed15
        let hash = hex!("5ae8317d34d1e595e3fa7247db80c0af4320cce1116de187f8f7e2e099c0d8d0");
        let signature = hex!("45c0b7f8c09a9e1f1cea0c25785594427b6bf8f9f878a8af0b1abbb48e16d0920d8becd0c220f67c51217eecfd7184ef0732481c843857e6bc7fc095c4f6b788");
        let _recovery_param = 1;
        let expected = hex!("044a071e8a6e10aada2b8cf39fa3b5fb3400b04e99ea8ae64ceea1a977dbeaf5d5f8c8fbd10b71ab14cd561f7df8eb6da50f8a8d81ba564342244d26d1d4211595");

        // Wrong recovery param leads to different pubkey
        let pubkey = api.secp256k1_recover_pubkey(&hash, &signature, 0).unwrap();
        assert_eq!(pubkey.len(), 65);
        assert_ne!(pubkey, expected);

        // Invalid recovery param leads to error
        let result = api.secp256k1_recover_pubkey(&hash, &signature, 42);
        match result.unwrap_err() {
            RecoverPubkeyError::InvalidRecoveryParam => {}
            err => panic!("Unexpected error: {err:?}"),
        }
    }

    #[test]
    fn secp256k1_recover_pubkey_fails_for_wrong_hash() {
        let api = MockApi::default();

        // https://gist.github.com/webmaster128/130b628d83621a33579751846699ed15
        let hash = hex!("5ae8317d34d1e595e3fa7247db80c0af4320cce1116de187f8f7e2e099c0d8d0");
        let signature = hex!("45c0b7f8c09a9e1f1cea0c25785594427b6bf8f9f878a8af0b1abbb48e16d0920d8becd0c220f67c51217eecfd7184ef0732481c843857e6bc7fc095c4f6b788");
        let recovery_param = 1;
        let expected = hex!("044a071e8a6e10aada2b8cf39fa3b5fb3400b04e99ea8ae64ceea1a977dbeaf5d5f8c8fbd10b71ab14cd561f7df8eb6da50f8a8d81ba564342244d26d1d4211595");

        // Wrong hash
        let mut corrupted_hash = hash;
        corrupted_hash[0] ^= 0x01;
        let pubkey = api
            .secp256k1_recover_pubkey(&corrupted_hash, &signature, recovery_param)
            .unwrap();
        assert_eq!(pubkey.len(), 65);
        assert_ne!(pubkey, expected);

        // Malformed hash
        let mut malformed_hash = hash.to_vec();
        malformed_hash.push(0x8a);
        let result = api.secp256k1_recover_pubkey(&malformed_hash, &signature, recovery_param);
        match result.unwrap_err() {
            RecoverPubkeyError::InvalidHashFormat => {}
            err => panic!("Unexpected error: {err:?}"),
        }
    }

    // Basic "works" test. Exhaustive tests on VM's side (packages/vm/src/imports.rs)
    #[test]
    fn ed25519_verify_works() {
        let api = MockApi::default();

        let msg = hex::decode(ED25519_MSG_HEX).unwrap();
        let signature = hex::decode(ED25519_SIG_HEX).unwrap();
        let public_key = hex::decode(ED25519_PUBKEY_HEX).unwrap();

        assert!(api.ed25519_verify(&msg, &signature, &public_key).unwrap());
    }

    // Basic "fails" test. Exhaustive tests on VM's side (packages/vm/src/imports.rs)
    #[test]
    fn ed25519_verify_fails() {
        let api = MockApi::default();

        let mut msg = hex::decode(ED25519_MSG_HEX).unwrap();
        // alter msg
        msg[0] ^= 0x01;
        let signature = hex::decode(ED25519_SIG_HEX).unwrap();
        let public_key = hex::decode(ED25519_PUBKEY_HEX).unwrap();

        assert!(!api.ed25519_verify(&msg, &signature, &public_key).unwrap());
    }

    // Basic "errors" test. Exhaustive tests on VM's side (packages/vm/src/imports.rs)
    #[test]
    fn ed25519_verify_errs() {
        let api = MockApi::default();

        let msg = hex::decode(ED25519_MSG_HEX).unwrap();
        let signature = hex::decode(ED25519_SIG_HEX).unwrap();
        let public_key = vec![];

        let res = api.ed25519_verify(&msg, &signature, &public_key);
        assert_eq!(res.unwrap_err(), VerificationError::InvalidPubkeyFormat);
    }

    // Basic "works" test.
    #[test]
    fn ed25519_batch_verify_works() {
        let api = MockApi::default();

        let msg = hex::decode(ED25519_MSG_HEX).unwrap();
        let signature = hex::decode(ED25519_SIG_HEX).unwrap();
        let public_key = hex::decode(ED25519_PUBKEY_HEX).unwrap();

        let msgs: Vec<&[u8]> = vec![&msg];
        let signatures: Vec<&[u8]> = vec![&signature];
        let public_keys: Vec<&[u8]> = vec![&public_key];

        assert!(api
            .ed25519_batch_verify(&msgs, &signatures, &public_keys)
            .unwrap());
    }

    // Basic "fails" test.
    #[test]
    fn ed25519_batch_verify_fails() {
        let api = MockApi::default();

        let mut msg = hex::decode(ED25519_MSG_HEX).unwrap();
        // alter msg
        msg[0] ^= 0x01;
        let signature = hex::decode(ED25519_SIG_HEX).unwrap();
        let public_key = hex::decode(ED25519_PUBKEY_HEX).unwrap();

        let msgs: Vec<&[u8]> = vec![&msg];
        let signatures: Vec<&[u8]> = vec![&signature];
        let public_keys: Vec<&[u8]> = vec![&public_key];

        assert!(!api
            .ed25519_batch_verify(&msgs, &signatures, &public_keys)
            .unwrap());
    }

    // Basic "errors" test.
    #[test]
    fn ed25519_batch_verify_errs() {
        let api = MockApi::default();

        let msg = hex::decode(ED25519_MSG_HEX).unwrap();
        let signature = hex::decode(ED25519_SIG_HEX).unwrap();
        let public_key: Vec<u8> = vec![0u8; 0];

        let msgs: Vec<&[u8]> = vec![msg.as_slice()];
        let signatures: Vec<&[u8]> = vec![signature.as_slice()];
        let public_keys: Vec<&[u8]> = vec![&public_key];

        let res = api.ed25519_batch_verify(&msgs, &signatures, &public_keys);
        assert_eq!(res.unwrap_err(), VerificationError::InvalidPubkeyFormat);
    }

    #[cfg(feature = "cosmwasm_1_1")]
    #[test]
    fn bank_querier_supply() {
        let addr1 = String::from("foo");
        let balance1 = vec![coin(123, "ELF"), coin(777, "FLY")];

        let addr2 = String::from("bar");
        let balance2 = coins(321, "ELF");

        let bank = BankQuerier::new(&[(&addr1, &balance1), (&addr2, &balance2)]);

        let elf = bank
            .query(&BankQuery::Supply {
                denom: "ELF".to_string(),
            })
            .unwrap()
            .unwrap();
        let res: SupplyResponse = from_json(elf).unwrap();
        assert_eq!(res.amount, coin(444, "ELF"));

        let fly = bank
            .query(&BankQuery::Supply {
                denom: "FLY".to_string(),
            })
            .unwrap()
            .unwrap();
        let res: SupplyResponse = from_json(fly).unwrap();
        assert_eq!(res.amount, coin(777, "FLY"));

        // if a denom does not exist, should return zero amount, instead of throwing an error
        let atom = bank
            .query(&BankQuery::Supply {
                denom: "ATOM".to_string(),
            })
            .unwrap()
            .unwrap();
        let res: SupplyResponse = from_json(atom).unwrap();
        assert_eq!(res.amount, coin(0, "ATOM"));
    }

    #[test]
    fn bank_querier_all_balances() {
        let addr = String::from("foobar");
        let balance = vec![coin(123, "ELF"), coin(777, "FLY")];
        let bank = BankQuerier::new(&[(&addr, &balance)]);

        let all = bank
            .query(&BankQuery::AllBalances { address: addr })
            .unwrap()
            .unwrap();
        let res: AllBalanceResponse = from_json(all).unwrap();
        assert_eq!(&res.amount, &balance);
    }

    #[test]
    fn bank_querier_one_balance() {
        let addr = String::from("foobar");
        let balance = vec![coin(123, "ELF"), coin(777, "FLY")];
        let bank = BankQuerier::new(&[(&addr, &balance)]);

        // one match
        let fly = bank
            .query(&BankQuery::Balance {
                address: addr.clone(),
                denom: "FLY".to_string(),
            })
            .unwrap()
            .unwrap();
        let res: BalanceResponse = from_json(fly).unwrap();
        assert_eq!(res.amount, coin(777, "FLY"));

        // missing denom
        let miss = bank
            .query(&BankQuery::Balance {
                address: addr,
                denom: "MISS".to_string(),
            })
            .unwrap()
            .unwrap();
        let res: BalanceResponse = from_json(miss).unwrap();
        assert_eq!(res.amount, coin(0, "MISS"));
    }

    #[test]
    fn bank_querier_missing_account() {
        let addr = String::from("foobar");
        let balance = vec![coin(123, "ELF"), coin(777, "FLY")];
        let bank = BankQuerier::new(&[(&addr, &balance)]);

        // all balances on empty account is empty vec
        let all = bank
            .query(&BankQuery::AllBalances {
                address: String::from("elsewhere"),
            })
            .unwrap()
            .unwrap();
        let res: AllBalanceResponse = from_json(all).unwrap();
        assert_eq!(res.amount, vec![]);

        // any denom on balances on empty account is empty coin
        let miss = bank
            .query(&BankQuery::Balance {
                address: String::from("elsewhere"),
                denom: "ELF".to_string(),
            })
            .unwrap()
            .unwrap();
        let res: BalanceResponse = from_json(miss).unwrap();
        assert_eq!(res.amount, coin(0, "ELF"));
    }

    #[cfg(feature = "cosmwasm_1_3")]
    #[test]
    fn bank_querier_metadata_works() {
        let mut bank = BankQuerier::new(&[]);
        bank.set_denom_metadata(
            &(0..100)
                .map(|i| DenomMetadata {
                    symbol: format!("FOO{i}"),
                    name: "Foo".to_string(),
                    description: "Foo coin".to_string(),
                    denom_units: vec![DenomUnit {
                        denom: "ufoo".to_string(),
                        exponent: 8,
                        aliases: vec!["microfoo".to_string(), "foobar".to_string()],
                    }],
                    display: "FOO".to_string(),
                    base: format!("ufoo{i}"),
                    uri: "https://foo.bar".to_string(),
                    uri_hash: "foo".to_string(),
                })
                .collect::<Vec<_>>(),
        );

        // querying first 10 should work
        let res = bank
            .query(&BankQuery::AllDenomMetadata {
                pagination: Some(PageRequest {
                    key: None,
                    limit: 10,
                    reverse: false,
                }),
            })
            .unwrap()
            .unwrap();
        let res: AllDenomMetadataResponse = from_json(res).unwrap();
        assert_eq!(res.metadata.len(), 10);
        assert!(res.next_key.is_some());

        // querying next 10 should also work
        let res2 = bank
            .query(&BankQuery::AllDenomMetadata {
                pagination: Some(PageRequest {
                    key: res.next_key,
                    limit: 10,
                    reverse: false,
                }),
            })
            .unwrap()
            .unwrap();
        let res2: AllDenomMetadataResponse = from_json(res2).unwrap();
        assert_eq!(res2.metadata.len(), 10);
        assert_ne!(res.metadata.last(), res2.metadata.first());
        // should have no overlap
        for m in res.metadata {
            assert!(!res2.metadata.contains(&m));
        }

        // querying all 100 should work
        let res = bank
            .query(&BankQuery::AllDenomMetadata {
                pagination: Some(PageRequest {
                    key: None,
                    limit: 100,
                    reverse: true,
                }),
            })
            .unwrap()
            .unwrap();
        let res: AllDenomMetadataResponse = from_json(res).unwrap();
        assert_eq!(res.metadata.len(), 100);
        assert!(res.next_key.is_none(), "no more data should be available");
        assert_eq!(res.metadata[0].symbol, "FOO99", "should have been reversed");

        let more_res = bank
            .query(&BankQuery::AllDenomMetadata {
                pagination: Some(PageRequest {
                    key: res.next_key,
                    limit: u32::MAX,
                    reverse: true,
                }),
            })
            .unwrap()
            .unwrap();
        let more_res: AllDenomMetadataResponse = from_json(more_res).unwrap();
        assert_eq!(
            more_res.metadata, res.metadata,
            "should be same as previous query"
        );
    }

    #[cfg(feature = "cosmwasm_1_3")]
    #[test]
    fn distribution_querier_delegator_withdraw_address() {
        let mut distribution = DistributionQuerier::default();
        distribution.set_withdraw_address("addr0", "withdraw0");

        let query = DistributionQuery::DelegatorWithdrawAddress {
            delegator_address: "addr0".to_string(),
        };

        let res = distribution.query(&query).unwrap().unwrap();
        let res: DelegatorWithdrawAddressResponse = from_json(res).unwrap();
        assert_eq!(res.withdraw_address, "withdraw0");

        let query = DistributionQuery::DelegatorWithdrawAddress {
            delegator_address: "addr1".to_string(),
        };

        let res = distribution.query(&query).unwrap().unwrap();
        let res: DelegatorWithdrawAddressResponse = from_json(res).unwrap();
        assert_eq!(res.withdraw_address, "addr1");
    }

    #[cfg(feature = "cosmwasm_1_4")]
    #[test]
    fn distribution_querier_delegator_validators() {
        let mut distribution = DistributionQuerier::default();
        distribution.set_validators("addr0", ["valoper1", "valoper2"]);

        let query = DistributionQuery::DelegatorValidators {
            delegator_address: "addr0".to_string(),
        };

        let res = distribution.query(&query).unwrap().unwrap();
        let res: DelegatorValidatorsResponse = from_json(res).unwrap();
        assert_eq!(res.validators, ["valoper1", "valoper2"]);

        let query = DistributionQuery::DelegatorValidators {
            delegator_address: "addr1".to_string(),
        };

        let res = distribution.query(&query).unwrap().unwrap();
        let res: DelegatorValidatorsResponse = from_json(res).unwrap();
        assert_eq!(res.validators, ([] as [String; 0]));
    }

    #[cfg(feature = "cosmwasm_1_4")]
    #[test]
    fn distribution_querier_delegation_rewards() {
        use crate::{Decimal256, DelegationTotalRewardsResponse, DelegatorReward};

        let mut distribution = DistributionQuerier::default();
        let valoper0_rewards = vec![
            DecCoin::new(Decimal256::from_atomics(1234u128, 0).unwrap(), "uatom"),
            DecCoin::new(Decimal256::from_atomics(56781234u128, 4).unwrap(), "utest"),
        ];
        distribution.set_rewards("valoper0", "addr0", valoper0_rewards.clone());

        // both exist / are set
        let query = DistributionQuery::DelegationRewards {
            delegator_address: "addr0".to_string(),
            validator_address: "valoper0".to_string(),
        };
        let res = distribution.query(&query).unwrap().unwrap();
        let res: DelegationRewardsResponse = from_json(res).unwrap();
        assert_eq!(res.rewards, valoper0_rewards);

        // delegator does not exist
        let query = DistributionQuery::DelegationRewards {
            delegator_address: "nonexistent".to_string(),
            validator_address: "valoper0".to_string(),
        };
        let res = distribution.query(&query).unwrap().unwrap();
        let res: DelegationRewardsResponse = from_json(res).unwrap();
        assert_eq!(res.rewards.len(), 0);

        // validator does not exist
        let query = DistributionQuery::DelegationRewards {
            delegator_address: "addr0".to_string(),
            validator_address: "valopernonexistent".to_string(),
        };
        let res = distribution.query(&query).unwrap().unwrap();
        let res: DelegationRewardsResponse = from_json(res).unwrap();
        assert_eq!(res.rewards.len(), 0);

        // add one more validator
        let valoper1_rewards = vec![DecCoin::new(Decimal256::one(), "uatom")];
        distribution.set_rewards("valoper1", "addr0", valoper1_rewards.clone());

        // total rewards
        let query = DistributionQuery::DelegationTotalRewards {
            delegator_address: "addr0".to_string(),
        };
        let res = distribution.query(&query).unwrap().unwrap();
        let res: DelegationTotalRewardsResponse = from_json(res).unwrap();
        assert_eq!(
            res.rewards,
            vec![
                DelegatorReward {
                    validator_address: "valoper0".into(),
                    reward: valoper0_rewards
                },
                DelegatorReward {
                    validator_address: "valoper1".into(),
                    reward: valoper1_rewards
                },
            ]
        );
        assert_eq!(
            res.total,
            [
                DecCoin::new(
                    Decimal256::from_atomics(1234u128, 0).unwrap() + Decimal256::one(),
                    "uatom"
                ),
                // total for utest should still be the same
                DecCoin::new(Decimal256::from_atomics(56781234u128, 4).unwrap(), "utest")
            ]
        );
    }

    #[cfg(feature = "stargate")]
    #[test]
    fn ibc_querier_channel_existing() {
        let chan1 = mock_ibc_channel("channel-0", IbcOrder::Ordered, "ibc");
        let chan2 = mock_ibc_channel("channel-1", IbcOrder::Ordered, "ibc");

        let ibc = IbcQuerier::new("myport", &[chan1.clone(), chan2]);

        // query existing
        let query = &IbcQuery::Channel {
            channel_id: "channel-0".to_string(),
            port_id: Some("my_port".to_string()),
        };
        let raw = ibc.query(query).unwrap().unwrap();
        let chan: ChannelResponse = from_json(raw).unwrap();
        assert_eq!(chan.channel, Some(chan1));
    }

    #[cfg(feature = "stargate")]
    #[test]
    fn ibc_querier_channel_existing_no_port() {
        let chan1 = IbcChannel {
            endpoint: IbcEndpoint {
                port_id: "myport".to_string(),
                channel_id: "channel-0".to_string(),
            },
            counterparty_endpoint: IbcEndpoint {
                port_id: "their_port".to_string(),
                channel_id: "channel-7".to_string(),
            },
            order: IbcOrder::Ordered,
            version: "ibc".to_string(),
            connection_id: "connection-2".to_string(),
        };
        let chan2 = mock_ibc_channel("channel-1", IbcOrder::Ordered, "ibc");

        let ibc = IbcQuerier::new("myport", &[chan1.clone(), chan2]);

        // query existing
        let query = &IbcQuery::Channel {
            channel_id: "channel-0".to_string(),
            port_id: Some("myport".to_string()),
        };
        let raw = ibc.query(query).unwrap().unwrap();
        let chan: ChannelResponse = from_json(raw).unwrap();
        assert_eq!(chan.channel, Some(chan1));
    }

    #[cfg(feature = "stargate")]
    #[test]
    fn ibc_querier_channel_none() {
        let chan1 = mock_ibc_channel("channel-0", IbcOrder::Ordered, "ibc");
        let chan2 = mock_ibc_channel("channel-1", IbcOrder::Ordered, "ibc");

        let ibc = IbcQuerier::new("myport", &[chan1, chan2]);

        // query non-existing
        let query = &IbcQuery::Channel {
            channel_id: "channel-0".to_string(),
            port_id: None,
        };
        let raw = ibc.query(query).unwrap().unwrap();
        let chan: ChannelResponse = from_json(raw).unwrap();
        assert_eq!(chan.channel, None);
    }

    #[cfg(feature = "stargate")]
    #[test]
    fn ibc_querier_channels_matching() {
        let chan1 = mock_ibc_channel("channel-0", IbcOrder::Ordered, "ibc");
        let chan2 = mock_ibc_channel("channel-1", IbcOrder::Ordered, "ibc");

        let ibc = IbcQuerier::new("myport", &[chan1.clone(), chan2.clone()]);

        // query channels matching "my_port" (should match both above)
        let query = &IbcQuery::ListChannels {
            port_id: Some("my_port".to_string()),
        };
        let raw = ibc.query(query).unwrap().unwrap();
        let res: ListChannelsResponse = from_json(raw).unwrap();
        assert_eq!(res.channels, vec![chan1, chan2]);
    }

    #[cfg(feature = "stargate")]
    #[test]
    fn ibc_querier_channels_no_matching() {
        let chan1 = mock_ibc_channel("channel-0", IbcOrder::Ordered, "ibc");
        let chan2 = mock_ibc_channel("channel-1", IbcOrder::Ordered, "ibc");

        let ibc = IbcQuerier::new("myport", &[chan1, chan2]);

        // query channels matching "myport" (should be none)
        let query = &IbcQuery::ListChannels { port_id: None };
        let raw = ibc.query(query).unwrap().unwrap();
        let res: ListChannelsResponse = from_json(raw).unwrap();
        assert_eq!(res.channels, vec![]);
    }

    #[cfg(feature = "stargate")]
    #[test]
    fn ibc_querier_port() {
        let chan1 = mock_ibc_channel("channel-0", IbcOrder::Ordered, "ibc");

        let ibc = IbcQuerier::new("myport", &[chan1]);

        // query channels matching "myport" (should be none)
        let query = &IbcQuery::PortId {};
        let raw = ibc.query(query).unwrap().unwrap();
        let res: PortIdResponse = from_json(raw).unwrap();
        assert_eq!(res.port_id, "myport");
    }

    #[cfg(feature = "staking")]
    #[test]
    fn staking_querier_all_validators() {
        let val1 = Validator {
            address: String::from("validator-one"),
            commission: Decimal::percent(1),
            max_commission: Decimal::percent(3),
            max_change_rate: Decimal::percent(1),
        };
        let val2 = Validator {
            address: String::from("validator-two"),
            commission: Decimal::permille(15),
            max_commission: Decimal::permille(40),
            max_change_rate: Decimal::permille(5),
        };

        let staking = StakingQuerier::new("ustake", &[val1.clone(), val2.clone()], &[]);

        // one match
        let raw = staking
            .query(&StakingQuery::AllValidators {})
            .unwrap()
            .unwrap();
        let vals: AllValidatorsResponse = from_json(raw).unwrap();
        assert_eq!(vals.validators, vec![val1, val2]);
    }

    #[cfg(feature = "staking")]
    #[test]
    fn staking_querier_validator() {
        let address1 = String::from("validator-one");
        let address2 = String::from("validator-two");
        let address_non_existent = String::from("wannabe-validator");

        let val1 = Validator {
            address: address1.clone(),
            commission: Decimal::percent(1),
            max_commission: Decimal::percent(3),
            max_change_rate: Decimal::percent(1),
        };
        let val2 = Validator {
            address: address2.clone(),
            commission: Decimal::permille(15),
            max_commission: Decimal::permille(40),
            max_change_rate: Decimal::permille(5),
        };

        let staking = StakingQuerier::new("ustake", &[val1.clone(), val2.clone()], &[]);

        // query 1
        let raw = staking
            .query(&StakingQuery::Validator { address: address1 })
            .unwrap()
            .unwrap();
        let res: ValidatorResponse = from_json(raw).unwrap();
        assert_eq!(res.validator, Some(val1));

        // query 2
        let raw = staking
            .query(&StakingQuery::Validator { address: address2 })
            .unwrap()
            .unwrap();
        let res: ValidatorResponse = from_json(raw).unwrap();
        assert_eq!(res.validator, Some(val2));

        // query non-existent
        let raw = staking
            .query(&StakingQuery::Validator {
                address: address_non_existent,
            })
            .unwrap()
            .unwrap();
        let res: ValidatorResponse = from_json(raw).unwrap();
        assert_eq!(res.validator, None);
    }

    #[cfg(feature = "staking")]
    // gets delegators from query or panic
    fn get_all_delegators(
        staking: &StakingQuerier,
        delegator: impl Into<String>,
    ) -> Vec<Delegation> {
        let raw = staking
            .query(&StakingQuery::AllDelegations {
                delegator: delegator.into(),
            })
            .unwrap()
            .unwrap();
        let dels: AllDelegationsResponse = from_json(raw).unwrap();
        dels.delegations
    }

    #[cfg(feature = "staking")]
    // gets full delegators from query or panic
    fn get_delegator(
        staking: &StakingQuerier,
        delegator: impl Into<String>,
        validator: impl Into<String>,
    ) -> Option<FullDelegation> {
        let raw = staking
            .query(&StakingQuery::Delegation {
                delegator: delegator.into(),
                validator: validator.into(),
            })
            .unwrap()
            .unwrap();
        let dels: DelegationResponse = from_json(raw).unwrap();
        dels.delegation
    }

    #[cfg(feature = "staking")]
    #[test]
    fn staking_querier_delegations() {
        let val1 = String::from("validator-one");
        let val2 = String::from("validator-two");

        let user_a = Addr::unchecked("investor");
        let user_b = Addr::unchecked("speculator");
        let user_c = Addr::unchecked("hodler");

        // we need multiple validators per delegator, so the queries provide different results
        let del1a = FullDelegation {
            delegator: user_a.clone(),
            validator: val1.clone(),
            amount: coin(100, "ustake"),
            can_redelegate: coin(100, "ustake"),
            accumulated_rewards: coins(5, "ustake"),
        };
        let del2a = FullDelegation {
            delegator: user_a.clone(),
            validator: val2.clone(),
            amount: coin(500, "ustake"),
            can_redelegate: coin(500, "ustake"),
            accumulated_rewards: coins(20, "ustake"),
        };

        // note we cannot have multiple delegations on one validator, they are collapsed into one
        let del1b = FullDelegation {
            delegator: user_b.clone(),
            validator: val1.clone(),
            amount: coin(500, "ustake"),
            can_redelegate: coin(0, "ustake"),
            accumulated_rewards: coins(0, "ustake"),
        };

        // and another one on val2
        let del2c = FullDelegation {
            delegator: user_c.clone(),
            validator: val2.clone(),
            amount: coin(8888, "ustake"),
            can_redelegate: coin(4567, "ustake"),
            accumulated_rewards: coins(900, "ustake"),
        };

        let staking = StakingQuerier::new(
            "ustake",
            &[],
            &[del1a.clone(), del1b.clone(), del2a.clone(), del2c.clone()],
        );

        // get all for user a
        let dels = get_all_delegators(&staking, user_a.clone());
        assert_eq!(dels, vec![del1a.clone().into(), del2a.clone().into()]);

        // get all for user b
        let dels = get_all_delegators(&staking, user_b.clone());
        assert_eq!(dels, vec![del1b.clone().into()]);

        // get all for user c
        let dels = get_all_delegators(&staking, user_c.clone());
        assert_eq!(dels, vec![del2c.clone().into()]);

        // for user with no delegations...
        let dels = get_all_delegators(&staking, String::from("no one"));
        assert_eq!(dels, vec![]);

        // filter a by validator (1 and 1)
        let dels = get_delegator(&staking, user_a.clone(), val1.clone());
        assert_eq!(dels, Some(del1a));
        let dels = get_delegator(&staking, user_a, val2.clone());
        assert_eq!(dels, Some(del2a));

        // filter b by validator (2 and 0)
        let dels = get_delegator(&staking, user_b.clone(), val1.clone());
        assert_eq!(dels, Some(del1b));
        let dels = get_delegator(&staking, user_b, val2.clone());
        assert_eq!(dels, None);

        // filter c by validator (0 and 1)
        let dels = get_delegator(&staking, user_c.clone(), val1);
        assert_eq!(dels, None);
        let dels = get_delegator(&staking, user_c, val2);
        assert_eq!(dels, Some(del2c));
    }

    #[test]
    fn wasm_querier_works() {
        let mut querier = WasmQuerier::default();

        let any_addr = "foo".to_string();

        // By default, querier errors for WasmQuery::Raw
        let system_err = querier
            .query(&WasmQuery::Raw {
                contract_addr: any_addr.clone(),
                key: b"the key".into(),
            })
            .unwrap_err();
        match system_err {
            SystemError::NoSuchContract { addr } => assert_eq!(addr, any_addr),
            err => panic!("Unexpected error: {err:?}"),
        }

        // By default, querier errors for WasmQuery::Smart
        let system_err = querier
            .query(&WasmQuery::Smart {
                contract_addr: any_addr.clone(),
                msg: b"{}".into(),
            })
            .unwrap_err();
        match system_err {
            SystemError::NoSuchContract { addr } => assert_eq!(addr, any_addr),
            err => panic!("Unexpected error: {err:?}"),
        }

        // By default, querier errors for WasmQuery::ContractInfo
        let system_err = querier
            .query(&WasmQuery::ContractInfo {
                contract_addr: any_addr.clone(),
            })
            .unwrap_err();
        match system_err {
            SystemError::NoSuchContract { addr } => assert_eq!(addr, any_addr),
            err => panic!("Unexpected error: {err:?}"),
        }

        #[cfg(feature = "cosmwasm_1_2")]
        {
            // By default, querier errors for WasmQuery::CodeInfo
            let system_err = querier
                .query(&WasmQuery::CodeInfo { code_id: 4 })
                .unwrap_err();
            match system_err {
                SystemError::NoSuchCode { code_id } => assert_eq!(code_id, 4),
                err => panic!("Unexpected error: {err:?}"),
            }
        }

        querier.update_handler(|request| {
            let constract1 = Addr::unchecked("contract1");
            let mut storage1 = HashMap::<Binary, Binary>::default();
            storage1.insert(b"the key".into(), b"the value".into());

            match request {
                WasmQuery::Raw { contract_addr, key } => {
                    if *contract_addr == constract1 {
                        if let Some(value) = storage1.get(key) {
                            SystemResult::Ok(ContractResult::Ok(value.clone()))
                        } else {
                            SystemResult::Ok(ContractResult::Ok(Binary::default()))
                        }
                    } else {
                        SystemResult::Err(SystemError::NoSuchContract {
                            addr: contract_addr.clone(),
                        })
                    }
                }
                WasmQuery::Smart { contract_addr, msg } => {
                    if *contract_addr == constract1 {
                        #[derive(Deserialize)]
                        struct MyMsg {}
                        let _msg: MyMsg = match from_json(msg) {
                            Ok(msg) => msg,
                            Err(err) => {
                                return SystemResult::Ok(ContractResult::Err(err.to_string()))
                            }
                        };
                        let response: Response = Response::new().set_data(b"good");
                        SystemResult::Ok(ContractResult::Ok(to_json_binary(&response).unwrap()))
                    } else {
                        SystemResult::Err(SystemError::NoSuchContract {
                            addr: contract_addr.clone(),
                        })
                    }
                }
                WasmQuery::ContractInfo { contract_addr } => {
                    if *contract_addr == constract1 {
                        let response = ContractInfoResponse {
                            code_id: 4,
                            creator: "lalala".into(),
                            admin: None,
                            pinned: false,
                            ibc_port: None,
                        };
                        SystemResult::Ok(ContractResult::Ok(to_json_binary(&response).unwrap()))
                    } else {
                        SystemResult::Err(SystemError::NoSuchContract {
                            addr: contract_addr.clone(),
                        })
                    }
                }
                #[cfg(feature = "cosmwasm_1_2")]
                WasmQuery::CodeInfo { code_id } => {
                    use crate::{CodeInfoResponse, HexBinary};
                    let code_id = *code_id;
                    if code_id == 4 {
                        let response = CodeInfoResponse {
                            code_id,
                            creator: "lalala".into(),
                            checksum: HexBinary::from_hex(
                                "84cf20810fd429caf58898c3210fcb71759a27becddae08dbde8668ea2f4725d",
                            )
                            .unwrap(),
                        };
                        SystemResult::Ok(ContractResult::Ok(to_json_binary(&response).unwrap()))
                    } else {
                        SystemResult::Err(SystemError::NoSuchCode { code_id })
                    }
                }
            }
        });

        // WasmQuery::Raw
        let result = querier.query(&WasmQuery::Raw {
            contract_addr: "contract1".into(),
            key: b"the key".into(),
        });
        match result {
            SystemResult::Ok(ContractResult::Ok(value)) => assert_eq!(value, b"the value" as &[u8]),
            res => panic!("Unexpected result: {res:?}"),
        }
        let result = querier.query(&WasmQuery::Raw {
            contract_addr: "contract1".into(),
            key: b"other key".into(),
        });
        match result {
            SystemResult::Ok(ContractResult::Ok(value)) => assert_eq!(value, b"" as &[u8]),
            res => panic!("Unexpected result: {res:?}"),
        }

        // WasmQuery::Smart
        let result = querier.query(&WasmQuery::Smart {
            contract_addr: "contract1".into(),
            msg: b"{}".into(),
        });
        match result {
            SystemResult::Ok(ContractResult::Ok(value)) => assert_eq!(
                value,
                br#"{"messages":[],"attributes":[],"events":[],"data":"Z29vZA=="}"# as &[u8]
            ),
            res => panic!("Unexpected result: {res:?}"),
        }
        let result = querier.query(&WasmQuery::Smart {
            contract_addr: "contract1".into(),
            msg: b"a broken request".into(),
        });
        match result {
            SystemResult::Ok(ContractResult::Err(err)) => {
                assert_eq!(err, "Error parsing into type cosmwasm_std::testing::mock::tests::wasm_querier_works::{{closure}}::MyMsg: Invalid type")
            }
            res => panic!("Unexpected result: {res:?}"),
        }

        // WasmQuery::ContractInfo
        let result = querier.query(&WasmQuery::ContractInfo {
            contract_addr: "contract1".into(),
        });
        match result {
            SystemResult::Ok(ContractResult::Ok(value)) => assert_eq!(
                value,
                br#"{"code_id":4,"creator":"lalala","admin":null,"pinned":false,"ibc_port":null}"#
                    as &[u8]
            ),
            res => panic!("Unexpected result: {res:?}"),
        }

        // WasmQuery::ContractInfo
        #[cfg(feature = "cosmwasm_1_2")]
        {
            let result = querier.query(&WasmQuery::CodeInfo { code_id: 4 });
            match result {
                SystemResult::Ok(ContractResult::Ok(value)) => assert_eq!(
                    value,
                    br#"{"code_id":4,"creator":"lalala","checksum":"84cf20810fd429caf58898c3210fcb71759a27becddae08dbde8668ea2f4725d"}"#
                ),
                res => panic!("Unexpected result: {res:?}"),
            }
        }
    }

    #[test]
    fn digit_sum_works() {
        assert_eq!(digit_sum(&[]), 0);
        assert_eq!(digit_sum(&[0]), 0);
        assert_eq!(digit_sum(&[0, 0]), 0);
        assert_eq!(digit_sum(&[0, 0, 0]), 0);

        assert_eq!(digit_sum(&[1, 0, 0]), 1);
        assert_eq!(digit_sum(&[0, 1, 0]), 1);
        assert_eq!(digit_sum(&[0, 0, 1]), 1);

        assert_eq!(digit_sum(&[1, 2, 3]), 6);

        assert_eq!(digit_sum(&[255, 1]), 256);
    }

    #[test]
    fn making_an_address_works() {
        let mock_api = MockApi::default();

        assert_eq!(
            mock_api.addr_make("creator").to_string(),
            "cosmwasm1h34lmpywh4upnjdg90cjf4j70aee6z8qqfspugamjp42e4q28kqs8s7vcp",
        );

        assert_eq!(
            mock_api.addr_make("").to_string(),
            "cosmwasm1uwcvgs5clswpfxhm7nyfjmaeysn6us0yvjdexn9yjkv3k7zjhp2sly4xh9",
        );

        let mock_api = MockApi::default().with_prefix("juno");
        assert_eq!(
            mock_api.addr_make("creator").to_string(),
            "juno1h34lmpywh4upnjdg90cjf4j70aee6z8qqfspugamjp42e4q28kqsksmtyp",
        );
    }

    #[test]
    #[should_panic(expected = "Generating address failed with reason: invalid length")]
    fn making_an_address_with_empty_prefix_should_panic() {
        MockApi::default().with_prefix("").addr_make("creator");
    }
}
