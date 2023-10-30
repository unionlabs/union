// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
#![cfg_attr(docsrs, doc(cfg(feature = "std")))]
extern crate std;

use crate::Error;
use core::convert::From;
use std::io;

impl From<Error> for io::Error {
    fn from(err: Error) -> Self {
        match err.raw_os_error() {
            Some(errno) => io::Error::from_raw_os_error(errno),
            None => io::Error::new(io::ErrorKind::Other, err),
        }
    }
}

impl std::error::Error for Error {}
