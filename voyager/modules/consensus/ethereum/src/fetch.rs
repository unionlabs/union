use enumorph::Enumorph;
use queue_msg::queue_msg;

#[queue_msg]
#[derive(Enumorph)]
pub enum ModuleFetch {
    FetchLightClientUpdate(FetchLightClientUpdate),
    FetchFinalityUpdate(FetchFinalityUpdate),
    FetchLightClientUpdates(FetchLightClientUpdates),
    FetchBootstrap(FetchBootstrap),
    FetchAccountUpdate(FetchAccountUpdate),
    FetchBeaconGenesis(FetchBeaconGenesis),
    FetchBeaconSpec(FetchBeaconSpec),
}

#[queue_msg]
pub struct FetchLightClientUpdate {
    pub period: u64,
}

#[queue_msg]
pub struct FetchFinalityUpdate {}

#[queue_msg]
pub struct FetchLightClientUpdates {
    pub trusted_period: u64,
    pub target_period: u64,
}

#[queue_msg]
pub struct FetchBootstrap {
    pub slot: u64,
}

#[queue_msg]
pub struct FetchAccountUpdate {
    pub slot: u64,
}

#[queue_msg]
pub struct FetchBeaconGenesis {}

#[queue_msg]
pub struct FetchBeaconSpec {}
