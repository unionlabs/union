#![cfg_attr(not(feature = "full"), allow(dead_code))]

//! Yield points for improved cooperative scheduling.
//!
//! Documentation for this can be found in the [`tokio::task`] module.
//!
//! [`tokio::task`]: crate::task.

// ```ignore
// # use tokio_stream::{Stream, StreamExt};
// async fn drop_all<I: Stream + Unpin>(mut input: I) {
//     while let Some(_) = input.next().await {
//         tokio::coop::proceed().await;
//     }
// }
// ```
//
// The `proceed` future will coordinate with the executor to make sure that
// every so often control is yielded back to the executor so it can run other
// tasks.
//
// # Placing yield points
//
// Voluntary yield points should be placed _after_ at least some work has been
// done. If they are not, a future sufficiently deep in the task hierarchy may
// end up _never_ getting to run because of the number of yield points that
// inevitably appear before it is reached. In general, you will want yield
// points to only appear in "leaf" futures -- those that do not themselves poll
// other futures. By doing this, you avoid double-counting each iteration of
// the outer future against the cooperating budget.

use std::cell::Cell;

thread_local! {
    static CURRENT: Cell<Budget> = const { Cell::new(Budget::unconstrained()) };
}

/// Opaque type tracking the amount of "work" a task may still do before
/// yielding back to the scheduler.
#[derive(Debug, Copy, Clone)]
pub(crate) struct Budget(Option<u8>);

impl Budget {
    /// Budget assigned to a task on each poll.
    ///
    /// The value itself is chosen somewhat arbitrarily. It needs to be high
    /// enough to amortize wakeup and scheduling costs, but low enough that we
    /// do not starve other tasks for too long. The value also needs to be high
    /// enough that particularly deep tasks are able to do at least some useful
    /// work at all.
    ///
    /// Note that as more yield points are added in the ecosystem, this value
    /// will probably also have to be raised.
    const fn initial() -> Budget {
        Budget(Some(128))
    }

    /// Returns an unconstrained budget. Operations will not be limited.
    const fn unconstrained() -> Budget {
        Budget(None)
    }

    fn has_remaining(self) -> bool {
        self.0.map(|budget| budget > 0).unwrap_or(true)
    }
}

/// Runs the given closure with a cooperative task budget. When the function
/// returns, the budget is reset to the value prior to calling the function.
#[inline(always)]
pub(crate) fn budget<R>(f: impl FnOnce() -> R) -> R {
    with_budget(Budget::initial(), f)
}

/// Runs the given closure with an unconstrained task budget. When the function returns, the budget
/// is reset to the value prior to calling the function.
#[inline(always)]
pub(crate) fn with_unconstrained<R>(f: impl FnOnce() -> R) -> R {
    with_budget(Budget::unconstrained(), f)
}

#[inline(always)]
fn with_budget<R>(budget: Budget, f: impl FnOnce() -> R) -> R {
    struct ResetGuard<'a> {
        cell: &'a Cell<Budget>,
        prev: Budget,
    }

    impl<'a> Drop for ResetGuard<'a> {
        fn drop(&mut self) {
            self.cell.set(self.prev);
        }
    }

    CURRENT.with(move |cell| {
        let prev = cell.get();

        cell.set(budget);

        let _guard = ResetGuard { cell, prev };

        f()
    })
}

#[inline(always)]
pub(crate) fn has_budget_remaining() -> bool {
    CURRENT.with(|cell| cell.get().has_remaining())
}

cfg_rt_multi_thread! {
    /// Sets the current task's budget.
    pub(crate) fn set(budget: Budget) {
        CURRENT.with(|cell| cell.set(budget))
    }
}

cfg_rt! {
    /// Forcibly removes the budgeting constraints early.
    ///
    /// Returns the remaining budget
    pub(crate) fn stop() -> Budget {
        CURRENT.with(|cell| {
            let prev = cell.get();
            cell.set(Budget::unconstrained());
            prev
        })
    }
}

cfg_coop! {
    use std::task::{Context, Poll};

    #[must_use]
    pub(crate) struct RestoreOnPending(Cell<Budget>);

    impl RestoreOnPending {
        pub(crate) fn made_progress(&self) {
            self.0.set(Budget::unconstrained());
        }
    }

    impl Drop for RestoreOnPending {
        fn drop(&mut self) {
            // Don't reset if budget was unconstrained or if we made progress.
            // They are both represented as the remembered budget being unconstrained.
            let budget = self.0.get();
            if !budget.is_unconstrained() {
                CURRENT.with(|cell| {
                    cell.set(budget);
                });
            }
        }
    }

    /// Returns `Poll::Pending` if the current task has exceeded its budget and should yield.
    ///
    /// When you call this method, the current budget is decremented. However, to ensure that
    /// progress is made every time a task is polled, the budget is automatically restored to its
    /// former value if the returned `RestoreOnPending` is dropped. It is the caller's
    /// responsibility to call `RestoreOnPending::made_progress` if it made progress, to ensure
    /// that the budget empties appropriately.
    ///
    /// Note that `RestoreOnPending` restores the budget **as it was before `poll_proceed`**.
    /// Therefore, if the budget is _further_ adjusted between when `poll_proceed` returns and
    /// `RestRestoreOnPending` is dropped, those adjustments are erased unless the caller indicates
    /// that progress was made.
    #[inline]
    pub(crate) fn poll_proceed(cx: &mut Context<'_>) -> Poll<RestoreOnPending> {
        CURRENT.with(|cell| {
            let mut budget = cell.get();

            if budget.decrement() {
                let restore = RestoreOnPending(Cell::new(cell.get()));
                cell.set(budget);
                Poll::Ready(restore)
            } else {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        })
    }

    impl Budget {
        /// Decrements the budget. Returns `true` if successful. Decrementing fails
        /// when there is not enough remaining budget.
        fn decrement(&mut self) -> bool {
            if let Some(num) = &mut self.0 {
                if *num > 0 {
                    *num -= 1;
                    true
                } else {
                    false
                }
            } else {
                true
            }
        }

        fn is_unconstrained(self) -> bool {
            self.0.is_none()
        }
    }
}

#[cfg(all(test, not(loom)))]
mod test {
    use super::*;

    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::wasm_bindgen_test as test;

    fn get() -> Budget {
        CURRENT.with(|cell| cell.get())
    }

    #[test]
    fn bugeting() {
        use futures::future::poll_fn;
        use tokio_test::*;

        assert!(get().0.is_none());

        let coop = assert_ready!(task::spawn(()).enter(|cx, _| poll_proceed(cx)));

        assert!(get().0.is_none());
        drop(coop);
        assert!(get().0.is_none());

        budget(|| {
            assert_eq!(get().0, Budget::initial().0);

            let coop = assert_ready!(task::spawn(()).enter(|cx, _| poll_proceed(cx)));
            assert_eq!(get().0.unwrap(), Budget::initial().0.unwrap() - 1);
            drop(coop);
            // we didn't make progress
            assert_eq!(get().0, Budget::initial().0);

            let coop = assert_ready!(task::spawn(()).enter(|cx, _| poll_proceed(cx)));
            assert_eq!(get().0.unwrap(), Budget::initial().0.unwrap() - 1);
            coop.made_progress();
            drop(coop);
            // we _did_ make progress
            assert_eq!(get().0.unwrap(), Budget::initial().0.unwrap() - 1);

            let coop = assert_ready!(task::spawn(()).enter(|cx, _| poll_proceed(cx)));
            assert_eq!(get().0.unwrap(), Budget::initial().0.unwrap() - 2);
            coop.made_progress();
            drop(coop);
            assert_eq!(get().0.unwrap(), Budget::initial().0.unwrap() - 2);

            budget(|| {
                assert_eq!(get().0, Budget::initial().0);

                let coop = assert_ready!(task::spawn(()).enter(|cx, _| poll_proceed(cx)));
                assert_eq!(get().0.unwrap(), Budget::initial().0.unwrap() - 1);
                coop.made_progress();
                drop(coop);
                assert_eq!(get().0.unwrap(), Budget::initial().0.unwrap() - 1);
            });

            assert_eq!(get().0.unwrap(), Budget::initial().0.unwrap() - 2);
        });

        assert!(get().0.is_none());

        budget(|| {
            let n = get().0.unwrap();

            for _ in 0..n {
                let coop = assert_ready!(task::spawn(()).enter(|cx, _| poll_proceed(cx)));
                coop.made_progress();
            }

            let mut task = task::spawn(poll_fn(|cx| {
                let coop = ready!(poll_proceed(cx));
                coop.made_progress();
                Poll::Ready(())
            }));

            assert_pending!(task.poll());
        });
    }
}
