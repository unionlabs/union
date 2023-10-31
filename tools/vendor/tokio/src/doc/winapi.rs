//! See [winapi].
//!
//! [winapi]: https://docs.rs/winapi

/// See [winapi::shared](https://docs.rs/winapi/*/winapi/shared/index.html).
pub mod shared {
    /// See [winapi::shared::winerror](https://docs.rs/winapi/*/winapi/shared/winerror/index.html).
    #[allow(non_camel_case_types)]
    pub mod winerror {
        /// See [winapi::shared::winerror::ERROR_ACCESS_DENIED][winapi]
        ///
        /// [winapi]: https://docs.rs/winapi/*/winapi/shared/winerror/constant.ERROR_ACCESS_DENIED.html
        pub type ERROR_ACCESS_DENIED = crate::doc::NotDefinedHere;

        /// See [winapi::shared::winerror::ERROR_PIPE_BUSY][winapi]
        ///
        /// [winapi]: https://docs.rs/winapi/*/winapi/shared/winerror/constant.ERROR_PIPE_BUSY.html
        pub type ERROR_PIPE_BUSY = crate::doc::NotDefinedHere;

        /// See [winapi::shared::winerror::ERROR_MORE_DATA][winapi]
        ///
        /// [winapi]: https://docs.rs/winapi/*/winapi/shared/winerror/constant.ERROR_MORE_DATA.html
        pub type ERROR_MORE_DATA = crate::doc::NotDefinedHere;
    }
}

/// See [winapi::um](https://docs.rs/winapi/*/winapi/um/index.html).
pub mod um {
    /// See [winapi::um::winbase](https://docs.rs/winapi/*/winapi/um/winbase/index.html).
    #[allow(non_camel_case_types)]
    pub mod winbase {
        /// See [winapi::um::winbase::PIPE_TYPE_MESSAGE][winapi]
        ///
        /// [winapi]: https://docs.rs/winapi/*/winapi/um/winbase/constant.PIPE_TYPE_MESSAGE.html
        pub type PIPE_TYPE_MESSAGE = crate::doc::NotDefinedHere;

        /// See [winapi::um::winbase::PIPE_TYPE_BYTE][winapi]
        ///
        /// [winapi]: https://docs.rs/winapi/*/winapi/um/winbase/constant.PIPE_TYPE_BYTE.html
        pub type PIPE_TYPE_BYTE = crate::doc::NotDefinedHere;

        /// See [winapi::um::winbase::PIPE_CLIENT_END][winapi]
        ///
        /// [winapi]: https://docs.rs/winapi/*/winapi/um/winbase/constant.PIPE_CLIENT_END.html
        pub type PIPE_CLIENT_END = crate::doc::NotDefinedHere;

        /// See [winapi::um::winbase::PIPE_SERVER_END][winapi]
        ///
        /// [winapi]: https://docs.rs/winapi/*/winapi/um/winbase/constant.PIPE_SERVER_END.html
        pub type PIPE_SERVER_END = crate::doc::NotDefinedHere;

        /// See [winapi::um::winbase::SECURITY_IDENTIFICATION][winapi]
        ///
        /// [winapi]: https://docs.rs/winapi/*/winapi/um/winbase/constant.SECURITY_IDENTIFICATION.html
        pub type SECURITY_IDENTIFICATION = crate::doc::NotDefinedHere;
    }

    /// See [winapi::um::minwinbase](https://docs.rs/winapi/*/winapi/um/minwinbase/index.html).
    #[allow(non_camel_case_types)]
    pub mod minwinbase {
        /// See [winapi::um::minwinbase::SECURITY_ATTRIBUTES][winapi]
        ///
        /// [winapi]: https://docs.rs/winapi/*/winapi/um/minwinbase/constant.SECURITY_ATTRIBUTES.html
        pub type SECURITY_ATTRIBUTES = crate::doc::NotDefinedHere;
    }
}
