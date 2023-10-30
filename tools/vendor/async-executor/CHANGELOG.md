# Version 1.4.1

- Remove dependency on deprecated `vec-arena`. (#23)

# Version 1.4.0

- Add `Executor::is_empty()` and `LocalExecutor::is_empty()`.

# Version 1.3.0

- Parametrize executors over a lifetime to allow spawning non-`static` futures.

# Version 1.2.0

- Update `async-task` to v4.

# Version 1.1.1

- Replace `AtomicU64` with `AtomicUsize`.

# Version 1.1.0

- Use atomics to make `Executor::run()` and `Executor::tick()` futures `Send + Sync`.

# Version 1.0.0

- Stabilize.

# Version 0.2.1

- Add `try_tick()` and `tick()` methods.

# Version 0.2.0

- Redesign the whole API.

# Version 0.1.2

- Add the `Spawner` API.

# Version 0.1.1

- Initial version
