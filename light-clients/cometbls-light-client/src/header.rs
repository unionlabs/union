use unionlabs::{ethereum_consts_traits::ChainSpec, ibc::lightclients::ethereum::header::Header};

// REVIEW: Unused?
#[derive(serde::Serialize, serde::Deserialize)]
pub enum ClientMessage<C: ChainSpec> {
    Header(Header<C>),
}
