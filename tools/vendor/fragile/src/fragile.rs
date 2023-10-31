use std::cmp;
use std::fmt;
use std::mem;
use std::num::NonZeroUsize;

use crate::errors::InvalidThreadAccess;
use crate::thread_id;
use std::mem::ManuallyDrop;

/// A [`Fragile<T>`] wraps a non sendable `T` to be safely send to other threads.
///
/// Once the value has been wrapped it can be sent to other threads but access
/// to the value on those threads will fail.
///
/// If the value needs destruction and the fragile wrapper is on another thread
/// the destructor will panic.  Alternatively you can use
/// [`Sticky`](crate::Sticky) which is not going to panic but might temporarily
/// leak the value.
pub struct Fragile<T> {
    // ManuallyDrop is necessary because we need to move out of here without running the
    // Drop code in functions like `into_inner`.
    value: ManuallyDrop<T>,
    thread_id: NonZeroUsize,
}

impl<T> Fragile<T> {
    /// Creates a new [`Fragile`] wrapping a `value`.
    ///
    /// The value that is moved into the [`Fragile`] can be non `Send` and
    /// will be anchored to the thread that created the object.  If the
    /// fragile wrapper type ends up being send from thread to thread
    /// only the original thread can interact with the value.
    pub fn new(value: T) -> Self {
        Fragile {
            value: ManuallyDrop::new(value),
            thread_id: thread_id::get(),
        }
    }

    /// Returns `true` if the access is valid.
    ///
    /// This will be `false` if the value was sent to another thread.
    pub fn is_valid(&self) -> bool {
        thread_id::get() == self.thread_id
    }

    #[inline(always)]
    fn assert_thread(&self) {
        if !self.is_valid() {
            panic!("trying to access wrapped value in fragile container from incorrect thread.");
        }
    }

    /// Consumes the `Fragile`, returning the wrapped value.
    ///
    /// # Panics
    ///
    /// Panics if called from a different thread than the one where the
    /// original value was created.
    pub fn into_inner(self) -> T {
        self.assert_thread();

        let mut this = ManuallyDrop::new(self);

        // SAFETY: `this` is not accessed beyond this point, and because it's in a ManuallyDrop its
        // destructor is not run.
        unsafe { ManuallyDrop::take(&mut this.value) }
    }

    /// Consumes the `Fragile`, returning the wrapped value if successful.
    ///
    /// The wrapped value is returned if this is called from the same thread
    /// as the one where the original value was created, otherwise the
    /// [`Fragile`] is returned as `Err(self)`.
    pub fn try_into_inner(self) -> Result<T, Self> {
        if thread_id::get() == self.thread_id {
            Ok(self.into_inner())
        } else {
            Err(self)
        }
    }

    /// Immutably borrows the wrapped value.
    ///
    /// # Panics
    ///
    /// Panics if the calling thread is not the one that wrapped the value.
    /// For a non-panicking variant, use [`try_get`](Self::try_get).
    pub fn get(&self) -> &T {
        self.assert_thread();
        &*self.value
    }

    /// Mutably borrows the wrapped value.
    ///
    /// # Panics
    ///
    /// Panics if the calling thread is not the one that wrapped the value.
    /// For a non-panicking variant, use [`try_get_mut`](Self::try_get_mut).
    pub fn get_mut(&mut self) -> &mut T {
        self.assert_thread();
        &mut *self.value
    }

    /// Tries to immutably borrow the wrapped value.
    ///
    /// Returns `None` if the calling thread is not the one that wrapped the value.
    pub fn try_get(&self) -> Result<&T, InvalidThreadAccess> {
        if thread_id::get() == self.thread_id {
            Ok(&*self.value)
        } else {
            Err(InvalidThreadAccess)
        }
    }

    /// Tries to mutably borrow the wrapped value.
    ///
    /// Returns `None` if the calling thread is not the one that wrapped the value.
    pub fn try_get_mut(&mut self) -> Result<&mut T, InvalidThreadAccess> {
        if thread_id::get() == self.thread_id {
            Ok(&mut *self.value)
        } else {
            Err(InvalidThreadAccess)
        }
    }
}

impl<T> Drop for Fragile<T> {
    fn drop(&mut self) {
        if mem::needs_drop::<T>() {
            if thread_id::get() == self.thread_id {
                // SAFETY: `ManuallyDrop::drop` cannot be called after this point.
                unsafe { ManuallyDrop::drop(&mut self.value) };
            } else {
                panic!("destructor of fragile object ran on wrong thread");
            }
        }
    }
}

impl<T> From<T> for Fragile<T> {
    #[inline]
    fn from(t: T) -> Fragile<T> {
        Fragile::new(t)
    }
}

impl<T: Clone> Clone for Fragile<T> {
    #[inline]
    fn clone(&self) -> Fragile<T> {
        Fragile::new(self.get().clone())
    }
}

impl<T: Default> Default for Fragile<T> {
    #[inline]
    fn default() -> Fragile<T> {
        Fragile::new(T::default())
    }
}

impl<T: PartialEq> PartialEq for Fragile<T> {
    #[inline]
    fn eq(&self, other: &Fragile<T>) -> bool {
        *self.get() == *other.get()
    }
}

impl<T: Eq> Eq for Fragile<T> {}

impl<T: PartialOrd> PartialOrd for Fragile<T> {
    #[inline]
    fn partial_cmp(&self, other: &Fragile<T>) -> Option<cmp::Ordering> {
        self.get().partial_cmp(other.get())
    }

    #[inline]
    fn lt(&self, other: &Fragile<T>) -> bool {
        *self.get() < *other.get()
    }

    #[inline]
    fn le(&self, other: &Fragile<T>) -> bool {
        *self.get() <= *other.get()
    }

    #[inline]
    fn gt(&self, other: &Fragile<T>) -> bool {
        *self.get() > *other.get()
    }

    #[inline]
    fn ge(&self, other: &Fragile<T>) -> bool {
        *self.get() >= *other.get()
    }
}

impl<T: Ord> Ord for Fragile<T> {
    #[inline]
    fn cmp(&self, other: &Fragile<T>) -> cmp::Ordering {
        self.get().cmp(other.get())
    }
}

impl<T: fmt::Display> fmt::Display for Fragile<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt::Display::fmt(self.get(), f)
    }
}

impl<T: fmt::Debug> fmt::Debug for Fragile<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self.try_get() {
            Ok(value) => f.debug_struct("Fragile").field("value", value).finish(),
            Err(..) => {
                struct InvalidPlaceholder;
                impl fmt::Debug for InvalidPlaceholder {
                    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                        f.write_str("<invalid thread>")
                    }
                }

                f.debug_struct("Fragile")
                    .field("value", &InvalidPlaceholder)
                    .finish()
            }
        }
    }
}

// this type is sync because access can only ever happy from the same thread
// that created it originally.  All other threads will be able to safely
// call some basic operations on the reference and they will fail.
unsafe impl<T> Sync for Fragile<T> {}

// The entire point of this type is to be Send
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl<T> Send for Fragile<T> {}

#[test]
fn test_basic() {
    use std::thread;
    let val = Fragile::new(true);
    assert_eq!(val.to_string(), "true");
    assert_eq!(val.get(), &true);
    assert!(val.try_get().is_ok());
    thread::spawn(move || {
        assert!(val.try_get().is_err());
    })
    .join()
    .unwrap();
}

#[test]
fn test_mut() {
    let mut val = Fragile::new(true);
    *val.get_mut() = false;
    assert_eq!(val.to_string(), "false");
    assert_eq!(val.get(), &false);
}

#[test]
#[should_panic]
fn test_access_other_thread() {
    use std::thread;
    let val = Fragile::new(true);
    thread::spawn(move || {
        val.get();
    })
    .join()
    .unwrap();
}

#[test]
fn test_noop_drop_elsewhere() {
    use std::thread;
    let val = Fragile::new(true);
    thread::spawn(move || {
        // force the move
        val.try_get().ok();
    })
    .join()
    .unwrap();
}

#[test]
fn test_panic_on_drop_elsewhere() {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::thread;
    let was_called = Arc::new(AtomicBool::new(false));
    struct X(Arc<AtomicBool>);
    impl Drop for X {
        fn drop(&mut self) {
            self.0.store(true, Ordering::SeqCst);
        }
    }
    let val = Fragile::new(X(was_called.clone()));
    assert!(thread::spawn(move || {
        val.try_get().ok();
    })
    .join()
    .is_err());
    assert!(!was_called.load(Ordering::SeqCst));
}

#[test]
fn test_rc_sending() {
    use std::rc::Rc;
    use std::sync::mpsc::channel;
    use std::thread;

    let val = Fragile::new(Rc::new(true));
    let (tx, rx) = channel();

    let thread = thread::spawn(move || {
        assert!(val.try_get().is_err());
        let here = val;
        tx.send(here).unwrap();
    });

    let rv = rx.recv().unwrap();
    assert!(**rv.get());

    thread.join().unwrap();
}
