# Working with Duck-Typed Interfaces

Liberal use of [the `structural`
attribute](./attributes/on-js-imports/structural.html) on imported methods,
getters, and setters allows you to define duck-typed interfaces. A duck-typed
interface is one where many different JavaScript objects that don't share the
same base class in their prototype chain and therefore are not `instanceof` the
same base can be used the same way.

## Defining a Duck-Typed Interface in Rust

```rust
{{#include ../../../examples/duck-typed-interfaces/src/lib.rs}}
```

## JavaScript Usage

```js
{{#include ../../../examples/duck-typed-interfaces/duck-typed-interfaces.js}}
```
