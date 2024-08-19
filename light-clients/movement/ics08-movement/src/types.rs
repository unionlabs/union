use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountAddress(pub [u8; 32]);

// TODO(aeryz): add `SerializeKey` and `DeserializeKey` impls
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// A BLS12381 public key
pub struct PublicKey {
    pub pubkey: [u8; 48],
}
