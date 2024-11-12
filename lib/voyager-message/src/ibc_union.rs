use serde::{Deserialize, Serialize};
use unionlabs::{
    bytes::Bytes,
    hash::H256,
    ics24::ethabi::{
        BatchPacketsPath, BatchReceiptsPath, ChannelPath, ClientStatePath, ConnectionPath,
        ConsensusStatePath, Path,
    },
};
use voyager_core::IbcVersionId;

use crate::{IbcSpec, IbcStorePathKey};

pub enum IbcUnion {}

impl IbcSpec for IbcUnion {
    const ID: IbcVersionId = IbcVersionId::new_static(IbcVersionId::UNION);

    type ClientId = u32;

    type StorePath = Path;

    type Datagram = IbcMsgs;

    type Event = IbcEvents;

    fn client_state_path(client_id: Self::ClientId) -> Self::StorePath {
        unionlabs::ics24::ethabi::ClientStatePath { client_id }.into()
    }

    fn consensus_state_path(
        client_id: Self::ClientId,
        height: unionlabs::ibc::core::client::height::Height,
    ) -> Self::StorePath {
        unionlabs::ics24::ethabi::ConsensusStatePath {
            client_id,
            height: height.height(),
        }
        .into()
    }
}

impl IbcStorePathKey for ClientStatePath {
    type Spec = IbcUnion;

    type Value = Bytes;
}

impl IbcStorePathKey for ConsensusStatePath {
    type Spec = IbcUnion;

    type Value = Bytes;
}

impl IbcStorePathKey for ConnectionPath {
    type Spec = IbcUnion;

    type Value = ibc_solidity::ibc::Connection;
}

impl IbcStorePathKey for ChannelPath {
    type Spec = IbcUnion;

    type Value = ibc_solidity::ibc::Channel;
}

impl IbcStorePathKey for BatchReceiptsPath {
    type Spec = IbcUnion;

    type Value = H256;
}

impl IbcStorePathKey for BatchPacketsPath {
    type Spec = IbcUnion;

    type Value = H256;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "@type", content = "@value", rename_all = "snake_case")]
pub enum IbcMsgs {
    CreateClient(ibc_solidity::ibc::MsgCreateClient),
    UpdateClient(ibc_solidity::ibc::MsgUpdateClient),
    ConnectionOpenInit(ibc_solidity::ibc::MsgConnectionOpenInit),
    ConnectionOpenTry(ibc_solidity::ibc::MsgConnectionOpenTry),
    ConnectionOpenAck(ibc_solidity::ibc::MsgConnectionOpenAck),
    ConnectionOpenConfirm(ibc_solidity::ibc::MsgConnectionOpenConfirm),
    ChannelOpenInit(ibc_solidity::ibc::MsgChannelOpenInit),
    ChannelOpenTry(ibc_solidity::ibc::MsgChannelOpenTry),
    ChannelOpenAck(ibc_solidity::ibc::MsgChannelOpenAck),
    ChannelOpenConfirm(ibc_solidity::ibc::MsgChannelOpenConfirm),
    ChannelCloseInit(ibc_solidity::ibc::MsgChannelCloseInit),
    ChannelCloseConfirm(ibc_solidity::ibc::MsgChannelCloseConfirm),
    PacketRecv(ibc_solidity::ibc::MsgPacketRecv),
    PacketAcknowledgement(ibc_solidity::ibc::MsgPacketAcknowledgement),
    PacketTimeout(ibc_solidity::ibc::MsgPacketTimeout),
    IntentPacketRecv(ibc_solidity::ibc::MsgIntentPacketRecv),
    BatchSend(ibc_solidity::ibc::MsgBatchSend),
    BatchAcks(ibc_solidity::ibc::MsgBatchAcks),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "@type", content = "@value", rename_all = "snake_case")]
pub enum IbcEvents {
    ClientRegistered(ibc_solidity::ibc::Ibc::ClientRegistered),
    ClientCreated(ibc_solidity::ibc::Ibc::ClientCreated),
    ClientUpdated(ibc_solidity::ibc::Ibc::ClientUpdated),
    ConnectionOpenInit(ibc_solidity::ibc::Ibc::ConnectionOpenInit),
    ConnectionOpenTry(ibc_solidity::ibc::Ibc::ConnectionOpenTry),
    ConnectionOpenAck(ibc_solidity::ibc::Ibc::ConnectionOpenAck),
    ConnectionOpenConfirm(ibc_solidity::ibc::Ibc::ConnectionOpenConfirm),
    ChannelOpenInit(ibc_solidity::ibc::Ibc::ChannelOpenInit),
    ChannelOpenTry(ibc_solidity::ibc::Ibc::ChannelOpenTry),
    ChannelOpenAck(ibc_solidity::ibc::Ibc::ChannelOpenAck),
    ChannelOpenConfirm(ibc_solidity::ibc::Ibc::ChannelOpenConfirm),
    ChannelCloseInit(ibc_solidity::ibc::Ibc::ChannelCloseInit),
    ChannelCloseConfirm(ibc_solidity::ibc::Ibc::ChannelCloseConfirm),
    SendPacket(ibc_solidity::ibc::Ibc::SendPacket),
    RecvPacket(ibc_solidity::ibc::Ibc::RecvPacket),
    RecvIntentPacket(ibc_solidity::ibc::Ibc::RecvIntentPacket),
    WriteAcknowledgement(ibc_solidity::ibc::Ibc::WriteAcknowledgement),
    AcknowledgePacket(ibc_solidity::ibc::Ibc::AcknowledgePacket),
    TimeoutPacket(ibc_solidity::ibc::Ibc::TimeoutPacket),
}
