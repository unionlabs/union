// Std
use std::{
    borrow::Cow,
    collections::HashMap,
    ffi::{OsStr, OsString},
    iter::Map,
    slice::Iter,
};

// Internal
use crate::{
    args::{MatchedArg, SubCommand},
    INVALID_UTF8,
};

/// Used to get information about the arguments that were supplied to the program at runtime by
/// the user. New instances of this struct are obtained by using the [`App::get_matches`] family of
/// methods.
///
/// # Examples
///
/// ```no_run
/// # use clap::{App, Arg};
/// let matches = App::new("MyApp")
///     .arg(Arg::with_name("out")
///         .long("output")
///         .required(true)
///         .takes_value(true))
///     .arg(Arg::with_name("debug")
///         .short("d")
///         .multiple(true))
///     .arg(Arg::with_name("cfg")
///         .short("c")
///         .takes_value(true))
///     .get_matches(); // builds the instance of ArgMatches
///
/// // to get information about the "cfg" argument we created, such as the value supplied we use
/// // various ArgMatches methods, such as ArgMatches::value_of
/// if let Some(c) = matches.value_of("cfg") {
///     println!("Value for -c: {}", c);
/// }
///
/// // The ArgMatches::value_of method returns an Option because the user may not have supplied
/// // that argument at runtime. But if we specified that the argument was "required" as we did
/// // with the "out" argument, we can safely unwrap because `clap` verifies that was actually
/// // used at runtime.
/// println!("Value for --output: {}", matches.value_of("out").unwrap());
///
/// // You can check the presence of an argument
/// if matches.is_present("out") {
///     // Another way to check if an argument was present, or if it occurred multiple times is to
///     // use occurrences_of() which returns 0 if an argument isn't found at runtime, or the
///     // number of times that it occurred, if it was. To allow an argument to appear more than
///     // once, you must use the .multiple(true) method, otherwise it will only return 1 or 0.
///     if matches.occurrences_of("debug") > 2 {
///         println!("Debug mode is REALLY on, don't be crazy");
///     } else {
///         println!("Debug mode kind of on");
///     }
/// }
/// ```
/// [`App::get_matches`]: ./struct.App.html#method.get_matches
#[derive(Debug, Clone)]
pub struct ArgMatches<'a> {
    #[doc(hidden)]
    pub args: HashMap<&'a str, MatchedArg>,
    #[doc(hidden)]
    pub subcommand: Option<Box<SubCommand<'a>>>,
    #[doc(hidden)]
    pub usage: Option<String>,
}

impl<'a> Default for ArgMatches<'a> {
    fn default() -> Self {
        ArgMatches {
            args: HashMap::new(),
            subcommand: None,
            usage: None,
        }
    }
}

impl<'a> ArgMatches<'a> {
    #[doc(hidden)]
    pub fn new() -> Self {
        ArgMatches {
            ..Default::default()
        }
    }

    /// Gets the value of a specific [option] or [positional] argument (i.e. an argument that takes
    /// an additional value at runtime). If the option wasn't present at runtime
    /// it returns `None`.
    ///
    /// *NOTE:* If getting a value for an option or positional argument that allows multiples,
    /// prefer [`ArgMatches::values_of`] as `ArgMatches::value_of` will only return the *first*
    /// value.
    ///
    /// # Panics
    ///
    /// This method will [`panic!`] if the value contains invalid UTF-8 code points.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("myapp")
    ///     .arg(Arg::with_name("output")
    ///         .takes_value(true))
    ///     .get_matches_from(vec!["myapp", "something"]);
    ///
    /// assert_eq!(m.value_of("output"), Some("something"));
    /// ```
    /// [option]: ./struct.Arg.html#method.takes_value
    /// [positional]: ./struct.Arg.html#method.index
    /// [`ArgMatches::values_of`]: ./struct.ArgMatches.html#method.values_of
    /// [`panic!`]: https://doc.rust-lang.org/std/macro.panic!.html
    pub fn value_of<S: AsRef<str>>(&self, name: S) -> Option<&str> {
        if let Some(arg) = self.args.get(name.as_ref()) {
            if let Some(v) = arg.vals.get(0) {
                return Some(v.to_str().expect(INVALID_UTF8));
            }
        }
        None
    }

    /// Gets the lossy value of a specific argument. If the argument wasn't present at runtime
    /// it returns `None`. A lossy value is one which contains invalid UTF-8 code points, those
    /// invalid points will be replaced with `\u{FFFD}`
    ///
    /// *NOTE:* If getting a value for an option or positional argument that allows multiples,
    /// prefer [`Arg::values_of_lossy`] as `value_of_lossy()` will only return the *first* value.
    ///
    /// # Examples
    ///
    #[cfg_attr(not(unix), doc = " ```ignore")]
    #[cfg_attr(unix, doc = " ```")]
    /// # use clap::{App, Arg};
    /// use std::ffi::OsString;
    /// use std::os::unix::ffi::{OsStrExt,OsStringExt};
    ///
    /// let m = App::new("utf8")
    ///     .arg(Arg::from_usage("<arg> 'some arg'"))
    ///     .get_matches_from(vec![OsString::from("myprog"),
    ///                             // "Hi {0xe9}!"
    ///                             OsString::from_vec(vec![b'H', b'i', b' ', 0xe9, b'!'])]);
    /// assert_eq!(&*m.value_of_lossy("arg").unwrap(), "Hi \u{FFFD}!");
    /// ```
    /// [`Arg::values_of_lossy`]: ./struct.ArgMatches.html#method.values_of_lossy
    pub fn value_of_lossy<S: AsRef<str>>(&'a self, name: S) -> Option<Cow<'a, str>> {
        if let Some(arg) = self.args.get(name.as_ref()) {
            if let Some(v) = arg.vals.get(0) {
                return Some(v.to_string_lossy());
            }
        }
        None
    }

    /// Gets the OS version of a string value of a specific argument. If the option wasn't present
    /// at runtime it returns `None`. An OS value on Unix-like systems is any series of bytes,
    /// regardless of whether or not they contain valid UTF-8 code points. Since [`String`]s in
    /// Rust are guaranteed to be valid UTF-8, a valid filename on a Unix system as an argument
    /// value may contain invalid UTF-8 code points.
    ///
    /// *NOTE:* If getting a value for an option or positional argument that allows multiples,
    /// prefer [`ArgMatches::values_of_os`] as `Arg::value_of_os` will only return the *first*
    /// value.
    ///
    /// # Examples
    ///
    #[cfg_attr(not(unix), doc = " ```ignore")]
    #[cfg_attr(unix, doc = " ```")]
    /// # use clap::{App, Arg};
    /// use std::ffi::OsString;
    /// use std::os::unix::ffi::{OsStrExt,OsStringExt};
    ///
    /// let m = App::new("utf8")
    ///     .arg(Arg::from_usage("<arg> 'some arg'"))
    ///     .get_matches_from(vec![OsString::from("myprog"),
    ///                             // "Hi {0xe9}!"
    ///                             OsString::from_vec(vec![b'H', b'i', b' ', 0xe9, b'!'])]);
    /// assert_eq!(&*m.value_of_os("arg").unwrap().as_bytes(), [b'H', b'i', b' ', 0xe9, b'!']);
    /// ```
    /// [`String`]: https://doc.rust-lang.org/std/string/struct.String.html
    /// [`ArgMatches::values_of_os`]: ./struct.ArgMatches.html#method.values_of_os
    pub fn value_of_os<S: AsRef<str>>(&self, name: S) -> Option<&OsStr> {
        self.args
            .get(name.as_ref())
            .and_then(|arg| arg.vals.get(0).map(|v| v.as_os_str()))
    }

    /// Gets a [`Values`] struct which implements [`Iterator`] for values of a specific argument
    /// (i.e. an argument that takes multiple values at runtime). If the option wasn't present at
    /// runtime it returns `None`
    ///
    /// # Panics
    ///
    /// This method will panic if any of the values contain invalid UTF-8 code points.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("myprog")
    ///     .arg(Arg::with_name("output")
    ///         .multiple(true)
    ///         .short("o")
    ///         .takes_value(true))
    ///     .get_matches_from(vec![
    ///         "myprog", "-o", "val1", "val2", "val3"
    ///     ]);
    /// let vals: Vec<&str> = m.values_of("output").unwrap().collect();
    /// assert_eq!(vals, ["val1", "val2", "val3"]);
    /// ```
    /// [`Values`]: ./struct.Values.html
    /// [`Iterator`]: https://doc.rust-lang.org/std/iter/trait.Iterator.html
    pub fn values_of<S: AsRef<str>>(&'a self, name: S) -> Option<Values<'a>> {
        if let Some(arg) = self.args.get(name.as_ref()) {
            fn to_str_slice(o: &OsString) -> &str {
                o.to_str().expect(INVALID_UTF8)
            }
            let to_str_slice: fn(&OsString) -> &str = to_str_slice; // coerce to fn pointer
            return Some(Values {
                iter: arg.vals.iter().map(to_str_slice),
            });
        }
        None
    }

    /// Gets the lossy values of a specific argument. If the option wasn't present at runtime
    /// it returns `None`. A lossy value is one where if it contains invalid UTF-8 code points,
    /// those invalid points will be replaced with `\u{FFFD}`
    ///
    /// # Examples
    ///
    #[cfg_attr(not(unix), doc = " ```ignore")]
    #[cfg_attr(unix, doc = " ```")]
    /// # use clap::{App, Arg};
    /// use std::ffi::OsString;
    /// use std::os::unix::ffi::OsStringExt;
    ///
    /// let m = App::new("utf8")
    ///     .arg(Arg::from_usage("<arg>... 'some arg'"))
    ///     .get_matches_from(vec![OsString::from("myprog"),
    ///                             // "Hi"
    ///                             OsString::from_vec(vec![b'H', b'i']),
    ///                             // "{0xe9}!"
    ///                             OsString::from_vec(vec![0xe9, b'!'])]);
    /// let mut itr = m.values_of_lossy("arg").unwrap().into_iter();
    /// assert_eq!(&itr.next().unwrap()[..], "Hi");
    /// assert_eq!(&itr.next().unwrap()[..], "\u{FFFD}!");
    /// assert_eq!(itr.next(), None);
    /// ```
    pub fn values_of_lossy<S: AsRef<str>>(&'a self, name: S) -> Option<Vec<String>> {
        if let Some(arg) = self.args.get(name.as_ref()) {
            return Some(
                arg.vals
                    .iter()
                    .map(|v| v.to_string_lossy().into_owned())
                    .collect(),
            );
        }
        None
    }

    /// Gets a [`OsValues`] struct which is implements [`Iterator`] for [`OsString`] values of a
    /// specific argument. If the option wasn't present at runtime it returns `None`. An OS value
    /// on Unix-like systems is any series of bytes, regardless of whether or not they contain
    /// valid UTF-8 code points. Since [`String`]s in Rust are guaranteed to be valid UTF-8, a valid
    /// filename as an argument value on Linux (for example) may contain invalid UTF-8 code points.
    ///
    /// # Examples
    ///
    #[cfg_attr(not(unix), doc = " ```ignore")]
    #[cfg_attr(unix, doc = " ```")]
    /// # use clap::{App, Arg};
    /// use std::ffi::{OsStr,OsString};
    /// use std::os::unix::ffi::{OsStrExt,OsStringExt};
    ///
    /// let m = App::new("utf8")
    ///     .arg(Arg::from_usage("<arg>... 'some arg'"))
    ///     .get_matches_from(vec![OsString::from("myprog"),
    ///                                 // "Hi"
    ///                                 OsString::from_vec(vec![b'H', b'i']),
    ///                                 // "{0xe9}!"
    ///                                 OsString::from_vec(vec![0xe9, b'!'])]);
    ///
    /// let mut itr = m.values_of_os("arg").unwrap().into_iter();
    /// assert_eq!(itr.next(), Some(OsStr::new("Hi")));
    /// assert_eq!(itr.next(), Some(OsStr::from_bytes(&[0xe9, b'!'])));
    /// assert_eq!(itr.next(), None);
    /// ```
    /// [`OsValues`]: ./struct.OsValues.html
    /// [`Iterator`]: https://doc.rust-lang.org/std/iter/trait.Iterator.html
    /// [`OsString`]: https://doc.rust-lang.org/std/ffi/struct.OsString.html
    /// [`String`]: https://doc.rust-lang.org/std/string/struct.String.html
    pub fn values_of_os<S: AsRef<str>>(&'a self, name: S) -> Option<OsValues<'a>> {
        fn to_str_slice(o: &OsString) -> &OsStr {
            &*o
        }
        let to_str_slice: fn(&'a OsString) -> &'a OsStr = to_str_slice; // coerce to fn pointer
        if let Some(arg) = self.args.get(name.as_ref()) {
            return Some(OsValues {
                iter: arg.vals.iter().map(to_str_slice),
            });
        }
        None
    }

    /// Returns `true` if an argument was present at runtime, otherwise `false`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("myprog")
    ///     .arg(Arg::with_name("debug")
    ///         .short("d"))
    ///     .get_matches_from(vec![
    ///         "myprog", "-d"
    ///     ]);
    ///
    /// assert!(m.is_present("debug"));
    /// ```
    pub fn is_present<S: AsRef<str>>(&self, name: S) -> bool {
        if let Some(ref sc) = self.subcommand {
            if sc.name == name.as_ref() {
                return true;
            }
        }
        self.args.contains_key(name.as_ref())
    }

    /// Returns the number of times an argument was used at runtime. If an argument isn't present
    /// it will return `0`.
    ///
    /// **NOTE:** This returns the number of times the argument was used, *not* the number of
    /// values. For example, `-o val1 val2 val3 -o val4` would return `2` (2 occurrences, but 4
    /// values).
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("myprog")
    ///     .arg(Arg::with_name("debug")
    ///         .short("d")
    ///         .multiple(true))
    ///     .get_matches_from(vec![
    ///         "myprog", "-d", "-d", "-d"
    ///     ]);
    ///
    /// assert_eq!(m.occurrences_of("debug"), 3);
    /// ```
    ///
    /// This next example shows that counts actual uses of the argument, not just `-`'s
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("myprog")
    ///     .arg(Arg::with_name("debug")
    ///         .short("d")
    ///         .multiple(true))
    ///     .arg(Arg::with_name("flag")
    ///         .short("f"))
    ///     .get_matches_from(vec![
    ///         "myprog", "-ddfd"
    ///     ]);
    ///
    /// assert_eq!(m.occurrences_of("debug"), 3);
    /// assert_eq!(m.occurrences_of("flag"), 1);
    /// ```
    pub fn occurrences_of<S: AsRef<str>>(&self, name: S) -> u64 {
        self.args.get(name.as_ref()).map_or(0, |a| a.occurs)
    }

    /// Gets the starting index of the argument in respect to all other arguments. Indices are
    /// similar to argv indices, but are not exactly 1:1.
    ///
    /// For flags (i.e. those arguments which don't have an associated value), indices refer
    /// to occurrence of the switch, such as `-f`, or `--flag`. However, for options the indices
    /// refer to the *values* `-o val` would therefore not represent two distinct indices, only the
    /// index for `val` would be recorded. This is by design.
    ///
    /// Besides the flag/option descrepancy, the primary difference between an argv index and clap
    /// index, is that clap continues counting once all arguments have properly seperated, whereas
    /// an argv index does not.
    ///
    /// The examples should clear this up.
    ///
    /// *NOTE:* If an argument is allowed multiple times, this method will only give the *first*
    /// index.
    ///
    /// # Examples
    ///
    /// The argv indices are listed in the comments below. See how they correspond to the clap
    /// indices. Note that if it's not listed in a clap index, this is becuase it's not saved in
    /// in an `ArgMatches` struct for querying.
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("myapp")
    ///     .arg(Arg::with_name("flag")
    ///         .short("f"))
    ///     .arg(Arg::with_name("option")
    ///         .short("o")
    ///         .takes_value(true))
    ///     .get_matches_from(vec!["myapp", "-f", "-o", "val"]);
    ///             // ARGV idices: ^0       ^1    ^2    ^3
    ///             // clap idices:          ^1          ^3
    ///
    /// assert_eq!(m.index_of("flag"), Some(1));
    /// assert_eq!(m.index_of("option"), Some(3));
    /// ```
    ///
    /// Now notice, if we use one of the other styles of options:
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("myapp")
    ///     .arg(Arg::with_name("flag")
    ///         .short("f"))
    ///     .arg(Arg::with_name("option")
    ///         .short("o")
    ///         .takes_value(true))
    ///     .get_matches_from(vec!["myapp", "-f", "-o=val"]);
    ///             // ARGV idices: ^0       ^1    ^2
    ///             // clap idices:          ^1       ^3
    ///
    /// assert_eq!(m.index_of("flag"), Some(1));
    /// assert_eq!(m.index_of("option"), Some(3));
    /// ```
    ///
    /// Things become much more complicated, or clear if we look at a more complex combination of
    /// flags. Let's also throw in the final option style for good measure.
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("myapp")
    ///     .arg(Arg::with_name("flag")
    ///         .short("f"))
    ///     .arg(Arg::with_name("flag2")
    ///         .short("F"))
    ///     .arg(Arg::with_name("flag3")
    ///         .short("z"))
    ///     .arg(Arg::with_name("option")
    ///         .short("o")
    ///         .takes_value(true))
    ///     .get_matches_from(vec!["myapp", "-fzF", "-oval"]);
    ///             // ARGV idices: ^0      ^1       ^2
    ///             // clap idices:         ^1,2,3    ^5
    ///             //
    ///             // clap sees the above as 'myapp -f -z -F -o val'
    ///             //                         ^0    ^1 ^2 ^3 ^4 ^5
    /// assert_eq!(m.index_of("flag"), Some(1));
    /// assert_eq!(m.index_of("flag2"), Some(3));
    /// assert_eq!(m.index_of("flag3"), Some(2));
    /// assert_eq!(m.index_of("option"), Some(5));
    /// ```
    ///
    /// One final combination of flags/options to see how they combine:
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("myapp")
    ///     .arg(Arg::with_name("flag")
    ///         .short("f"))
    ///     .arg(Arg::with_name("flag2")
    ///         .short("F"))
    ///     .arg(Arg::with_name("flag3")
    ///         .short("z"))
    ///     .arg(Arg::with_name("option")
    ///         .short("o")
    ///         .takes_value(true)
    ///         .multiple(true))
    ///     .get_matches_from(vec!["myapp", "-fzFoval"]);
    ///             // ARGV idices: ^0       ^1
    ///             // clap idices:          ^1,2,3^5
    ///             //
    ///             // clap sees the above as 'myapp -f -z -F -o val'
    ///             //                         ^0    ^1 ^2 ^3 ^4 ^5
    /// assert_eq!(m.index_of("flag"), Some(1));
    /// assert_eq!(m.index_of("flag2"), Some(3));
    /// assert_eq!(m.index_of("flag3"), Some(2));
    /// assert_eq!(m.index_of("option"), Some(5));
    /// ```
    ///
    /// The last part to mention is when values are sent in multiple groups with a [delimiter].
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("myapp")
    ///     .arg(Arg::with_name("option")
    ///         .short("o")
    ///         .takes_value(true)
    ///         .multiple(true))
    ///     .get_matches_from(vec!["myapp", "-o=val1,val2,val3"]);
    ///             // ARGV idices: ^0       ^1
    ///             // clap idices:             ^2   ^3   ^4
    ///             //
    ///             // clap sees the above as 'myapp -o val1 val2 val3'
    ///             //                         ^0    ^1 ^2   ^3   ^4
    /// assert_eq!(m.index_of("option"), Some(2));
    /// ```
    /// [`ArgMatches`]: ./struct.ArgMatches.html
    /// [delimiter]: ./struct.Arg.html#method.value_delimiter
    pub fn index_of<S: AsRef<str>>(&self, name: S) -> Option<usize> {
        if let Some(arg) = self.args.get(name.as_ref()) {
            if let Some(i) = arg.indices.get(0) {
                return Some(*i);
            }
        }
        None
    }

    /// Gets all indices of the argument in respect to all other arguments. Indices are
    /// similar to argv indices, but are not exactly 1:1.
    ///
    /// For flags (i.e. those arguments which don't have an associated value), indices refer
    /// to occurrence of the switch, such as `-f`, or `--flag`. However, for options the indices
    /// refer to the *values* `-o val` would therefore not represent two distinct indices, only the
    /// index for `val` would be recorded. This is by design.
    ///
    /// *NOTE:* For more information about how clap indices compare to argv indices, see
    /// [`ArgMatches::index_of`]
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("myapp")
    ///     .arg(Arg::with_name("option")
    ///         .short("o")
    ///         .takes_value(true)
    ///         .use_delimiter(true)
    ///         .multiple(true))
    ///     .get_matches_from(vec!["myapp", "-o=val1,val2,val3"]);
    ///             // ARGV idices: ^0       ^1
    ///             // clap idices:             ^2   ^3   ^4
    ///             //
    ///             // clap sees the above as 'myapp -o val1 val2 val3'
    ///             //                         ^0    ^1 ^2   ^3   ^4
    /// assert_eq!(m.indices_of("option").unwrap().collect::<Vec<_>>(), &[2, 3, 4]);
    /// ```
    ///
    /// Another quick example is when flags and options are used together
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("myapp")
    ///     .arg(Arg::with_name("option")
    ///         .short("o")
    ///         .takes_value(true)
    ///         .multiple(true))
    ///     .arg(Arg::with_name("flag")
    ///         .short("f")
    ///         .multiple(true))
    ///     .get_matches_from(vec!["myapp", "-o", "val1", "-f", "-o", "val2", "-f"]);
    ///             // ARGV idices: ^0       ^1    ^2      ^3    ^4    ^5      ^6
    ///             // clap idices:                ^2      ^3          ^5      ^6
    ///
    /// assert_eq!(m.indices_of("option").unwrap().collect::<Vec<_>>(), &[2, 5]);
    /// assert_eq!(m.indices_of("flag").unwrap().collect::<Vec<_>>(), &[3, 6]);
    /// ```
    ///
    /// One final example, which is an odd case; if we *don't* use  value delimiter as we did with
    /// the first example above instead of `val1`, `val2` and `val3` all being distinc values, they
    /// would all be a single value of `val1,val2,val3`, in which case case they'd only receive a
    /// single index.
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("myapp")
    ///     .arg(Arg::with_name("option")
    ///         .short("o")
    ///         .takes_value(true)
    ///         .multiple(true))
    ///     .get_matches_from(vec!["myapp", "-o=val1,val2,val3"]);
    ///             // ARGV idices: ^0       ^1
    ///             // clap idices:             ^2
    ///             //
    ///             // clap sees the above as 'myapp -o "val1,val2,val3"'
    ///             //                         ^0    ^1  ^2
    /// assert_eq!(m.indices_of("option").unwrap().collect::<Vec<_>>(), &[2]);
    /// ```
    /// [`ArgMatches`]: ./struct.ArgMatches.html
    /// [`ArgMatches::index_of`]: ./struct.ArgMatches.html#method.index_of
    /// [delimiter]: ./struct.Arg.html#method.value_delimiter
    pub fn indices_of<S: AsRef<str>>(&'a self, name: S) -> Option<Indices<'a>> {
        if let Some(arg) = self.args.get(name.as_ref()) {
            fn to_usize(i: &usize) -> usize {
                *i
            }
            let to_usize: fn(&usize) -> usize = to_usize; // coerce to fn pointer
            return Some(Indices {
                iter: arg.indices.iter().map(to_usize),
            });
        }
        None
    }

    /// Because [`Subcommand`]s are essentially "sub-[`App`]s" they have their own [`ArgMatches`]
    /// as well. This method returns the [`ArgMatches`] for a particular subcommand or `None` if
    /// the subcommand wasn't present at runtime.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg, SubCommand};
    /// let app_m = App::new("myprog")
    ///     .arg(Arg::with_name("debug")
    ///         .short("d"))
    ///     .subcommand(SubCommand::with_name("test")
    ///         .arg(Arg::with_name("opt")
    ///             .long("option")
    ///             .takes_value(true)))
    ///     .get_matches_from(vec![
    ///         "myprog", "-d", "test", "--option", "val"
    ///     ]);
    ///
    /// // Both parent commands, and child subcommands can have arguments present at the same times
    /// assert!(app_m.is_present("debug"));
    ///
    /// // Get the subcommand's ArgMatches instance
    /// if let Some(sub_m) = app_m.subcommand_matches("test") {
    ///     // Use the struct like normal
    ///     assert_eq!(sub_m.value_of("opt"), Some("val"));
    /// }
    /// ```
    /// [`Subcommand`]: ./struct.SubCommand.html
    /// [`App`]: ./struct.App.html
    /// [`ArgMatches`]: ./struct.ArgMatches.html
    pub fn subcommand_matches<S: AsRef<str>>(&self, name: S) -> Option<&ArgMatches<'a>> {
        if let Some(ref s) = self.subcommand {
            if s.name == name.as_ref() {
                return Some(&s.matches);
            }
        }
        None
    }

    /// Because [`Subcommand`]s are essentially "sub-[`App`]s" they have their own [`ArgMatches`]
    /// as well.But simply getting the sub-[`ArgMatches`] doesn't help much if we don't also know
    /// which subcommand was actually used. This method returns the name of the subcommand that was
    /// used at runtime, or `None` if one wasn't.
    ///
    /// *NOTE*: Subcommands form a hierarchy, where multiple subcommands can be used at runtime,
    /// but only a single subcommand from any group of sibling commands may used at once.
    ///
    /// An ASCII art depiction may help explain this better...Using a fictional version of `git` as
    /// the demo subject. Imagine the following are all subcommands of `git` (note, the author is
    /// aware these aren't actually all subcommands in the real `git` interface, but it makes
    /// explanation easier)
    ///
    /// ```notrust
    ///              Top Level App (git)                         TOP
    ///                              |
    ///       -----------------------------------------
    ///      /             |                \          \
    ///   clone          push              add       commit      LEVEL 1
    ///     |           /    \            /    \       |
    ///    url      origin   remote    ref    name   message     LEVEL 2
    ///             /                  /\
    ///          path            remote  local                   LEVEL 3
    /// ```
    ///
    /// Given the above fictional subcommand hierarchy, valid runtime uses would be (not an all
    /// inclusive list, and not including argument options per command for brevity and clarity):
    ///
    /// ```sh
    /// $ git clone url
    /// $ git push origin path
    /// $ git add ref local
    /// $ git commit message
    /// ```
    ///
    /// Notice only one command per "level" may be used. You could not, for example, do `$ git
    /// clone url push origin path`
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{App, Arg, SubCommand};
    ///  let app_m = App::new("git")
    ///      .subcommand(SubCommand::with_name("clone"))
    ///      .subcommand(SubCommand::with_name("push"))
    ///      .subcommand(SubCommand::with_name("commit"))
    ///      .get_matches();
    ///
    /// match app_m.subcommand_name() {
    ///     Some("clone")  => {}, // clone was used
    ///     Some("push")   => {}, // push was used
    ///     Some("commit") => {}, // commit was used
    ///     _              => {}, // Either no subcommand or one not tested for...
    /// }
    /// ```
    /// [`Subcommand`]: ./struct.SubCommand.html
    /// [`App`]: ./struct.App.html
    /// [`ArgMatches`]: ./struct.ArgMatches.html
    pub fn subcommand_name(&self) -> Option<&str> {
        self.subcommand.as_ref().map(|sc| &sc.name[..])
    }

    /// This brings together [`ArgMatches::subcommand_matches`] and [`ArgMatches::subcommand_name`]
    /// by returning a tuple with both pieces of information.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{App, Arg, SubCommand};
    ///  let app_m = App::new("git")
    ///      .subcommand(SubCommand::with_name("clone"))
    ///      .subcommand(SubCommand::with_name("push"))
    ///      .subcommand(SubCommand::with_name("commit"))
    ///      .get_matches();
    ///
    /// match app_m.subcommand() {
    ///     ("clone",  Some(sub_m)) => {}, // clone was used
    ///     ("push",   Some(sub_m)) => {}, // push was used
    ///     ("commit", Some(sub_m)) => {}, // commit was used
    ///     _                       => {}, // Either no subcommand or one not tested for...
    /// }
    /// ```
    ///
    /// Another useful scenario is when you want to support third party, or external, subcommands.
    /// In these cases you can't know the subcommand name ahead of time, so use a variable instead
    /// with pattern matching!
    ///
    /// ```rust
    /// # use clap::{App, AppSettings};
    /// // Assume there is an external subcommand named "subcmd"
    /// let app_m = App::new("myprog")
    ///     .setting(AppSettings::AllowExternalSubcommands)
    ///     .get_matches_from(vec![
    ///         "myprog", "subcmd", "--option", "value", "-fff", "--flag"
    ///     ]);
    ///
    /// // All trailing arguments will be stored under the subcommand's sub-matches using an empty
    /// // string argument name
    /// match app_m.subcommand() {
    ///     (external, Some(sub_m)) => {
    ///          let ext_args: Vec<&str> = sub_m.values_of("").unwrap().collect();
    ///          assert_eq!(external, "subcmd");
    ///          assert_eq!(ext_args, ["--option", "value", "-fff", "--flag"]);
    ///     },
    ///     _ => {},
    /// }
    /// ```
    /// [`ArgMatches::subcommand_matches`]: ./struct.ArgMatches.html#method.subcommand_matches
    /// [`ArgMatches::subcommand_name`]: ./struct.ArgMatches.html#method.subcommand_name
    pub fn subcommand(&self) -> (&str, Option<&ArgMatches<'a>>) {
        self.subcommand
            .as_ref()
            .map_or(("", None), |sc| (&sc.name[..], Some(&sc.matches)))
    }

    /// Returns a string slice of the usage statement for the [`App`] or [`SubCommand`]
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{App, Arg, SubCommand};
    /// let app_m = App::new("myprog")
    ///     .subcommand(SubCommand::with_name("test"))
    ///     .get_matches();
    ///
    /// println!("{}", app_m.usage());
    /// ```
    /// [`Subcommand`]: ./struct.SubCommand.html
    /// [`App`]: ./struct.App.html
    pub fn usage(&self) -> &str {
        self.usage.as_ref().map_or("", |u| &u[..])
    }
}

// The following were taken and adapated from vec_map source
// repo: https://github.com/contain-rs/vec-map
// commit: be5e1fa3c26e351761b33010ddbdaf5f05dbcc33
// license: MIT - Copyright (c) 2015 The Rust Project Developers

/// An iterator for getting multiple values out of an argument via the [`ArgMatches::values_of`]
/// method.
///
/// # Examples
///
/// ```rust
/// # use clap::{App, Arg};
/// let m = App::new("myapp")
///     .arg(Arg::with_name("output")
///         .short("o")
///         .multiple(true)
///         .takes_value(true))
///     .get_matches_from(vec!["myapp", "-o", "val1", "val2"]);
///
/// let mut values = m.values_of("output").unwrap();
///
/// assert_eq!(values.next(), Some("val1"));
/// assert_eq!(values.next(), Some("val2"));
/// assert_eq!(values.next(), None);
/// ```
/// [`ArgMatches::values_of`]: ./struct.ArgMatches.html#method.values_of
#[derive(Debug, Clone)]
pub struct Values<'a> {
    iter: Map<Iter<'a, OsString>, fn(&'a OsString) -> &'a str>,
}

impl<'a> Iterator for Values<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        self.iter.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a> DoubleEndedIterator for Values<'a> {
    fn next_back(&mut self) -> Option<&'a str> {
        self.iter.next_back()
    }
}

impl<'a> ExactSizeIterator for Values<'a> {}

/// Creates an empty iterator.
impl<'a> Default for Values<'a> {
    fn default() -> Self {
        static EMPTY: [OsString; 0] = [];
        // This is never called because the iterator is empty:
        fn to_str_slice(_: &OsString) -> &str {
            unreachable!()
        }
        Values {
            iter: EMPTY[..].iter().map(to_str_slice),
        }
    }
}

/// An iterator for getting multiple values out of an argument via the [`ArgMatches::values_of_os`]
/// method. Usage of this iterator allows values which contain invalid UTF-8 code points unlike
/// [`Values`].
///
/// # Examples
///
#[cfg_attr(not(unix), doc = " ```ignore")]
#[cfg_attr(unix, doc = " ```")]
/// # use clap::{App, Arg};
/// use std::ffi::OsString;
/// use std::os::unix::ffi::{OsStrExt,OsStringExt};
///
/// let m = App::new("utf8")
///     .arg(Arg::from_usage("<arg> 'some arg'"))
///     .get_matches_from(vec![OsString::from("myprog"),
///                             // "Hi {0xe9}!"
///                             OsString::from_vec(vec![b'H', b'i', b' ', 0xe9, b'!'])]);
/// assert_eq!(&*m.value_of_os("arg").unwrap().as_bytes(), [b'H', b'i', b' ', 0xe9, b'!']);
/// ```
/// [`ArgMatches::values_of_os`]: ./struct.ArgMatches.html#method.values_of_os
/// [`Values`]: ./struct.Values.html
#[derive(Debug, Clone)]
pub struct OsValues<'a> {
    iter: Map<Iter<'a, OsString>, fn(&'a OsString) -> &'a OsStr>,
}

impl<'a> Iterator for OsValues<'a> {
    type Item = &'a OsStr;

    fn next(&mut self) -> Option<&'a OsStr> {
        self.iter.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a> DoubleEndedIterator for OsValues<'a> {
    fn next_back(&mut self) -> Option<&'a OsStr> {
        self.iter.next_back()
    }
}

impl<'a> ExactSizeIterator for OsValues<'a> {}

/// Creates an empty iterator.
impl<'a> Default for OsValues<'a> {
    fn default() -> Self {
        static EMPTY: [OsString; 0] = [];
        // This is never called because the iterator is empty:
        fn to_str_slice(_: &OsString) -> &OsStr {
            unreachable!()
        }
        OsValues {
            iter: EMPTY[..].iter().map(to_str_slice),
        }
    }
}

/// An iterator for getting multiple indices out of an argument via the [`ArgMatches::indices_of`]
/// method.
///
/// # Examples
///
/// ```rust
/// # use clap::{App, Arg};
/// let m = App::new("myapp")
///     .arg(Arg::with_name("output")
///         .short("o")
///         .multiple(true)
///         .takes_value(true))
///     .get_matches_from(vec!["myapp", "-o", "val1", "val2"]);
///
/// let mut indices = m.indices_of("output").unwrap();
///
/// assert_eq!(indices.next(), Some(2));
/// assert_eq!(indices.next(), Some(3));
/// assert_eq!(indices.next(), None);
/// ```
/// [`ArgMatches::indices_of`]: ./struct.ArgMatches.html#method.indices_of
#[derive(Debug, Clone)]
pub struct Indices<'a> {
    // would rather use '_, but: https://github.com/rust-lang/rust/issues/48469
    iter: Map<Iter<'a, usize>, fn(&'a usize) -> usize>,
}

impl<'a> Iterator for Indices<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        self.iter.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a> DoubleEndedIterator for Indices<'a> {
    fn next_back(&mut self) -> Option<usize> {
        self.iter.next_back()
    }
}

impl<'a> ExactSizeIterator for Indices<'a> {}

/// Creates an empty iterator.
impl<'a> Default for Indices<'a> {
    fn default() -> Self {
        static EMPTY: [usize; 0] = [];
        // This is never called because the iterator is empty:
        fn to_usize(_: &usize) -> usize {
            unreachable!()
        }
        Indices {
            iter: EMPTY[..].iter().map(to_usize),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        let mut values: Values = Values::default();
        assert_eq!(values.next(), None);
    }

    #[test]
    fn test_default_values_with_shorter_lifetime() {
        let matches = ArgMatches::new();
        let mut values = matches.values_of("").unwrap_or_default();
        assert_eq!(values.next(), None);
    }

    #[test]
    fn test_default_osvalues() {
        let mut values: OsValues = OsValues::default();
        assert_eq!(values.next(), None);
    }

    #[test]
    fn test_default_osvalues_with_shorter_lifetime() {
        let matches = ArgMatches::new();
        let mut values = matches.values_of_os("").unwrap_or_default();
        assert_eq!(values.next(), None);
    }

    #[test]
    fn test_default_indices() {
        let mut indices: Indices = Indices::default();
        assert_eq!(indices.next(), None);
    }

    #[test]
    fn test_default_indices_with_shorter_lifetime() {
        let matches = ArgMatches::new();
        let mut indices = matches.indices_of("").unwrap_or_default();
        assert_eq!(indices.next(), None);
    }
}
