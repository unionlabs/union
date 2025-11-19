#[derive(Drop, starknet::Event)]
pub struct CreateWrappedToken {
    pub path: u256,
    #[key]
    pub channel_id: u32,
    pub base_token: ByteArray,
    // TODO(aeryz): could also be an address, check deterministic address creation
    #[key]
    pub quote_token: ByteArray,
    pub metadata: ByteArray,
    pub kind: u8,
}
