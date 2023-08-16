use protos::tendermint::types::SimpleValidator;

#[derive(Debug, Clone, PartialEq)]
pub struct ValidatorSetCommit {
    pub validators: Vec<SimpleValidator>,
    pub signatures: Vec<Vec<u8>>,
    pub bitmap: Vec<u8>,
}
