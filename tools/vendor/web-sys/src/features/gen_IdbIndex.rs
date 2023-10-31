#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = IDBIndex , typescript_type = "IDBIndex")]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `IdbIndex` class."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/IDBIndex)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `IdbIndex`*"]
    pub type IdbIndex;
    # [wasm_bindgen (structural , method , getter , js_class = "IDBIndex" , js_name = name)]
    #[doc = "Getter for the `name` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/IDBIndex/name)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `IdbIndex`*"]
    pub fn name(this: &IdbIndex) -> String;
    # [wasm_bindgen (structural , method , setter , js_class = "IDBIndex" , js_name = name)]
    #[doc = "Setter for the `name` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/IDBIndex/name)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `IdbIndex`*"]
    pub fn set_name(this: &IdbIndex, value: &str);
    #[cfg(feature = "IdbObjectStore")]
    # [wasm_bindgen (structural , method , getter , js_class = "IDBIndex" , js_name = objectStore)]
    #[doc = "Getter for the `objectStore` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/IDBIndex/objectStore)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `IdbIndex`, `IdbObjectStore`*"]
    pub fn object_store(this: &IdbIndex) -> IdbObjectStore;
    # [wasm_bindgen (structural , catch , method , getter , js_class = "IDBIndex" , js_name = keyPath)]
    #[doc = "Getter for the `keyPath` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/IDBIndex/keyPath)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `IdbIndex`*"]
    pub fn key_path(this: &IdbIndex) -> Result<::wasm_bindgen::JsValue, JsValue>;
    # [wasm_bindgen (structural , method , getter , js_class = "IDBIndex" , js_name = multiEntry)]
    #[doc = "Getter for the `multiEntry` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/IDBIndex/multiEntry)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `IdbIndex`*"]
    pub fn multi_entry(this: &IdbIndex) -> bool;
    # [wasm_bindgen (structural , method , getter , js_class = "IDBIndex" , js_name = unique)]
    #[doc = "Getter for the `unique` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/IDBIndex/unique)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `IdbIndex`*"]
    pub fn unique(this: &IdbIndex) -> bool;
    # [wasm_bindgen (structural , method , getter , js_class = "IDBIndex" , js_name = locale)]
    #[doc = "Getter for the `locale` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/IDBIndex/locale)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `IdbIndex`*"]
    pub fn locale(this: &IdbIndex) -> Option<String>;
    # [wasm_bindgen (structural , method , getter , js_class = "IDBIndex" , js_name = isAutoLocale)]
    #[doc = "Getter for the `isAutoLocale` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/IDBIndex/isAutoLocale)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `IdbIndex`*"]
    pub fn is_auto_locale(this: &IdbIndex) -> bool;
    #[cfg(feature = "IdbRequest")]
    # [wasm_bindgen (catch , method , structural , js_class = "IDBIndex" , js_name = count)]
    #[doc = "The `count()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/IDBIndex/count)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `IdbIndex`, `IdbRequest`*"]
    pub fn count(this: &IdbIndex) -> Result<IdbRequest, JsValue>;
    #[cfg(feature = "IdbRequest")]
    # [wasm_bindgen (catch , method , structural , js_class = "IDBIndex" , js_name = count)]
    #[doc = "The `count()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/IDBIndex/count)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `IdbIndex`, `IdbRequest`*"]
    pub fn count_with_key(
        this: &IdbIndex,
        key: &::wasm_bindgen::JsValue,
    ) -> Result<IdbRequest, JsValue>;
    #[cfg(feature = "IdbRequest")]
    # [wasm_bindgen (catch , method , structural , js_class = "IDBIndex" , js_name = get)]
    #[doc = "The `get()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/IDBIndex/get)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `IdbIndex`, `IdbRequest`*"]
    pub fn get(this: &IdbIndex, key: &::wasm_bindgen::JsValue) -> Result<IdbRequest, JsValue>;
    #[cfg(feature = "IdbRequest")]
    # [wasm_bindgen (catch , method , structural , js_class = "IDBIndex" , js_name = getAll)]
    #[doc = "The `getAll()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/IDBIndex/getAll)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `IdbIndex`, `IdbRequest`*"]
    pub fn get_all(this: &IdbIndex) -> Result<IdbRequest, JsValue>;
    #[cfg(feature = "IdbRequest")]
    # [wasm_bindgen (catch , method , structural , js_class = "IDBIndex" , js_name = getAll)]
    #[doc = "The `getAll()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/IDBIndex/getAll)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `IdbIndex`, `IdbRequest`*"]
    pub fn get_all_with_key(
        this: &IdbIndex,
        key: &::wasm_bindgen::JsValue,
    ) -> Result<IdbRequest, JsValue>;
    #[cfg(feature = "IdbRequest")]
    # [wasm_bindgen (catch , method , structural , js_class = "IDBIndex" , js_name = getAll)]
    #[doc = "The `getAll()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/IDBIndex/getAll)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `IdbIndex`, `IdbRequest`*"]
    pub fn get_all_with_key_and_limit(
        this: &IdbIndex,
        key: &::wasm_bindgen::JsValue,
        limit: u32,
    ) -> Result<IdbRequest, JsValue>;
    #[cfg(feature = "IdbRequest")]
    # [wasm_bindgen (catch , method , structural , js_class = "IDBIndex" , js_name = getAllKeys)]
    #[doc = "The `getAllKeys()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/IDBIndex/getAllKeys)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `IdbIndex`, `IdbRequest`*"]
    pub fn get_all_keys(this: &IdbIndex) -> Result<IdbRequest, JsValue>;
    #[cfg(feature = "IdbRequest")]
    # [wasm_bindgen (catch , method , structural , js_class = "IDBIndex" , js_name = getAllKeys)]
    #[doc = "The `getAllKeys()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/IDBIndex/getAllKeys)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `IdbIndex`, `IdbRequest`*"]
    pub fn get_all_keys_with_key(
        this: &IdbIndex,
        key: &::wasm_bindgen::JsValue,
    ) -> Result<IdbRequest, JsValue>;
    #[cfg(feature = "IdbRequest")]
    # [wasm_bindgen (catch , method , structural , js_class = "IDBIndex" , js_name = getAllKeys)]
    #[doc = "The `getAllKeys()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/IDBIndex/getAllKeys)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `IdbIndex`, `IdbRequest`*"]
    pub fn get_all_keys_with_key_and_limit(
        this: &IdbIndex,
        key: &::wasm_bindgen::JsValue,
        limit: u32,
    ) -> Result<IdbRequest, JsValue>;
    #[cfg(feature = "IdbRequest")]
    # [wasm_bindgen (catch , method , structural , js_class = "IDBIndex" , js_name = getKey)]
    #[doc = "The `getKey()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/IDBIndex/getKey)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `IdbIndex`, `IdbRequest`*"]
    pub fn get_key(this: &IdbIndex, key: &::wasm_bindgen::JsValue) -> Result<IdbRequest, JsValue>;
    #[cfg(feature = "IdbRequest")]
    # [wasm_bindgen (catch , method , structural , js_class = "IDBIndex" , js_name = openCursor)]
    #[doc = "The `openCursor()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/IDBIndex/openCursor)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `IdbIndex`, `IdbRequest`*"]
    pub fn open_cursor(this: &IdbIndex) -> Result<IdbRequest, JsValue>;
    #[cfg(feature = "IdbRequest")]
    # [wasm_bindgen (catch , method , structural , js_class = "IDBIndex" , js_name = openCursor)]
    #[doc = "The `openCursor()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/IDBIndex/openCursor)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `IdbIndex`, `IdbRequest`*"]
    pub fn open_cursor_with_range(
        this: &IdbIndex,
        range: &::wasm_bindgen::JsValue,
    ) -> Result<IdbRequest, JsValue>;
    #[cfg(all(feature = "IdbCursorDirection", feature = "IdbRequest",))]
    # [wasm_bindgen (catch , method , structural , js_class = "IDBIndex" , js_name = openCursor)]
    #[doc = "The `openCursor()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/IDBIndex/openCursor)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `IdbCursorDirection`, `IdbIndex`, `IdbRequest`*"]
    pub fn open_cursor_with_range_and_direction(
        this: &IdbIndex,
        range: &::wasm_bindgen::JsValue,
        direction: IdbCursorDirection,
    ) -> Result<IdbRequest, JsValue>;
    #[cfg(feature = "IdbRequest")]
    # [wasm_bindgen (catch , method , structural , js_class = "IDBIndex" , js_name = openKeyCursor)]
    #[doc = "The `openKeyCursor()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/IDBIndex/openKeyCursor)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `IdbIndex`, `IdbRequest`*"]
    pub fn open_key_cursor(this: &IdbIndex) -> Result<IdbRequest, JsValue>;
    #[cfg(feature = "IdbRequest")]
    # [wasm_bindgen (catch , method , structural , js_class = "IDBIndex" , js_name = openKeyCursor)]
    #[doc = "The `openKeyCursor()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/IDBIndex/openKeyCursor)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `IdbIndex`, `IdbRequest`*"]
    pub fn open_key_cursor_with_range(
        this: &IdbIndex,
        range: &::wasm_bindgen::JsValue,
    ) -> Result<IdbRequest, JsValue>;
    #[cfg(all(feature = "IdbCursorDirection", feature = "IdbRequest",))]
    # [wasm_bindgen (catch , method , structural , js_class = "IDBIndex" , js_name = openKeyCursor)]
    #[doc = "The `openKeyCursor()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/IDBIndex/openKeyCursor)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `IdbCursorDirection`, `IdbIndex`, `IdbRequest`*"]
    pub fn open_key_cursor_with_range_and_direction(
        this: &IdbIndex,
        range: &::wasm_bindgen::JsValue,
        direction: IdbCursorDirection,
    ) -> Result<IdbRequest, JsValue>;
}
