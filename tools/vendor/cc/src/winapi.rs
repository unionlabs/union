// Copyright © 2015-2017 winapi-rs developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your option.
// All files in the project carrying such notice may not be copied, modified, or distributed
// except according to those terms.

#![allow(bad_style)]

use std::os::raw;

pub type wchar_t = u16;

pub type UINT = raw::c_uint;
pub type LPUNKNOWN = *mut IUnknown;
pub type REFIID = *const IID;
pub type IID = GUID;
pub type REFCLSID = *const IID;
pub type PVOID = *mut raw::c_void;
pub type USHORT = raw::c_ushort;
pub type ULONG = raw::c_ulong;
pub type LONG = raw::c_long;
pub type DWORD = u32;
pub type LPVOID = *mut raw::c_void;
pub type HRESULT = raw::c_long;
pub type LPFILETIME = *mut FILETIME;
pub type BSTR = *mut OLECHAR;
pub type OLECHAR = WCHAR;
pub type WCHAR = wchar_t;
pub type LPCOLESTR = *const OLECHAR;
pub type LCID = DWORD;
pub type LPCWSTR = *const WCHAR;
pub type PULONGLONG = *mut ULONGLONG;
pub type ULONGLONG = u64;

pub const S_OK: HRESULT = 0;
pub const S_FALSE: HRESULT = 1;
pub const COINIT_MULTITHREADED: u32 = 0x0;

pub type CLSCTX = u32;

pub const CLSCTX_INPROC_SERVER: CLSCTX = 0x1;
pub const CLSCTX_INPROC_HANDLER: CLSCTX = 0x2;
pub const CLSCTX_LOCAL_SERVER: CLSCTX = 0x4;
pub const CLSCTX_REMOTE_SERVER: CLSCTX = 0x10;

pub const CLSCTX_ALL: CLSCTX =
    CLSCTX_INPROC_SERVER | CLSCTX_INPROC_HANDLER | CLSCTX_LOCAL_SERVER | CLSCTX_REMOTE_SERVER;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct GUID {
    pub Data1: raw::c_ulong,
    pub Data2: raw::c_ushort,
    pub Data3: raw::c_ushort,
    pub Data4: [raw::c_uchar; 8],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct FILETIME {
    pub dwLowDateTime: DWORD,
    pub dwHighDateTime: DWORD,
}

pub trait Interface {
    fn uuidof() -> GUID;
}

#[link(name = "ole32")]
#[link(name = "oleaut32")]
extern "C" {}

extern "system" {
    pub fn CoInitializeEx(pvReserved: LPVOID, dwCoInit: DWORD) -> HRESULT;
    pub fn CoCreateInstance(
        rclsid: REFCLSID,
        pUnkOuter: LPUNKNOWN,
        dwClsContext: DWORD,
        riid: REFIID,
        ppv: *mut LPVOID,
    ) -> HRESULT;
    pub fn SysFreeString(bstrString: BSTR);
    pub fn SysStringLen(pbstr: BSTR) -> UINT;
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct SAFEARRAYBOUND {
    pub cElements: ULONG,
    pub lLbound: LONG,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct SAFEARRAY {
    pub cDims: USHORT,
    pub fFeatures: USHORT,
    pub cbElements: ULONG,
    pub cLocks: ULONG,
    pub pvData: PVOID,
    pub rgsabound: [SAFEARRAYBOUND; 1],
}

pub type LPSAFEARRAY = *mut SAFEARRAY;

macro_rules! DEFINE_GUID {
    (
        $name:ident, $l:expr, $w1:expr, $w2:expr,
        $b1:expr, $b2:expr, $b3:expr, $b4:expr, $b5:expr, $b6:expr, $b7:expr, $b8:expr
    ) => {
        pub const $name: $crate::winapi::GUID = $crate::winapi::GUID {
            Data1: $l,
            Data2: $w1,
            Data3: $w2,
            Data4: [$b1, $b2, $b3, $b4, $b5, $b6, $b7, $b8],
        };
    };
}

macro_rules! RIDL {
    (#[uuid($($uuid:expr),+)]
    interface $interface:ident ($vtbl:ident) {$(
        fn $method:ident($($p:ident : $t:ty,)*) -> $rtr:ty,
    )+}) => (
        #[repr(C)]
        pub struct $vtbl {
            $(pub $method: unsafe extern "system" fn(
                This: *mut $interface,
                $($p: $t),*
            ) -> $rtr,)+
        }
        #[repr(C)]
        pub struct $interface {
            pub lpVtbl: *const $vtbl,
        }
        RIDL!{@impl $interface {$(fn $method($($p: $t,)*) -> $rtr,)+}}
        RIDL!{@uuid $interface $($uuid),+}
    );
    (#[uuid($($uuid:expr),+)]
    interface $interface:ident ($vtbl:ident) : $pinterface:ident ($pvtbl:ident) {
    }) => (
        #[repr(C)]
        pub struct $vtbl {
            pub parent: $pvtbl,
        }
        #[repr(C)]
        pub struct $interface {
            pub lpVtbl: *const $vtbl,
        }
        RIDL!{@deref $interface $pinterface}
        RIDL!{@uuid $interface $($uuid),+}
    );
    (#[uuid($($uuid:expr),+)]
    interface $interface:ident ($vtbl:ident) : $pinterface:ident ($pvtbl:ident) {$(
        fn $method:ident($($p:ident : $t:ty,)*) -> $rtr:ty,
    )+}) => (
        #[repr(C)]
        pub struct $vtbl {
            pub parent: $pvtbl,
            $(pub $method: unsafe extern "system" fn(
                This: *mut $interface,
                $($p: $t,)*
            ) -> $rtr,)+
        }
        #[repr(C)]
        pub struct $interface {
            pub lpVtbl: *const $vtbl,
        }
        RIDL!{@impl $interface {$(fn $method($($p: $t,)*) -> $rtr,)+}}
        RIDL!{@deref $interface $pinterface}
        RIDL!{@uuid $interface $($uuid),+}
    );
    (@deref $interface:ident $pinterface:ident) => (
        impl ::std::ops::Deref for $interface {
            type Target = $pinterface;
            #[inline]
            fn deref(&self) -> &$pinterface {
                unsafe { &*(self as *const $interface as *const $pinterface) }
            }
        }
    );
    (@impl $interface:ident {$(
        fn $method:ident($($p:ident : $t:ty,)*) -> $rtr:ty,
    )+}) => (
        impl $interface {
            $(#[inline] pub unsafe fn $method(&self, $($p: $t,)*) -> $rtr {
                ((*self.lpVtbl).$method)(self as *const _ as *mut _, $($p,)*)
            })+
        }
    );
    (@uuid $interface:ident
        $l:expr, $w1:expr, $w2:expr,
        $b1:expr, $b2:expr, $b3:expr, $b4:expr, $b5:expr, $b6:expr, $b7:expr, $b8:expr
    ) => (
        impl $crate::winapi::Interface for $interface {
            #[inline]
            fn uuidof() -> $crate::winapi::GUID {
                $crate::winapi::GUID {
                    Data1: $l,
                    Data2: $w1,
                    Data3: $w2,
                    Data4: [$b1, $b2, $b3, $b4, $b5, $b6, $b7, $b8],
                }
            }
        }
    );
}

RIDL! {#[uuid(0x00000000, 0x0000, 0x0000, 0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46)]
interface IUnknown(IUnknownVtbl) {
    fn QueryInterface(
        riid: REFIID,
        ppvObject: *mut *mut raw::c_void,
    ) -> HRESULT,
    fn AddRef() -> ULONG,
    fn Release() -> ULONG,
}}
