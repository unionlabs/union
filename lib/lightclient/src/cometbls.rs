use std::future::Future;

use chain_utils::evm::{EthCallExt, EthereumStateRead, Evm, TupleToOption};
use serde::{Deserialize, Serialize};
use unionlabs::{
    ethereum::Address,
    ethereum_consts_traits::{ChainSpec, Mainnet, Minimal},
    google::protobuf::any::Any,
    ibc::{
        core::{client::height::Height, connection::connection_end::ConnectionEnd},
        lightclients::{cometbls, wasm},
    },
    id::ClientId,
    proof::{ChannelEndPath, ConnectionPath, IbcPath},
    traits::{Chain, ChainOf, ClientStateOf, HeightOf, LightClientBase},
    TryFromProto,
};

use crate::ethereum::{EthereumMainnet, EthereumMinimal};

/// The solidity light client, tracking the state of the 08-wasm light client on union.
pub struct CometblsMinimal {
    chain: Evm<Minimal>,
}

/// The solidity light client, tracking the state of the 08-wasm light client on union.
pub struct CometblsMainnet {
    chain: Evm<Mainnet>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CometblsConfig {
    pub client_type: String,
    pub cometbls_client_address: Address,
}

impl LightClientBase for CometblsMainnet {
    type HostChain = Evm<Mainnet>;
    type Counterparty = EthereumMainnet;

    type ClientId = ClientId;
    type ClientType = String;

    type Config = CometblsConfig;

    fn chain(&self) -> &Self::HostChain {
        &self.chain
    }

    fn from_chain(chain: Self::HostChain) -> Self {
        Self { chain }
    }

    fn channel(
        &self,
        channel_id: unionlabs::id::ChannelId,
        port_id: unionlabs::id::PortId,
        at: HeightOf<Self::HostChain>,
    ) -> impl Future<Output = unionlabs::ibc::core::channel::channel::Channel> + '_ {
        read_ibc_state::<ChainOf<Self::Counterparty>, _, _>(
            &self.chain,
            ChannelEndPath {
                port_id,
                channel_id,
            },
            at,
        )
    }

    fn connection(
        &self,
        connection_id: unionlabs::id::ConnectionId,
        at: HeightOf<Self::HostChain>,
    ) -> impl Future<
        Output = ConnectionEnd<
            Self::ClientId,
            <Self::Counterparty as LightClientBase>::ClientId,
            String,
        >,
    > + '_ {
        read_ibc_state::<ChainOf<Self::Counterparty>, _, _>(
            &self.chain,
            ConnectionPath { connection_id },
            at,
        )
    }

    fn query_client_state(
        &self,
        client_id: <Self::HostChain as Chain>::ClientId,
        height: HeightOf<Self::HostChain>,
    ) -> impl Future<Output = ClientStateOf<<Self::Counterparty as LightClientBase>::HostChain>> + '_
    {
        query_client_state(&self.chain, client_id, height)
    }
}

impl LightClientBase for CometblsMinimal {
    type HostChain = Evm<Minimal>;
    type Counterparty = EthereumMinimal;

    type ClientId = ClientId;
    type ClientType = String;

    type Config = CometblsConfig;

    fn chain(&self) -> &Self::HostChain {
        &self.chain
    }

    fn from_chain(chain: Self::HostChain) -> Self {
        Self { chain }
    }

    fn channel(
        &self,
        channel_id: unionlabs::id::ChannelId,
        port_id: unionlabs::id::PortId,
        at: HeightOf<Self::HostChain>,
    ) -> impl Future<Output = unionlabs::ibc::core::channel::channel::Channel> + '_ {
        read_ibc_state::<ChainOf<Self::Counterparty>, _, _>(
            &self.chain,
            ChannelEndPath {
                port_id,
                channel_id,
            },
            at,
        )
    }

    fn connection(
        &self,
        connection_id: unionlabs::id::ConnectionId,
        at: HeightOf<Self::HostChain>,
    ) -> impl Future<
        Output = ConnectionEnd<
            Self::ClientId,
            <Self::Counterparty as LightClientBase>::ClientId,
            String,
        >,
    > + '_ {
        read_ibc_state::<ChainOf<Self::Counterparty>, _, _>(
            &self.chain,
            ConnectionPath { connection_id },
            at,
        )
    }

    fn query_client_state(
        &self,
        client_id: <Self::HostChain as Chain>::ClientId,
        height: HeightOf<Self::HostChain>,
    ) -> impl Future<Output = ClientStateOf<<Self::Counterparty as LightClientBase>::HostChain>> + '_
    {
        query_client_state(&self.chain, client_id, height)
    }
}

async fn query_client_state<C: ChainSpec>(
    evm: &Evm<C>,
    client_id: chain_utils::evm::EvmClientId,
    height: Height,
) -> Any<wasm::client_state::ClientState<cometbls::client_state::ClientState>> {
    let execution_height = evm.execution_height(height).await;

    let (client_state_bytes, is_found) = evm
        .readonly_ibc_handler
        .get_client_state(client_id.to_string())
        .block(execution_height)
        .await
        .unwrap();

    assert!(is_found);

    Any::try_from_proto_bytes(&client_state_bytes).unwrap()
}

async fn read_ibc_state<Counterparty, C, P>(
evm: &Evm<C>,
p: P,
at: HeightOf<Evm<C>>,
) -> P::Output
where
Counterparty: Chain,
C: ChainSpec,
P: IbcPath<Evm<C>, Counterparty>
    + EthereumStateRead<
        C,
        Counterparty,
        Encoded = <<<P as EthereumStateRead<C, Counterparty>>::EthCall as EthCallExt>::Return as TupleToOption>::Inner,
    > + 'static,
<P::EthCall as EthCallExt>::Return: TupleToOption,
{
    let execution_block_number = evm.execution_height(at).await;

    evm.read_ibc_state(p.into_eth_call(), execution_block_number)
        .await
        .unwrap()
        .map(|x| P::decode_ibc_state(x))
        .unwrap()
}
