# Iterating over JavaScript Values

## Methods That Return `js_sys::Iterator`

Some JavaScript collections have methods for iterating over their values or
keys:

* [`Map::values`](https://rustwasm.github.io/wasm-bindgen/api/js_sys/struct.Map.html#method.values)
* [`Set::keys`](https://rustwasm.github.io/wasm-bindgen/api/js_sys/struct.Set.html#method.keys)
* etc...

These methods return
[`js_sys::Iterator`](https://rustwasm.github.io/wasm-bindgen/api/js_sys/struct.Iterator.html),
which is the Rust representation of a JavaScript object that has a `next` method
that either returns the next item in the iteration, notes that iteration has
completed, or throws an error. That is, `js_sys::Iterator` represents an object
that implements [the duck-typed JavaScript iteration
protocol](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Iteration_protocols).

`js_sys::Iterator` can be converted into a Rust iterator either by reference
(into
[`js_sys::Iter<'a>`](https://rustwasm.github.io/wasm-bindgen/api/js_sys/struct.Iter.html))
or by value (into
[`js_sys::IntoIter`](https://rustwasm.github.io/wasm-bindgen/api/js_sys/struct.IntoIter.html)). The
Rust iterator will yield items of type `Result<JsValue>`. If it yields an
`Ok(...)`, then the JS iterator protocol returned an element. If it yields an
`Err(...)`, then the JS iterator protocol threw an exception.

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn count_strings_in_set(set: &js_sys::Set) -> u32 {
    let mut count = 0;

    // Call `keys` to get an iterator over the set's elements. Because this is
    // in a `for ... in ...` loop, Rust will automatically call its
    // `IntoIterator` trait implementation to convert it into a Rust iterator.
    for x in set.keys() {
        // We know the built-in iterator for set elements won't throw
        // exceptions, so just unwrap the element. If this was an untrusted
        // iterator, we might want to explicitly handle the case where it throws
        // an exception instead of returning a `{ value, done }` object.
        let x = x.unwrap();

        // If `x` is a string, increment our count of strings in the set!
        if x.is_string() {
            count += 1;
        }
    }

    count
}
```

## Iterating Over <u>Any</u> JavaScript Object that Implements the Iterator Protocol

You could manually test for whether an object implements JS's duck-typed
iterator protocol, and if so, convert it into a `js_sys::Iterator` that you can
finally iterate over. You don't need to do this by-hand, however, since we
bundled this up as [the `js_sys::try_iter`
function!](https://rustwasm.github.io/wasm-bindgen/api/js_sys/fn.try_iter.html)

For example, we can write a function that collects the numbers from any JS
iterable and returns them as an `Array`:

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn collect_numbers(some_iterable: &JsValue) -> Result<js_sys::Array, JsValue> {
    let nums = js_sys::Array::new();

    let iterator = js_sys::try_iter(some_iterable)?.ok_or_else(|| {
        "need to pass iterable JS values!"
    })?;

    for x in iterator {
        // If the iterator's `next` method throws an error, propagate it
        // up to the caller.
        let x = x?;

        // If `x` is a number, add it to our array of numbers!
        if x.as_f64().is_some() {
            nums.push(&x);
        }
    }

    Ok(nums)
}
```
