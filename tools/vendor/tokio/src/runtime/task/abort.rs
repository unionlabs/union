use crate::runtime::task::{Id, RawTask};
use std::fmt;
use std::panic::{RefUnwindSafe, UnwindSafe};

/// An owned permission to abort a spawned task, without awaiting its completion.
///
/// Unlike a [`JoinHandle`], an `AbortHandle` does *not* represent the
/// permission to await the task's completion, only to terminate it.
///
/// The task may be aborted by calling the [`AbortHandle::abort`] method.
/// Dropping an `AbortHandle` releases the permission to terminate the task
/// --- it does *not* abort the task.
///
/// **Note**: This is an [unstable API][unstable]. The public API of this type
/// may break in 1.x releases. See [the documentation on unstable
/// features][unstable] for details.
///
/// [unstable]: crate#unstable-features
/// [`JoinHandle`]: crate::task::JoinHandle
#[cfg_attr(docsrs, doc(cfg(all(feature = "rt", tokio_unstable))))]
#[cfg_attr(not(tokio_unstable), allow(unreachable_pub))]
pub struct AbortHandle {
    raw: Option<RawTask>,
    id: Id,
}

impl AbortHandle {
    pub(super) fn new(raw: Option<RawTask>, id: Id) -> Self {
        Self { raw, id }
    }

    /// Abort the task associated with the handle.
    ///
    /// Awaiting a cancelled task might complete as usual if the task was
    /// already completed at the time it was cancelled, but most likely it
    /// will fail with a [cancelled] `JoinError`.
    ///
    /// If the task was already cancelled, such as by [`JoinHandle::abort`],
    /// this method will do nothing.
    ///
    /// [cancelled]: method@super::error::JoinError::is_cancelled
    /// [`JoinHandle::abort`]: method@super::JoinHandle::abort
    // the `AbortHandle` type is only publicly exposed when `tokio_unstable` is
    // enabled, but it is still defined for testing purposes.
    #[cfg_attr(not(tokio_unstable), allow(unreachable_pub))]
    pub fn abort(&self) {
        if let Some(ref raw) = self.raw {
            raw.remote_abort();
        }
    }

    /// Checks if the task associated with this `AbortHandle` has finished.
    ///
    /// Please note that this method can return `false` even if `abort` has been
    /// called on the task. This is because the cancellation process may take
    /// some time, and this method does not return `true` until it has
    /// completed.
    #[cfg_attr(not(tokio_unstable), allow(unreachable_pub))]
    pub fn is_finished(&self) -> bool {
        if let Some(raw) = self.raw {
            let state = raw.header().state.load();
            state.is_complete()
        } else {
            true
        }
    }

    /// Returns a [task ID] that uniquely identifies this task relative to other
    /// currently spawned tasks.
    ///
    /// **Note**: This is an [unstable API][unstable]. The public API of this type
    /// may break in 1.x releases. See [the documentation on unstable
    /// features][unstable] for details.
    ///
    /// [task ID]: crate::task::Id
    /// [unstable]: crate#unstable-features
    #[cfg(tokio_unstable)]
    #[cfg_attr(docsrs, doc(cfg(tokio_unstable)))]
    pub fn id(&self) -> super::Id {
        self.id.clone()
    }
}

unsafe impl Send for AbortHandle {}
unsafe impl Sync for AbortHandle {}

impl UnwindSafe for AbortHandle {}
impl RefUnwindSafe for AbortHandle {}

impl fmt::Debug for AbortHandle {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("AbortHandle")
            .field("id", &self.id)
            .finish()
    }
}

impl Drop for AbortHandle {
    fn drop(&mut self) {
        if let Some(raw) = self.raw.take() {
            raw.drop_abort_handle();
        }
    }
}
