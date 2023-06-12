use bip32::XPrv;
use ethers::types::H256;
use futures::{Future, FutureExt};
use k256::{ecdsa::Signature, schnorr::signature::Signer};
use prost::Message;
use protos::{
    cosmos::{ics23::v1 as ics23_v1, staking, tx},
    google,
    ibc::{
        core::{
            channel::v1 as channel_v1, client::v1 as client_v1, commitment::v1 as commitment_v1,
            connection::v1 as connection_v1,
        },
        lightclients::wasm::v1 as wasm_v1,
    },
    union::ibc::lightclients::{cometbls::v1 as cometbls_v1, ethereum::v1 as ethereum_v1},
};
use strum::ParseError;
use tendermint_rpc::{Client, WebSocketClient};
use tokio::task::JoinHandle;

use crate::{
    account_info_of_signer,
    chain::{
        evm::Cometbls,
        msgs::{
            channel::{
                Channel, Counterparty as ChannelCounterparty, MsgChannelOpenAck,
                MsgChannelOpenConfirm, MsgChannelOpenInit, MsgChannelOpenTry, MsgRecvPacket,
                Packet, State as ChannelState,
            },
            cometbls,
            connection::{
                Counterparty as ConnectionCounterparty, MsgConnectionOpenAck,
                MsgConnectionOpenConfirm, MsgConnectionOpenInit, MsgConnectionOpenTry,
                State as ConnectionState, Version,
            },
            ethereum, wasm, ConnectionEnd, Duration, Fraction, Height, MerklePrefix, MerkleRoot,
            StateProof, Timestamp, UnknownEnumVariant,
        },
        Connect, LightClient,
    },
    cosmos_to_eth::CHAIN_ID,
    eth_to_cosmos::{broadcast_tx_commit, signer_from_pk, signer_from_sk},
};

use super::msgs::ethereum::{
    AccountUpdate, BeaconBlockHeader, ExecutionPayloadHeader, LightClientHeader, LightClientUpdate,
    Proof, SyncAggregate, SyncCommittee, TrustedSyncCommittee,
};

/// The 08-wasm light client running on the union chain.
pub struct Ethereum {
    signer: XPrv,
    tm_client: WebSocketClient,
    driver_handle: JoinHandle<Result<(), tendermint_rpc::Error>>,
    wasm_code_id: H256,
}

impl Ethereum {
    pub async fn new(signer: XPrv, wasm_code_id: H256) -> Self {
        let (tm_client, driver) =
            WebSocketClient::builder("ws://127.0.0.1:26657/websocket".parse().unwrap())
                .compat_mode(tendermint_rpc::client::CompatMode::V0_37)
                .build()
                .await
                .unwrap();

        let driver_handle = tokio::spawn(async move { driver.run().await });

        Self {
            signer,
            tm_client,
            driver_handle,
            wasm_code_id,
        }
    }

    pub async fn broadcast_tx_commit(
        &self,
        messages: impl IntoIterator<Item = google::protobuf::Any>,
    ) -> tendermint_rpc::endpoint::broadcast::tx_commit::Response {
        let account = account_info_of_signer(&self.signer).await;

        let sign_doc = tx::v1beta1::SignDoc {
            body_bytes: tx::v1beta1::TxBody {
                messages: messages.into_iter().collect(),
                // TODO(benluelo): What do we want to use as our memo?
                memo: "".into(),
                timeout_height: 123_123_123,
                extension_options: vec![],
                non_critical_extension_options: vec![],
            }
            .encode_to_vec(),
            auth_info_bytes: tx::v1beta1::AuthInfo {
                signer_infos: [tx::v1beta1::SignerInfo {
                    public_key: Some(google::protobuf::Any {
                        type_url: "/cosmos.crypto.secp256k1.PubKey".to_string(),
                        value: self
                            .signer
                            .public_key()
                            .public_key()
                            .to_bytes()
                            .to_vec()
                            .encode_to_vec(),
                    }),
                    mode_info: Some(tx::v1beta1::ModeInfo {
                        sum: Some(tx::v1beta1::mode_info::Sum::Single(
                            tx::v1beta1::mode_info::Single {
                                mode: tx::signing::v1beta1::SignMode::Direct.into(),
                            },
                        )),
                    }),
                    sequence: account.sequence,
                }]
                .to_vec(),
                fee: Some(tx::v1beta1::Fee {
                    amount: vec![protos::cosmos::base::v1beta1::Coin {
                        denom: "stake".to_string(),
                        amount: "1".to_string(),
                    }],
                    gas_limit: 5_000_000,
                    payer: "".to_string(),
                    granter: "".to_string(),
                }),
                tip: None,
            }
            .encode_to_vec(),
            // TODO(benluelo): Pass this in somehow
            chain_id: "union-devnet-1".to_string(),
            account_number: account.account_number,
        };

        let alice_signature =
            Signer::<Signature>::try_sign(self.signer.private_key(), &sign_doc.encode_to_vec())
                .unwrap()
                .to_vec();

        let tx_raw = tx::v1beta1::TxRaw {
            body_bytes: sign_doc.body_bytes,
            auth_info_bytes: sign_doc.auth_info_bytes,
            signatures: [alice_signature].to_vec(),
        };

        self.tm_client
            .broadcast_tx_commit(tx_raw.encode_to_vec())
            .await
            .unwrap()
    }
}

impl LightClient for Ethereum {
    type ClientState = Any<wasm::ClientState<ethereum::ClientState>>;
    type ConsensusState = Any<wasm::ConsensusState<ethereum::ConsensusState>>;
    type UpdateClientMessage = wasm::Header<ethereum::Header>;

    fn chain_id(&self) -> impl Future<Output = String> + '_ {
        async move {
            self.tm_client
                .status()
                .await
                .unwrap()
                .node_info
                .network
                .to_string()
        }
    }

    fn create_client(
        &self,
        client_state: Self::ClientState,
        consensus_state: Self::ConsensusState,
    ) -> impl futures::Future<Output = String> + '_ {
        async move {
            let msg = google::protobuf::Any {
                type_url: "/ibc.core.client.v1.MsgCreateClient".into(),
                value: client_v1::MsgCreateClient {
                    signer: signer_from_pk(
                        &self.signer.public_key().public_key().to_bytes().to_vec(),
                    ),
                    client_state: Some(client_state.into_proto()),
                    consensus_state: Some(consensus_state.into_proto()),
                }
                .encode_to_vec(),
            };

            broadcast_tx_commit([msg].to_vec())
                .await
                .deliver_tx
                .events
                .into_iter()
                .find(|event| event.kind == "create_client")
                .unwrap()
                .attributes
                .into_iter()
                .find(|attr| attr.key == "client_id")
                .unwrap()
                .value
        }
    }

    fn update_client(
        &self,
        client_id: String,
        msg: Self::UpdateClientMessage,
    ) -> impl futures::Future<Output = ()> + '_ {
        async move {
            self.broadcast_tx_commit([google::protobuf::Any {
                type_url: "/ibc.core.client.v1.MsgCreateClient".into(),
                value: client_v1::MsgUpdateClient {
                    client_id,
                    client_message: Some(Any(msg).into_proto()),
                    signer: signer_from_sk(&self.signer),
                }
                .encode_to_vec(),
            }])
            .await;
        }
    }

    fn consensus_state_proof(
        &self,
        client_id: String,
        counterparty_height: Height,
        self_height: Height,
    ) -> impl Future<Output = StateProof<Self::ConsensusState>> + '_ {
        async move { todo!() }
    }

    fn client_state_proof(
        &self,
        client_id: String,
        self_height: Height,
    ) -> impl Future<Output = StateProof<Self::ClientState>> + '_ {
        async move {
            let path = format!("clients/{client_id}/clientState");

            let query_result = self
                .tm_client
                .abci_query(
                    Some("store/ibc/key".to_string()),
                    path,
                    // TODO(benluelo): Pass height as parameter
                    Some(self_height.revision_number.try_into().unwrap()),
                    true,
                )
                .await
                .unwrap();

            StateProof {
                state: google::protobuf::Any::decode(&*query_result.value)
                    .unwrap()
                    .try_into()
                    .unwrap(),
                proof: commitment_v1::MerkleProof {
                    proofs: query_result
                        .proof
                        .unwrap()
                        .ops
                        .into_iter()
                        .map(|op| ics23_v1::CommitmentProof::decode(op.data.as_slice()).unwrap())
                        .collect::<Vec<_>>(),
                }
                .encode_to_vec(),
                proof_height: Height {
                    // TODO(benluelo): Figure out revision number
                    revision_number: 0,
                    revision_height: query_result.height.value(),
                },
            }
        }
    }

    fn connection_state_proof(
        &self,
        connection_id: String,
        self_height: Height,
    ) -> impl Future<Output = StateProof<ConnectionEnd>> + '_ {
        async move { todo!() }
    }

    fn query_latest_height(&self) -> impl Future<Output = Height> + '_ {
        async move {
            let height = self
                .tm_client
                .latest_commit()
                .await
                .unwrap()
                .signed_header
                .header
                .height
                .value();

            let chain_id = self.chain_id().await;

            Height {
                revision_number: chain_id.split('-').last().unwrap().parse().unwrap(),
                // revision_number: 0,
                revision_height: height,
            }
        }
    }
}

impl Connect<Cometbls> for Ethereum {
    // fn generate_counterparty_handshake_client_state(
    //     &self,
    //     counterparty_state: <Cometbls as LightClient>::ClientState,
    // ) -> impl Future<Output = Self::HandshakeClientState> + '_ {
    //     async move {
    //         super::msgs::wasm::ClientState {
    //             data: Any(super::msgs::cometbls::ClientState {
    //                 chain_id: todo!(),
    //                 trust_level: todo!(),
    //                 trusting_period: todo!(),
    //                 unbonding_period: todo!(),
    //                 max_clock_drift: todo!(),
    //                 frozen_height: todo!(),
    //             }),
    //             code_id: todo!(),
    //             latest_height: todo!(),
    //         }
    //     }
    // }

    fn connection_open_init(
        &self,
        msg: MsgConnectionOpenInit,
    ) -> impl futures::Future<Output = String> + '_ {
        self.broadcast_tx_commit([google::protobuf::Any {
            type_url: "/ibc.core.connection.v1.MsgConnectionOpenInit".to_string(),
            value: msg.into_proto_with_signer(&self.signer).encode_to_vec(),
        }])
        .map(|response| {
            response
                .deliver_tx
                .events
                .into_iter()
                .find(|event| event.kind == "connection_open_init")
                .unwrap()
                .attributes
                .into_iter()
                .find(|attr| attr.key == "connection_id")
                .unwrap()
                .value
        })
    }

    fn connection_open_try(
        &self,
        msg: MsgConnectionOpenTry<<Cometbls as LightClient>::ClientState>,
    ) -> impl futures::Future<Output = String> + '_ {
        self.broadcast_tx_commit([google::protobuf::Any {
            type_url: "/ibc.core.connection.v1.MsgConnectionOpenTry".to_string(),
            value: dbg!(msg)
                .into_proto_with_signer(&self.signer)
                .encode_to_vec(),
        }])
        .map(|response| {
            response
                .deliver_tx
                .events
                .into_iter()
                .find(|event| dbg!(event).kind == "connection_open_try")
                .unwrap()
                .attributes
                .into_iter()
                .find(|attr| attr.key == "connection_id")
                .unwrap()
                .value
        })
    }

    fn connection_open_ack(
        &self,
        msg: MsgConnectionOpenAck<<Cometbls as LightClient>::ClientState>,
    ) -> impl futures::Future<Output = ()> + '_ {
        async move {
            self.broadcast_tx_commit([google::protobuf::Any {
                type_url: "/ibc.core.connection.v1.MsgConnectionOpenAck".to_string(),
                value: msg.into_proto_with_signer(&self.signer).encode_to_vec(),
            }])
            .await;
        }
    }

    fn connection_open_confirm(
        &self,
        msg: MsgConnectionOpenConfirm,
    ) -> impl futures::Future<Output = ()> + '_ {
        async move {
            self.broadcast_tx_commit([google::protobuf::Any {
                type_url: "/ibc.core.connection.v1.MsgConnectionOpenConfirm".to_string(),
                value: msg.into_proto_with_signer(&self.signer).encode_to_vec(),
            }])
            .await;
        }
    }

    fn channel_open_init(
        &self,
        msg: MsgChannelOpenInit,
    ) -> impl futures::Future<Output = String> + '_ {
        async move {
            self.broadcast_tx_commit([google::protobuf::Any {
                type_url: "/ibc.channel.v1.MsgChannelOpenInit".to_string(),
                value: msg.into_proto_with_signer(&self.signer).encode_to_vec(),
            }])
            .await
            .deliver_tx
            .events
            .into_iter()
            .find(|event| event.kind == "channel_open_init")
            .unwrap()
            .attributes
            .into_iter()
            .find(|attr| attr.key == "channel_id")
            .unwrap()
            .value
        }
    }

    fn channel_open_try(&self, msg: MsgChannelOpenTry) -> impl futures::Future<Output = ()> + '_ {
        async move {
            self.broadcast_tx_commit([google::protobuf::Any {
                type_url: "/ibc.channel.v1.MsgChannelOpenTry".to_string(),
                value: msg.into_proto_with_signer(&self.signer).encode_to_vec(),
            }])
            .await;
        }
    }

    fn channel_open_ack(&self, msg: MsgChannelOpenAck) -> impl futures::Future<Output = ()> + '_ {
        async move {
            self.broadcast_tx_commit([google::protobuf::Any {
                type_url: "/ibc.channel.v1.MsgchannelOpenAck".to_string(),
                value: msg.into_proto_with_signer(&self.signer).encode_to_vec(),
            }])
            .await;
        }
    }

    fn channel_open_confirm(
        &self,
        msg: MsgChannelOpenConfirm,
    ) -> impl futures::Future<Output = ()> + '_ {
        async move {
            self.broadcast_tx_commit([google::protobuf::Any {
                type_url: "/ibc.channel.v1.MsgChannelOpenConfirm".to_string(),
                value: msg.into_proto_with_signer(&self.signer).encode_to_vec(),
            }])
            .await;
        }
    }

    fn recv_packet(&self, msg: MsgRecvPacket) -> impl futures::Future<Output = ()> + '_ {
        async move { todo!() }
    }

    fn generate_counterparty_client_state(
        &self,
        height: Height,
    ) -> impl Future<Output = <Cometbls as LightClient>::ClientState> + '_ {
        async move {
            let params =
                staking::v1beta1::query_client::QueryClient::connect("http://0.0.0.0:9090")
                    .await
                    .unwrap()
                    .params(staking::v1beta1::QueryParamsRequest {})
                    .await
                    .unwrap()
                    .into_inner()
                    .params
                    .unwrap();

            let commit = self
                .tm_client
                .commit(tendermint::block::Height::try_from(height.revision_height).unwrap())
                .await
                .unwrap();

            let height = commit.signed_header.header.height;

            let unbonding_period = std::time::Duration::new(
                params
                    .unbonding_time
                    .clone()
                    .unwrap()
                    .seconds
                    .try_into()
                    .unwrap(),
                params
                    .unbonding_time
                    .clone()
                    .unwrap()
                    .nanos
                    .try_into()
                    .unwrap(),
            );

            Any(wasm::ClientState {
                data: cometbls::ClientState {
                    // TODO(benluelo): Pass this in somehow
                    chain_id: CHAIN_ID.into(),
                    // https://github.com/cometbft/cometbft/blob/da0e55604b075bac9e1d5866cb2e62eaae386dd9/light/verifier.go#L16
                    trust_level: Fraction {
                        numerator: 1,
                        denominator: 3,
                    },
                    // https://github.com/cosmos/relayer/blob/23d1e5c864b35d133cad6a0ef06970a2b1e1b03f/relayer/chains/cosmos/provider.go#L177
                    trusting_period: Duration {
                        seconds: (unbonding_period * 85 / 100).as_secs().try_into().unwrap(),
                        nanos: (unbonding_period * 85 / 100)
                            .subsec_nanos()
                            .try_into()
                            .unwrap(),
                    },
                    unbonding_period: Duration {
                        seconds: unbonding_period.as_secs().try_into().unwrap(),
                        nanos: unbonding_period.subsec_nanos().try_into().unwrap(),
                    },
                    // https://github.com/cosmos/relayer/blob/23d1e5c864b35d133cad6a0ef06970a2b1e1b03f/relayer/chains/cosmos/provider.go#L177
                    max_clock_drift: Duration {
                        seconds: 60 * 10,
                        nanos: 0,
                    },
                    frozen_height: Height {
                        revision_number: 0,
                        revision_height: 0,
                    },
                },
                code_id: self.wasm_code_id.to_fixed_bytes().to_vec(),
                latest_height: Height {
                    revision_number: self
                        .chain_id()
                        .await
                        .split('-')
                        .last()
                        .unwrap()
                        .parse()
                        .unwrap(),
                    revision_height: height.value(),
                },
            })
        }
    }

    fn generate_counterparty_consensus_state(
        &self,
        height: Height,
    ) -> impl Future<Output = <Cometbls as LightClient>::ConsensusState> + '_ {
        async move {
            let commit = self
                .tm_client
                .commit(tendermint::block::Height::try_from(height.revision_height).unwrap())
                .await
                .unwrap();

            Any(wasm::ConsensusState {
                data: cometbls::ConsensusState {
                    root: MerkleRoot {
                        hash: commit.signed_header.header.app_hash.as_bytes().to_vec(),
                    },
                    next_validators_hash: commit
                        .signed_header
                        .header
                        .next_validators_hash
                        .as_bytes()
                        .to_vec(),
                },
                timestamp: commit
                    .signed_header
                    .header
                    .time
                    .unix_timestamp()
                    .try_into()
                    .unwrap(),
            })
        }
    }

    fn generate_counterparty_update_client_message(
        &self,
        trusted_height: Height,
    ) -> impl Future<Output = <Cometbls as LightClient>::UpdateClientMessage> + '_ {
        async move { todo!() }
    }
}

/// Wrapper type to indicate that a type is to be serialized as an Any.
#[derive(Debug, Clone)]
pub struct Any<T>(pub T);

impl<T> From<Any<T>> for google::protobuf::Any
where
    T: IntoProto,
    <T as IntoProto>::Proto: TypeUrl,
{
    fn from(val: Any<T>) -> Self {
        google::protobuf::Any {
            type_url: <T as IntoProto>::Proto::TYPE_URL.to_string(),
            value: val.0.into_proto().encode_to_vec(),
        }
    }
}

impl<T> IntoProto for Any<T>
where
    T: IntoProto,
    <T as IntoProto>::Proto: TypeUrl,
{
    type Proto = google::protobuf::Any;
}

#[derive(Debug)]
pub enum TryFromAnyError<T: TryFromProto>
where
    <T as TryFrom<T::Proto>>::Error: std::fmt::Debug,
{
    IncorrectTypeUrl {
        found: String,
        expected: &'static str,
    },
    Prost(prost::DecodeError),
    TryFromProto(<T as TryFrom<T::Proto>>::Error),
}

impl<T> TryFrom<google::protobuf::Any> for Any<T>
where
    T: TryFromProto,
    T::Proto: TypeUrl + Default,
    <T as TryFrom<T::Proto>>::Error: std::fmt::Debug,
{
    type Error = TryFromAnyError<T>;

    fn try_from(value: google::protobuf::Any) -> Result<Self, Self::Error> {
        if value.type_url != T::Proto::TYPE_URL {
            Err(TryFromAnyError::IncorrectTypeUrl {
                found: value.type_url,
                expected: T::Proto::TYPE_URL,
            })
        } else {
            T::Proto::decode(&*value.value)
                .map_err(TryFromAnyError::Prost)?
                .try_into()
                .map_err(TryFromAnyError::TryFromProto)
                .map(Any)
        }
    }
}

// these traits allow for generic impls over T: Into<Proto>, however a type can only impl IntoProto
// for one type, so types such as `Fraction` that are defined in multiple places may cause issues

pub trait IntoProto: Into<Self::Proto> {
    type Proto: prost::Message;

    fn into_proto(self) -> Self::Proto {
        self.into()
    }
}

macro_rules! into_proto {
    ($(
        [$type_url:literal]
        $from:ty => $to:ty;
    )+) => {
        $(
            impl IntoProto for $from {
                type Proto = $to;
            }

            impl TypeUrl for $to {
                const TYPE_URL: &'static str = $type_url;
            }
        )+
    };
}

into_proto! {
    ["/ibc.core.connection.v1.ConnectionEnd"]
    ConnectionEnd => connection_v1::ConnectionEnd;

    ["/union.ibc.lightclients.cometbls.v1.ConsensusState"]
    cometbls::ConsensusState => cometbls_v1::ConsensusState;
    ["/union.ibc.lightclients.cometbls.v1.ClientState"]
    cometbls::ClientState => cometbls_v1::ClientState;

    ["/union.ibc.lightclients.ethereum.v1.ClientState"]
    ethereum::ClientState => ethereum_v1::ClientState;
    ["/union.ibc.lightclients.ethereum.v1.ConsensusState"]
    ethereum::ConsensusState => ethereum_v1::ConsensusState;
    ["/union.ibc.lightclients.ethereum.v1.Header"]
    ethereum::Header => ethereum_v1::Header;
}

// impl<T> IntoProto for T
// where
//     T: prost::Message,
// {
//     type Proto = Self;
// }

pub trait FromProto: From<Self::Proto> {
    type Proto: prost::Message;

    fn from_proto(proto: Self::Proto) -> Self {
        proto.into()
    }
}

pub trait TryFromProto: TryFrom<Self::Proto> {
    type Proto: prost::Message;

    fn try_from_proto(proto: Self::Proto) -> Result<Self, <Self as TryFrom<Self::Proto>>::Error> {
        proto.try_into()
    }
}

impl<T> TryFromProto for T
where
    T: FromProto,
{
    type Proto = T::Proto;
}

// impl IntoProto for T where T: T

trait TypeUrl: prost::Message {
    const TYPE_URL: &'static str;
}

trait MsgIntoProto {
    type Proto;

    fn into_proto_with_signer(self, signer: &XPrv) -> Self::Proto;
}

impl TypeUrl for wasm_v1::ClientState {
    const TYPE_URL: &'static str = "/ibc.lightclients.wasm.v1.ClientState";
}

impl<Data: IntoProto> From<wasm::ClientState<Data>> for wasm_v1::ClientState {
    fn from(val: wasm::ClientState<Data>) -> Self {
        wasm_v1::ClientState {
            data: val.data.into_proto().encode_to_vec(),
            code_id: val.code_id,
            latest_height: Some(val.latest_height.into()),
        }
    }
}

impl<Data: IntoProto> From<wasm::ConsensusState<Data>> for wasm_v1::ConsensusState {
    fn from(value: wasm::ConsensusState<Data>) -> Self {
        wasm_v1::ConsensusState {
            data: value.data.into_proto().encode_to_vec(),
            timestamp: value.timestamp,
        }
    }
}

impl<Data: IntoProto> IntoProto for wasm::ClientState<Data> {
    type Proto = wasm_v1::ClientState;
}

impl<Data: IntoProto> IntoProto for wasm::Header<Data> {
    type Proto = wasm_v1::Header;
}

impl<Data: IntoProto> From<wasm::Header<Data>> for wasm_v1::Header {
    fn from(value: wasm::Header<Data>) -> Self {
        Self {
            data: value.data.into_proto().encode_to_vec(),
            height: Some(value.height.into()),
        }
    }
}

impl TypeUrl for wasm_v1::ConsensusState {
    const TYPE_URL: &'static str = "/ibc.lightclients.wasm.v1.ConsensusState";
}

impl TypeUrl for wasm_v1::Header {
    const TYPE_URL: &'static str = "/ibc.lightclients.wasm.v1.Header";
}

impl<Data: IntoProto> IntoProto for wasm::ConsensusState<Data> {
    type Proto = wasm_v1::ConsensusState;
}

impl MsgIntoProto for MsgConnectionOpenInit {
    type Proto = connection_v1::MsgConnectionOpenInit;

    fn into_proto_with_signer(self, signer: &XPrv) -> Self::Proto {
        connection_v1::MsgConnectionOpenInit {
            client_id: self.client_id,
            counterparty: Some(self.counterparty.into()),
            version: Some(self.version.into()),
            delay_period: self.delay_period,
            signer: signer_from_sk(signer),
        }
    }
}

impl<ClientState> MsgIntoProto for MsgConnectionOpenTry<ClientState>
where
    ClientState: IntoProto<Proto = google::protobuf::Any>,
    // <ClientState as IntoProto>::Proto: ,
{
    type Proto = connection_v1::MsgConnectionOpenTry;

    fn into_proto_with_signer(self, signer: &XPrv) -> Self::Proto {
        #[allow(deprecated)]
        connection_v1::MsgConnectionOpenTry {
            client_id: self.client_id,
            previous_connection_id: "".to_string(),
            client_state: Some(self.client_state.into_proto()),
            counterparty: Some(self.counterparty.into()),
            delay_period: self.delay_period,
            counterparty_versions: self
                .counterparty_versions
                .into_iter()
                .map(Into::into)
                .collect(),
            proof_height: Some(self.proof_height.into()),
            proof_init: self.proof_init,
            proof_client: self.proof_client,
            proof_consensus: self.proof_consensus,
            consensus_height: Some(self.consensus_height.into()),
            signer: signer_from_sk(signer),
            host_consensus_state_proof: vec![],
        }
    }
}

impl<ClientState> MsgIntoProto for MsgConnectionOpenAck<ClientState>
where
    ClientState: IntoProto<Proto = google::protobuf::Any>,
    // <ClientState as IntoProto>::Proto: TypeUrl,
{
    type Proto = connection_v1::MsgConnectionOpenAck;

    fn into_proto_with_signer(self, signer: &XPrv) -> Self::Proto {
        connection_v1::MsgConnectionOpenAck {
            connection_id: self.connection_id,
            counterparty_connection_id: self.counterparty_connection_id,
            version: Some(self.version.into()),
            client_state: Some(self.client_state.into_proto()),
            proof_height: Some(self.proof_height.into()),
            proof_try: self.proof_try,
            proof_client: self.proof_client,
            proof_consensus: self.proof_consensus,
            consensus_height: Some(self.consensus_height.into()),
            signer: signer_from_sk(signer),
            host_consensus_state_proof: vec![],
        }
    }
}

impl MsgIntoProto for MsgConnectionOpenConfirm {
    type Proto = connection_v1::MsgConnectionOpenConfirm;

    fn into_proto_with_signer(self, signer: &XPrv) -> Self::Proto {
        connection_v1::MsgConnectionOpenConfirm {
            connection_id: self.connection_id,
            proof_ack: self.proof_ack,
            proof_height: Some(self.proof_height.into()),
            signer: signer_from_sk(signer),
        }
    }
}

impl MsgIntoProto for MsgChannelOpenInit {
    type Proto = channel_v1::MsgChannelOpenInit;

    fn into_proto_with_signer(self, signer: &XPrv) -> Self::Proto {
        channel_v1::MsgChannelOpenInit {
            port_id: self.port_id,
            channel: Some(self.channel.into()),
            signer: signer_from_sk(signer),
        }
    }
}

impl MsgIntoProto for MsgChannelOpenTry {
    type Proto = channel_v1::MsgChannelOpenTry;

    fn into_proto_with_signer(self, signer: &XPrv) -> Self::Proto {
        #[allow(deprecated)]
        channel_v1::MsgChannelOpenTry {
            port_id: self.port_id,
            channel: Some(self.channel.into()),
            counterparty_version: self.counterparty_version,
            proof_init: self.proof_init,
            proof_height: Some(self.proof_height.into()),
            previous_channel_id: "".to_string(),
            signer: signer_from_sk(signer),
        }
    }
}

impl MsgIntoProto for MsgChannelOpenAck {
    type Proto = channel_v1::MsgChannelOpenAck;

    fn into_proto_with_signer(self, signer: &XPrv) -> Self::Proto {
        channel_v1::MsgChannelOpenAck {
            port_id: self.port_id,
            channel_id: self.channel_id,
            counterparty_version: self.counterparty_version,
            counterparty_channel_id: self.counterparty_channel_id,
            proof_try: self.proof_try,
            proof_height: Some(self.proof_height.into()),
            signer: signer_from_sk(signer),
        }
    }
}

impl MsgIntoProto for MsgChannelOpenConfirm {
    type Proto = channel_v1::MsgChannelOpenConfirm;

    fn into_proto_with_signer(self, signer: &XPrv) -> Self::Proto {
        channel_v1::MsgChannelOpenConfirm {
            port_id: self.port_id,
            channel_id: self.channel_id,
            proof_ack: self.proof_ack,
            proof_height: Some(self.proof_height.into()),
            signer: signer_from_sk(signer),
        }
    }
}

impl From<client_v1::Height> for Height {
    fn from(proto: client_v1::Height) -> Self {
        Self {
            revision_number: proto.revision_number,
            revision_height: proto.revision_height,
        }
    }
}

impl From<Height> for client_v1::Height {
    fn from(value: Height) -> client_v1::Height {
        client_v1::Height {
            revision_number: value.revision_number,
            revision_height: value.revision_height,
        }
    }
}

/// A protobuf field was none unexpectedly.
#[derive(Debug)]
pub struct MissingField(&'static str);

impl From<ConnectionCounterparty> for connection_v1::Counterparty {
    fn from(value: ConnectionCounterparty) -> Self {
        Self {
            client_id: value.client_id,
            connection_id: value.connection_id,
            prefix: Some(value.prefix.into()),
        }
    }
}

impl TryFrom<connection_v1::Counterparty> for ConnectionCounterparty {
    type Error = MissingField;

    fn try_from(value: connection_v1::Counterparty) -> Result<Self, Self::Error> {
        Ok(Self {
            client_id: value.client_id,
            connection_id: value.connection_id,
            prefix: value.prefix.ok_or(MissingField("prefix"))?.into(),
        })
    }
}

impl TryFrom<connection_v1::Version> for Version {
    type Error = strum::ParseError;

    fn try_from(proto: connection_v1::Version) -> Result<Self, Self::Error> {
        Ok(Self {
            identifier: proto.identifier,
            features: proto
                .features
                .into_iter()
                .map(|feature| feature.parse())
                .collect::<Result<_, _>>()?,
        })
    }
}

impl From<Version> for connection_v1::Version {
    fn from(value: Version) -> Self {
        Self {
            identifier: value.identifier,
            features: value
                .features
                .into_iter()
                .map(|feature| <&'static str>::from(feature).to_string())
                .collect(),
        }
    }
}

impl From<commitment_v1::MerklePrefix> for MerklePrefix {
    fn from(proto: commitment_v1::MerklePrefix) -> Self {
        Self {
            key_prefix: proto.key_prefix,
        }
    }
}

impl From<MerklePrefix> for commitment_v1::MerklePrefix {
    fn from(value: MerklePrefix) -> Self {
        Self {
            key_prefix: value.key_prefix,
        }
    }
}

impl MsgIntoProto for MsgRecvPacket {
    type Proto = channel_v1::MsgRecvPacket;

    fn into_proto_with_signer(self, signer: &XPrv) -> Self::Proto {
        channel_v1::MsgRecvPacket {
            packet: Some(self.packet.into()),
            proof_commitment: self.proof_commitment,
            proof_height: Some(self.proof_height.into()),
            signer: signer_from_sk(signer),
        }
    }
}

impl From<Packet> for channel_v1::Packet {
    fn from(value: Packet) -> Self {
        Self {
            sequence: value.sequence,
            source_port: value.source_port,
            source_channel: value.source_channel,
            destination_port: value.destination_port,
            destination_channel: value.destination_channel,
            data: value.data,
            timeout_height: Some(value.timeout_height.into()),
            timeout_timestamp: value.timeout_timestamp,
        }
    }
}

impl TryFrom<channel_v1::Packet> for Packet {
    type Error = MissingField;

    fn try_from(proto: channel_v1::Packet) -> Result<Self, Self::Error> {
        Ok(Packet {
            sequence: proto.sequence,
            source_port: proto.source_port,
            source_channel: proto.source_channel,
            destination_port: proto.destination_port,
            destination_channel: proto.destination_channel,
            data: proto.data,
            timeout_height: proto
                .timeout_height
                .ok_or(MissingField("timeout_height"))?
                .into(),
            timeout_timestamp: proto.timeout_timestamp,
        })
    }
}

impl From<Channel> for channel_v1::Channel {
    fn from(value: Channel) -> Self {
        Self {
            state: value.state as i32,
            ordering: value.ordering as i32,
            counterparty: Some(value.counterparty.into()),
            connection_hops: value.connection_hops,
            version: value.version,
        }
    }
}

impl TryFrom<channel_v1::Channel> for Channel {
    type Error = MissingField;

    fn try_from(proto: channel_v1::Channel) -> Result<Self, Self::Error> {
        Ok(Channel {
            // state: super::msgs::connection::State::from_i32(proto.state),
            // ordering: super::msgs::channel::Order::from_i32(proto.ordering),
            state: todo!(),
            ordering: todo!(),
            counterparty: proto
                .counterparty
                .ok_or(MissingField("counterparty"))?
                .into(),
            connection_hops: proto.connection_hops,
            version: proto.version,
        })
    }
}

impl From<ethereum::ClientState> for ethereum_v1::ClientState {
    fn from(value: ethereum::ClientState) -> Self {
        Self {
            genesis_validators_root: value.genesis_validators_root,
            min_sync_committee_participants: value.min_sync_committee_participants,
            genesis_time: value.genesis_time,
            fork_parameters: Some(value.fork_parameters.into()),
            seconds_per_slot: value.seconds_per_slot,
            slots_per_epoch: value.slots_per_epoch,
            epochs_per_sync_committee_period: value.epochs_per_sync_committee_period,
            trust_level: Some(value.trust_level.into()),
            trusting_period: value.trusting_period,
            latest_slot: value.latest_slot,
            frozen_height: Some(value.frozen_height.into()),
            counterparty_commitment_slot: value.counterparty_commitment_slot,
        }
    }
}

impl From<ethereum::ConsensusState> for ethereum_v1::ConsensusState {
    fn from(value: ethereum::ConsensusState) -> Self {
        Self {
            slot: value.slot,
            storage_root: value.storage_root,
            timestamp: value.timestamp,
            current_sync_committee: value.current_sync_committee,
            next_sync_committee: value.next_sync_committee,
        }
    }
}

impl TryFrom<ethereum_v1::ClientState> for ethereum::ClientState {
    type Error = MissingField;

    fn try_from(value: ethereum_v1::ClientState) -> Result<Self, Self::Error> {
        Ok(Self {
            genesis_validators_root: value.genesis_validators_root,
            min_sync_committee_participants: value.min_sync_committee_participants,
            genesis_time: value.genesis_time,
            fork_parameters: value
                .fork_parameters
                .ok_or(MissingField("fork_parameters"))?
                .try_into()?,
            seconds_per_slot: value.seconds_per_slot,
            slots_per_epoch: value.slots_per_epoch,
            epochs_per_sync_committee_period: value.epochs_per_sync_committee_period,
            trust_level: value.trust_level.ok_or(MissingField("trust_level"))?.into(),
            trusting_period: value.trusting_period,
            latest_slot: value.latest_slot,
            frozen_height: value
                .frozen_height
                .ok_or(MissingField("frozen_height"))?
                .into(),
            counterparty_commitment_slot: value.counterparty_commitment_slot,
        })
    }
}

impl TryFromProto for ethereum::ClientState {
    type Proto = ethereum_v1::ClientState;
}

impl From<Fraction> for ethereum_v1::Fraction {
    fn from(value: Fraction) -> Self {
        Self {
            numerator: value.numerator,
            denominator: value.denominator,
        }
    }
}

impl From<Fraction> for cometbls_v1::Fraction {
    fn from(value: Fraction) -> Self {
        Self {
            numerator: value.numerator,
            denominator: value.denominator,
        }
    }
}

// impl From<super::msgs::Fraction> for lightclients::tendermint_v1::Fraction {
//     fn from(value: super::msgs::Fraction) -> Self {
//         Self {
//             numerator: value.numerator,
//             denominator: value.denominator,
//         }
//     }
// }

impl From<cometbls_v1::Fraction> for Fraction {
    fn from(value: cometbls_v1::Fraction) -> Self {
        Self {
            numerator: value.numerator,
            denominator: value.denominator,
        }
    }
}

impl From<Duration> for google::protobuf::Duration {
    fn from(value: Duration) -> Self {
        Self {
            seconds: value.seconds,
            nanos: value.nanos,
        }
    }
}

impl From<google::protobuf::Duration> for Duration {
    fn from(value: google::protobuf::Duration) -> Self {
        Self {
            seconds: value.seconds,
            nanos: value.nanos,
        }
    }
}

impl From<ethereum_v1::Fraction> for Fraction {
    fn from(proto: ethereum_v1::Fraction) -> Self {
        Self {
            numerator: proto.numerator,
            denominator: proto.denominator,
        }
    }
}

impl From<ethereum::ForkParameters> for ethereum_v1::ForkParameters {
    fn from(value: ethereum::ForkParameters) -> Self {
        Self {
            genesis_fork_version: value.genesis_fork_version,
            genesis_slot: value.genesis_slot,
            altair: Some(value.altair.into()),
            bellatrix: Some(value.bellatrix.into()),
            capella: Some(value.capella.into()),
            eip4844: Some(value.eip4844.into()),
        }
    }
}

impl TryFrom<ethereum_v1::ForkParameters> for ethereum::ForkParameters {
    type Error = MissingField;

    fn try_from(proto: ethereum_v1::ForkParameters) -> Result<Self, Self::Error> {
        Ok(Self {
            genesis_fork_version: proto.genesis_fork_version,
            genesis_slot: proto.genesis_slot,
            altair: proto.altair.ok_or(MissingField("altair"))?.into(),
            bellatrix: proto.bellatrix.ok_or(MissingField("bellatrix"))?.into(),
            capella: proto.capella.ok_or(MissingField("capella"))?.into(),
            eip4844: proto.eip4844.ok_or(MissingField("eip4844"))?.into(),
        })
    }
}

impl From<ethereum::Fork> for ethereum_v1::Fork {
    fn from(value: ethereum::Fork) -> Self {
        Self {
            version: value.version,
            epoch: value.epoch,
        }
    }
}

impl From<ethereum_v1::Fork> for ethereum::Fork {
    fn from(proto: ethereum_v1::Fork) -> Self {
        Self {
            version: proto.version,
            epoch: proto.epoch,
        }
    }
}

impl From<ChannelCounterparty> for channel_v1::Counterparty {
    fn from(value: ChannelCounterparty) -> Self {
        Self {
            port_id: value.port_id,
            channel_id: value.channel_id,
        }
    }
}

impl From<channel_v1::Counterparty> for ChannelCounterparty {
    fn from(proto: channel_v1::Counterparty) -> Self {
        Self {
            port_id: proto.port_id,
            channel_id: proto.channel_id,
        }
    }
}

#[derive(Debug)]
pub enum TryFromWasmClientStateError<Err> {
    TryFromProto(Err),
    Prost(prost::DecodeError),
}

impl<Data> TryFrom<wasm_v1::ClientState> for wasm::ClientState<Data>
where
    Data: TryFromProto,
    <Data as TryFromProto>::Proto: prost::Message + Default,
{
    type Error =
        TryFromWasmClientStateError<<Data as TryFrom<<Data as TryFromProto>::Proto>>::Error>;

    fn try_from(value: wasm_v1::ClientState) -> Result<Self, Self::Error> {
        Ok(Self {
            data: Data::try_from_proto(
                <Data as TryFromProto>::Proto::decode(&*value.data)
                    .map_err(TryFromWasmClientStateError::Prost)?,
            )
            .map_err(TryFromWasmClientStateError::TryFromProto)?,
            code_id: value.code_id,
            latest_height: value.latest_height.unwrap().into(),
        })
    }
}

#[derive(Debug)]
pub enum TryFromWasmConsensusStateError<Err> {
    TryFromProto(Err),
    Prost(prost::DecodeError),
}

impl<Data> TryFrom<wasm_v1::ConsensusState> for wasm::ConsensusState<Data>
where
    Data: TryFromProto,
    <Data as TryFromProto>::Proto: prost::Message + Default,
{
    type Error =
        TryFromWasmConsensusStateError<<Data as TryFrom<<Data as TryFromProto>::Proto>>::Error>;

    fn try_from(value: wasm_v1::ConsensusState) -> Result<Self, Self::Error> {
        Ok(Self {
            data: Data::try_from_proto(
                <Data as TryFromProto>::Proto::decode(&*value.data)
                    .map_err(TryFromWasmConsensusStateError::Prost)?,
            )
            .map_err(TryFromWasmConsensusStateError::TryFromProto)?,
            timestamp: value.timestamp,
        })
    }
}

impl<Data> TryFromProto for wasm::ClientState<Data>
where
    Data: TryFromProto,
    <Data as TryFromProto>::Proto: prost::Message + Default,
{
    type Proto = wasm_v1::ClientState;
}

impl<Data> TryFromProto for wasm::ConsensusState<Data>
where
    Data: TryFromProto,
    <Data as TryFromProto>::Proto: prost::Message + Default,
{
    type Proto = wasm_v1::ConsensusState;
}

// impl TypeUrl for lightclients::tendermint_v1::ClientState {
//     const TYPE_URL: &'static str = "/ibc.lightclients.tendermint.v1.ClientState";
// }

// impl From<super::msgs::tendermint::ClientState> for lightclients::tendermint_v1::ClientState {
//     fn from(val: super::msgs::tendermint::ClientState) -> Self {
//         #[allow(deprecated)]
//         lightclients::tendermint_v1::ClientState {
//             latest_height: Some(val.latest_height.into()),
//             chain_id: val.chain_id,
//             trust_level: Some(val.trust_level.into()),
//             trusting_period: Some(val.trusting_period.into()),
//             unbonding_period: Some(val.unbonding_period.into()),
//             max_clock_drift: Some(val.max_clock_drift.into()),
//             frozen_height: Some(val.frozen_height.into()),
//             proof_specs: val.proof_specs.into_iter().map(Into::into).collect(),
//             upgrade_path: val.upgrade_path,
//             allow_update_after_expiry: true,
//             allow_update_after_misbehaviour: true,
//         }
//     }
// }

// impl IntoProto for super::msgs::tendermint::ClientState {
//     type Proto = lightclients::tendermint_v1::ClientState;
// }

// impl From<super::msgs::ics23::ProofSpec> for ics23_v1::ProofSpec {
//     fn from(value: super::msgs::ics23::ProofSpec) -> Self {
//         Self {
//             leaf_spec: Some(value.leaf_spec.into()),
//             inner_spec: Some(value.inner_spec.into()),
//             max_depth: value.max_depth,
//             min_depth: value.min_depth,
//         }
//     }
// }

// impl From<super::msgs::ics23::InnerSpec> for ics23_v1::InnerSpec {
//     fn from(value: super::msgs::ics23::InnerSpec) -> Self {
//         Self {
//             child_order: value.child_order,
//             child_size: value.child_size,
//             min_prefix_length: value.min_prefix_length,
//             max_prefix_length: value.max_prefix_length,
//             empty_child: value.empty_child,
//             // TODO(benluelo): Better conversion here, go into the proto generated enum and then cast
//             hash: value.hash as i32,
//         }
//     }
// }

// impl From<super::msgs::ics23::LeafOp> for ics23_v1::LeafOp {
//     fn from(value: super::msgs::ics23::LeafOp) -> Self {
//         Self {
//             hash: value.hash as i32,
//             prehash_key: value.prehash_key as i32,
//             prehash_value: value.prehash_value as i32,
//             length: value.length as i32,
//             prefix: value.prefix,
//         }
//     }
// }

pub enum TryFromConnnectionEndError {
    ParseError(ParseError),
    UnknownEnumVariant(UnknownEnumVariant<i32>),
    MissingField(MissingField),
}

impl TryFrom<connection_v1::ConnectionEnd> for ConnectionEnd {
    type Error = TryFromConnnectionEndError;

    fn try_from(val: connection_v1::ConnectionEnd) -> Result<Self, Self::Error> {
        Ok(Self {
            client_id: val.client_id,
            versions: val
                .versions
                .into_iter()
                .map(|x| x.try_into().map_err(TryFromConnnectionEndError::ParseError))
                .collect::<Result<_, _>>()?,
            state: val
                .state
                .try_into()
                .map_err(TryFromConnnectionEndError::UnknownEnumVariant)?,
            counterparty: val
                .counterparty
                .ok_or(TryFromConnnectionEndError::MissingField(MissingField(
                    "counterparty",
                )))?
                .try_into()
                .map_err(TryFromConnnectionEndError::MissingField)?,
            delay_period: val.delay_period,
        })
    }
}

impl From<ConnectionEnd> for connection_v1::ConnectionEnd {
    fn from(val: ConnectionEnd) -> Self {
        Self {
            client_id: val.client_id,
            versions: val.versions.into_iter().map(|x| x.into()).collect(),
            state: val.state as i32,
            counterparty: Some(val.counterparty.into()),
            delay_period: val.delay_period,
        }
    }
}

impl From<ConnectionState> for connection_v1::State {
    fn from(value: ConnectionState) -> Self {
        match value {
            ConnectionState::UninitializedUnspecified => {
                connection_v1::State::UninitializedUnspecified
            }
            ConnectionState::Init => connection_v1::State::Init,
            ConnectionState::Tryopen => connection_v1::State::Tryopen,
            ConnectionState::Open => connection_v1::State::Open,
        }
    }
}

impl From<connection_v1::State> for ConnectionState {
    fn from(value: connection_v1::State) -> Self {
        match value {
            connection_v1::State::UninitializedUnspecified => {
                ConnectionState::UninitializedUnspecified
            }
            connection_v1::State::Init => ConnectionState::Init,
            connection_v1::State::Tryopen => ConnectionState::Tryopen,
            connection_v1::State::Open => ConnectionState::Open,
        }
    }
}

impl From<cometbls::ClientState> for cometbls_v1::ClientState {
    fn from(value: cometbls::ClientState) -> Self {
        Self {
            chain_id: value.chain_id,
            trust_level: Some(value.trust_level.into()),
            trusting_period: Some(value.trusting_period.into()),
            unbonding_period: Some(value.unbonding_period.into()),
            max_clock_drift: Some(value.max_clock_drift.into()),
            frozen_height: Some(value.frozen_height.into()),
        }
    }
}

impl TryFromProto for cometbls::ClientState {
    type Proto = cometbls_v1::ClientState;
}

impl TryFrom<cometbls_v1::ClientState> for cometbls::ClientState {
    type Error = MissingField;

    fn try_from(value: cometbls_v1::ClientState) -> Result<Self, Self::Error> {
        Ok(Self {
            chain_id: value.chain_id,
            trust_level: value.trust_level.ok_or(MissingField("trust_level"))?.into(),
            trusting_period: value
                .trusting_period
                .ok_or(MissingField("trusting_period"))?
                .into(),
            unbonding_period: value
                .unbonding_period
                .ok_or(MissingField("unbonding_period"))?
                .into(),
            max_clock_drift: value
                .max_clock_drift
                .ok_or(MissingField("max_clock_drift"))?
                .into(),
            frozen_height: value
                .frozen_height
                .ok_or(MissingField("frozen_height"))?
                .into(),
        })
    }
}

impl TryFrom<cometbls_v1::ConsensusState> for cometbls::ConsensusState {
    type Error = MissingField;

    fn try_from(value: cometbls_v1::ConsensusState) -> Result<Self, Self::Error> {
        Ok(Self {
            root: value.root.ok_or(MissingField("root"))?.into(),
            next_validators_hash: value.next_validators_hash,
        })
    }
}

impl TryFromProto for cometbls::ConsensusState {
    type Proto = cometbls_v1::ConsensusState;
}

impl From<cometbls::ConsensusState> for cometbls_v1::ConsensusState {
    fn from(value: cometbls::ConsensusState) -> Self {
        Self {
            root: Some(value.root.into()),
            next_validators_hash: value.next_validators_hash,
        }
    }
}

impl From<MerkleRoot> for commitment_v1::MerkleRoot {
    fn from(value: MerkleRoot) -> Self {
        Self { hash: value.hash }
    }
}

impl From<commitment_v1::MerkleRoot> for MerkleRoot {
    fn from(value: commitment_v1::MerkleRoot) -> Self {
        Self { hash: value.hash }
    }
}

impl From<Timestamp> for google::protobuf::Timestamp {
    fn from(value: Timestamp) -> Self {
        Self {
            seconds: value.seconds,
            nanos: value.nanos,
        }
    }
}

impl From<ethereum::Header> for ethereum_v1::Header {
    fn from(value: ethereum::Header) -> Self {
        Self {
            trusted_sync_committee: Some(value.trusted_sync_committee.into()),
            consensus_update: Some(value.consensus_update.into()),
            account_update: Some(value.account_update.into()),
            timestamp: value.timestamp,
        }
    }
}

impl From<AccountUpdate> for ethereum_v1::AccountUpdate {
    fn from(value: AccountUpdate) -> Self {
        Self {
            proofs: value.proofs.into_iter().map(|x| x.into()).collect(),
        }
    }
}

impl From<Proof> for ethereum_v1::Proof {
    fn from(value: Proof) -> Self {
        Self {
            key: value.key,
            value: value.value,
            proof: value.proof,
        }
    }
}

impl From<LightClientUpdate> for ethereum_v1::LightClientUpdate {
    fn from(value: LightClientUpdate) -> Self {
        Self {
            attested_header: Some(value.attested_header.into()),
            next_sync_committee: Some(value.next_sync_committee.into()),
            next_sync_committee_branch: value.next_sync_committee_branch,
            finalized_header: Some(value.finalized_header.into()),
            finality_branch: value.finality_branch,
            sync_aggregate: Some(value.sync_aggregate.into()),
            signature_slot: value.signature_slot,
        }
    }
}

impl From<LightClientHeader> for ethereum_v1::LightClientHeader {
    fn from(value: LightClientHeader) -> Self {
        Self {
            beacon: Some(value.beacon.into()),
            execution: Some(value.execution.into()),
            execution_branch: value.execution_branch,
        }
    }
}

impl From<BeaconBlockHeader> for ethereum_v1::BeaconBlockHeader {
    fn from(value: BeaconBlockHeader) -> Self {
        Self {
            slot: value.slot,
            proposer_index: value.proposer_index,
            parent_root: value.parent_root,
            state_root: value.state_root,
            body_root: value.body_root,
        }
    }
}

impl From<ExecutionPayloadHeader> for ethereum_v1::ExecutionPayloadHeader {
    fn from(value: ExecutionPayloadHeader) -> Self {
        Self {
            parent_hash: value.parent_hash,
            fee_recipient: value.fee_recipient,
            state_root: value.state_root,
            receipts_root: value.receipts_root,
            logs_bloom: value.logs_bloom,
            prev_randao: value.prev_randao,
            block_number: value.block_number,
            gas_limit: value.gas_limit,
            gas_used: value.gas_used,
            timestamp: value.timestamp,
            extra_data: value.extra_data,
            base_fee_per_gas: value.base_fee_per_gas,
            block_hash: value.block_hash,
            transactions_root: value.transactions_root,
            withdrawals_root: value.withdrawals_root,
        }
    }
}

impl From<TrustedSyncCommittee> for ethereum_v1::TrustedSyncCommittee {
    fn from(value: TrustedSyncCommittee) -> Self {
        Self {
            trusted_height: Some(value.trusted_height.into()),
            sync_committee: Some(value.sync_committee.into()),
            is_next: value.is_next,
        }
    }
}

impl From<SyncCommittee> for ethereum_v1::SyncCommittee {
    fn from(value: SyncCommittee) -> Self {
        Self {
            pubkeys: value.pubkeys,
            aggregate_pubkey: value.aggregate_pubkey,
        }
    }
}

impl From<SyncAggregate> for ethereum_v1::SyncAggregate {
    fn from(value: SyncAggregate) -> Self {
        Self {
            sync_committee_bits: value.sync_committee_bits,
            sync_committee_signature: value.sync_committee_signature,
        }
    }
}
