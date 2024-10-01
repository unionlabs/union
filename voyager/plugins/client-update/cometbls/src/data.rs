use enumorph::Enumorph;
use macros::model;
use subset_of::SubsetOf;
use unionlabs::{
    ibc::core::client::height::Height,
    tendermint::types::{signed_header::SignedHeader, validator::Validator},
    union::galois::prove_response,
};

#[model]
#[derive(Enumorph, SubsetOf)]
#[allow(clippy::large_enum_variant)]
pub enum ModuleData {
    UntrustedCommit(UntrustedCommit),
    TrustedValidators(TrustedValidators),
    UntrustedValidators(UntrustedValidators),
    ProveResponse(ProveResponse),
}

#[model]
pub struct UntrustedCommit {
    pub height: Height,
    pub signed_header: SignedHeader,
}

#[model]
pub struct TrustedCommit {
    pub height: Height,
    pub signed_header: SignedHeader,
}

#[model]
pub struct TrustedValidators {
    pub height: Height,
    pub validators: Vec<Validator>,
}

#[model]
pub struct UntrustedValidators {
    pub height: Height,
    pub validators: Vec<Validator>,
}

#[model]
pub struct ProveResponse {
    pub prove_response: prove_response::ProveResponse,
}
