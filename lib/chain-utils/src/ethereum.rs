use std::{fmt::Debug, marker::PhantomData, num::NonZeroU64, ops::Div, sync::Arc};

use beacon_api::client::BeaconApiClient;
use contracts::{
    cometbls_client::CometblsClientErrors,
    ibc_channel_handshake::{IBCChannelHandshakeErrors, IBCChannelHandshakeEvents},
    ibc_client::{IBCClientErrors, IBCClientEvents},
    ibc_connection::{IBCConnectionErrors, IBCConnectionEvents},
    ibc_handler::{
        GetChannelCall, GetChannelReturn, GetClientStateCall, GetClientStateReturn,
        GetConnectionCall, GetConnectionReturn, GetConsensusStateCall, GetConsensusStateReturn,
        GetHashedPacketAcknowledgementCommitmentCall,
        GetHashedPacketAcknowledgementCommitmentReturn, GetHashedPacketCommitmentCall,
        GetHashedPacketCommitmentReturn, HasPacketReceiptCall, IBCHandler,
        IbcCoreConnectionV1ConnectionEndData, NextClientSequenceCall, NextConnectionSequenceCall,
        OwnershipTransferredFilter,
    },
    ibc_packet::{IBCPacketErrors, IBCPacketEvents, WriteAcknowledgementFilter},
    shared_types::IbcCoreChannelV1ChannelData,
};
use ethers::{
    abi::{AbiDecode, Tokenizable},
    contract::{ContractError, EthCall, EthLogDecode},
    core::k256::ecdsa,
    middleware::{NonceManagerMiddleware, SignerMiddleware},
    providers::{Middleware, Provider, ProviderError, Ws, WsClientError},
    signers::{LocalWallet, Wallet},
    utils::secret_key_to_address,
};
use frame_support_procedural::{CloneNoBound, DebugNoBound};
use futures::Future;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use typenum::Unsigned;
use unionlabs::{
    encoding::{Decode, EthAbi},
    ethereum::config::ChainSpec,
    hash::{H160, H256},
    ibc::{
        core::client::height::{Height, IsHeight},
        lightclients::{
            ethereum::{self, storage_proof::StorageProof},
            tendermint::fraction::Fraction,
        },
    },
    ics24::{
        AcknowledgementPath, ChannelEndPath, ClientConsensusStatePath, ClientStatePath,
        CommitmentPath, ConnectionPath, IbcPath, NextClientSequencePath,
        NextConnectionSequencePath, ReceiptPath,
    },
    id::{ChannelId, ClientId, PortId},
    option_unwrap,
    traits::{Chain, ClientIdOf, ClientState, FromStrExact, HeightOf},
    uint::U256,
};

/// The slot of the `mapping(bytes32 => bytes32) public commitments` mapping in the `IBCStore` contract.
pub const IBC_HANDLER_COMMITMENTS_SLOT: U256 = U256::from_limbs([0, 0, 0, 0]);

use crate::{private_key::PrivateKey, Pool};

pub type EthereumSignerMiddleware =
    SignerMiddleware<NonceManagerMiddleware<Provider<Ws>>, Wallet<ecdsa::SigningKey>>;

// NOTE: ClientType bound is temporary until I figure out a better way to deal with client types
/// An Ethereum-based chain. This can be any chain that is based off of and settles on Ethereum (i.e. Ethereum mainnet/ Sepolia, L2s such as Scroll).
pub trait EthereumChain:
    Chain<IbcStateEncoding = EthAbi, StateProof = StorageProof, ClientType = String>
{
    /// Fetch the execution height associated with the given beacon slot. For [`Ethereum`], this will simply be the execution block number, but for L2s this will fetch the settled height at the L1 block number.
    fn execution_height_of_beacon_slot(&self, slot: u64) -> impl Future<Output = u64>;

    /// The provider connected to this chain's [JSON-RPC](https://ethereum.org/en/developers/docs/apis/json-rpc/).
    fn provider(&self) -> Arc<Provider<Ws>>;

    /// The address of the [`IBCHandler`] smart contract deployed natively on this chain.
    fn ibc_handler_address(&self) -> H160;

    fn get_proof(
        &self,
        address: H160,
        location: U256,
        block: u64,
    ) -> impl Future<Output = unionlabs::ibc::lightclients::ethereum::storage_proof::StorageProof>;
}

pub trait EthereumChainExt: EthereumChain {
    /// Convenience method to construct an [`IBCHandler`] instance for this chain.
    fn ibc_handler(&self) -> IBCHandler<Provider<Ws>> {
        IBCHandler::new(self.ibc_handler_address(), self.provider())
    }
}

impl<T: EthereumChain> EthereumChainExt for T {}

impl<C: ChainSpec, S: EthereumSignersConfig> EthereumChain for Ethereum<C, S> {
    async fn execution_height_of_beacon_slot(&self, slot: u64) -> u64 {
        self.beacon_api_client
            .execution_height(beacon_api::client::BlockId::Slot(slot))
            .await
            .unwrap()
    }

    fn provider(&self) -> Arc<Provider<Ws>> {
        self.provider.clone()
    }

    fn ibc_handler_address(&self) -> H160 {
        self.ibc_handler_address
    }

    async fn get_proof(
        &self,
        address: H160,
        location: U256,
        block: u64,
    ) -> unionlabs::ibc::lightclients::ethereum::storage_proof::StorageProof {
        let proof = self
            .provider
            .get_proof(
                ethers::types::H160::from(address),
                vec![location.to_be_bytes().into()],
                Some(block.into()),
            )
            .await
            .unwrap();

        let proof = match <[_; 1]>::try_from(proof.storage_proof) {
            Ok([proof]) => proof,
            Err(invalid) => {
                panic!("received invalid response from eth_getProof, expected length of 1 but got `{invalid:#?}`");
            }
        };

        unionlabs::ibc::lightclients::ethereum::storage_proof::StorageProof {
            proofs: [unionlabs::ibc::lightclients::ethereum::proof::Proof {
                key: U256::from_be_bytes(proof.key.to_fixed_bytes()),
                value: proof.value.into(),
                proof: proof
                    .proof
                    .into_iter()
                    .map(|bytes| bytes.to_vec())
                    .collect(),
            }]
            .to_vec(),
        }
    }
}

// TODO(benluelo): Generic over middleware?
#[derive(DebugNoBound, CloneNoBound)]
pub struct Ethereum<C: ChainSpec, S: EthereumSignersConfig = ReadWrite> {
    pub chain_id: U256,

    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,

    pub ibc_handlers: S::Out,
    pub provider: Arc<Provider<Ws>>,
    pub beacon_api_client: BeaconApiClient<C>,
}

#[derive(DebugNoBound, CloneNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct Config<S: EthereumSignersConfig = ReadWrite> {
    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,

    /// The signer that will be used to submit transactions by voyager.
    // TODO: Write a custom ser/de implementation for this struct to avoid this hackery
    #[serde(skip_serializing_if = "is_unit", default)]
    pub signers: S::Config,

    /// The RPC endpoint for the execution chain.
    pub eth_rpc_api: String,
    /// The RPC endpoint for the beacon chain.
    pub eth_beacon_rpc_api: String,
}

// lol
fn is_unit<T: 'static>(_: &T) -> bool {
    std::any::TypeId::of::<T>() == std::any::TypeId::of::<()>()
}

pub trait EthereumSignersConfig: Send + Sync + 'static {
    type Config: Debug + Clone + Default + Serialize + DeserializeOwned + Send + Sync + 'static;
    type Out: Debug + Clone + Send + Sync + 'static;

    fn new(
        config: Self::Config,
        ibc_handler_address: H160,
        chain_id: u64,
        provider: Provider<Ws>,
    ) -> Self::Out;
}

pub enum Readonly {}

impl EthereumSignersConfig for Readonly {
    type Config = ();
    type Out = ();

    fn new(
        config: Self::Config,
        _ibc_handler_address: H160,
        _chain_id: u64,
        _provider: Provider<Ws>,
    ) -> Self::Out {
        config
    }
}

pub enum ReadWrite {}

impl EthereumSignersConfig for ReadWrite {
    type Config = Vec<PrivateKey<ecdsa::SigningKey>>;
    type Out = Pool<IBCHandler<EthereumSignerMiddleware>>;

    fn new(
        config: Self::Config,
        ibc_handler_address: H160,
        chain_id: u64,
        provider: Provider<Ws>,
    ) -> Self::Out {
        Pool::new(config.into_iter().map(|signer| {
            let signing_key: ecdsa::SigningKey = signer.value();
            let address = secret_key_to_address(&signing_key);

            let wallet = LocalWallet::new_with_signer(signing_key, address, chain_id);

            let signer_middleware = Arc::new(SignerMiddleware::new(
                NonceManagerMiddleware::new(provider.clone(), address),
                wallet.clone(),
            ));

            IBCHandler::new(ibc_handler_address, signer_middleware.clone())
        }))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EthereumChainType<C: ChainSpec>(PhantomData<fn() -> C>);

pub const ETHEREUM_REVISION_NUMBER: u64 = 0;

impl<C: ChainSpec> FromStrExact for EthereumChainType<C> {
    const EXPECTING: &'static str = {
        const PREFIX: [u8; 4] = *b"eth-";

        const fn concat(cs: &[u8]) -> [u8; 11] {
            [
                PREFIX[0], PREFIX[1], PREFIX[2], PREFIX[3], cs[0], cs[1], cs[2], cs[3], cs[4],
                cs[5], cs[6],
            ]
        }

        // generic_const_exprs is still incomplete :(
        const CHAIN_SPEC_LEN: usize = 7;
        assert!(
            C::EXPECTING.len() == CHAIN_SPEC_LEN,
            "ChainSpec string value is expected to be 7 bytes"
        );

        match core::str::from_utf8(const { &concat(C::EXPECTING.as_bytes()) }) {
            Ok(ok) => ok,
            Err(_) => {
                panic!()
            }
        }
    };
}

macro_rules! try_decode {
    ($($expr:expr),+) => {
        $(
            if let Ok(ok) = $expr {
                return Ok(ok);
            }
        )+
    };
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum IBCHandlerEvents {
    PacketEvent(IBCPacketEvents),
    ConnectionEvent(IBCConnectionEvents),
    ChannelEvent(IBCChannelHandshakeEvents),
    ClientEvent(IBCClientEvents),
    OwnableEvent(OwnershipTransferredFilter),
}

impl EthLogDecode for IBCHandlerEvents {
    fn decode_log(log: &ethers::abi::RawLog) -> Result<Self, ethers::abi::Error>
    where
        Self: Sized,
    {
        try_decode! {
            IBCPacketEvents::decode_log(log).map(IBCHandlerEvents::PacketEvent),
            IBCConnectionEvents::decode_log(log).map(IBCHandlerEvents::ConnectionEvent),
            IBCChannelHandshakeEvents::decode_log(log).map(IBCHandlerEvents::ChannelEvent),
            IBCClientEvents::decode_log(log).map(IBCHandlerEvents::ClientEvent),
            OwnershipTransferredFilter::decode_log(log).map(IBCHandlerEvents::OwnableEvent)
        };
        Err(ethers::abi::Error::InvalidData)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum IbcHandlerErrors {
    PacketErrors(IBCPacketErrors),
    ConnectionErrors(IBCConnectionErrors),
    ChannelErrors(IBCChannelHandshakeErrors),
    ClientErrors(IBCClientErrors),
    CometblsClientErrors(CometblsClientErrors),
}

impl AbiDecode for IbcHandlerErrors {
    fn decode(bytes: impl AsRef<[u8]>) -> Result<Self, ethers::abi::AbiError> {
        try_decode! {
            IBCPacketErrors::decode(&bytes).map(IbcHandlerErrors::PacketErrors),
            IBCConnectionErrors::decode(&bytes).map(IbcHandlerErrors::ConnectionErrors),
            IBCChannelHandshakeErrors::decode(&bytes).map(IbcHandlerErrors::ChannelErrors),
            IBCClientErrors::decode(&bytes).map(IbcHandlerErrors::ClientErrors),
            CometblsClientErrors::decode(&bytes).map(IbcHandlerErrors::CometblsClientErrors)
        };
        Err(ethers::abi::Error::InvalidData.into())
    }
}

impl<C: ChainSpec, S: EthereumSignersConfig> Chain for Ethereum<C, S> {
    type ChainType = EthereumChainType<C>;

    type SelfClientState = ethereum::client_state::ClientState;
    type SelfConsensusState = ethereum::consensus_state::ConsensusState;

    type StoredClientState<Tr: Chain> = Tr::SelfClientState;
    type StoredConsensusState<Tr: Chain> = Tr::SelfConsensusState;

    type Header = ethereum::header::Header<C>;

    type Height = Height;

    type ClientId = EthereumClientId;

    type IbcStateEncoding = EthAbi;

    type ClientType = String;

    type Error = beacon_api::errors::Error;

    type StateProof = unionlabs::ibc::lightclients::ethereum::storage_proof::StorageProof;

    fn chain_id(&self) -> <Self::SelfClientState as ClientState>::ChainId {
        self.chain_id
    }

    async fn query_latest_height(&self) -> Result<Height, Self::Error> {
        self.beacon_api_client
            .finality_update()
            .await
            .map(|response| self.make_height(response.data.attested_header.beacon.slot))
    }

    async fn query_latest_height_as_destination(&self) -> Result<Height, Self::Error> {
        let height = self
            .beacon_api_client
            .block(beacon_api::client::BlockId::Head)
            .await?
            .data
            .message
            .slot;

        // HACK: we introduced this because we were using alchemy for the
        // execution endpoint and our custom beacon endpoint that rely on
        // its own execution chain. Alchemy was a bit delayed and the
        // execution height for the beacon head wasn't existing for few
        // secs. We wait for an extra beacon head to let alchemy catch up.
        // We should be able to remove that once we rely on an execution
        // endpoint that is itself used by the beacon endpoint (no different
        // POV).
        loop {
            let next_height = self
                .beacon_api_client
                .block(beacon_api::client::BlockId::Head)
                .await?
                .data
                .message
                .slot;
            if next_height > height {
                break;
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        }

        Ok(self.make_height(height))
    }

    async fn query_latest_timestamp(&self) -> Result<i64, Self::Error> {
        Ok(self
            .beacon_api_client
            .finality_update()
            .await?
            .data
            .attested_header
            .execution
            .timestamp
            .try_into()
            .unwrap())
    }

    async fn self_client_state(&self, beacon_height: Height) -> Self::SelfClientState {
        let genesis = self.beacon_api_client.genesis().await.unwrap().data;

        ethereum::client_state::ClientState {
            chain_id: self.chain_id,
            genesis_validators_root: genesis.genesis_validators_root,
            genesis_time: genesis.genesis_time,
            fork_parameters: self
                .beacon_api_client
                .spec()
                .await
                .unwrap()
                .data
                .into_fork_parameters(),
            // REVIEW: Is this a preset config param? Or a per-chain config?
            seconds_per_slot: C::SECONDS_PER_SLOT::U64,
            slots_per_epoch: C::SLOTS_PER_EPOCH::U64,
            epochs_per_sync_committee_period: C::EPOCHS_PER_SYNC_COMMITTEE_PERIOD::U64,
            // In nanoseconds
            trusting_period: 100_000_000 * 1_000_000_000,
            latest_slot: beacon_height.revision_height,
            min_sync_committee_participants: 0,
            trust_level: Fraction {
                numerator: 1,
                denominator: const { option_unwrap!(NonZeroU64::new(3)) },
            },
            frozen_height: Height {
                revision_number: 0,
                revision_height: 0,
            },
            ibc_commitment_slot: U256::from(0),
            ibc_contract_address: self.ibc_handler_address,
        }
    }

    async fn self_consensus_state(&self, beacon_height: Height) -> Self::SelfConsensusState {
        let trusted_header = self
            .beacon_api_client
            .header(beacon_api::client::BlockId::Slot(
                beacon_height.revision_height,
            ))
            .await
            .unwrap()
            .data;

        let bootstrap = self
            .beacon_api_client
            .bootstrap(trusted_header.root)
            .await
            .unwrap()
            .data;

        assert!(bootstrap.header.beacon.slot == beacon_height.revision_height);

        let light_client_update = {
            let current_period = beacon_height.revision_height.div(C::PERIOD::U64);

            tracing::info!(%current_period);

            let light_client_updates = self
                .beacon_api_client
                .light_client_updates(current_period, 1)
                .await
                .unwrap();

            let [light_client_update] = &*light_client_updates.0 else {
                panic!()
            };

            light_client_update.data.clone()
        };

        // Normalize to nanos in order to be compliant with cosmos
        let timestamp = bootstrap.header.execution.timestamp * 1_000_000_000;
        ethereum::consensus_state::ConsensusState {
            slot: bootstrap.header.beacon.slot,
            state_root: bootstrap.header.execution.state_root,
            // TODO: Should this shouldn't be the default but fetched via eth_getProof
            storage_root: H256::default(),
            timestamp,
            current_sync_committee: bootstrap.current_sync_committee.aggregate_pubkey,
            next_sync_committee: light_client_update
                .next_sync_committee
                .map(|nsc| nsc.aggregate_pubkey),
        }
    }

    async fn read_ack(
        &self,
        tx_hash: H256,
        destination_channel_id: ChannelId,
        destination_port_id: PortId,
        sequence: NonZeroU64,
    ) -> Vec<u8> {
        read_ack(
            self,
            tx_hash,
            destination_port_id,
            destination_channel_id,
            sequence,
        )
        .await
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EthereumInitError {
    #[error("unable to connect to websocket")]
    Ws(#[from] WsClientError),
    #[error("provider error")]
    Provider(#[from] ProviderError),
}

impl<C: ChainSpec, S: EthereumSignersConfig> Ethereum<C, S> {
    pub async fn new(config: Config<S>) -> Result<Self, EthereumInitError> {
        let provider = Provider::new(Ws::connect(config.eth_rpc_api).await?);

        let chain_id = provider.get_chainid().await?;

        Ok(Self {
            chain_id: U256(chain_id),
            ibc_handlers: S::new(
                config.signers,
                config.ibc_handler_address,
                chain_id.as_u64(),
                provider.clone(),
            ),
            ibc_handler_address: config.ibc_handler_address,
            provider: Arc::new(provider),
            beacon_api_client: BeaconApiClient::new(config.eth_beacon_rpc_api).await,
        })
    }
}

pub(crate) async fn read_ack<Hc: EthereumChainExt>(
    c: &Hc,
    tx_hash: H256,
    destination_port_id: PortId,
    destination_channel_id: ChannelId,
    sequence: NonZeroU64,
) -> Vec<u8> {
    c.provider()
        .get_transaction_receipt(tx_hash)
        .await
        .unwrap()
        .unwrap()
        .logs
        .into_iter()
        .map(|log| <WriteAcknowledgementFilter as EthLogDecode>::decode_log(&log.into()))
        .find_map(|e| match e {
            Ok(WriteAcknowledgementFilter {
                destination_port: ack_dst_port_id,
                destination_channel,
                sequence: ack_sequence,
                acknowledgement,
            }) if ack_dst_port_id == destination_port_id.to_string()
                && destination_channel == destination_channel_id.to_string()
                && Some(sequence) == NonZeroU64::new(ack_sequence) =>
            {
                Some(acknowledgement)
            }
            _ => None,
        })
        .unwrap_or_default()
        .to_vec()
}

impl<C: ChainSpec, S: EthereumSignersConfig> Ethereum<C, S> {
    pub fn make_height(&self, height: impl Into<u64>) -> Height {
        // REVIEW: Consider using the fork revision?
        {
            Height {
                revision_number: ETHEREUM_REVISION_NUMBER,
                revision_height: height.into(),
            }
        }
    }
}

pub trait IbcHandlerExt<M: Middleware> {
    fn ibc_state_read<P, Hc, Tr>(
        &self,
        execution_block_number: u64,
        path: P,
    ) -> impl Future<Output = Result<P::Value, ContractError<M>>> + Send
    where
        P: EthereumStateRead<Hc, Tr> + 'static,
        Hc: EthereumChain,
        Tr: Chain;

    fn eth_call<Call>(
        &self,
        call: Call,
        at_execution_height: u64,
    ) -> impl Future<
        Output = Result<
            Option<<<Call as EthCallExt>::Return as TupleToOption>::Inner>,
            ContractError<M>,
        >,
    > + Send
    where
        Call: EthCallExt + 'static,
        Call::Return: TupleToOption;
}

impl<M: Middleware> IbcHandlerExt<M> for IBCHandler<M> {
    async fn ibc_state_read<P, Hc, Tr>(
        &self,
        execution_block_number: u64,
        path: P,
    ) -> Result<P::Value, ContractError<M>>
    where
        P: EthereumStateRead<Hc, Tr> + 'static,
        Hc: EthereumChain,
        Tr: Chain,
    {
        self.method_hash::<P::EthCall, <P::EthCall as EthCallExt>::Return>(
            P::EthCall::selector(),
            path.into_eth_call(),
        )
        .expect("valid contract selector")
        .block(execution_block_number)
        .call()
        .await
        .map(P::decode_ibc_state)
    }

    async fn eth_call<Call>(
        &self,
        call: Call,
        at_execution_height: u64,
    ) -> Result<Option<<<Call as EthCallExt>::Return as TupleToOption>::Inner>, ContractError<M>>
    where
        Call: EthCallExt + 'static,
        Call::Return: TupleToOption,
    {
        self.method_hash::<Call, Call::Return>(Call::selector(), call)
            .expect("valid contract selector")
            .block(at_execution_height)
            .call()
            .await
            .map(Call::Return::tuple_to_option)
    }
}

pub type EthereumClientId = ClientId;

/// Many contract calls return some form of [`(bool, T)`] as a way to emulate nullable/[`Option`].
/// This trait allows for easy conversion from the aforementioned tuple to an [`Option`].
pub trait TupleToOption {
    type Inner;

    fn tuple_to_option(self) -> Option<Self::Inner>;
}

impl TupleToOption for GetClientStateReturn {
    type Inner = Vec<u8>;

    fn tuple_to_option(self) -> Option<Self::Inner> {
        self.1.then_some(self.0.to_vec())
    }
}

impl TupleToOption for GetConsensusStateReturn {
    type Inner = Vec<u8>;

    fn tuple_to_option(self) -> Option<Self::Inner> {
        self.p1.then_some(self.consensus_state_bytes.to_vec())
    }
}

impl TupleToOption for GetConnectionReturn {
    type Inner = IbcCoreConnectionV1ConnectionEndData;

    fn tuple_to_option(self) -> Option<Self::Inner> {
        self.1.then_some(self.0)
    }
}

impl TupleToOption for GetChannelReturn {
    type Inner = IbcCoreChannelV1ChannelData;

    fn tuple_to_option(self) -> Option<Self::Inner> {
        self.1.then_some(self.0)
    }
}

impl TupleToOption for GetHashedPacketCommitmentReturn {
    type Inner = [u8; 32];

    fn tuple_to_option(self) -> Option<Self::Inner> {
        self.1.then_some(self.0)
    }
}

impl TupleToOption for GetHashedPacketAcknowledgementCommitmentReturn {
    type Inner = [u8; 32];

    fn tuple_to_option(self) -> Option<Self::Inner> {
        self.1.then_some(self.0)
    }
}

/// Wrapper trait for a contract call's signature, to map the input type to the return type.
/// `ethers` generates both of these types, but doesn't correlate them.
pub trait EthCallExt: EthCall {
    type Return: Tokenizable + Send + Sync + 'static;
}

macro_rules! impl_eth_call_ext {
    ($($Call:ident -> $Return:ident;)+) => {
        $(
            impl EthCallExt for $Call {
                type Return = $Return;
            }
        )+
    }
}

impl_eth_call_ext! {
    GetClientStateCall                           -> GetClientStateReturn;
    GetConsensusStateCall                        -> GetConsensusStateReturn;
    GetConnectionCall                            -> GetConnectionReturn;
    GetChannelCall                               -> GetChannelReturn;
    GetHashedPacketCommitmentCall                -> GetHashedPacketCommitmentReturn;
    GetHashedPacketAcknowledgementCommitmentCall -> GetHashedPacketAcknowledgementCommitmentReturn;
    HasPacketReceiptCall                         -> bool;
    NextConnectionSequenceCall                   -> u64;
    NextClientSequenceCall                       -> u64;
}

pub fn next_epoch_timestamp<C: ChainSpec>(slot: u64, genesis_timestamp: u64) -> u64 {
    let next_epoch_slot = slot + (C::SLOTS_PER_EPOCH::U64 - (slot % C::SLOTS_PER_EPOCH::U64));
    genesis_timestamp + (next_epoch_slot * C::SECONDS_PER_SLOT::U64)
}

pub trait EthereumStateRead<Hc, Tr>: IbcPath<Hc, Tr>
where
    Hc: EthereumChain,
    Tr: Chain,
{
    type EthCall: EthCallExt + 'static;

    fn into_eth_call(self) -> Self::EthCall;

    fn decode_ibc_state(encoded: <Self::EthCall as EthCallExt>::Return) -> Self::Value;
}

impl<Hc, Tr> EthereumStateRead<Hc, Tr> for ClientStatePath<ClientIdOf<Hc>>
where
    Hc: EthereumChain,
    Tr: Chain,
    Self::Value: Decode<Hc::IbcStateEncoding>,
{
    type EthCall = GetClientStateCall;

    fn into_eth_call(self) -> Self::EthCall {
        Self::EthCall {
            client_id: self.client_id.to_string(),
        }
    }

    fn decode_ibc_state(encoded: <Self::EthCall as EthCallExt>::Return) -> Self::Value {
        <Self::Value as Decode<EthAbi>>::decode(&encoded.0).unwrap()
    }
}

impl<Hc, Tr> EthereumStateRead<Hc, Tr> for ClientConsensusStatePath<ClientIdOf<Hc>, HeightOf<Tr>>
where
    Hc: EthereumChain,
    Tr: Chain,
    Self::Value: Decode<EthAbi>,
{
    type EthCall = GetConsensusStateCall;

    fn into_eth_call(self) -> Self::EthCall {
        Self::EthCall {
            client_id: self.client_id.to_string(),
            height: self.height.into_height().into(),
        }
    }

    fn decode_ibc_state(encoded: <Self::EthCall as EthCallExt>::Return) -> Self::Value {
        <Self::Value as Decode<EthAbi>>::decode(&encoded.consensus_state_bytes).unwrap()
    }
}

impl<Hc, Tr> EthereumStateRead<Hc, Tr> for ConnectionPath
where
    Hc: EthereumChain,
    Tr: Chain,
{
    type EthCall = GetConnectionCall;

    fn into_eth_call(self) -> Self::EthCall {
        Self::EthCall {
            connection_id: self.connection_id.to_string(),
        }
    }

    fn decode_ibc_state(encoded: <Self::EthCall as EthCallExt>::Return) -> Self::Value {
        encoded.0.try_into().unwrap()
    }
}

impl<Hc, Tr> EthereumStateRead<Hc, Tr> for ChannelEndPath
where
    Hc: EthereumChain,
    Tr: Chain,
{
    type EthCall = GetChannelCall;

    fn into_eth_call(self) -> Self::EthCall {
        Self::EthCall {
            port_id: self.port_id.to_string(),
            channel_id: self.channel_id.to_string(),
        }
    }

    fn decode_ibc_state(encoded: <Self::EthCall as EthCallExt>::Return) -> Self::Value {
        encoded.0.try_into().unwrap()
    }
}

impl<Hc, Tr> EthereumStateRead<Hc, Tr> for CommitmentPath
where
    Hc: EthereumChain,
    Tr: Chain,
{
    type EthCall = GetHashedPacketCommitmentCall;

    fn into_eth_call(self) -> Self::EthCall {
        Self::EthCall {
            port_id: self.port_id.to_string(),
            channel_id: self.channel_id.to_string(),
            sequence: self.sequence.get(),
        }
    }

    fn decode_ibc_state(encoded: <Self::EthCall as EthCallExt>::Return) -> Self::Value {
        encoded.0.into()
    }
}

impl<Hc, Tr> EthereumStateRead<Hc, Tr> for AcknowledgementPath
where
    Hc: EthereumChain,
    Tr: Chain,
{
    type EthCall = GetHashedPacketAcknowledgementCommitmentCall;

    fn into_eth_call(self) -> Self::EthCall {
        Self::EthCall {
            port_id: self.port_id.to_string(),
            channel_id: self.channel_id.to_string(),
            sequence: self.sequence.get(),
        }
    }

    fn decode_ibc_state(encoded: <Self::EthCall as EthCallExt>::Return) -> Self::Value {
        encoded.0.into()
    }
}

impl<Hc, Tr> EthereumStateRead<Hc, Tr> for NextConnectionSequencePath
where
    Hc: EthereumChain,
    Tr: Chain,
{
    type EthCall = NextConnectionSequenceCall;

    fn into_eth_call(self) -> Self::EthCall {
        Self::EthCall {}
    }

    fn decode_ibc_state(encoded: <Self::EthCall as EthCallExt>::Return) -> Self::Value {
        encoded
    }
}

impl<Hc, Tr> EthereumStateRead<Hc, Tr> for NextClientSequencePath
where
    Hc: EthereumChain,
    Tr: Chain,
{
    type EthCall = NextClientSequenceCall;

    fn into_eth_call(self) -> Self::EthCall {
        Self::EthCall {}
    }

    fn decode_ibc_state(encoded: <Self::EthCall as EthCallExt>::Return) -> Self::Value {
        encoded
    }
}

impl<Hc, Tr> EthereumStateRead<Hc, Tr> for ReceiptPath
where
    Hc: EthereumChain,
    Tr: Chain,
{
    type EthCall = HasPacketReceiptCall;

    fn into_eth_call(self) -> Self::EthCall {
        Self::EthCall {
            port_id: self.port_id.to_string(),
            channel_id: self.channel_id.to_string(),
            sequence: self.sequence.into(),
        }
    }

    fn decode_ibc_state(encoded: <Self::EthCall as EthCallExt>::Return) -> Self::Value {
        encoded
    }
}

#[test]
fn eth_chain_type() {
    assert_eq!(
        <<Ethereum<unionlabs::ethereum::config::Mainnet> as Chain>::ChainType as FromStrExact>::EXPECTING,
        "eth-mainnet",
    );
    assert_eq!(
        <<Ethereum<unionlabs::ethereum::config::Minimal> as Chain>::ChainType as FromStrExact>::EXPECTING,
        "eth-minimal",
    );
}
