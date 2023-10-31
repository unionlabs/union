#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = DataTransferItem , typescript_type = "DataTransferItem")]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `DataTransferItem` class."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/DataTransferItem)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `DataTransferItem`*"]
    pub type DataTransferItem;
    # [wasm_bindgen (structural , method , getter , js_class = "DataTransferItem" , js_name = kind)]
    #[doc = "Getter for the `kind` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/DataTransferItem/kind)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `DataTransferItem`*"]
    pub fn kind(this: &DataTransferItem) -> String;
    # [wasm_bindgen (structural , method , getter , js_class = "DataTransferItem" , js_name = type)]
    #[doc = "Getter for the `type` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/DataTransferItem/type)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `DataTransferItem`*"]
    pub fn type_(this: &DataTransferItem) -> String;
    #[cfg(feature = "File")]
    # [wasm_bindgen (catch , method , structural , js_class = "DataTransferItem" , js_name = getAsFile)]
    #[doc = "The `getAsFile()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/DataTransferItem/getAsFile)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `DataTransferItem`, `File`*"]
    pub fn get_as_file(this: &DataTransferItem) -> Result<Option<File>, JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "DataTransferItem" , js_name = getAsString)]
    #[doc = "The `getAsString()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/DataTransferItem/getAsString)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `DataTransferItem`*"]
    pub fn get_as_string(
        this: &DataTransferItem,
        callback: Option<&::js_sys::Function>,
    ) -> Result<(), JsValue>;
    #[cfg(feature = "FileSystemEntry")]
    # [wasm_bindgen (catch , method , structural , js_class = "DataTransferItem" , js_name = webkitGetAsEntry)]
    #[doc = "The `webkitGetAsEntry()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/DataTransferItem/webkitGetAsEntry)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `DataTransferItem`, `FileSystemEntry`*"]
    pub fn webkit_get_as_entry(this: &DataTransferItem)
        -> Result<Option<FileSystemEntry>, JsValue>;
}
