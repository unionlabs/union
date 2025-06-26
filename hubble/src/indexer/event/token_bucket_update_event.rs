use serde::{Deserialize, Serialize};

use crate::indexer::event::{
    header::Header,
    types::{Capacity, Denom, RefillRate},
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TokenBucketUpdateEvent {
    #[serde(flatten)]
    pub header: Header,
    pub denom: Denom,
    pub capacity: Capacity,
    pub refill_rate: RefillRate,
}
