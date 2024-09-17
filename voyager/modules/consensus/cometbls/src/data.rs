use enumorph::Enumorph;
use queue_msg::{queue_msg, SubsetOf};
use unionlabs::{
    ibc::core::client::height::Height,
    tendermint::types::{signed_header::SignedHeader, validator::Validator},
    union::galois::prove_response,
};

#[queue_msg]
#[derive(Enumorph, SubsetOf)]
pub enum ModuleData {
    UntrustedCommit(UntrustedCommit),
    TrustedValidators(TrustedValidators),
    UntrustedValidators(UntrustedValidators),
    ProveResponse(ProveResponse),
}

#[queue_msg]
pub struct UntrustedCommit {
    pub height: Height,
    pub signed_header: SignedHeader,
}

#[queue_msg]
pub struct TrustedCommit {
    pub height: Height,
    pub signed_header: SignedHeader,
}

#[queue_msg]
pub struct TrustedValidators {
    pub height: Height,
    pub validators: Vec<Validator>,
}

#[queue_msg]
pub struct UntrustedValidators {
    pub height: Height,
    pub validators: Vec<Validator>,
}

#[queue_msg]
pub struct ProveResponse {
    pub prove_response: prove_response::ProveResponse,
}
