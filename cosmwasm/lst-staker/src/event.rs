use cosmwasm_event::Event;
use cosmwasm_std::{Addr, Decimal256};

#[derive(Event)]
#[event("set_lst_hub_address")]
pub struct SetLstHubAddress {
    pub address: Addr,
}

#[derive(Event)]
#[event("rebase")]
pub struct Rebase {
    pub restaked_rewards: u128,
}

#[derive(Event)]
#[event("unstake")]
pub struct Unstake {
    pub total: u128,
}

#[derive(Event)]
#[event("stake")]
pub struct Stake {
    pub total: u128,
    pub pending_rewards: u128,
}

#[derive(Event)]
#[event("set_validators")]
pub struct SetValidators {
    pub total_shares: u128,
}

#[derive(Event)]
#[event("validator_configured")]
pub struct ValidatorConfigured<'a> {
    pub address: &'a str,
    pub shares: u128,
    pub weight: Decimal256,
}
