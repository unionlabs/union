// vim: tw=80
//! A powerful mock object library for Rust.
//!
//! Mockall provides tools to create mock versions of almost any trait
//! or struct. They can be used in unit tests as a stand-in for the real
//! object.
//!
//! # Usage
//!
//! There are two ways to use Mockall.  The easiest is to use
//! [`#[automock]`](attr.automock.html).  It can mock most traits, or structs
//! that only have a single `impl` block.  For things it can't handle, there is
//! [`mock!`].
//!
//! Whichever method is used, the basic idea is the same.
//! * Create a mock struct.  It's name will be the same as the original, with
//!   "Mock" prepended.
//! * In your test, instantiate the mock struct with its `new` or `default`
//!   method.
//! * Set expectations on the mock struct.  Each expectation can have required
//!   argument matchers, a required call count, and a required position in a
//!   [`Sequence`].  Each expectation must also have a return value.
//! * Supply the mock object to the code that you're testing.  It will return
//!   the preprogrammed return values supplied in the previous step.  Any
//!   accesses contrary to your expectations will cause a panic.
//!
//! # User Guide
//!
//! * [`Getting started`](#getting-started)
//! * [`Static Return values`](#static-return-values)
//! * [`Matching arguments`](#matching-arguments)
//! * [`Call counts`](#call-counts)
//! * [`Sequences`](#sequences)
//! * [`Checkpoints`](#checkpoints)
//! * [`Reference arguments`](#reference-arguments)
//! * [`Reference return values`](#reference-return-values)
//! * [`impl Trait`](#impl-trait)
//! * [`Mocking structs`](#mocking-structs)
//! * [`Generic methods`](#generic-methods)
//! * [`Methods with generic lifetimes`](#methods-with-generic-lifetimes)
//! * [`Generic traits and structs`](#generic-traits-and-structs)
//! * [`Associated types`](#associated-types)
//! * [`Multiple and inherited traits`](#multiple-and-inherited-traits)
//! * [`External traits`](#external-traits)
//! * [`Static methods`](#static-methods)
//! * [`Modules`](#modules)
//! * [`Foreign functions`](#foreign-functions)
//! * [`Debug`](#debug)
//! * [`Async Traits`](#async-traits)
//! * [`Crate features`](#crate-features)
//! * [`Examples`](#examples)
//!
//! ## Getting Started
//! ```
//! use mockall::*;
//! use mockall::predicate::*;
//! #[automock]
//! trait MyTrait {
//!     fn foo(&self, x: u32) -> u32;
//! }
//!
//! fn call_with_four(x: &MyTrait) -> u32 {
//!     x.foo(4)
//! }
//!
//! let mut mock = MockMyTrait::new();
//! mock.expect_foo()
//!     .with(predicate::eq(4))
//!     .times(1)
//!     .returning(|x| x + 1);
//! assert_eq!(5, call_with_four(&mock));
//! ```
//!
//! ## Static Return values
//!
//! Every expectation must have an associated return value (though when the
//! **nightly** feature is enabled expectations will automatically return the
//! default values of their return types, if their return types implement
//! `Default`.).  For methods that return a `static` value, the macros will
//! generate an `Expectation` struct like
//! [`this`](examples::__mock_MockFoo_Foo::__foo::Expectation).
//! There are two ways to set such an expectation's return value: with a
//! constant
//! ([`return_const`](examples::__mock_MockFoo_Foo::__foo::Expectation::return_const))
//! or a closure
//! ([`returning`](examples::__mock_MockFoo_Foo::__foo::Expectation::returning)).
//! A closure will take the method's arguments by value.
//!
//! ```
//! # use mockall::*;
//! #[automock]
//! trait MyTrait {
//!     fn foo(&self) -> u32;
//!     fn bar(&self, x: u32, y: u32) -> u32;
//! }
//!
//! let mut mock = MockMyTrait::new();
//! mock.expect_foo()
//!     .return_const(42u32);
//! mock.expect_bar()
//!     .returning(|x, y| x + y);
//! ```
//!
//! Additionally, constants that aren't `Clone` can be returned with the
//! [`return_once`](examples::__mock_MockFoo_Foo::__foo::Expectation::return_once)
//! method.
//!
//! ```
//! # use mockall::*;
//! struct NonClone();
//! #[automock]
//! trait Foo {
//!     fn foo(&self) -> NonClone;
//! }
//!
//! # fn main() {
//! let mut mock = MockFoo::new();
//! let r = NonClone{};
//! mock.expect_foo()
//!     .return_once(move || r);
//! # }
//! ```
//!
//! `return_once` can also be used for computing the return value with an
//! `FnOnce` closure.  This is useful for returning a non-`Clone` value and also
//! triggering side effects at the same time.
//!
//! ```
//! # use mockall::*;
//! fn do_something() {}
//!
//! struct NonClone();
//!
//! #[automock]
//! trait Foo {
//!     fn foo(&self) -> NonClone;
//! }
//!
//! # fn main() {
//! let mut mock = MockFoo::new();
//! let r = NonClone{};
//! mock.expect_foo()
//!     .return_once(move || {
//!         do_something();
//!         r
//!     });
//! # }
//! ```
//!
//! Mock objects are always `Send`.  If you need to use a return type that
//! isn't, you can use the
//! [`return_const_st`](examples::__mock_MockFoo_Foo::__foo::Expectation::return_const_st),
//! [`returning_st`](examples::__mock_MockFoo_Foo::__foo::Expectation::returning_st),
//! or
//! [`return_once_st`](examples::__mock_MockFoo_Foo::__foo::Expectation::return_once_st),
//! methods. If you need to match arguments that are not `Send`, you can use the
//! [`withf_st`](examples::__mock_MockFoo_Foo::__foo::Expectation::withf_st)
//! These take a non-`Send` object and add runtime access checks.  The wrapped
//! object will be `Send`, but accessing it from multiple threads will cause a
//! runtime panic.
//!
//! ```
//! # use mockall::*;
//! # use std::rc::Rc;
//! #[automock]
//! trait Foo {
//!     fn foo(&self, x: Rc<u32>) -> Rc<u32>;   // Rc<u32> isn't Send
//! }
//!
//! # fn main() {
//! let mut mock = MockFoo::new();
//! let x = Rc::new(5);
//! let argument = x.clone();
//! mock.expect_foo()
//!     .withf_st(move |x| *x == argument)
//!     .returning_st(move |_| Rc::new(42u32));
//! assert_eq!(42, *mock.foo(x));
//! # }
//! ```
//!
//! ## Matching arguments
//!
//! Optionally, expectations may have argument matchers set.  A matcher will
//! verify that the expectation was called with the expected arguments, or panic
//! otherwise.  A matcher is anything that implements the [`Predicate`] trait.
//! For example:
//!
//! ```should_panic
//! # use mockall::*;
//! # use mockall::predicate::*;
//! #[automock]
//! trait Foo {
//!     fn foo(&self, x: u32);
//! }
//!
//! let mut mock = MockFoo::new();
//! mock.expect_foo()
//!     .with(eq(42))
//!     .return_const(());
//!
//! mock.foo(0);    // Panics!
//! ```
//!
//! See [`predicate`] for a list of Mockall's builtin predicate functions.
//! For convenience,
//! [`withf`](examples::__mock_MockFoo_Foo::__foo::Expectation::withf)
//! is a shorthand for setting the commonly used
//! [`function`] predicate.  The arguments to the predicate function are the
//! method's arguments, *by reference*.  For example:
//!
//! ```should_panic
//! # use mockall::*;
//! #[automock]
//! trait Foo {
//!     fn foo(&self, x: u32, y: u32);
//! }
//!
//! # fn main() {
//! let mut mock = MockFoo::new();
//! mock.expect_foo()
//!     .withf(|x: &u32, y: &u32| x == y)
//!     .return_const(());
//!
//! mock.foo(2 + 2, 5);    // Panics!
//! # }
//! ```
//!
//! ### Matching multiple calls
//!
//! Matchers can also be used to discriminate between different invocations of
//! the same function.  Used that way, they can provide different return values
//! for different arguments.  The way this works is that on a method call, all
//! expectations set on a given method are evaluated in FIFO order.  The first
//! matching expectation is used.  Only if none of the expectations match does
//! Mockall panic.  For example:
//!
//! ```
//! # use mockall::*;
//! # use mockall::predicate::*;
//! #[automock]
//! trait Foo {
//!     fn foo(&self, x: u32) -> u32;
//! }
//!
//! # fn main() {
//! let mut mock = MockFoo::new();
//! mock.expect_foo()
//!     .with(eq(5))
//!     .return_const(50u32);
//! mock.expect_foo()
//!     .with(eq(6))
//!     .return_const(60u32);
//! # }
//! ```
//!
//! One common pattern is to use multiple expectations in order of decreasing
//! specificity.  The last expectation can provide a default or fallback value,
//! and earlier ones can be more specific.  For example:
//!
//! ```
//! # use mockall::*;
//! # use mockall::predicate::*;
//! #[automock]
//! trait Foo {
//!     fn open(&self, path: String) -> Option<u32>;
//! }
//!
//! let mut mock = MockFoo::new();
//! mock.expect_open()
//!     .with(eq(String::from("something.txt")))
//!     .returning(|_| Some(5));
//! mock.expect_open()
//!     .return_const(None);
//! ```
//!
//! ## Call counts
//!
//! By default, every expectation is allowed to be called an unlimited number of
//! times.  But Mockall can optionally verify that an expectation was called a
//! fixed number of times, or any number of times within a given range.
//!
//! ```should_panic
//! # use mockall::*;
//! # use mockall::predicate::*;
//! #[automock]
//! trait Foo {
//!     fn foo(&self, x: u32);
//! }
//!
//! let mut mock = MockFoo::new();
//! mock.expect_foo()
//!     .times(1)
//!     .return_const(());
//!
//! mock.foo(0);    // Ok
//! mock.foo(1);    // Panics!
//! ```
//!
//! See also
//! [`never`](examples::__mock_MockFoo_Foo::__foo::Expectation::never) and
//! [`times`](examples::__mock_MockFoo_Foo::__foo::Expectation::times).
//!
//! ## Sequences
//!
//! By default expectations may be matched in any order.  But it's possible to
//! specify the order by using a [`Sequence`].  Any expectations may be added to
//! the same sequence.  They don't even need to come from the same object.
//!
//! ```should_panic(expected = "Method sequence violation")
//! # use mockall::*;
//! #[automock]
//! trait Foo {
//!     fn foo(&self);
//! }
//!
//! # fn main() {
//! let mut seq = Sequence::new();
//!
//! let mut mock1 = MockFoo::new();
//! mock1.expect_foo()
//!     .times(1)
//!     .in_sequence(&mut seq)
//!     .returning(|| ());
//!
//! let mut mock2 = MockFoo::new();
//! mock2.expect_foo()
//!     .times(1)
//!     .in_sequence(&mut seq)
//!     .returning(|| ());
//!
//! mock2.foo();    // Panics!  mock1.foo should've been called first.
//! # }
//! ```
//!
//! ## Checkpoints
//!
//! Sometimes its useful to validate all expectations mid-test, throw them away,
//! and add new ones.  That's what checkpoints do.  Every mock object has a
//! `checkpoint` method.  When called, it will immediately validate all methods'
//! expectations.  So any expectations that haven't satisfied their call count
//! will panic.  Afterwards, those expectations will be cleared so you can add
//! new expectations and keep testing.
//!
//! ```should_panic
//! # use mockall::*;
//! #[automock]
//! trait Foo {
//!     fn foo(&self);
//! }
//!
//! let mut mock = MockFoo::new();
//! mock.expect_foo()
//!     .times(2)
//!     .returning(|| ());
//!
//! mock.foo();
//! mock.checkpoint();  // Panics!  foo hasn't yet been called twice.
//! ```
//!
//! ```should_panic
//! # use mockall::*;
//! #[automock]
//! trait Foo {
//!     fn foo(&self);
//! }
//!
//! # fn main() {
//! let mut mock = MockFoo::new();
//! mock.expect_foo()
//!     .times(1)
//!     .returning(|| ());
//!
//! mock.foo();
//! mock.checkpoint();
//! mock.foo();         // Panics!  The expectation has been cleared.
//! # }
//! ```
//!
//! ## Reference arguments
//!
//! Mockall can mock methods with reference arguments, too.  There's one catch:
//! the matcher [`Predicate`] will take reference arguments by value, not by
//! reference.
//!
//! ```
//! # use mockall::*;
//! #[automock]
//! trait Foo {
//!     fn foo(&self, x: &u32) -> u32;
//! }
//!
//! let mut mock = MockFoo::new();
//! let e = mock.expect_foo()
//!     // Note that x is a &u32, not a &&u32
//!     .withf(|x: &u32| *x == 5)
//!     .returning(|x: &u32| *x + 1);
//!
//! assert_eq!(6, mock.foo(&5));
//! ```
//!
//! ## Reference return values
//!
//! Mockall can also use reference return values.  There is one restriction: the
//! lifetime of the returned reference must be either the same as the lifetime
//! of the mock object, or `'static`.
//!
//! Mockall creates different expectation types for methods that return
//! references.  Their API is the same as the basic `Expectation`, except for
//! setting return values.
//!
//! Methods that return `'static` references work just like methods that return
//! any other `'static` value.
//! ```
//! # use mockall::*;
//! struct Thing(u32);
//!
//! #[automock]
//! trait Container {
//!     fn get(&self, i: u32) -> &'static Thing;
//! }
//!
//! # fn main() {
//! const THING: Thing = Thing(42);
//! let mut mock = MockContainer::new();
//! mock.expect_get()
//!     .return_const(&THING);
//!
//! assert_eq!(42, mock.get(0).0);
//! # }
//! ```
//!
//! Methods that take a `&self` argument use an `Expectation` class like
//! [this](examples::__mock_MockFoo_Foo::__bar::Expectation),
//! which
//! gets its return value from the
//! [`return_const`](examples::__mock_MockFoo_Foo::__bar::Expectation::return_const) method.
//!
//! ```
//! # use mockall::*;
//! struct Thing(u32);
//!
//! #[automock]
//! trait Container {
//!     fn get(&self, i: u32) -> &Thing;
//! }
//!
//! # fn main() {
//! let thing = Thing(42);
//! let mut mock = MockContainer::new();
//! mock.expect_get()
//!     .return_const(thing);
//!
//! assert_eq!(42, mock.get(0).0);
//! # }
//! ```
//!
//! Methods that take a `&mut self` argument use an `Expectation` class like
//! [this](examples::__mock_MockFoo_Foo::__baz::Expectation),
//! class, regardless of whether the return value is actually mutable.  They can
//! take their return value either from the
//! [`return_var`](examples::__mock_MockFoo_Foo::__baz::Expectation::return_var)
//! or
//! [`returning`](examples::__mock_MockFoo_Foo::__baz::Expectation::returning)
//! methods.
//!
//! ```
//! # use mockall::*;
//! struct Thing(u32);
//!
//! #[automock]
//! trait Container {
//!     fn get_mut(&mut self, i: u32) -> &mut Thing;
//! }
//!
//! # fn main() {
//! let thing = Thing(42);
//! let mut mock = MockContainer::new();
//! mock.expect_get_mut()
//!     .return_var(thing);
//!
//! mock.get_mut(0).0 = 43;
//! assert_eq!(43, mock.get_mut(0).0);
//! # }
//! ```
//!
//! Unsized types that are common targets for
//! [`Deref`](core::ops::Deref)
//! are special.  Mockall
//! will automatically use the type's owned form for the Expectation.
//! Currently, the
//! [`CStr`](std::ffi::CStr),
//! [`OsStr`](std::ffi::OsStr),
//! [`Path`](std::path::Path),
//! [`Slice`][std::slice],
//! and
//! [`str`](std::str)
//! types are supported.  Using this feature is automatic:
//!
//! ```
//! # use mockall::*;
//! #[automock]
//! trait Foo {
//!     fn name(&self) -> &str;
//! }
//!
//! let mut mock = MockFoo::new();
//! mock.expect_name().return_const("abcd".to_owned());
//! assert_eq!("abcd", mock.name());
//! ```
//!
//! Similarly, Mockall will use a Boxed trait object for the Expectation of
//! methods that return references to trait objects.
//!
//! ```
//! # use mockall::*;
//! # use std::fmt::Display;
//! #[automock]
//! trait Foo {
//!     fn name(&self) -> &dyn Display;
//! }
//!
//! # fn main() {
//! let mut mock = MockFoo::new();
//! mock.expect_name().return_const(Box::new("abcd"));
//! assert_eq!("abcd", format!("{}", mock.name()));
//! # }
//! ```
//!
//!
//! ## Impl Trait
//!
//! Rust 1.26.0 introduced the `impl Trait` feature.  It allows functions to
//! return concrete but unnamed types (and, less usefully, to take them as
//! arguments).  It's *almost* the same as `Box<dyn Trait>` but without the
//! extra allocation.  Mockall supports deriving mocks for methods that return
//! `impl Trait`, with limitations.  When you derive the mock for such a method,
//! Mockall internally transforms the Expectation's return type to `Box<dyn
//! Trait>`, without changing the mock method's signature.  So you can use it
//! like this:
//!
//! ```
//! # use mockall::*;
//! # use std::fmt::Debug;
//! struct Foo {}
//! #[automock]
//! impl Foo {
//!     fn foo(&self) -> impl Debug {
//!         // ...
//!         # 4
//!     }
//! }
//!
//! # fn main() {
//! let mut mock = MockFoo::new();
//! mock.expect_foo()
//!     .returning(|| Box::new(String::from("Hello, World!")));
//! println!("{:?}", mock.foo());
//! # }
//! ```
//!
//! However, `impl Trait` isn't *exactly* equivalent to `Box<dyn Trait>` but
//! with fewer allocations.  There are some things the former can do but the
//! latter can't.  For one thing, you can't build a trait object out of a
//! `Sized` trait.  So this won't work:
//!
//! ```compile_fail
//! # use mockall::*;
//! struct Foo {}
//! #[automock]
//! impl Foo {
//!     fn foo(&self) -> impl Clone {
//!         // ...
//!         # 4
//!     }
//! }
//! ```
//!
//! Nor can you create a trait object that implements two or more non-auto
//! types.  So this won't work either:
//!
//! ```compile_fail
//! # use mockall::*;
//! struct Foo {}
//! #[automock]
//! impl Foo {
//!     fn foo(&self) -> impl Debug + Display {
//!         // ...
//!         # 4
//!     }
//! }
//! ```
//!
//! For such cases, there is no magic bullet.  The best way to mock methods like
//! those would be to refactor them to return named (but possibly opaque) types
//! instead.
//!
//! See Also [`impl-trait-for-returning-complex-types-with-ease.html`](https://rust-lang-nursery.github.io/edition-guide/rust-2018/trait-system/impl-trait-for-returning-complex-types-with-ease)
//!
//! ### impl Future
//!
//! Rust 1.36.0 added the `Future` trait.  Unlike virtually every trait that
//! preceeded it, `Box<dyn Future>` is mostly useless.  Instead, you usually
//! need a `Pin<Box<dyn Future>>`.  So that's what Mockall will do when you mock
//! a method returning `impl Future` or the related `impl Stream`.  Just
//! remember to use `pin` in your expectations, like this:
//!
//! ```
//! # use mockall::*;
//! # use std::fmt::Debug;
//! # use futures::{Future, future};
//! struct Foo {}
//! #[automock]
//! impl Foo {
//!     fn foo(&self) -> impl Future<Output=i32> {
//!         // ...
//!         # future::ready(42)
//!     }
//! }
//!
//! # fn main() {
//! let mut mock = MockFoo::new();
//! mock.expect_foo()
//!     .returning(|| Box::pin(future::ready(42)));
//! # }
//! ```
//!
//! ## Mocking structs
//!
//! Mockall mocks structs as well as traits.  The problem here is a namespace
//! problem: it's hard to supply the mock object to your code under test,
//! because it has a different name.  The solution is to alter import paths
//! during test.  The easiest way to do that is with the
//! [`mockall_double`](https://docs.rs/mockall_double/latest) crate.
//!
//! [`#[automock]`](attr.automock.html)
//! works for structs that have a single `impl` block:
//! ```no_run
//! use mockall_double::double;
//! mod thing {
//!     use mockall::automock;
//!     pub struct Thing{}
//!     #[automock]
//!     impl Thing {
//!         pub fn foo(&self) -> u32 {
//!             // ...
//!             # unimplemented!()
//!         }
//!     }
//! }
//!
//! #[double]
//! use thing::Thing;
//!
//! fn do_stuff(thing: &Thing) -> u32 {
//!     thing.foo()
//! }
//!
//! #[cfg(test)]
//! mod t {
//!     use super::*;
//!
//!     #[test]
//!     fn test_foo() {
//!         let mut mock = Thing::default();
//!         mock.expect_foo().returning(|| 42);
//!         do_stuff(&mock);
//!     }
//! }
//! # fn main() {}
//! ```
//! For structs with more than one `impl` block or that have unsupported
//! `#[derive(X)]` attributes, e.g. `Clone`, see [`mock!`] instead.
//!
//! ## Generic methods
//!
//! Generic methods can be mocked, too.  Effectively each generic method is an
//! infinite set of regular methods, and each of those works just like any other
//! regular method.  The expect_* method is generic, too, and usually must be
//! called with a turbofish.  The only restrictions on mocking generic methods
//! are that all generic parameters must be `'static`, and generic lifetime
//! parameters are not allowed.
//!
//! ```
//! # use mockall::*;
//! #[automock]
//! trait Foo {
//!     fn foo<T: 'static>(&self, t: T) -> i32;
//! }
//!
//! let mut mock = MockFoo::new();
//! mock.expect_foo::<i16>()
//!     .returning(|t| i32::from(t));
//! mock.expect_foo::<i8>()
//!     .returning(|t| -i32::from(t));
//!
//! assert_eq!(5, mock.foo(5i16));
//! assert_eq!(-5, mock.foo(5i8));
//! ```
//!
//! ## Methods with generic lifetimes
//!
//! A method with a lifetime parameter is technically a generic method, but
//! Mockall treats it like a non-generic method that must work for all possible
//! lifetimes.  Mocking such a method is similar to mocking a non-generic
//! method, with a few additional restrictions.  One restriction is that you
//! can't match calls with `with`, you must use `withf` instead.  Another is
//! that the generic lifetime may not appear as part of the return type.
//! Finally, no method may have both generic lifetime parameters *and* generic
//! type parameters.
//!
//! ```
//! # use mockall::*;
//! struct X<'a>(&'a i32);
//!
//! #[automock]
//! trait Foo {
//!     fn foo<'a>(&self, x: X<'a>) -> i32;
//! }
//!
//! # fn main() {
//! let mut mock = MockFoo::new();
//! mock.expect_foo()
//!     .withf(|f| *f.0 == 5)
//!     .return_const(42);
//! let x = X(&5);
//! assert_eq!(42, mock.foo(x));
//! # }
//! ```
//!
//! ## Generic traits and structs
//!
//! Mocking generic structs and generic traits is not a problem.  The mock
//! struct will be generic, too.  The same restrictions apply as with mocking
//! generic methods: each generic parameter must be `'static`, and generic
//! lifetime parameters are not allowed.
//!
//! ```
//! # use mockall::*;
//! #[automock]
//! trait Foo<T: 'static> {
//!     fn foo(&self, t: T) -> i32;
//! }
//!
//! # fn main() {
//! let mut mock = MockFoo::<i16>::new();
//! mock.expect_foo()
//!     .returning(|t| i32::from(t));
//! assert_eq!(5, mock.foo(5i16));
//! # }
//! ```
//!
//! ## Associated types
//!
//! Traits with associated types can be mocked too.  Unlike generic traits, the
//! mock struct will not be generic.  Instead, you must specify the associated
//! types when defining the mock struct.  They're specified as metaitems to the
//! [`#[automock]`](attr.automock.html) attribute.
//!
//! ```
//! # use mockall::*;
//! #[automock(type Key=u16; type Value=i32;)]
//! pub trait A {
//!     type Key;
//!     type Value;
//!     fn foo(&self, k: Self::Key) -> Self::Value;
//! }
//!
//! let mut mock = MockA::new();
//! mock.expect_foo()
//!     .returning(|x: u16| i32::from(x));
//! assert_eq!(4, mock.foo(4));
//! ```
//!
//! ## Multiple and inherited traits
//!
//! Creating a mock struct that implements multiple traits, whether inherited or
//! not, requires using the [`mock!`] macro.  But once created,
//! using it is just the same as using any other mock object:
//!
//! ```
//! # use mockall::*;
//! pub trait A {
//!     fn foo(&self);
//! }
//!
//! pub trait B: A {
//!     fn bar(&self);
//! }
//!
//! mock! {
//!     // Structure to mock
//!     C {}
//!     // First trait to implement on C
//!     impl A for C {
//!         fn foo(&self);
//!     }
//!     // Second trait to implement on C
//!     impl B for C {
//!         fn bar(&self);
//!     }
//! }
//! # fn main() {
//! let mut mock = MockC::new();
//! mock.expect_foo().returning(|| ());
//! mock.expect_bar().returning(|| ());
//! mock.foo();
//! mock.bar();
//! # }
//! ```
//!
//! ## External traits
//!
//! Mockall can mock traits and structs defined in external crates that are
//! beyond your control, but you must use [`mock!`] instead of
//! [`#[automock]`](attr.automock.html).  Mock an external trait like this:
//!
//! ```
//! # use mockall::*;
//! mock! {
//!     MyStruct {}     // Name of the mock struct, less the "Mock" prefix
//!     impl Clone for MyStruct {   // specification of the trait to mock
//!         fn clone(&self) -> Self;
//!     }
//! }
//!
//! # fn main() {
//! let mut mock1 = MockMyStruct::new();
//! let mock2 = MockMyStruct::new();
//! mock1.expect_clone()
//!     .return_once(move || mock2);
//! let cloned = mock1.clone();
//! # }
//! ```
//!
//! ## Static methods
//!
//! Mockall can also mock static methods.  But be careful!  The expectations are
//! global.  If you want to use a static method in multiple tests, you must
//! provide your own synchronization. See the [`synchronization
//! example`](https://github.com/asomers/mockall/blob/master/mockall/examples/synchronization.rs)
//! for a basic implementation. For ordinary methods, expectations are
//! set on the mock object.  But static methods don't have any mock object.
//! Instead, you must create a `Context` object just to set their expectations.
//!
//! ```
//! # use mockall::*;
//! #[automock]
//! pub trait A {
//!     fn foo() -> u32;
//! }
//!
//! let ctx = MockA::foo_context();
//! ctx.expect().returning(|| 99);
//! assert_eq!(99, MockA::foo());
//! ```
//!
//! A common pattern is mocking a trait with a constructor method.  In this case,
//! you can easily set the mock constructor method to return a mock object.
//!
//! ```
//! # use mockall::*;
//! struct Foo{}
//! #[automock]
//! impl Foo {
//!     fn from_i32(x: i32) -> Self {
//!         // ...
//!         # unimplemented!()
//!     }
//!     fn foo(&self) -> i32 {
//!         // ...
//!         # unimplemented!()
//!     }
//! }
//!
//! # fn main() {
//! let ctx = MockFoo::from_i32_context();
//! ctx.expect()
//!     .returning(|x| {
//!         let mut mock = MockFoo::default();
//!         mock.expect_foo()
//!             .return_const(x);
//!         mock
//!     });
//! let foo = MockFoo::from_i32(42);
//! assert_eq!(42, foo.foo());
//! # }
//! ```
//!
//! ### Generic static methods
//!
//! Mocking static methods of generic structs or traits, whether or not the
//! methods themselves are generic, should work seamlessly.
//!
//! ```
//! # use mockall::*;
//! #[automock]
//! trait Foo<T: 'static> {
//!     fn new(t: T) -> MockFoo<T>;
//! }
//!
//! # fn main() {
//! let ctx = MockFoo::<u32>::new_context();
//! ctx.expect()
//!     .returning(|_| MockFoo::default());
//! let mock = MockFoo::<u32>::new(42u32);
//! # }
//! ```
//!
//! ### Context checkpoints
//!
//! The context object cleans up all expectations when it leaves scope.  It also
//! has a `checkpoint` method that functions just like a mock object's
//! `checkpoint` method.
//!
//! ```should_panic
//! # use mockall::*;
//! #[automock]
//! pub trait A {
//!     fn foo() -> u32;
//! }
//!
//! let ctx = MockA::foo_context();
//! ctx.expect()
//!     .times(1)
//!     .returning(|| 99);
//! ctx.checkpoint();   // Panics!
//! ```
//!
//! A mock object's checkpoint method does *not* checkpoint static methods.
//! This behavior is useful when using multiple mock objects at once.  For
//! example:
//!
//! ```
//! # use mockall::*;
//! #[automock]
//! pub trait A {
//!     fn build() -> Self;
//!     fn bar(&self) -> i32;
//! }
//!
//! # fn main() {
//! let ctx = MockA::build_context();
//! ctx.expect()
//!     .times(2)
//!     .returning(|| MockA::default());
//! let mut mock0 = MockA::build();
//! mock0.expect_bar().return_const(4);
//! mock0.bar();
//! mock0.checkpoint();     // Does not checkpoint the build method
//! let mock1 = MockA::build();
//! # }
//! ```
//!
//! One more thing: Mockall normally creates a zero-argument `new` method for
//! every mock struct.  But it *won't* do that when mocking a struct that
//! already has a method named `new`.  The `default` method will still be
//! present.
//!
//! ## Modules
//!
//! In addition to mocking types, Mockall can also derive mocks for
//! entire modules of Rust functions.  Mockall will generate a new module named
//! "mock_xxx", if "xxx" is the original module's name.  You can also use
//! `#[double]` to selectively import the mock module.
//!
//! Be careful!  Module functions are static and so have the same caveats as
//! [static methods](#static-methods) described above.
//!
//! ```
//! # use mockall::*;
//! # use mockall_double::*;
//! mod outer {
//!     use mockall::automock;
//!     #[automock()]
//!     pub(super) mod inner {
//!         pub fn bar(x: u32) -> i64 {
//!             // ...
//!             # 4
//!         }
//!     }
//! }
//!
//! #[double]
//! use outer::inner;
//!
//! #[cfg(test)]
//! mod t {
//!     use super::*;
//!
//!     #[test]
//!     fn test_foo_bar() {
//!         let ctx = inner::bar_context();
//!         ctx.expect()
//!             .returning(|x| i64::from(x + 1));
//!         assert_eq!(5, inner::bar(4));
//!     }
//! }
//! # fn main() {}
//! ```
//!
//! ### Foreign functions
//!
//! One reason to mock modules is when working with foreign functions.  Modules
//! may contain foreign functions, even though structs and traits may not.  Like
//! static methods, the expectations are global.
//!
//! ```
//! # use mockall_double::*;
//! mod outer {
//!     # use mockall::*;
//!     #[automock]
//!     pub mod ffi {
//!         extern "C" {
//!             pub fn foo(x: u32) -> i64;
//!         }
//!     }
//! }
//!
//! #[double]
//! use outer::ffi;
//!
//! fn do_stuff() -> i64 {
//!     unsafe{ ffi::foo(42) }
//! }
//!
//! #[cfg(test)]
//! mod t {
//!     use super::*;
//!
//!     #[test]
//!     fn test_foo() {
//!         let ctx = ffi::foo_context();
//!         ctx.expect()
//!             .returning(|x| i64::from(x + 1));
//!         assert_eq!(43, do_stuff());
//!     }
//! }
//! # fn main() {}
//! ```
//!
//! ## Debug
//!
//! `#[automock]` will automatically generate `Debug` impls when mocking traits
//! and struct impls.  `mock!` will too, if you add a `#[derive(Debug)]`, like
//! this:
//! ```no_run
//! # use mockall::*;
//! mock! {
//!     #[derive(Debug)]
//!     pub Foo {}
//! }
//! # fn main() {
//! #     format!("{:?}", &MockFoo::default());
//! # }
//! ```
//!
//! ## Async Traits
//!
//! Async traits aren't yet (as of 1.47.0) a part of the Rust language.  But
//! they're available from the
//! [`async_trait`](https://docs.rs/async-trait/0.1.38/async_trait/) crate.
//! Mockall is compatible with this crate, with two important limitations:
//!
//! * The `#[automock]` attribute must appear _before_ the `#[async_trait]`
//! attribute.
//!
//! * The `#[async_trait]` macro must be imported with its canonical name.
//!
//! ```
//! # use async_trait::async_trait;
//! # use mockall::*;
//! // async_trait works with both #[automock]
//! #[automock]
//! #[async_trait]
//! pub trait Foo {
//!    async fn foo(&self) -> u32;
//! }
//! // and mock!
//! mock! {
//!     pub Bar {}
//!     #[async_trait]
//!     impl Foo for Bar {
//!         async fn foo(&self) -> u32;
//!     }
//! }
//! # fn main() {}
//! ```
//!
//! ## Crate features
//!
//! Mockall has a **nightly** feature.  Currently this feature has two
//! effects:
//!
//! * The compiler will produce better error messages.
//!
//! * Expectations for methods whose return type implements `Default` needn't
//!   have their return values explicitly set.  Instead, they will automatically
//!   return the default value.
//!
//! With **nightly** enabled, you can omit the return value like this:
#![cfg_attr(feature = "nightly", doc = "```")]
#![cfg_attr(not(feature = "nightly"), doc = "```should_panic")]
//! # use mockall::*;
//! #[automock]
//! trait Foo {
//!     fn foo(&self) -> Vec<u32>;
//! }
//!
//! let mut mock = MockFoo::new();
//! mock.expect_foo();
//! assert!(mock.foo().is_empty());
//! ```
//!
//! ## Examples
//!
//! For additional examples of Mockall in action, including detailed
//! documentation on the autogenerated methods, see
//! [`examples`](examples).
//!
//! [`Predicate`]: trait.Predicate.html
//! [`Sequence`]: Sequence
//! [`cfg-if`]: https://crates.io/crates/cfg-if
//! [`function`]: predicate/fn.function.html
//! [`mock!`]: macro.mock.html
//! [`predicate`]: predicate/index.html

#![cfg_attr(feature = "nightly", feature(specialization))]
// Allow the incomplete_feature warning for specialization.  We know it's
// incomplete; that's why it's guarded by the "nightly" feature.
#![cfg_attr(feature = "nightly", allow(incomplete_features))]

#![cfg_attr(feature = "nightly", feature(doc_cfg))]
#![cfg_attr(test, deny(warnings))]
#![warn(missing_docs)]

use downcast::*;
use std::{
    any,
    fmt::{self, Debug, Formatter},
    marker::PhantomData,
    ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo,
          RangeToInclusive},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering}
    },
};

#[doc(hidden)]
pub use downcast::{Any, Downcast};
#[doc(hidden)]
pub use fragile::Fragile;

/// For mocking static methods
#[doc(hidden)]
pub use lazy_static::lazy_static;

pub use predicates::{
    boolean::PredicateBooleanExt,
    prelude::{
        Predicate, PredicateBoxExt, PredicateFileContentExt, PredicateStrExt,
        predicate
    }
};
#[doc(hidden)]
pub use predicates_tree::CaseTreeExt;

#[cfg(doc)]
extern crate self as mockall;
#[cfg(doc)]
pub mod examples;

/// Automatically generate mock types for structs and traits.
///
/// This is by far the easiest way to use Mockall.  It works on almost all
/// traits, and almost all structs that have a single `impl` block.  In either
/// case, it will generate a mock struct whose name is the name of the mocked
/// struct/trait prepended with "Mock".  For each method of the original, the
/// mock struct will have a method named `expect_whatever` that allows you to
/// set expectations.  There will also be one `checkpoint` method that calls
/// [`checkpoint`] for every single mocked method.
///
/// # Examples
///
/// The simplest use case is mocking a no-frills trait
/// ```
/// # use mockall_derive::*;
/// #[automock]
/// pub trait Foo {
///     fn foo(&self, key: i16);
/// }
///
/// let mock = MockFoo::new();
/// ```
///
/// Mocking a structure:
/// ```
/// # use mockall_derive::*;
/// struct Foo {}
/// #[automock]
/// impl Foo {
///     fn foo(&self) -> u32 {
///         // ...
///         # unimplemented!()
///     }
/// }
/// ```
///
/// You can also mock a trait impl on a struct:
/// ```
/// # use mockall_derive::*;
/// pub trait Foo {
///     fn foo(&self, key: i16);
/// }
/// struct Bar{}
/// #[automock]
/// impl Foo for Bar {
///     fn foo(&self, key: i16){
///         // ...
///         # unimplemented!()
///     }
/// }
///
/// let mock = MockBar::new();
/// ```
///
/// Mocking a trait with associated types requires adding a metaitem to the
/// attribute:
/// ```
/// # use mockall_derive::*;
/// #[automock(type Item=u32;)]
/// trait Foo {
///     type Item;
///     fn foo(&self) -> Self::Item;
/// }
/// ```
///
/// Finally, `#[automock]` can also mock foreign functions.  This requires
/// another metaitem to specify the mock module name.
///
/// ```
/// # use mockall_derive::*;
/// #[automock(mod mock_ffi;)]
/// extern "C" {
///     pub fn foo() -> u32;
/// }
/// ```
///
/// [`checkpoint`]: ../mockall/index.html#checkpoints
///
/// # Limitations
///
/// `#[automock]` can't handle everything.  There are some cases where
/// you will need to use [`mock`] instead:
/// * Mocking a struct that has multiple `impl` blocks, including
///   structs that implement traits.
/// * Mocking a struct or trait defined in another crate.
/// * Mocking a trait with trait bounds.
/// * If the autogenerated "MockFoo" name isn't acceptable, and you want
///   to choose your own name for the mock structure.
pub use mockall_derive::automock;

/// Manually mock a structure.
///
/// Sometimes `automock` can't be used.  In those cases you can use `mock!`,
/// which basically involves repeating the struct's or trait's definitions.
///
/// The format is:
///
/// * Optional visibility specifier
/// * Real structure name and generics fields
/// * 0 or more methods of the structure, written without bodies, enclosed in a
///   {} block
/// * 0 or more impl blocks implementing traits on the structure, also without
///   bodies.
///
/// # Examples
///
/// Mock a trait.  This is the simplest use case.
/// ```
/// # use mockall_derive::mock;
/// trait Foo {
///     fn foo(&self, x: u32);
/// }
/// mock!{
///     pub MyStruct<T: Clone + 'static> {
///         fn bar(&self) -> u8;
///     }
///     impl<T: Clone + 'static> Foo for MyStruct<T> {
///         fn foo(&self, x: u32);
///     }
/// }
/// # fn main() {}
/// ```
/// Mocking an unsupported `#[derive(X)]` attribute, e.g. `Clone`, is
/// similar.
/// ```
/// # use mockall_derive::mock;
/// #[derive(Clone)]
/// struct MyStruct;
///
/// mock!{
///     pub MyStruct {
///         fn bar(&self);
///     }
///     impl Clone for MyStruct {
///         fn clone(&self) -> Self;
///     }
/// }
/// # fn main() {}
/// ```
///
/// When mocking a generic struct's implementation of a generic trait, use the
/// same namespace for their generic parameters.  For example, if you wanted to
/// mock `Rc`, do
/// ```
/// # use mockall_derive::mock;
/// mock!{
///     pub Rc<T: 'static> {}
///     impl<T: 'static> AsRef<T> for Rc<T> {
///         fn as_ref(&self) -> &T;
///     }
/// }
/// # fn main() {}
/// ```
/// *not*
/// ```compile_fail
/// # use mockall_derive::mock;
/// mock!{
///     pub Rc<Q: 'static> {}
///     impl<T: 'static> AsRef<T> for Rc<T> {
///         fn as_ref(&self) -> &T;
///     }
/// }
/// # fn main() {}
/// ```
/// Associated types can easily be mocked by specifying a concrete type in the
/// `mock!{}` invocation.
/// ```
/// # use mockall_derive::mock;
/// mock!{
///     MyIter {}
///     impl Iterator for MyIter {
///         type Item=u32;
///
///         fn next(&mut self) -> Option<<Self as Iterator>::Item>;
///     }
/// }
/// # fn main() {}
/// ```
pub use mockall_derive::mock;

#[doc(hidden)]
pub trait AnyExpectations : Any + Send + Sync {}
downcast!(dyn AnyExpectations);

#[doc(hidden)]
pub trait ReturnDefault<O> {
    fn maybe_return_default() -> Option<O>;
    fn return_default() -> Result<O, &'static str>;
}

#[derive(Default)]
#[doc(hidden)]
pub struct DefaultReturner<O>(PhantomData<O>);

::cfg_if::cfg_if! {
    if #[cfg(feature = "nightly")] {
        impl<O> ReturnDefault<O> for DefaultReturner<O> {
            default fn maybe_return_default() -> Option<O> {
                None
            }

            default fn return_default() -> Result<O, &'static str> {
                Err("Can only return default values for types that impl std::Default")
            }
        }

        impl<O: Default> ReturnDefault<O> for DefaultReturner<O> {
            fn maybe_return_default() -> Option<O> {
                Some(O::default())
            }

            fn return_default() -> Result<O, &'static str> {
                Ok(O::default())
            }
        }
    } else {
        impl<O> ReturnDefault<O> for DefaultReturner<O> {
            fn maybe_return_default() -> Option<O> {
                None
            }

            fn return_default() -> Result<O, &'static str> {
                Err("Returning default values requires the \"nightly\" feature")
            }
        }
    }
}

#[doc(hidden)]
pub struct MaybeDebugger<'a, T>(pub &'a T);
::cfg_if::cfg_if! {
    if #[cfg(feature = "nightly")] {
        impl<'a, T> Debug for MaybeDebugger<'a, T> {
            default fn fmt(&self, f: &mut Formatter<'_>)
                -> Result<(), fmt::Error>
            {
                write!(f, "?")
            }
        }
        impl<'a, T: Debug> Debug for MaybeDebugger<'a, T> {
            fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
                self.0.fmt(f)
            }
        }
    } else {
        impl<'a, T> Debug for MaybeDebugger<'a, T> {
            fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
                write!(f, "?")
            }
        }
    }
}

// Though it's not entirely correct, we treat usize::max_value() as
// approximately infinity.
#[derive(Debug)]
#[doc(hidden)]
pub struct TimesRange(Range<usize>);

impl Default for TimesRange {
    fn default() -> TimesRange {
        // By default, allow any number of calls
        TimesRange(0..usize::max_value())
    }
}

impl From<usize> for TimesRange {
    fn from(n: usize) -> TimesRange {
        TimesRange(n..(n+1))
    }
}

impl From<Range<usize>> for TimesRange {
    fn from(r: Range<usize>) -> TimesRange {
        assert!(r.end > r.start, "Backwards range");
        TimesRange(r)
    }
}

impl From<RangeFrom<usize>> for TimesRange {
    fn from(r: RangeFrom<usize>) -> TimesRange {
        TimesRange(r.start..usize::max_value())
    }
}

impl From<RangeFull> for TimesRange {
    fn from(_: RangeFull) -> TimesRange {
        TimesRange(0..usize::max_value())
    }
}

impl From<RangeInclusive<usize>> for TimesRange {
    fn from(r: RangeInclusive<usize>) -> TimesRange {
        assert!(r.end() >= r.start(), "Backwards range");
        TimesRange(*r.start()..*r.end() + 1)
    }
}

impl From<RangeTo<usize>> for TimesRange {
    fn from(r: RangeTo<usize>) -> TimesRange {
        TimesRange(0..r.end)
    }
}

impl From<RangeToInclusive<usize>> for TimesRange {
    fn from(r: RangeToInclusive<usize>) -> TimesRange {
        TimesRange(0..r.end + 1)
    }
}

#[derive(PartialEq)]
#[doc(hidden)]
pub enum ExpectedCalls {
    Satisfied,
    TooMany,
    TooFew,
}

#[derive(Debug, Default)]
#[doc(hidden)]
pub struct Times{
    /// How many times has the expectation already been called?
    count: AtomicUsize,
    range: TimesRange
}

#[doc(hidden)]
impl Times {
    pub fn call(&self) -> Result<(), String> {
        let count = self.count.fetch_add(1, Ordering::Relaxed) + 1;
        if count >= self.range.0.end {
            if self.range.0.end == 1 {
                Err("should not have been called".to_owned())
            } else {
                Err(format!(
                    "called {} times which is more than the expected {}",
                    count,
                    self.range.0.end - 1
                ))
            }
        } else {
            Ok(())
        }
    }

    pub fn any(&mut self) {
        self.range.0 = 0..usize::max_value();
    }

    /// Return how many times this expectation has been called
    pub fn count(&self) -> usize {
        self.count.load(Ordering::Relaxed)
    }

    /// Has this expectation already been called the maximum allowed number of
    /// times?
    pub fn is_done(&self) -> bool {
        self.count.load(Ordering::Relaxed) >= self.range.0.end - 1
    }

    /// Is it required that this expectation be called an exact number of times,
    /// or may it be satisfied by a range of call counts?
    pub fn is_exact(&self) -> bool {
        (self.range.0.end - self.range.0.start) == 1
    }

    /// Has this expectation already been called the expected number of times?
    /// If not, was it too many or too few?
    pub fn is_satisfied(&self) -> ExpectedCalls {
        let satisfied_lower_bound = self.count.load(Ordering::Relaxed) >= self.range.0.start;
        let satisfied_upper_bound = self.count.load(Ordering::Relaxed) < self.range.0.end;
        if satisfied_lower_bound && satisfied_upper_bound {
            ExpectedCalls::Satisfied
        } else if satisfied_lower_bound {
            ExpectedCalls::TooMany
        } else {
            ExpectedCalls::TooFew
        }
    }

    /// The maximum number of times that this expectation must be called
    pub fn maximum(&self) -> usize {
        self.range.0.end - 1
    }

    /// The minimum number of times that this expectation must be called
    pub fn minimum(&self) -> usize {
        self.range.0.start
    }

    // https://github.com/rust-lang/rust-clippy/issues/3307
    #[allow(clippy::range_plus_one)]
    pub fn n(&mut self, n: usize) {
        self.range.0 = n..(n+1);
    }

    pub fn never(&mut self) {
        self.range.0 = 0..1;
    }

    pub fn range(&mut self, range: Range<usize>) {
        assert!(range.end > range.start, "Backwards range");
        self.range.0 = range;
    }

    pub fn times<T: Into<TimesRange>>(&mut self, t: T) {
        self.range = t.into();
    }
}

/// Non-generic keys to `GenericExpectation` internal storage
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Key(any::TypeId);

#[doc(hidden)]
impl Key {
    pub fn new<T: 'static + ?Sized>() -> Self {
        Key(any::TypeId::of::<T>())
    }
}

#[doc(hidden)]
pub struct SeqHandle {
    inner: Arc<SeqInner>,
    seq: usize
}

impl SeqHandle {
    /// Tell the Sequence that this expectation has been fully satisfied
    pub fn satisfy(&self) {
        self.inner.satisfy(self.seq);
    }

    /// Verify that this handle was called in the correct order
    pub fn verify(&self, desc: &str) {
        self.inner.verify(self.seq, desc);
    }
}

#[derive(Default)]
struct SeqInner {
    satisfaction_level: AtomicUsize,
}

impl SeqInner {
    /// Record the call identified by `seq` as fully satisfied.
    fn satisfy(&self, seq: usize) {
        let old_sl = self.satisfaction_level.fetch_add(1, Ordering::Relaxed);
        assert_eq!(old_sl, seq, "Method sequence violation.  Was an already-satisfied method called another time?");
    }

    /// Verify that the call identified by `seq` was called in the correct order
    fn verify(&self, seq: usize, desc: &str) {
        assert_eq!(seq, self.satisfaction_level.load(Ordering::Relaxed),
            "{}: Method sequence violation", desc)
    }
}

/// Used to enforce that mock calls must happen in the sequence specified.
///
/// Each expectation must expect to be called a fixed number of times.  Once
/// satisfied, the next expectation in the sequence will expect to be called.
///
/// # Examples
/// ```
/// # use mockall::*;
/// #[automock]
/// trait Foo {
///     fn foo(&self);
///     fn bar(&self) -> u32;
/// }
/// let mut seq = Sequence::new();
///
/// let mut mock0 = MockFoo::new();
/// let mut mock1 = MockFoo::new();
///
/// mock0.expect_foo()
///     .times(1)
///     .returning(|| ())
///     .in_sequence(&mut seq);
///
/// mock1.expect_bar()
///     .times(1)
///     .returning(|| 42)
///     .in_sequence(&mut seq);
///
/// mock0.foo();
/// mock1.bar();
/// ```
///
/// It is an error to add an expectation to a `Sequence` if its call count is
/// unspecified.
/// ```should_panic(expected = "with an exact call count")
/// # use mockall::*;
/// #[automock]
/// trait Foo {
///     fn foo(&self);
/// }
/// let mut seq = Sequence::new();
///
/// let mut mock = MockFoo::new();
/// mock.expect_foo()
///     .returning(|| ())
///     .in_sequence(&mut seq);  // panics!
/// ```
#[derive(Default)]
pub struct Sequence {
    inner: Arc<SeqInner>,
    next_seq: usize,
}

impl Sequence {
    /// Create a new empty [`Sequence`]
    pub fn new() -> Self {
        Self::default()
    }

    /// Not for public consumption, but it must be public so the generated code
    /// can call it.
    #[doc(hidden)]
    pub fn next_handle(&mut self) -> SeqHandle {
        let handle = SeqHandle{inner: self.inner.clone(), seq: self.next_seq};
        self.next_seq += 1;
        handle
    }
}
