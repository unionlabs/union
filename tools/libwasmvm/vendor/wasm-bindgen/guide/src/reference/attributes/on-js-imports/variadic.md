# Variadic Parameters

In javascript, both the types of function arguments, and the number of function arguments are
dynamic. For example

```js
function sum(...rest) {
    let i;
    // the old way
    let old_way = 0;
    for (i=0; i<arguments.length; i++) {
        old_way += arguments[i];
    }
    // the new way
    let new_way = 0;
    for (i=0; i<rest.length; i++) {
        new_way += rest[i];
    }
    // both give the same answer
    assert(old_way === new_way);
    return new_way;
}
```

This function doesn't translate directly into rust, since we don't currently support variadic
arguments on the wasm target. To bind to it, we use a slice as the last argument, and annotate the
function as variadic:

```rust
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(variadic)]
    fn sum(args: &[i32]) -> i32;
}
```

when we call this function, the last argument will be expanded as the javascript expects.


To export a rust function to javascript with a variadic argument, we will use the same bindgen variadic attribute and assume that the last argument will be the variadic array. For example the following rust function:

```rust
#[wasm_bindgen(variadic)]
pub  fn  variadic_function(arr: &JsValue) -> JsValue {
	arr.into()
}
```

will generate the following TS interface

```ts
export  function  variadic_function(...arr:  any):  any;
```
