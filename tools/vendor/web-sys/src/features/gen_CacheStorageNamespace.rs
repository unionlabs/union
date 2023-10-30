#![allow(unused_imports)]
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
#[doc = "The `CacheStorageNamespace` enum."]
#[doc = ""]
#[doc = "*This API requires the following crate features to be activated: `CacheStorageNamespace`*"]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheStorageNamespace {
    Content = "content",
    Chrome = "chrome",
}
