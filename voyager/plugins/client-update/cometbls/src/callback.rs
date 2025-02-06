use cometbls_light_client_types::{header::Header, light_header::LightHeader};
use enumorph::Enumorph;
use macros::model;
use subset_of::SubsetOf;
use unionlabs::ibc::core::client::height::Height;
use voyager_message::{
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
pub struct AggregateHeader {}

impl Module {
    pub fn aggregate_header(
        &self,
        _: AggregateHeader,
        prove_responses: impl IntoIterator<Item = ProveResponse>,
    ) -> Op<VoyagerMessage> {
        let make_header = |ProveResponse {
                               update_from,
                               header,
                               prove_response: response,
                           }| {
            (
                DecodedHeaderMeta {
                    height: Height::new_with_revision(
                        update_from.revision(),
                        header.height.inner().try_into().unwrap(),
                    ),
                },
                serde_json::to_value(Header {
                    signed_header: LightHeader {
                        height: header.height,
                        time: header.time,
                        validators_hash: header.validators_hash.into_encoding(),
                        next_validators_hash: header.next_validators_hash.into_encoding(),
                        app_hash: header.app_hash.into_encoding(),
                    },
                    trusted_height: update_from,
                    zero_knowledge_proof: response.proof.evm_proof.into(),
                })
                .unwrap(),
            )
        };
        data(OrderedHeaders {
            headers: prove_responses.into_iter().map(make_header).collect(),
        })
    }
}
