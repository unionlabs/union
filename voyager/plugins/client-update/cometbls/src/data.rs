use enumorph::Enumorph;
use macros::model;
use subset_of::SubsetOf;

#[model]
#[derive(Enumorph, SubsetOf)]
#[allow(clippy::large_enum_variant)]
pub enum ModuleData {
    ProveResponse(ProveResponse),
}

#[model]
pub struct ProveResponse {
    pub prove_response: galois_rpc::prove_response::ProveResponse,
}
