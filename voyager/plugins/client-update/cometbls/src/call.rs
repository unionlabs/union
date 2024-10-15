use enumorph::Enumorph;
use voyager_message::macros::model;
use unionlabs::{ibc::core::client::height::Height, union::galois::prove_request::ProveRequest};

#[model]
#[derive(Enumorph)]
#[allow(clippy::large_enum_variant)]
pub enum ModuleCall {
    FetchUpdate(FetchUpdate),
    FetchProveRequest(FetchProveRequest),
}

#[model]
pub struct FetchUpdate {
    pub update_from: Height,
    pub update_to: Height,
}

#[model]
pub struct FetchProveRequest {
    pub request: ProveRequest,
}
