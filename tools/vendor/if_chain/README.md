# if_chain

[![Build Status](https://travis-ci.com/lambda-fairy/if_chain.svg?branch=master)](https://travis-ci.com/lambda-fairy/if_chain) [![Cargo](https://img.shields.io/crates/v/if_chain.svg)](https://crates.io/crates/if_chain)

This crate provides a single macro called `if_chain!`.

`if_chain!` lets you write long chains of nested `if` and `if let` statements without the associated rightward drift. It also supports multiple patterns (e.g. `if let Foo(a) | Bar(a) = b`) in places where Rust would normally not allow them.

For more information on this crate, see the [documentation](https://docs.rs/if_chain) and associated [blog post](https://lambda.xyz/blog/if-chain).
