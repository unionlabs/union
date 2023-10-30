
scopeguard
==========

Rust crate for a convenient RAII scope guard that will run a given closure when
it goes out of scope, even if the code between panics (assuming unwinding panic).

The `defer!` macro and `guard` are `no_std` compatible (require only core),
but the on unwinding / not on uwinding strategies requires linking to `std`.

Requires Rust 1.20.


Please read the `API documentation here`__

__ https://docs.rs/scopeguard/

|build_status|_ |crates|_

.. |build_status| image:: https://travis-ci.org/bluss/scopeguard.svg
.. _build_status: https://travis-ci.org/bluss/scopeguard

.. |crates| image:: http://meritbadge.herokuapp.com/scopeguard
.. _crates: https://crates.io/crates/scopeguard

How to use
----------

.. code:: rust

    #[macro_use(defer)] extern crate scopeguard;

    use scopeguard::guard;

    fn f() {
        defer!(println!("Called at return or panic"));
        panic!();
    }

    use std::fs::File;
    use std::io::Write;

    fn g() {
        let f = File::create("newfile.txt").unwrap();
        let mut file = guard(f, |f| {
            // write file at return or panic
            let _ = f.sync_all();
        });
        // Access the file through the scope guard itself
        file.write_all(b"test me\n").unwrap();
    }

Recent Changes
--------------

- 1.1.0

  - Change macros (``defer!``, ``defer_on_success!`` and ``defer_on_unwind!``)
    to accept statements. (by @konsumlamm)

- 1.0.0

  - Change the closure type from ``FnMut(&mut T)`` to ``FnOnce(T)``:
    Passing the inner value by value instead of a mutable reference is a
    breaking change, but allows the guard closure to consume it. (by @tormol)

  - Add ``defer_on_success!{}``, ``guard_on_success()`` and ``OnSuccess``
    strategy, which triggers when scope is exited *without* panic. It's the
    opposite to ``OnUnwind`` / ``guard_on_unwind()`` / ``defer_on_unwind!{}``.

  - Add ``ScopeGuard::into_inner()``, which "defuses" the guard and returns the
    guarded value. (by @tormol)

  - Implement ``Sync`` for guards with non-``Sync`` closures.

  - Require Rust 1.20

- 0.3.3

  - Use ``#[inline]`` on a few more functions by @stjepang (#14)
  - Add examples to crate documentation

- 0.3.2

  - Add crate categories

- 0.3.1

  - Add ``defer_on_unwind!``, ``Strategy`` trait
  - Rename ``Guard`` → ``ScopeGuard``
  - Add ``ScopeGuard::with_strategy``.
  - ``ScopeGuard`` now implements ``Debug``.
  - Require Rust 1.11

- 0.2.0

  - Require Rust 1.6
  - Use `no_std` unconditionally
  - No other changes

- 0.1.2

  - Add macro ``defer!()``
