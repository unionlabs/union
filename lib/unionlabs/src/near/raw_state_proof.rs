use macros::model;

#[model]
pub struct RawStateProof {
    pub state_proof: Vec<Vec<u8>>,
}
