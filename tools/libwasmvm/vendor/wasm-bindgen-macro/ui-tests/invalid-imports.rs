use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    type A;

    fn f() -> &'static u32;

    #[wasm_bindgen(method)]
    fn f1();
    #[wasm_bindgen(method)]
    fn f2(x: u32);
    #[wasm_bindgen(method)]
    fn f3(x: &&u32);
    #[wasm_bindgen(method)]
    fn f4(x: &foo::Bar);
    #[wasm_bindgen(method)]
    fn f4(x: &::Bar);
    #[wasm_bindgen(method)]
    fn f4(x: &Bar<T>);
    #[wasm_bindgen(method)]
    fn f4(x: &dyn Fn(T));

    #[wasm_bindgen(constructor)]
    fn f();
    #[wasm_bindgen(constructor)]
    fn f() -> ::Bar;
    #[wasm_bindgen(constructor)]
    fn f() -> &Bar;

    #[wasm_bindgen(catch)]
    fn f() -> u32;
    #[wasm_bindgen(catch)]
    fn f() -> &u32;
    #[wasm_bindgen(catch)]
    fn f() -> Result<>;
    #[wasm_bindgen(catch)]
    fn f() -> Result<'a>;
}

fn main() {}
