//! ## Documentation: Feature Flags
//!
//! Available [compile-time feature flags](https://doc.rust-lang.org/cargo/reference/features.html#dependency-features)
//!
//! #### Default Features
//!
//! * **std**: _Not Currently Used._ Placeholder for supporting `no_std` environments in a backwards compatible manner.
//! * **color**: Turns on colored error messages.
//! * **suggestions**: Turns on the `Did you mean '--myoption'?` feature for when users make typos.
//!
//! #### Optional features
//!
//! * **deprecated**: Guided experience to prepare for next breaking release (at different stages of development, this may become default)
//! * **derive**: Enables the custom derive (i.e. `#[derive(Parser)]`). Without this you must use one of the other methods of creating a `clap` CLI listed above.
//! * **cargo**: Turns on macros that read values from [`CARGO_*` environment variables](https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-crates).
//! * **env**: Turns on the usage of environment variables during parsing.
//! * **regex**: Enables regex validators.
//! * **unicode**: Turns on support for unicode characters (including emoji) in arguments and help messages.
//! * **wrap_help**: Turns on the help text wrapping feature, based on the terminal size.
//!
//! #### Experimental features
//!
//! **Warning:** These may contain breaking changes between minor releases.
//!
//! * **unstable-replace**: Enable [`Command::replace`](https://github.com/clap-rs/clap/issues/2836)
//! * **unstable-grouped**: Enable [`ArgMatches::grouped_values_of`](https://github.com/clap-rs/clap/issues/2924)
//! * **unstable-v4**: Preview features which will be stable on the v4.0 release
