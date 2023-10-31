use crate::backtrace::Backtrace;
use crate::chain::Chain;
#[cfg(any(feature = "std", anyhow_no_ptr_addr_of))]
use crate::ptr::Mut;
use crate::ptr::{Own, Ref};
use crate::{Error, StdError};
use alloc::boxed::Box;
use core::any::TypeId;
use core::fmt::{self, Debug, Display};
use core::mem::ManuallyDrop;
#[cfg(not(anyhow_no_ptr_addr_of))]
use core::ptr;
use core::ptr::NonNull;

#[cfg(feature = "std")]
use core::ops::{Deref, DerefMut};

impl Error {
    /// Create a new error object from any error type.
    ///
    /// The error type must be threadsafe and `'static`, so that the `Error`
    /// will be as well.
    ///
    /// If the error type does not provide a backtrace, a backtrace will be
    /// created here to ensure that a backtrace exists.
    #[cfg(feature = "std")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "std")))]
    #[cold]
    #[must_use]
    pub fn new<E>(error: E) -> Self
    where
        E: StdError + Send + Sync + 'static,
    {
        let backtrace = backtrace_if_absent!(error);
        Error::from_std(error, backtrace)
    }

    /// Create a new error object from a printable error message.
    ///
    /// If the argument implements std::error::Error, prefer `Error::new`
    /// instead which preserves the underlying error's cause chain and
    /// backtrace. If the argument may or may not implement std::error::Error
    /// now or in the future, use `anyhow!(err)` which handles either way
    /// correctly.
    ///
    /// `Error::msg("...")` is equivalent to `anyhow!("...")` but occasionally
    /// convenient in places where a function is preferable over a macro, such
    /// as iterator or stream combinators:
    ///
    /// ```
    /// # mod ffi {
    /// #     pub struct Input;
    /// #     pub struct Output;
    /// #     pub async fn do_some_work(_: Input) -> Result<Output, &'static str> {
    /// #         unimplemented!()
    /// #     }
    /// # }
    /// #
    /// # use ffi::{Input, Output};
    /// #
    /// use anyhow::{Error, Result};
    /// use futures::stream::{Stream, StreamExt, TryStreamExt};
    ///
    /// async fn demo<S>(stream: S) -> Result<Vec<Output>>
    /// where
    ///     S: Stream<Item = Input>,
    /// {
    ///     stream
    ///         .then(ffi::do_some_work) // returns Result<Output, &str>
    ///         .map_err(Error::msg)
    ///         .try_collect()
    ///         .await
    /// }
    /// ```
    #[cold]
    #[must_use]
    pub fn msg<M>(message: M) -> Self
    where
        M: Display + Debug + Send + Sync + 'static,
    {
        Error::from_adhoc(message, backtrace!())
    }

    #[cfg(feature = "std")]
    #[cold]
    pub(crate) fn from_std<E>(error: E, backtrace: Option<Backtrace>) -> Self
    where
        E: StdError + Send + Sync + 'static,
    {
        let vtable = &ErrorVTable {
            object_drop: object_drop::<E>,
            object_ref: object_ref::<E>,
            #[cfg(anyhow_no_ptr_addr_of)]
            object_mut: object_mut::<E>,
            object_boxed: object_boxed::<E>,
            object_downcast: object_downcast::<E>,
            #[cfg(anyhow_no_ptr_addr_of)]
            object_downcast_mut: object_downcast_mut::<E>,
            object_drop_rest: object_drop_front::<E>,
            #[cfg(all(not(backtrace), feature = "backtrace"))]
            object_backtrace: no_backtrace,
        };

        // Safety: passing vtable that operates on the right type E.
        unsafe { Error::construct(error, vtable, backtrace) }
    }

    #[cold]
    pub(crate) fn from_adhoc<M>(message: M, backtrace: Option<Backtrace>) -> Self
    where
        M: Display + Debug + Send + Sync + 'static,
    {
        use crate::wrapper::MessageError;
        let error: MessageError<M> = MessageError(message);
        let vtable = &ErrorVTable {
            object_drop: object_drop::<MessageError<M>>,
            object_ref: object_ref::<MessageError<M>>,
            #[cfg(all(feature = "std", anyhow_no_ptr_addr_of))]
            object_mut: object_mut::<MessageError<M>>,
            object_boxed: object_boxed::<MessageError<M>>,
            object_downcast: object_downcast::<M>,
            #[cfg(anyhow_no_ptr_addr_of)]
            object_downcast_mut: object_downcast_mut::<M>,
            object_drop_rest: object_drop_front::<M>,
            #[cfg(all(not(backtrace), feature = "backtrace"))]
            object_backtrace: no_backtrace,
        };

        // Safety: MessageError is repr(transparent) so it is okay for the
        // vtable to allow casting the MessageError<M> to M.
        unsafe { Error::construct(error, vtable, backtrace) }
    }

    #[cold]
    pub(crate) fn from_display<M>(message: M, backtrace: Option<Backtrace>) -> Self
    where
        M: Display + Send + Sync + 'static,
    {
        use crate::wrapper::DisplayError;
        let error: DisplayError<M> = DisplayError(message);
        let vtable = &ErrorVTable {
            object_drop: object_drop::<DisplayError<M>>,
            object_ref: object_ref::<DisplayError<M>>,
            #[cfg(all(feature = "std", anyhow_no_ptr_addr_of))]
            object_mut: object_mut::<DisplayError<M>>,
            object_boxed: object_boxed::<DisplayError<M>>,
            object_downcast: object_downcast::<M>,
            #[cfg(anyhow_no_ptr_addr_of)]
            object_downcast_mut: object_downcast_mut::<M>,
            object_drop_rest: object_drop_front::<M>,
            #[cfg(all(not(backtrace), feature = "backtrace"))]
            object_backtrace: no_backtrace,
        };

        // Safety: DisplayError is repr(transparent) so it is okay for the
        // vtable to allow casting the DisplayError<M> to M.
        unsafe { Error::construct(error, vtable, backtrace) }
    }

    #[cfg(feature = "std")]
    #[cold]
    pub(crate) fn from_context<C, E>(context: C, error: E, backtrace: Option<Backtrace>) -> Self
    where
        C: Display + Send + Sync + 'static,
        E: StdError + Send + Sync + 'static,
    {
        let error: ContextError<C, E> = ContextError { context, error };

        let vtable = &ErrorVTable {
            object_drop: object_drop::<ContextError<C, E>>,
            object_ref: object_ref::<ContextError<C, E>>,
            #[cfg(anyhow_no_ptr_addr_of)]
            object_mut: object_mut::<ContextError<C, E>>,
            object_boxed: object_boxed::<ContextError<C, E>>,
            object_downcast: context_downcast::<C, E>,
            #[cfg(anyhow_no_ptr_addr_of)]
            object_downcast_mut: context_downcast_mut::<C, E>,
            object_drop_rest: context_drop_rest::<C, E>,
            #[cfg(all(not(backtrace), feature = "backtrace"))]
            object_backtrace: no_backtrace,
        };

        // Safety: passing vtable that operates on the right type.
        unsafe { Error::construct(error, vtable, backtrace) }
    }

    #[cfg(feature = "std")]
    #[cold]
    pub(crate) fn from_boxed(
        error: Box<dyn StdError + Send + Sync>,
        backtrace: Option<Backtrace>,
    ) -> Self {
        use crate::wrapper::BoxedError;
        let error = BoxedError(error);
        let vtable = &ErrorVTable {
            object_drop: object_drop::<BoxedError>,
            object_ref: object_ref::<BoxedError>,
            #[cfg(anyhow_no_ptr_addr_of)]
            object_mut: object_mut::<BoxedError>,
            object_boxed: object_boxed::<BoxedError>,
            object_downcast: object_downcast::<Box<dyn StdError + Send + Sync>>,
            #[cfg(anyhow_no_ptr_addr_of)]
            object_downcast_mut: object_downcast_mut::<Box<dyn StdError + Send + Sync>>,
            object_drop_rest: object_drop_front::<Box<dyn StdError + Send + Sync>>,
            #[cfg(all(not(backtrace), feature = "backtrace"))]
            object_backtrace: no_backtrace,
        };

        // Safety: BoxedError is repr(transparent) so it is okay for the vtable
        // to allow casting to Box<dyn StdError + Send + Sync>.
        unsafe { Error::construct(error, vtable, backtrace) }
    }

    // Takes backtrace as argument rather than capturing it here so that the
    // user sees one fewer layer of wrapping noise in the backtrace.
    //
    // Unsafe because the given vtable must have sensible behavior on the error
    // value of type E.
    #[cold]
    unsafe fn construct<E>(
        error: E,
        vtable: &'static ErrorVTable,
        backtrace: Option<Backtrace>,
    ) -> Self
    where
        E: StdError + Send + Sync + 'static,
    {
        let inner: Box<ErrorImpl<E>> = Box::new(ErrorImpl {
            vtable,
            backtrace,
            _object: error,
        });
        // Erase the concrete type of E from the compile-time type system. This
        // is equivalent to the safe unsize coercion from Box<ErrorImpl<E>> to
        // Box<ErrorImpl<dyn StdError + Send + Sync + 'static>> except that the
        // result is a thin pointer. The necessary behavior for manipulating the
        // underlying ErrorImpl<E> is preserved in the vtable provided by the
        // caller rather than a builtin fat pointer vtable.
        let inner = Own::new(inner).cast::<ErrorImpl>();
        Error { inner }
    }

    /// Wrap the error value with additional context.
    ///
    /// For attaching context to a `Result` as it is propagated, the
    /// [`Context`][crate::Context] extension trait may be more convenient than
    /// this function.
    ///
    /// The primary reason to use `error.context(...)` instead of
    /// `result.context(...)` via the `Context` trait would be if the context
    /// needs to depend on some data held by the underlying error:
    ///
    /// ```
    /// # use std::fmt::{self, Debug, Display};
    /// #
    /// # type T = ();
    /// #
    /// # impl std::error::Error for ParseError {}
    /// # impl Debug for ParseError {
    /// #     fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    /// #         unimplemented!()
    /// #     }
    /// # }
    /// # impl Display for ParseError {
    /// #     fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    /// #         unimplemented!()
    /// #     }
    /// # }
    /// #
    /// use anyhow::Result;
    /// use std::fs::File;
    /// use std::path::Path;
    ///
    /// struct ParseError {
    ///     line: usize,
    ///     column: usize,
    /// }
    ///
    /// fn parse_impl(file: File) -> Result<T, ParseError> {
    ///     # const IGNORE: &str = stringify! {
    ///     ...
    ///     # };
    ///     # unimplemented!()
    /// }
    ///
    /// pub fn parse(path: impl AsRef<Path>) -> Result<T> {
    ///     let file = File::open(&path)?;
    ///     parse_impl(file).map_err(|error| {
    ///         let context = format!(
    ///             "only the first {} lines of {} are valid",
    ///             error.line, path.as_ref().display(),
    ///         );
    ///         anyhow::Error::new(error).context(context)
    ///     })
    /// }
    /// ```
    #[cold]
    #[must_use]
    pub fn context<C>(self, context: C) -> Self
    where
        C: Display + Send + Sync + 'static,
    {
        let error: ContextError<C, Error> = ContextError {
            context,
            error: self,
        };

        let vtable = &ErrorVTable {
            object_drop: object_drop::<ContextError<C, Error>>,
            object_ref: object_ref::<ContextError<C, Error>>,
            #[cfg(all(feature = "std", anyhow_no_ptr_addr_of))]
            object_mut: object_mut::<ContextError<C, Error>>,
            object_boxed: object_boxed::<ContextError<C, Error>>,
            object_downcast: context_chain_downcast::<C>,
            #[cfg(anyhow_no_ptr_addr_of)]
            object_downcast_mut: context_chain_downcast_mut::<C>,
            object_drop_rest: context_chain_drop_rest::<C>,
            #[cfg(all(not(backtrace), feature = "backtrace"))]
            object_backtrace: context_backtrace::<C>,
        };

        // As the cause is anyhow::Error, we already have a backtrace for it.
        let backtrace = None;

        // Safety: passing vtable that operates on the right type.
        unsafe { Error::construct(error, vtable, backtrace) }
    }

    /// Get the backtrace for this Error.
    ///
    /// In order for the backtrace to be meaningful, one of the two environment
    /// variables `RUST_LIB_BACKTRACE=1` or `RUST_BACKTRACE=1` must be defined
    /// and `RUST_LIB_BACKTRACE` must not be `0`. Backtraces are somewhat
    /// expensive to capture in Rust, so we don't necessarily want to be
    /// capturing them all over the place all the time.
    ///
    /// - If you want panics and errors to both have backtraces, set
    ///   `RUST_BACKTRACE=1`;
    /// - If you want only errors to have backtraces, set
    ///   `RUST_LIB_BACKTRACE=1`;
    /// - If you want only panics to have backtraces, set `RUST_BACKTRACE=1` and
    ///   `RUST_LIB_BACKTRACE=0`.
    ///
    /// # Stability
    ///
    /// Standard library backtraces are only available on the nightly channel.
    /// Tracking issue: [rust-lang/rust#53487][tracking].
    ///
    /// On stable compilers, this function is only available if the crate's
    /// "backtrace" feature is enabled, and will use the `backtrace` crate as
    /// the underlying backtrace implementation.
    ///
    /// ```toml
    /// [dependencies]
    /// anyhow = { version = "1.0", features = ["backtrace"] }
    /// ```
    ///
    /// [tracking]: https://github.com/rust-lang/rust/issues/53487
    #[cfg(any(backtrace, feature = "backtrace"))]
    #[cfg_attr(doc_cfg, doc(cfg(any(nightly, feature = "backtrace"))))]
    pub fn backtrace(&self) -> &impl_backtrace!() {
        unsafe { ErrorImpl::backtrace(self.inner.by_ref()) }
    }

    /// An iterator of the chain of source errors contained by this Error.
    ///
    /// This iterator will visit every error in the cause chain of this error
    /// object, beginning with the error that this error object was created
    /// from.
    ///
    /// # Example
    ///
    /// ```
    /// use anyhow::Error;
    /// use std::io;
    ///
    /// pub fn underlying_io_error_kind(error: &Error) -> Option<io::ErrorKind> {
    ///     for cause in error.chain() {
    ///         if let Some(io_error) = cause.downcast_ref::<io::Error>() {
    ///             return Some(io_error.kind());
    ///         }
    ///     }
    ///     None
    /// }
    /// ```
    #[cfg(feature = "std")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "std")))]
    #[cold]
    pub fn chain(&self) -> Chain {
        unsafe { ErrorImpl::chain(self.inner.by_ref()) }
    }

    /// The lowest level cause of this error &mdash; this error's cause's
    /// cause's cause etc.
    ///
    /// The root cause is the last error in the iterator produced by
    /// [`chain()`][Error::chain].
    #[cfg(feature = "std")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "std")))]
    pub fn root_cause(&self) -> &(dyn StdError + 'static) {
        self.chain().last().unwrap()
    }

    /// Returns true if `E` is the type held by this error object.
    ///
    /// For errors with context, this method returns true if `E` matches the
    /// type of the context `C` **or** the type of the error on which the
    /// context has been attached. For details about the interaction between
    /// context and downcasting, [see here].
    ///
    /// [see here]: trait.Context.html#effect-on-downcasting
    pub fn is<E>(&self) -> bool
    where
        E: Display + Debug + Send + Sync + 'static,
    {
        self.downcast_ref::<E>().is_some()
    }

    /// Attempt to downcast the error object to a concrete type.
    pub fn downcast<E>(mut self) -> Result<E, Self>
    where
        E: Display + Debug + Send + Sync + 'static,
    {
        let target = TypeId::of::<E>();
        let inner = self.inner.by_mut();
        unsafe {
            // Use vtable to find NonNull<()> which points to a value of type E
            // somewhere inside the data structure.
            #[cfg(not(anyhow_no_ptr_addr_of))]
            let addr = match (vtable(inner.ptr).object_downcast)(inner.by_ref(), target) {
                Some(addr) => addr.by_mut().extend(),
                None => return Err(self),
            };
            #[cfg(anyhow_no_ptr_addr_of)]
            let addr = match (vtable(inner.ptr).object_downcast_mut)(inner, target) {
                Some(addr) => addr.extend(),
                None => return Err(self),
            };

            // Prepare to read E out of the data structure. We'll drop the rest
            // of the data structure separately so that E is not dropped.
            let outer = ManuallyDrop::new(self);

            // Read E from where the vtable found it.
            let error = addr.cast::<E>().read();

            // Drop rest of the data structure outside of E.
            (vtable(outer.inner.ptr).object_drop_rest)(outer.inner, target);

            Ok(error)
        }
    }

    /// Downcast this error object by reference.
    ///
    /// # Example
    ///
    /// ```
    /// # use anyhow::anyhow;
    /// # use std::fmt::{self, Display};
    /// # use std::task::Poll;
    /// #
    /// # #[derive(Debug)]
    /// # enum DataStoreError {
    /// #     Censored(()),
    /// # }
    /// #
    /// # impl Display for DataStoreError {
    /// #     fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    /// #         unimplemented!()
    /// #     }
    /// # }
    /// #
    /// # impl std::error::Error for DataStoreError {}
    /// #
    /// # const REDACTED_CONTENT: () = ();
    /// #
    /// # let error = anyhow!("...");
    /// # let root_cause = &error;
    /// #
    /// # let ret =
    /// // If the error was caused by redaction, then return a tombstone instead
    /// // of the content.
    /// match root_cause.downcast_ref::<DataStoreError>() {
    ///     Some(DataStoreError::Censored(_)) => Ok(Poll::Ready(REDACTED_CONTENT)),
    ///     None => Err(error),
    /// }
    /// # ;
    /// ```
    pub fn downcast_ref<E>(&self) -> Option<&E>
    where
        E: Display + Debug + Send + Sync + 'static,
    {
        let target = TypeId::of::<E>();
        unsafe {
            // Use vtable to find NonNull<()> which points to a value of type E
            // somewhere inside the data structure.
            let addr = (vtable(self.inner.ptr).object_downcast)(self.inner.by_ref(), target)?;
            Some(addr.cast::<E>().deref())
        }
    }

    /// Downcast this error object by mutable reference.
    pub fn downcast_mut<E>(&mut self) -> Option<&mut E>
    where
        E: Display + Debug + Send + Sync + 'static,
    {
        let target = TypeId::of::<E>();
        unsafe {
            // Use vtable to find NonNull<()> which points to a value of type E
            // somewhere inside the data structure.

            #[cfg(not(anyhow_no_ptr_addr_of))]
            let addr =
                (vtable(self.inner.ptr).object_downcast)(self.inner.by_ref(), target)?.by_mut();

            #[cfg(anyhow_no_ptr_addr_of)]
            let addr = (vtable(self.inner.ptr).object_downcast_mut)(self.inner.by_mut(), target)?;

            Some(addr.cast::<E>().deref_mut())
        }
    }
}

#[cfg(feature = "std")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "std")))]
impl<E> From<E> for Error
where
    E: StdError + Send + Sync + 'static,
{
    #[cold]
    fn from(error: E) -> Self {
        let backtrace = backtrace_if_absent!(error);
        Error::from_std(error, backtrace)
    }
}

#[cfg(feature = "std")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "std")))]
impl Deref for Error {
    type Target = dyn StdError + Send + Sync + 'static;

    fn deref(&self) -> &Self::Target {
        unsafe { ErrorImpl::error(self.inner.by_ref()) }
    }
}

#[cfg(feature = "std")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "std")))]
impl DerefMut for Error {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { ErrorImpl::error_mut(self.inner.by_mut()) }
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        unsafe { ErrorImpl::display(self.inner.by_ref(), formatter) }
    }
}

impl Debug for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        unsafe { ErrorImpl::debug(self.inner.by_ref(), formatter) }
    }
}

impl Drop for Error {
    fn drop(&mut self) {
        unsafe {
            // Invoke the vtable's drop behavior.
            (vtable(self.inner.ptr).object_drop)(self.inner);
        }
    }
}

struct ErrorVTable {
    object_drop: unsafe fn(Own<ErrorImpl>),
    object_ref: unsafe fn(Ref<ErrorImpl>) -> Ref<dyn StdError + Send + Sync + 'static>,
    #[cfg(all(feature = "std", anyhow_no_ptr_addr_of))]
    object_mut: unsafe fn(Mut<ErrorImpl>) -> &mut (dyn StdError + Send + Sync + 'static),
    object_boxed: unsafe fn(Own<ErrorImpl>) -> Box<dyn StdError + Send + Sync + 'static>,
    object_downcast: unsafe fn(Ref<ErrorImpl>, TypeId) -> Option<Ref<()>>,
    #[cfg(anyhow_no_ptr_addr_of)]
    object_downcast_mut: unsafe fn(Mut<ErrorImpl>, TypeId) -> Option<Mut<()>>,
    object_drop_rest: unsafe fn(Own<ErrorImpl>, TypeId),
    #[cfg(all(not(backtrace), feature = "backtrace"))]
    object_backtrace: unsafe fn(Ref<ErrorImpl>) -> Option<&Backtrace>,
}

// Safety: requires layout of *e to match ErrorImpl<E>.
unsafe fn object_drop<E>(e: Own<ErrorImpl>) {
    // Cast back to ErrorImpl<E> so that the allocator receives the correct
    // Layout to deallocate the Box's memory.
    let unerased = e.cast::<ErrorImpl<E>>().boxed();
    drop(unerased);
}

// Safety: requires layout of *e to match ErrorImpl<E>.
unsafe fn object_drop_front<E>(e: Own<ErrorImpl>, target: TypeId) {
    // Drop the fields of ErrorImpl other than E as well as the Box allocation,
    // without dropping E itself. This is used by downcast after doing a
    // ptr::read to take ownership of the E.
    let _ = target;
    let unerased = e.cast::<ErrorImpl<ManuallyDrop<E>>>().boxed();
    drop(unerased);
}

// Safety: requires layout of *e to match ErrorImpl<E>.
unsafe fn object_ref<E>(e: Ref<ErrorImpl>) -> Ref<dyn StdError + Send + Sync + 'static>
where
    E: StdError + Send + Sync + 'static,
{
    // Attach E's native StdError vtable onto a pointer to self._object.

    let unerased = e.cast::<ErrorImpl<E>>();

    #[cfg(not(anyhow_no_ptr_addr_of))]
    return Ref::from_raw(NonNull::new_unchecked(
        ptr::addr_of!((*unerased.as_ptr())._object) as *mut E,
    ));

    #[cfg(anyhow_no_ptr_addr_of)]
    return Ref::new(&unerased.deref()._object);
}

// Safety: requires layout of *e to match ErrorImpl<E>, and for `e` to be derived
// from a `&mut`
#[cfg(all(feature = "std", anyhow_no_ptr_addr_of))]
unsafe fn object_mut<E>(e: Mut<ErrorImpl>) -> &mut (dyn StdError + Send + Sync + 'static)
where
    E: StdError + Send + Sync + 'static,
{
    // Attach E's native StdError vtable onto a pointer to self._object.
    &mut e.cast::<ErrorImpl<E>>().deref_mut()._object
}

// Safety: requires layout of *e to match ErrorImpl<E>.
unsafe fn object_boxed<E>(e: Own<ErrorImpl>) -> Box<dyn StdError + Send + Sync + 'static>
where
    E: StdError + Send + Sync + 'static,
{
    // Attach ErrorImpl<E>'s native StdError vtable. The StdError impl is below.
    e.cast::<ErrorImpl<E>>().boxed()
}

// Safety: requires layout of *e to match ErrorImpl<E>.
unsafe fn object_downcast<E>(e: Ref<ErrorImpl>, target: TypeId) -> Option<Ref<()>>
where
    E: 'static,
{
    if TypeId::of::<E>() == target {
        // Caller is looking for an E pointer and e is ErrorImpl<E>, take a
        // pointer to its E field.

        let unerased = e.cast::<ErrorImpl<E>>();

        #[cfg(not(anyhow_no_ptr_addr_of))]
        return Some(
            Ref::from_raw(NonNull::new_unchecked(
                ptr::addr_of!((*unerased.as_ptr())._object) as *mut E,
            ))
            .cast::<()>(),
        );

        #[cfg(anyhow_no_ptr_addr_of)]
        return Some(Ref::new(&unerased.deref()._object).cast::<()>());
    } else {
        None
    }
}

// Safety: requires layout of *e to match ErrorImpl<E>.
#[cfg(anyhow_no_ptr_addr_of)]
unsafe fn object_downcast_mut<E>(e: Mut<ErrorImpl>, target: TypeId) -> Option<Mut<()>>
where
    E: 'static,
{
    if TypeId::of::<E>() == target {
        // Caller is looking for an E pointer and e is ErrorImpl<E>, take a
        // pointer to its E field.
        let unerased = e.cast::<ErrorImpl<E>>().deref_mut();
        Some(Mut::new(&mut unerased._object).cast::<()>())
    } else {
        None
    }
}

#[cfg(all(not(backtrace), feature = "backtrace"))]
fn no_backtrace(e: Ref<ErrorImpl>) -> Option<&Backtrace> {
    let _ = e;
    None
}

// Safety: requires layout of *e to match ErrorImpl<ContextError<C, E>>.
#[cfg(feature = "std")]
unsafe fn context_downcast<C, E>(e: Ref<ErrorImpl>, target: TypeId) -> Option<Ref<()>>
where
    C: 'static,
    E: 'static,
{
    if TypeId::of::<C>() == target {
        let unerased = e.cast::<ErrorImpl<ContextError<C, E>>>().deref();
        Some(Ref::new(&unerased._object.context).cast::<()>())
    } else if TypeId::of::<E>() == target {
        let unerased = e.cast::<ErrorImpl<ContextError<C, E>>>().deref();
        Some(Ref::new(&unerased._object.error).cast::<()>())
    } else {
        None
    }
}

// Safety: requires layout of *e to match ErrorImpl<ContextError<C, E>>.
#[cfg(all(feature = "std", anyhow_no_ptr_addr_of))]
unsafe fn context_downcast_mut<C, E>(e: Mut<ErrorImpl>, target: TypeId) -> Option<Mut<()>>
where
    C: 'static,
    E: 'static,
{
    if TypeId::of::<C>() == target {
        let unerased = e.cast::<ErrorImpl<ContextError<C, E>>>().deref_mut();
        Some(Mut::new(&mut unerased._object.context).cast::<()>())
    } else if TypeId::of::<E>() == target {
        let unerased = e.cast::<ErrorImpl<ContextError<C, E>>>().deref_mut();
        Some(Mut::new(&mut unerased._object.error).cast::<()>())
    } else {
        None
    }
}

// Safety: requires layout of *e to match ErrorImpl<ContextError<C, E>>.
#[cfg(feature = "std")]
unsafe fn context_drop_rest<C, E>(e: Own<ErrorImpl>, target: TypeId)
where
    C: 'static,
    E: 'static,
{
    // Called after downcasting by value to either the C or the E and doing a
    // ptr::read to take ownership of that value.
    if TypeId::of::<C>() == target {
        let unerased = e
            .cast::<ErrorImpl<ContextError<ManuallyDrop<C>, E>>>()
            .boxed();
        drop(unerased);
    } else {
        let unerased = e
            .cast::<ErrorImpl<ContextError<C, ManuallyDrop<E>>>>()
            .boxed();
        drop(unerased);
    }
}

// Safety: requires layout of *e to match ErrorImpl<ContextError<C, Error>>.
unsafe fn context_chain_downcast<C>(e: Ref<ErrorImpl>, target: TypeId) -> Option<Ref<()>>
where
    C: 'static,
{
    let unerased = e.cast::<ErrorImpl<ContextError<C, Error>>>().deref();
    if TypeId::of::<C>() == target {
        Some(Ref::new(&unerased._object.context).cast::<()>())
    } else {
        // Recurse down the context chain per the inner error's vtable.
        let source = &unerased._object.error;
        (vtable(source.inner.ptr).object_downcast)(source.inner.by_ref(), target)
    }
}

// Safety: requires layout of *e to match ErrorImpl<ContextError<C, Error>>.
#[cfg(anyhow_no_ptr_addr_of)]
unsafe fn context_chain_downcast_mut<C>(e: Mut<ErrorImpl>, target: TypeId) -> Option<Mut<()>>
where
    C: 'static,
{
    let unerased = e.cast::<ErrorImpl<ContextError<C, Error>>>().deref_mut();
    if TypeId::of::<C>() == target {
        Some(Mut::new(&mut unerased._object.context).cast::<()>())
    } else {
        // Recurse down the context chain per the inner error's vtable.
        let source = &mut unerased._object.error;
        (vtable(source.inner.ptr).object_downcast_mut)(source.inner.by_mut(), target)
    }
}

// Safety: requires layout of *e to match ErrorImpl<ContextError<C, Error>>.
unsafe fn context_chain_drop_rest<C>(e: Own<ErrorImpl>, target: TypeId)
where
    C: 'static,
{
    // Called after downcasting by value to either the C or one of the causes
    // and doing a ptr::read to take ownership of that value.
    if TypeId::of::<C>() == target {
        let unerased = e
            .cast::<ErrorImpl<ContextError<ManuallyDrop<C>, Error>>>()
            .boxed();
        // Drop the entire rest of the data structure rooted in the next Error.
        drop(unerased);
    } else {
        let unerased = e
            .cast::<ErrorImpl<ContextError<C, ManuallyDrop<Error>>>>()
            .boxed();
        // Read the Own<ErrorImpl> from the next error.
        let inner = unerased._object.error.inner;
        drop(unerased);
        let vtable = vtable(inner.ptr);
        // Recursively drop the next error using the same target typeid.
        (vtable.object_drop_rest)(inner, target);
    }
}

// Safety: requires layout of *e to match ErrorImpl<ContextError<C, Error>>.
#[cfg(all(not(backtrace), feature = "backtrace"))]
#[allow(clippy::unnecessary_wraps)]
unsafe fn context_backtrace<C>(e: Ref<ErrorImpl>) -> Option<&Backtrace>
where
    C: 'static,
{
    let unerased = e.cast::<ErrorImpl<ContextError<C, Error>>>().deref();
    let backtrace = ErrorImpl::backtrace(unerased._object.error.inner.by_ref());
    Some(backtrace)
}

// NOTE: If working with `ErrorImpl<()>`, references should be avoided in favor
// of raw pointers and `NonNull`.
// repr C to ensure that E remains in the final position.
#[repr(C)]
pub(crate) struct ErrorImpl<E = ()> {
    vtable: &'static ErrorVTable,
    backtrace: Option<Backtrace>,
    // NOTE: Don't use directly. Use only through vtable. Erased type may have
    // different alignment.
    _object: E,
}

// Reads the vtable out of `p`. This is the same as `p.as_ref().vtable`, but
// avoids converting `p` into a reference.
unsafe fn vtable(p: NonNull<ErrorImpl>) -> &'static ErrorVTable {
    // NOTE: This assumes that `ErrorVTable` is the first field of ErrorImpl.
    *(p.as_ptr() as *const &'static ErrorVTable)
}

// repr C to ensure that ContextError<C, E> has the same layout as
// ContextError<ManuallyDrop<C>, E> and ContextError<C, ManuallyDrop<E>>.
#[repr(C)]
pub(crate) struct ContextError<C, E> {
    pub context: C,
    pub error: E,
}

impl<E> ErrorImpl<E> {
    fn erase(&self) -> Ref<ErrorImpl> {
        // Erase the concrete type of E but preserve the vtable in self.vtable
        // for manipulating the resulting thin pointer. This is analogous to an
        // unsize coercion.
        Ref::new(self).cast::<ErrorImpl>()
    }
}

impl ErrorImpl {
    pub(crate) unsafe fn error(this: Ref<Self>) -> &(dyn StdError + Send + Sync + 'static) {
        // Use vtable to attach E's native StdError vtable for the right
        // original type E.
        (vtable(this.ptr).object_ref)(this).deref()
    }

    #[cfg(feature = "std")]
    pub(crate) unsafe fn error_mut(this: Mut<Self>) -> &mut (dyn StdError + Send + Sync + 'static) {
        // Use vtable to attach E's native StdError vtable for the right
        // original type E.

        #[cfg(not(anyhow_no_ptr_addr_of))]
        return (vtable(this.ptr).object_ref)(this.by_ref())
            .by_mut()
            .deref_mut();

        #[cfg(anyhow_no_ptr_addr_of)]
        return (vtable(this.ptr).object_mut)(this);
    }

    #[cfg(any(backtrace, feature = "backtrace"))]
    pub(crate) unsafe fn backtrace(this: Ref<Self>) -> &Backtrace {
        // This unwrap can only panic if the underlying error's backtrace method
        // is nondeterministic, which would only happen in maliciously
        // constructed code.
        this.deref()
            .backtrace
            .as_ref()
            .or_else(|| {
                #[cfg(backtrace)]
                return Self::error(this).backtrace();
                #[cfg(all(not(backtrace), feature = "backtrace"))]
                return (vtable(this.ptr).object_backtrace)(this);
            })
            .expect("backtrace capture failed")
    }

    #[cold]
    pub(crate) unsafe fn chain(this: Ref<Self>) -> Chain {
        Chain::new(Self::error(this))
    }
}

impl<E> StdError for ErrorImpl<E>
where
    E: StdError,
{
    #[cfg(backtrace)]
    fn backtrace(&self) -> Option<&Backtrace> {
        Some(unsafe { ErrorImpl::backtrace(self.erase()) })
    }

    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        unsafe { ErrorImpl::error(self.erase()).source() }
    }
}

impl<E> Debug for ErrorImpl<E>
where
    E: Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        unsafe { ErrorImpl::debug(self.erase(), formatter) }
    }
}

impl<E> Display for ErrorImpl<E>
where
    E: Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        unsafe { Display::fmt(ErrorImpl::error(self.erase()), formatter) }
    }
}

impl From<Error> for Box<dyn StdError + Send + Sync + 'static> {
    #[cold]
    fn from(error: Error) -> Self {
        let outer = ManuallyDrop::new(error);
        unsafe {
            // Use vtable to attach ErrorImpl<E>'s native StdError vtable for
            // the right original type E.
            (vtable(outer.inner.ptr).object_boxed)(outer.inner)
        }
    }
}

impl From<Error> for Box<dyn StdError + Send + 'static> {
    fn from(error: Error) -> Self {
        Box::<dyn StdError + Send + Sync>::from(error)
    }
}

impl From<Error> for Box<dyn StdError + 'static> {
    fn from(error: Error) -> Self {
        Box::<dyn StdError + Send + Sync>::from(error)
    }
}

#[cfg(feature = "std")]
impl AsRef<dyn StdError + Send + Sync> for Error {
    fn as_ref(&self) -> &(dyn StdError + Send + Sync + 'static) {
        &**self
    }
}

#[cfg(feature = "std")]
impl AsRef<dyn StdError> for Error {
    fn as_ref(&self) -> &(dyn StdError + 'static) {
        &**self
    }
}
