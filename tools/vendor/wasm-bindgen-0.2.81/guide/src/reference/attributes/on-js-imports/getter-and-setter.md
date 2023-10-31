# `getter` and `setter`

These two attributes can be combined with `method` to indicate that this is a
getter or setter method. A `getter`-tagged function by default accesses the
JavaScript property with the same name as the getter function. A `setter`'s
function name is currently required to start with `set_` and the property it
accesses is the suffix after `set\_`.

Consider the following JavaScript class that has a getter and setter for the
`white_russians` property:

```js
class TheDude {
  get white_russians() {
    ...
  }
  set white_russians(val) {
    ...
  }
}
```

We would import this with the following `#[wasm_bindgen]` attributes:

```rust
#[wasm_bindgen]
extern "C" {
    type TheDude;

    #[wasm_bindgen(method, getter)]
    fn white_russians(this: &TheDude) -> u32;

    #[wasm_bindgen(method, setter)]
    fn set_white_russians(this: &TheDude, val: u32);
}
```

Here we're importing the `TheDude` type and defining the ability to access each
object's `white_russians` property. The first function here is a getter and will
be available in Rust as `the_dude.white_russians()`, and the latter is the
setter which is accessible as `the_dude.set_white_russians(2)`. Note that both
functions have a `this` argument as they're tagged with `method`.

Finally, you can also pass an argument to the `getter` and `setter`
properties to configure what property is accessed. When the property is
explicitly specified then there is no restriction on the method name. For
example the below is equivalent to the above:

```rust
#[wasm_bindgen]
extern "C" {
    type TheDude;

    #[wasm_bindgen(method, getter = white_russians)]
    fn my_custom_getter_name(this: &TheDude) -> u32;

    #[wasm_bindgen(method, setter = white_russians)]
    fn my_custom_setter_name(this: &TheDude, val: u32);
}
```

Heads up! `getter` and `setter` functions are found on the constructor's
prototype chain once at load time, cached, and then the cached accessor is
invoked on each access. If you need to dynamically walk the prototype chain on
every access, add the `structural` attribute!

```js
// This is the default function Rust will invoke on `the_dude.white_russians()`:
const white_russians = Object.getOwnPropertyDescriptor(
  TheDude.prototype,
  "white_russians"
).get;

// This is what you get by adding `structural`:
const white_russians = function(the_dude) {
  return the_dude.white_russians;
};
```
