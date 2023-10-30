#[macro_export]
#[doc(hidden)]
macro_rules! attr {
    (#[$name:ident = $value:expr] $($rest:tt)*) => {
        #[$name = $value]
        $($rest)*
    };
}
