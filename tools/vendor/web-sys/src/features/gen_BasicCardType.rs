#![allow(unused_imports)]
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
#[doc = "The `BasicCardType` enum."]
#[doc = ""]
#[doc = "*This API requires the following crate features to be activated: `BasicCardType`*"]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BasicCardType {
    Credit = "credit",
    Debit = "debit",
    Prepaid = "prepaid",
}
