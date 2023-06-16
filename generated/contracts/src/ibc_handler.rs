pub use ibc_handler::*;
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
pub mod ibc_handler {
    pub use super::super::shared_types::*;
    #[rustfmt::skip]
    const __ABI: &str = "[{\"inputs\":[{\"internalType\":\"struct IbcCoreChannelV1Packet.Data\",\"name\":\"packet\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"sequence\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"source_port\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"source_channel\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"destination_port\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"destination_channel\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"data\",\"type\":\"bytes\",\"components\":[]},{\"internalType\":\"struct IbcCoreClientV1Height.Data\",\"name\":\"timeout_height\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"revision_number\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"revision_height\",\"type\":\"uint64\",\"components\":[]}]},{\"internalType\":\"uint64\",\"name\":\"timeout_timestamp\",\"type\":\"uint64\",\"components\":[]}],\"indexed\":false},{\"internalType\":\"bytes\",\"name\":\"acknowledgement\",\"type\":\"bytes\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"AcknowledgePacket\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"GeneratedChannelIdentifier\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"GeneratedClientIdentifier\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"GeneratedConnectionIdentifier\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"struct IbcCoreChannelV1Packet.Data\",\"name\":\"packet\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"sequence\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"source_port\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"source_channel\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"destination_port\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"destination_channel\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"data\",\"type\":\"bytes\",\"components\":[]},{\"internalType\":\"struct IbcCoreClientV1Height.Data\",\"name\":\"timeout_height\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"revision_number\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"revision_height\",\"type\":\"uint64\",\"components\":[]}]},{\"internalType\":\"uint64\",\"name\":\"timeout_timestamp\",\"type\":\"uint64\",\"components\":[]}],\"indexed\":false}],\"type\":\"event\",\"name\":\"RecvPacket\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"uint64\",\"name\":\"sequence\",\"type\":\"uint64\",\"components\":[],\"indexed\":false},{\"internalType\":\"string\",\"name\":\"sourcePort\",\"type\":\"string\",\"components\":[],\"indexed\":false},{\"internalType\":\"string\",\"name\":\"sourceChannel\",\"type\":\"string\",\"components\":[],\"indexed\":false},{\"internalType\":\"struct IbcCoreClientV1Height.Data\",\"name\":\"timeoutHeight\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"revision_number\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"revision_height\",\"type\":\"uint64\",\"components\":[]}],\"indexed\":false},{\"internalType\":\"uint64\",\"name\":\"timeoutTimestamp\",\"type\":\"uint64\",\"components\":[],\"indexed\":false},{\"internalType\":\"bytes\",\"name\":\"data\",\"type\":\"bytes\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"SendPacket\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"destinationPortId\",\"type\":\"string\",\"components\":[],\"indexed\":false},{\"internalType\":\"string\",\"name\":\"destinationChannel\",\"type\":\"string\",\"components\":[],\"indexed\":false},{\"internalType\":\"uint64\",\"name\":\"sequence\",\"type\":\"uint64\",\"components\":[],\"indexed\":false},{\"internalType\":\"bytes\",\"name\":\"acknowledgement\",\"type\":\"bytes\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"WriteAcknowledgement\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"struct IBCMsgs.MsgPacketAcknowledgement\",\"name\":\"msg_\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct IbcCoreChannelV1Packet.Data\",\"name\":\"packet\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"sequence\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"source_port\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"source_channel\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"destination_port\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"destination_channel\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"data\",\"type\":\"bytes\",\"components\":[]},{\"internalType\":\"struct IbcCoreClientV1Height.Data\",\"name\":\"timeout_height\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"revision_number\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"revision_height\",\"type\":\"uint64\",\"components\":[]}]},{\"internalType\":\"uint64\",\"name\":\"timeout_timestamp\",\"type\":\"uint64\",\"components\":[]}]},{\"internalType\":\"bytes\",\"name\":\"acknowledgement\",\"type\":\"bytes\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"proof\",\"type\":\"bytes\",\"components\":[]},{\"internalType\":\"struct IbcCoreClientV1Height.Data\",\"name\":\"proofHeight\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"revision_number\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"revision_height\",\"type\":\"uint64\",\"components\":[]}]}]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"acknowledgePacket\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"portId\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"moduleAddress\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"bindPort\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"portId\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"channelId\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"pure\",\"type\":\"function\",\"name\":\"channelCapabilityPath\",\"outputs\":[{\"internalType\":\"bytes\",\"name\":\"\",\"type\":\"bytes\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"struct IBCMsgs.MsgChannelCloseConfirm\",\"name\":\"msg_\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"string\",\"name\":\"portId\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"channelId\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"proofInit\",\"type\":\"bytes\",\"components\":[]},{\"internalType\":\"struct IbcCoreClientV1Height.Data\",\"name\":\"proofHeight\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"revision_number\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"revision_height\",\"type\":\"uint64\",\"components\":[]}]}]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"channelCloseConfirm\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"struct IBCMsgs.MsgChannelCloseInit\",\"name\":\"msg_\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"string\",\"name\":\"portId\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"channelId\",\"type\":\"string\",\"components\":[]}]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"channelCloseInit\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"struct IBCMsgs.MsgChannelOpenAck\",\"name\":\"msg_\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"string\",\"name\":\"portId\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"channelId\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"counterpartyVersion\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"counterpartyChannelId\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"proofTry\",\"type\":\"bytes\",\"components\":[]},{\"internalType\":\"struct IbcCoreClientV1Height.Data\",\"name\":\"proofHeight\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"revision_number\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"revision_height\",\"type\":\"uint64\",\"components\":[]}]}]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"channelOpenAck\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"struct IBCMsgs.MsgChannelOpenConfirm\",\"name\":\"msg_\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"string\",\"name\":\"portId\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"channelId\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"proofAck\",\"type\":\"bytes\",\"components\":[]},{\"internalType\":\"struct IbcCoreClientV1Height.Data\",\"name\":\"proofHeight\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"revision_number\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"revision_height\",\"type\":\"uint64\",\"components\":[]}]}]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"channelOpenConfirm\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"struct IBCMsgs.MsgChannelOpenInit\",\"name\":\"msg_\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"string\",\"name\":\"portId\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"struct IbcCoreChannelV1Channel.Data\",\"name\":\"channel\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"enum IbcCoreChannelV1GlobalEnums.State\",\"name\":\"state\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"enum IbcCoreChannelV1GlobalEnums.Order\",\"name\":\"ordering\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"struct IbcCoreChannelV1Counterparty.Data\",\"name\":\"counterparty\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"string\",\"name\":\"port_id\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"channel_id\",\"type\":\"string\",\"components\":[]}]},{\"internalType\":\"string[]\",\"name\":\"connection_hops\",\"type\":\"string[]\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"version\",\"type\":\"string\",\"components\":[]}]}]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"channelOpenInit\",\"outputs\":[{\"internalType\":\"string\",\"name\":\"channelId\",\"type\":\"string\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"struct IBCMsgs.MsgChannelOpenTry\",\"name\":\"msg_\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"string\",\"name\":\"portId\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"struct IbcCoreChannelV1Channel.Data\",\"name\":\"channel\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"enum IbcCoreChannelV1GlobalEnums.State\",\"name\":\"state\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"enum IbcCoreChannelV1GlobalEnums.Order\",\"name\":\"ordering\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"struct IbcCoreChannelV1Counterparty.Data\",\"name\":\"counterparty\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"string\",\"name\":\"port_id\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"channel_id\",\"type\":\"string\",\"components\":[]}]},{\"internalType\":\"string[]\",\"name\":\"connection_hops\",\"type\":\"string[]\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"version\",\"type\":\"string\",\"components\":[]}]},{\"internalType\":\"string\",\"name\":\"counterpartyVersion\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"proofInit\",\"type\":\"bytes\",\"components\":[]},{\"internalType\":\"struct IbcCoreClientV1Height.Data\",\"name\":\"proofHeight\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"revision_number\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"revision_height\",\"type\":\"uint64\",\"components\":[]}]}]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"channelOpenTry\",\"outputs\":[{\"internalType\":\"string\",\"name\":\"channelId\",\"type\":\"string\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"struct IBCMsgs.MsgConnectionOpenAck\",\"name\":\"msg_\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"string\",\"name\":\"connectionId\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"clientStateBytes\",\"type\":\"bytes\",\"components\":[]},{\"internalType\":\"struct IbcCoreConnectionV1Version.Data\",\"name\":\"version\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"string\",\"name\":\"identifier\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"string[]\",\"name\":\"features\",\"type\":\"string[]\",\"components\":[]}]},{\"internalType\":\"string\",\"name\":\"counterpartyConnectionID\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"proofTry\",\"type\":\"bytes\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"proofClient\",\"type\":\"bytes\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"proofConsensus\",\"type\":\"bytes\",\"components\":[]},{\"internalType\":\"struct IbcCoreClientV1Height.Data\",\"name\":\"proofHeight\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"revision_number\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"revision_height\",\"type\":\"uint64\",\"components\":[]}]},{\"internalType\":\"struct IbcCoreClientV1Height.Data\",\"name\":\"consensusHeight\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"revision_number\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"revision_height\",\"type\":\"uint64\",\"components\":[]}]}]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"connectionOpenAck\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"struct IBCMsgs.MsgConnectionOpenConfirm\",\"name\":\"msg_\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"string\",\"name\":\"connectionId\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"proofAck\",\"type\":\"bytes\",\"components\":[]},{\"internalType\":\"struct IbcCoreClientV1Height.Data\",\"name\":\"proofHeight\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"revision_number\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"revision_height\",\"type\":\"uint64\",\"components\":[]}]}]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"connectionOpenConfirm\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"struct IBCMsgs.MsgConnectionOpenInit\",\"name\":\"msg_\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"string\",\"name\":\"clientId\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"struct IbcCoreConnectionV1Counterparty.Data\",\"name\":\"counterparty\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"string\",\"name\":\"client_id\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"connection_id\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"struct IbcCoreCommitmentV1MerklePrefix.Data\",\"name\":\"prefix\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"bytes\",\"name\":\"key_prefix\",\"type\":\"bytes\",\"components\":[]}]}]},{\"internalType\":\"uint64\",\"name\":\"delayPeriod\",\"type\":\"uint64\",\"components\":[]}]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"connectionOpenInit\",\"outputs\":[{\"internalType\":\"string\",\"name\":\"connectionId\",\"type\":\"string\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"struct IBCMsgs.MsgConnectionOpenTry\",\"name\":\"msg_\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct IbcCoreConnectionV1Counterparty.Data\",\"name\":\"counterparty\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"string\",\"name\":\"client_id\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"connection_id\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"struct IbcCoreCommitmentV1MerklePrefix.Data\",\"name\":\"prefix\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"bytes\",\"name\":\"key_prefix\",\"type\":\"bytes\",\"components\":[]}]}]},{\"internalType\":\"uint64\",\"name\":\"delayPeriod\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"clientId\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"clientStateBytes\",\"type\":\"bytes\",\"components\":[]},{\"internalType\":\"struct IbcCoreConnectionV1Version.Data[]\",\"name\":\"counterpartyVersions\",\"type\":\"tuple[]\",\"components\":[{\"internalType\":\"string\",\"name\":\"identifier\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"string[]\",\"name\":\"features\",\"type\":\"string[]\",\"components\":[]}]},{\"internalType\":\"bytes\",\"name\":\"proofInit\",\"type\":\"bytes\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"proofClient\",\"type\":\"bytes\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"proofConsensus\",\"type\":\"bytes\",\"components\":[]},{\"internalType\":\"struct IbcCoreClientV1Height.Data\",\"name\":\"proofHeight\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"revision_number\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"revision_height\",\"type\":\"uint64\",\"components\":[]}]},{\"internalType\":\"struct IbcCoreClientV1Height.Data\",\"name\":\"consensusHeight\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"revision_number\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"revision_height\",\"type\":\"uint64\",\"components\":[]}]}]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"connectionOpenTry\",\"outputs\":[{\"internalType\":\"string\",\"name\":\"connectionId\",\"type\":\"string\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"struct IBCMsgs.MsgCreateClient\",\"name\":\"msg_\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"string\",\"name\":\"clientType\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"clientStateBytes\",\"type\":\"bytes\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"consensusStateBytes\",\"type\":\"bytes\",\"components\":[]}]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"createClient\",\"outputs\":[{\"internalType\":\"string\",\"name\":\"clientId\",\"type\":\"string\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"portId\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"channelId\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"getChannel\",\"outputs\":[{\"internalType\":\"struct IbcCoreChannelV1Channel.Data\",\"name\":\"\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"enum IbcCoreChannelV1GlobalEnums.State\",\"name\":\"state\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"enum IbcCoreChannelV1GlobalEnums.Order\",\"name\":\"ordering\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"struct IbcCoreChannelV1Counterparty.Data\",\"name\":\"counterparty\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"string\",\"name\":\"port_id\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"channel_id\",\"type\":\"string\",\"components\":[]}]},{\"internalType\":\"string[]\",\"name\":\"connection_hops\",\"type\":\"string[]\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"version\",\"type\":\"string\",\"components\":[]}]},{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"clientId\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"getClientState\",\"outputs\":[{\"internalType\":\"bytes\",\"name\":\"\",\"type\":\"bytes\",\"components\":[]},{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"connectionId\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"getConnection\",\"outputs\":[{\"internalType\":\"struct IbcCoreConnectionV1ConnectionEnd.Data\",\"name\":\"\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"string\",\"name\":\"client_id\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"struct IbcCoreConnectionV1Version.Data[]\",\"name\":\"versions\",\"type\":\"tuple[]\",\"components\":[{\"internalType\":\"string\",\"name\":\"identifier\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"string[]\",\"name\":\"features\",\"type\":\"string[]\",\"components\":[]}]},{\"internalType\":\"enum IbcCoreConnectionV1GlobalEnums.State\",\"name\":\"state\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"struct IbcCoreConnectionV1Counterparty.Data\",\"name\":\"counterparty\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"string\",\"name\":\"client_id\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"connection_id\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"struct IbcCoreCommitmentV1MerklePrefix.Data\",\"name\":\"prefix\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"bytes\",\"name\":\"key_prefix\",\"type\":\"bytes\",\"components\":[]}]}]},{\"internalType\":\"uint64\",\"name\":\"delay_period\",\"type\":\"uint64\",\"components\":[]}]},{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"connectionId\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"getConnectionSerialize\",\"outputs\":[{\"internalType\":\"bytes\",\"name\":\"\",\"type\":\"bytes\",\"components\":[]},{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"clientId\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"struct IbcCoreClientV1Height.Data\",\"name\":\"height\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"revision_number\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"revision_height\",\"type\":\"uint64\",\"components\":[]}]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"getConsensusState\",\"outputs\":[{\"internalType\":\"bytes\",\"name\":\"consensusStateBytes\",\"type\":\"bytes\",\"components\":[]},{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"getExpectedTimePerBlock\",\"outputs\":[{\"internalType\":\"uint64\",\"name\":\"\",\"type\":\"uint64\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"portId\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"channelId\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"sequence\",\"type\":\"uint64\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"getHashedPacketAcknowledgementCommitment\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"portId\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"channelId\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"sequence\",\"type\":\"uint64\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"getHashedPacketCommitment\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"portId\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"channelId\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"getNextSequenceSend\",\"outputs\":[{\"internalType\":\"uint64\",\"name\":\"\",\"type\":\"uint64\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"portId\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"channelId\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"sequence\",\"type\":\"uint64\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"hasPacketReceipt\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"portId\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"pure\",\"type\":\"function\",\"name\":\"portCapabilityPath\",\"outputs\":[{\"internalType\":\"bytes\",\"name\":\"\",\"type\":\"bytes\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"struct IBCMsgs.MsgPacketRecv\",\"name\":\"msg_\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct IbcCoreChannelV1Packet.Data\",\"name\":\"packet\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"sequence\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"source_port\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"source_channel\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"destination_port\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"destination_channel\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"data\",\"type\":\"bytes\",\"components\":[]},{\"internalType\":\"struct IbcCoreClientV1Height.Data\",\"name\":\"timeout_height\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"revision_number\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"revision_height\",\"type\":\"uint64\",\"components\":[]}]},{\"internalType\":\"uint64\",\"name\":\"timeout_timestamp\",\"type\":\"uint64\",\"components\":[]}]},{\"internalType\":\"bytes\",\"name\":\"proof\",\"type\":\"bytes\",\"components\":[]},{\"internalType\":\"struct IbcCoreClientV1Height.Data\",\"name\":\"proofHeight\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"revision_number\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"revision_height\",\"type\":\"uint64\",\"components\":[]}]}]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"recvPacket\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"clientType\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"contract ILightClient\",\"name\":\"client\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"registerClient\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"sourcePort\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"sourceChannel\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"struct IbcCoreClientV1Height.Data\",\"name\":\"timeoutHeight\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"revision_number\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"revision_height\",\"type\":\"uint64\",\"components\":[]}]},{\"internalType\":\"uint64\",\"name\":\"timeoutTimestamp\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"data\",\"type\":\"bytes\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"sendPacket\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"uint64\",\"name\":\"expectedTimePerBlock_\",\"type\":\"uint64\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"setExpectedTimePerBlock\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"struct IBCMsgs.MsgUpdateClient\",\"name\":\"msg_\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"string\",\"name\":\"clientId\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"clientMessage\",\"type\":\"bytes\",\"components\":[]}]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"updateClient\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"destinationPortId\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"destinationChannel\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"sequence\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"acknowledgement\",\"type\":\"bytes\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"writeAcknowledgement\",\"outputs\":[]}]";
    ///The parsed JSON ABI of the contract.
    pub static IBCHANDLER_ABI: ::ethers::contract::Lazy<::ethers::core::abi::Abi> =
        ::ethers::contract::Lazy::new(|| {
            ::ethers::core::utils::__serde_json::from_str(__ABI).expect("ABI is always valid")
        });
    pub struct IBCHandler<M>(::ethers::contract::Contract<M>);
    impl<M> ::core::clone::Clone for IBCHandler<M> {
        fn clone(&self) -> Self {
            Self(::core::clone::Clone::clone(&self.0))
        }
    }
    impl<M> ::core::ops::Deref for IBCHandler<M> {
        type Target = ::ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> ::core::ops::DerefMut for IBCHandler<M> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<M> ::core::fmt::Debug for IBCHandler<M> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple(stringify!(IBCHandler))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ::ethers::providers::Middleware> IBCHandler<M> {
        /// Creates a new contract instance with the specified `ethers` client at
        /// `address`. The contract derefs to a `ethers::Contract` object.
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(::ethers::contract::Contract::new(
                address.into(),
                IBCHANDLER_ABI.clone(),
                client,
            ))
        }
        ///Calls the contract's `acknowledgePacket` (0x59f37976) function
        pub fn acknowledge_packet(
            &self,
            msg: MsgPacketAcknowledgement,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([89, 243, 121, 118], (msg,))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `bindPort` (0x117e886a) function
        pub fn bind_port(
            &self,
            port_id: ::std::string::String,
            module_address: ::ethers::core::types::Address,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([17, 126, 136, 106], (port_id, module_address))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `channelCapabilityPath` (0x3bc3339f) function
        pub fn channel_capability_path(
            &self,
            port_id: ::std::string::String,
            channel_id: ::std::string::String,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::Bytes> {
            self.0
                .method_hash([59, 195, 51, 159], (port_id, channel_id))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `channelCloseConfirm` (0x25cbc3a6) function
        pub fn channel_close_confirm(
            &self,
            msg: MsgChannelCloseConfirm,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([37, 203, 195, 166], (msg,))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `channelCloseInit` (0xa06cb3a2) function
        pub fn channel_close_init(
            &self,
            msg: MsgChannelCloseInit,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([160, 108, 179, 162], (msg,))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `channelOpenAck` (0x256c4199) function
        pub fn channel_open_ack(
            &self,
            msg: MsgChannelOpenAck,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([37, 108, 65, 153], (msg,))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `channelOpenConfirm` (0x5bd51b62) function
        pub fn channel_open_confirm(
            &self,
            msg: MsgChannelOpenConfirm,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([91, 213, 27, 98], (msg,))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `channelOpenInit` (0xdd3469fc) function
        pub fn channel_open_init(
            &self,
            msg: MsgChannelOpenInit,
        ) -> ::ethers::contract::builders::ContractCall<M, ::std::string::String> {
            self.0
                .method_hash([221, 52, 105, 252], (msg,))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `channelOpenTry` (0x11b88a15) function
        pub fn channel_open_try(
            &self,
            msg: MsgChannelOpenTry,
        ) -> ::ethers::contract::builders::ContractCall<M, ::std::string::String> {
            self.0
                .method_hash([17, 184, 138, 21], (msg,))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `connectionOpenAck` (0xb531861f) function
        pub fn connection_open_ack(
            &self,
            msg: MsgConnectionOpenAck,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([181, 49, 134, 31], (msg,))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `connectionOpenConfirm` (0x6a728f2c) function
        pub fn connection_open_confirm(
            &self,
            msg: MsgConnectionOpenConfirm,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([106, 114, 143, 44], (msg,))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `connectionOpenInit` (0x01c6400f) function
        pub fn connection_open_init(
            &self,
            msg: MsgConnectionOpenInit,
        ) -> ::ethers::contract::builders::ContractCall<M, ::std::string::String> {
            self.0
                .method_hash([1, 198, 64, 15], (msg,))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `connectionOpenTry` (0x04f68e5c) function
        pub fn connection_open_try(
            &self,
            msg: MsgConnectionOpenTry,
        ) -> ::ethers::contract::builders::ContractCall<M, ::std::string::String> {
            self.0
                .method_hash([4, 246, 142, 92], (msg,))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `createClient` (0xd5a24481) function
        pub fn create_client(
            &self,
            msg: MsgCreateClient,
        ) -> ::ethers::contract::builders::ContractCall<M, ::std::string::String> {
            self.0
                .method_hash([213, 162, 68, 129], (msg,))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getChannel` (0x3000217a) function
        pub fn get_channel(
            &self,
            port_id: ::std::string::String,
            channel_id: ::std::string::String,
        ) -> ::ethers::contract::builders::ContractCall<M, (IbcCoreChannelV1ChannelData, bool)>
        {
            self.0
                .method_hash([48, 0, 33, 122], (port_id, channel_id))
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
        ///Calls the contract's `getConnection` (0x27711a69) function
        pub fn get_connection(
            &self,
            connection_id: ::std::string::String,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            (IbcCoreConnectionV1ConnectionEndData, bool),
        > {
            self.0
                .method_hash([39, 113, 26, 105], connection_id)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getConnectionSerialize` (0xc5c1e8fd) function
        pub fn get_connection_serialize(
            &self,
            connection_id: ::std::string::String,
        ) -> ::ethers::contract::builders::ContractCall<M, (::ethers::core::types::Bytes, bool)>
        {
            self.0
                .method_hash([197, 193, 232, 253], connection_id)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getConsensusState` (0x6cf44bf4) function
        pub fn get_consensus_state(
            &self,
            client_id: ::std::string::String,
            height: IbcCoreClientV1HeightData,
        ) -> ::ethers::contract::builders::ContractCall<M, (::ethers::core::types::Bytes, bool)>
        {
            self.0
                .method_hash([108, 244, 75, 244], (client_id, height))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getExpectedTimePerBlock` (0xec75d829) function
        pub fn get_expected_time_per_block(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, u64> {
            self.0
                .method_hash([236, 117, 216, 41], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getHashedPacketAcknowledgementCommitment` (0x5be164ee) function
        pub fn get_hashed_packet_acknowledgement_commitment(
            &self,
            port_id: ::std::string::String,
            channel_id: ::std::string::String,
            sequence: u64,
        ) -> ::ethers::contract::builders::ContractCall<M, ([u8; 32], bool)> {
            self.0
                .method_hash([91, 225, 100, 238], (port_id, channel_id, sequence))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getHashedPacketCommitment` (0x23402a33) function
        pub fn get_hashed_packet_commitment(
            &self,
            port_id: ::std::string::String,
            channel_id: ::std::string::String,
            sequence: u64,
        ) -> ::ethers::contract::builders::ContractCall<M, ([u8; 32], bool)> {
            self.0
                .method_hash([35, 64, 42, 51], (port_id, channel_id, sequence))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getNextSequenceSend` (0x582418b6) function
        pub fn get_next_sequence_send(
            &self,
            port_id: ::std::string::String,
            channel_id: ::std::string::String,
        ) -> ::ethers::contract::builders::ContractCall<M, u64> {
            self.0
                .method_hash([88, 36, 24, 182], (port_id, channel_id))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `hasPacketReceipt` (0x5a9afac3) function
        pub fn has_packet_receipt(
            &self,
            port_id: ::std::string::String,
            channel_id: ::std::string::String,
            sequence: u64,
        ) -> ::ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([90, 154, 250, 195], (port_id, channel_id, sequence))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `portCapabilityPath` (0x2570dae0) function
        pub fn port_capability_path(
            &self,
            port_id: ::std::string::String,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::Bytes> {
            self.0
                .method_hash([37, 112, 218, 224], port_id)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `recvPacket` (0x236ebd70) function
        pub fn recv_packet(
            &self,
            msg: MsgPacketRecv,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([35, 110, 189, 112], (msg,))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `registerClient` (0x18c19870) function
        pub fn register_client(
            &self,
            client_type: ::std::string::String,
            client: ::ethers::core::types::Address,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([24, 193, 152, 112], (client_type, client))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `sendPacket` (0xae4cd201) function
        pub fn send_packet(
            &self,
            source_port: ::std::string::String,
            source_channel: ::std::string::String,
            timeout_height: IbcCoreClientV1HeightData,
            timeout_timestamp: u64,
            data: ::ethers::core::types::Bytes,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash(
                    [174, 76, 210, 1],
                    (
                        source_port,
                        source_channel,
                        timeout_height,
                        timeout_timestamp,
                        data,
                    ),
                )
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `setExpectedTimePerBlock` (0x27184c13) function
        pub fn set_expected_time_per_block(
            &self,
            expected_time_per_block: u64,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([39, 24, 76, 19], expected_time_per_block)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `updateClient` (0xda6cea55) function
        pub fn update_client(
            &self,
            msg: MsgUpdateClient,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([218, 108, 234, 85], (msg,))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `writeAcknowledgement` (0xb56e79de) function
        pub fn write_acknowledgement(
            &self,
            destination_port_id: ::std::string::String,
            destination_channel: ::std::string::String,
            sequence: u64,
            acknowledgement: ::ethers::core::types::Bytes,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash(
                    [181, 110, 121, 222],
                    (
                        destination_port_id,
                        destination_channel,
                        sequence,
                        acknowledgement,
                    ),
                )
                .expect("method not found (this should never happen)")
        }
        ///Gets the contract's `AcknowledgePacket` event
        pub fn acknowledge_packet_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<::std::sync::Arc<M>, M, AcknowledgePacketFilter>
        {
            self.0.event()
        }
        ///Gets the contract's `GeneratedChannelIdentifier` event
        pub fn generated_channel_identifier_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            GeneratedChannelIdentifierFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `GeneratedClientIdentifier` event
        pub fn generated_client_identifier_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            GeneratedClientIdentifierFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `GeneratedConnectionIdentifier` event
        pub fn generated_connection_identifier_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            GeneratedConnectionIdentifierFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `RecvPacket` event
        pub fn recv_packet_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<::std::sync::Arc<M>, M, RecvPacketFilter> {
            self.0.event()
        }
        ///Gets the contract's `SendPacket` event
        pub fn send_packet_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<::std::sync::Arc<M>, M, SendPacketFilter> {
            self.0.event()
        }
        ///Gets the contract's `WriteAcknowledgement` event
        pub fn write_acknowledgement_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<::std::sync::Arc<M>, M, WriteAcknowledgementFilter>
        {
            self.0.event()
        }
        /// Returns an `Event` builder for all the events of this contract.
        pub fn events(
            &self,
        ) -> ::ethers::contract::builders::Event<::std::sync::Arc<M>, M, IBCHandlerEvents> {
            self.0
                .event_with_filter(::core::default::Default::default())
        }
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>> for IBCHandler<M> {
        fn from(contract: ::ethers::contract::Contract<M>) -> Self {
            Self::new(contract.address(), contract.client())
        }
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[ethevent(
        name = "AcknowledgePacket",
        abi = "AcknowledgePacket((uint64,string,string,string,string,bytes,(uint64,uint64),uint64),bytes)"
    )]
    pub struct AcknowledgePacketFilter {
        pub packet: IbcCoreChannelV1PacketData,
        pub acknowledgement: ::ethers::core::types::Bytes,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[ethevent(
        name = "GeneratedChannelIdentifier",
        abi = "GeneratedChannelIdentifier(string)"
    )]
    pub struct GeneratedChannelIdentifierFilter(pub ::std::string::String);
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[ethevent(
        name = "GeneratedClientIdentifier",
        abi = "GeneratedClientIdentifier(string)"
    )]
    pub struct GeneratedClientIdentifierFilter(pub ::std::string::String);
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[ethevent(
        name = "GeneratedConnectionIdentifier",
        abi = "GeneratedConnectionIdentifier(string)"
    )]
    pub struct GeneratedConnectionIdentifierFilter(pub ::std::string::String);
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[ethevent(
        name = "RecvPacket",
        abi = "RecvPacket((uint64,string,string,string,string,bytes,(uint64,uint64),uint64))"
    )]
    pub struct RecvPacketFilter {
        pub packet: IbcCoreChannelV1PacketData,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[ethevent(
        name = "SendPacket",
        abi = "SendPacket(uint64,string,string,(uint64,uint64),uint64,bytes)"
    )]
    pub struct SendPacketFilter {
        pub sequence: u64,
        pub source_port: ::std::string::String,
        pub source_channel: ::std::string::String,
        pub timeout_height: IbcCoreClientV1HeightData,
        pub timeout_timestamp: u64,
        pub data: ::ethers::core::types::Bytes,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[ethevent(
        name = "WriteAcknowledgement",
        abi = "WriteAcknowledgement(string,string,uint64,bytes)"
    )]
    pub struct WriteAcknowledgementFilter {
        pub destination_port_id: ::std::string::String,
        pub destination_channel: ::std::string::String,
        pub sequence: u64,
        pub acknowledgement: ::ethers::core::types::Bytes,
    }
    ///Container type for all of the contract's events
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum IBCHandlerEvents {
        AcknowledgePacketFilter(AcknowledgePacketFilter),
        GeneratedChannelIdentifierFilter(GeneratedChannelIdentifierFilter),
        GeneratedClientIdentifierFilter(GeneratedClientIdentifierFilter),
        GeneratedConnectionIdentifierFilter(GeneratedConnectionIdentifierFilter),
        RecvPacketFilter(RecvPacketFilter),
        SendPacketFilter(SendPacketFilter),
        WriteAcknowledgementFilter(WriteAcknowledgementFilter),
    }
    impl ::ethers::contract::EthLogDecode for IBCHandlerEvents {
        fn decode_log(
            log: &::ethers::core::abi::RawLog,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::Error> {
            if let Ok(decoded) = AcknowledgePacketFilter::decode_log(log) {
                return Ok(IBCHandlerEvents::AcknowledgePacketFilter(decoded));
            }
            if let Ok(decoded) = GeneratedChannelIdentifierFilter::decode_log(log) {
                return Ok(IBCHandlerEvents::GeneratedChannelIdentifierFilter(decoded));
            }
            if let Ok(decoded) = GeneratedClientIdentifierFilter::decode_log(log) {
                return Ok(IBCHandlerEvents::GeneratedClientIdentifierFilter(decoded));
            }
            if let Ok(decoded) = GeneratedConnectionIdentifierFilter::decode_log(log) {
                return Ok(IBCHandlerEvents::GeneratedConnectionIdentifierFilter(
                    decoded,
                ));
            }
            if let Ok(decoded) = RecvPacketFilter::decode_log(log) {
                return Ok(IBCHandlerEvents::RecvPacketFilter(decoded));
            }
            if let Ok(decoded) = SendPacketFilter::decode_log(log) {
                return Ok(IBCHandlerEvents::SendPacketFilter(decoded));
            }
            if let Ok(decoded) = WriteAcknowledgementFilter::decode_log(log) {
                return Ok(IBCHandlerEvents::WriteAcknowledgementFilter(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData)
        }
    }
    impl ::core::fmt::Display for IBCHandlerEvents {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::AcknowledgePacketFilter(element) => ::core::fmt::Display::fmt(element, f),
                Self::GeneratedChannelIdentifierFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::GeneratedClientIdentifierFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::GeneratedConnectionIdentifierFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::RecvPacketFilter(element) => ::core::fmt::Display::fmt(element, f),
                Self::SendPacketFilter(element) => ::core::fmt::Display::fmt(element, f),
                Self::WriteAcknowledgementFilter(element) => ::core::fmt::Display::fmt(element, f),
            }
        }
    }
    impl ::core::convert::From<AcknowledgePacketFilter> for IBCHandlerEvents {
        fn from(value: AcknowledgePacketFilter) -> Self {
            Self::AcknowledgePacketFilter(value)
        }
    }
    impl ::core::convert::From<GeneratedChannelIdentifierFilter> for IBCHandlerEvents {
        fn from(value: GeneratedChannelIdentifierFilter) -> Self {
            Self::GeneratedChannelIdentifierFilter(value)
        }
    }
    impl ::core::convert::From<GeneratedClientIdentifierFilter> for IBCHandlerEvents {
        fn from(value: GeneratedClientIdentifierFilter) -> Self {
            Self::GeneratedClientIdentifierFilter(value)
        }
    }
    impl ::core::convert::From<GeneratedConnectionIdentifierFilter> for IBCHandlerEvents {
        fn from(value: GeneratedConnectionIdentifierFilter) -> Self {
            Self::GeneratedConnectionIdentifierFilter(value)
        }
    }
    impl ::core::convert::From<RecvPacketFilter> for IBCHandlerEvents {
        fn from(value: RecvPacketFilter) -> Self {
            Self::RecvPacketFilter(value)
        }
    }
    impl ::core::convert::From<SendPacketFilter> for IBCHandlerEvents {
        fn from(value: SendPacketFilter) -> Self {
            Self::SendPacketFilter(value)
        }
    }
    impl ::core::convert::From<WriteAcknowledgementFilter> for IBCHandlerEvents {
        fn from(value: WriteAcknowledgementFilter) -> Self {
            Self::WriteAcknowledgementFilter(value)
        }
    }
    ///Container type for all input parameters for the `acknowledgePacket` function with signature `acknowledgePacket(((uint64,string,string,string,string,bytes,(uint64,uint64),uint64),bytes,bytes,(uint64,uint64)))` and selector `0x59f37976`
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
        name = "acknowledgePacket",
        abi = "acknowledgePacket(((uint64,string,string,string,string,bytes,(uint64,uint64),uint64),bytes,bytes,(uint64,uint64)))"
    )]
    pub struct AcknowledgePacketCall {
        pub msg: MsgPacketAcknowledgement,
    }
    ///Container type for all input parameters for the `bindPort` function with signature `bindPort(string,address)` and selector `0x117e886a`
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
    #[ethcall(name = "bindPort", abi = "bindPort(string,address)")]
    pub struct BindPortCall {
        pub port_id: ::std::string::String,
        pub module_address: ::ethers::core::types::Address,
    }
    ///Container type for all input parameters for the `channelCapabilityPath` function with signature `channelCapabilityPath(string,string)` and selector `0x3bc3339f`
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
        name = "channelCapabilityPath",
        abi = "channelCapabilityPath(string,string)"
    )]
    pub struct ChannelCapabilityPathCall {
        pub port_id: ::std::string::String,
        pub channel_id: ::std::string::String,
    }
    ///Container type for all input parameters for the `channelCloseConfirm` function with signature `channelCloseConfirm((string,string,bytes,(uint64,uint64)))` and selector `0x25cbc3a6`
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
        name = "channelCloseConfirm",
        abi = "channelCloseConfirm((string,string,bytes,(uint64,uint64)))"
    )]
    pub struct ChannelCloseConfirmCall {
        pub msg: MsgChannelCloseConfirm,
    }
    ///Container type for all input parameters for the `channelCloseInit` function with signature `channelCloseInit((string,string))` and selector `0xa06cb3a2`
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
    #[ethcall(name = "channelCloseInit", abi = "channelCloseInit((string,string))")]
    pub struct ChannelCloseInitCall {
        pub msg: MsgChannelCloseInit,
    }
    ///Container type for all input parameters for the `channelOpenAck` function with signature `channelOpenAck((string,string,string,string,bytes,(uint64,uint64)))` and selector `0x256c4199`
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
        name = "channelOpenAck",
        abi = "channelOpenAck((string,string,string,string,bytes,(uint64,uint64)))"
    )]
    pub struct ChannelOpenAckCall {
        pub msg: MsgChannelOpenAck,
    }
    ///Container type for all input parameters for the `channelOpenConfirm` function with signature `channelOpenConfirm((string,string,bytes,(uint64,uint64)))` and selector `0x5bd51b62`
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
        name = "channelOpenConfirm",
        abi = "channelOpenConfirm((string,string,bytes,(uint64,uint64)))"
    )]
    pub struct ChannelOpenConfirmCall {
        pub msg: MsgChannelOpenConfirm,
    }
    ///Container type for all input parameters for the `channelOpenInit` function with signature `channelOpenInit((string,(uint8,uint8,(string,string),string[],string)))` and selector `0xdd3469fc`
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
        name = "channelOpenInit",
        abi = "channelOpenInit((string,(uint8,uint8,(string,string),string[],string)))"
    )]
    pub struct ChannelOpenInitCall {
        pub msg: MsgChannelOpenInit,
    }
    ///Container type for all input parameters for the `channelOpenTry` function with signature `channelOpenTry((string,(uint8,uint8,(string,string),string[],string),string,bytes,(uint64,uint64)))` and selector `0x11b88a15`
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
        name = "channelOpenTry",
        abi = "channelOpenTry((string,(uint8,uint8,(string,string),string[],string),string,bytes,(uint64,uint64)))"
    )]
    pub struct ChannelOpenTryCall {
        pub msg: MsgChannelOpenTry,
    }
    ///Container type for all input parameters for the `connectionOpenAck` function with signature `connectionOpenAck((string,bytes,(string,string[]),string,bytes,bytes,bytes,(uint64,uint64),(uint64,uint64)))` and selector `0xb531861f`
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
        name = "connectionOpenAck",
        abi = "connectionOpenAck((string,bytes,(string,string[]),string,bytes,bytes,bytes,(uint64,uint64),(uint64,uint64)))"
    )]
    pub struct ConnectionOpenAckCall {
        pub msg: MsgConnectionOpenAck,
    }
    ///Container type for all input parameters for the `connectionOpenConfirm` function with signature `connectionOpenConfirm((string,bytes,(uint64,uint64)))` and selector `0x6a728f2c`
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
        name = "connectionOpenConfirm",
        abi = "connectionOpenConfirm((string,bytes,(uint64,uint64)))"
    )]
    pub struct ConnectionOpenConfirmCall {
        pub msg: MsgConnectionOpenConfirm,
    }
    ///Container type for all input parameters for the `connectionOpenInit` function with signature `connectionOpenInit((string,(string,string,(bytes)),uint64))` and selector `0x01c6400f`
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
        name = "connectionOpenInit",
        abi = "connectionOpenInit((string,(string,string,(bytes)),uint64))"
    )]
    pub struct ConnectionOpenInitCall {
        pub msg: MsgConnectionOpenInit,
    }
    ///Container type for all input parameters for the `connectionOpenTry` function with signature `connectionOpenTry(((string,string,(bytes)),uint64,string,bytes,(string,string[])[],bytes,bytes,bytes,(uint64,uint64),(uint64,uint64)))` and selector `0x04f68e5c`
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
        name = "connectionOpenTry",
        abi = "connectionOpenTry(((string,string,(bytes)),uint64,string,bytes,(string,string[])[],bytes,bytes,bytes,(uint64,uint64),(uint64,uint64)))"
    )]
    pub struct ConnectionOpenTryCall {
        pub msg: MsgConnectionOpenTry,
    }
    ///Container type for all input parameters for the `createClient` function with signature `createClient((string,bytes,bytes))` and selector `0xd5a24481`
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
    #[ethcall(name = "createClient", abi = "createClient((string,bytes,bytes))")]
    pub struct CreateClientCall {
        pub msg: MsgCreateClient,
    }
    ///Container type for all input parameters for the `getChannel` function with signature `getChannel(string,string)` and selector `0x3000217a`
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
    #[ethcall(name = "getChannel", abi = "getChannel(string,string)")]
    pub struct GetChannelCall {
        pub port_id: ::std::string::String,
        pub channel_id: ::std::string::String,
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
    ///Container type for all input parameters for the `getConnection` function with signature `getConnection(string)` and selector `0x27711a69`
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
    #[ethcall(name = "getConnection", abi = "getConnection(string)")]
    pub struct GetConnectionCall {
        pub connection_id: ::std::string::String,
    }
    ///Container type for all input parameters for the `getConnectionSerialize` function with signature `getConnectionSerialize(string)` and selector `0xc5c1e8fd`
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
        name = "getConnectionSerialize",
        abi = "getConnectionSerialize(string)"
    )]
    pub struct GetConnectionSerializeCall {
        pub connection_id: ::std::string::String,
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
        pub height: IbcCoreClientV1HeightData,
    }
    ///Container type for all input parameters for the `getExpectedTimePerBlock` function with signature `getExpectedTimePerBlock()` and selector `0xec75d829`
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
    #[ethcall(name = "getExpectedTimePerBlock", abi = "getExpectedTimePerBlock()")]
    pub struct GetExpectedTimePerBlockCall;
    ///Container type for all input parameters for the `getHashedPacketAcknowledgementCommitment` function with signature `getHashedPacketAcknowledgementCommitment(string,string,uint64)` and selector `0x5be164ee`
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
        name = "getHashedPacketAcknowledgementCommitment",
        abi = "getHashedPacketAcknowledgementCommitment(string,string,uint64)"
    )]
    pub struct GetHashedPacketAcknowledgementCommitmentCall {
        pub port_id: ::std::string::String,
        pub channel_id: ::std::string::String,
        pub sequence: u64,
    }
    ///Container type for all input parameters for the `getHashedPacketCommitment` function with signature `getHashedPacketCommitment(string,string,uint64)` and selector `0x23402a33`
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
        name = "getHashedPacketCommitment",
        abi = "getHashedPacketCommitment(string,string,uint64)"
    )]
    pub struct GetHashedPacketCommitmentCall {
        pub port_id: ::std::string::String,
        pub channel_id: ::std::string::String,
        pub sequence: u64,
    }
    ///Container type for all input parameters for the `getNextSequenceSend` function with signature `getNextSequenceSend(string,string)` and selector `0x582418b6`
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
        name = "getNextSequenceSend",
        abi = "getNextSequenceSend(string,string)"
    )]
    pub struct GetNextSequenceSendCall {
        pub port_id: ::std::string::String,
        pub channel_id: ::std::string::String,
    }
    ///Container type for all input parameters for the `hasPacketReceipt` function with signature `hasPacketReceipt(string,string,uint64)` and selector `0x5a9afac3`
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
        name = "hasPacketReceipt",
        abi = "hasPacketReceipt(string,string,uint64)"
    )]
    pub struct HasPacketReceiptCall {
        pub port_id: ::std::string::String,
        pub channel_id: ::std::string::String,
        pub sequence: u64,
    }
    ///Container type for all input parameters for the `portCapabilityPath` function with signature `portCapabilityPath(string)` and selector `0x2570dae0`
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
    #[ethcall(name = "portCapabilityPath", abi = "portCapabilityPath(string)")]
    pub struct PortCapabilityPathCall {
        pub port_id: ::std::string::String,
    }
    ///Container type for all input parameters for the `recvPacket` function with signature `recvPacket(((uint64,string,string,string,string,bytes,(uint64,uint64),uint64),bytes,(uint64,uint64)))` and selector `0x236ebd70`
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
        name = "recvPacket",
        abi = "recvPacket(((uint64,string,string,string,string,bytes,(uint64,uint64),uint64),bytes,(uint64,uint64)))"
    )]
    pub struct RecvPacketCall {
        pub msg: MsgPacketRecv,
    }
    ///Container type for all input parameters for the `registerClient` function with signature `registerClient(string,address)` and selector `0x18c19870`
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
    #[ethcall(name = "registerClient", abi = "registerClient(string,address)")]
    pub struct RegisterClientCall {
        pub client_type: ::std::string::String,
        pub client: ::ethers::core::types::Address,
    }
    ///Container type for all input parameters for the `sendPacket` function with signature `sendPacket(string,string,(uint64,uint64),uint64,bytes)` and selector `0xae4cd201`
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
        name = "sendPacket",
        abi = "sendPacket(string,string,(uint64,uint64),uint64,bytes)"
    )]
    pub struct SendPacketCall {
        pub source_port: ::std::string::String,
        pub source_channel: ::std::string::String,
        pub timeout_height: IbcCoreClientV1HeightData,
        pub timeout_timestamp: u64,
        pub data: ::ethers::core::types::Bytes,
    }
    ///Container type for all input parameters for the `setExpectedTimePerBlock` function with signature `setExpectedTimePerBlock(uint64)` and selector `0x27184c13`
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
        name = "setExpectedTimePerBlock",
        abi = "setExpectedTimePerBlock(uint64)"
    )]
    pub struct SetExpectedTimePerBlockCall {
        pub expected_time_per_block: u64,
    }
    ///Container type for all input parameters for the `updateClient` function with signature `updateClient((string,bytes))` and selector `0xda6cea55`
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
    #[ethcall(name = "updateClient", abi = "updateClient((string,bytes))")]
    pub struct UpdateClientCall {
        pub msg: MsgUpdateClient,
    }
    ///Container type for all input parameters for the `writeAcknowledgement` function with signature `writeAcknowledgement(string,string,uint64,bytes)` and selector `0xb56e79de`
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
        name = "writeAcknowledgement",
        abi = "writeAcknowledgement(string,string,uint64,bytes)"
    )]
    pub struct WriteAcknowledgementCall {
        pub destination_port_id: ::std::string::String,
        pub destination_channel: ::std::string::String,
        pub sequence: u64,
        pub acknowledgement: ::ethers::core::types::Bytes,
    }
    ///Container type for all of the contract's call
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum IBCHandlerCalls {
        AcknowledgePacket(AcknowledgePacketCall),
        BindPort(BindPortCall),
        ChannelCapabilityPath(ChannelCapabilityPathCall),
        ChannelCloseConfirm(ChannelCloseConfirmCall),
        ChannelCloseInit(ChannelCloseInitCall),
        ChannelOpenAck(ChannelOpenAckCall),
        ChannelOpenConfirm(ChannelOpenConfirmCall),
        ChannelOpenInit(ChannelOpenInitCall),
        ChannelOpenTry(ChannelOpenTryCall),
        ConnectionOpenAck(ConnectionOpenAckCall),
        ConnectionOpenConfirm(ConnectionOpenConfirmCall),
        ConnectionOpenInit(ConnectionOpenInitCall),
        ConnectionOpenTry(ConnectionOpenTryCall),
        CreateClient(CreateClientCall),
        GetChannel(GetChannelCall),
        GetClientState(GetClientStateCall),
        GetConnection(GetConnectionCall),
        GetConnectionSerialize(GetConnectionSerializeCall),
        GetConsensusState(GetConsensusStateCall),
        GetExpectedTimePerBlock(GetExpectedTimePerBlockCall),
        GetHashedPacketAcknowledgementCommitment(GetHashedPacketAcknowledgementCommitmentCall),
        GetHashedPacketCommitment(GetHashedPacketCommitmentCall),
        GetNextSequenceSend(GetNextSequenceSendCall),
        HasPacketReceipt(HasPacketReceiptCall),
        PortCapabilityPath(PortCapabilityPathCall),
        RecvPacket(RecvPacketCall),
        RegisterClient(RegisterClientCall),
        SendPacket(SendPacketCall),
        SetExpectedTimePerBlock(SetExpectedTimePerBlockCall),
        UpdateClient(UpdateClientCall),
        WriteAcknowledgement(WriteAcknowledgementCall),
    }
    impl ::ethers::core::abi::AbiDecode for IBCHandlerCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded) =
                <AcknowledgePacketCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::AcknowledgePacket(decoded));
            }
            if let Ok(decoded) = <BindPortCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::BindPort(decoded));
            }
            if let Ok(decoded) =
                <ChannelCapabilityPathCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::ChannelCapabilityPath(decoded));
            }
            if let Ok(decoded) =
                <ChannelCloseConfirmCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::ChannelCloseConfirm(decoded));
            }
            if let Ok(decoded) =
                <ChannelCloseInitCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::ChannelCloseInit(decoded));
            }
            if let Ok(decoded) =
                <ChannelOpenAckCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::ChannelOpenAck(decoded));
            }
            if let Ok(decoded) =
                <ChannelOpenConfirmCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::ChannelOpenConfirm(decoded));
            }
            if let Ok(decoded) =
                <ChannelOpenInitCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::ChannelOpenInit(decoded));
            }
            if let Ok(decoded) =
                <ChannelOpenTryCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::ChannelOpenTry(decoded));
            }
            if let Ok(decoded) =
                <ConnectionOpenAckCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::ConnectionOpenAck(decoded));
            }
            if let Ok(decoded) =
                <ConnectionOpenConfirmCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::ConnectionOpenConfirm(decoded));
            }
            if let Ok(decoded) =
                <ConnectionOpenInitCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::ConnectionOpenInit(decoded));
            }
            if let Ok(decoded) =
                <ConnectionOpenTryCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::ConnectionOpenTry(decoded));
            }
            if let Ok(decoded) = <CreateClientCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::CreateClient(decoded));
            }
            if let Ok(decoded) = <GetChannelCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::GetChannel(decoded));
            }
            if let Ok(decoded) =
                <GetClientStateCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::GetClientState(decoded));
            }
            if let Ok(decoded) = <GetConnectionCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::GetConnection(decoded));
            }
            if let Ok(decoded) =
                <GetConnectionSerializeCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::GetConnectionSerialize(decoded));
            }
            if let Ok(decoded) =
                <GetConsensusStateCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::GetConsensusState(decoded));
            }
            if let Ok(decoded) =
                <GetExpectedTimePerBlockCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::GetExpectedTimePerBlock(decoded));
            }
            if let Ok(decoded)
                = <GetHashedPacketAcknowledgementCommitmentCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::GetHashedPacketAcknowledgementCommitment(decoded));
            }
            if let Ok(decoded) =
                <GetHashedPacketCommitmentCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::GetHashedPacketCommitment(decoded));
            }
            if let Ok(decoded) =
                <GetNextSequenceSendCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::GetNextSequenceSend(decoded));
            }
            if let Ok(decoded) =
                <HasPacketReceiptCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::HasPacketReceipt(decoded));
            }
            if let Ok(decoded) =
                <PortCapabilityPathCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::PortCapabilityPath(decoded));
            }
            if let Ok(decoded) = <RecvPacketCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::RecvPacket(decoded));
            }
            if let Ok(decoded) =
                <RegisterClientCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::RegisterClient(decoded));
            }
            if let Ok(decoded) = <SendPacketCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::SendPacket(decoded));
            }
            if let Ok(decoded) =
                <SetExpectedTimePerBlockCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::SetExpectedTimePerBlock(decoded));
            }
            if let Ok(decoded) = <UpdateClientCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::UpdateClient(decoded));
            }
            if let Ok(decoded) =
                <WriteAcknowledgementCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::WriteAcknowledgement(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for IBCHandlerCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                Self::AcknowledgePacket(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::BindPort(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::ChannelCapabilityPath(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ChannelCloseConfirm(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ChannelCloseInit(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::ChannelOpenAck(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::ChannelOpenConfirm(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ChannelOpenInit(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::ChannelOpenTry(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::ConnectionOpenAck(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::ConnectionOpenConfirm(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ConnectionOpenInit(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ConnectionOpenTry(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::CreateClient(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::GetChannel(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::GetClientState(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::GetConnection(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::GetConnectionSerialize(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetConsensusState(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::GetExpectedTimePerBlock(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetHashedPacketAcknowledgementCommitment(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetHashedPacketCommitment(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetNextSequenceSend(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::HasPacketReceipt(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::PortCapabilityPath(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RecvPacket(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::RegisterClient(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::SendPacket(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::SetExpectedTimePerBlock(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::UpdateClient(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::WriteAcknowledgement(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
            }
        }
    }
    impl ::core::fmt::Display for IBCHandlerCalls {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::AcknowledgePacket(element) => ::core::fmt::Display::fmt(element, f),
                Self::BindPort(element) => ::core::fmt::Display::fmt(element, f),
                Self::ChannelCapabilityPath(element) => ::core::fmt::Display::fmt(element, f),
                Self::ChannelCloseConfirm(element) => ::core::fmt::Display::fmt(element, f),
                Self::ChannelCloseInit(element) => ::core::fmt::Display::fmt(element, f),
                Self::ChannelOpenAck(element) => ::core::fmt::Display::fmt(element, f),
                Self::ChannelOpenConfirm(element) => ::core::fmt::Display::fmt(element, f),
                Self::ChannelOpenInit(element) => ::core::fmt::Display::fmt(element, f),
                Self::ChannelOpenTry(element) => ::core::fmt::Display::fmt(element, f),
                Self::ConnectionOpenAck(element) => ::core::fmt::Display::fmt(element, f),
                Self::ConnectionOpenConfirm(element) => ::core::fmt::Display::fmt(element, f),
                Self::ConnectionOpenInit(element) => ::core::fmt::Display::fmt(element, f),
                Self::ConnectionOpenTry(element) => ::core::fmt::Display::fmt(element, f),
                Self::CreateClient(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetChannel(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetClientState(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetConnection(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetConnectionSerialize(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetConsensusState(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetExpectedTimePerBlock(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetHashedPacketAcknowledgementCommitment(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::GetHashedPacketCommitment(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetNextSequenceSend(element) => ::core::fmt::Display::fmt(element, f),
                Self::HasPacketReceipt(element) => ::core::fmt::Display::fmt(element, f),
                Self::PortCapabilityPath(element) => ::core::fmt::Display::fmt(element, f),
                Self::RecvPacket(element) => ::core::fmt::Display::fmt(element, f),
                Self::RegisterClient(element) => ::core::fmt::Display::fmt(element, f),
                Self::SendPacket(element) => ::core::fmt::Display::fmt(element, f),
                Self::SetExpectedTimePerBlock(element) => ::core::fmt::Display::fmt(element, f),
                Self::UpdateClient(element) => ::core::fmt::Display::fmt(element, f),
                Self::WriteAcknowledgement(element) => ::core::fmt::Display::fmt(element, f),
            }
        }
    }
    impl ::core::convert::From<AcknowledgePacketCall> for IBCHandlerCalls {
        fn from(value: AcknowledgePacketCall) -> Self {
            Self::AcknowledgePacket(value)
        }
    }
    impl ::core::convert::From<BindPortCall> for IBCHandlerCalls {
        fn from(value: BindPortCall) -> Self {
            Self::BindPort(value)
        }
    }
    impl ::core::convert::From<ChannelCapabilityPathCall> for IBCHandlerCalls {
        fn from(value: ChannelCapabilityPathCall) -> Self {
            Self::ChannelCapabilityPath(value)
        }
    }
    impl ::core::convert::From<ChannelCloseConfirmCall> for IBCHandlerCalls {
        fn from(value: ChannelCloseConfirmCall) -> Self {
            Self::ChannelCloseConfirm(value)
        }
    }
    impl ::core::convert::From<ChannelCloseInitCall> for IBCHandlerCalls {
        fn from(value: ChannelCloseInitCall) -> Self {
            Self::ChannelCloseInit(value)
        }
    }
    impl ::core::convert::From<ChannelOpenAckCall> for IBCHandlerCalls {
        fn from(value: ChannelOpenAckCall) -> Self {
            Self::ChannelOpenAck(value)
        }
    }
    impl ::core::convert::From<ChannelOpenConfirmCall> for IBCHandlerCalls {
        fn from(value: ChannelOpenConfirmCall) -> Self {
            Self::ChannelOpenConfirm(value)
        }
    }
    impl ::core::convert::From<ChannelOpenInitCall> for IBCHandlerCalls {
        fn from(value: ChannelOpenInitCall) -> Self {
            Self::ChannelOpenInit(value)
        }
    }
    impl ::core::convert::From<ChannelOpenTryCall> for IBCHandlerCalls {
        fn from(value: ChannelOpenTryCall) -> Self {
            Self::ChannelOpenTry(value)
        }
    }
    impl ::core::convert::From<ConnectionOpenAckCall> for IBCHandlerCalls {
        fn from(value: ConnectionOpenAckCall) -> Self {
            Self::ConnectionOpenAck(value)
        }
    }
    impl ::core::convert::From<ConnectionOpenConfirmCall> for IBCHandlerCalls {
        fn from(value: ConnectionOpenConfirmCall) -> Self {
            Self::ConnectionOpenConfirm(value)
        }
    }
    impl ::core::convert::From<ConnectionOpenInitCall> for IBCHandlerCalls {
        fn from(value: ConnectionOpenInitCall) -> Self {
            Self::ConnectionOpenInit(value)
        }
    }
    impl ::core::convert::From<ConnectionOpenTryCall> for IBCHandlerCalls {
        fn from(value: ConnectionOpenTryCall) -> Self {
            Self::ConnectionOpenTry(value)
        }
    }
    impl ::core::convert::From<CreateClientCall> for IBCHandlerCalls {
        fn from(value: CreateClientCall) -> Self {
            Self::CreateClient(value)
        }
    }
    impl ::core::convert::From<GetChannelCall> for IBCHandlerCalls {
        fn from(value: GetChannelCall) -> Self {
            Self::GetChannel(value)
        }
    }
    impl ::core::convert::From<GetClientStateCall> for IBCHandlerCalls {
        fn from(value: GetClientStateCall) -> Self {
            Self::GetClientState(value)
        }
    }
    impl ::core::convert::From<GetConnectionCall> for IBCHandlerCalls {
        fn from(value: GetConnectionCall) -> Self {
            Self::GetConnection(value)
        }
    }
    impl ::core::convert::From<GetConnectionSerializeCall> for IBCHandlerCalls {
        fn from(value: GetConnectionSerializeCall) -> Self {
            Self::GetConnectionSerialize(value)
        }
    }
    impl ::core::convert::From<GetConsensusStateCall> for IBCHandlerCalls {
        fn from(value: GetConsensusStateCall) -> Self {
            Self::GetConsensusState(value)
        }
    }
    impl ::core::convert::From<GetExpectedTimePerBlockCall> for IBCHandlerCalls {
        fn from(value: GetExpectedTimePerBlockCall) -> Self {
            Self::GetExpectedTimePerBlock(value)
        }
    }
    impl ::core::convert::From<GetHashedPacketAcknowledgementCommitmentCall> for IBCHandlerCalls {
        fn from(value: GetHashedPacketAcknowledgementCommitmentCall) -> Self {
            Self::GetHashedPacketAcknowledgementCommitment(value)
        }
    }
    impl ::core::convert::From<GetHashedPacketCommitmentCall> for IBCHandlerCalls {
        fn from(value: GetHashedPacketCommitmentCall) -> Self {
            Self::GetHashedPacketCommitment(value)
        }
    }
    impl ::core::convert::From<GetNextSequenceSendCall> for IBCHandlerCalls {
        fn from(value: GetNextSequenceSendCall) -> Self {
            Self::GetNextSequenceSend(value)
        }
    }
    impl ::core::convert::From<HasPacketReceiptCall> for IBCHandlerCalls {
        fn from(value: HasPacketReceiptCall) -> Self {
            Self::HasPacketReceipt(value)
        }
    }
    impl ::core::convert::From<PortCapabilityPathCall> for IBCHandlerCalls {
        fn from(value: PortCapabilityPathCall) -> Self {
            Self::PortCapabilityPath(value)
        }
    }
    impl ::core::convert::From<RecvPacketCall> for IBCHandlerCalls {
        fn from(value: RecvPacketCall) -> Self {
            Self::RecvPacket(value)
        }
    }
    impl ::core::convert::From<RegisterClientCall> for IBCHandlerCalls {
        fn from(value: RegisterClientCall) -> Self {
            Self::RegisterClient(value)
        }
    }
    impl ::core::convert::From<SendPacketCall> for IBCHandlerCalls {
        fn from(value: SendPacketCall) -> Self {
            Self::SendPacket(value)
        }
    }
    impl ::core::convert::From<SetExpectedTimePerBlockCall> for IBCHandlerCalls {
        fn from(value: SetExpectedTimePerBlockCall) -> Self {
            Self::SetExpectedTimePerBlock(value)
        }
    }
    impl ::core::convert::From<UpdateClientCall> for IBCHandlerCalls {
        fn from(value: UpdateClientCall) -> Self {
            Self::UpdateClient(value)
        }
    }
    impl ::core::convert::From<WriteAcknowledgementCall> for IBCHandlerCalls {
        fn from(value: WriteAcknowledgementCall) -> Self {
            Self::WriteAcknowledgement(value)
        }
    }
    ///Container type for all return fields from the `channelCapabilityPath` function with signature `channelCapabilityPath(string,string)` and selector `0x3bc3339f`
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
    pub struct ChannelCapabilityPathReturn(pub ::ethers::core::types::Bytes);
    ///Container type for all return fields from the `channelOpenInit` function with signature `channelOpenInit((string,(uint8,uint8,(string,string),string[],string)))` and selector `0xdd3469fc`
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
    pub struct ChannelOpenInitReturn {
        pub channel_id: ::std::string::String,
    }
    ///Container type for all return fields from the `channelOpenTry` function with signature `channelOpenTry((string,(uint8,uint8,(string,string),string[],string),string,bytes,(uint64,uint64)))` and selector `0x11b88a15`
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
    pub struct ChannelOpenTryReturn {
        pub channel_id: ::std::string::String,
    }
    ///Container type for all return fields from the `connectionOpenInit` function with signature `connectionOpenInit((string,(string,string,(bytes)),uint64))` and selector `0x01c6400f`
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
    pub struct ConnectionOpenInitReturn {
        pub connection_id: ::std::string::String,
    }
    ///Container type for all return fields from the `connectionOpenTry` function with signature `connectionOpenTry(((string,string,(bytes)),uint64,string,bytes,(string,string[])[],bytes,bytes,bytes,(uint64,uint64),(uint64,uint64)))` and selector `0x04f68e5c`
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
    pub struct ConnectionOpenTryReturn {
        pub connection_id: ::std::string::String,
    }
    ///Container type for all return fields from the `createClient` function with signature `createClient((string,bytes,bytes))` and selector `0xd5a24481`
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
        pub client_id: ::std::string::String,
    }
    ///Container type for all return fields from the `getChannel` function with signature `getChannel(string,string)` and selector `0x3000217a`
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
    pub struct GetChannelReturn(pub IbcCoreChannelV1ChannelData, pub bool);
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
    ///Container type for all return fields from the `getConnection` function with signature `getConnection(string)` and selector `0x27711a69`
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
    pub struct GetConnectionReturn(pub IbcCoreConnectionV1ConnectionEndData, pub bool);
    ///Container type for all return fields from the `getConnectionSerialize` function with signature `getConnectionSerialize(string)` and selector `0xc5c1e8fd`
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
    pub struct GetConnectionSerializeReturn(pub ::ethers::core::types::Bytes, pub bool);
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
    pub struct GetConsensusStateReturn {
        pub consensus_state_bytes: ::ethers::core::types::Bytes,
        pub p1: bool,
    }
    ///Container type for all return fields from the `getExpectedTimePerBlock` function with signature `getExpectedTimePerBlock()` and selector `0xec75d829`
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
    pub struct GetExpectedTimePerBlockReturn(pub u64);
    ///Container type for all return fields from the `getHashedPacketAcknowledgementCommitment` function with signature `getHashedPacketAcknowledgementCommitment(string,string,uint64)` and selector `0x5be164ee`
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
    pub struct GetHashedPacketAcknowledgementCommitmentReturn(pub [u8; 32], pub bool);
    ///Container type for all return fields from the `getHashedPacketCommitment` function with signature `getHashedPacketCommitment(string,string,uint64)` and selector `0x23402a33`
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
    pub struct GetHashedPacketCommitmentReturn(pub [u8; 32], pub bool);
    ///Container type for all return fields from the `getNextSequenceSend` function with signature `getNextSequenceSend(string,string)` and selector `0x582418b6`
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
    pub struct GetNextSequenceSendReturn(pub u64);
    ///Container type for all return fields from the `hasPacketReceipt` function with signature `hasPacketReceipt(string,string,uint64)` and selector `0x5a9afac3`
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
    pub struct HasPacketReceiptReturn(pub bool);
    ///Container type for all return fields from the `portCapabilityPath` function with signature `portCapabilityPath(string)` and selector `0x2570dae0`
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
    pub struct PortCapabilityPathReturn(pub ::ethers::core::types::Bytes);
    ///`MsgChannelCloseConfirm(string,string,bytes,(uint64,uint64))`
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
    pub struct MsgChannelCloseConfirm {
        pub port_id: ::std::string::String,
        pub channel_id: ::std::string::String,
        pub proof_init: ::ethers::core::types::Bytes,
        pub proof_height: IbcCoreClientV1HeightData,
    }
    ///`MsgChannelCloseInit(string,string)`
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
    pub struct MsgChannelCloseInit {
        pub port_id: ::std::string::String,
        pub channel_id: ::std::string::String,
    }
    ///`MsgChannelOpenAck(string,string,string,string,bytes,(uint64,uint64))`
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
    pub struct MsgChannelOpenAck {
        pub port_id: ::std::string::String,
        pub channel_id: ::std::string::String,
        pub counterparty_version: ::std::string::String,
        pub counterparty_channel_id: ::std::string::String,
        pub proof_try: ::ethers::core::types::Bytes,
        pub proof_height: IbcCoreClientV1HeightData,
    }
    ///`MsgChannelOpenConfirm(string,string,bytes,(uint64,uint64))`
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
    pub struct MsgChannelOpenConfirm {
        pub port_id: ::std::string::String,
        pub channel_id: ::std::string::String,
        pub proof_ack: ::ethers::core::types::Bytes,
        pub proof_height: IbcCoreClientV1HeightData,
    }
    ///`MsgChannelOpenInit(string,(uint8,uint8,(string,string),string[],string))`
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
    pub struct MsgChannelOpenInit {
        pub port_id: ::std::string::String,
        pub channel: IbcCoreChannelV1ChannelData,
    }
    ///`MsgChannelOpenTry(string,(uint8,uint8,(string,string),string[],string),string,bytes,(uint64,uint64))`
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
    pub struct MsgChannelOpenTry {
        pub port_id: ::std::string::String,
        pub channel: IbcCoreChannelV1ChannelData,
        pub counterparty_version: ::std::string::String,
        pub proof_init: ::ethers::core::types::Bytes,
        pub proof_height: IbcCoreClientV1HeightData,
    }
    ///`MsgConnectionOpenAck(string,bytes,(string,string[]),string,bytes,bytes,bytes,(uint64,uint64),(uint64,uint64))`
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
    pub struct MsgConnectionOpenAck {
        pub connection_id: ::std::string::String,
        pub client_state_bytes: ::ethers::core::types::Bytes,
        pub version: IbcCoreConnectionV1VersionData,
        pub counterparty_connection_id: ::std::string::String,
        pub proof_try: ::ethers::core::types::Bytes,
        pub proof_client: ::ethers::core::types::Bytes,
        pub proof_consensus: ::ethers::core::types::Bytes,
        pub proof_height: IbcCoreClientV1HeightData,
        pub consensus_height: IbcCoreClientV1HeightData,
    }
    ///`MsgConnectionOpenConfirm(string,bytes,(uint64,uint64))`
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
    pub struct MsgConnectionOpenConfirm {
        pub connection_id: ::std::string::String,
        pub proof_ack: ::ethers::core::types::Bytes,
        pub proof_height: IbcCoreClientV1HeightData,
    }
    ///`MsgConnectionOpenInit(string,(string,string,(bytes)),uint64)`
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
    pub struct MsgConnectionOpenInit {
        pub client_id: ::std::string::String,
        pub counterparty: IbcCoreConnectionV1CounterpartyData,
        pub delay_period: u64,
    }
    ///`MsgConnectionOpenTry((string,string,(bytes)),uint64,string,bytes,(string,string[])[],bytes,bytes,bytes,(uint64,uint64),(uint64,uint64))`
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
    pub struct MsgConnectionOpenTry {
        pub counterparty: IbcCoreConnectionV1CounterpartyData,
        pub delay_period: u64,
        pub client_id: ::std::string::String,
        pub client_state_bytes: ::ethers::core::types::Bytes,
        pub counterparty_versions: ::std::vec::Vec<IbcCoreConnectionV1VersionData>,
        pub proof_init: ::ethers::core::types::Bytes,
        pub proof_client: ::ethers::core::types::Bytes,
        pub proof_consensus: ::ethers::core::types::Bytes,
        pub proof_height: IbcCoreClientV1HeightData,
        pub consensus_height: IbcCoreClientV1HeightData,
    }
    ///`MsgCreateClient(string,bytes,bytes)`
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
    pub struct MsgCreateClient {
        pub client_type: ::std::string::String,
        pub client_state_bytes: ::ethers::core::types::Bytes,
        pub consensus_state_bytes: ::ethers::core::types::Bytes,
    }
    ///`MsgPacketAcknowledgement((uint64,string,string,string,string,bytes,(uint64,uint64),uint64),bytes,bytes,(uint64,uint64))`
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
    pub struct MsgPacketAcknowledgement {
        pub packet: IbcCoreChannelV1PacketData,
        pub acknowledgement: ::ethers::core::types::Bytes,
        pub proof: ::ethers::core::types::Bytes,
        pub proof_height: IbcCoreClientV1HeightData,
    }
    ///`MsgPacketRecv((uint64,string,string,string,string,bytes,(uint64,uint64),uint64),bytes,(uint64,uint64))`
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
    pub struct MsgPacketRecv {
        pub packet: IbcCoreChannelV1PacketData,
        pub proof: ::ethers::core::types::Bytes,
        pub proof_height: IbcCoreClientV1HeightData,
    }
    ///`MsgUpdateClient(string,bytes)`
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
    pub struct MsgUpdateClient {
        pub client_id: ::std::string::String,
        pub client_message: ::ethers::core::types::Bytes,
    }
    ///`IbcCoreChannelV1ChannelData(uint8,uint8,(string,string),string[],string)`
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
    pub struct IbcCoreChannelV1ChannelData {
        pub state: u8,
        pub ordering: u8,
        pub counterparty: IbcCoreChannelV1CounterpartyData,
        pub connection_hops: ::std::vec::Vec<::std::string::String>,
        pub version: ::std::string::String,
    }
    ///`IbcCoreChannelV1CounterpartyData(string,string)`
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
    pub struct IbcCoreChannelV1CounterpartyData {
        pub port_id: ::std::string::String,
        pub channel_id: ::std::string::String,
    }
    ///`IbcCoreChannelV1PacketData(uint64,string,string,string,string,bytes,(uint64,uint64),uint64)`
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
    pub struct IbcCoreChannelV1PacketData {
        pub sequence: u64,
        pub source_port: ::std::string::String,
        pub source_channel: ::std::string::String,
        pub destination_port: ::std::string::String,
        pub destination_channel: ::std::string::String,
        pub data: ::ethers::core::types::Bytes,
        pub timeout_height: IbcCoreClientV1HeightData,
        pub timeout_timestamp: u64,
    }
    ///`IbcCoreCommitmentV1MerklePrefixData(bytes)`
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
    pub struct IbcCoreCommitmentV1MerklePrefixData {
        pub key_prefix: ::ethers::core::types::Bytes,
    }
    ///`IbcCoreConnectionV1ConnectionEndData(string,(string,string[])[],uint8,(string,string,(bytes)),uint64)`
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
    pub struct IbcCoreConnectionV1ConnectionEndData {
        pub client_id: ::std::string::String,
        pub versions: ::std::vec::Vec<IbcCoreConnectionV1VersionData>,
        pub state: u8,
        pub counterparty: IbcCoreConnectionV1CounterpartyData,
        pub delay_period: u64,
    }
    ///`IbcCoreConnectionV1CounterpartyData(string,string,(bytes))`
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
    pub struct IbcCoreConnectionV1CounterpartyData {
        pub client_id: ::std::string::String,
        pub connection_id: ::std::string::String,
        pub prefix: IbcCoreCommitmentV1MerklePrefixData,
    }
    ///`IbcCoreConnectionV1VersionData(string,string[])`
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
    pub struct IbcCoreConnectionV1VersionData {
        pub identifier: ::std::string::String,
        pub features: ::std::vec::Vec<::std::string::String>,
    }
}
