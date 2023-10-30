#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = ChannelPixelLayout)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `ChannelPixelLayout` dictionary."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `ChannelPixelLayout`*"]
    pub type ChannelPixelLayout;
}
impl ChannelPixelLayout {
    #[cfg(feature = "ChannelPixelLayoutDataType")]
    #[doc = "Construct a new `ChannelPixelLayout`."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `ChannelPixelLayout`, `ChannelPixelLayoutDataType`*"]
    pub fn new(
        data_type: ChannelPixelLayoutDataType,
        height: u32,
        offset: u32,
        skip: u32,
        stride: u32,
        width: u32,
    ) -> Self {
        #[allow(unused_mut)]
        let mut ret: Self = ::wasm_bindgen::JsCast::unchecked_into(::js_sys::Object::new());
        ret.data_type(data_type);
        ret.height(height);
        ret.offset(offset);
        ret.skip(skip);
        ret.stride(stride);
        ret.width(width);
        ret
    }
    #[cfg(feature = "ChannelPixelLayoutDataType")]
    #[doc = "Change the `dataType` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `ChannelPixelLayout`, `ChannelPixelLayoutDataType`*"]
    pub fn data_type(&mut self, val: ChannelPixelLayoutDataType) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(
            self.as_ref(),
            &JsValue::from("dataType"),
            &JsValue::from(val),
        );
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
    #[doc = "Change the `height` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `ChannelPixelLayout`*"]
    pub fn height(&mut self, val: u32) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r =
            ::js_sys::Reflect::set(self.as_ref(), &JsValue::from("height"), &JsValue::from(val));
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
    #[doc = "Change the `offset` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `ChannelPixelLayout`*"]
    pub fn offset(&mut self, val: u32) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r =
            ::js_sys::Reflect::set(self.as_ref(), &JsValue::from("offset"), &JsValue::from(val));
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
    #[doc = "Change the `skip` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `ChannelPixelLayout`*"]
    pub fn skip(&mut self, val: u32) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(self.as_ref(), &JsValue::from("skip"), &JsValue::from(val));
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
    #[doc = "Change the `stride` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `ChannelPixelLayout`*"]
    pub fn stride(&mut self, val: u32) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r =
            ::js_sys::Reflect::set(self.as_ref(), &JsValue::from("stride"), &JsValue::from(val));
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
    #[doc = "Change the `width` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `ChannelPixelLayout`*"]
    pub fn width(&mut self, val: u32) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(self.as_ref(), &JsValue::from("width"), &JsValue::from(val));
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
}
