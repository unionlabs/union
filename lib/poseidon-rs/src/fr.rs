#![allow(
    clippy::too_many_arguments, // fork
    clippy::disallowed_types // fork
)]

use ethereum_types::U256;
use ff::{Field, PrimeFieldDecodingError, PrimeFieldRepr};

#[derive(PrimeField)]
#[PrimeFieldModulus = "21888242871839275222246405745257275088548364400416034343698204186575808495617"]
#[PrimeFieldGenerator = "7"]
pub struct Fr(FrRepr);

use ff::PrimeField;

impl Fr {
    #[must_use]
    pub fn as_u256(self) -> U256 {
        self.into()
    }

    #[must_use]
    pub fn from_u64(val: u64) -> Self {
        let val: U256 = val.into();
        Fr::from_repr(FrRepr(val.0)).expect("u64 should in field")
    }

    #[must_use]
    pub fn from_usize(val: usize) -> Self {
        Self::from_u64(val as u64)
    }

    pub fn from_big_endian(data: &[u8]) -> Result<Self, PrimeFieldDecodingError> {
        let repr = if data.len() == 32 {
            let mut repr = FrRepr::default();
            repr.read_be(data).unwrap();
            repr
        } else {
            let val = U256::from_big_endian(data);
            FrRepr(val.0)
        };
        Fr::from_repr(repr)
    }

    pub fn from_little_endian(data: &[u8]) -> Result<Self, PrimeFieldDecodingError> {
        let repr = if data.len() == 32 {
            let mut repr = FrRepr::default();
            repr.read_le(data).unwrap();
            repr
        } else {
            let val = U256::from_little_endian(data);
            FrRepr(val.0)
        };
        Fr::from_repr(repr)
    }

    #[must_use]
    pub fn bytes(&self) -> [u64; 4] {
        self.0 .0
    }

    #[must_use]
    pub fn to_big_endian(self) -> [u8; 32] {
        let mut buf = [0_u8; 32];
        self.into_repr().write_be(&mut buf[..]).unwrap();
        buf
    }

    #[must_use]
    pub fn to_little_endian(self) -> [u8; 32] {
        let mut buf = [0_u8; 32];
        self.into_repr().write_le(&mut buf[..]).unwrap();
        buf
    }
}

impl From<u64> for Fr {
    fn from(val: u64) -> Self {
        Fr::from_u64(val)
    }
}

impl From<usize> for Fr {
    fn from(val: usize) -> Self {
        Fr::from_usize(val)
    }
}

impl From<Fr> for U256 {
    fn from(f: Fr) -> Self {
        let repr = f.into_repr();
        let required_length = repr.as_ref().len() * 8;
        let mut buf: Vec<u8> = Vec::with_capacity(required_length);
        repr.write_be(&mut buf).unwrap();
        U256::from_big_endian(&buf)
    }
}

#[test]
fn fr_serde() {
    let decimal = "4417881134626180770308697923359573201005643519861877412381846989312604493735";
    assert_eq!(
        Fr::from_str(decimal).unwrap(),
        Fr::from_repr(FrRepr(U256::from_str_radix(decimal, 10).unwrap().0)).unwrap()
    );
}
