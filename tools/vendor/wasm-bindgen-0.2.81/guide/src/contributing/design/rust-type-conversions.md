# Rust Type conversions

Previously we've been seeing mostly abridged versions of type conversions when
values enter Rust. Here we'll go into some more depth about how this is
implemented. There are two categories of traits for converting values, traits
for converting values from Rust to JS and traits for the other way around.

## From Rust to JS

First up let's take a look at going from Rust to JS:

```rust
pub trait IntoWasmAbi: WasmDescribe {
    type Abi: WasmAbi;
    fn into_abi(self, extra: &mut Stack) -> Self::Abi;
}
```

And that's it! This is actually the only trait needed currently for translating
a Rust value to a JS one. There's a few points here:

* We'll get to `WasmDescribe` later in this section
* The associated type `Abi` is what will actually be generated as an argument to
  the wasm export. The bound `WasmAbi` is only implemented for types like `u32`
  and `f64`, those which can be placed on the boundary and transmitted
  losslessly.
* And finally we have the `into_abi` function, returning the `Abi` associated
  type which will be actually passed to JS. There's also this `Stack` parameter,
  however. Not all Rust values can be communicated in 32 bits to the `Stack`
  parameter allows transmitting more data, explained in a moment.

This trait is implemented for all types that can be converted to JS and is
unconditionally used during codegen. For example you'll often see `IntoWasmAbi
for Foo` but also `IntoWasmAbi for &'a Foo`.

The `IntoWasmAbi` trait is used in two locations. First it's used to convert
return values of Rust exported functions to JS. Second it's used to convert the
Rust arguments of JS functions imported to Rust.

## From JS to Rust

Unfortunately the opposite direction from above, going from JS to Rust, is a bit
more complicated. Here we've got three traits:

```rust
pub trait FromWasmAbi: WasmDescribe {
    type Abi: WasmAbi;
    unsafe fn from_abi(js: Self::Abi, extra: &mut Stack) -> Self;
}

pub trait RefFromWasmAbi: WasmDescribe {
    type Abi: WasmAbi;
    type Anchor: Deref<Target=Self>;
    unsafe fn ref_from_abi(js: Self::Abi, extra: &mut Stack) -> Self::Anchor;
}

pub trait RefMutFromWasmAbi: WasmDescribe {
    type Abi: WasmAbi;
    type Anchor: DerefMut<Target=Self>;
    unsafe fn ref_mut_from_abi(js: Self::Abi, extra: &mut Stack) -> Self::Anchor;
}
```

The `FromWasmAbi` is relatively straightforward, basically the opposite of
`IntoWasmAbi`. It takes the ABI argument (typically the same as
`IntoWasmAbi::Abi`) and then the auxiliary stack to produce an instance of
`Self`. This trait is implemented primarily for types that *don't* have internal
lifetimes or are references.

The latter two traits here are mostly the same, and are intended for generating
references (both shared and mutable references). They look almost the same as
`FromWasmAbi` except that they return an `Anchor` type which implements a
`Deref` trait rather than `Self`.

The `Ref*` traits allow having arguments in functions that are references rather
than bare types, for example `&str`, `&JsValue`, or `&[u8]`. The `Anchor` here
is required to ensure that the lifetimes don't persist beyond one function call
and remain anonymous.

The `From*` family of traits are used for converting the Rust arguments in Rust
exported functions to JS. They are also used for the return value in JS
functions imported into Rust.

## Global stack

Mentioned above not all Rust types will fit within 32 bits. While we can
communicate an `f64` we don't necessarily have the ability to use all the bits.
Types like `&str` need to communicate two items, a pointer and a length (64
bits). Other types like `&Closure<dyn Fn()>` have even more information to
transmit.

As a result we need a method of communicating more data through the signatures
of functions. While we could add more arguments this is somewhat difficult to do
in the world of closures where code generation isn't quite as dynamic as a
procedural macro. Consequently a "global stack" is used to transmit extra
data for a function call.

The global stack is a fixed-sized static allocation in the wasm module. This
stack is temporary scratch space for any one function call from either JS to
Rust or Rust to JS. Both Rust and the JS shim generated have pointers to this
global stack and will read/write information from it.

Using this scheme whenever we want to pass `&str` from JS to Rust we can pass
the pointer as the actual ABI argument and the length is then placed in the next
spot on the global stack.

The `Stack` argument to the conversion traits above looks like:

```rust
pub trait Stack {
    fn push(&mut self, bits: u32);
    fn pop(&mut self) -> u32;
}
```

A trait is used here to facilitate testing but typically the calls don't end up
being virtually dispatched at runtime.
