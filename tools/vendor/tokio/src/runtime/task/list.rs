//! This module has containers for storing the tasks spawned on a scheduler. The
//! `OwnedTasks` container is thread-safe but can only store tasks that
//! implement Send. The `LocalOwnedTasks` container is not thread safe, but can
//! store non-Send tasks.
//!
//! The collections can be closed to prevent adding new tasks during shutdown of
//! the scheduler with the collection.

use crate::future::Future;
use crate::loom::cell::UnsafeCell;
use crate::loom::sync::Mutex;
use crate::runtime::task::{JoinHandle, LocalNotified, Notified, Schedule, Task};
use crate::util::linked_list::{Link, LinkedList};

use std::marker::PhantomData;

// The id from the module below is used to verify whether a given task is stored
// in this OwnedTasks, or some other task. The counter starts at one so we can
// use zero for tasks not owned by any list.
//
// The safety checks in this file can technically be violated if the counter is
// overflown, but the checks are not supposed to ever fail unless there is a
// bug in Tokio, so we accept that certain bugs would not be caught if the two
// mixed up runtimes happen to have the same id.

cfg_has_atomic_u64! {
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_OWNED_TASKS_ID: AtomicU64 = AtomicU64::new(1);

    fn get_next_id() -> u64 {
        loop {
            let id = NEXT_OWNED_TASKS_ID.fetch_add(1, Ordering::Relaxed);
            if id != 0 {
                return id;
            }
        }
    }
}

cfg_not_has_atomic_u64! {
    use std::sync::atomic::{AtomicU32, Ordering};

    static NEXT_OWNED_TASKS_ID: AtomicU32 = AtomicU32::new(1);

    fn get_next_id() -> u64 {
        loop {
            let id = NEXT_OWNED_TASKS_ID.fetch_add(1, Ordering::Relaxed);
            if id != 0 {
                return u64::from(id);
            }
        }
    }
}

pub(crate) struct OwnedTasks<S: 'static> {
    inner: Mutex<OwnedTasksInner<S>>,
    id: u64,
}
pub(crate) struct LocalOwnedTasks<S: 'static> {
    inner: UnsafeCell<OwnedTasksInner<S>>,
    id: u64,
    _not_send_or_sync: PhantomData<*const ()>,
}
struct OwnedTasksInner<S: 'static> {
    list: LinkedList<Task<S>, <Task<S> as Link>::Target>,
    closed: bool,
}

impl<S: 'static> OwnedTasks<S> {
    pub(crate) fn new() -> Self {
        Self {
            inner: Mutex::new(OwnedTasksInner {
                list: LinkedList::new(),
                closed: false,
            }),
            id: get_next_id(),
        }
    }

    /// Binds the provided task to this OwnedTasks instance. This fails if the
    /// OwnedTasks has been closed.
    pub(crate) fn bind<T>(
        &self,
        task: T,
        scheduler: S,
        id: super::Id,
    ) -> (JoinHandle<T::Output>, Option<Notified<S>>)
    where
        S: Schedule,
        T: Future + Send + 'static,
        T::Output: Send + 'static,
    {
        let (task, notified, join) = super::new_task(task, scheduler, id);

        unsafe {
            // safety: We just created the task, so we have exclusive access
            // to the field.
            task.header().set_owner_id(self.id);
        }

        let mut lock = self.inner.lock();
        if lock.closed {
            drop(lock);
            drop(notified);
            task.shutdown();
            (join, None)
        } else {
            lock.list.push_front(task);
            (join, Some(notified))
        }
    }

    /// Asserts that the given task is owned by this OwnedTasks and convert it to
    /// a LocalNotified, giving the thread permission to poll this task.
    #[inline]
    pub(crate) fn assert_owner(&self, task: Notified<S>) -> LocalNotified<S> {
        assert_eq!(task.header().get_owner_id(), self.id);

        // safety: All tasks bound to this OwnedTasks are Send, so it is safe
        // to poll it on this thread no matter what thread we are on.
        LocalNotified {
            task: task.0,
            _not_send: PhantomData,
        }
    }

    /// Shuts down all tasks in the collection. This call also closes the
    /// collection, preventing new items from being added.
    pub(crate) fn close_and_shutdown_all(&self)
    where
        S: Schedule,
    {
        // The first iteration of the loop was unrolled so it can set the
        // closed bool.
        let first_task = {
            let mut lock = self.inner.lock();
            lock.closed = true;
            lock.list.pop_back()
        };
        match first_task {
            Some(task) => task.shutdown(),
            None => return,
        }

        loop {
            let task = match self.inner.lock().list.pop_back() {
                Some(task) => task,
                None => return,
            };

            task.shutdown();
        }
    }

    pub(crate) fn remove(&self, task: &Task<S>) -> Option<Task<S>> {
        let task_id = task.header().get_owner_id();
        if task_id == 0 {
            // The task is unowned.
            return None;
        }

        assert_eq!(task_id, self.id);

        // safety: We just checked that the provided task is not in some other
        // linked list.
        unsafe { self.inner.lock().list.remove(task.header().into()) }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.inner.lock().list.is_empty()
    }
}

impl<S: 'static> LocalOwnedTasks<S> {
    pub(crate) fn new() -> Self {
        Self {
            inner: UnsafeCell::new(OwnedTasksInner {
                list: LinkedList::new(),
                closed: false,
            }),
            id: get_next_id(),
            _not_send_or_sync: PhantomData,
        }
    }

    pub(crate) fn bind<T>(
        &self,
        task: T,
        scheduler: S,
        id: super::Id,
    ) -> (JoinHandle<T::Output>, Option<Notified<S>>)
    where
        S: Schedule,
        T: Future + 'static,
        T::Output: 'static,
    {
        let (task, notified, join) = super::new_task(task, scheduler, id);

        unsafe {
            // safety: We just created the task, so we have exclusive access
            // to the field.
            task.header().set_owner_id(self.id);
        }

        if self.is_closed() {
            drop(notified);
            task.shutdown();
            (join, None)
        } else {
            self.with_inner(|inner| {
                inner.list.push_front(task);
            });
            (join, Some(notified))
        }
    }

    /// Shuts down all tasks in the collection. This call also closes the
    /// collection, preventing new items from being added.
    pub(crate) fn close_and_shutdown_all(&self)
    where
        S: Schedule,
    {
        self.with_inner(|inner| inner.closed = true);

        while let Some(task) = self.with_inner(|inner| inner.list.pop_back()) {
            task.shutdown();
        }
    }

    pub(crate) fn remove(&self, task: &Task<S>) -> Option<Task<S>> {
        let task_id = task.header().get_owner_id();
        if task_id == 0 {
            // The task is unowned.
            return None;
        }

        assert_eq!(task_id, self.id);

        self.with_inner(|inner|
            // safety: We just checked that the provided task is not in some
            // other linked list.
            unsafe { inner.list.remove(task.header().into()) })
    }

    /// Asserts that the given task is owned by this LocalOwnedTasks and convert
    /// it to a LocalNotified, giving the thread permission to poll this task.
    #[inline]
    pub(crate) fn assert_owner(&self, task: Notified<S>) -> LocalNotified<S> {
        assert_eq!(task.header().get_owner_id(), self.id);

        // safety: The task was bound to this LocalOwnedTasks, and the
        // LocalOwnedTasks is not Send or Sync, so we are on the right thread
        // for polling this task.
        LocalNotified {
            task: task.0,
            _not_send: PhantomData,
        }
    }

    #[inline]
    fn with_inner<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&mut OwnedTasksInner<S>) -> T,
    {
        // safety: This type is not Sync, so concurrent calls of this method
        // can't happen.  Furthermore, all uses of this method in this file make
        // sure that they don't call `with_inner` recursively.
        self.inner.with_mut(|ptr| unsafe { f(&mut *ptr) })
    }

    pub(crate) fn is_closed(&self) -> bool {
        self.with_inner(|inner| inner.closed)
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.with_inner(|inner| inner.list.is_empty())
    }
}

#[cfg(all(test))]
mod tests {
    use super::*;

    // This test may run in parallel with other tests, so we only test that ids
    // come in increasing order.
    #[test]
    fn test_id_not_broken() {
        let mut last_id = get_next_id();
        assert_ne!(last_id, 0);

        for _ in 0..1000 {
            let next_id = get_next_id();
            assert_ne!(next_id, 0);
            assert!(last_id < next_id);
            last_id = next_id;
        }
    }
}
