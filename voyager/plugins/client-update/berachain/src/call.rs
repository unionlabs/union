use enumorph::Enumorph;
use macros::model;
use unionlabs::ibc::core::client::height::Height;

#[model]
#[derive(Enumorph)]
pub enum ModuleCall {
    FetchUpdate(FetchUpdate),
}

#[model]
pub struct FetchUpdate {
    pub update_from: Height,
    pub update_to: Height,
}
