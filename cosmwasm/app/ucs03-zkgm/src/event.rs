use cosmwasm_event::Event;
use cosmwasm_std::{Addr, Uint256};
use ibc_union_spec::ChannelId;
use unionlabs_primitives::{Bytes, U256};

#[derive(Event)]
#[event("solver")]
pub struct Solver {
    pub market_maker: Bytes,
}

#[derive(Event)]
#[event("token_bucket_update")]
pub struct TokenBucketUpdate {
    pub denom: String,
    pub capacity: Uint256,
    pub refill_rate: Uint256,
}

#[derive(Event)]
#[event("create_proxy_account")]
pub struct CreateProxyAccount {
    pub path: U256,
    pub channel_id: ChannelId,
    pub owner: Bytes,
    pub address: Addr,
}
