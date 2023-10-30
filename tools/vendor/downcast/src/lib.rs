#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
mod std {
    pub use core::*;
}

use std::any::{Any as StdAny, TypeId, type_name};
use std::fmt::{self, Debug, Display};

#[cfg(feature = "std")]
use std::{error::Error, rc::Rc, sync::Arc};

// ++++++++++++++++++++ Any ++++++++++++++++++++

pub trait Any: StdAny {
    #[doc(hidden)]
    fn as_any(&self) -> &dyn StdAny;
    
    #[doc(hidden)]
    fn as_any_mut(&mut self) -> &mut dyn StdAny;
    
    #[doc(hidden)]
    #[cfg(feature = "std")]
    fn into_any(self: Box<Self>) -> Box<dyn StdAny>;
    
    #[doc(hidden)]
    #[cfg(feature = "std")]
    fn into_any_rc(self: Rc<Self>) -> Rc<dyn StdAny>;
    
    fn type_name(&self) -> &'static str;
}

impl<T> Any for T where T: StdAny {
    #[doc(hidden)]
    fn as_any(&self) -> &dyn StdAny { self }
    
    #[doc(hidden)]
    fn as_any_mut(&mut self) -> &mut dyn StdAny { self }
    
    #[cfg(feature = "std")]
    fn into_any(self: Box<Self>) -> Box<dyn StdAny> { self }
    
    #[cfg(feature = "std")]
    fn into_any_rc(self: Rc<Self>) -> Rc<dyn StdAny> { self }
    
    fn type_name(&self) -> &'static str { type_name::<Self>() }
}

#[cfg(feature = "std")]
pub trait AnySync: Any + Send + Sync {
    fn into_any_arc(self: Arc<Self>) -> Arc<dyn StdAny + Send + Sync>;
}

#[cfg(feature = "std")]
impl<T> AnySync for T where T: Any + Send + Sync {
    fn into_any_arc(self: Arc<Self>) -> Arc<dyn StdAny + Send + Sync> { self }
}

// ++++++++++++++++++++ TypeMismatch ++++++++++++++++++++

#[derive(Debug, Clone, Copy)]
pub struct TypeMismatch {
    pub expected: &'static str,
    pub found: &'static str,
}

impl TypeMismatch {
    pub fn new<T, O>(found_obj: &O) -> Self
        where T: Any + ?Sized, O: Any + ?Sized
    {
        TypeMismatch {
            expected: type_name::<T>(),
            found: found_obj.type_name(),
        }
    }
}

impl Display for TypeMismatch {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "Type mismatch: Expected '{}', found '{}'!", self.expected, self.found)
    }
}

#[cfg(feature = "std")]
impl Error for TypeMismatch {}

// ++++++++++++++++++++ DowncastError ++++++++++++++++++++

pub struct DowncastError<O> {
    mismatch: TypeMismatch,
    object: O,
}

impl<O> DowncastError<O> {
    pub fn new(mismatch: TypeMismatch, object: O) -> Self {
        Self{ mismatch, object }
    }
    pub fn type_mismatch(&self) -> TypeMismatch { self.mismatch }
    pub fn into_object(self) -> O { self.object }
}

impl<O> Debug for DowncastError<O> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("DowncastError")
            .field("mismatch", &self.mismatch)
            .finish()
    }
}

impl<O> Display for DowncastError<O> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.mismatch, fmt)
    }
}

#[cfg(feature = "std")]
impl<O> Error for DowncastError<O> {}

// ++++++++++++++++++++ Downcast ++++++++++++++++++++

pub trait Downcast<T>: Any
    where T: Any
{
    fn is_type(&self) -> bool { self.type_id() == TypeId::of::<T>() }

    fn downcast_ref(&self) -> Result<&T, TypeMismatch> {
        if self.is_type() {
            Ok(self.as_any().downcast_ref().unwrap())
        } else {
            Err(TypeMismatch::new::<T, Self>(self))
        }
    }

    fn downcast_mut(&mut self) -> Result<&mut T, TypeMismatch> {
        if self.is_type() {
            Ok(self.as_any_mut().downcast_mut().unwrap())
        } else {
            Err(TypeMismatch::new::<T, Self>(self))
        }
    }

    #[cfg(feature = "std")]
    fn downcast(self: Box<Self>) -> Result<Box<T>, DowncastError<Box<Self>>> {
        if self.is_type() {
            Ok(self.into_any().downcast().unwrap())
        } else {
            let mismatch = TypeMismatch::new::<T, Self>(&*self);
            Err(DowncastError::new(mismatch, self))
        }
    }

    #[cfg(feature = "std")]
    fn downcast_rc(self: Rc<Self>) -> Result<Rc<T>, DowncastError<Rc<Self>>> {
        if self.is_type() {
            Ok(self.into_any_rc().downcast().unwrap())
        } else {
            let mismatch = TypeMismatch::new::<T, Self>(&*self);
            Err(DowncastError::new(mismatch, self))
        }
    }
}

#[cfg(feature = "std")]
pub trait DowncastSync<T>: Downcast<T> + AnySync
    where T: AnySync
{
    fn downcast_arc(self: Arc<Self>) -> Result<Arc<T>, DowncastError<Arc<Self>>> {
        if self.is_type() {
            Ok(self.into_any_arc().downcast().unwrap())
        } else {
            let mismatch = TypeMismatch::new::<T, Self>(&*self);
            Err(DowncastError::new(mismatch, self))
        }
    }
}

// ++++++++++++++++++++ macros ++++++++++++++++++++

#[doc(hidden)]
pub mod _std {
    #[cfg(feature = "std")]
    pub use std::*;
    #[cfg(not(feature = "std"))]
    pub use core::*;
}

/// Implements [`Downcast`](trait.Downcast.html) for your trait-object-type.
///
/// ```ignore
/// impl_downcast!(Foo);
/// impl_downcast!(<B> Foo<B> where B: Bar);
/// impl_downcast!(<B> Foo<Bar = B>);
/// ```
///
/// expands to
///
/// ```ignore
/// impl<T> Downcast<T> for Foo
///     where T: Any
/// {}
///
/// impl<T, B> Downcast<T> for Foo<B>
///     where T: Any, B: Bar
/// {}
///
/// impl<T, B> Downcast<T> for Foo<Bar = B>
///     where T: Any
/// {}
/// ```
#[macro_export]
macro_rules! impl_downcast {
    (<$($params:ident),+ $(,)*> $base:ty $(where $($bounds:tt)+)*) => {
        impl<_T, $($params),+> $crate::Downcast<_T> for $base
            where _T: $crate::Any, $($params: 'static,)* $($($bounds)+)*
        {}
    };
    ($base:ty) => {
        impl<_T> $crate::Downcast<_T> for $base
            where _T: $crate::Any
        {}
    };
}

/// Implements [`Downcast`](trait.Downcast.html) and [`DowncastSync`](trait.DowncastSync.html) for your trait-object-type.
#[cfg(feature = "std")]
#[macro_export]
macro_rules! impl_downcast_sync {
    (<$($params:ident),+ $(,)*> $base:ty $(where $($bounds:tt)+)*) => {
        impl<_T, $($params),+> $crate::Downcast<_T> for $base
            where _T: $crate::Any, $($params: 'static,)* $($($bounds)+)*
        {}
    
        impl<_T, $($params),+> $crate::DowncastSync<_T> for $base
            where _T: $crate::AnySync, $($params: 'static,)* $($($bounds)+)*
        {}
    };
    ($base:ty) => {
        impl<_T> $crate::Downcast<_T> for $base
            where _T: $crate::Any
        {}
        
        impl<_T> $crate::DowncastSync<_T> for $base
            where _T: $crate::AnySync
        {}
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! downcast_methods_core {
    (@items) => {
        #[allow(unused, missing_docs)]
        pub fn is<_T>(&self) -> bool
            where _T: $crate::Any, Self: $crate::Downcast<_T>
        {
            $crate::Downcast::<_T>::is_type(self)
        }

        #[allow(unused, missing_docs)]
        pub fn downcast_ref<_T>(&self) -> $crate::_std::result::Result<&_T, $crate::TypeMismatch>
            where _T: $crate::Any, Self: $crate::Downcast<_T>
        {
            $crate::Downcast::<_T>::downcast_ref(self)
        }

        #[allow(unused, missing_docs)]
        pub fn downcast_mut<_T>(&mut self) -> $crate::_std::result::Result<&mut _T, $crate::TypeMismatch>
            where _T: $crate::Any, Self: $crate::Downcast<_T>
        {
            $crate::Downcast::<_T>::downcast_mut(self)
        }
    };
    (<$($params:ident),+ $(,)*> $base:ty $(where $($bounds:tt)+)*) => {
        impl<$($params),+> $base
            where $($params: 'static,)* $($($bounds)+)*
        {
            $crate::downcast_methods_core!(@items);
        }
    };
    ($base:ty) => {
        impl $base {
            $crate::downcast_methods_core!(@items);
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! downcast_methods_std {
    (@items) => {
        $crate::downcast_methods_core!(@items);

        #[allow(unused, missing_docs)]
        pub fn downcast<_T>(self: $crate::_std::boxed::Box<Self>) -> $crate::_std::result::Result<$crate::_std::boxed::Box<_T>, $crate::DowncastError<$crate::_std::boxed::Box<Self>>>
            where _T: $crate::Any, Self: $crate::Downcast<_T>
        {
            $crate::Downcast::<_T>::downcast(self)
        }

        #[allow(unused, missing_docs)]
        pub fn downcast_rc<_T>(self: $crate::_std::rc::Rc<Self>) -> $crate::_std::result::Result<$crate::_std::rc::Rc<_T>, $crate::DowncastError<$crate::_std::rc::Rc<Self>>>
            where _T: $crate::Any, Self: $crate::Downcast<_T>
        {
            $crate::Downcast::<_T>::downcast_rc(self)
        }
    };
    (<$($params:ident),+ $(,)*> $base:ty $(where $($bounds:tt)+)*) => {
        impl<$($params),+> $base
            $(where $($bounds)+)*
        {
            $crate::downcast_methods_std!(@items);
        }
    };
    ($base:ty) => {
        impl $base {
            $crate::downcast_methods_std!(@items);
        }
    };
}

#[doc(hidden)]
#[cfg(feature = "std")]
#[macro_export]
macro_rules! downcast_sync_methods {
    (@items) => {
        $crate::downcast_methods_std!(@items);

        #[allow(unused, missing_docs)]
        pub fn downcast_arc<_T>(self: $crate::_std::sync::Arc<Self>) -> $crate::_std::result::Result<$crate::_std::sync::Arc<_T>, $crate::DowncastError<$crate::_std::sync::Arc<Self>>>
            where _T: $crate::AnySync, Self: $crate::DowncastSync<_T>
        {
            $crate::DowncastSync::<_T>::downcast_arc(self)
        }
    };
    (<$($params:ident),+ $(,)*> $base:ty $(where $($bounds:tt)+)*) => {
        impl<$($params),+> $base
            $(where $($bounds)+)*
        {
            $crate::downcast_sync_methods!(@items);
        }
    };
    ($base:ty) => {
        impl $base {
            $crate::downcast_sync_methods!(@items);
        }
    };
}


/// Generate `downcast`-methods for your trait-object-type.
///
/// ```ignore
/// downcast_methods!(Foo);
/// downcast_methods!(<B> Foo<B> where B: Bar);
/// downcast_methods!(<B> Foo<Bar = B>);
/// ```
///
/// ```ignore
/// impl dyn Foo {
/// /* impl<B> dyn Foo<B> where B: Bar { */
/// /* impl<B> dyn Foo<Bar = B> { */
///
///     pub fn is<T>(&self) -> bool
///         where T: Any, Self: Downcast<T>
///     { ... }
///
///     pub fn downcast_ref<T>(&self) -> Result<&T, TypeMismatch>
///         where T: Any, Self: Downcast<T>
///     { ... }
///
///     pub fn downcast_mut<T>(&mut self) -> Result<&mut T, TypeMismatch>
///         where T: Any, Self: Downcast<T>
///     { ... }
/// }
/// ```
#[cfg(not(feature = "std"))]
#[macro_export]
macro_rules! downcast_methods {
    ($($tt:tt)+) => { $crate::downcast_methods_core!($($tt)+); }
}

/// Generate `downcast`-methods for your trait-object-type.
///
/// ```ignore
/// downcast_methods!(Foo);
/// downcast_methods!(<B> Foo<B> where B: Bar);
/// downcast_methods!(<B> Foo<Bar = B>);
/// ```
///
/// ```ignore
/// impl dyn Foo {
/// /* impl<B> dyn Foo<B> where B: Bar {  */
/// /* impl<B> dyn Foo<Bar = B> {  */
///
///     pub fn is<T>(&self) -> bool
///         where T: Any, Self: Downcast<T>
///     { ... }
///
///     pub fn downcast_ref<T>(&self) -> Result<&T, TypeMismatch>
///         where T: Any, Self: Downcast<T>
///     { ... }
///
///     pub fn downcast_mut<T>(&mut self) -> Result<&mut T, TypeMismatch>
///         where T: Any, Self: Downcast<T>
///     { ... }
///
///     pub fn downcast<T>(self: Box<Self>) -> Result<Box<T>, DowncastError<Box<T>>>
///         where T: Any, Self: Downcast<T>
///     { ... }
/// }
/// ```
#[cfg(feature = "std")]
#[macro_export]
macro_rules! downcast_methods {
    ($($tt:tt)+) => { $crate::downcast_methods_std!($($tt)+); }
}

/// Implements [`Downcast`](trait.Downcast.html) and generates
/// `downcast`-methods for your trait-object-type.
///
/// See [`impl_downcast`](macro.impl_downcast.html),
/// [`downcast_methods`](macro.downcast_methods.html).
#[macro_export]
macro_rules! downcast {
    ($($tt:tt)+) => {
        $crate::impl_downcast!($($tt)+);
        $crate::downcast_methods!($($tt)+);
    }
}

/// Implements [`DowncastSync`](trait.DowncastSync.html) and generates
/// `downcast`-methods for your trait-object-type.
///
/// See [`impl_downcast_sync`](macro.impl_downcast.html),
/// [`downcast_sync_methods`](macro.downcast_methods.html).
#[cfg(feature = "std")]
#[macro_export]
macro_rules! downcast_sync {
    ($($tt:tt)+) => {
        $crate::impl_downcast_sync!($($tt)+);
        $crate::downcast_sync_methods!($($tt)+);
    }
}

// NOTE: We only implement the trait, because implementing the methods won't
// be possible when we replace downcast::Any by std::any::Any.
downcast!(dyn Any);
downcast!((dyn Any + Send));
downcast!((dyn Any + Sync));
#[cfg(feature = "std")]
downcast_sync!(dyn AnySync);

