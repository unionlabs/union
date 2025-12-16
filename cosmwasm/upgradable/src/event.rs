use std::num::NonZero;

use cosmwasm_event::Event;
use serde_json::Value;

#[derive(Event)]
#[event("upgrade")]
pub struct Upgrade<'a> {
    pub new_code_id: NonZero<u64>,
    pub msg: &'a Value,
}
