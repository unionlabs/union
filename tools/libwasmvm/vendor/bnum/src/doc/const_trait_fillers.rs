macro_rules! impl_desc {
    () => {
        "The latest version of the nightly Rust compiler at the time `v0.7.0` was published (`v1.71.0-nightly`) unfortunately dropped support for the `const` implementation of common standard library traits such as `Add`, `BitOr`, etc. These methods below have therefore been provided to allow use of some of the methods these traits provided, in a const context."
    };
}

pub(crate) use impl_desc;
