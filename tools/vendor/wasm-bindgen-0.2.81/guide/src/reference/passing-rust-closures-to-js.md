# Passing Rust Closures to Imported JavaScript Functions

The `#[wasm_bindgen]` attribute supports Rust closures being passed to
JavaScript in two variants:

1. Stack-lifetime closures that should not be invoked by JavaScript again after
   the imported JavaScript function that the closure was passed to returns.

2. Heap-allocated closures that can be invoked any number of times, but must be
   explicitly deallocated when finished.

## Stack-Lifetime Closures

Closures with a stack lifetime are passed to JavaScript as either `&dyn Fn` or `&mut
dyn FnMut` trait objects:

```rust
// Import JS functions that take closures

#[wasm_bindgen]
extern "C" {
    fn takes_immutable_closure(f: &dyn Fn());

    fn takes_mutable_closure(f: &mut dyn FnMut());
}

// Usage

takes_immutable_closure(&|| {
    // ...
});

let mut times_called = 0;
takes_mutable_closure(&mut || {
    times_called += 1;
});
```

**Once these imported functions return, the closures that were given to them
will become invalidated, and any future attempts to call those closures from
JavaScript will raise an exception.**

Closures also support arguments and return values like exports do, for example:

```rust
#[wasm_bindgen]
extern "C" {
    fn takes_closure_that_takes_int_and_returns_string(x: &dyn Fn(u32) -> String);
}

takes_closure_that_takes_int_and_returns_string(&|x: u32| -> String {
    format!("x is {}", x)
});
```

## Heap-Allocated Closures

Sometimes the discipline of stack-lifetime closures is not desired. For example,
you'd like to schedule a closure to be run on the next turn of the event loop in
JavaScript through `setTimeout`. For this, you want the imported function to
return but the JavaScript closure still needs to be valid!

For this scenario, you need the `Closure` type, which is defined in the
`wasm_bindgen` crate, exported in `wasm_bindgen::prelude`, and represents a
"long lived" closure.
The `Closure` type is currently behind a feature which needs to be enabled:

```toml
[dependencies]
wasm-bindgen = {version = "^0.2", features = ["nightly"]}
```

The validity of the JavaScript closure is tied to the lifetime of the `Closure`
in Rust. **Once a `Closure` is dropped, it will deallocate its internal memory
and invalidate the corresponding JavaScript function so that any further
attempts to invoke it raise an exception.**

Like stack closures a `Closure` supports both `Fn` and `FnMut` closures, as well
as arguments and returns.

```rust
#[wasm_bindgen]
extern "C" {
    fn setInterval(closure: &Closure<dyn FnMut()>, millis: u32) -> f64;
    fn cancelInterval(token: f64);

    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub struct Interval {
    closure: Closure<dyn FnMut()>,
    token: f64,
}

impl Interval {
    pub fn new<F: 'static>(millis: u32, f: F) -> Interval
    where
        F: FnMut()
    {
        // Construct a new closure.
        let closure = Closure::new(f);

        // Pass the closure to JS, to run every n milliseconds.
        let token = setInterval(&closure, millis);

        Interval { closure, token }
    }
}

// When the Interval is destroyed, cancel its `setInterval` timer.
impl Drop for Interval {
    fn drop(&mut self) {
        cancelInterval(self.token);
    }
}

// Keep logging "hello" every second until the resulting `Interval` is dropped.
#[wasm_bindgen]
pub fn hello() -> Interval {
    Interval::new(1_000, || log("hello"))
}
```
