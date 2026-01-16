use enumorph::Enumorph;
use macros::model;
use subset_of::SubsetOf;
use unionlabs::ibc::core::client::height::Height;
use voyager_sdk::primitives::ChainId;

#[model]
#[derive(Enumorph, SubsetOf)]
#[allow(clippy::large_enum_variant)]
pub enum ModuleData {
    ProveResponse(ProveResponse),
}

#[model]
pub struct ProveResponse {
    pub update_from: Height,
    pub prove_request: galois_rpc::prove_request::ProveRequest,
    pub prove_response: galois_rpc::prove_response::ProveResponse,
    pub counterparty_chain_id: ChainId,
}
