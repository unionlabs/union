use crate::{task::JoinHandle, util::error::CONTEXT_MISSING_ERROR};

use std::future::Future;

cfg_rt! {
    /// Spawns a new asynchronous task, returning a
    /// [`JoinHandle`](super::JoinHandle) for it.
    ///
    /// Spawning a task enables the task to execute concurrently to other tasks. The
    /// spawned task may execute on the current thread, or it may be sent to a
    /// different thread to be executed. The specifics depend on the current
    /// [`Runtime`](crate::runtime::Runtime) configuration.
    ///
    /// There is no guarantee that a spawned task will execute to completion.
    /// When a runtime is shutdown, all outstanding tasks are dropped,
    /// regardless of the lifecycle of that task.
    ///
    /// This function must be called from the context of a Tokio runtime. Tasks running on
    /// the Tokio runtime are always inside its context, but you can also enter the context
    /// using the [`Runtime::enter`](crate::runtime::Runtime::enter()) method.
    ///
    /// # Examples
    ///
    /// In this example, a server is started and `spawn` is used to start a new task
    /// that processes each received connection.
    ///
    /// ```no_run
    /// use tokio::net::{TcpListener, TcpStream};
    ///
    /// use std::io;
    ///
    /// async fn process(socket: TcpStream) {
    ///     // ...
    /// # drop(socket);
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() -> io::Result<()> {
    ///     let listener = TcpListener::bind("127.0.0.1:8080").await?;
    ///
    ///     loop {
    ///         let (socket, _) = listener.accept().await?;
    ///
    ///         tokio::spawn(async move {
    ///             // Process each socket concurrently.
    ///             process(socket).await
    ///         });
    ///     }
    /// }
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if called from **outside** of the Tokio runtime.
    ///
    /// # Using `!Send` values from a task
    ///
    /// The task supplied to `spawn` must implement `Send`. However, it is
    /// possible to **use** `!Send` values from the task as long as they only
    /// exist between calls to `.await`.
    ///
    /// For example, this will work:
    ///
    /// ```
    /// use tokio::task;
    ///
    /// use std::rc::Rc;
    ///
    /// fn use_rc(rc: Rc<()>) {
    ///     // Do stuff w/ rc
    /// # drop(rc);
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     tokio::spawn(async {
    ///         // Force the `Rc` to stay in a scope with no `.await`
    ///         {
    ///             let rc = Rc::new(());
    ///             use_rc(rc.clone());
    ///         }
    ///
    ///         task::yield_now().await;
    ///     }).await.unwrap();
    /// }
    /// ```
    ///
    /// This will **not** work:
    ///
    /// ```compile_fail
    /// use tokio::task;
    ///
    /// use std::rc::Rc;
    ///
    /// fn use_rc(rc: Rc<()>) {
    ///     // Do stuff w/ rc
    /// # drop(rc);
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     tokio::spawn(async {
    ///         let rc = Rc::new(());
    ///
    ///         task::yield_now().await;
    ///
    ///         use_rc(rc.clone());
    ///     }).await.unwrap();
    /// }
    /// ```
    ///
    /// Holding on to a `!Send` value across calls to `.await` will result in
    /// an unfriendly compile error message similar to:
    ///
    /// ```text
    /// `[... some type ...]` cannot be sent between threads safely
    /// ```
    ///
    /// or:
    ///
    /// ```text
    /// error[E0391]: cycle detected when processing `main`
    /// ```
    #[track_caller]
    pub fn spawn<T>(future: T) -> JoinHandle<T::Output>
    where
        T: Future + Send + 'static,
        T::Output: Send + 'static,
    {
        // preventing stack overflows on debug mode, by quickly sending the
        // task to the heap.
        if cfg!(debug_assertions) && std::mem::size_of::<T>() > 2048 {
            spawn_inner(Box::pin(future), None)
        } else {
            spawn_inner(future, None)
        }
    }

    #[track_caller]
    pub(super) fn spawn_inner<T>(future: T, name: Option<&str>) -> JoinHandle<T::Output>
    where
        T: Future + Send + 'static,
        T::Output: Send + 'static,
    {
        use crate::runtime::{task, context};
        let id = task::Id::next();
        let spawn_handle = context::spawn_handle().expect(CONTEXT_MISSING_ERROR);
        let task = crate::util::trace::task(future, "task", name, id.as_u64());
        spawn_handle.spawn(task, id)
    }
}
