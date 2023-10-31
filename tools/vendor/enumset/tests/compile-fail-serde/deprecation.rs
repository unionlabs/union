#![deny(deprecated)]

use enumset::*;
use serde_derive::*;

#[derive(Serialize, Deserialize, EnumSetType, Debug)]
#[enumset(serialize_as_map)]
#[serde(crate="enumset::__internal::serde")]
pub enum MapEnum {
    A, B, C, D, E, F, G, H,
}

#[derive(Serialize, Deserialize, EnumSetType, Debug)]
#[enumset(serialize_as_list)]
#[serde(crate="enumset::__internal::serde")]
pub enum ListEnum {
    A, B, C, D, E, F, G, H,
}

fn main() {}
