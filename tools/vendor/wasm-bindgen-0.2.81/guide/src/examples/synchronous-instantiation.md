# Synchronous Instantiation

[View full source code][code]

[code]: https://github.com/rustwasm/wasm-bindgen/tree/master/examples/synchronous-instantiation

This example shows how to synchronously initialize a WebAssembly module as opposed to [asynchronously][without-bundler]. In most cases, the default way of asynchronously initializing a module will suffice. However, there might be use cases where you'd like to lazy load a module on demand and synchronously compile and instantiate it. Note that this only works off the main thread and since compilation and instantiation of large modules can be expensive you should only use this method if it's absolutely required in your use case. Otherwise you should use the [default method][without-bundler].

For this deployment strategy bundlers like Webpack are not required. For more information on deployment see the [dedicated
documentation][deployment].

First let's take a look at our tiny lib:

```rust
{{#include ../../../examples/synchronous-instantiation/src/lib.rs}}
```

Next, let's have a look at the `index.html`:

```html
{{#include ../../../examples/synchronous-instantiation/index.html}}
```

Otherwise the rest of the magic happens in `worker.js`:

```js
{{#include ../../../examples/synchronous-instantiation/worker.js}}
```

And that's it! Be sure to read up on the [deployment options][deployment] to see what it means to deploy without a bundler.

[deployment]: ../reference/deployment.html
[without-bundler]: ./without-a-bundler.html
