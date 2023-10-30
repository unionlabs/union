# `final`

The `final` attribute is the converse of the [`structural`
attribute](structural.html). It configures how `wasm-bindgen` will generate JS
imports to call the imported function. Notably a function imported by `final`
never changes after it was imported, whereas a function imported by default (or
with `structural`) is subject to runtime lookup rules such as walking the
prototype chain of an object. Note that `final` is not suitable for accessing
data descriptor properties of JS objects; to accomplish this, use the `structural`
attribute.

[host-bindings]: https://github.com/WebAssembly/host-bindings
[reference-types]: https://github.com/WebAssembly/reference-types

The `final` attribute is intended to be purely related to performance. It
ideally has no user-visible effect, and `structural` imports (the default)
should be able to transparently switch to `final` eventually.

The eventual performance aspect is that with the [host bindings
proposal][host-bindings] then `wasm-bindgen` will need to generate far fewer JS
function shims to import than it does today. For example, consider this import
today:

```rust
#[wasm_bindgen]
extern "C" {
    type Foo;
    #[wasm_bindgen(method)]
    fn bar(this: &Foo, argument: &str) -> JsValue;
}
```

**Without the `final` attribute** the generated JS looks like this:

```js
// without `final`
export function __wbg_bar_a81456386e6b526f(arg0, arg1, arg2) {
    let varg1 = getStringFromWasm(arg1, arg2);
    return addHeapObject(getObject(arg0).bar(varg1));
}
```

We can see here that this JS function shim is required, but it's all relatively
self-contained. It does, however, execute the `bar` method in a duck-type-y
fashion in the sense that it never validates `getObject(arg0)` is of type `Foo`
to actually call the `Foo.prototype.bar` method.

If we instead, however, write this:

```rust
#[wasm_bindgen]
extern "C" {
    type Foo;
    #[wasm_bindgen(method, final)] // note the change here
    fn bar(this: &Foo, argument: &str) -> JsValue;
}
```

it generates this JS glue (roughly):

```js
const __wbg_bar_target = Foo.prototype.bar;

export function __wbg_bar_a81456386e6b526f(arg0, arg1, arg2) {
    let varg1 = getStringFromWasm(arg1, arg2);
    return addHeapObject(__wbg_bar_target.call(getObject(arg0), varg1));
}
```

The difference here is pretty subtle, but we can see how the function being
called is hoisted out of the generated shim and is bound to always be
`Foo.prototype.bar`. This then uses the `Function.call` method to invoke that
function with `getObject(arg0)` as the receiver.

But wait, there's still a JS function shim here even with `final`! That's true,
and this is simply a fact of future WebAssembly proposals not being implemented
yet. The semantics, though, match the future [host bindings
proposal][host-bindings] because the method being called is determined exactly
once, and it's located on the prototype chain rather than being resolved at
runtime when the function is called.

## Interaction with future proposals

If you're curious to see how our JS function shim will be eliminated entirely,
let's take a look at the generated bindings. We're starting off with this:

```js
const __wbg_bar_target = Foo.prototype.bar;

export function __wbg_bar_a81456386e6b526f(arg0, arg1, arg2) {
    let varg1 = getStringFromWasm(arg1, arg2);
    return addHeapObject(__wbg_bar_target.call(getObject(arg0), varg1));
}
```

... and once the [reference types proposal][reference-types] is implemented then
we won't need some of these pesky functions. That'll transform our generated JS
shim to look like:

```js
const __wbg_bar_target = Foo.prototype.bar;

export function __wbg_bar_a81456386e6b526f(arg0, arg1, arg2) {
    let varg1 = getStringFromWasm(arg1, arg2);
    return __wbg_bar_target.call(arg0, varg1);
}
```

Getting better! Next up we need the host bindings proposal. Note that the
proposal is undergoing some changes right now so it's tough to link to reference
documentation, but it suffices to say that it'll empower us with at least two
different features.

First, host bindings promises to provide the concept of "argument conversions".
The `arg1` and `arg2` values here are actually a pointer and a length to a utf-8
encoded string, and with host bindings we'll be able to annotate that this
import should take those two arguments and convert them to a JS string (that is,
the *host* should do this, the WebAssembly engine). Using that feature we can
further trim this down to:

```js
const __wbg_bar_target = Foo.prototype.bar;

export function __wbg_bar_a81456386e6b526f(arg0, varg1) {
    return __wbg_bar_target.call(arg0, varg1);
}
```

And finally, the second promise of the host bindings proposal is that we can
flag a function call to indicate the first argument is the `this` binding of the
function call. Today the `this` value of all called imported functions is
`undefined`, and this flag (configured with host bindings) will indicate the
first argument here is actually the `this`.

With that in mind we can further transform this to:

```js
export const __wbg_bar_a81456386e6b526f = Foo.prototype.bar;
```

and voila! We, with [reference types][reference-types] and [host
bindings][host-bindings], now have no JS function shim at all necessary to call
the imported function. Additionally future wasm proposals to the ES module
system may also mean that don't even need the `export const ...` here too.

It's also worth pointing out that with all these wasm proposals implemented the
default way to import the `bar` function (aka `structural`) would generate a JS
function shim that looks like:

```js
export function __wbg_bar_a81456386e6b526f(varg1) {
    return this.bar(varg1);
}
```

where this import is still subject to runtime prototype chain lookups and such.
