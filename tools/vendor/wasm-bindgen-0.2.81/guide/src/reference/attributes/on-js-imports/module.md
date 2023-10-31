# `module = "blah"`

The `module` attributes configures the module from which items are imported. For
example,

```rust
#[wasm_bindgen(module = "wu/tang/clan")]
extern "C" {
    type ThirtySixChambers;
}
```

generates JavaScript import glue like:

```js
import { ThirtySixChambers } from "wu/tang/clan";
```

If a `module` attribute is not present, then the global scope is used
instead. For example,

```rust
#[wasm_bindgen]
extern "C" {
    fn illmatic() -> u32;
}
```

generates JavaScript import glue like:

```js
let illmatic = this.illmatic;
```

Note that if the string specified with `module` starts with `./`, `../`, or `/`
then it's interpreted as a path to a [local JS snippet](../../js-snippets.html).
If this doesn't work for your use case you might be interested in the
[`raw_module` attribute](raw_module.html)
