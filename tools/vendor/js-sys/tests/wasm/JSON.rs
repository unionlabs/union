use js_sys::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
fn parse_array() {
    let js_array = JSON::parse("[1, 2, 3]").unwrap();
    assert!(Array::is_array(&js_array));

    let array = Array::from(&js_array);
    assert_eq!(array.length(), 3);
    assert_eq!(array.pop(), 3);
    assert_eq!(array.pop(), 2);
    assert_eq!(array.pop(), 1);
}

#[wasm_bindgen_test]
fn parse_object() {
    let js_object = JSON::parse("{\"x\": 5, \"y\": true, \"z\": [\"foo\", \"bar\"]}").unwrap();
    assert!(js_object.is_object());

    let obj = Object::from(js_object);
    let keys = Object::keys(&obj);
    assert_eq!(keys.length(), 3);
    assert_eq!(keys.pop().as_string().unwrap(), "z");
    assert_eq!(keys.pop().as_string().unwrap(), "y");
    assert_eq!(keys.pop().as_string().unwrap(), "x");

    let values = Object::values(&obj);
    assert_eq!(values.length(), 3);

    let z = values.pop();
    assert!(Array::is_array(&z));
    let z_array = Array::from(&z);
    assert_eq!(z_array.length(), 2);

    let y = values.pop();
    assert_eq!(y.as_bool(), Some(true));

    let x = values.pop();
    assert_eq!(x.as_f64().unwrap(), 5.0);
}

#[wasm_bindgen_test]
fn parse_error() {
    let js_object = JSON::parse("invalid json");
    assert!(js_object.is_err());
    let err = js_object.unwrap_err();
    assert!(err.is_instance_of::<Error>());
}

#[wasm_bindgen_test]
fn stringify() {
    let arr = Array::new();
    arr.push(&JsValue::from(1));
    arr.push(&JsValue::from(true));
    arr.push(&JsValue::from("hello"));

    let str1: String = JSON::stringify(&JsValue::from(arr)).unwrap().into();
    assert_eq!(str1, "[1,true,\"hello\"]");

    let obj = Object::new();
    Reflect::set(obj.as_ref(), &JsValue::from("foo"), &JsValue::from("bar")).unwrap();
    let str2: String = JSON::stringify(&JsValue::from(obj)).unwrap().into();
    assert_eq!(str2, "{\"foo\":\"bar\"}");
}

#[wasm_bindgen_test]
fn stringify_error() {
    let func = Function::new_no_args("throw new Error(\"rust really rocks\")");
    let obj = Object::new();
    Reflect::set(obj.as_ref(), &JsValue::from("toJSON"), func.as_ref()).unwrap();

    let result = JSON::stringify(&JsValue::from(obj));
    assert!(result.is_err());
    let err_obj = result.unwrap_err();
    assert!(err_obj.is_instance_of::<Error>());
    let err: &Error = err_obj.dyn_ref().unwrap();
    let err_msg: String = From::from(err.message());
    assert!(err_msg.contains("rust really rocks"));
}

#[wasm_bindgen_test]
fn stringify_with_replacer() {
    let obj = Object::new();
    Reflect::set(obj.as_ref(), &JsValue::from("foo"), &JsValue::from("bar")).unwrap();
    Reflect::set(
        obj.as_ref(),
        &JsValue::from("hello"),
        &JsValue::from("world"),
    )
    .unwrap();

    let replacer_array = Array::new();
    replacer_array.push(&JsValue::from("hello"));
    let output1: String =
        JSON::stringify_with_replacer(&JsValue::from(obj.clone()), &JsValue::from(replacer_array))
            .unwrap()
            .into();
    assert_eq!(output1, "{\"hello\":\"world\"}");

    let replacer_func =
        Function::new_with_args("key, value", "return key === 'hello' ? undefined : value");
    let output2: String =
        JSON::stringify_with_replacer(&JsValue::from(obj), &JsValue::from(replacer_func))
            .unwrap()
            .into();
    assert_eq!(output2, "{\"foo\":\"bar\"}");
}

#[wasm_bindgen_test]
fn stringify_with_replacer_error() {
    let arr = Array::new();
    arr.push(&JsValue::from(1));
    arr.push(&JsValue::from(true));
    arr.push(&JsValue::from("hello"));

    let replacer = Function::new_no_args("throw new Error(\"rust really rocks\")");

    let result = JSON::stringify_with_replacer(&JsValue::from(arr), &JsValue::from(replacer));
    assert!(result.is_err());
    let err_obj = result.unwrap_err();
    assert!(err_obj.is_instance_of::<Error>());
    let err: &Error = err_obj.dyn_ref().unwrap();
    let err_msg: String = From::from(err.message());
    assert!(err_msg.contains("rust really rocks"));
}

#[wasm_bindgen_test]
fn stringify_with_replacer_and_space() {
    let arr = Array::new();
    arr.push(&JsValue::from(1));
    arr.push(&JsValue::from(true));
    arr.push(&JsValue::from("hello"));

    let output1: String = JSON::stringify_with_replacer_and_space(
        &JsValue::from(arr),
        &JsValue::NULL,
        &JsValue::from(4),
    )
    .unwrap()
    .into();
    assert_eq!(output1, "[\n    1,\n    true,\n    \"hello\"\n]");

    let obj = Object::new();
    Reflect::set(obj.as_ref(), &JsValue::from("foo"), &JsValue::from("bar")).unwrap();
    Reflect::set(
        obj.as_ref(),
        &JsValue::from("hello"),
        &JsValue::from("world"),
    )
    .unwrap();

    let replacer_array = Array::new();
    replacer_array.push(&JsValue::from("hello"));
    let output2: String = JSON::stringify_with_replacer_and_space(
        &JsValue::from(obj.clone()),
        &JsValue::from(replacer_array),
        &JsValue::from(4),
    )
    .unwrap()
    .into();
    assert_eq!(output2, "{\n    \"hello\": \"world\"\n}");

    let replacer_func =
        Function::new_with_args("key, value", "return key === 'hello' ? undefined : value");
    let output3: String = JSON::stringify_with_replacer_and_space(
        &JsValue::from(obj),
        &JsValue::from(replacer_func),
        &JsValue::from(4),
    )
    .unwrap()
    .into();
    assert_eq!(output3, "{\n    \"foo\": \"bar\"\n}");
}

#[wasm_bindgen_test]
fn stringify_with_replacer_and_space_error() {
    let arr = Array::new();
    arr.push(&JsValue::from(1));
    arr.push(&JsValue::from(true));
    arr.push(&JsValue::from("hello"));

    let replacer = Function::new_no_args("throw new Error(\"rust really rocks\")");

    let result = JSON::stringify_with_replacer_and_space(
        &JsValue::from(arr),
        &JsValue::from(replacer),
        &JsValue::from(4),
    );
    assert!(result.is_err());
    let err_obj = result.unwrap_err();
    assert!(err_obj.is_instance_of::<Error>());
    let err: &Error = err_obj.dyn_ref().unwrap();
    let err_msg: String = From::from(err.message());
    assert!(err_msg.contains("rust really rocks"));
}
