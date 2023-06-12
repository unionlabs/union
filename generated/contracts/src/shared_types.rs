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
