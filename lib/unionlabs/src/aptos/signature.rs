use macros::model;
use unionlabs_primitives::H768;

#[model]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct AggregateSignature {
    pub validator_bitmask: ValidatorBitmask,
    pub sig: Option<H768>,
}

#[model]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct ValidatorBitmask {
    pub inner: Vec<u8>,
}
