use js_sys::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;

#[wasm_bindgen(module = "tests/wasm/WebAssembly.js")]
extern "C" {
    #[wasm_bindgen(js_name = getWasmArray)]
    fn get_wasm_array() -> Uint8Array;

    #[wasm_bindgen(js_name = getTableObject)]
    fn get_table_object() -> Object;

    #[wasm_bindgen(js_name = getInvalidTableObject)]
    fn get_invalid_table_object() -> Object;

    #[wasm_bindgen(js_name = getImports)]
    fn get_imports() -> Object;
}

fn get_invalid_wasm() -> JsValue {
    ArrayBuffer::new(42).into()
}

fn get_bad_type_wasm() -> JsValue {
    2.into()
}

fn get_valid_wasm() -> JsValue {
    get_wasm_array().into()
}

#[wasm_bindgen_test]
fn validate() {
    assert!(!WebAssembly::validate(&get_invalid_wasm()).unwrap());

    assert!(WebAssembly::validate(&get_bad_type_wasm()).is_err());
}

#[wasm_bindgen_test]
async fn compile_compile_error() {
    let p = WebAssembly::compile(&get_invalid_wasm());
    let e = JsFuture::from(p).await.unwrap_err();
    assert!(e.is_instance_of::<WebAssembly::CompileError>());
}

#[wasm_bindgen_test]
async fn compile_type_error() {
    let p = WebAssembly::compile(&get_bad_type_wasm());
    let e = JsFuture::from(p).await.unwrap_err();
    assert!(e.is_instance_of::<TypeError>());
}

#[wasm_bindgen_test]
async fn compile_valid() {
    let p = WebAssembly::compile(&get_valid_wasm());
    let module = JsFuture::from(p).await.unwrap();
    assert!(module.is_instance_of::<WebAssembly::Module>());
}

#[wasm_bindgen_test]
fn module_inheritance() {
    let module = WebAssembly::Module::new(&get_valid_wasm()).unwrap();
    assert!(module.is_instance_of::<WebAssembly::Module>());
    assert!(module.is_instance_of::<Object>());

    let _: &Object = module.as_ref();
}

#[wasm_bindgen_test]
fn module_error() {
    let error = WebAssembly::Module::new(&get_invalid_wasm()).err().unwrap();
    assert!(error.is_instance_of::<WebAssembly::CompileError>());

    let error = WebAssembly::Module::new(&get_bad_type_wasm())
        .err()
        .unwrap();
    assert!(error.is_instance_of::<TypeError>());
}

#[wasm_bindgen_test]
fn module_custom_sections() {
    let module = WebAssembly::Module::new(&get_valid_wasm()).unwrap();
    let cust_sec = WebAssembly::Module::custom_sections(&module, "abcd");
    assert_eq!(cust_sec.length(), 0);
}

#[wasm_bindgen_test]
fn module_exports() {
    let module = WebAssembly::Module::new(&get_valid_wasm()).unwrap();
    let exports = WebAssembly::Module::exports(&module);
    assert_eq!(exports.length(), 1);
}

#[wasm_bindgen_test]
fn module_imports() {
    let module = WebAssembly::Module::new(&get_valid_wasm()).unwrap();
    let imports = WebAssembly::Module::imports(&module);
    assert_eq!(imports.length(), 1);
}

#[wasm_bindgen_test]
fn table_inheritance() {
    let table = WebAssembly::Table::new(&get_table_object().into()).unwrap();
    assert!(table.is_instance_of::<WebAssembly::Table>());
    assert!(table.is_instance_of::<Object>());

    let _: &Object = table.as_ref();
}

#[wasm_bindgen_test]
fn table_error() {
    let error = WebAssembly::Table::new(&get_invalid_table_object())
        .err()
        .unwrap();
    assert!(error.is_instance_of::<RangeError>());
}

#[wasm_bindgen_test]
fn table() {
    let table = WebAssembly::Table::new(&get_table_object().into()).unwrap();
    assert_eq!(table.length(), 1);

    assert!(table.get(0).is_ok());
    assert!(table.get(999).is_err());

    table.grow(1).unwrap();
    assert_eq!(table.length(), 2);

    let f = table.get(0).unwrap();
    table.set(1, &f).unwrap();
}

#[wasm_bindgen_test]
fn compile_error_inheritance() {
    let error = WebAssembly::CompileError::new("");
    assert!(error.is_instance_of::<WebAssembly::CompileError>());
    assert!(error.is_instance_of::<Error>());

    let _: &Error = error.as_ref();
}

#[wasm_bindgen_test]
fn link_error_inheritance() {
    let error = WebAssembly::LinkError::new("");
    assert!(error.is_instance_of::<WebAssembly::LinkError>());
    assert!(error.is_instance_of::<Error>());

    let _: &Error = error.as_ref();
}

#[wasm_bindgen_test]
fn runtime_error_inheritance() {
    let error = WebAssembly::RuntimeError::new("");
    assert!(error.is_instance_of::<WebAssembly::RuntimeError>());
    assert!(error.is_instance_of::<Error>());

    let _: &Error = error.as_ref();
}

#[wasm_bindgen_test]
fn webassembly_instance() {
    let module = WebAssembly::Module::new(&get_valid_wasm()).unwrap();
    let imports = get_imports();
    let instance = WebAssembly::Instance::new(&module, &imports).unwrap();

    // Inheritance chain is correct.
    assert!(instance.is_instance_of::<WebAssembly::Instance>());
    assert!(instance.is_instance_of::<Object>());
    let _: &Object = instance.as_ref();

    // Has expected exports.
    let exports = instance.exports();
    assert!(Reflect::has(exports.as_ref(), &"exported_func".into()).unwrap());
}

#[wasm_bindgen_test]
async fn instantiate_module() {
    let module = WebAssembly::Module::new(&get_valid_wasm()).unwrap();
    let imports = get_imports();
    let p = WebAssembly::instantiate_module(&module, &imports);
    let inst = JsFuture::from(p).await.unwrap();
    assert!(inst.is_instance_of::<WebAssembly::Instance>());
}

#[wasm_bindgen_test]
fn memory_works() {
    let obj = Object::new();
    Reflect::set(obj.as_ref(), &"initial".into(), &1.into()).unwrap();
    let mem = WebAssembly::Memory::new(&obj).unwrap();
    assert!(mem.is_instance_of::<WebAssembly::Memory>());
    assert!(mem.is_instance_of::<Object>());
    assert!(mem.buffer().is_instance_of::<ArrayBuffer>());
    assert_eq!(mem.grow(1), 1);
    assert_eq!(mem.grow(2), 2);
    assert_eq!(mem.grow(3), 4);
    assert_eq!(
        mem.buffer()
            .dyn_into::<ArrayBuffer>()
            .unwrap()
            .byte_length(),
        7 * 64 * 1024,
    );
}
