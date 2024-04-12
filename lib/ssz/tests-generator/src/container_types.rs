use serde::Deserialize;
use ssz::{
    types::{BitList, BitVector, List, Vector},
    Ssz,
};
use typenum::U;

#[derive(Debug, PartialEq, Clone, Deserialize, Ssz)]
#[serde(rename_all = "UPPERCASE")]
pub struct SingleFieldTestStruct {
    pub a: u8,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Ssz)]
#[serde(rename_all = "UPPERCASE")]
pub struct SmallTestStruct {
    pub a: u16,
    pub b: u16,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Ssz)]
#[serde(rename_all = "UPPERCASE")]
pub struct FixedTestStruct {
    pub a: u8,
    pub b: u64,
    pub c: u32,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Ssz)]
#[serde(rename_all = "UPPERCASE")]
pub struct VarTestStruct {
    pub a: u16,
    pub b: List<u16, U<1024>>,
    pub c: u8,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Ssz)]
#[serde(rename_all = "UPPERCASE")]
pub struct ComplexTestStruct {
    pub a: u16,
    pub b: List<u16, U<128>>,
    pub c: u8,
    #[serde(with = "::serde_utils::hex_string")]
    pub d: List<u8, U<256>>,
    pub e: VarTestStruct,
    pub f: Vector<FixedTestStruct, U<4>>,
    pub g: Vector<VarTestStruct, U<2>>,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Ssz)]
#[serde(rename_all = "UPPERCASE")]
pub struct BitsStruct {
    pub a: BitList<U<5>>,
    pub b: BitVector<U<2>>,
    pub c: BitVector<U<1>>,
    pub d: BitList<U<6>>,
    pub e: BitVector<U<8>>,
}
