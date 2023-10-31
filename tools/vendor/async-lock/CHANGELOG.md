# Version 2.6.0

- Add `OnceCell`. (#27)
- Support wasm64.

# Version 2.5.0

- Fix an issue where the future returned by `Mutex::lock_arc`/`Semaphore::acquire_arc` holds a reference to `self`. (#20, #21)

# Version 2.4.0

- Add WASM support. (#14)

# Version 2.3.0

- Merge all subcrates.

# Version 2.2.0

- Add functions to upgrade and downgrade `RwLock` guards.
- Make all constructors `const fn`.

# Version 2.1.3

- Add `#![forbid(unsafe_code)]`.

# Version 2.1.2

- Update dependencies.

# Version 2.1.1

- Update crate description.

# Version 2.1.0

- Add `Barrier` and `Semaphore`.

# Version 2.0.1

- Update crate description.

# Version 2.0.0

- Only re-export `async-mutex` and `async-rwlock`.

# Version 1.1.5

- Replace the implementation with `async-mutex`.

# Version 1.1.4

- Replace `usize::MAX` with `std::usize::MAX`.

# Version 1.1.3

- Update dependencies.

# Version 1.1.2

- Fix a deadlock issue.

# Version 1.1.1

- Fix some typos.

# Version 1.1.0

- Make locking fair.
- Add `LockGuard::source()`.

# Version 1.0.2

- Bump the `event-listener` version.
- Add tests.

# Version 1.0.1

- Update Cargo categories.

# Version 1.0.0

- Initial version
