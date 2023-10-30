// Copyright 2013-2015 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, improper_ctypes)]

#![cfg_attr(all(feature="mac_os_10_7_support", feature="mac_os_10_8_features"), feature(linkage))] // back-compat requires weak linkage

pub mod array;
pub mod attributed_string;
pub mod base;
pub mod bundle;
pub mod characterset;
pub mod data;
pub mod date;
pub mod dictionary;
pub mod error;
pub mod filedescriptor;
pub mod messageport;
pub mod number;
pub mod propertylist;
pub mod runloop;
pub mod set;
pub mod string;
pub mod timezone;
pub mod url;
pub mod uuid;
pub mod mach_port;
