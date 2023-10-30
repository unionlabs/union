# WebSockets Example

[View full source code][code] or [view the compiled example online][online]

[online]: https://rustwasm.github.io/wasm-bindgen/exbuild/websockets/
[code]: https://github.com/rustwasm/wasm-bindgen/tree/master/examples/websockets/

This example connects to an echo server on `wss://echo.websocket.org`,
sends a `ping` message, and receives the response.

## `Cargo.toml`

The `Cargo.toml` enables features necessary to create a `WebSocket` object and
to access events such as `MessageEvent` or `ErrorEvent`.

```toml
{{#include ../../../examples/websockets/Cargo.toml}}
```

## `src/lib.rs`

This code shows the basic steps required to work with a `WebSocket`.
At first it opens the connection, then subscribes to events `onmessage`, `onerror`, `onopen`.
After the socket is opened it sends a `ping` message, receives an echoed response
and prints it to the browser console.

```rust
{{#include ../../../examples/websockets/src/lib.rs}}
```
