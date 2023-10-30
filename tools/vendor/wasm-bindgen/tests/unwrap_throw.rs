extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;

#[test]
fn unwrap_throw_ok() {
    assert_eq!(Some(42).unwrap_throw(), 42);
    let x: Result<i32, ()> = Ok(42);
    assert_eq!(x.unwrap_throw(), 42);
}

#[test]
#[should_panic]
fn unwrap_throw_none() {
    let x: Option<i32> = None;
    x.unwrap_throw();
}

#[test]
#[should_panic]
fn unwrap_throw_err() {
    let x: Result<i32, ()> = Err(());
    x.unwrap_throw();
}
