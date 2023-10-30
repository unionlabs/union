use crate::fmt::display;
use crate::kind::Trivial;
use crate::string::CxxString;
use crate::weak_ptr::{WeakPtr, WeakPtrTarget};
use crate::ExternType;
use core::ffi::c_void;
use core::fmt::{self, Debug, Display};
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::ops::Deref;

/// Binding to C++ `std::shared_ptr<T>`.
#[repr(C)]
pub struct SharedPtr<T>
where
    T: SharedPtrTarget,
{
    repr: [MaybeUninit<*mut c_void>; 2],
    ty: PhantomData<T>,
}

impl<T> SharedPtr<T>
where
    T: SharedPtrTarget,
{
    /// Makes a new SharedPtr wrapping a null pointer.
    ///
    /// Matches the behavior of default-constructing a std::shared\_ptr.
    pub fn null() -> Self {
        let mut shared_ptr = MaybeUninit::<SharedPtr<T>>::uninit();
        let new = shared_ptr.as_mut_ptr().cast();
        unsafe {
            T::__null(new);
            shared_ptr.assume_init()
        }
    }

    /// Allocates memory on the heap and makes a SharedPtr owner for it.
    pub fn new(value: T) -> Self
    where
        T: ExternType<Kind = Trivial>,
    {
        let mut shared_ptr = MaybeUninit::<SharedPtr<T>>::uninit();
        let new = shared_ptr.as_mut_ptr().cast();
        unsafe {
            T::__new(value, new);
            shared_ptr.assume_init()
        }
    }

    /// Checks whether the SharedPtr does not own an object.
    ///
    /// This is the opposite of [std::shared_ptr\<T\>::operator bool](https://en.cppreference.com/w/cpp/memory/shared_ptr/operator_bool).
    pub fn is_null(&self) -> bool {
        let this = self as *const Self as *const c_void;
        let ptr = unsafe { T::__get(this) };
        ptr.is_null()
    }

    /// Returns a reference to the object owned by this SharedPtr if any,
    /// otherwise None.
    pub fn as_ref(&self) -> Option<&T> {
        let this = self as *const Self as *const c_void;
        unsafe { T::__get(this).as_ref() }
    }

    /// Constructs new WeakPtr as a non-owning reference to the object managed
    /// by `self`. If `self` manages no object, the WeakPtr manages no object
    /// too.
    ///
    /// Matches the behavior of [std::weak_ptr\<T\>::weak_ptr(const std::shared_ptr\<T\> \&)](https://en.cppreference.com/w/cpp/memory/weak_ptr/weak_ptr).
    pub fn downgrade(self: &SharedPtr<T>) -> WeakPtr<T>
    where
        T: WeakPtrTarget,
    {
        let this = self as *const Self as *const c_void;
        let mut weak_ptr = MaybeUninit::<WeakPtr<T>>::uninit();
        let new = weak_ptr.as_mut_ptr().cast();
        unsafe {
            T::__downgrade(this, new);
            weak_ptr.assume_init()
        }
    }
}

unsafe impl<T> Send for SharedPtr<T> where T: Send + Sync + SharedPtrTarget {}
unsafe impl<T> Sync for SharedPtr<T> where T: Send + Sync + SharedPtrTarget {}

impl<T> Clone for SharedPtr<T>
where
    T: SharedPtrTarget,
{
    fn clone(&self) -> Self {
        let mut shared_ptr = MaybeUninit::<SharedPtr<T>>::uninit();
        let new = shared_ptr.as_mut_ptr().cast();
        let this = self as *const Self as *mut c_void;
        unsafe {
            T::__clone(this, new);
            shared_ptr.assume_init()
        }
    }
}

// SharedPtr is not a self-referential type and is safe to move out of a Pin,
// regardless whether the pointer's target is Unpin.
impl<T> Unpin for SharedPtr<T> where T: SharedPtrTarget {}

impl<T> Drop for SharedPtr<T>
where
    T: SharedPtrTarget,
{
    fn drop(&mut self) {
        let this = self as *mut Self as *mut c_void;
        unsafe { T::__drop(this) }
    }
}

impl<T> Deref for SharedPtr<T>
where
    T: SharedPtrTarget,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self.as_ref() {
            Some(target) => target,
            None => panic!(
                "called deref on a null SharedPtr<{}>",
                display(T::__typename),
            ),
        }
    }
}

impl<T> Debug for SharedPtr<T>
where
    T: Debug + SharedPtrTarget,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self.as_ref() {
            None => formatter.write_str("nullptr"),
            Some(value) => Debug::fmt(value, formatter),
        }
    }
}

impl<T> Display for SharedPtr<T>
where
    T: Display + SharedPtrTarget,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self.as_ref() {
            None => formatter.write_str("nullptr"),
            Some(value) => Display::fmt(value, formatter),
        }
    }
}

/// Trait bound for types which may be used as the `T` inside of a
/// `SharedPtr<T>` in generic code.
///
/// This trait has no publicly callable or implementable methods. Implementing
/// it outside of the CXX codebase is not supported.
///
/// # Example
///
/// A bound `T: SharedPtrTarget` may be necessary when manipulating
/// [`SharedPtr`] in generic code.
///
/// ```
/// use cxx::memory::{SharedPtr, SharedPtrTarget};
/// use std::fmt::Display;
///
/// pub fn take_generic_ptr<T>(ptr: SharedPtr<T>)
/// where
///     T: SharedPtrTarget + Display,
/// {
///     println!("the shared_ptr points to: {}", *ptr);
/// }
/// ```
///
/// Writing the same generic function without a `SharedPtrTarget` trait bound
/// would not compile.
pub unsafe trait SharedPtrTarget {
    #[doc(hidden)]
    fn __typename(f: &mut fmt::Formatter) -> fmt::Result;
    #[doc(hidden)]
    unsafe fn __null(new: *mut c_void);
    #[doc(hidden)]
    unsafe fn __new(value: Self, new: *mut c_void)
    where
        Self: Sized,
    {
        // Opoaque C types do not get this method because they can never exist
        // by value on the Rust side of the bridge.
        let _ = value;
        let _ = new;
        unreachable!()
    }
    #[doc(hidden)]
    unsafe fn __clone(this: *const c_void, new: *mut c_void);
    #[doc(hidden)]
    unsafe fn __get(this: *const c_void) -> *const Self;
    #[doc(hidden)]
    unsafe fn __drop(this: *mut c_void);
}

macro_rules! impl_shared_ptr_target {
    ($segment:expr, $name:expr, $ty:ty) => {
        unsafe impl SharedPtrTarget for $ty {
            fn __typename(f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str($name)
            }
            unsafe fn __null(new: *mut c_void) {
                extern "C" {
                    attr! {
                        #[link_name = concat!("cxxbridge1$std$shared_ptr$", $segment, "$null")]
                        fn __null(new: *mut c_void);
                    }
                }
                unsafe { __null(new) }
            }
            unsafe fn __new(value: Self, new: *mut c_void) {
                extern "C" {
                    attr! {
                        #[link_name = concat!("cxxbridge1$std$shared_ptr$", $segment, "$uninit")]
                        fn __uninit(new: *mut c_void) -> *mut c_void;
                    }
                }
                unsafe { __uninit(new).cast::<$ty>().write(value) }
            }
            unsafe fn __clone(this: *const c_void, new: *mut c_void) {
                extern "C" {
                    attr! {
                        #[link_name = concat!("cxxbridge1$std$shared_ptr$", $segment, "$clone")]
                        fn __clone(this: *const c_void, new: *mut c_void);
                    }
                }
                unsafe { __clone(this, new) }
            }
            unsafe fn __get(this: *const c_void) -> *const Self {
                extern "C" {
                    attr! {
                        #[link_name = concat!("cxxbridge1$std$shared_ptr$", $segment, "$get")]
                        fn __get(this: *const c_void) -> *const c_void;
                    }
                }
                unsafe { __get(this) }.cast()
            }
            unsafe fn __drop(this: *mut c_void) {
                extern "C" {
                    attr! {
                        #[link_name = concat!("cxxbridge1$std$shared_ptr$", $segment, "$drop")]
                        fn __drop(this: *mut c_void);
                    }
                }
                unsafe { __drop(this) }
            }
        }
    };
}

macro_rules! impl_shared_ptr_target_for_primitive {
    ($ty:ident) => {
        impl_shared_ptr_target!(stringify!($ty), stringify!($ty), $ty);
    };
}

impl_shared_ptr_target_for_primitive!(bool);
impl_shared_ptr_target_for_primitive!(u8);
impl_shared_ptr_target_for_primitive!(u16);
impl_shared_ptr_target_for_primitive!(u32);
impl_shared_ptr_target_for_primitive!(u64);
impl_shared_ptr_target_for_primitive!(usize);
impl_shared_ptr_target_for_primitive!(i8);
impl_shared_ptr_target_for_primitive!(i16);
impl_shared_ptr_target_for_primitive!(i32);
impl_shared_ptr_target_for_primitive!(i64);
impl_shared_ptr_target_for_primitive!(isize);
impl_shared_ptr_target_for_primitive!(f32);
impl_shared_ptr_target_for_primitive!(f64);

impl_shared_ptr_target!("string", "CxxString", CxxString);
