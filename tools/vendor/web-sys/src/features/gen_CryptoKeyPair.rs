#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = CryptoKeyPair)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `CryptoKeyPair` dictionary."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `CryptoKeyPair`*"]
    pub type CryptoKeyPair;
}
impl CryptoKeyPair {
    #[cfg(feature = "CryptoKey")]
    #[doc = "Construct a new `CryptoKeyPair`."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `CryptoKey`, `CryptoKeyPair`*"]
    pub fn new(private_key: &CryptoKey, public_key: &CryptoKey) -> Self {
        #[allow(unused_mut)]
        let mut ret: Self = ::wasm_bindgen::JsCast::unchecked_into(::js_sys::Object::new());
        ret.private_key(private_key);
        ret.public_key(public_key);
        ret
    }
    #[cfg(feature = "CryptoKey")]
    #[doc = "Change the `privateKey` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `CryptoKey`, `CryptoKeyPair`*"]
    pub fn private_key(&mut self, val: &CryptoKey) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(
            self.as_ref(),
            &JsValue::from("privateKey"),
            &JsValue::from(val),
        );
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
    #[cfg(feature = "CryptoKey")]
    #[doc = "Change the `publicKey` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `CryptoKey`, `CryptoKeyPair`*"]
    pub fn public_key(&mut self, val: &CryptoKey) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(
            self.as_ref(),
            &JsValue::from("publicKey"),
            &JsValue::from(val),
        );
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
}
