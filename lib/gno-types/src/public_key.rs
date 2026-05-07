use serde::{Deserialize, Serialize};
use unionlabs::primitives::{Bytes, encoding::Base64};

// TODO: These are fixed sizes, not arbitrary bytes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "@type", content = "value", rename_all = "snake_case")]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub enum PublicKey {
    #[serde(rename = "/tm.PubKeyEd25519")]
    Ed25519(Bytes<Base64>),
    #[serde(rename = "/tm.PubKeySecp256k1")]
    Secp256k1(Bytes<Base64>),
}

#[cfg(test)]
mod tests {
    use unionlabs::test_utils::assert_json_roundtrip;

    use super::*;

    #[test]
    fn roundtrip() {
        let json = r#"
            {
              "type": "/tm.PubKeySecp256k1",
              "value": "Auh5XsGBWLOxumt62xmwjc0atN4TKdUMyJMJ8UN0yvgW"
            }
        "#;

        let public_key = serde_json::from_str::<PublicKey>(json).unwrap();

        assert_json_roundtrip(&public_key);
    }
}
