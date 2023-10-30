# Version 1.2.0

- Return `Task` from `unblock` instead of returning opaque type. (#25)

# Version 1.1.0

- Add an environment variable to customize the maximum number of threads. (#21)

# Version 1.0.2

- Update `futures-lite`.

# Version 1.0.1

- Use `async-task`.

# Version 1.0.0

- Stabilize.

# Version 0.6.1

- Add probabilistic yielding to improve fairness.

# Version 0.6.0

- Remove the `unblock!` macro.

# Version 0.5.2

- Implement `Sync` for `Unblock`.

# Version 0.5.1

- Add `Unblock::with_capacity()`.
- Add `unblock()` function.
- An optimization in task spawning.

# Version 0.5.0

- Simplify the API to just `unblock!` and `Unblock`.

# Version 0.4.7

- Simplify dependencies for faster compilation.

# Version 0.4.6

- Update doc comment on `Unblock`.

# Version 0.4.5

- Implement `AsyncSeek`/`Seek` for `Unblock`/`BlockOn`.

# Version 0.4.4

- Remove the initial poll in block_on that caused lost wakeups.

# Version 0.4.3

- Fix a bug where a closed `Receiver` causes panics.

# Version 0.4.2

- Start thread numbering from 1.

# Version 0.4.1

- Attach names to spawned threads.

# Version 0.4.0

- Remove `Future` impl for `Blocking`.
- Add `unblock()`.
- Rename `blocking!` to `unblock!`.
- Rename `Blocking` to `Unblock`.
- Add `block_on()`, `block_on!`, and `BlockOn`.

# Version 0.3.2

- Make `Blocking` implement `Send` in more cases.

# Version 0.3.1

- Add `Blocking::with_mut()`.

# Version 0.3.0

- Remove `Blocking::spawn()`.
- Implement `Future` for `Blocking` only when the inner type is a `FnOnce`.

# Version 0.2.0

- Initial version

# Version 0.1.0

- Reserved crate name
