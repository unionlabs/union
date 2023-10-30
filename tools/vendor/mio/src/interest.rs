use std::num::NonZeroU8;
use std::{fmt, ops};

/// Interest used in registering.
///
/// Interest are used in [registering] [`event::Source`]s with [`Poll`], they
/// indicate what readiness should be monitored for. For example if a socket is
/// registered with [readable] interests and the socket becomes writable, no
/// event will be returned from a call to [`poll`].
///
/// [registering]: struct.Registry.html#method.register
/// [`event::Source`]: ./event/trait.Source.html
/// [`Poll`]: struct.Poll.html
/// [readable]: struct.Interest.html#associatedconstant.READABLE
/// [`poll`]: struct.Poll.html#method.poll
#[derive(Copy, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub struct Interest(NonZeroU8);

// These must be unique.
const READABLE: u8 = 0b0001;
const WRITABLE: u8 = 0b0010;
// The following are not available on all platforms.
#[cfg_attr(
    not(any(
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "ios",
        target_os = "macos"
    )),
    allow(dead_code)
)]
const AIO: u8 = 0b0100;
#[cfg_attr(not(target_os = "freebsd"), allow(dead_code))]
const LIO: u8 = 0b1000;

impl Interest {
    /// Returns a `Interest` set representing readable interests.
    pub const READABLE: Interest = Interest(unsafe { NonZeroU8::new_unchecked(READABLE) });

    /// Returns a `Interest` set representing writable interests.
    pub const WRITABLE: Interest = Interest(unsafe { NonZeroU8::new_unchecked(WRITABLE) });

    /// Returns a `Interest` set representing AIO completion interests.
    #[cfg(any(
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "ios",
        target_os = "macos"
    ))]
    pub const AIO: Interest = Interest(unsafe { NonZeroU8::new_unchecked(AIO) });

    /// Returns a `Interest` set representing LIO completion interests.
    #[cfg(target_os = "freebsd")]
    pub const LIO: Interest = Interest(unsafe { NonZeroU8::new_unchecked(LIO) });

    /// Add together two `Interest`.
    ///
    /// This does the same thing as the `BitOr` implementation, but is a
    /// constant function.
    ///
    /// ```
    /// use mio::Interest;
    ///
    /// const INTERESTS: Interest = Interest::READABLE.add(Interest::WRITABLE);
    /// # fn silent_dead_code_warning(_: Interest) { }
    /// # silent_dead_code_warning(INTERESTS)
    /// ```
    #[allow(clippy::should_implement_trait)]
    pub const fn add(self, other: Interest) -> Interest {
        Interest(unsafe { NonZeroU8::new_unchecked(self.0.get() | other.0.get()) })
    }

    /// Removes `other` `Interest` from `self`.
    ///
    /// Returns `None` if the set would be empty after removing `other`.
    ///
    /// ```
    /// use mio::Interest;
    ///
    /// const RW_INTERESTS: Interest = Interest::READABLE.add(Interest::WRITABLE);
    ///
    /// // As long a one interest remain this will return `Some`.
    /// let w_interest = RW_INTERESTS.remove(Interest::READABLE).unwrap();
    /// assert!(!w_interest.is_readable());
    /// assert!(w_interest.is_writable());
    ///
    /// // Removing all interests from the set will return `None`.
    /// assert_eq!(w_interest.remove(Interest::WRITABLE), None);
    ///
    /// // Its also possible to remove multiple interests at once.
    /// assert_eq!(RW_INTERESTS.remove(RW_INTERESTS), None);
    /// ```
    pub fn remove(self, other: Interest) -> Option<Interest> {
        NonZeroU8::new(self.0.get() & !other.0.get()).map(Interest)
    }

    /// Returns true if the value includes readable readiness.
    pub const fn is_readable(self) -> bool {
        (self.0.get() & READABLE) != 0
    }

    /// Returns true if the value includes writable readiness.
    pub const fn is_writable(self) -> bool {
        (self.0.get() & WRITABLE) != 0
    }

    /// Returns true if `Interest` contains AIO readiness
    pub const fn is_aio(self) -> bool {
        (self.0.get() & AIO) != 0
    }

    /// Returns true if `Interest` contains LIO readiness
    pub const fn is_lio(self) -> bool {
        (self.0.get() & LIO) != 0
    }
}

impl ops::BitOr for Interest {
    type Output = Self;

    #[inline]
    fn bitor(self, other: Self) -> Self {
        self.add(other)
    }
}

impl ops::BitOrAssign for Interest {
    #[inline]
    fn bitor_assign(&mut self, other: Self) {
        self.0 = (*self | other).0;
    }
}

impl fmt::Debug for Interest {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut one = false;
        if self.is_readable() {
            if one {
                write!(fmt, " | ")?
            }
            write!(fmt, "READABLE")?;
            one = true
        }
        if self.is_writable() {
            if one {
                write!(fmt, " | ")?
            }
            write!(fmt, "WRITABLE")?;
            one = true
        }
        #[cfg(any(
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "ios",
            target_os = "macos"
        ))]
        {
            if self.is_aio() {
                if one {
                    write!(fmt, " | ")?
                }
                write!(fmt, "AIO")?;
                one = true
            }
        }
        #[cfg(any(target_os = "freebsd"))]
        {
            if self.is_lio() {
                if one {
                    write!(fmt, " | ")?
                }
                write!(fmt, "LIO")?;
                one = true
            }
        }
        debug_assert!(one, "printing empty interests");
        Ok(())
    }
}
