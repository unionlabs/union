Async trait methods
===================

[<img alt="github" src="https://img.shields.io/badge/github-dtolnay/async--trait-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/dtolnay/async-trait)
[<img alt="crates.io" src="https://img.shields.io/crates/v/async-trait.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/async-trait)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-async--trait-66c2a5?style=for-the-badge&labelColor=555555&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/async-trait)
[<img alt="build status" src="https://img.shields.io/github/workflow/status/dtolnay/async-trait/CI/master?style=for-the-badge" height="20">](https://github.com/dtolnay/async-trait/actions?query=branch%3Amaster)

The initial round of stabilizations for the async/await language feature in Rust
1.39 did not include support for async fn in traits. Trying to include an async
fn in a trait produces the following error:

```rust
trait MyTrait {
    async fn f() {}
}
```

```console
error[E0706]: trait fns cannot be declared `async`
 --> src/main.rs:4:5
  |
4 |     async fn f() {}
  |     ^^^^^^^^^^^^^^^
```

This crate provides an attribute macro to make async fn in traits work.

Please refer to [*why async fn in traits are hard*][hard] for a deeper analysis
of how this implementation differs from what the compiler and language hope to
deliver in the future.

[hard]: https://smallcultfollowing.com/babysteps/blog/2019/10/26/async-fn-in-traits-are-hard/

<br>

## Example

This example implements the core of a highly effective advertising platform
using async fn in a trait.

The only thing to notice here is that we write an `#[async_trait]` macro on top
of traits and trait impls that contain async fn, and then they work.

```rust
use async_trait::async_trait;

#[async_trait]
trait Advertisement {
    async fn run(&self);
}

struct Modal;

#[async_trait]
impl Advertisement for Modal {
    async fn run(&self) {
        self.render_fullscreen().await;
        for _ in 0..4u16 {
            remind_user_to_join_mailing_list().await;
        }
        self.hide_for_now().await;
    }
}

struct AutoplayingVideo {
    media_url: String,
}

#[async_trait]
impl Advertisement for AutoplayingVideo {
    async fn run(&self) {
        let stream = connect(&self.media_url).await;
        stream.play().await;

        // Video probably persuaded user to join our mailing list!
        Modal.run().await;
    }
}
```

<br>

## Supported features

It is the intention that all features of Rust traits should work nicely with
\#\[async_trait\], but the edge cases are numerous. *Please file an issue if you
see unexpected borrow checker errors, type errors, or warnings.* There is no use
of `unsafe` in the expanded code, so rest assured that if your code compiles it
can't be that badly broken.

- &#128077;&ensp;Self by value, by reference, by mut reference, or no self;
- &#128077;&ensp;Any number of arguments, any return value;
- &#128077;&ensp;Generic type parameters and lifetime parameters;
- &#128077;&ensp;Associated types;
- &#128077;&ensp;Having async and non-async functions in the same trait;
- &#128077;&ensp;Default implementations provided by the trait;
- &#128077;&ensp;Elided lifetimes;
- &#128077;&ensp;Dyn-capable traits.

<br>

## Explanation

Async fns get transformed into methods that return `Pin<Box<dyn Future + Send +
'async_trait>>` and delegate to a private async freestanding function.

For example the `impl Advertisement for AutoplayingVideo` above would be
expanded as:

```rust
impl Advertisement for AutoplayingVideo {
    fn run<'async_trait>(
        &'async_trait self,
    ) -> Pin<Box<dyn std::future::Future<Output = ()> + Send + 'async_trait>>
    where
        Self: Sync + 'async_trait,
    {
        async fn run(_self: &AutoplayingVideo) {
            /* the original method body */
        }

        Box::pin(run(self))
    }
}
```

<br>

## Non-threadsafe futures

Not all async traits need futures that are `dyn Future + Send`. To avoid having
Send and Sync bounds placed on the async trait methods, invoke the async trait
macro as `#[async_trait(?Send)]` on both the trait and the impl blocks.

<br>

## Elided lifetimes

Be aware that async fn syntax does not allow lifetime elision outside of `&` and
`&mut` references. (This is true even when not using #\[async_trait\].)
Lifetimes must be named or marked by the placeholder `'_`.

Fortunately the compiler is able to diagnose missing lifetimes with a good error
message.

```rust
type Elided<'a> = &'a usize;

#[async_trait]
trait Test {
    async fn test(not_okay: Elided, okay: &usize) {}
}
```

```console
error[E0726]: implicit elided lifetime not allowed here
 --> src/main.rs:9:29
  |
9 |     async fn test(not_okay: Elided, okay: &usize) {}
  |                             ^^^^^^- help: indicate the anonymous lifetime: `<'_>`
```

The fix is to name the lifetime or use `'_`.

```rust
#[async_trait]
trait Test {
    // either
    async fn test<'e>(elided: Elided<'e>) {}
    // or
    async fn test(elided: Elided<'_>) {}
}
```

<br>

## Dyn traits

Traits with async methods can be used as trait objects as long as they meet the
usual requirements for dyn -- no methods with type parameters, no self by value,
no associated types, etc.

```rust
#[async_trait]
pub trait ObjectSafe {
    async fn f(&self);
    async fn g(&mut self);
}

impl ObjectSafe for MyType {...}

let value: MyType = ...;
let object = &value as &dyn ObjectSafe;  // make trait object
```

The one wrinkle is in traits that provide default implementations of async
methods. In order for the default implementation to produce a future that is
Send, the async\_trait macro must emit a bound of `Self: Sync` on trait methods
that take `&self` and a bound `Self: Send` on trait methods that take `&mut
self`. An example of the former is visible in the expanded code in the
explanation section above.

If you make a trait with async methods that have default implementations,
everything will work except that the trait cannot be used as a trait object.
Creating a value of type `&dyn Trait` will produce an error that looks like
this:

```console
error: the trait `Test` cannot be made into an object
 --> src/main.rs:8:5
  |
8 |     async fn cannot_dyn(&self) {}
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
```

For traits that need to be object safe and need to have default implementations
for some async methods, there are two resolutions. Either you can add Send
and/or Sync as supertraits (Send if there are `&mut self` methods with default
implementations, Sync if there are `&self` methods with default implementations)
to constrain all implementors of the trait such that the default implementations
are applicable to them:

```rust
#[async_trait]
pub trait ObjectSafe: Sync {  // added supertrait
    async fn can_dyn(&self) {}
}

let object = &value as &dyn ObjectSafe;
```

or you can strike the problematic methods from your trait object by bounding
them with `Self: Sized`:

```rust
#[async_trait]
pub trait ObjectSafe {
    async fn cannot_dyn(&self) where Self: Sized {}

    // presumably other methods
}

let object = &value as &dyn ObjectSafe;
```

<br>

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
