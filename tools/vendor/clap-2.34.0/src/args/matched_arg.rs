// Std
use std::ffi::OsString;

#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct MatchedArg {
    #[doc(hidden)]
    pub occurs: u64,
    #[doc(hidden)]
    pub indices: Vec<usize>,
    #[doc(hidden)]
    pub vals: Vec<OsString>,
}

impl Default for MatchedArg {
    fn default() -> Self {
        MatchedArg {
            occurs: 1,
            indices: Vec::new(),
            vals: Vec::new(),
        }
    }
}

impl MatchedArg {
    pub fn new() -> Self {
        MatchedArg::default()
    }
}
