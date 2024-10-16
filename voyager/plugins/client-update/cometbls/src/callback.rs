use enumorph::Enumorph;
use voyager_message::macros::model;
use subset_of::SubsetOf;
use unionlabs::ibc::{
    core::client::height::Height,
    lightclients::cometbls::{header::Header, signed_header::SignedHeader},
};
use voyager_message::{
    core::ChainId,
    data::{DecodedHeaderMeta, OrderedHeaders},
    VoyagerMessage,
};
use voyager_vm::{data, Op};

use crate::{data::ProveResponse, Module};

#[model]
#[derive(Enumorph, SubsetOf)]
#[allow(clippy::large_enum_variant)]
pub enum ModuleCallback {
    AggregateHeader(AggregateHeader),
}

#[model]
pub struct AggregateHeader {
    pub chain_id: ChainId<'static>,

    pub signed_header: unionlabs::tendermint::types::signed_header::SignedHeader,

    pub update_from: Height,
    pub update_to: Height,
}

impl Module {
    pub fn aggregate_header(
        &self,
        AggregateHeader {
            signed_header,
            chain_id: _,
            update_from,
            update_to: _,
        }: AggregateHeader,
        ProveResponse {
            prove_response: response,
        }: ProveResponse,
    ) -> Op<VoyagerMessage> {
        data(OrderedHeaders {
            headers: vec![(
                DecodedHeaderMeta {
                    height: Height::new_with_revision(
                        update_from.revision(),
                        signed_header.header.height.inner().try_into().unwrap(),
                    ),
                },
                serde_json::to_value(Header {
                    signed_header: SignedHeader {
                        height: signed_header.header.height,
                        time: signed_header.header.time,
                        validators_hash: signed_header.header.validators_hash,
                        next_validators_hash: signed_header.header.next_validators_hash,
                        app_hash: signed_header.header.app_hash,
                    },
                    trusted_height: update_from,
                    zero_knowledge_proof: response.proof.evm_proof,
                })
                .unwrap(),
            )],
        })
    }
}
