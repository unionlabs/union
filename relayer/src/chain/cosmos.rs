use bip32::XPrv;
use futures::{Future, FutureExt};
use k256::{ecdsa::Signature, schnorr::signature::Signer};
use prost::Message;
use protos::{
    cosmos::{ics23, staking, tx},
    google,
    ibc::{
        core::{
            channel::{
                self,
                v1::{
                    MsgChannelOpenAck, MsgChannelOpenConfirm, MsgChannelOpenInit,
                    MsgChannelOpenTry, MsgRecvPacket,
                },
            },
            client, commitment,
            connection::{
                self,
                v1::{
                    MsgConnectionOpenAck, MsgConnectionOpenConfirm, MsgConnectionOpenInit,
                    MsgConnectionOpenTry,
                },
            },
        },
        lightclients::{self, wasm},
    },
    union::ibc::lightclients::{cometbls, ethereum},
};
use strum::ParseError;
use tendermint_rpc::{Client, WebSocketClient};
use tokio::task::JoinHandle;

use crate::{
    account_info_of_signer,
    cosmos_to_eth::CHAIN_ID,
    eth_to_cosmos::{broadcast_tx_commit, signer_from_pk, signer_from_sk},
};

use super::{evm::Cometbls, Connect, LightClient};

/// The 08-wasm light client running on the union chain.
pub struct Ethereum {
    signer: XPrv,
    tm_client: WebSocketClient,
    driver_handle: JoinHandle<Result<(), tendermint_rpc::Error>>,
}

impl Ethereum {
    pub async fn new(signer: XPrv) -> Self {
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
    type ClientState = super::msgs::wasm::ClientState<super::msgs::ethereum::ClientState>;
    type ConsensusState = super::msgs::wasm::ConsensusState<super::msgs::ethereum::ConsensusState>;
    type UpdateClientMessage = ();

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
                value: client::v1::MsgCreateClient {
                    signer: signer_from_pk(
                        &self.signer.public_key().public_key().to_bytes().to_vec(),
                    ),
                    client_state: Some(Any(client_state).into_proto()),
                    consensus_state: Some(Any(consensus_state).into_proto()),
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
        async move { todo!() }
    }

    fn consensus_state_proof(
        &self,
        client_id: String,
        counterparty_height: super::msgs::Height,
        self_height: super::msgs::Height,
    ) -> impl Future<Output = super::msgs::StateProof<Self::ConsensusState>> + '_ {
        async move { todo!() }
    }

    fn client_state_proof(
        &self,
        client_id: String,
        self_height: super::msgs::Height,
    ) -> impl Future<Output = super::msgs::StateProof<Self::ClientState>> + '_ {
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

            super::msgs::StateProof {
                state: wasm::v1::ClientState::decode(&*query_result.value)
                    .unwrap()
                    .try_into()
                    .unwrap(),
                proof: commitment::v1::MerkleProof {
                    proofs: query_result
                        .proof
                        .unwrap()
                        .ops
                        .into_iter()
                        .map(|op| ics23::v1::CommitmentProof::decode(op.data.as_slice()).unwrap())
                        .collect::<Vec<_>>(),
                }
                .encode_to_vec(),
                proof_height: super::msgs::Height {
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
        self_height: super::msgs::Height,
    ) -> impl Future<Output = super::msgs::StateProof<super::msgs::ConnectionEnd>> + '_ {
        async move { todo!() }
    }

    fn query_latest_height(&self) -> impl Future<Output = super::msgs::Height> + '_ {
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

            super::msgs::Height {
                revision_number: 0,
                revision_height: height,
            }
        }
    }
}

impl Connect<Cometbls> for Ethereum {
    type HandshakeClientState =
        super::msgs::wasm::ClientState<Any<<Cometbls as LightClient>::ClientState>>;

    fn generate_counterparty_handshake_client_state(
        &self,
        counterparty_state: <Cometbls as LightClient>::ClientState,
    ) -> impl Future<Output = Self::HandshakeClientState> + '_ {
        async move {
            super::msgs::wasm::ClientState {
                data: Any(super::msgs::cometbls::ClientState {
                    chain_id: todo!(),
                    trust_level: todo!(),
                    trusting_period: todo!(),
                    unbonding_period: todo!(),
                    max_clock_drift: todo!(),
                    frozen_height: todo!(),
                    latest_height: todo!(),
                }),
                code_id: todo!(),
                latest_height: todo!(),
            }
        }
    }

    fn connection_open_init(
        &self,
        msg: super::msgs::connection::MsgConnectionOpenInit,
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
        msg: super::msgs::connection::MsgConnectionOpenTry<Self::HandshakeClientState>,
    ) -> impl futures::Future<Output = String> + '_ {
        self.broadcast_tx_commit([google::protobuf::Any {
            type_url: "/ibc.core.connection.v1.MsgConnectionOpenTry".to_string(),
            value: msg.into_proto_with_signer(&self.signer).encode_to_vec(),
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
        msg: super::msgs::connection::MsgConnectionOpenAck<<Cometbls as LightClient>::ClientState>,
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
        msg: super::msgs::connection::MsgConnectionOpenConfirm,
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
        msg: super::msgs::channel::MsgChannelOpenInit,
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

    fn channel_open_try(
        &self,
        msg: super::msgs::channel::MsgChannelOpenTry,
    ) -> impl futures::Future<Output = ()> + '_ {
        async move {
            self.broadcast_tx_commit([google::protobuf::Any {
                type_url: "/ibc.channel.v1.MsgChannelOpenTry".to_string(),
                value: msg.into_proto_with_signer(&self.signer).encode_to_vec(),
            }])
            .await;
        }
    }

    fn channel_open_ack(
        &self,
        msg: super::msgs::channel::MsgChannelOpenAck,
    ) -> impl futures::Future<Output = ()> + '_ {
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
        msg: super::msgs::channel::MsgChannelOpenConfirm,
    ) -> impl futures::Future<Output = ()> + '_ {
        async move {
            self.broadcast_tx_commit([google::protobuf::Any {
                type_url: "/ibc.channel.v1.MsgChannelOpenConfirm".to_string(),
                value: msg.into_proto_with_signer(&self.signer).encode_to_vec(),
            }])
            .await;
        }
    }

    fn recv_packet(
        &self,
        msg: super::msgs::channel::MsgRecvPacket,
    ) -> impl futures::Future<Output = ()> + '_ {
        async move { todo!() }
    }

    fn generate_counterparty_client_state(
        &self,
        height: super::msgs::Height,
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

            super::msgs::cometbls::ClientState {
                // TODO(benluelo): Pass this in somehow
                chain_id: CHAIN_ID.into(),
                // https://github.com/cometbft/cometbft/blob/da0e55604b075bac9e1d5866cb2e62eaae386dd9/light/verifier.go#L16
                trust_level: super::msgs::Fraction {
                    numerator: 1,
                    denominator: 3,
                },
                // https://github.com/cosmos/relayer/blob/23d1e5c864b35d133cad6a0ef06970a2b1e1b03f/relayer/chains/cosmos/provider.go#L177
                trusting_period: super::msgs::Duration {
                    seconds: (unbonding_period * 85 / 100).as_secs().try_into().unwrap(),
                    nanos: (unbonding_period * 85 / 100)
                        .subsec_nanos()
                        .try_into()
                        .unwrap(),
                },
                unbonding_period: super::msgs::Duration {
                    seconds: unbonding_period.as_secs().try_into().unwrap(),
                    nanos: unbonding_period.subsec_nanos().try_into().unwrap(),
                },
                // https://github.com/cosmos/relayer/blob/23d1e5c864b35d133cad6a0ef06970a2b1e1b03f/relayer/chains/cosmos/provider.go#L177
                max_clock_drift: super::msgs::Duration {
                    seconds: 60 * 10,
                    nanos: 0,
                },
                frozen_height: super::msgs::Height {
                    revision_number: 0,
                    revision_height: 0,
                },
                latest_height: super::msgs::Height {
                    // extract this to a constant
                    revision_number: 0,
                    revision_height: height.value(),
                },
            }
        }
    }

    fn generate_counterparty_consensus_state(
        &self,
        height: super::msgs::Height,
    ) -> impl Future<Output = <Cometbls as LightClient>::ConsensusState> + '_ {
        async move {
            let commit = self
                .tm_client
                .commit(tendermint::block::Height::try_from(height.revision_height).unwrap())
                .await
                .unwrap();

            super::msgs::cometbls::ConsensusState {
                timestamp: {
                    let ts = commit.signed_header.header.time;
                    super::msgs::Timestamp {
                        seconds: ts.unix_timestamp(),
                        nanos: (ts.unix_timestamp_nanos()
                            - (ts.unix_timestamp() as i128 * 1_000_000_000_i128))
                            .try_into()
                            .unwrap(),
                    }
                },
                root: super::msgs::MerkleRoot {
                    hash: commit.signed_header.header.app_hash.as_bytes().to_vec(),
                },
                next_validators_hash: commit
                    .signed_header
                    .header
                    .next_validators_hash
                    .as_bytes()
                    .to_vec(),
            }
        }
    }

    fn generate_counterparty_update_client_message(
        &self,
    ) -> impl Future<Output = <Cometbls as LightClient>::UpdateClientMessage> + '_ {
        async move { todo!() }
    }
}

/// Wrapper type to indicate that a type is to be serialized as an Any.
pub struct Any<T>(T);

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

pub enum TryFromAnyError {
    IncorrectTypeUrl {
        found: String,
        expected: &'static str,
    },

    Prost(prost::DecodeError),
}

impl<T: TypeUrl + Default> TryFrom<google::protobuf::Any> for Any<T> {
    type Error = TryFromAnyError;

    fn try_from(value: google::protobuf::Any) -> Result<Self, Self::Error> {
        if value.type_url != T::TYPE_URL {
            Err(TryFromAnyError::IncorrectTypeUrl {
                found: value.type_url,
                expected: T::TYPE_URL,
            })
        } else {
            T::decode(&*value.value)
                .map_err(TryFromAnyError::Prost)
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

impl TypeUrl for wasm::v1::ClientState {
    const TYPE_URL: &'static str = "/ibc.lightclients.wasm.v1.ClientState";
}

impl TypeUrl for ethereum::v1::ClientState {
    const TYPE_URL: &'static str = "/ibc.lightclients.ethereum.v1.ClientState";
}

impl TypeUrl for ethereum::v1::ConsensusState {
    const TYPE_URL: &'static str = "/ibc.lightclients.ethereum.v1.ConsensusState";
}

impl<Data: IntoProto> From<super::msgs::wasm::ClientState<Data>> for wasm::v1::ClientState {
    fn from(val: super::msgs::wasm::ClientState<Data>) -> Self {
        wasm::v1::ClientState {
            data: val.data.into_proto().encode_to_vec(),
            code_id: val.code_id,
            latest_height: Some(val.latest_height.into()),
        }
    }
}

impl<Data: IntoProto> From<super::msgs::wasm::ConsensusState<Data>> for wasm::v1::ConsensusState {
    fn from(value: super::msgs::wasm::ConsensusState<Data>) -> Self {
        wasm::v1::ConsensusState {
            data: value.data.into_proto().encode_to_vec(),
            timestamp: value.timestamp,
        }
    }
}

impl<Data: IntoProto> IntoProto for super::msgs::wasm::ClientState<Data> {
    type Proto = wasm::v1::ClientState;
}

impl TypeUrl for wasm::v1::ConsensusState {
    const TYPE_URL: &'static str = "/ibc.lightclients.wasm.v1.ConsensusState";
}

impl<Data: IntoProto> IntoProto for super::msgs::wasm::ConsensusState<Data> {
    type Proto = wasm::v1::ConsensusState;
}

impl MsgIntoProto for super::msgs::connection::MsgConnectionOpenInit {
    type Proto = MsgConnectionOpenInit;

    fn into_proto_with_signer(self, signer: &XPrv) -> Self::Proto {
        MsgConnectionOpenInit {
            client_id: self.client_id,
            counterparty: Some(self.counterparty.into()),
            version: Some(self.version.into()),
            delay_period: self.delay_period,
            signer: signer_from_sk(signer),
        }
    }
}

impl<ClientState> MsgIntoProto for super::msgs::connection::MsgConnectionOpenTry<ClientState>
where
    ClientState: IntoProto,
    <ClientState as IntoProto>::Proto: TypeUrl,
{
    type Proto = MsgConnectionOpenTry;

    fn into_proto_with_signer(self, signer: &XPrv) -> Self::Proto {
        #[allow(deprecated)]
        MsgConnectionOpenTry {
            client_id: self.client_id,
            previous_connection_id: "".to_string(),
            client_state: Some(Any(self.client_state).into()),
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

impl<ClientState> MsgIntoProto for super::msgs::connection::MsgConnectionOpenAck<ClientState>
where
    ClientState: IntoProto,
    <ClientState as IntoProto>::Proto: TypeUrl,
{
    type Proto = MsgConnectionOpenAck;

    fn into_proto_with_signer(self, signer: &XPrv) -> Self::Proto {
        MsgConnectionOpenAck {
            connection_id: self.connection_id,
            counterparty_connection_id: self.counterparty_connection_id,
            version: Some(self.version.into()),
            client_state: Some(Any(self.client_state).into()),
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

impl MsgIntoProto for super::msgs::connection::MsgConnectionOpenConfirm {
    type Proto = MsgConnectionOpenConfirm;

    fn into_proto_with_signer(self, signer: &XPrv) -> Self::Proto {
        MsgConnectionOpenConfirm {
            connection_id: self.connection_id,
            proof_ack: self.proof_ack,
            proof_height: Some(self.proof_height.into()),
            signer: signer_from_sk(signer),
        }
    }
}

impl MsgIntoProto for super::msgs::channel::MsgChannelOpenInit {
    type Proto = MsgChannelOpenInit;

    fn into_proto_with_signer(self, signer: &XPrv) -> Self::Proto {
        MsgChannelOpenInit {
            port_id: self.port_id,
            channel: Some(self.channel.into()),
            signer: signer_from_sk(signer),
        }
    }
}

impl MsgIntoProto for super::msgs::channel::MsgChannelOpenTry {
    type Proto = MsgChannelOpenTry;

    fn into_proto_with_signer(self, signer: &XPrv) -> Self::Proto {
        #[allow(deprecated)]
        MsgChannelOpenTry {
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

impl MsgIntoProto for super::msgs::channel::MsgChannelOpenAck {
    type Proto = MsgChannelOpenAck;

    fn into_proto_with_signer(self, signer: &XPrv) -> Self::Proto {
        MsgChannelOpenAck {
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

impl MsgIntoProto for super::msgs::channel::MsgChannelOpenConfirm {
    type Proto = MsgChannelOpenConfirm;

    fn into_proto_with_signer(self, signer: &XPrv) -> Self::Proto {
        MsgChannelOpenConfirm {
            port_id: self.port_id,
            channel_id: self.channel_id,
            proof_ack: self.proof_ack,
            proof_height: Some(self.proof_height.into()),
            signer: signer_from_sk(signer),
        }
    }
}

impl From<client::v1::Height> for super::msgs::Height {
    fn from(proto: client::v1::Height) -> Self {
        Self {
            revision_number: proto.revision_number,
            revision_height: proto.revision_height,
        }
    }
}

impl From<super::msgs::Height> for client::v1::Height {
    fn from(value: super::msgs::Height) -> client::v1::Height {
        client::v1::Height {
            revision_number: value.revision_number,
            revision_height: value.revision_height,
        }
    }
}

/// A protobuf field was none unexpectedly.
#[derive(Debug)]
pub struct MissingField(&'static str);

impl From<super::msgs::connection::Counterparty> for connection::v1::Counterparty {
    fn from(value: super::msgs::connection::Counterparty) -> Self {
        Self {
            client_id: value.client_id,
            connection_id: value.connection_id,
            prefix: Some(value.prefix.into()),
        }
    }
}

impl TryFrom<connection::v1::Counterparty> for super::msgs::connection::Counterparty {
    type Error = MissingField;

    fn try_from(value: connection::v1::Counterparty) -> Result<Self, Self::Error> {
        Ok(Self {
            client_id: value.client_id,
            connection_id: value.connection_id,
            prefix: value.prefix.ok_or(MissingField("prefix"))?.into(),
        })
    }
}

impl TryFrom<connection::v1::Version> for super::msgs::connection::Version {
    type Error = strum::ParseError;

    fn try_from(proto: connection::v1::Version) -> Result<Self, Self::Error> {
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

impl From<super::msgs::connection::Version> for connection::v1::Version {
    fn from(value: super::msgs::connection::Version) -> Self {
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

impl From<commitment::v1::MerklePrefix> for super::msgs::MerklePrefix {
    fn from(proto: commitment::v1::MerklePrefix) -> Self {
        Self {
            key_prefix: proto.key_prefix,
        }
    }
}

impl From<super::msgs::MerklePrefix> for commitment::v1::MerklePrefix {
    fn from(value: super::msgs::MerklePrefix) -> Self {
        Self {
            key_prefix: value.key_prefix,
        }
    }
}

impl MsgIntoProto for super::msgs::channel::MsgRecvPacket {
    type Proto = MsgRecvPacket;

    fn into_proto_with_signer(self, signer: &XPrv) -> Self::Proto {
        MsgRecvPacket {
            packet: Some(self.packet.into()),
            proof_commitment: self.proof_commitment,
            proof_height: Some(self.proof_height.into()),
            signer: signer_from_sk(signer),
        }
    }
}

impl From<super::msgs::channel::Packet> for channel::v1::Packet {
    fn from(value: super::msgs::channel::Packet) -> Self {
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

impl TryFrom<channel::v1::Packet> for super::msgs::channel::Packet {
    type Error = MissingField;

    fn try_from(proto: channel::v1::Packet) -> Result<Self, Self::Error> {
        Ok(super::msgs::channel::Packet {
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

impl From<super::msgs::channel::Channel> for channel::v1::Channel {
    fn from(value: super::msgs::channel::Channel) -> Self {
        Self {
            state: value.state as i32,
            ordering: value.ordering as i32,
            counterparty: Some(value.counterparty.into()),
            connection_hops: value.connection_hops,
            version: value.version,
        }
    }
}

impl TryFrom<channel::v1::Channel> for super::msgs::channel::Channel {
    type Error = MissingField;

    fn try_from(proto: channel::v1::Channel) -> Result<Self, Self::Error> {
        Ok(super::msgs::channel::Channel {
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

impl From<super::msgs::ethereum::ClientState> for ethereum::v1::ClientState {
    fn from(value: super::msgs::ethereum::ClientState) -> Self {
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

impl From<super::msgs::ethereum::ConsensusState> for ethereum::v1::ConsensusState {
    fn from(value: super::msgs::ethereum::ConsensusState) -> Self {
        Self {
            slot: value.slot,
            storage_root: value.storage_root,
            timestamp: value.timestamp,
            current_sync_committee: value.current_sync_committee,
            next_sync_committee: value.next_sync_committee,
        }
    }
}

impl TryFrom<ethereum::v1::ClientState> for super::msgs::ethereum::ClientState {
    type Error = MissingField;

    fn try_from(value: ethereum::v1::ClientState) -> Result<Self, Self::Error> {
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

impl TryFromProto for super::msgs::ethereum::ClientState {
    type Proto = ethereum::v1::ClientState;
}

impl IntoProto for super::msgs::ethereum::ClientState {
    type Proto = ethereum::v1::ClientState;
}

impl IntoProto for super::msgs::ethereum::ConsensusState {
    type Proto = ethereum::v1::ConsensusState;
}

impl From<super::msgs::Fraction> for ethereum::v1::Fraction {
    fn from(value: super::msgs::Fraction) -> Self {
        Self {
            numerator: value.numerator,
            denominator: value.denominator,
        }
    }
}

impl From<super::msgs::Fraction> for cometbls::v1::Fraction {
    fn from(value: super::msgs::Fraction) -> Self {
        Self {
            numerator: value.numerator,
            denominator: value.denominator,
        }
    }
}

impl From<super::msgs::Fraction> for lightclients::tendermint::v1::Fraction {
    fn from(value: super::msgs::Fraction) -> Self {
        Self {
            numerator: value.numerator,
            denominator: value.denominator,
        }
    }
}

impl From<super::msgs::Duration> for google::protobuf::Duration {
    fn from(value: super::msgs::Duration) -> Self {
        Self {
            seconds: value.seconds,
            nanos: value.nanos,
        }
    }
}

impl From<ethereum::v1::Fraction> for super::msgs::Fraction {
    fn from(proto: ethereum::v1::Fraction) -> Self {
        Self {
            numerator: proto.numerator,
            denominator: proto.denominator,
        }
    }
}

impl From<super::msgs::ethereum::ForkParameters> for ethereum::v1::ForkParameters {
    fn from(value: super::msgs::ethereum::ForkParameters) -> Self {
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

impl TryFrom<ethereum::v1::ForkParameters> for super::msgs::ethereum::ForkParameters {
    type Error = MissingField;

    fn try_from(proto: ethereum::v1::ForkParameters) -> Result<Self, Self::Error> {
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

impl From<super::msgs::ethereum::Fork> for ethereum::v1::Fork {
    fn from(value: super::msgs::ethereum::Fork) -> Self {
        Self {
            version: value.version,
            epoch: value.epoch,
        }
    }
}

impl From<ethereum::v1::Fork> for super::msgs::ethereum::Fork {
    fn from(proto: ethereum::v1::Fork) -> Self {
        Self {
            version: proto.version,
            epoch: proto.epoch,
        }
    }
}

impl From<super::msgs::channel::Counterparty> for channel::v1::Counterparty {
    fn from(value: super::msgs::channel::Counterparty) -> Self {
        Self {
            port_id: value.port_id,
            channel_id: value.channel_id,
        }
    }
}

impl From<channel::v1::Counterparty> for super::msgs::channel::Counterparty {
    fn from(proto: channel::v1::Counterparty) -> Self {
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

impl<Data> TryFrom<wasm::v1::ClientState> for super::msgs::wasm::ClientState<Data>
where
    Data: TryFromProto,
    <Data as TryFromProto>::Proto: prost::Message + Default,
{
    type Error =
        TryFromWasmClientStateError<<Data as TryFrom<<Data as TryFromProto>::Proto>>::Error>;

    fn try_from(value: wasm::v1::ClientState) -> Result<Self, Self::Error> {
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

impl TypeUrl for lightclients::tendermint::v1::ClientState {
    const TYPE_URL: &'static str = "/ibc.lightclients.tendermint.v1.ClientState";
}

impl From<super::msgs::tendermint::ClientState> for lightclients::tendermint::v1::ClientState {
    fn from(val: super::msgs::tendermint::ClientState) -> Self {
        #[allow(deprecated)]
        lightclients::tendermint::v1::ClientState {
            latest_height: Some(val.latest_height.into()),
            chain_id: val.chain_id,
            trust_level: Some(val.trust_level.into()),
            trusting_period: Some(val.trusting_period.into()),
            unbonding_period: Some(val.unbonding_period.into()),
            max_clock_drift: Some(val.max_clock_drift.into()),
            frozen_height: Some(val.frozen_height.into()),
            proof_specs: val.proof_specs.into_iter().map(Into::into).collect(),
            upgrade_path: val.upgrade_path,
            allow_update_after_expiry: true,
            allow_update_after_misbehaviour: true,
        }
    }
}

impl IntoProto for super::msgs::tendermint::ClientState {
    type Proto = lightclients::tendermint::v1::ClientState;
}

impl From<super::msgs::ics23::ProofSpec> for ics23::v1::ProofSpec {
    fn from(value: super::msgs::ics23::ProofSpec) -> Self {
        Self {
            leaf_spec: Some(value.leaf_spec.into()),
            inner_spec: Some(value.inner_spec.into()),
            max_depth: value.max_depth,
            min_depth: value.min_depth,
        }
    }
}

impl From<super::msgs::ics23::InnerSpec> for ics23::v1::InnerSpec {
    fn from(value: super::msgs::ics23::InnerSpec) -> Self {
        Self {
            child_order: value.child_order,
            child_size: value.child_size,
            min_prefix_length: value.min_prefix_length,
            max_prefix_length: value.max_prefix_length,
            empty_child: value.empty_child,
            // TODO(benluelo): Better conversion here, go into the proto generated enum and then cast
            hash: value.hash as i32,
        }
    }
}

impl From<super::msgs::ics23::LeafOp> for ics23::v1::LeafOp {
    fn from(value: super::msgs::ics23::LeafOp) -> Self {
        Self {
            hash: value.hash as i32,
            prehash_key: value.prehash_key as i32,
            prehash_value: value.prehash_value as i32,
            length: value.length as i32,
            prefix: value.prefix,
        }
    }
}

impl IntoProto for super::msgs::ConnectionEnd {
    type Proto = connection::v1::ConnectionEnd;
}

pub enum TryFromConnnectionEndError {
    ParseError(ParseError),
    UnknownEnumVariant(super::msgs::UnknownEnumVariant<i32>),
    MissingField(MissingField),
}

impl TryFrom<connection::v1::ConnectionEnd> for super::msgs::ConnectionEnd {
    type Error = TryFromConnnectionEndError;

    fn try_from(val: connection::v1::ConnectionEnd) -> Result<Self, Self::Error> {
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

impl From<super::msgs::ConnectionEnd> for connection::v1::ConnectionEnd {
    fn from(val: super::msgs::ConnectionEnd) -> Self {
        Self {
            client_id: val.client_id,
            versions: val.versions.into_iter().map(|x| x.into()).collect(),
            state: val.state as i32,
            counterparty: Some(val.counterparty.into()),
            delay_period: val.delay_period,
        }
    }
}

impl From<super::msgs::connection::State> for connection::v1::State {
    fn from(value: super::msgs::connection::State) -> Self {
        match value {
            super::msgs::connection::State::UninitializedUnspecified => {
                connection::v1::State::UninitializedUnspecified
            }
            super::msgs::connection::State::Init => connection::v1::State::Init,
            super::msgs::connection::State::Tryopen => connection::v1::State::Tryopen,
            super::msgs::connection::State::Open => connection::v1::State::Open,
        }
    }
}

impl From<connection::v1::State> for super::msgs::connection::State {
    fn from(value: connection::v1::State) -> Self {
        match value {
            connection::v1::State::UninitializedUnspecified => {
                super::msgs::connection::State::UninitializedUnspecified
            }
            connection::v1::State::Init => super::msgs::connection::State::Init,
            connection::v1::State::Tryopen => super::msgs::connection::State::Tryopen,
            connection::v1::State::Open => super::msgs::connection::State::Open,
        }
    }
}

impl From<super::msgs::cometbls::ClientState> for cometbls::v1::ClientState {
    fn from(value: super::msgs::cometbls::ClientState) -> Self {
        Self {
            chain_id: value.chain_id,
            trust_level: Some(value.trust_level.into()),
            trusting_period: Some(value.trusting_period.into()),
            unbonding_period: Some(value.unbonding_period.into()),
            max_clock_drift: Some(value.max_clock_drift.into()),
            frozen_height: Some(value.frozen_height.into()),
            latest_height: Some(value.latest_height.into()),
        }
    }
}

impl IntoProto for super::msgs::cometbls::ClientState {
    type Proto = cometbls::v1::ClientState;
}

impl TypeUrl for cometbls::v1::ClientState {
    const TYPE_URL: &'static str = "/union.ibc.lightclients.cometbls.v1.ClientState";
}

impl From<super::msgs::cometbls::ConsensusState> for cometbls::v1::ConsensusState {
    fn from(value: super::msgs::cometbls::ConsensusState) -> Self {
        Self {
            timestamp: Some(value.timestamp.into()),
            root: Some(value.root.into()),
            next_validators_hash: value.next_validators_hash,
        }
    }
}

impl From<super::msgs::MerkleRoot> for commitment::v1::MerkleRoot {
    fn from(value: super::msgs::MerkleRoot) -> Self {
        Self { hash: value.hash }
    }
}

impl From<super::msgs::Timestamp> for google::protobuf::Timestamp {
    fn from(value: super::msgs::Timestamp) -> Self {
        Self {
            seconds: value.seconds,
            nanos: value.nanos,
        }
    }
}
