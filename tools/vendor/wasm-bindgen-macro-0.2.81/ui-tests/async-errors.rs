#![allow(unreachable_code)]
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct MyType;

#[wasm_bindgen]
pub async fn good1() { loop {} }
#[wasm_bindgen]
pub async fn good2() -> JsValue { loop {} }
#[wasm_bindgen]
pub async fn good3() -> u32 { loop {} }
#[wasm_bindgen]
pub async fn good4() -> MyType { loop {} }
#[wasm_bindgen]
pub async fn good5() -> Result<(), JsValue> { loop {} }
#[wasm_bindgen]
pub async fn good6() -> Result<JsValue, JsValue> { loop {} }
#[wasm_bindgen]
pub async fn good7() -> Result<u32, JsValue> { loop {} }
#[wasm_bindgen]
pub async fn good8() -> Result<MyType, JsValue> { loop {} }
#[wasm_bindgen]
pub async fn good9() -> Result<MyType, u32> { loop {} }
#[wasm_bindgen]
pub async fn good10() -> Result<MyType, MyType> { loop {} }

pub struct BadType;

#[wasm_bindgen]
pub async fn bad1() -> Result<(), ()> { loop {} }
#[wasm_bindgen]
pub async fn bad2() -> Result<(), BadType> { loop {} }
#[wasm_bindgen]
pub async fn bad3() -> BadType { loop {} }
#[wasm_bindgen]
pub async fn bad4() -> Result<BadType, JsValue> { loop {} }


fn main() {}
