use enumorph::Enumorph;
use queue_msg::queue_msg;
use unionlabs::{ibc::core::client::height::Height, union::galois::prove_request::ProveRequest};

#[queue_msg]
#[derive(Enumorph)]
pub enum ModuleFetch {
    FetchUntrustedCommit(FetchUntrustedCommit),
    FetchTrustedValidators(FetchTrustedValidators),
    FetchUntrustedValidators(FetchUntrustedValidators),
    FetchProveRequest(FetchProveRequest),
}

#[queue_msg]
pub struct FetchTrustedCommit {
    pub height: Height,
}

#[queue_msg]
pub struct FetchUntrustedCommit {
    pub height: Height,
}

#[queue_msg]
pub struct FetchTrustedValidators {
    pub height: Height,
}

#[queue_msg]
pub struct FetchUntrustedValidators {
    pub height: Height,
}

#[queue_msg]
pub struct FetchProveRequest {
    pub request: ProveRequest,
}
