#![allow(clippy::blacklisted_name)]

use crate::{body::HttpBody, BoxError};

mod test_client;
pub(crate) use self::test_client::*;

pub(crate) fn assert_send<T: Send>() {}
pub(crate) fn assert_sync<T: Sync>() {}
pub(crate) fn assert_unpin<T: Unpin>() {}

pub(crate) struct NotSendSync(*const ());
