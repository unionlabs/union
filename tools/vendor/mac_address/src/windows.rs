use core::convert::TryInto;
use std::{convert::TryFrom, ffi::OsString, os::windows::ffi::OsStringExt, ptr, slice};
use winapi::shared::{ntdef::ULONG, winerror::ERROR_SUCCESS, ws2def::AF_UNSPEC};
use winapi::um::{iphlpapi::GetAdaptersAddresses, iptypes::IP_ADAPTER_ADDRESSES_LH};

use crate::MacAddressError;

const GAA_FLAG_NONE: ULONG = 0x0000;

/// Uses bindings to the `Iphlpapi.h` Windows header to fetch the interface
/// devices list with
/// [GetAdaptersAddresses][https://msdn.microsoft.com/en-us/library/windows/desktop/aa365915(v=vs.85).aspx]
/// then loops over the returned list until it finds a network device with a MAC
/// address, and returns it.
///
/// If it fails to find a device, it returns a `NoDevicesFound` error.
pub fn get_mac(name: Option<&str>) -> Result<Option<[u8; 6]>, MacAddressError> {
    let adapters = get_adapters()?;
    // Safety: We don't use the pointer after `adapters` is dropped
    let mut ptr = unsafe { adapters.ptr() };

    loop {
        // Break if we've gone through all devices
        if ptr.is_null() {
            break;
        }

        let bytes = unsafe { convert_mac_bytes(ptr) };

        if let Some(name) = name {
            #[cfg(not(target_pointer_width = "32"))]
            let adapter_name = unsafe { construct_string((*ptr).FriendlyName) };

            #[cfg(target_pointer_width = "32")]
            let adapter_name = unsafe { construct_string(ptr.read_unaligned().FriendlyName) };

            if adapter_name == name {
                return Ok(Some(bytes));
            }
        } else if bytes.iter().any(|&x| x != 0) {
            return Ok(Some(bytes));
        }

        // Otherwise go to the next device
        #[cfg(target_pointer_width = "32")]
        {
            ptr = unsafe { ptr.read_unaligned().Next };
        }

        #[cfg(not(target_pointer_width = "32"))]
        {
            ptr = unsafe { (*ptr).Next };
        }
    }

    Ok(None)
}

pub fn get_ifname(mac: &[u8; 6]) -> Result<Option<String>, MacAddressError> {
    let adapters = get_adapters()?;

    // Safety: We don't use the pointer after `adapters` is dropped
    let mut ptr = unsafe { adapters.ptr() };

    loop {
        // Break if we've gone through all devices
        if ptr.is_null() {
            break;
        }

        let bytes = unsafe { convert_mac_bytes(ptr) };

        if &bytes == mac {
            #[cfg(not(target_pointer_width = "32"))]
            let adapter_name = unsafe { construct_string((*ptr).FriendlyName) };

            #[cfg(target_pointer_width = "32")]
            let adapter_name = unsafe { construct_string(ptr.read_unaligned().FriendlyName) };

            let adapter_name = adapter_name
                .into_string()
                .map_err(|_| MacAddressError::InternalError)?;
            return Ok(Some(adapter_name));
        }

        // Otherwise go to the next device
        #[cfg(target_pointer_width = "32")]
        {
            ptr = unsafe { ptr.read_unaligned().Next };
        }

        #[cfg(not(target_pointer_width = "32"))]
        {
            ptr = unsafe { (*ptr).Next };
        }
    }

    Ok(None)
}

/// Copy over the 6 MAC address bytes to the buffer.
pub(crate) unsafe fn convert_mac_bytes(ptr: *mut IP_ADAPTER_ADDRESSES_LH) -> [u8; 6] {
    #[cfg(target_pointer_width = "32")]
    return ptr.read_unaligned().PhysicalAddress[..6]
        .try_into()
        .unwrap();

    #[cfg(not(target_pointer_width = "32"))]
    return ((*ptr).PhysicalAddress)[..6].try_into().unwrap();
}

pub(crate) struct AdaptersList {
    ptr: *mut IP_ADAPTER_ADDRESSES_LH,
    size: usize,
}

impl AdaptersList {
    /// Safety: The pointer returned by this method MUST NOT be used after
    /// `self` has gone out of scope. This pointer may also be null.
    pub(crate) unsafe fn ptr(&self) -> *mut IP_ADAPTER_ADDRESSES_LH {
        self.ptr
    }
}

impl Drop for AdaptersList {
    fn drop(&mut self) {
        if !self.ptr.is_null() && self.size != 0 {
            unsafe {
                std::alloc::dealloc(
                    self.ptr as *mut u8,
                    std::alloc::Layout::from_size_align(
                        self.size,
                        core::mem::align_of::<IP_ADAPTER_ADDRESSES_LH>(),
                    )
                    .unwrap(),
                )
            };
        }
    }
}

pub(crate) fn get_adapters() -> Result<AdaptersList, MacAddressError> {
    let mut buf_len = 0;

    // This will get the number of bytes we need to allocate for all devices
    unsafe {
        GetAdaptersAddresses(
            AF_UNSPEC as u32,
            GAA_FLAG_NONE,
            ptr::null_mut(),
            ptr::null_mut(),
            &mut buf_len,
        );
    }

    if buf_len == 0 {
        return Ok(AdaptersList {
            ptr: ptr::null_mut(),
            size: 0,
        });
    }

    // Allocate `buf_len` bytes, and create a raw pointer to it with the correct alignment
    // Safety:
    let adapters_list: *mut IP_ADAPTER_ADDRESSES_LH = unsafe {
        std::alloc::alloc(
            std::alloc::Layout::from_size_align(
                usize::try_from(buf_len).map_err(|_| MacAddressError::InternalError)?,
                core::mem::align_of::<IP_ADAPTER_ADDRESSES_LH>(),
            )
            .unwrap(),
        )
    } as *mut IP_ADAPTER_ADDRESSES_LH;

    // Get our list of adapters
    let result = unsafe {
        GetAdaptersAddresses(
            // [IN] Family
            AF_UNSPEC as u32,
            // [IN] Flags
            GAA_FLAG_NONE,
            // [IN] Reserved
            ptr::null_mut(),
            // [INOUT] AdapterAddresses
            adapters_list,
            // [INOUT] SizePointer
            &mut buf_len,
        )
    };

    let adapters_list = AdaptersList {
        ptr: adapters_list,
        // Cast OK, we checked it above
        size: buf_len as usize,
    };

    // Make sure we were successful
    if result != ERROR_SUCCESS {
        return Err(MacAddressError::InternalError);
    }

    Ok(adapters_list)
}

unsafe fn construct_string(ptr: *mut u16) -> OsString {
    let slice = slice::from_raw_parts(ptr, get_null_position(ptr));
    OsStringExt::from_wide(slice)
}

unsafe fn get_null_position(ptr: *mut u16) -> usize {
    assert!(!ptr.is_null());

    for i in 0.. {
        if *ptr.offset(i) == 0 {
            return i as usize;
        }
    }

    unreachable!()
}
