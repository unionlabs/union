use macros::model;

// REVIEW: Are these fields fixed size?
#[model(proto(raw(protos::union::galois::api::v3::ZeroKnowledgeProof), into, from))]
pub struct ZeroKnowledgeProof {
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_string"))]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub content: Vec<u8>,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_string"))]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub compressed_content: Vec<u8>,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_string"))]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub evm_proof: Vec<u8>,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_string"))]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub public_inputs: Vec<u8>,
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::union::galois::zero_knowledge_proof::ZeroKnowledgeProof;

    impl From<ZeroKnowledgeProof> for protos::union::galois::api::v3::ZeroKnowledgeProof {
        fn from(value: ZeroKnowledgeProof) -> Self {
            Self {
                content: value.content,
                compressed_content: value.compressed_content,
                evm_proof: value.evm_proof,
                public_inputs: value.public_inputs,
            }
        }
    }

    impl From<protos::union::galois::api::v3::ZeroKnowledgeProof> for ZeroKnowledgeProof {
        fn from(value: protos::union::galois::api::v3::ZeroKnowledgeProof) -> Self {
            Self {
                content: value.content,
                compressed_content: value.compressed_content,
                evm_proof: value.evm_proof,
                public_inputs: value.public_inputs,
            }
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
