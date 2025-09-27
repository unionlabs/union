use ibc::types::ChannelId;
use starknet::ContractAddress;

#[derive(Drop, starknet::Event)]
pub struct CreateWrappedToken {
    pub path: u256,
    #[key]
    pub channel_id: ChannelId,
    pub base_token: ByteArray,
    // TODO(aeryz): could also be an address, check deterministic address creation
    #[key]
    pub quote_token: ContractAddress,
    pub metadata: ByteArray,
    pub kind: u8,
}
