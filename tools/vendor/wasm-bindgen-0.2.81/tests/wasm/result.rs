use std::fmt;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

#[derive(Clone, Debug)]
pub enum MyError {
    Variant,
    InflightShouldBeFalse,
}
// shouldn't technically need this, surely
impl std::error::Error for MyError {}
impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MyError::{:?}", self)
    }
}

#[wasm_bindgen(module = "tests/wasm/result.js")]
extern "C" {
    fn error_new(message: &str) -> JsValue;
}

impl Into<JsValue> for MyError {
    fn into(self) -> JsValue {
        error_new(&format!("{}", self))
    }
}

macro_rules! call_test {
    ($test_fn:ident, $js_fn:ident) => {
        #[wasm_bindgen_test]
        fn $test_fn() {
            #[wasm_bindgen(module = "tests/wasm/result.js")]
            extern "C" {
                fn $js_fn();
            }
            $js_fn();
        }
    };
}

#[wasm_bindgen]
pub fn return_my_err() -> Result<i32, MyError> {
    let e = Err(MyError::Variant)?;
    Ok(e)
}
call_test!(test_err, call_err);

#[wasm_bindgen]
pub fn return_my_ok() -> Result<i32, MyError> {
    Ok(5)
}
call_test!(test_ok, call_ok);

#[wasm_bindgen]
pub struct Struct {
    inflight: bool,
}

call_test!(test_struct, all_struct_methods);

struct ResetOnDrop<'a> {
    flag: &'a mut bool,
}

impl<'a> Drop for ResetOnDrop<'a> {
    fn drop(&mut self) {
        *self.flag = false;
    }
}

impl<'a> ResetOnDrop<'a> {
    fn new(flag: &'a mut bool) -> Result<Self, MyError> {
        if *flag {
            return Err(MyError::InflightShouldBeFalse);
        }
        Ok(Self { flag })
    }
}

#[wasm_bindgen]
impl Struct {
    #[wasm_bindgen]
    pub fn new() -> Result<Struct, MyError> {
        Ok(Struct { inflight: false })
    }

    #[wasm_bindgen]
    pub fn new_err() -> Result<Struct, MyError> {
        Err(MyError::Variant.into())
    }

    #[wasm_bindgen]
    pub fn return_ok(&mut self) -> Result<i32, MyError> {
        let _guard = ResetOnDrop::new(&mut self.inflight)?;
        Ok(5)
    }

    #[wasm_bindgen]
    pub fn return_err(&mut self) -> Result<i32, MyError> {
        let guard = ResetOnDrop::new(&mut self.inflight)?;
        let err = Err(MyError::Variant);
        let nope = err?;
        // we are checking both for the flag being reset (from js, via is_inflight)
        // and for the running of drop code
        drop(guard);
        Ok(nope)
    }

    #[wasm_bindgen]
    pub fn is_inflight(&self) -> bool {
        self.inflight
    }
}

// check some more Ok types
#[wasm_bindgen]
pub fn return_string() -> Result<String, MyError> {
    Ok("string here".into())
}
call_test!(test_return_string, call_return_string);

// now we check that jsvalue works, as it did before

#[wasm_bindgen]
pub fn return_jsvalue_ok() -> Result<i32, JsValue> {
    Ok(5)
}
call_test!(test_jsvalue_ok, call_jsvalue_ok);

#[wasm_bindgen]
pub fn return_jsvalue_err() -> Result<i32, JsValue> {
    Err(JsValue::from(-1i32))
}
call_test!(test_jsvalue_err, call_jsvalue_err);

// test strings (they have a deferred free, in a finally block: tricky)
#[wasm_bindgen]
pub fn return_string_ok() -> Result<String, String> {
    Ok("Ok".into())
}
call_test!(test_string_ok, call_string_ok);
#[wasm_bindgen]
pub fn return_string_err() -> Result<String, String> {
    Err("Er".into())
}
call_test!(test_string_err, call_string_err);

// test enums
#[wasm_bindgen]
pub enum MyEnum {
    One = 1,
    Two = 2,
}
#[wasm_bindgen]
pub fn return_enum_ok() -> Result<MyEnum, MyError> {
    Ok(MyEnum::Two)
}
call_test!(test_enum_ok, call_enum_ok);
#[wasm_bindgen]
pub fn return_enum_err() -> Result<MyEnum, MyError> {
    Err(MyError::Variant)
}
call_test!(test_enum_err, call_enum_err);

// T = Unit
#[wasm_bindgen]
pub fn return_unit_ok() -> Result<(), MyError> {
    Ok(())
}
#[wasm_bindgen]
pub fn return_unit_err() -> Result<(), MyError> {
    Err(MyError::Variant)
}
call_test!(test_unit, call_unit);

// T = Option<f64>
#[wasm_bindgen]
pub fn return_option_ok_some() -> Result<Option<f64>, MyError> {
    Ok(Some(10f64))
}
#[wasm_bindgen]
pub fn return_option_ok_none() -> Result<Option<f64>, MyError> {
    Ok(None)
}
#[wasm_bindgen]
pub fn return_option_err() -> Result<Option<f64>, MyError> {
    Err(MyError::Variant)
}
call_test!(test_option, call_option);
