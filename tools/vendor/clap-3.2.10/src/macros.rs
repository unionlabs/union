/// Deprecated in [Issue #3087](https://github.com/clap-rs/clap/issues/3087), maybe [`clap::Parser`][crate::Parser] would fit your use case?
#[cfg(feature = "yaml")]
#[cfg_attr(
    feature = "deprecated",
    deprecated(
        since = "3.0.0",
        note = "Deprecated in Issue #3087, maybe clap::Parser would fit your use case?"
    )
)]
#[doc(hidden)]
#[macro_export]
macro_rules! load_yaml {
    ($yaml:expr) => {
        &$crate::YamlLoader::load_from_str(include_str!($yaml)).expect("failed to load YAML file")
            [0]
    };
}

/// Deprecated, replaced with [`ArgMatches::value_of_t`][crate::ArgMatches::value_of_t]
#[macro_export]
#[cfg_attr(
    feature = "deprecated",
    deprecated(since = "3.0.0", note = "Replaced with `ArgMatches::get_one`")
)]
#[doc(hidden)]
macro_rules! value_t {
    ($m:ident, $v:expr, $t:ty) => {
        $crate::value_t!($m.value_of($v), $t)
    };
    ($m:ident.value_of($v:expr), $t:ty) => {
        $m.value_of_t::<$t>($v)
    };
}

/// Deprecated, replaced with [`ArgMatches::value_of_t_or_exit`][crate::ArgMatches::value_of_t_or_exit]
#[macro_export]
#[cfg_attr(
    feature = "deprecated",
    deprecated(since = "3.0.0", note = "Replaced with `ArgMatches::get_one`")
)]
#[doc(hidden)]
macro_rules! value_t_or_exit {
    ($m:ident, $v:expr, $t:ty) => {
        value_t_or_exit!($m.value_of($v), $t)
    };
    ($m:ident.value_of($v:expr), $t:ty) => {
        $m.value_of_t_or_exit::<$t>($v)
    };
}

/// Deprecated, replaced with [`ArgMatches::values_of_t`][crate::ArgMatches::value_of_t]
#[macro_export]
#[cfg_attr(
    feature = "deprecated",
    deprecated(since = "3.0.0", note = "Replaced with `ArgMatches::get_many`")
)]
#[doc(hidden)]
macro_rules! values_t {
    ($m:ident, $v:expr, $t:ty) => {
        values_t!($m.values_of($v), $t)
    };
    ($m:ident.values_of($v:expr), $t:ty) => {
        $m.values_of_t::<$t>($v)
    };
}

/// Deprecated, replaced with [`ArgMatches::values_of_t_or_exit`][crate::ArgMatches::value_of_t_or_exit]
#[macro_export]
#[cfg_attr(
    feature = "deprecated",
    deprecated(since = "3.0.0", note = "Replaced with `ArgMatches::get_many`")
)]
#[doc(hidden)]
macro_rules! values_t_or_exit {
    ($m:ident, $v:expr, $t:ty) => {
        values_t_or_exit!($m.values_of($v), $t)
    };
    ($m:ident.values_of($v:expr), $t:ty) => {
        $m.values_of_t_or_exit::<$t>($v)
    };
}

#[cfg_attr(
    feature = "deprecated",
    deprecated(since = "3.0.0", note = "Replaced with `ValueEnum`")
)]
#[doc(hidden)]
#[macro_export]
macro_rules! _clap_count_exprs {
    () => { 0 };
    ($e:expr) => { 1 };
    ($e:expr, $($es:expr),+) => { 1 + $crate::_clap_count_exprs!($($es),*) };
}

/// Deprecated, replaced with [`ArgEnum`][crate::ArgEnum]
#[cfg_attr(
    feature = "deprecated",
    deprecated(since = "3.0.0", note = "Replaced with `ValueEnum`")
)]
#[doc(hidden)]
#[macro_export]
macro_rules! arg_enum {
    (@as_item $($i:item)*) => ($($i)*);
    (@impls ( $($tts:tt)* ) -> ($e:ident, $($v:ident),+)) => {
        $crate::arg_enum!(@as_item
        $($tts)*

        impl ::std::str::FromStr for $e {
            type Err = String;

            fn from_str(s: &str) -> ::std::result::Result<Self,Self::Err> {
                #[allow(deprecated, unused_imports)]
                use ::std::ascii::AsciiExt;
                match s {
                    $(stringify!($v) |
                    _ if s.eq_ignore_ascii_case(stringify!($v)) => Ok($e::$v)),+,
                    _ => Err({
                        let v = vec![
                            $(stringify!($v),)+
                        ];
                        format!("valid values: {}",
                            v.join(", "))
                    }),
                }
            }
        }
        impl ::std::fmt::Display for $e {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                match *self {
                    $($e::$v => write!(f, stringify!($v)),)+
                }
            }
        }
        impl $e {
            #[allow(dead_code)]
            pub fn variants() -> [&'static str; $crate::_clap_count_exprs!($(stringify!($v)),+)] {
                [
                    $(stringify!($v),)+
                ]
            }
        });
    };
    ($(#[$($m:meta),+])+ pub enum $e:ident { $($v:ident $(=$val:expr)*,)+ } ) => {
        $crate::arg_enum!(@impls
            ($(#[$($m),+])+
            pub enum $e {
                $($v$(=$val)*),+
            }) -> ($e, $($v),+)
        );
    };
    ($(#[$($m:meta),+])+ pub enum $e:ident { $($v:ident $(=$val:expr)*),+ } ) => {
        $crate::arg_enum!(@impls
            ($(#[$($m),+])+
            pub enum $e {
                $($v$(=$val)*),+
            }) -> ($e, $($v),+)
        );
    };
    ($(#[$($m:meta),+])+ enum $e:ident { $($v:ident $(=$val:expr)*,)+ } ) => {
        $crate::arg_enum!(@impls
            ($(#[$($m),+])+
             enum $e {
                 $($v$(=$val)*),+
             }) -> ($e, $($v),+)
        );
    };
    ($(#[$($m:meta),+])+ enum $e:ident { $($v:ident $(=$val:expr)*),+ } ) => {
        $crate::arg_enum!(@impls
            ($(#[$($m),+])+
            enum $e {
                $($v$(=$val)*),+
            }) -> ($e, $($v),+)
        );
    };
    (pub enum $e:ident { $($v:ident $(=$val:expr)*,)+ } ) => {
        $crate::arg_enum!(@impls
            (pub enum $e {
                $($v$(=$val)*),+
            }) -> ($e, $($v),+)
        );
    };
    (pub enum $e:ident { $($v:ident $(=$val:expr)*),+ } ) => {
        $crate::arg_enum!(@impls
            (pub enum $e {
                $($v$(=$val)*),+
            }) -> ($e, $($v),+)
        );
    };
    (enum $e:ident { $($v:ident $(=$val:expr)*,)+ } ) => {
        $crate::arg_enum!(@impls
            (enum $e {
                $($v$(=$val)*),+
            }) -> ($e, $($v),+)
        );
    };
    (enum $e:ident { $($v:ident $(=$val:expr)*),+ } ) => {
        $crate::arg_enum!(@impls
            (enum $e {
                $($v$(=$val)*),+
            }) -> ($e, $($v),+)
        );
    };
}

/// Allows you to pull the version from your Cargo.toml at compile time as
/// `MAJOR.MINOR.PATCH_PKGVERSION_PRE`
///
/// # Examples
///
/// ```no_run
/// # #[macro_use]
/// # extern crate clap;
/// # use clap::Command;
/// # fn main() {
/// let m = Command::new("cmd")
///             .version(crate_version!())
///             .get_matches();
/// # }
/// ```
#[cfg(feature = "cargo")]
#[macro_export]
macro_rules! crate_version {
    () => {
        env!("CARGO_PKG_VERSION")
    };
}

/// Allows you to pull the authors for the command from your Cargo.toml at
/// compile time in the form:
/// `"author1 lastname <author1@example.com>:author2 lastname <author2@example.com>"`
///
/// You can replace the colons with a custom separator by supplying a
/// replacement string, so, for example,
/// `crate_authors!(",\n")` would become
/// `"author1 lastname <author1@example.com>,\nauthor2 lastname <author2@example.com>,\nauthor3 lastname <author3@example.com>"`
///
/// # Examples
///
/// ```no_run
/// # #[macro_use]
/// # extern crate clap;
/// # use clap::Command;
/// # fn main() {
/// let m = Command::new("cmd")
///             .author(crate_authors!("\n"))
///             .get_matches();
/// # }
/// ```
#[cfg(feature = "cargo")]
#[macro_export]
macro_rules! crate_authors {
    ($sep:expr) => {{
        static CACHED: clap::__macro_refs::once_cell::sync::Lazy<String> =
            clap::__macro_refs::once_cell::sync::Lazy::new(|| {
                env!("CARGO_PKG_AUTHORS").replace(':', $sep)
            });

        let s: &'static str = &*CACHED;
        s
    }};
    () => {
        env!("CARGO_PKG_AUTHORS")
    };
}

/// Allows you to pull the description from your Cargo.toml at compile time.
///
/// # Examples
///
/// ```no_run
/// # #[macro_use]
/// # extern crate clap;
/// # use clap::Command;
/// # fn main() {
/// let m = Command::new("cmd")
///             .about(crate_description!())
///             .get_matches();
/// # }
/// ```
#[cfg(feature = "cargo")]
#[macro_export]
macro_rules! crate_description {
    () => {
        env!("CARGO_PKG_DESCRIPTION")
    };
}

/// Allows you to pull the name from your Cargo.toml at compile time.
///
/// # Examples
///
/// ```no_run
/// # #[macro_use]
/// # extern crate clap;
/// # use clap::Command;
/// # fn main() {
/// let m = Command::new(crate_name!())
///             .get_matches();
/// # }
/// ```
#[cfg(feature = "cargo")]
#[macro_export]
macro_rules! crate_name {
    () => {
        env!("CARGO_PKG_NAME")
    };
}

/// Allows you to build the `Command` instance from your Cargo.toml at compile time.
///
/// **NOTE:** Changing the values in your `Cargo.toml` does not trigger a re-build automatically,
/// and therefore won't change the generated output until you recompile.
///
/// In some cases you can "trick" the compiler into triggering a rebuild when your
/// `Cargo.toml` is changed by including this in your `src/main.rs` file
/// `include_str!("../Cargo.toml");`
///
/// # Examples
///
/// ```no_run
/// # #[macro_use]
/// # extern crate clap;
/// # fn main() {
/// let m = command!().get_matches();
/// # }
/// ```
#[cfg(feature = "cargo")]
#[macro_export]
macro_rules! command {
    () => {{
        $crate::command!($crate::crate_name!())
    }};
    ($name:expr) => {{
        let mut cmd = $crate::Command::new($name).version($crate::crate_version!());

        let author = $crate::crate_authors!();
        if !author.is_empty() {
            cmd = cmd.author(author)
        }

        let about = $crate::crate_description!();
        if !about.is_empty() {
            cmd = cmd.about(about)
        }

        cmd
    }};
}

/// Requires `cargo` feature flag to be enabled.
#[cfg(not(feature = "cargo"))]
#[macro_export]
macro_rules! command {
    () => {{
        compile_error!("`cargo` feature flag is required");
    }};
    ($name:expr) => {{
        compile_error!("`cargo` feature flag is required");
    }};
}

/// Deprecated, replaced with [`clap::command!`][crate::command]
#[cfg(feature = "cargo")]
#[cfg_attr(
    feature = "deprecated",
    deprecated(since = "3.1.0", note = "Replaced with `clap::command!")
)]
#[macro_export]
macro_rules! app_from_crate {
    () => {{
        let mut cmd = $crate::Command::new($crate::crate_name!()).version($crate::crate_version!());

        let author = $crate::crate_authors!(", ");
        if !author.is_empty() {
            cmd = cmd.author(author)
        }

        let about = $crate::crate_description!();
        if !about.is_empty() {
            cmd = cmd.about(about)
        }

        cmd
    }};
    ($sep:expr) => {{
        let mut cmd = $crate::Command::new($crate::crate_name!()).version($crate::crate_version!());

        let author = $crate::crate_authors!($sep);
        if !author.is_empty() {
            cmd = cmd.author(author)
        }

        let about = $crate::crate_description!();
        if !about.is_empty() {
            cmd = cmd.about(about)
        }

        cmd
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! arg_impl {
    ( @string $val:ident ) => {
        stringify!($val)
    };
    ( @string $val:literal ) => {{
        let ident_or_string_literal: &str = $val;
        ident_or_string_literal
    }};
    ( @string $val:tt ) => {
        ::std::compile_error!("Only identifiers or string literals supported");
    };
    ( @string ) => {
        None
    };

    ( @char $val:ident ) => {{
        let ident_or_char_literal = stringify!($val);
        debug_assert_eq!(
            ident_or_char_literal.len(),
            1,
            "Single-letter identifier expected, got {}",
            ident_or_char_literal
        );
        ident_or_char_literal.chars().next().unwrap()
    }};
    ( @char $val:literal ) => {{
        let ident_or_char_literal: char = $val;
        ident_or_char_literal
    }};
    ( @char ) => {{
        None
    }};

    (
        @arg
        ($arg:expr)
        --$long:ident
        $($tail:tt)*
    ) => {
        $crate::arg_impl! {
            @arg
            ({
                debug_assert_eq!($arg.get_value_names(), None, "Flags should precede values");
                #[allow(deprecated)]
                {
                    debug_assert!(!$arg.is_multiple_occurrences_set(), "Flags should precede `...`");
                }

                let mut arg = $arg;
                let long = $crate::arg_impl! { @string $long };
                if arg.get_id().is_empty() {
                    arg = arg.id(long);
                }
                arg.long(long)
            })
            $($tail)*
        }
    };
    (
        @arg
        ($arg:expr)
        --$long:literal
        $($tail:tt)*
    ) => {
        $crate::arg_impl! {
            @arg
            ({
                debug_assert_eq!($arg.get_value_names(), None, "Flags should precede values");
                #[allow(deprecated)]
                {
                    debug_assert!(!$arg.is_multiple_occurrences_set(), "Flags should precede `...`");
                }

                let mut arg = $arg;
                let long = $crate::arg_impl! { @string $long };
                if arg.get_id().is_empty() {
                    arg = arg.id(long);
                }
                arg.long(long)
            })
            $($tail)*
        }
    };
    (
        @arg
        ($arg:expr)
        -$short:ident
        $($tail:tt)*
    ) => {
        $crate::arg_impl! {
            @arg
            ({
                debug_assert_eq!($arg.get_long(), None, "Short flags should precede long flags");
                debug_assert_eq!($arg.get_value_names(), None, "Flags should precede values");
                #[allow(deprecated)]
                {
                    debug_assert!(!$arg.is_multiple_occurrences_set(), "Flags should precede `...`");
                }

                $arg.short($crate::arg_impl! { @char $short })
            })
            $($tail)*
        }
    };
    (
        @arg
        ($arg:expr)
        -$short:literal
        $($tail:tt)*
    ) => {
        $crate::arg_impl! {
            @arg
            ({
                debug_assert_eq!($arg.get_long(), None, "Short flags should precede long flags");
                debug_assert_eq!($arg.get_value_names(), None, "Flags should precede values");
                #[allow(deprecated)]
                {
                    debug_assert!(!$arg.is_multiple_occurrences_set(), "Flags should precede `...`");
                }

                $arg.short($crate::arg_impl! { @char $short })
            })
            $($tail)*
        }
    };
    (
        @arg
        ($arg:expr)
        <$value_name:ident>
        $($tail:tt)*
    ) => {
        $crate::arg_impl! {
            @arg
            ({
                #[allow(deprecated)]
                {
                    debug_assert!(!$arg.is_multiple_occurrences_set(), "Values should precede `...`");
                }
                debug_assert_eq!($arg.get_value_names(), None, "Multiple values not yet supported");

                let mut arg = $arg;

                arg = arg.required(true);
                arg = arg.takes_value(true);

                let value_name = $crate::arg_impl! { @string $value_name };
                if arg.get_id().is_empty() {
                    arg = arg.id(value_name);
                }
                arg.value_name(value_name)
            })
            $($tail)*
        }
    };
    (
        @arg
        ($arg:expr)
        <$value_name:literal>
        $($tail:tt)*
    ) => {
        $crate::arg_impl! {
            @arg
            ({
                #[allow(deprecated)]
                {
                    debug_assert!(!$arg.is_multiple_occurrences_set(), "Values should precede `...`");
                }
                debug_assert_eq!($arg.get_value_names(), None, "Multiple values not yet supported");

                let mut arg = $arg;

                arg = arg.required(true);
                arg = arg.takes_value(true);

                let value_name = $crate::arg_impl! { @string $value_name };
                if arg.get_id().is_empty() {
                    arg = arg.id(value_name);
                }
                arg.value_name(value_name)
            })
            $($tail)*
        }
    };
    (
        @arg
        ($arg:expr)
        [$value_name:ident]
        $($tail:tt)*
    ) => {
        $crate::arg_impl! {
            @arg
            ({
                #[allow(deprecated)]
                {
                    debug_assert!(!$arg.is_multiple_occurrences_set(), "Values should precede `...`");
                }
                debug_assert_eq!($arg.get_value_names(), None, "Multiple values not yet supported");

                let mut arg = $arg;

                if arg.get_long().is_none() && arg.get_short().is_none() {
                    arg = arg.required(false);
                } else {
                    arg = arg.min_values(0).max_values(1);
                }
                arg = arg.takes_value(true);

                let value_name = $crate::arg_impl! { @string $value_name };
                if arg.get_id().is_empty() {
                    arg = arg.id(value_name);
                }
                arg.value_name(value_name)
            })
            $($tail)*
        }
    };
    (
        @arg
        ($arg:expr)
        [$value_name:literal]
        $($tail:tt)*
    ) => {
        $crate::arg_impl! {
            @arg
            ({
                #[allow(deprecated)]
                {
                    debug_assert!(!$arg.is_multiple_occurrences_set(), "Values should precede `...`");
                }
                debug_assert_eq!($arg.get_value_names(), None, "Multiple values not yet supported");

                let mut arg = $arg;

                if arg.get_long().is_none() && arg.get_short().is_none() {
                    arg = arg.required(false);
                } else {
                    arg = arg.min_values(0).max_values(1);
                }
                arg = arg.takes_value(true);

                let value_name = $crate::arg_impl! { @string $value_name };
                if arg.get_id().is_empty() {
                    arg = arg.id(value_name);
                }
                arg.value_name(value_name)
            })
            $($tail)*
        }
    };
    (
        @arg
        ($arg:expr)
        ...
        $($tail:tt)*
    ) => {
        $crate::arg_impl! {
            @arg
            ({#[allow(deprecated)]{
                $arg.multiple_occurrences(true)
            }})
            $($tail)*
        }
    };
    (
        @arg
        ($arg:expr)
        $help:literal
    ) => {
        $arg.help($help)
    };
    (
        @arg
        ($arg:expr)
    ) => {
        $arg
    };
}

/// Create an [`Arg`] from a usage string.
///
/// Allows creation of basic settings for the [`Arg`].
///
/// **NOTE**: Not all settings may be set using the usage string method. Some properties are
/// only available via the builder pattern.
///
/// # Syntax
///
/// Usage strings typically following the form:
///
/// ```notrust
/// [explicit name] [short] [long] [value names] [...] [help string]
/// ```
///
/// ### Explicit Name
///
/// The name may be either a bare-word or a string, followed by a `:`, like `name:` or
/// `"name":`.
///
/// *Note:* This is an optional field, if it's omitted the argument will use one of the additional
/// fields as the name using the following priority order:
///
///  1. Explicit Name
///  2. Long
///  3. Value Name
///
/// See [`Arg::name`][crate::Arg::name].
///
/// ### Short
///
/// A short flag is a `-` followed by either a bare-character or quoted character, like `-f` or
/// `-'f'`.
///
/// See [`Arg::short`][crate::Arg::short].
///
/// ### Long
///
/// A long flag is a `--` followed by either a bare-word or a string, like `--foo` or
/// `--"foo"`.
///
/// See [`Arg::long`][crate::Arg::long].
///
/// ### Values (Value Notation)
///
/// This is set by placing bare-word between:
/// - `[]` like `[FOO]`
///   - Positional argument: optional
///   - Named argument: optional value
/// - `<>` like `<FOO>`: required
///
/// See [`Arg::value_name`][crate::Arg::value_name].
///
/// ### `...`
///
/// `...` (three consecutive dots/periods) specifies that this argument may occur multiple
/// times (not to be confused with multiple values per occurrence).
///
/// See [`Arg::multiple_occurrences`][crate::Arg::multiple_occurrences].
///
/// ### Help String
///
/// The help string is denoted between a pair of double quotes `""` and may contain any
/// characters.
///
/// # Examples
///
/// ```rust
/// # use clap::{Command, Arg, arg};
/// Command::new("prog")
///     .args(&[
///         arg!(--config <FILE> "a required file for the configuration and no short"),
///         arg!(-d --debug ... "turns on debugging information and allows multiples"),
///         arg!([input] "an optional input file to use")
/// ])
/// # ;
/// ```
/// [`Arg`]: ./struct.Arg.html
#[macro_export]
macro_rules! arg {
    ( $name:ident: $($tail:tt)+ ) => {
        $crate::arg_impl! {
            @arg ($crate::Arg::new($crate::arg_impl! { @string $name })) $($tail)+
        }
    };
    ( $($tail:tt)+ ) => {{
        let arg = $crate::arg_impl! {
            @arg ($crate::Arg::default()) $($tail)+
        };
        debug_assert!(!arg.get_id().is_empty(), "Without a value or long flag, the `name:` prefix is required");
        arg
    }};
}

/// Deprecated, replaced with [`clap::Parser`][crate::Parser] and [`clap::arg!`][crate::arg] (Issue clap-rs/clap#2835)
#[cfg_attr(
    feature = "deprecated",
    deprecated(
        since = "3.0.0",
        note = "Replaced with `clap::Parser` for a declarative API (Issue clap-rs/clap#2835)"
    )
)]
#[doc(hidden)]
#[macro_export]
macro_rules! clap_app {
    (@app ($builder:expr)) => { $builder };
    (@app ($builder:expr) (@arg ($name:expr): $($tail:tt)*) $($tt:tt)*) => {
        $crate::clap_app!{ @app
            ($builder.arg(
                $crate::clap_app!{ @arg ($crate::Arg::new($name)) (-) $($tail)* }))
            $($tt)*
        }
    };
    (@app ($builder:expr) (@arg $name:ident: $($tail:tt)*) $($tt:tt)*) => {
        $crate::clap_app!{ @app
            ($builder.arg(
                $crate::clap_app!{ @arg ($crate::Arg::new(stringify!($name))) (-) $($tail)* }))
            $($tt)*
        }
    };
    (@app ($builder:expr) (@setting $setting:ident) $($tt:tt)*) => {
        $crate::clap_app!{ @app
            ($builder.setting($crate::AppSettings::$setting))
            $($tt)*
        }
    };
// Treat the application builder as an argument to set its attributes
    (@app ($builder:expr) (@attributes $($attr:tt)*) $($tt:tt)*) => {
        $crate::clap_app!{ @app ($crate::clap_app!{ @arg ($builder) $($attr)* }) $($tt)* }
    };
    (@app ($builder:expr) (@group $name:ident => $($tail:tt)*) $($tt:tt)*) => {
        $crate::clap_app!{ @app
            ($crate::clap_app!{ @group ($builder, $crate::ArgGroup::new(stringify!($name))) $($tail)* })
            $($tt)*
        }
    };
    (@app ($builder:expr) (@group $name:ident !$ident:ident => $($tail:tt)*) $($tt:tt)*) => {
        $crate::clap_app!{ @app
            ($crate::clap_app!{ @group ($builder, $crate::ArgGroup::new(stringify!($name)).$ident(false)) $($tail)* })
            $($tt)*
        }
    };
    (@app ($builder:expr) (@group $name:ident +$ident:ident => $($tail:tt)*) $($tt:tt)*) => {
        $crate::clap_app!{ @app
            ($crate::clap_app!{ @group ($builder, $crate::ArgGroup::new(stringify!($name)).$ident(true)) $($tail)* })
            $($tt)*
        }
    };
// Handle subcommand creation
    (@app ($builder:expr) (@subcommand $name:ident => $($tail:tt)*) $($tt:tt)*) => {
        $crate::clap_app!{ @app
            ($builder.subcommand(
                $crate::clap_app!{ @app ($crate::Command::new(stringify!($name))) $($tail)* }
            ))
            $($tt)*
        }
    };
// Yaml like function calls - used for setting various meta directly against the app
    (@app ($builder:expr) ($ident:ident: $($v:expr),*) $($tt:tt)*) => {
// $crate::clap_app!{ @app ($builder.$ident($($v),*)) $($tt)* }
        $crate::clap_app!{ @app
            ($builder.$ident($($v),*))
            $($tt)*
        }
    };

// Add members to group and continue argument handling with the parent builder
    (@group ($builder:expr, $group:expr)) => { $builder.group($group) };
    // Treat the group builder as an argument to set its attributes
    (@group ($builder:expr, $group:expr) (@attributes $($attr:tt)*) $($tt:tt)*) => {
        $crate::clap_app!{ @group ($builder, $crate::clap_app!{ @arg ($group) (-) $($attr)* }) $($tt)* }
    };
    (@group ($builder:expr, $group:expr) (@arg $name:ident: $($tail:tt)*) $($tt:tt)*) => {
        $crate::clap_app!{ @group
            ($crate::clap_app!{ @app ($builder) (@arg $name: $($tail)*) },
             $group.arg(stringify!($name)))
            $($tt)*
        }
    };

// No more tokens to munch
    (@arg ($arg:expr) $modes:tt) => { $arg };
// Shorthand tokens influenced by the usage_string
    (@arg ($arg:expr) $modes:tt --($long:expr) $($tail:tt)*) => {
        $crate::clap_app!{ @arg ($arg.long($long)) $modes $($tail)* }
    };
    (@arg ($arg:expr) $modes:tt --$long:ident $($tail:tt)*) => {
        $crate::clap_app!{ @arg ($arg.long(stringify!($long))) $modes $($tail)* }
    };
    (@arg ($arg:expr) $modes:tt -$short:ident $($tail:tt)*) => {
        $crate::clap_app!{ @arg ($arg.short(stringify!($short).chars().next().unwrap())) $modes $($tail)* }
    };
    (@arg ($arg:expr) (-) <$var:ident> $($tail:tt)*) => {
        $crate::clap_app!{ @arg ($arg.value_name(stringify!($var))) (+) +takes_value +required $($tail)* }
    };
    (@arg ($arg:expr) (+) <$var:ident> $($tail:tt)*) => {
        $crate::clap_app!{ @arg ($arg.value_name(stringify!($var))) (+) $($tail)* }
    };
    (@arg ($arg:expr) (-) [$var:ident] $($tail:tt)*) => {
        $crate::clap_app!{ @arg ($arg.value_name(stringify!($var))) (+) +takes_value $($tail)* }
    };
    (@arg ($arg:expr) (+) [$var:ident] $($tail:tt)*) => {
        $crate::clap_app!{ @arg ($arg.value_name(stringify!($var))) (+) $($tail)* }
    };
    (@arg ($arg:expr) $modes:tt ... $($tail:tt)*) => {
        $crate::clap_app!{ @arg ($arg) $modes +multiple +takes_value $($tail)* }
    };
// Shorthand magic
    (@arg ($arg:expr) $modes:tt #{$n:expr, $m:expr} $($tail:tt)*) => {
        $crate::clap_app!{ @arg ($arg) $modes min_values($n) max_values($m) $($tail)* }
    };
    (@arg ($arg:expr) $modes:tt * $($tail:tt)*) => {
        $crate::clap_app!{ @arg ($arg) $modes +required $($tail)* }
    };
// !foo -> .foo(false)
    (@arg ($arg:expr) $modes:tt !$ident:ident $($tail:tt)*) => {
        $crate::clap_app!{ @arg ($arg.$ident(false)) $modes $($tail)* }
    };
// +foo -> .foo(true)
    (@arg ($arg:expr) $modes:tt +$ident:ident $($tail:tt)*) => {
        $crate::clap_app!{ @arg ($arg.$ident(true)) $modes $($tail)* }
    };
// Validator
    (@arg ($arg:expr) $modes:tt {$fn_:expr} $($tail:tt)*) => {
        $crate::clap_app!{ @arg ($arg.validator($fn_)) $modes $($tail)* }
    };
    (@as_expr $expr:expr) => { $expr };
// Help
    (@arg ($arg:expr) $modes:tt $desc:tt) => { $arg.help($crate::clap_app!{ @as_expr $desc }) };
// Handle functions that need to be called multiple times for each argument
    (@arg ($arg:expr) $modes:tt $ident:ident[$($target:ident)*] $($tail:tt)*) => {
        $crate::clap_app!{ @arg ($arg $( .$ident(stringify!($target)) )*) $modes $($tail)* }
    };
// Inherit builder's functions, e.g. `index(2)`, `requires_if("val", "arg")`
    (@arg ($arg:expr) $modes:tt $ident:ident($($expr:expr),*) $($tail:tt)*) => {
        $crate::clap_app!{ @arg ($arg.$ident($($expr),*)) $modes $($tail)* }
    };
// Inherit builder's functions with trailing comma, e.g. `index(2,)`, `requires_if("val", "arg",)`
    (@arg ($arg:expr) $modes:tt $ident:ident($($expr:expr,)*) $($tail:tt)*) => {
        $crate::clap_app!{ @arg ($arg.$ident($($expr),*)) $modes $($tail)* }
    };

// Build a subcommand outside of an app.
    (@subcommand $name:ident => $($tail:tt)*) => {
        $crate::clap_app!{ @app ($crate::Command::new(stringify!($name))) $($tail)* }
    };
// Start the magic
    (($name:expr) => $($tail:tt)*) => {{
        $crate::clap_app!{ @app ($crate::Command::new($name)) $($tail)*}
    }};

    ($name:ident => $($tail:tt)*) => {{
        $crate::clap_app!{ @app ($crate::Command::new(stringify!($name))) $($tail)*}
    }};
}

macro_rules! impl_settings {
    ($settings:ident, $flags:ident,
        $(
            $(#[$inner:ident $($args:tt)*])*
            $setting:ident => $flag:path
        ),+
    ) => {
        impl $flags {
            #[allow(dead_code)]
            pub(crate) fn empty() -> Self {
                $flags(Flags::empty())
            }

            #[allow(dead_code)]
            pub(crate) fn insert(&mut self, rhs: Self) {
                self.0.insert(rhs.0);
            }

            #[allow(dead_code)]
            pub(crate) fn remove(&mut self, rhs: Self) {
                self.0.remove(rhs.0);
            }

            #[allow(dead_code)]
            pub(crate) fn set(&mut self, s: $settings) {
                #[allow(deprecated)]  // some Settings might be deprecated
                match s {
                    $(
                        $(#[$inner $($args)*])*
                        $settings::$setting => self.0.insert($flag),
                    )*
                }
            }

            #[allow(dead_code)]
            pub(crate) fn unset(&mut self, s: $settings) {
                #[allow(deprecated)]  // some Settings might be deprecated
                match s {
                    $(
                        $(#[$inner $($args)*])*
                        $settings::$setting => self.0.remove($flag),
                    )*
                }
            }

            #[allow(dead_code)]
            pub(crate) fn is_set(&self, s: $settings) -> bool {
                #[allow(deprecated)]  // some Settings might be deprecated
                match s {
                    $(
                        $(#[$inner $($args)*])*
                        $settings::$setting => self.0.contains($flag),
                    )*
                }
            }
        }

        impl BitOr for $flags {
            type Output = Self;

            fn bitor(mut self, rhs: Self) -> Self::Output {
                self.0.insert(rhs.0);
                self
            }
        }

        impl From<$settings> for $flags {
            fn from(setting: $settings) -> Self {
                let mut flags = $flags::empty();
                flags.set(setting);
                flags
            }
        }

        impl BitOr<$settings> for $flags {
            type Output = Self;

            fn bitor(mut self, rhs: $settings) -> Self::Output {
                self.set(rhs);
                self
            }
        }

        impl BitOr for $settings {
            type Output = $flags;

            fn bitor(self, rhs: Self) -> Self::Output {
                let mut flags = $flags::empty();
                flags.set(self);
                flags.set(rhs);
                flags
            }
        }
    }
}

// Convenience for writing to stderr thanks to https://github.com/BurntSushi
macro_rules! wlnerr {
    ($($arg:tt)*) => ({
        use std::io::{Write, stderr};
        writeln!(&mut stderr(), $($arg)*).ok();
    })
}

#[cfg(feature = "debug")]
macro_rules! debug {
    ($($arg:tt)*) => ({
        let prefix = format!("[{:>w$}] \t", module_path!(), w = 28);
        let body = format!($($arg)*);
        let mut color = $crate::output::fmt::Colorizer::new($crate::output::fmt::Stream::Stderr, $crate::ColorChoice::Auto);
        color.hint(prefix);
        color.hint(body);
        color.none("\n");
        let _ = color.print();
    })
}

#[cfg(not(feature = "debug"))]
macro_rules! debug {
    ($($arg:tt)*) => {};
}
