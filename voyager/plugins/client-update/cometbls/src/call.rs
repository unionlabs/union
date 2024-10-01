use enumorph::Enumorph;
use macros::model;
use unionlabs::{ibc::core::client::height::Height, union::galois::prove_request::ProveRequest};

#[model]
#[derive(Enumorph)]
#[allow(clippy::large_enum_variant)]
pub enum ModuleCall {
    FetchUpdate(FetchUpdate),
    FetchUntrustedCommit(FetchUntrustedCommit),
    FetchTrustedValidators(FetchTrustedValidators),
    FetchUntrustedValidators(FetchUntrustedValidators),
    FetchProveRequest(FetchProveRequest),
}

#[model]
pub struct FetchUpdate {
    pub update_from: Height,
    pub update_to: Height,
}

#[model]
pub struct FetchTrustedCommit {
    pub height: Height,
}

#[model]
pub struct FetchUntrustedCommit {
    pub height: Height,
}

#[model]
pub struct FetchTrustedValidators {
    pub height: Height,
}

#[model]
pub struct FetchUntrustedValidators {
    pub height: Height,
}

#[model]
pub struct FetchProveRequest {
    pub request: ProveRequest,
}
