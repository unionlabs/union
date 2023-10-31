/// Waits on multiple concurrent branches, returning when **all** branches
/// complete with `Ok(_)` or on the first `Err(_)`.
///
/// The `try_join!` macro must be used inside of async functions, closures, and
/// blocks.
///
/// Similar to [`join!`], the `try_join!` macro takes a list of async
/// expressions and evaluates them concurrently on the same task. Each async
/// expression evaluates to a future and the futures from each expression are
/// multiplexed on the current task. The `try_join!` macro returns when **all**
/// branches return with `Ok` or when the **first** branch returns with `Err`.
///
/// [`join!`]: macro@join
///
/// # Notes
///
/// The supplied futures are stored inline and does not require allocating a
/// `Vec`.
///
/// ### Runtime characteristics
///
/// By running all async expressions on the current task, the expressions are
/// able to run **concurrently** but not in **parallel**. This means all
/// expressions are run on the same thread and if one branch blocks the thread,
/// all other expressions will be unable to continue. If parallelism is
/// required, spawn each async expression using [`tokio::spawn`] and pass the
/// join handle to `try_join!`.
///
/// [`tokio::spawn`]: crate::spawn
///
/// # Examples
///
/// Basic try_join with two branches.
///
/// ```
/// async fn do_stuff_async() -> Result<(), &'static str> {
///     // async work
/// # Ok(())
/// }
///
/// async fn more_async_work() -> Result<(), &'static str> {
///     // more here
/// # Ok(())
/// }
///
/// #[tokio::main]
/// async fn main() {
///     let res = tokio::try_join!(
///         do_stuff_async(),
///         more_async_work());
///
///     match res {
///          Ok((first, second)) => {
///              // do something with the values
///          }
///          Err(err) => {
///             println!("processing failed; error = {}", err);
///          }
///     }
/// }
/// ```
///
/// Using `try_join!` with spawned tasks.
///
/// ```
/// use tokio::task::JoinHandle;
///
/// async fn do_stuff_async() -> Result<(), &'static str> {
///     // async work
/// # Err("failed")
/// }
///
/// async fn more_async_work() -> Result<(), &'static str> {
///     // more here
/// # Ok(())
/// }
///
/// async fn flatten<T>(handle: JoinHandle<Result<T, &'static str>>) -> Result<T, &'static str> {
///     match handle.await {
///         Ok(Ok(result)) => Ok(result),
///         Ok(Err(err)) => Err(err),
///         Err(err) => Err("handling failed"),
///     }
/// }
///
/// #[tokio::main]
/// async fn main() {
///     let handle1 = tokio::spawn(do_stuff_async());
///     let handle2 = tokio::spawn(more_async_work());
///     match tokio::try_join!(flatten(handle1), flatten(handle2)) {
///         Ok(val) => {
///             // do something with the values
///         }
///         Err(err) => {
///             println!("Failed with {}.", err);
///             # assert_eq!(err, "failed");
///         }
///     }
/// }
/// ```
#[macro_export]
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
macro_rules! try_join {
    (@ {
        // One `_` for each branch in the `try_join!` macro. This is not used once
        // normalization is complete.
        ( $($count:tt)* )

        // The expression `0+1+1+ ... +1` equal to the number of branches.
        ( $($total:tt)* )

        // Normalized try_join! branches
        $( ( $($skip:tt)* ) $e:expr, )*

    }) => {{
        use $crate::macros::support::{maybe_done, poll_fn, Future, Pin};
        use $crate::macros::support::Poll::{Ready, Pending};

        // Safety: nothing must be moved out of `futures`. This is to satisfy
        // the requirement of `Pin::new_unchecked` called below.
        let mut futures = ( $( maybe_done($e), )* );

        // Each time the future created by poll_fn is polled, a different future will be polled first
        // to ensure every future passed to join! gets a chance to make progress even if
        // one of the futures consumes the whole budget.
        //
        // This is number of futures that will be skipped in the first loop
        // iteration the next time.
        let mut skip_next_time: u32 = 0;

        poll_fn(move |cx| {
            const COUNT: u32 = $($total)*;

            let mut is_pending = false;

            let mut to_run = COUNT;

            // The number of futures that will be skipped in the first loop iteration
            let mut skip = skip_next_time;

            skip_next_time = if skip + 1 == COUNT { 0 } else { skip + 1 };

            // This loop runs twice and the first `skip` futures
            // are not polled in the first iteration.
            loop {
            $(
                if skip == 0 {
                    if to_run == 0 {
                        // Every future has been polled
                        break;
                    }
                    to_run -= 1;

                    // Extract the future for this branch from the tuple.
                    let ( $($skip,)* fut, .. ) = &mut futures;

                    // Safety: future is stored on the stack above
                    // and never moved.
                    let mut fut = unsafe { Pin::new_unchecked(fut) };

                    // Try polling
                    if fut.as_mut().poll(cx).is_pending() {
                        is_pending = true;
                    } else if fut.as_mut().output_mut().expect("expected completed future").is_err() {
                        return Ready(Err(fut.take_output().expect("expected completed future").err().unwrap()))
                    }
                } else {
                    // Future skipped, one less future to skip in the next iteration
                    skip -= 1;
                }
            )*
            }

            if is_pending {
                Pending
            } else {
                Ready(Ok(($({
                    // Extract the future for this branch from the tuple.
                    let ( $($skip,)* fut, .. ) = &mut futures;

                    // Safety: future is stored on the stack above
                    // and never moved.
                    let mut fut = unsafe { Pin::new_unchecked(fut) };

                    fut
                        .take_output()
                        .expect("expected completed future")
                        .ok()
                        .expect("expected Ok(_)")
                },)*)))
            }
        }).await
    }};

    // ===== Normalize =====

    (@ { ( $($s:tt)* ) ( $($n:tt)* ) $($t:tt)* } $e:expr, $($r:tt)* ) => {
      $crate::try_join!(@{ ($($s)* _) ($($n)* + 1) $($t)* ($($s)*) $e, } $($r)*)
    };

    // ===== Entry point =====

    ( $($e:expr),* $(,)?) => {
        $crate::try_join!(@{ () (0) } $($e,)*)
    };
}
