# Accessing Properties of Untyped JavaScript Values

To read and write arbitrary properties from any untyped JavaScript value
regardless if it is an `instanceof` some JavaScript class or not, use [the
`js_sys::Reflect` APIs][js-sys-reflect]. These APIs are bindings to the
[JavaScript builtin `Reflect` object][mdn-reflect] and its methods.

You might also benefit from [using duck-typed
interfaces](./working-with-duck-typed-interfaces.html) instead of working with
untyped values.

## Reading Properties with `js_sys::Reflect::get`

[API documentation for `js_sys::Reflect::get`.](https://docs.rs/js-sys/0.3.39/js_sys/Reflect/fn.get.html)

A function that returns the value of a property.

#### Rust Usage

```rust
let value = js_sys::Reflect::get(&target, &property_key)?;
```

#### JavaScript Equivalent

```js
let value = target[property_key];
```

## Writing Properties with `js_sys::Reflect::set`

[API documentation for `js_sys::Reflect::set`.](https://docs.rs/js-sys/0.3.39/js_sys/Reflect/fn.set.html)

A function that assigns a value to a property. Returns a boolean that is true if
the update was successful.

#### Rust Usage

```rust
js_sys::Reflect::set(&target, &property_key, &value)?;
```

#### JavaScript Equivalent

```js
target[property_key] = value;
```

## Determining if a Property Exists with `js_sys::Reflect::has`

[API documentation for `js_sys::Reflect::has`.](https://docs.rs/js-sys/0.3.39/js_sys/Reflect/fn.has.html)

The JavaScript `in` operator as function. Returns a boolean indicating whether
an own or inherited property exists on the target.

#### Rust Usage

```rust
if js_sys::Reflect::has(&target, &property_key)? {
    // ...
} else {
    // ...
}
```

#### JavaScript Equivalent

```js
if (property_key in target) {
    // ...
} else {
    // ...
}
```

## But wait — there's more!

See [the `js_sys::Reflect` API documentation][js-sys-reflect] for the full
listing of JavaScript value reflection and introspection capabilities.

[js-sys-reflect]: https://docs.rs/js-sys/latest/js_sys/Reflect/index.html
[mdn-reflect]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Reflect
