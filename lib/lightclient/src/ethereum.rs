use core::future::Future;
use std::fmt::Debug;

use chain_utils::union::{AbciStateRead, Union};
use serde::{Deserialize, Serialize};
use unionlabs::{
    hash::H256,
    ibc::core::{client::height::Height, connection::connection_end::ConnectionEnd},
    proof::{ChannelEndPath, ClientStatePath, ConnectionPath, IbcPath},
    traits::{Chain, ChainOf, ClientStateOf, ConsensusStateOf, HeightOf, LightClientBase},
    Proto, TryFromProto, TryFromProtoErrorOf,
};

use crate::cometbls::{CometblsMainnet, CometblsMinimal};

/// The 08-wasm light client tracking ethereum, running on the union chain.
pub struct EthereumMinimal {
    chain: <Self as LightClientBase>::HostChain,
}

/// The 08-wasm light client tracking ethereum, running on the union chain.
pub struct EthereumMainnet {
    chain: <Self as LightClientBase>::HostChain,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EthereumConfig {
    pub checksum: H256,
}

impl LightClientBase for EthereumMinimal {
    type HostChain = Union;
    type Counterparty = CometblsMinimal;

    type Config = EthereumConfig;

    fn chain(&self) -> &Self::HostChain {
        &self.chain
    }

    fn from_chain(chain: Self::HostChain) -> Self {
        Self { chain }
    }

    fn query_client_state(
        &self,
        client_id: <Self::HostChain as Chain>::ClientId,
        height: HeightOf<Self::HostChain>,
    ) -> impl Future<Output = ClientStateOf<<Self::Counterparty as LightClientBase>::HostChain>> + '_
    {
        query_client_state::<Self>(&self.chain, client_id, height)
    }

    fn channel(
        &self,
        channel_id: unionlabs::id::ChannelId,
        port_id: unionlabs::id::PortId,
        at: HeightOf<Self::HostChain>,
    ) -> impl Future<Output = unionlabs::ibc::core::channel::channel::Channel> + '_ {
        read_ibc_state::<ChainOf<Self::Counterparty>, _>(
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
            <Self::HostChain as Chain>::ClientId,
            <<Self::Counterparty as LightClientBase>::HostChain as Chain>::ClientId,
            String,
        >,
    > + '_ {
        read_ibc_state::<ChainOf<Self::Counterparty>, _>(
            &self.chain,
            ConnectionPath { connection_id },
            at,
        )
    }
}

impl LightClientBase for EthereumMainnet {
    type HostChain = Union;
    type Counterparty = CometblsMainnet;

    type Config = EthereumConfig;

    fn chain(&self) -> &Self::HostChain {
        &self.chain
    }

    fn from_chain(chain: Self::HostChain) -> Self {
        Self { chain }
    }

    fn query_client_state(
        &self,
        client_id: <Self::HostChain as Chain>::ClientId,
        height: HeightOf<Self::HostChain>,
    ) -> impl Future<Output = ClientStateOf<<Self::Counterparty as LightClientBase>::HostChain>> + '_
    {
        query_client_state::<Self>(&self.chain, client_id, height)
    }

    fn channel(
        &self,
        channel_id: unionlabs::id::ChannelId,
        port_id: unionlabs::id::PortId,
        at: HeightOf<Self::HostChain>,
    ) -> impl Future<Output = unionlabs::ibc::core::channel::channel::Channel> + '_ {
        read_ibc_state::<ChainOf<Self::Counterparty>, _>(
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
            <Self::HostChain as Chain>::ClientId,
            <<Self::Counterparty as LightClientBase>::HostChain as Chain>::ClientId,
            String,
        >,
    > + '_ {
        read_ibc_state::<ChainOf<Self::Counterparty>, _>(
            &self.chain,
            ConnectionPath { connection_id },
            at,
        )
    }
}

async fn read_ibc_state<Counterparty, P>(union: &Union, path: P, at: HeightOf<Union>) -> P::Output
where
    Counterparty: Chain,
    ClientStateOf<Counterparty>: TryFromProto,
    ConsensusStateOf<Counterparty>: TryFromProto,
    P: IbcPath<Union, Counterparty> + AbciStateRead<Counterparty> + 'static,
{
    let mut client =
        protos::cosmos::base::tendermint::v1beta1::service_client::ServiceClient::connect(
            union.grpc_url.clone(),
        )
        .await
        .unwrap();

    let query_result = client
        .abci_query(
            protos::cosmos::base::tendermint::v1beta1::AbciQueryRequest {
                data: path.to_string().into_bytes(),
                path: "store/ibc/key".to_string(),
                height: i64::try_from(at.revision_height).unwrap(),
                prove: false,
            },
        )
        .await
        .unwrap()
        .into_inner();

    P::from_abci_bytes(query_result.value)
}

async fn query_client_state<L>(
    union: &Union,
    client_id: chain_utils::union::UnionClientId,
    height: Height,
) -> ClientStateOf<<L::Counterparty as LightClientBase>::HostChain>
where
    L: LightClientBase<HostChain = Union>,
    ClientStateOf<<L::Counterparty as LightClientBase>::HostChain>: Proto<Proto = protos::google::protobuf::Any>
        + TryFrom<protos::google::protobuf::Any>
        + TryFromProto<Proto = protos::google::protobuf::Any>,
    // NOTE: This bound can be removed once we don't unwrap anymore
    TryFromProtoErrorOf<ClientStateOf<<L::Counterparty as LightClientBase>::HostChain>>: Debug,
    <<L::Counterparty as LightClientBase>::HostChain as Chain>::SelfClientState: Proto<Proto = protos::google::protobuf::Any>
        + TryFrom<protos::google::protobuf::Any>
        + TryFromProto<Proto = protos::google::protobuf::Any>,
{
    let mut client =
        protos::cosmos::base::tendermint::v1beta1::service_client::ServiceClient::connect(
            union.grpc_url.clone(),
        )
        .await
        .unwrap();

    <ClientStateOf<<L::Counterparty as LightClientBase>::HostChain>>::try_from_proto_bytes(
        &client
            .abci_query(
                protos::cosmos::base::tendermint::v1beta1::AbciQueryRequest {
                    data: ClientStatePath { client_id }.to_string().into_bytes(),
                    path: "store/ibc/key".to_string(),
                    height: height.revision_height.try_into().unwrap(),
                    prove: false,
                },
            )
            .await
            .unwrap()
            .into_inner()
            .value,
    )
    .unwrap()
}
