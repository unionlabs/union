#![cfg_attr(
    thiserror_nightly_testing,
    feature(error_generic_member_access, provide_any)
)]

#[cfg(thiserror_nightly_testing)]
pub mod structs {
    use std::backtrace::Backtrace;
    use thiserror::Error;

    #[derive(Error, Debug)]
    #[error("...")]
    pub struct OptSourceNoBacktrace {
        #[source]
        source: Option<anyhow::Error>,
    }

    #[derive(Error, Debug)]
    #[error("...")]
    pub struct OptSourceAlwaysBacktrace {
        #[source]
        source: Option<anyhow::Error>,
        backtrace: Backtrace,
    }

    #[derive(Error, Debug)]
    #[error("...")]
    pub struct NoSourceOptBacktrace {
        #[backtrace]
        backtrace: Option<Backtrace>,
    }

    #[derive(Error, Debug)]
    #[error("...")]
    pub struct AlwaysSourceOptBacktrace {
        source: anyhow::Error,
        #[backtrace]
        backtrace: Option<Backtrace>,
    }

    #[derive(Error, Debug)]
    #[error("...")]
    pub struct OptSourceOptBacktrace {
        #[source]
        source: Option<anyhow::Error>,
        #[backtrace]
        backtrace: Option<Backtrace>,
    }
}

#[cfg(thiserror_nightly_testing)]
pub mod enums {
    use std::backtrace::Backtrace;
    use thiserror::Error;

    #[derive(Error, Debug)]
    pub enum OptSourceNoBacktrace {
        #[error("...")]
        Test {
            #[source]
            source: Option<anyhow::Error>,
        },
    }

    #[derive(Error, Debug)]
    pub enum OptSourceAlwaysBacktrace {
        #[error("...")]
        Test {
            #[source]
            source: Option<anyhow::Error>,
            backtrace: Backtrace,
        },
    }

    #[derive(Error, Debug)]
    pub enum NoSourceOptBacktrace {
        #[error("...")]
        Test {
            #[backtrace]
            backtrace: Option<Backtrace>,
        },
    }

    #[derive(Error, Debug)]
    pub enum AlwaysSourceOptBacktrace {
        #[error("...")]
        Test {
            source: anyhow::Error,
            #[backtrace]
            backtrace: Option<Backtrace>,
        },
    }

    #[derive(Error, Debug)]
    pub enum OptSourceOptBacktrace {
        #[error("...")]
        Test {
            #[source]
            source: Option<anyhow::Error>,
            #[backtrace]
            backtrace: Option<Backtrace>,
        },
    }
}

#[test]
#[cfg_attr(not(thiserror_nightly_testing), ignore)]
fn test_option() {}
