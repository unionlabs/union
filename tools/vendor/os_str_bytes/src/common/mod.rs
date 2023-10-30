use std::borrow::Cow;
use std::convert::Infallible;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::result;

#[cfg(all(target_vendor = "fortanix", target_env = "sgx"))]
use std::os::fortanix_sgx as os;
#[cfg(target_os = "hermit")]
use std::os::hermit as os;
#[cfg(target_os = "solid_asp3")]
use std::os::solid as os;
#[cfg(unix)]
use std::os::unix as os;
#[cfg(target_os = "wasi")]
use std::os::wasi as os;
#[cfg(target_os = "xous")]
use std::os::xous as os;

use os::ffi::OsStrExt;
use os::ffi::OsStringExt;

if_raw_str! {
    pub(super) mod raw;
}

pub(super) type EncodingError = Infallible;

type Result<T> = result::Result<T, EncodingError>;

pub(super) fn os_str_from_bytes(string: &[u8]) -> Result<Cow<'_, OsStr>> {
    Ok(Cow::Borrowed(OsStrExt::from_bytes(string)))
}

pub(super) fn os_str_to_bytes(os_string: &OsStr) -> Cow<'_, [u8]> {
    Cow::Borrowed(OsStrExt::as_bytes(os_string))
}

pub(super) fn os_string_from_vec(string: Vec<u8>) -> Result<OsString> {
    Ok(OsStringExt::from_vec(string))
}

pub(super) fn os_string_into_vec(os_string: OsString) -> Vec<u8> {
    OsStringExt::into_vec(os_string)
}
