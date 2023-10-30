# Inheritance in `web-sys`

Inheritance between JS classes is the bread and butter of how the DOM works on
the web, and as a result it's quite important for `web-sys` to provide access to
this inheritance hierarchy as well! There are few ways you can access the
inheritance hierarchy when using `web-sys`.

### Accessing parent classes using `Deref`

Like smart pointers in Rust, all types in `web_sys` implement `Deref` to their
parent JS class. This means, for example, if you have a `web_sys::Element` you
can create a `web_sys::Node` from that implicitly:

```rust
let element: &Element = ...;

element.append_child(..); // call a method on `Node`

method_expecting_a_node(&element); // coerce to `&Node` implicitly

let node: &Node = &element; // explicitly coerce to `&Node`
```

Using `Deref` allows ergonomic transitioning up the inheritance hierarchy to the
parent class and beyond, giving you access to all the methods using the `.`
operator.

### Accessing parent classes using `AsRef`

In addition to `Deref`, the `AsRef` trait is implemented for all types in
`web_sys` for all types in the inheritance hierarchy. For example for the
`HtmlAnchorElement` type you'll find:

```rust
impl AsRef<HtmlElement> for HtmlAnchorElement
impl AsRef<Element> for HtmlAnchorElement
impl AsRef<Node> for HtmlAnchorElement
impl AsRef<EventTarget> for HtmlAnchorElement
impl AsRef<Object> for HtmlAnchorElement
impl AsRef<JsValue> for HtmlAnchorElement
```

You can use `.as_ref()` to explicitly get a reference to any parent class from
from a type in `web_sys`. Note that because of the number of `AsRef`
implementations you'll likely need to have type inference guidance as well.

### Accessing child clases using `JsCast`

Finally the `wasm_bindgen::JsCast` trait can be used to implement all manner of
casts between types. It supports static unchecked casts between types as well as
dynamic runtime-checked casts (using `instanceof`) between types.

More documentation about this can be found [on the trait itself][jscast]

[jscast]: https://docs.rs/wasm-bindgen/0.2/wasm_bindgen/trait.JsCast.html
