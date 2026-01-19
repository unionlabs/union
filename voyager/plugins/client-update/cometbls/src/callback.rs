use cometbls_groth16_verifier::ZKP;
use cometbls_light_client_types::{ChainId, header::Header, light_header::LightHeader};
use enumorph::Enumorph;
use garaga_rs::calldata::{
    G1PointBigUint, G2PointBigUint,
    cometbls_groth16::CometblsGroth16VerifyingKey,
    full_proof_with_hints::groth16::{Groth16Proof, Groth16VerificationKey},
};
use macros::model;
use num_bigint::BigUint;
use serde_json::Value;
use subset_of::SubsetOf;
use substrate_bn::{AffineG1, AffineG2, Fq, G1, G2};
use unionlabs::{
    ibc::core::client::height::Height,
    primitives::{Bytes, H256},
};
use voyager_sdk::{
    message::{
        VoyagerMessage,
        data::{DecodedHeaderMeta, OrderedHeaders},
    },
    rpc::{RpcError, RpcResult},
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
    ) -> RpcResult<Op<VoyagerMessage>> {
        let make_header = |ProveResponse {
                               update_from,
                               prove_request,
                               prove_response: response,
                               counterparty_chain_id,
                           }|
         -> RpcResult<(DecodedHeaderMeta, Value)> {
            Ok((
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
                {
                    let mut header = Header {
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
                    };

                    if self.cairo_chain_ids.contains(&counterparty_chain_id) {
                        let zkp = prepare_zkp_for_cairo(
                            &prove_request.vote.chain_id,
                            response.trusted_validator_set_root,
                            &header,
                        )
                        .map_err(RpcError::unprocessable)?;
                        header.zero_knowledge_proof = zkp;
                    }

                    serde_json::to_value(header).unwrap()
                },
            ))
        };
        Ok(data(OrderedHeaders {
            headers: prove_responses
                .into_iter()
                .map(make_header)
                .collect::<RpcResult<Vec<_>>>()?,
        }))
    }
}

/// This is required since we are using [garaga](https://github.com/keep-starknet-strange/garaga) to optimize
/// the on-chain verification. Via `garaga`, we are computing some hints for the pairing checks to reduce the
/// amount of on-chain compute we do.
fn prepare_zkp_for_cairo(
    chain_id: &str,
    trusted_validators_hash: H256,
    header: &Header,
) -> Result<Bytes, cometbls_groth16_verifier::Error> {
    let zkp = ZKP::try_from(header.zero_knowledge_proof.as_ref())?;

    let public_inputs = cometbls_groth16_verifier::public_inputs(
        &ChainId::from_string(chain_id).unwrap(),
        trusted_validators_hash,
        &header.signed_header,
        &zkp,
    )?;

    let make_big_uint = |point: Fq| {
        let mut buffer = Vec::new();
        point.to_big_endian(&mut buffer).unwrap();
        BigUint::from_bytes_be(&buffer)
    };

    let make_g1_affine = |point: AffineG1| G1PointBigUint {
        x: make_big_uint(point.x()),
        y: make_big_uint(point.y()),
    };

    let make_g1 = |point: G1| G1PointBigUint {
        x: make_big_uint(point.x()),
        y: make_big_uint(point.y()),
    };

    let make_g2_affine = |point: AffineG2| G2PointBigUint {
        x0: make_big_uint(point.x().real()),
        x1: make_big_uint(point.x().imaginary()),
        y0: make_big_uint(point.y().real()),
        y1: make_big_uint(point.y().imaginary()),
    };

    let make_g2 = |point: G2| G2PointBigUint {
        x0: make_big_uint(point.x().real()),
        x1: make_big_uint(point.x().imaginary()),
        y0: make_big_uint(point.y().real()),
        y1: make_big_uint(point.y().imaginary()),
    };

    let proof = Groth16Proof {
        a: make_g1_affine(zkp.proof.a),
        b: make_g2_affine(zkp.proof.b),
        c: make_g1_affine(zkp.proof.c),
        public_inputs: public_inputs
            .into_iter()
            .map(|pi| {
                let mut buffer = Vec::new();
                pi.to_big_endian(&mut buffer).unwrap();
                BigUint::from_bytes_be(&buffer)
            })
            .collect(),
        image_id_journal_risc0: None,
        vkey_public_values_sp1: None,
    };

    let vk = Groth16VerificationKey {
        alpha: make_g1(cometbls_groth16_verifier::ALPHA_G1),
        beta: make_g2(cometbls_groth16_verifier::BETA_NEG_G2),
        gamma: make_g2(cometbls_groth16_verifier::GAMMA_NEG_G2),
        delta: make_g2(cometbls_groth16_verifier::DELTA_NEG_G2),
        ic: cometbls_groth16_verifier::GAMMA_ABC_G1
            .into_iter()
            .map(make_g1)
            .collect(),
    };

    let vk = CometblsGroth16VerifyingKey {
        groth16_vk: vk,
        commitment_key_g: make_g2(cometbls_groth16_verifier::PEDERSEN_G),
        commitment_key_g_root_sigma_neg: make_g2(
            cometbls_groth16_verifier::PEDERSEN_G_ROOT_SIGMA_NEG,
        ),
    };

    let proof_commitment = make_g1_affine(zkp.proof_commitment);
    let proof_commitment_pok = make_g1_affine(zkp.proof_commitment_pok);

    let _proof = garaga_rs::calldata::cometbls_groth16::CometblsGroth16Proof::generate_calldata(
        proof,
        vk,
        proof_commitment,
        proof_commitment_pok,
    );

    // TODO(aeryz): The encoding of the proof is not determined yet. The `proof` type here returns a bunch of felt252's
    // so we will probably pass it along as is.

    todo!()
}
