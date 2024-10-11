# blst [![Crates.io](https://img.shields.io/crates/v/blst.svg)](https://crates.io/crates/blst)

The `blst` crate provides a rust interface to the blst BLS12-381 signature library.

## Build
[bindgen](https://github.com/rust-lang/rust-bindgen) is used to generate FFI bindings to blst.h. Then [build.rs](https://github.com/supranational/blst/blob/master/bindings/rust/build.rs) invokes C compiler to compile everything into libblst.a within the rust target build area. On Linux it's possible to choose compiler by setting `CC` environment variable.

Everything can be built and run with the typical cargo commands:

```
cargo test
cargo bench
```

If the target application crashes with an "illegal instruction" exception [after copying to an older system], activate `portable` feature when building blst. Conversely, if you compile on an older Intel system, but will execute the binary on a newer one, consider instead activating <nobr>`force-adx`</nobr> feature. Though keep in mind that [cc](https://crates.io/crates/cc) passes the value of `CFLAGS` environment variable to the C compiler, and if set to contain specific flags, it can interfere with feature selection. <nobr>`-D__BLST_PORTABLE__`</nobr> and <nobr>`-D__ADX__`</nobr> are the said features' equivalents.

To compile for WebAssembly, your clang has to recognize `--target=wasm32`. Alternatively you can build your project with `CC` environment variable set to `emcc`, the [Emscripten compiler](https://emscripten.org), and `AR` set to `emar`, naturally, with both commands available on your `PATH`.

While `cargo test`'s dependencies happen to require at least Rust 1.65, the library by itself can be compiled with earlier compiler versions. Though in order to use Rust version prior 1.56 you would need to pin`zeroize` to "=1.3.0" and `zeroize_derive` to "=1.3.3" in **your** project Cargo.toml. Even `cc` might require pinning to "=1.0.79". And if you find yourself with Rust 1.56 through 1.64 as the only option and want to execute `cargo test` you'd need to pin some of `[dev-dependencies]` versions in **this** project's Cargo.toml by uncommenting following lines:

```
csv = "=1.1.6"
byteorder = "=1.4.3"
regex = "=1.7.3"
rayon = "=1.6.1"
rayon-core = "=1.10.1"
```

## Usage
There are two primary modes of operation that can be chosen based on declaration path:

For minimal-pubkey-size operations:
```rust
use blst::min_pk::*;
```

For minimal-signature-size operations:
```rust
use blst::min_sig::*;
```

There are five structs with inherent implementations that provide the BLS12-381 signature functionality.
```
SecretKey
PublicKey
AggregatePublicKey
Signature
AggregateSignature
```

A simple example for generating a key, signing a message, and verifying the message:
```rust
use blst::min_pk::SecretKey;

let mut rng = rand::thread_rng();
let mut ikm = [0u8; 32];
rng.fill_bytes(&mut ikm);

let sk = SecretKey::key_gen(&ikm, &[]).unwrap();
let pk = sk.sk_to_pk();

let dst = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_NUL_";
let msg = b"blst is such a blast";
let sig = sk.sign(msg, dst, &[]);

let err = sig.verify(true, msg, dst, &[], &pk, true);
assert_eq!(err, blst::BLST_ERROR::BLST_SUCCESS);
```

See the tests in src/lib.rs and benchmarks in benches/blst_benches.rs for further examples of usage.
