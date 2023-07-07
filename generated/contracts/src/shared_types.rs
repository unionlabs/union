///`IbcCoreChannelV1CounterpartyData(string,string)`
#[derive(
    Clone,
    ::ethers::contract::EthAbiType,
    ::ethers::contract::EthAbiCodec,
    Default,
    Debug,
    PartialEq,
    Eq,
    Hash
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
    Hash
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
///`IbcCoreClientV1HeightData(uint64,uint64)`
#[derive(
    Clone,
    ::ethers::contract::EthAbiType,
    ::ethers::contract::EthAbiCodec,
    Default,
    Debug,
    PartialEq,
    Eq,
    Hash
)]
pub struct IbcCoreClientV1HeightData {
    pub revision_number: u64,
    pub revision_height: u64,
}
