#![allow(unused_imports)]
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
#[doc = "The `SupportedType` enum."]
#[doc = ""]
#[doc = "*This API requires the following crate features to be activated: `SupportedType`*"]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportedType {
    TextHtml = "text/html",
    TextXml = "text/xml",
    ApplicationXml = "application/xml",
    ApplicationXhtmlXml = "application/xhtml+xml",
    ImageSvgXml = "image/svg+xml",
}
