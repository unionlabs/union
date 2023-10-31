#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = AuthenticatorResponse , extends = :: js_sys :: Object , js_name = AuthenticatorAssertionResponse , typescript_type = "AuthenticatorAssertionResponse")]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `AuthenticatorAssertionResponse` class."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/AuthenticatorAssertionResponse)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `AuthenticatorAssertionResponse`*"]
    pub type AuthenticatorAssertionResponse;
    # [wasm_bindgen (structural , method , getter , js_class = "AuthenticatorAssertionResponse" , js_name = authenticatorData)]
    #[doc = "Getter for the `authenticatorData` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/AuthenticatorAssertionResponse/authenticatorData)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `AuthenticatorAssertionResponse`*"]
    pub fn authenticator_data(this: &AuthenticatorAssertionResponse) -> ::js_sys::ArrayBuffer;
    # [wasm_bindgen (structural , method , getter , js_class = "AuthenticatorAssertionResponse" , js_name = signature)]
    #[doc = "Getter for the `signature` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/AuthenticatorAssertionResponse/signature)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `AuthenticatorAssertionResponse`*"]
    pub fn signature(this: &AuthenticatorAssertionResponse) -> ::js_sys::ArrayBuffer;
    # [wasm_bindgen (structural , method , getter , js_class = "AuthenticatorAssertionResponse" , js_name = userHandle)]
    #[doc = "Getter for the `userHandle` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/AuthenticatorAssertionResponse/userHandle)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `AuthenticatorAssertionResponse`*"]
    pub fn user_handle(this: &AuthenticatorAssertionResponse) -> Option<::js_sys::ArrayBuffer>;
}
