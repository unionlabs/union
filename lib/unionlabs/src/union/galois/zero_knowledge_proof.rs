use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use crate::{Proto, TypeUrl};

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
// REVIEW: Are these fields fixed size?
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct ZeroKnowledgeProof {
    #[serde(with = "::serde_utils::hex_string")]
    pub content: Vec<u8>,
    #[serde(with = "::serde_utils::hex_string")]
    pub compressed_content: Vec<u8>,
    #[serde(with = "::serde_utils::hex_string")]
    pub evm_proof: Vec<u8>,
    #[serde(with = "::serde_utils::hex_string")]
    pub public_inputs: Vec<u8>,
}

impl Debug for ZeroKnowledgeProof {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ZeroKnowledgeProof")
            .field("content", &serde_utils::to_hex(&self.content))
            .field(
                "compressed_content",
                &serde_utils::to_hex(&self.compressed_content),
            )
            .field("evm_proof", &serde_utils::to_hex(&self.evm_proof))
            .field("public_inputs", &serde_utils::to_hex(&self.public_inputs))
            .finish()
    }
}

impl Proto for ZeroKnowledgeProof {
    type Proto = protos::union::galois::api::v2::ZeroKnowledgeProof;
}

impl TypeUrl for protos::union::galois::api::v2::ZeroKnowledgeProof {
    const TYPE_URL: &'static str = "/union.galois.api.v2.ZeroKnowledgeProof";
}

impl From<ZeroKnowledgeProof> for protos::union::galois::api::v2::ZeroKnowledgeProof {
    fn from(value: ZeroKnowledgeProof) -> Self {
        Self {
            content: value.content,
            compressed_content: value.compressed_content,
            evm_proof: value.evm_proof,
            public_inputs: value.public_inputs,
        }
    }
}

impl From<protos::union::galois::api::v2::ZeroKnowledgeProof> for ZeroKnowledgeProof {
    fn from(value: protos::union::galois::api::v2::ZeroKnowledgeProof) -> Self {
        Self {
            content: value.content,
            compressed_content: value.compressed_content,
            evm_proof: value.evm_proof,
            public_inputs: value.public_inputs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde() {
        let json = serde_json::to_string_pretty(&ZeroKnowledgeProof {
            content: [].into(),
            compressed_content: [].into(),
            evm_proof: [].into(),
            public_inputs: [].into(),
        })
        .unwrap();

        println!("{json}");
    }
}
