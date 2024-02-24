use futures::Future;
use unionlabs::{encoding::Proto, google::protobuf::any::Any, hash::H256, traits::Chain};

use crate::cosmos_sdk::CosmosSdkChain;

#[derive(Debug, Clone)]
pub struct Wasm<C: Chain>(pub C);

pub trait Wraps<T: CosmosSdkChain + Chain>: CosmosSdkChain + Chain {
    fn inner(&self) -> &T;
}

impl<T: CosmosSdkChain> CosmosSdkChain for Wasm<T> {
    fn grpc_url(&self) -> String {
        self.0.grpc_url()
    }

    fn fee_denom(&self) -> String {
        self.0.fee_denom()
    }

    fn tm_client(&self) -> &tendermint_rpc::WebSocketClient {
        self.0.tm_client()
    }

    fn signers(&self) -> &crate::Pool<unionlabs::signer::CosmosSigner> {
        self.0.signers()
    }

    fn checksum_cache(&self) -> &std::sync::Arc<dashmap::DashMap<H256, unionlabs::WasmClientType>> {
        self.0.checksum_cache()
    }
}

impl<T: CosmosSdkChain + Chain> Wraps<T> for T {
    fn inner(&self) -> &T {
        self
    }
}

impl<T: CosmosSdkChain + Chain> Wraps<T> for Wasm<T>
where
    Wasm<T>: Chain,
{
    fn inner(&self) -> &T {
        &self.0
    }
}

impl<Hc: CosmosSdkChain> Chain for Wasm<Hc> {
    type ChainType = Hc::ChainType;

    type SelfClientState = Hc::SelfClientState;
    type SelfConsensusState = Hc::SelfConsensusState;
    type Header = Hc::Header;

    type StoredClientState<Tr: Chain> =
        Any<unionlabs::ibc::lightclients::wasm::client_state::ClientState<Tr::SelfClientState>>;
    type StoredConsensusState<Tr: Chain> = Any<
        unionlabs::ibc::lightclients::wasm::consensus_state::ConsensusState<Tr::SelfConsensusState>,
    >;

    type Height = Hc::Height;

    type ClientId = Hc::ClientId;
    type ClientType = Hc::ClientType;

    type Error = Hc::Error;

    type IbcStateEncoding = Proto;

    type StateProof = Hc::StateProof;

    fn chain_id(&self) -> <Self::SelfClientState as unionlabs::traits::ClientState>::ChainId {
        self.0.chain_id()
    }

    fn query_latest_height(&self) -> impl Future<Output = Result<Self::Height, Self::Error>> + '_ {
        self.0.query_latest_height()
    }

    fn query_latest_height_as_destination(
        &self,
    ) -> impl Future<Output = Result<Self::Height, Self::Error>> + '_ {
        self.0.query_latest_height_as_destination()
    }

    fn query_latest_timestamp(&self) -> impl Future<Output = Result<i64, Self::Error>> + '_ {
        self.0.query_latest_timestamp()
    }

    fn self_client_state(
        &self,
        height: Self::Height,
    ) -> impl Future<Output = Self::SelfClientState> + '_ {
        self.0.self_client_state(height)
    }

    fn self_consensus_state(
        &self,
        height: Self::Height,
    ) -> impl Future<Output = Self::SelfConsensusState> + '_ {
        self.0.self_consensus_state(height)
    }

    fn read_ack(
        &self,
        tx_hash: unionlabs::hash::H256,
        destination_channel_id: unionlabs::id::ChannelId,
        destination_port_id: unionlabs::id::PortId,
        sequence: u64,
    ) -> impl Future<Output = Vec<u8>> + '_ {
        self.0.read_ack(
            tx_hash,
            destination_channel_id,
            destination_port_id,
            sequence,
        )
    }
}
