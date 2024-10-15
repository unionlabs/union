use enumorph::Enumorph;
use voyager_message::macros::model;
use subset_of::SubsetOf;
use unionlabs::union::galois::prove_response;

#[model]
#[derive(Enumorph, SubsetOf)]
#[allow(clippy::large_enum_variant)]
pub enum ModuleData {
    ProveResponse(ProveResponse),
}

#[model]
pub struct ProveResponse {
    pub prove_response: prove_response::ProveResponse,
}
