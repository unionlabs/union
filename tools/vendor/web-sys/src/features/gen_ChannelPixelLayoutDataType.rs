#![allow(unused_imports)]
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
#[doc = "The `ChannelPixelLayoutDataType` enum."]
#[doc = ""]
#[doc = "*This API requires the following crate features to be activated: `ChannelPixelLayoutDataType`*"]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelPixelLayoutDataType {
    Uint8 = "uint8",
    Int8 = "int8",
    Uint16 = "uint16",
    Int16 = "int16",
    Uint32 = "uint32",
    Int32 = "int32",
    Float32 = "float32",
    Float64 = "float64",
}
