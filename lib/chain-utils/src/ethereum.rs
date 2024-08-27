use std::{fmt::Debug, marker::PhantomData, ops::Div, sync::Arc};

use beacon_api::client::BeaconApiClient;
use contracts::{
    cometbls_client::CometblsClientErrors,
    i_light_client::ILightClient,
    ibc_channel_handshake::{IBCChannelHandshakeErrors, IBCChannelHandshakeEvents},
    ibc_client::{IBCClientErrors, IBCClientEvents},
    ibc_connection::{IBCConnectionErrors, IBCConnectionEvents},
    ibc_handler::{IBCHandler, OwnershipTransferredFilter},
    ibc_packet::{IBCPacketErrors, IBCPacketEvents},
};
use ethers::{
    abi::AbiDecode,
    contract::{ContractError, EthLogDecode},
    core::k256::ecdsa,
    middleware::{NonceManagerMiddleware, SignerMiddleware},
    providers::{Middleware, Provider, Ws},
    signers::{LocalWallet, Wallet},
    utils::secret_key_to_address,
};
use frame_support_procedural::{CloneNoBound, DebugNoBound};
use futures::Future;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_utils::Hex;
use tracing::{debug, instrument};
use typenum::Unsigned;
use unionlabs::{
    encoding::EthAbi,
    ethereum::{
        config::{ChainSpec, Mainnet, Minimal},
        ibc_commitment_key, IBC_HANDLER_COMMITMENTS_SLOT,
    },
    hash::H160,
    ibc::{
        core::client::height::Height,
        lightclients::ethereum::{self, storage_proof::StorageProof},
    },
    ics24::{
        AcknowledgementPath, ChannelEndPath, ClientConsensusStatePath, ClientStatePath,
        CommitmentPath, ConnectionPath, IbcPath, NextClientSequencePath,
        NextConnectionSequencePath, NextSequenceAckPath, NextSequenceRecvPath,
        NextSequenceSendPath, ReceiptPath,
    },
    id::ClientId,
    iter,
    traits::{Chain, ClientState, FromStrExact},
    uint::U256,
};

use crate::keyring::{ChainKeyring, ConcurrentKeyring, KeyringConfig, KeyringEntry, SignerBalance};

pub type EthereumKeyring = ConcurrentKeyring<H160, IBCHandler<EthereumSignerMiddleware>>;

pub type EthereumSignerMiddleware =
    SignerMiddleware<NonceManagerMiddleware<Provider<Ws>>, Wallet<ecdsa::SigningKey>>;

pub trait EthereumChain =
    Chain<IbcStateEncoding = EthAbi, ClientType = String, StateProof = StorageProof>;

// NOTE: ClientType bound is temporary until I figure out a better way to deal with client types
/// A chain running the EVM and our solidity IBC stack. This can be any Ethereum L1 or L2, or a chain running the EVM in a different environment (such as Berachain).
pub trait EthereumIbcChain {
    /// The provider connected to this chain's [JSON-RPC](https://ethereum.org/en/developers/docs/apis/json-rpc/).
    fn provider(&self) -> Arc<Provider<Ws>>;

    /// The address of the [`IBCHandler`] smart contract deployed natively on this chain.
    fn ibc_handler_address(&self) -> H160;
}

/// An Ethereum-based chain. This can be any chain that is based off of and settles on Ethereum (i.e. Ethereum mainnet/ Sepolia, L2s such as Scroll).
pub trait EthereumConsensusChain: EthereumIbcChain {
    /// Fetch the execution height associated with the given beacon slot. For [`Ethereum`], this will simply be the execution block number, but for L2s this will fetch the settled height at the L1 block number.
    fn execution_height_of_beacon_slot(&self, slot: u64) -> impl Future<Output = u64>;

    // NOTE: This is a stopgap solution until we stop using ethers and write our own eth rpc library
    fn get_proof(
        &self,
        address: H160,
        location: U256,
        block: u64,
    ) -> impl Future<Output = StorageProof>;
}

#[diagnostic::on_unimplemented(message = "{Self} does not implement `EthereumChain`")]
pub trait EthereumIbcChainExt: EthereumIbcChain {
    /// Convenience method to construct an [`IBCHandler`] instance for this chain.
    fn ibc_handler(&self) -> IBCHandler<Provider<Ws>> {
        IBCHandler::new(self.ibc_handler_address(), self.provider())
    }
}

impl<T: EthereumIbcChain> EthereumIbcChainExt for T {}

impl<C: ChainSpec, S: EthereumSignersConfig> EthereumIbcChain for Ethereum<C, S> {
    fn provider(&self) -> Arc<Provider<Ws>> {
        self.provider.clone()
    }

    fn ibc_handler_address(&self) -> H160 {
        self.ibc_handler_address
    }
}

impl<C: ChainSpec, S: EthereumSignersConfig> EthereumConsensusChain for Ethereum<C, S> {
    async fn execution_height_of_beacon_slot(&self, slot: u64) -> u64 {
        let execution_height = self
            .beacon_api_client
            .execution_height(beacon_api::client::BlockId::Slot(slot))
            .await
            .unwrap();

        debug!("beacon slot {slot} is execution height {execution_height}");

        execution_height
    }

    async fn get_proof(&self, _address: H160, _location: U256, _block: u64) -> StorageProof {
        todo!()
        // get_proof(self, address, location, block).await
    }
}

// TODO(benluelo): Generic over middleware?
#[derive(DebugNoBound, CloneNoBound)]
pub struct Ethereum<C: ChainSpec, S: EthereumSignersConfig = ReadWrite> {
    pub chain_id: U256,

    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,
    pub multicall_address: H160,

    pub keyring: S::Out,
    pub provider: Arc<Provider<Ws>>,
    pub beacon_api_client: BeaconApiClient,
    pub max_gas_price: Option<U256>,
    __marker: PhantomData<fn() -> C>,
}

#[derive(DebugNoBound, CloneNoBound, enumorph::Enumorph)]
pub enum AnyEthereum<S: EthereumSignersConfig = ReadWrite> {
    Mainnet(Ethereum<Mainnet, S>),
    Minimal(Ethereum<Minimal, S>),
}

impl<S: EthereumSignersConfig> EthereumIbcChain for AnyEthereum<S> {
    fn provider(&self) -> Arc<Provider<Ws>> {
        match self {
            AnyEthereum::Mainnet(eth) => eth.provider(),
            AnyEthereum::Minimal(eth) => eth.provider(),
        }
    }

    fn ibc_handler_address(&self) -> H160 {
        match self {
            AnyEthereum::Mainnet(eth) => eth.ibc_handler_address(),
            AnyEthereum::Minimal(eth) => eth.ibc_handler_address(),
        }
    }
}

impl<S: EthereumSignersConfig> EthereumConsensusChain for AnyEthereum<S> {
    async fn execution_height_of_beacon_slot(&self, slot: u64) -> u64 {
        match self {
            AnyEthereum::Mainnet(eth) => eth.execution_height_of_beacon_slot(slot).await,
            AnyEthereum::Minimal(eth) => eth.execution_height_of_beacon_slot(slot).await,
        }
    }

    async fn get_proof(&self, address: H160, location: U256, block: u64) -> StorageProof {
        match self {
            AnyEthereum::Mainnet(eth) => eth.get_proof(address, location, block).await,
            AnyEthereum::Minimal(eth) => eth.get_proof(address, location, block).await,
        }
    }
}

impl<S: EthereumSignersConfig> AnyEthereum<S> {
    pub async fn new(_config: Config<S>) -> Result<Self, AnyEthereumError> {
        // match Ethereum::<Mainnet, S>::new(config.clone()).await {
        //     Ok(eth) => return Ok(eth.into()),
        //     Err(EthereumInitError::Beacon(beacon_api::client::NewError::IncorrectChainSpec)) => {}
        //     Err(err) => {
        //         return Err(AnyEthereumError::Mainnet(err));
        //     }
        // }

        // match Ethereum::<Minimal, S>::new(config).await {
        //     Ok(eth) => return Ok(eth.into()),
        //     Err(EthereumInitError::Beacon(beacon_api::client::NewError::IncorrectChainSpec)) => {}
        //     Err(err) => {
        //         return Err(AnyEthereumError::Minimal(err));
        //     }
        // }

        // Err(AnyEthereumError::UnknownChainSpec)

        todo!()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AnyEthereumError {
    // #[error("error creating minimal beacon client")]
    // Minimal(#[source] EthereumInitError),
    // #[error("error creating mainnet beacon client")]
    // Mainnet(#[source] EthereumInitError),
    // #[error("unknown chain spec")]
    // UnknownChainSpec,
}

#[derive(DebugNoBound, CloneNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct Config<S: EthereumSignersConfig = ReadWrite> {
    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,
    pub multicall_address: H160,

    /// The signer that will be used to submit transactions by voyager.
    // TODO: Write a custom ser/de implementation for this struct to avoid this hackery
    #[serde(skip_serializing_if = "is_unit", default)]
    pub keyring: S::Config,

    /// The RPC endpoint for the execution chain.
    pub eth_rpc_api: String,
    /// The RPC endpoint for the beacon chain.
    pub eth_beacon_rpc_api: String,

    #[serde(default)]
    pub max_gas_price: Option<U256>,
}

// lol
fn is_unit<T: 'static>(_: &T) -> bool {
    std::any::TypeId::of::<T>() == std::any::TypeId::of::<()>()
}

impl<C: ChainSpec> ChainKeyring for Ethereum<C, ReadWrite> {
    type Address = H160;

    type Signer = IBCHandler<EthereumSignerMiddleware>;

    fn keyring(&self) -> &ConcurrentKeyring<Self::Address, Self::Signer> {
        &self.keyring
    }

    async fn balances(&self) -> Vec<SignerBalance<Self::Address>> {
        balance_of_signers(&self.keyring, &self.provider).await
    }
}

pub async fn balance_of_signers(
    keyring: &EthereumKeyring,
    provider: &Provider<Ws>,
) -> Vec<SignerBalance<H160>> {
    let mut out_vec = vec![];

    for (key_name, &address) in keyring.keys() {
        out_vec.push(SignerBalance {
            key_name: key_name.to_owned(),
            address,
            balance: provider
                .get_balance(ethers::types::H160::from(address), None)
                .await
                .unwrap()
                .as_u128(),
            denom: "".to_owned(),
        });
    }

    out_vec
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
    type Config = KeyringConfig;
    type Out = EthereumKeyring;

    fn new(
        config: Self::Config,
        ibc_handler_address: H160,
        chain_id: u64,
        provider: Provider<Ws>,
    ) -> Self::Out {
        ConcurrentKeyring::new(
            config.name,
            config.keys.into_iter().map(|config| {
                let signing_key = <ecdsa::SigningKey as bip32::PrivateKey>::from_bytes(
                    &config.value().as_slice().try_into().unwrap(),
                )
                .unwrap();

                let address = secret_key_to_address(&signing_key);

                let wallet = LocalWallet::new_with_signer(signing_key, address, chain_id);

                let signer_middleware = Arc::new(SignerMiddleware::new(
                    NonceManagerMiddleware::new(provider.clone(), address),
                    wallet.clone(),
                ));

                KeyringEntry {
                    name: config.name(),
                    address: address.into(),
                    signer: IBCHandler::new(ibc_handler_address, signer_middleware.clone()),
                }
            }),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EthereumChainType<C: ChainSpec>(PhantomData<fn() -> C>);

pub const ETHEREUM_REVISION_NUMBER: u64 = 0;

impl<C: ChainSpec> FromStrExact for EthereumChainType<C> {
    const EXPECTING: &'static str = {
        match core::str::from_utf8(
            const {
                let mut buf = [0_u8; 32];

                iter! {
                    for (i, b) in enumerate(b"eth-") {
                        buf[i] = b;
                    }
                }

                iter! {
                    for (i, b) in enumerate(C::EXPECTING.as_bytes()) {
                        buf[4 + i] = b;
                    }
                }

                buf
            }
            .split_at(4 + C::EXPECTING.len())
            .0,
        ) {
            Ok(ok) => ok,
            Err(_) => panic!("called `Result::unwrap()` on an `Err` value"),
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

    type ClientId = ClientId;

    type IbcStateEncoding = EthAbi;

    type ClientType = String;

    type Error = beacon_api::errors::Error;

    type StateProof = StorageProof;

    fn chain_id(&self) -> <Self::SelfClientState as ClientState>::ChainId {
        self.chain_id
    }

    async fn query_latest_height(&self) -> Result<Height, Self::Error> {
        todo!()
    }

    async fn query_latest_height_as_destination(&self) -> Result<Height, Self::Error> {
        todo!()
    }

    async fn query_latest_timestamp(&self) -> Result<i64, Self::Error> {
        todo!()
    }

    async fn self_client_state(&self, beacon_height: Height) -> Self::SelfClientState {
        let genesis = self.beacon_api_client.genesis().await.unwrap().data;

        let spec = self.beacon_api_client.spec().await.unwrap().data;

        ethereum::client_state::ClientState {
            chain_id: self.chain_id,
            genesis_validators_root: genesis.genesis_validators_root,
            genesis_time: genesis.genesis_time,
            fork_parameters: spec.to_fork_parameters(),
            // REVIEW: Is this a preset config param? Or a per-chain config?
            seconds_per_slot: C::SECONDS_PER_SLOT::U64,
            slots_per_epoch: C::SLOTS_PER_EPOCH::U64,
            epochs_per_sync_committee_period: C::EPOCHS_PER_SYNC_COMMITTEE_PERIOD::U64,
            latest_slot: beacon_height.revision_height,
            min_sync_committee_participants: 0,
            frozen_height: Height {
                revision_number: 0,
                revision_height: 0,
            },
            ibc_commitment_slot: IBC_HANDLER_COMMITMENTS_SLOT,
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

            debug!(%current_period);

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
            storage_root: self
                .provider
                .get_proof(
                    ethers::types::H160::from(self.ibc_handler_address.0),
                    vec![],
                    Some(bootstrap.header.execution.block_number.into()),
                )
                .await
                .unwrap()
                .storage_hash
                .0
                .into(),
            timestamp,
            current_sync_committee: bootstrap.current_sync_committee.aggregate_pubkey,
            next_sync_committee: light_client_update
                .next_sync_committee
                .map(|nsc| nsc.aggregate_pubkey),
        }
    }
}

pub trait IbcHandlerExt<M: Middleware> {
    fn ibc_state_read<P>(
        &self,
        execution_block_number: u64,
        path: P,
    ) -> impl Future<Output = Result<P::Value, ContractError<M>>> + Send
    where
        P: EthereumStateRead + 'static;
}

impl<M: Middleware> IbcHandlerExt<M> for IBCHandler<M> {
    async fn ibc_state_read<P>(
        &self,
        execution_block_number: u64,
        path: P,
    ) -> Result<P::Value, ContractError<M>>
    where
        P: EthereumStateRead + 'static,
    {
        Ok(path.read(self, execution_block_number).await)
    }
}

pub trait EthereumStateRead: IbcPath {
    fn read<M: Middleware>(
        self,
        ibc_handler: &IBCHandler<M>,
        execution_block_number: u64,
    ) -> impl Future<Output = Self::Value> + Send + '_;
}

impl EthereumStateRead for ClientStatePath {
    async fn read<M: Middleware>(
        self,
        ibc_handler: &IBCHandler<M>,
        execution_block_number: u64,
    ) -> Self::Value {
        let client_address = ibc_handler
            .get_client(self.client_id.to_string())
            .block(execution_block_number)
            .call()
            .await
            .unwrap();

        Hex(ILightClient::new(client_address, ibc_handler.client())
            .get_client_state(self.client_id.to_string())
            .block(execution_block_number)
            .call()
            .await
            .unwrap()
            .to_vec())
    }
}

impl EthereumStateRead for ClientConsensusStatePath {
    async fn read<M: Middleware>(
        self,
        ibc_handler: &IBCHandler<M>,
        execution_block_number: u64,
    ) -> Self::Value {
        let client_address = ibc_handler
            .get_client(self.client_id.to_string())
            .block(execution_block_number)
            .call()
            .await
            .unwrap();

        Hex(ILightClient::new(client_address, ibc_handler.client())
            .get_consensus_state(self.client_id.to_string(), self.height.into())
            .block(execution_block_number)
            .call()
            .await
            .unwrap()
            .to_vec())
    }
}

impl EthereumStateRead for ConnectionPath {
    async fn read<M: Middleware>(
        self,
        ibc_handler: &IBCHandler<M>,
        execution_block_number: u64,
    ) -> Self::Value {
        ibc_handler
            .get_connection(self.connection_id.to_string())
            .block(execution_block_number)
            .call()
            .await
            .unwrap()
            .try_into()
            .unwrap()
    }
}

impl EthereumStateRead for ChannelEndPath {
    #[instrument(
        skip_all,
        fields(
            port_id = %self.port_id,
            channel_id = %self.channel_id,
            %execution_block_number,
            ibc_handler_address = %H160::from(ibc_handler.address()),
        )
    )]
    async fn read<M: Middleware>(
        self,
        ibc_handler: &IBCHandler<M>,
        execution_block_number: u64,
    ) -> Self::Value {
        debug!("fetching channel");

        let raw = ibc_handler
            .get_channel(self.port_id.to_string(), self.channel_id.to_string())
            .block(execution_block_number)
            .call()
            .await
            .unwrap();

        debug!(?raw);

        raw.try_into().unwrap()
    }
}

impl EthereumStateRead for CommitmentPath {
    async fn read<M: Middleware>(
        self,
        ibc_handler: &IBCHandler<M>,
        execution_block_number: u64,
    ) -> Self::Value {
        ibc_handler
            .client()
            .get_storage_at(
                ibc_handler.address(),
                ibc_commitment_key(&self.to_string(), IBC_HANDLER_COMMITMENTS_SLOT)
                    .to_be_bytes()
                    .into(),
                Some(execution_block_number.into()),
            )
            .await
            .unwrap()
            .into()
    }
}

impl EthereumStateRead for AcknowledgementPath {
    async fn read<M: Middleware>(
        self,
        ibc_handler: &IBCHandler<M>,
        execution_block_number: u64,
    ) -> Self::Value {
        ibc_handler
            .client()
            .get_storage_at(
                ibc_handler.address(),
                ibc_commitment_key(&self.to_string(), IBC_HANDLER_COMMITMENTS_SLOT)
                    .to_be_bytes()
                    .into(),
                Some(execution_block_number.into()),
            )
            .await
            .unwrap()
            .into()
    }
}

impl EthereumStateRead for NextConnectionSequencePath {
    async fn read<M: Middleware>(
        self,
        ibc_handler: &IBCHandler<M>,
        execution_block_number: u64,
    ) -> Self::Value {
        U256::from_be_bytes(
            ibc_handler
                .client()
                .get_storage_at(
                    ibc_handler.address(),
                    ibc_commitment_key(&self.to_string(), IBC_HANDLER_COMMITMENTS_SLOT)
                        .to_be_bytes()
                        .into(),
                    Some(execution_block_number.into()),
                )
                .await
                .unwrap()
                .0,
        )
        .try_into()
        .unwrap()
    }
}

impl EthereumStateRead for NextClientSequencePath {
    async fn read<M: Middleware>(
        self,
        ibc_handler: &IBCHandler<M>,
        execution_block_number: u64,
    ) -> Self::Value {
        U256::from_be_bytes(
            ibc_handler
                .client()
                .get_storage_at(
                    ibc_handler.address(),
                    ibc_commitment_key(&self.to_string(), IBC_HANDLER_COMMITMENTS_SLOT)
                        .to_be_bytes()
                        .into(),
                    Some(execution_block_number.into()),
                )
                .await
                .unwrap()
                .0,
        )
        .try_into()
        .unwrap()
    }
}

impl EthereumStateRead for ReceiptPath {
    async fn read<M: Middleware>(
        self,
        ibc_handler: &IBCHandler<M>,
        execution_block_number: u64,
    ) -> Self::Value {
        match u64::try_from(U256::from_be_bytes(
            ibc_handler
                .client()
                .get_storage_at(
                    ibc_handler.address(),
                    ibc_commitment_key(&self.to_string(), IBC_HANDLER_COMMITMENTS_SLOT)
                        .to_be_bytes()
                        .into(),
                    Some(execution_block_number.into()),
                )
                .await
                .unwrap()
                .0,
        ))
        .unwrap()
        {
            0 => false,
            1 => true,
            n => panic!("not a bool??? {n}"),
        }
    }
}

impl EthereumStateRead for NextSequenceRecvPath {
    async fn read<M: Middleware>(
        self,
        ibc_handler: &IBCHandler<M>,
        execution_block_number: u64,
    ) -> Self::Value {
        u64::try_from(U256::from_be_bytes(
            ibc_handler
                .client()
                .get_storage_at(
                    ibc_handler.address(),
                    ibc_commitment_key(&self.to_string(), IBC_HANDLER_COMMITMENTS_SLOT)
                        .to_be_bytes()
                        .into(),
                    Some(execution_block_number.into()),
                )
                .await
                .unwrap()
                .0,
        ))
        .unwrap()
    }
}

impl EthereumStateRead for NextSequenceSendPath {
    async fn read<M: Middleware>(
        self,
        ibc_handler: &IBCHandler<M>,
        execution_block_number: u64,
    ) -> Self::Value {
        u64::try_from(U256::from_be_bytes(
            ibc_handler
                .client()
                .get_storage_at(
                    ibc_handler.address(),
                    ibc_commitment_key(&self.to_string(), IBC_HANDLER_COMMITMENTS_SLOT)
                        .to_be_bytes()
                        .into(),
                    Some(execution_block_number.into()),
                )
                .await
                .unwrap()
                .0,
        ))
        .unwrap()
    }
}

impl EthereumStateRead for NextSequenceAckPath {
    async fn read<M: Middleware>(
        self,
        ibc_handler: &IBCHandler<M>,
        execution_block_number: u64,
    ) -> Self::Value {
        u64::try_from(U256::from_be_bytes(
            ibc_handler
                .client()
                .get_storage_at(
                    ibc_handler.address(),
                    ibc_commitment_key(&self.to_string(), IBC_HANDLER_COMMITMENTS_SLOT)
                        .to_be_bytes()
                        .into(),
                    Some(execution_block_number.into()),
                )
                .await
                .unwrap()
                .0,
        ))
        .unwrap()
    }
}
