#[cfg(feature = "std")]
use std::prelude::v1::*;

use core::slice;
use core::str;

use crate::cast::JsObject;
use crate::convert::OptionIntoWasmAbi;
use crate::convert::{FromWasmAbi, IntoWasmAbi, RefFromWasmAbi, RefMutFromWasmAbi, WasmAbi};
use cfg_if::cfg_if;

if_std! {
    use core::mem;
    use crate::convert::OptionFromWasmAbi;
}

#[repr(C)]
pub struct WasmSlice {
    pub ptr: u32,
    pub len: u32,
}

unsafe impl WasmAbi for WasmSlice {}

#[inline]
fn null_slice() -> WasmSlice {
    WasmSlice { ptr: 0, len: 0 }
}

macro_rules! vectors {
    ($($t:ident)*) => ($(
        if_std! {
            impl IntoWasmAbi for Box<[$t]> {
                type Abi = WasmSlice;

                #[inline]
                fn into_abi(self) -> WasmSlice {
                    let ptr = self.as_ptr();
                    let len = self.len();
                    mem::forget(self);
                    WasmSlice {
                        ptr: ptr.into_abi(),
                        len: len as u32,
                    }
                }
            }

            impl OptionIntoWasmAbi for Box<[$t]> {
                #[inline]
                fn none() -> WasmSlice { null_slice() }
            }

            impl FromWasmAbi for Box<[$t]> {
                type Abi = WasmSlice;

                #[inline]
                unsafe fn from_abi(js: WasmSlice) -> Self {
                    let ptr = <*mut $t>::from_abi(js.ptr);
                    let len = js.len as usize;
                    Vec::from_raw_parts(ptr, len, len).into_boxed_slice()
                }
            }

            impl OptionFromWasmAbi for Box<[$t]> {
                #[inline]
                fn is_none(slice: &WasmSlice) -> bool { slice.ptr == 0 }
            }
        }

        impl<'a> IntoWasmAbi for &'a [$t] {
            type Abi = WasmSlice;

            #[inline]
            fn into_abi(self) -> WasmSlice {
                WasmSlice {
                    ptr: self.as_ptr().into_abi(),
                    len: self.len() as u32,
                }
            }
        }

        impl<'a> OptionIntoWasmAbi for &'a [$t] {
            #[inline]
            fn none() -> WasmSlice { null_slice() }
        }

        impl<'a> IntoWasmAbi for &'a mut [$t] {
            type Abi = WasmSlice;

            #[inline]
            fn into_abi(self) -> WasmSlice {
                (&*self).into_abi()
            }
        }

        impl<'a> OptionIntoWasmAbi for &'a mut [$t] {
            #[inline]
            fn none() -> WasmSlice { null_slice() }
        }

        impl RefFromWasmAbi for [$t] {
            type Abi = WasmSlice;
            type Anchor = Box<[$t]>;

            #[inline]
            unsafe fn ref_from_abi(js: WasmSlice) -> Box<[$t]> {
                <Box<[$t]>>::from_abi(js)
            }
        }

        impl RefMutFromWasmAbi for [$t] {
            type Abi = WasmSlice;
            type Anchor = &'static mut [$t];

            #[inline]
            unsafe fn ref_mut_from_abi(js: WasmSlice)
                -> &'static mut [$t]
            {
                slice::from_raw_parts_mut(
                    <*mut $t>::from_abi(js.ptr),
                    js.len as usize,
                )
            }
        }
    )*)
}

vectors! {
    u8 i8 u16 i16 u32 i32 u64 i64 usize isize f32 f64
}

cfg_if! {
    if #[cfg(feature = "enable-interning")] {
        #[inline]
        fn unsafe_get_cached_str(x: &str) -> Option<WasmSlice> {
            // This uses 0 for the ptr as an indication that it is a JsValue and not a str.
            crate::cache::intern::unsafe_get_str(x).map(|x| WasmSlice { ptr: 0, len: x })
        }

    } else {
        #[inline]
        fn unsafe_get_cached_str(_x: &str) -> Option<WasmSlice> {
            None
        }
    }
}

if_std! {
    impl<T> IntoWasmAbi for Vec<T> where Box<[T]>: IntoWasmAbi<Abi = WasmSlice> {
        type Abi = <Box<[T]> as IntoWasmAbi>::Abi;

        #[inline]
        fn into_abi(self) -> Self::Abi {
            self.into_boxed_slice().into_abi()
        }
    }

    impl<T> OptionIntoWasmAbi for Vec<T> where Box<[T]>: IntoWasmAbi<Abi = WasmSlice> {
        #[inline]
        fn none() -> WasmSlice { null_slice() }
    }

    impl<T> FromWasmAbi for Vec<T> where Box<[T]>: FromWasmAbi<Abi = WasmSlice> {
        type Abi = <Box<[T]> as FromWasmAbi>::Abi;

        #[inline]
        unsafe fn from_abi(js: Self::Abi) -> Self {
            <Box<[T]>>::from_abi(js).into()
        }
    }

    impl<T> OptionFromWasmAbi for Vec<T> where Box<[T]>: FromWasmAbi<Abi = WasmSlice> {
        #[inline]
        fn is_none(abi: &WasmSlice) -> bool { abi.ptr == 0 }
    }

    impl IntoWasmAbi for String {
        type Abi = <Vec<u8> as IntoWasmAbi>::Abi;

        #[inline]
        fn into_abi(self) -> Self::Abi {
            // This is safe because the JsValue is immediately looked up in the heap and
            // then returned, so use-after-free cannot occur.
            unsafe_get_cached_str(&self).unwrap_or_else(|| self.into_bytes().into_abi())
        }
    }

    impl OptionIntoWasmAbi for String {
        #[inline]
        fn none() -> Self::Abi { null_slice() }
    }

    impl FromWasmAbi for String {
        type Abi = <Vec<u8> as FromWasmAbi>::Abi;

        #[inline]
        unsafe fn from_abi(js: Self::Abi) -> Self {
            String::from_utf8_unchecked(<Vec<u8>>::from_abi(js))
        }
    }

    impl OptionFromWasmAbi for String {
        #[inline]
        fn is_none(slice: &WasmSlice) -> bool { slice.ptr == 0 }
    }
}

impl<'a> IntoWasmAbi for &'a str {
    type Abi = <&'a [u8] as IntoWasmAbi>::Abi;

    #[inline]
    fn into_abi(self) -> Self::Abi {
        // This is safe because the JsValue is immediately looked up in the heap and
        // then returned, so use-after-free cannot occur.
        unsafe_get_cached_str(self).unwrap_or_else(|| self.as_bytes().into_abi())
    }
}

impl<'a> OptionIntoWasmAbi for &'a str {
    #[inline]
    fn none() -> Self::Abi {
        null_slice()
    }
}

impl RefFromWasmAbi for str {
    type Abi = <[u8] as RefFromWasmAbi>::Abi;
    type Anchor = Box<str>;

    #[inline]
    unsafe fn ref_from_abi(js: Self::Abi) -> Self::Anchor {
        mem::transmute::<Box<[u8]>, Box<str>>(<Box<[u8]>>::from_abi(js))
    }
}

if_std! {
    use crate::JsValue;

    impl IntoWasmAbi for Box<[JsValue]> {
        type Abi = WasmSlice;

        #[inline]
        fn into_abi(self) -> WasmSlice {
            let ptr = self.as_ptr();
            let len = self.len();
            mem::forget(self);
            WasmSlice {
                ptr: ptr.into_abi(),
                len: len as u32,
            }
        }
    }

    impl OptionIntoWasmAbi for Box<[JsValue]> {
        #[inline]
        fn none() -> WasmSlice { null_slice() }
    }

    impl FromWasmAbi for Box<[JsValue]> {
        type Abi = WasmSlice;

        #[inline]
        unsafe fn from_abi(js: WasmSlice) -> Self {
            let ptr = <*mut JsValue>::from_abi(js.ptr);
            let len = js.len as usize;
            Vec::from_raw_parts(ptr, len, len).into_boxed_slice()
        }
    }

    impl OptionFromWasmAbi for Box<[JsValue]> {
        #[inline]
        fn is_none(slice: &WasmSlice) -> bool { slice.ptr == 0 }
    }

    impl<T> IntoWasmAbi for Box<[T]> where T: JsObject {
        type Abi = WasmSlice;

        #[inline]
        fn into_abi(self) -> WasmSlice {
            let ptr = self.as_ptr();
            let len = self.len();
            mem::forget(self);
            WasmSlice {
                ptr: ptr.into_abi(),
                len: len as u32,
            }
        }
    }

    impl<T> OptionIntoWasmAbi for Box<[T]> where T: JsObject {
        #[inline]
        fn none() -> WasmSlice { null_slice() }
    }

    impl<T> FromWasmAbi for Box<[T]> where T: JsObject {
        type Abi = WasmSlice;

        #[inline]
        unsafe fn from_abi(js: WasmSlice) -> Self {
            let ptr = <*mut JsValue>::from_abi(js.ptr);
            let len = js.len as usize;
            let vec: Vec<T> = Vec::from_raw_parts(ptr, len, len).drain(..).map(|js_value| T::unchecked_from_js(js_value)).collect();
            return vec.into_boxed_slice();
        }
    }

    impl<T> OptionFromWasmAbi for Box<[T]> where T: JsObject {
        #[inline]
        fn is_none(slice: &WasmSlice) -> bool { slice.ptr == 0 }
    }
}
