// No longer used. Old versions of serde used this macro for supporting targets
// that did not yet have 128-bit integer support.
#[macro_export]
#[doc(hidden)]
macro_rules! serde_if_integer128 {
    ($($tt:tt)*) => {
        $($tt)*
    };
}
