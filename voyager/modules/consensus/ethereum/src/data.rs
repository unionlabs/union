use beacon_api::types::Spec;
use enumorph::Enumorph;
use queue_msg::{queue_msg, SubsetOf};
use unionlabs::{
    ethereum::beacon::{
        genesis_data::GenesisData, light_client_bootstrap::UnboundedLightClientBootstrap,
        light_client_finality_update::UnboundedLightClientFinalityUpdate,
    },
    ibc::lightclients::ethereum::{
        account_update::AccountUpdate, header::UnboundedHeader,
        light_client_update::UnboundedLightClientUpdate,
    },
};

#[queue_msg]
#[derive(Enumorph, SubsetOf)]
pub enum ModuleData {
    FinalityUpdate(FinalityUpdate),
    LightClientUpdates(LightClientUpdates),
    LightClientUpdate(LightClientUpdate),
    Bootstrap(BootstrapData),
    AccountUpdate(AccountUpdateData),
    BeaconGenesis(BeaconGenesis),
    BeaconSpec(BeaconSpec),
    Header(Header),
}

#[queue_msg]
pub struct BootstrapData {
    pub slot: u64,
    pub bootstrap: UnboundedLightClientBootstrap,
}

#[queue_msg]
pub struct AccountUpdateData {
    pub slot: u64,
    pub update: AccountUpdate,
}

#[queue_msg]
pub struct BeaconGenesis {
    pub genesis: GenesisData,
}

#[queue_msg]
pub struct FinalityUpdate {
    pub finality_update: UnboundedLightClientFinalityUpdate,
}

#[queue_msg]
pub struct LightClientUpdates {
    pub light_client_updates: Vec<UnboundedLightClientUpdate>,
}

#[queue_msg]
pub struct LightClientUpdate {
    pub update: UnboundedLightClientUpdate,
}

#[queue_msg]
pub struct Header {
    pub header: UnboundedHeader,
}

#[queue_msg]
pub struct BeaconSpec {
    pub spec: Spec,
}
