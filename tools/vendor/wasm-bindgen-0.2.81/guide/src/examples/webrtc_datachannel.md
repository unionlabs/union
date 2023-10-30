# WebRTC DataChannel Example

[View full source code][code] or [view the compiled example online][online]

[online]: https://rustwasm.github.io/wasm-bindgen/exbuild/webrtc_datachannel/
[code]: https://github.com/rustwasm/wasm-bindgen/tree/master/examples/webrtc_datachannel/

This example creates 2 peer connections and 2 data channels in single browser tab.
Send ping/pong between `peer1.dc` and `peer2.dc`.

## `Cargo.toml`

The `Cargo.toml` enables features necessary to use WebRTC DataChannel and its negotiation.

```toml
{{#include ../../../examples/webrtc_datachannel/Cargo.toml}}
```

## `src/lib.rs`

The Rust code connects WebRTC data channel.

```rust
{{#include ../../../examples/webrtc_datachannel/src/lib.rs}}
```
