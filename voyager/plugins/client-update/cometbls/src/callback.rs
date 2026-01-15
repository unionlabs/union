use cometbls_light_client_types::{header::Header, light_header::LightHeader};
use enumorph::Enumorph;
use macros::model;
use subset_of::SubsetOf;
use unionlabs::ibc::core::client::height::Height;
use voyager_sdk::{
    message::{
        VoyagerMessage,
        data::{DecodedHeaderMeta, OrderedHeaders},
    },
    vm::{Op, data},
};

use crate::{Module, data::ProveResponse};

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
                               prove_request,
                               prove_response: response,
                           }| {
            (
                DecodedHeaderMeta {
                    height: Height::new_with_revision(
                        update_from.revision(),
                        prove_request
                            .untrusted_header
                            .height
                            .inner()
                            .try_into()
                            .unwrap(),
                    ),
                },
                serde_json::to_value(Header {
                    signed_header: LightHeader {
                        height: prove_request.untrusted_header.height,
                        time: prove_request.untrusted_header.time,
                        validators_hash: prove_request
                            .untrusted_header
                            .validators_hash
                            .into_encoding(),
                        next_validators_hash: prove_request
                            .untrusted_header
                            .next_validators_hash
                            .into_encoding(),
                        app_hash: prove_request.untrusted_header.app_hash.into_encoding(),
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
