// Copyright 2021 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Implementation for ESP-IDF
use crate::Error;
use core::ffi::c_void;

extern "C" {
    fn esp_fill_random(buf: *mut c_void, len: usize) -> u32;
}

pub fn getrandom_inner(dest: &mut [u8]) -> Result<(), Error> {
    // Not that NOT enabling WiFi, BT, or the voltage noise entropy source (via `bootloader_random_enable`)
    // will cause ESP-IDF to return pseudo-random numbers based on the voltage noise entropy, after the initial boot process:
    // https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-reference/system/random.html
    //
    // However tracking if some of these entropy sources is enabled is way too difficult to implement here
    unsafe { esp_fill_random(dest.as_mut_ptr().cast(), dest.len()) };

    Ok(())
}
