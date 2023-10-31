use event_listener::Event;

use crate::Mutex;

/// A counter to synchronize multiple tasks at the same time.
#[derive(Debug)]
pub struct Barrier {
    n: usize,
    state: Mutex<State>,
    event: Event,
}

#[derive(Debug)]
struct State {
    count: usize,
    generation_id: u64,
}

impl Barrier {
    /// Creates a barrier that can block the given number of tasks.
    ///
    /// A barrier will block `n`-1 tasks which call [`wait()`] and then wake up all tasks
    /// at once when the `n`th task calls [`wait()`].
    ///
    /// [`wait()`]: `Barrier::wait()`
    ///
    /// # Examples
    ///
    /// ```
    /// use async_lock::Barrier;
    ///
    /// let barrier = Barrier::new(5);
    /// ```
    pub const fn new(n: usize) -> Barrier {
        Barrier {
            n,
            state: Mutex::new(State {
                count: 0,
                generation_id: 0,
            }),
            event: Event::new(),
        }
    }

    /// Blocks the current task until all tasks reach this point.
    ///
    /// Barriers are reusable after all tasks have synchronized, and can be used continuously.
    ///
    /// Returns a [`BarrierWaitResult`] indicating whether this task is the "leader", meaning the
    /// last task to call this method.
    ///
    /// # Examples
    ///
    /// ```
    /// use async_lock::Barrier;
    /// use futures_lite::future;
    /// use std::sync::Arc;
    /// use std::thread;
    ///
    /// let barrier = Arc::new(Barrier::new(5));
    ///
    /// for _ in 0..5 {
    ///     let b = barrier.clone();
    ///     thread::spawn(move || {
    ///         future::block_on(async {
    ///             // The same messages will be printed together.
    ///             // There will NOT be interleaving of "before" and "after".
    ///             println!("before wait");
    ///             b.wait().await;
    ///             println!("after wait");
    ///         });
    ///     });
    /// }
    /// ```
    pub async fn wait(&self) -> BarrierWaitResult {
        let mut state = self.state.lock().await;
        let local_gen = state.generation_id;
        state.count += 1;

        if state.count < self.n {
            while local_gen == state.generation_id && state.count < self.n {
                let listener = self.event.listen();
                drop(state);
                listener.await;
                state = self.state.lock().await;
            }
            BarrierWaitResult { is_leader: false }
        } else {
            state.count = 0;
            state.generation_id = state.generation_id.wrapping_add(1);
            self.event.notify(std::usize::MAX);
            BarrierWaitResult { is_leader: true }
        }
    }
}

/// Returned by [`Barrier::wait()`] when all tasks have called it.
///
/// # Examples
///
/// ```
/// # futures_lite::future::block_on(async {
/// use async_lock::Barrier;
///
/// let barrier = Barrier::new(1);
/// let barrier_wait_result = barrier.wait().await;
/// # });
/// ```
#[derive(Debug, Clone)]
pub struct BarrierWaitResult {
    is_leader: bool,
}

impl BarrierWaitResult {
    /// Returns `true` if this task was the last to call to [`Barrier::wait()`].
    ///
    /// # Examples
    ///
    /// ```
    /// # futures_lite::future::block_on(async {
    /// use async_lock::Barrier;
    /// use futures_lite::future;
    ///
    /// let barrier = Barrier::new(2);
    /// let (a, b) = future::zip(barrier.wait(), barrier.wait()).await;
    /// assert_eq!(a.is_leader(), false);
    /// assert_eq!(b.is_leader(), true);
    /// # });
    /// ```
    pub fn is_leader(&self) -> bool {
        self.is_leader
    }
}
