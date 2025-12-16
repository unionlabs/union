use cosmwasm_event::Event;
use cosmwasm_std::Addr;

#[derive(Event)]
#[event("proxy_created")]
pub struct ProxyCreated<'a> {
    pub creator: &'a Addr,
    pub proxy: &'a Addr,
}
