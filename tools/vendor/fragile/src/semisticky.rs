use std::cmp;
use std::fmt;
use std::mem;

use crate::errors::InvalidThreadAccess;
use crate::fragile::Fragile;
use crate::sticky::Sticky;
use crate::StackToken;

enum SemiStickyImpl<T: 'static> {
    Fragile(Box<Fragile<T>>),
    Sticky(Sticky<T>),
}

/// A [`SemiSticky<T>`] keeps a value T stored in a thread if it has a drop.
///
/// This is a combined version of [`Fragile`] and [`Sticky`].  If the type
/// does not have a drop it will effectively be a [`Fragile`], otherwise it
/// will be internally behave like a [`Sticky`].
///
/// This type requires `T: 'static` for the same reasons as [`Sticky`] and
/// also uses [`StackToken`]s.
pub struct SemiSticky<T: 'static> {
    inner: SemiStickyImpl<T>,
}

impl<T> SemiSticky<T> {
    /// Creates a new [`SemiSticky`] wrapping a `value`.
    ///
    /// The value that is moved into the `SemiSticky` can be non `Send` and
    /// will be anchored to the thread that created the object.  If the
    /// sticky wrapper type ends up being send from thread to thread
    /// only the original thread can interact with the value.  In case the
    /// value does not have `Drop` it will be stored in the [`SemiSticky`]
    /// instead.
    pub fn new(value: T) -> Self {
        SemiSticky {
            inner: if mem::needs_drop::<T>() {
                SemiStickyImpl::Sticky(Sticky::new(value))
            } else {
                SemiStickyImpl::Fragile(Box::new(Fragile::new(value)))
            },
        }
    }

    /// Returns `true` if the access is valid.
    ///
    /// This will be `false` if the value was sent to another thread.
    pub fn is_valid(&self) -> bool {
        match self.inner {
            SemiStickyImpl::Fragile(ref inner) => inner.is_valid(),
            SemiStickyImpl::Sticky(ref inner) => inner.is_valid(),
        }
    }

    /// Consumes the [`SemiSticky`], returning the wrapped value.
    ///
    /// # Panics
    ///
    /// Panics if called from a different thread than the one where the
    /// original value was created.
    pub fn into_inner(self) -> T {
        match self.inner {
            SemiStickyImpl::Fragile(inner) => inner.into_inner(),
            SemiStickyImpl::Sticky(inner) => inner.into_inner(),
        }
    }

    /// Consumes the [`SemiSticky`], returning the wrapped value if successful.
    ///
    /// The wrapped value is returned if this is called from the same thread
    /// as the one where the original value was created, otherwise the
    /// [`SemiSticky`] is returned as `Err(self)`.
    pub fn try_into_inner(self) -> Result<T, Self> {
        match self.inner {
            SemiStickyImpl::Fragile(inner) => inner.try_into_inner().map_err(|inner| SemiSticky {
                inner: SemiStickyImpl::Fragile(Box::new(inner)),
            }),
            SemiStickyImpl::Sticky(inner) => inner.try_into_inner().map_err(|inner| SemiSticky {
                inner: SemiStickyImpl::Sticky(inner),
            }),
        }
    }

    /// Immutably borrows the wrapped value.
    ///
    /// # Panics
    ///
    /// Panics if the calling thread is not the one that wrapped the value.
    /// For a non-panicking variant, use [`try_get`](Self::try_get).
    pub fn get<'stack>(&'stack self, _proof: &'stack StackToken) -> &'stack T {
        match self.inner {
            SemiStickyImpl::Fragile(ref inner) => inner.get(),
            SemiStickyImpl::Sticky(ref inner) => inner.get(_proof),
        }
    }

    /// Mutably borrows the wrapped value.
    ///
    /// # Panics
    ///
    /// Panics if the calling thread is not the one that wrapped the value.
    /// For a non-panicking variant, use [`try_get_mut`](Self::try_get_mut).
    pub fn get_mut<'stack>(&'stack mut self, _proof: &'stack StackToken) -> &'stack mut T {
        match self.inner {
            SemiStickyImpl::Fragile(ref mut inner) => inner.get_mut(),
            SemiStickyImpl::Sticky(ref mut inner) => inner.get_mut(_proof),
        }
    }

    /// Tries to immutably borrow the wrapped value.
    ///
    /// Returns `None` if the calling thread is not the one that wrapped the value.
    pub fn try_get<'stack>(
        &'stack self,
        _proof: &'stack StackToken,
    ) -> Result<&'stack T, InvalidThreadAccess> {
        match self.inner {
            SemiStickyImpl::Fragile(ref inner) => inner.try_get(),
            SemiStickyImpl::Sticky(ref inner) => inner.try_get(_proof),
        }
    }

    /// Tries to mutably borrow the wrapped value.
    ///
    /// Returns `None` if the calling thread is not the one that wrapped the value.
    pub fn try_get_mut<'stack>(
        &'stack mut self,
        _proof: &'stack StackToken,
    ) -> Result<&'stack mut T, InvalidThreadAccess> {
        match self.inner {
            SemiStickyImpl::Fragile(ref mut inner) => inner.try_get_mut(),
            SemiStickyImpl::Sticky(ref mut inner) => inner.try_get_mut(_proof),
        }
    }
}

impl<T> From<T> for SemiSticky<T> {
    #[inline]
    fn from(t: T) -> SemiSticky<T> {
        SemiSticky::new(t)
    }
}

impl<T: Clone> Clone for SemiSticky<T> {
    #[inline]
    fn clone(&self) -> SemiSticky<T> {
        crate::stack_token!(tok);
        SemiSticky::new(self.get(tok).clone())
    }
}

impl<T: Default> Default for SemiSticky<T> {
    #[inline]
    fn default() -> SemiSticky<T> {
        SemiSticky::new(T::default())
    }
}

impl<T: PartialEq> PartialEq for SemiSticky<T> {
    #[inline]
    fn eq(&self, other: &SemiSticky<T>) -> bool {
        crate::stack_token!(tok);
        *self.get(tok) == *other.get(tok)
    }
}

impl<T: Eq> Eq for SemiSticky<T> {}

impl<T: PartialOrd> PartialOrd for SemiSticky<T> {
    #[inline]
    fn partial_cmp(&self, other: &SemiSticky<T>) -> Option<cmp::Ordering> {
        crate::stack_token!(tok);
        self.get(tok).partial_cmp(other.get(tok))
    }

    #[inline]
    fn lt(&self, other: &SemiSticky<T>) -> bool {
        crate::stack_token!(tok);
        *self.get(tok) < *other.get(tok)
    }

    #[inline]
    fn le(&self, other: &SemiSticky<T>) -> bool {
        crate::stack_token!(tok);
        *self.get(tok) <= *other.get(tok)
    }

    #[inline]
    fn gt(&self, other: &SemiSticky<T>) -> bool {
        crate::stack_token!(tok);
        *self.get(tok) > *other.get(tok)
    }

    #[inline]
    fn ge(&self, other: &SemiSticky<T>) -> bool {
        crate::stack_token!(tok);
        *self.get(tok) >= *other.get(tok)
    }
}

impl<T: Ord> Ord for SemiSticky<T> {
    #[inline]
    fn cmp(&self, other: &SemiSticky<T>) -> cmp::Ordering {
        crate::stack_token!(tok);
        self.get(tok).cmp(other.get(tok))
    }
}

impl<T: fmt::Display> fmt::Display for SemiSticky<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        crate::stack_token!(tok);
        fmt::Display::fmt(self.get(tok), f)
    }
}

impl<T: fmt::Debug> fmt::Debug for SemiSticky<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        crate::stack_token!(tok);
        match self.try_get(tok) {
            Ok(value) => f.debug_struct("SemiSticky").field("value", value).finish(),
            Err(..) => {
                struct InvalidPlaceholder;
                impl fmt::Debug for InvalidPlaceholder {
                    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                        f.write_str("<invalid thread>")
                    }
                }

                f.debug_struct("SemiSticky")
                    .field("value", &InvalidPlaceholder)
                    .finish()
            }
        }
    }
}

#[test]
fn test_basic() {
    use std::thread;
    let val = SemiSticky::new(true);
    crate::stack_token!(tok);
    assert_eq!(val.to_string(), "true");
    assert_eq!(val.get(tok), &true);
    assert!(val.try_get(tok).is_ok());
    thread::spawn(move || {
        crate::stack_token!(tok);
        assert!(val.try_get(tok).is_err());
    })
    .join()
    .unwrap();
}

#[test]
fn test_mut() {
    let mut val = SemiSticky::new(true);
    crate::stack_token!(tok);
    *val.get_mut(tok) = false;
    assert_eq!(val.to_string(), "false");
    assert_eq!(val.get(tok), &false);
}

#[test]
#[should_panic]
fn test_access_other_thread() {
    use std::thread;
    let val = SemiSticky::new(true);
    thread::spawn(move || {
        crate::stack_token!(tok);
        val.get(tok);
    })
    .join()
    .unwrap();
}

#[test]
fn test_drop_same_thread() {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    let was_called = Arc::new(AtomicBool::new(false));
    struct X(Arc<AtomicBool>);
    impl Drop for X {
        fn drop(&mut self) {
            self.0.store(true, Ordering::SeqCst);
        }
    }
    let val = SemiSticky::new(X(was_called.clone()));
    mem::drop(val);
    assert!(was_called.load(Ordering::SeqCst));
}

#[test]
fn test_noop_drop_elsewhere() {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::thread;

    let was_called = Arc::new(AtomicBool::new(false));

    {
        let was_called = was_called.clone();
        thread::spawn(move || {
            struct X(Arc<AtomicBool>);
            impl Drop for X {
                fn drop(&mut self) {
                    self.0.store(true, Ordering::SeqCst);
                }
            }

            let val = SemiSticky::new(X(was_called.clone()));
            assert!(thread::spawn(move || {
                // moves it here but do not deallocate
                crate::stack_token!(tok);
                val.try_get(tok).ok();
            })
            .join()
            .is_ok());

            assert!(!was_called.load(Ordering::SeqCst));
        })
        .join()
        .unwrap();
    }

    assert!(was_called.load(Ordering::SeqCst));
}

#[test]
fn test_rc_sending() {
    use std::rc::Rc;
    use std::thread;
    let val = SemiSticky::new(Rc::new(true));
    thread::spawn(move || {
        crate::stack_token!(tok);
        assert!(val.try_get(tok).is_err());
    })
    .join()
    .unwrap();
}
