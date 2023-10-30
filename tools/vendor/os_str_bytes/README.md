# OsStr Bytes

This crate allows interacting with the data stored by [`OsStr`] and
[`OsString`], without resorting to panics or corruption for invalid UTF-8.
Thus, methods can be used that are already defined on [`[u8]`][slice] and
[`Vec<u8>`].

Typically, the only way to losslessly construct [`OsStr`] or [`OsString`] from
a byte sequence is to use `OsStr::new(str::from_utf8(bytes)?)`, which requires
the bytes to be valid in UTF-8. However, since this crate makes conversions
directly between the platform encoding and raw bytes, even some strings invalid
in UTF-8 can be converted.

[![GitHub Build Status](https://github.com/dylni/os_str_bytes/workflows/build/badge.svg?branch=master)](https://github.com/dylni/os_str_bytes/actions?query=branch%3Amaster)

## Usage

Add the following lines to your "Cargo.toml" file:

```toml
[dependencies]
os_str_bytes = "6.5"
```

See the [documentation] for available functionality and examples.

## Rust version support

The minimum supported Rust toolchain version depends on the platform:

<table>
    <tr>
        <th>Target</th>
        <th>Target Triple</th>
        <th>Minimum Version</th>
    </tr>
    <tr>
        <td>Fortanix</td>
        <td><code>*-fortanix-*-sgx</code></td>
        <td>nightly (<a href="https://doc.rust-lang.org/unstable-book/library-features/sgx-platform.html"><code>sgx_platform</code></a>)</td>
    </tr>
    <tr>
        <td>HermitCore</td>
        <td><code>*-*-hermit</code></td>
        <td>nightly (<a href="https://github.com/hermitcore/rusty-hermit/blob/86232e295ff5c50db6c283a47cff3f38a0d1b012/rust-toolchain.toml#L3"><code>rust-toolchain.toml</code></a>)</td>
    </tr>
    <tr>
        <td>SOLID</td>
        <td><code>*-*-solid_asp3(-*)</code></td>
        <td>1.57.0</td>
    </tr>
    <tr>
        <td>Unix</td>
        <td>Unix</td>
        <td>1.57.0</td>
    </tr>
    <tr>
        <td>WASI</td>
        <td><code>*-wasi</code></td>
        <td>1.57.0</td>
    </tr>
    <tr>
        <td>WebAssembly</td>
        <td><code>wasm32-*-unknown</code></td>
        <td>1.57.0</td>
    </tr>
    <tr>
        <td>Windows</td>
        <td><code>*-*-windows-*</code></td>
        <td>1.57.0</td>
    </tr>
    <tr>
        <td>Xous</td>
        <td><code>*-*-xous-*</code></td>
        <td>unstable</td>
    </tr>
</table>

Minor version updates may increase these version requirements. However, the
previous two Rust releases will always be supported. If the minimum Rust
version must not be increased, use a tilde requirement to prevent updating this
crate's minor version:

```toml
[dependencies]
os_str_bytes = "~6.5"
```

## License

Licensing terms are specified in [COPYRIGHT].

Unless you explicitly state otherwise, any contribution submitted for inclusion
in this crate, as defined in [LICENSE-APACHE], shall be licensed according to
[COPYRIGHT], without any additional terms or conditions.

[COPYRIGHT]: https://github.com/dylni/os_str_bytes/blob/master/COPYRIGHT
[documentation]: https://docs.rs/os_str_bytes
[LICENSE-APACHE]: https://github.com/dylni/os_str_bytes/blob/master/LICENSE-APACHE
[slice]: https://doc.rust-lang.org/std/primitive.slice.html
[`OsStr`]: https://doc.rust-lang.org/std/ffi/struct.OsStr.html
[`OsString`]: https://doc.rust-lang.org/std/ffi/struct.OsString.html
[`Vec<u8>`]: https://doc.rust-lang.org/std/vec/struct.Vec.html
