#![allow(unused_imports)]
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
#[doc = "The `NotificationPermission` enum."]
#[doc = ""]
#[doc = "*This API requires the following crate features to be activated: `NotificationPermission`*"]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationPermission {
    Default = "default",
    Denied = "denied",
    Granted = "granted",
}
