use enumorph::Enumorph;
use macros::model;
use unionlabs::ibc::core::client::height::Height;

#[model]
#[derive(Enumorph)]
#[allow(clippy::large_enum_variant)]
pub enum ModuleCall {
    FetchUpdateBoot(FetchUpdateBoot),
    FetchUpdate(FetchUpdate),
    FetchProveRequest(FetchProveRequest),
}

#[model]
pub struct FetchUpdateBoot {
    pub update_from: Height,
    pub update_to: Height,
}

#[model]
pub struct FetchUpdate {
    pub update_from: Height,
    pub update_to: Height,
}

#[model]
pub struct FetchProveRequest {
    pub update_from: Height,
    pub request: galois_rpc::prove_request::ProveRequest,
}
