# Version 4.3.0

- Bump MSRV to Rust 1.47. (#30)
- Evaluate the layouts for the tasks at compile time. (#30)
- Add layout_info field to TaskVTable so that debuggers can decode raw tasks. (#29)

# Version 4.2.0

- Add `Task::is_finished`. (#19)

# Version 4.1.0

- Add `FallibleTask`. (#21)

# Version 4.0.3

- Document the return value of `Runnable::run()` better.

# Version 4.0.2

- Nits in the docs.

# Version 4.0.1

- Nits in the docs.

# Version 4.0.0

- Rename `Task` to `Runnable`.
- Rename `JoinHandle` to `Task`.
- Cancel `Task` on drop.
- Add `Task::detach()` and `Task::cancel()`.
- Add `spawn_unchecked()`.

# Version 3.0.0

- Use `ThreadId` in `spawn_local` because OS-provided IDs can get recycled.
- Add `std` feature to `Cargo.toml`.

# Version 2.1.1

- Allocate large futures on the heap.

# Version 2.1.0

- `JoinHandle` now only evaluates after the task's future has been dropped.

# Version 2.0.0

- Return `true` in `Task::run()`.

# Version 1.3.1

- Make `spawn_local` available only on unix and windows.

# Version 1.3.0

- Add `waker_fn`.

# Version 1.2.1

- Add the `no-std` category to the package.

# Version 1.2.0

- The crate is now marked with `#![no_std]`.
- Add `Task::waker` and `JoinHandle::waker`.
- Add `Task::into_raw` and `Task::from_raw`.

# Version 1.1.1

- Fix a use-after-free bug where the schedule function is dropped while running.

# Version 1.1.0

- If a task is dropped or canceled outside the `run` method, it gets re-scheduled.
- Add `spawn_local` constructor.

# Version 1.0.0

- Initial release
