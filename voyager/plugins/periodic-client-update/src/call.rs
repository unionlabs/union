use macros::model;
use voyager_message::{
    core::{ChainId, IbcSpecId},
    RawClientId,
};
use voyager_vm::BoxDynError;

#[model]
pub enum ModuleCall {
    CheckForClientAge(CheckForClientAge),
}

#[model]
#[derive(clap::Args)]
pub struct CheckForClientAge {
    #[arg(value_parser(|s: &str| Ok::<_, BoxDynError>(ChainId::new(s.to_owned()))))]
    pub chain_id: ChainId,
    #[arg(value_parser(|s: &str| Ok::<_, BoxDynError>(IbcSpecId::new(s.to_owned()))))]
    pub ibc_spec_id: IbcSpecId,
    pub client_id: RawClientId,
    /// The maximum amount of blocks this client can lag behind the latest finalized height of the chain it's tracking.
    pub max_age: u64,
}
