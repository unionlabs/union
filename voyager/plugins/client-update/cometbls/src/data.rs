use cometbft_types::types::header::Header;
use enumorph::Enumorph;
use macros::model;
use subset_of::SubsetOf;
use unionlabs::ibc::core::client::height::Height;

#[model]
#[derive(Enumorph, SubsetOf)]
#[allow(clippy::large_enum_variant)]
pub enum ModuleData {
    ProveResponse(ProveResponse),
}

#[model]
pub struct ProveResponse {
    pub update_from: Height,
    pub header: Header,
    pub prove_response: galois_rpc::prove_response::ProveResponse,
}
