#[cfg(feature = "yaml")]
use std::collections::BTreeMap;
use std::fmt::{Debug, Formatter, Result};

#[cfg(feature = "yaml")]
use yaml_rust::Yaml;

/// `ArgGroup`s are a family of related [arguments] and way for you to express, "Any of these
/// arguments". By placing arguments in a logical group, you can create easier requirement and
/// exclusion rules instead of having to list each argument individually, or when you want a rule
/// to apply "any but not all" arguments.
///
/// For instance, you can make an entire `ArgGroup` required. If [`ArgGroup::multiple(true)`] is
/// set, this means that at least one argument from that group must be present. If
/// [`ArgGroup::multiple(false)`] is set (the default), one and *only* one must be present.
///
/// You can also do things such as name an entire `ArgGroup` as a [conflict] or [requirement] for
/// another argument, meaning any of the arguments that belong to that group will cause a failure
/// if present, or must present respectively.
///
/// Perhaps the most common use of `ArgGroup`s is to require one and *only* one argument to be
/// present out of a given set. Imagine that you had multiple arguments, and you want one of them
/// to be required, but making all of them required isn't feasible because perhaps they conflict
/// with each other. For example, lets say that you were building an application where one could
/// set a given version number by supplying a string with an option argument, i.e.
/// `--set-ver v1.2.3`, you also wanted to support automatically using a previous version number
/// and simply incrementing one of the three numbers. So you create three flags `--major`,
/// `--minor`, and `--patch`. All of these arguments shouldn't be used at one time but you want to
/// specify that *at least one* of them is used. For this, you can create a group.
///
/// Finally, you may use `ArgGroup`s to pull a value from a group of arguments when you don't care
/// exactly which argument was actually used at runtime.
///
/// # Examples
///
/// The following example demonstrates using an `ArgGroup` to ensure that one, and only one, of
/// the arguments from the specified group is present at runtime.
///
/// ```rust
/// # use clap::{App, ArgGroup, ErrorKind};
/// let result = App::new("app")
///     .args_from_usage(
///         "--set-ver [ver] 'set the version manually'
///          --major         'auto increase major'
///          --minor         'auto increase minor'
///          --patch         'auto increase patch'")
///     .group(ArgGroup::with_name("vers")
///          .args(&["set-ver", "major", "minor", "patch"])
///          .required(true))
///     .get_matches_from_safe(vec!["app", "--major", "--patch"]);
/// // Because we used two args in the group it's an error
/// assert!(result.is_err());
/// let err = result.unwrap_err();
/// assert_eq!(err.kind, ErrorKind::ArgumentConflict);
/// ```
/// This next example shows a passing parse of the same scenario
///
/// ```rust
/// # use clap::{App, ArgGroup};
/// let result = App::new("app")
///     .args_from_usage(
///         "--set-ver [ver] 'set the version manually'
///          --major         'auto increase major'
///          --minor         'auto increase minor'
///          --patch         'auto increase patch'")
///     .group(ArgGroup::with_name("vers")
///          .args(&["set-ver", "major", "minor","patch"])
///          .required(true))
///     .get_matches_from_safe(vec!["app", "--major"]);
/// assert!(result.is_ok());
/// let matches = result.unwrap();
/// // We may not know which of the args was used, so we can test for the group...
/// assert!(matches.is_present("vers"));
/// // we could also alternatively check each arg individually (not shown here)
/// ```
/// [`ArgGroup::multiple(true)`]: ./struct.ArgGroup.html#method.multiple
/// [arguments]: ./struct.Arg.html
/// [conflict]: ./struct.Arg.html#method.conflicts_with
/// [requirement]: ./struct.Arg.html#method.requires
#[derive(Default)]
pub struct ArgGroup<'a> {
    #[doc(hidden)]
    pub name: &'a str,
    #[doc(hidden)]
    pub args: Vec<&'a str>,
    #[doc(hidden)]
    pub required: bool,
    #[doc(hidden)]
    pub requires: Option<Vec<&'a str>>,
    #[doc(hidden)]
    pub conflicts: Option<Vec<&'a str>>,
    #[doc(hidden)]
    pub multiple: bool,
}

impl<'a> ArgGroup<'a> {
    /// Creates a new instance of `ArgGroup` using a unique string name. The name will be used to
    /// get values from the group or refer to the group inside of conflict and requirement rules.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, ArgGroup};
    /// ArgGroup::with_name("config")
    /// # ;
    /// ```
    pub fn with_name(n: &'a str) -> Self {
        ArgGroup {
            name: n,
            required: false,
            args: vec![],
            requires: None,
            conflicts: None,
            multiple: false,
        }
    }

    /// Creates a new instance of `ArgGroup` from a .yml (YAML) file.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # #[macro_use]
    /// # extern crate clap;
    /// # use clap::ArgGroup;
    /// # fn main() {
    /// let yml = load_yaml!("group.yml");
    /// let ag = ArgGroup::from_yaml(yml);
    /// # }
    /// ```
    #[cfg(feature = "yaml")]
    pub fn from_yaml(y: &'a Yaml) -> ArgGroup<'a> {
        ArgGroup::from(y.as_hash().unwrap())
    }

    /// Adds an [argument] to this group by name
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg, ArgGroup};
    /// let m = App::new("myprog")
    ///     .arg(Arg::with_name("flag")
    ///         .short("f"))
    ///     .arg(Arg::with_name("color")
    ///         .short("c"))
    ///     .group(ArgGroup::with_name("req_flags")
    ///         .arg("flag")
    ///         .arg("color"))
    ///     .get_matches_from(vec!["myprog", "-f"]);
    /// // maybe we don't know which of the two flags was used...
    /// assert!(m.is_present("req_flags"));
    /// // but we can also check individually if needed
    /// assert!(m.is_present("flag"));
    /// ```
    /// [argument]: ./struct.Arg.html
    pub fn arg(mut self, n: &'a str) -> Self {
        assert!(
            self.name != n,
            "ArgGroup '{}' can not have same name as arg inside it",
            &*self.name
        );
        self.args.push(n);
        self
    }

    /// Adds multiple [arguments] to this group by name
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg, ArgGroup};
    /// let m = App::new("myprog")
    ///     .arg(Arg::with_name("flag")
    ///         .short("f"))
    ///     .arg(Arg::with_name("color")
    ///         .short("c"))
    ///     .group(ArgGroup::with_name("req_flags")
    ///         .args(&["flag", "color"]))
    ///     .get_matches_from(vec!["myprog", "-f"]);
    /// // maybe we don't know which of the two flags was used...
    /// assert!(m.is_present("req_flags"));
    /// // but we can also check individually if needed
    /// assert!(m.is_present("flag"));
    /// ```
    /// [arguments]: ./struct.Arg.html
    pub fn args(mut self, ns: &[&'a str]) -> Self {
        for n in ns {
            self = self.arg(n);
        }
        self
    }

    /// Allows more than one of the ['Arg']s in this group to be used. (Default: `false`)
    ///
    /// # Examples
    ///
    /// Notice in this example we use *both* the `-f` and `-c` flags which are both part of the
    /// group
    ///
    /// ```rust
    /// # use clap::{App, Arg, ArgGroup};
    /// let m = App::new("myprog")
    ///     .arg(Arg::with_name("flag")
    ///         .short("f"))
    ///     .arg(Arg::with_name("color")
    ///         .short("c"))
    ///     .group(ArgGroup::with_name("req_flags")
    ///         .args(&["flag", "color"])
    ///         .multiple(true))
    ///     .get_matches_from(vec!["myprog", "-f", "-c"]);
    /// // maybe we don't know which of the two flags was used...
    /// assert!(m.is_present("req_flags"));
    /// ```
    /// In this next example, we show the default behavior (i.e. `multiple(false)) which will throw
    /// an error if more than one of the args in the group was used.
    ///
    /// ```rust
    /// # use clap::{App, Arg, ArgGroup, ErrorKind};
    /// let result = App::new("myprog")
    ///     .arg(Arg::with_name("flag")
    ///         .short("f"))
    ///     .arg(Arg::with_name("color")
    ///         .short("c"))
    ///     .group(ArgGroup::with_name("req_flags")
    ///         .args(&["flag", "color"]))
    ///     .get_matches_from_safe(vec!["myprog", "-f", "-c"]);
    /// // Because we used both args in the group it's an error
    /// assert!(result.is_err());
    /// let err = result.unwrap_err();
    /// assert_eq!(err.kind, ErrorKind::ArgumentConflict);
    /// ```
    /// ['Arg']: ./struct.Arg.html
    pub fn multiple(mut self, m: bool) -> Self {
        self.multiple = m;
        self
    }

    /// Sets the group as required or not. A required group will be displayed in the usage string
    /// of the application in the format `<arg|arg2|arg3>`. A required `ArgGroup` simply states
    /// that one argument from this group *must* be present at runtime (unless
    /// conflicting with another argument).
    ///
    /// **NOTE:** This setting only applies to the current [`App`] / [`SubCommand`], and not
    /// globally.
    ///
    /// **NOTE:** By default, [`ArgGroup::multiple`] is set to `false` which when combined with
    /// `ArgGroup::required(true)` states, "One and *only one* arg must be used from this group.
    /// Use of more than one arg is an error." Vice setting `ArgGroup::multiple(true)` which
    /// states, '*At least* one arg from this group must be used. Using multiple is OK."
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg, ArgGroup, ErrorKind};
    /// let result = App::new("myprog")
    ///     .arg(Arg::with_name("flag")
    ///         .short("f"))
    ///     .arg(Arg::with_name("color")
    ///         .short("c"))
    ///     .group(ArgGroup::with_name("req_flags")
    ///         .args(&["flag", "color"])
    ///         .required(true))
    ///     .get_matches_from_safe(vec!["myprog"]);
    /// // Because we didn't use any of the args in the group, it's an error
    /// assert!(result.is_err());
    /// let err = result.unwrap_err();
    /// assert_eq!(err.kind, ErrorKind::MissingRequiredArgument);
    /// ```
    /// [`App`]: ./struct.App.html
    /// [`SubCommand`]: ./struct.SubCommand.html
    /// [`ArgGroup::multiple`]: ./struct.ArgGroup.html#method.multiple
    pub fn required(mut self, r: bool) -> Self {
        self.required = r;
        self
    }

    /// Sets the requirement rules of this group. This is not to be confused with a
    /// [required group]. Requirement rules function just like [argument requirement rules], you
    /// can name other arguments or groups that must be present when any one of the arguments from
    /// this group is used.
    ///
    /// **NOTE:** The name provided may be an argument, or group name
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg, ArgGroup, ErrorKind};
    /// let result = App::new("myprog")
    ///     .arg(Arg::with_name("flag")
    ///         .short("f"))
    ///     .arg(Arg::with_name("color")
    ///         .short("c"))
    ///     .arg(Arg::with_name("debug")
    ///         .short("d"))
    ///     .group(ArgGroup::with_name("req_flags")
    ///         .args(&["flag", "color"])
    ///         .requires("debug"))
    ///     .get_matches_from_safe(vec!["myprog", "-c"]);
    /// // because we used an arg from the group, and the group requires "-d" to be used, it's an
    /// // error
    /// assert!(result.is_err());
    /// let err = result.unwrap_err();
    /// assert_eq!(err.kind, ErrorKind::MissingRequiredArgument);
    /// ```
    /// [required group]: ./struct.ArgGroup.html#method.required
    /// [argument requirement rules]: ./struct.Arg.html#method.requires
    pub fn requires(mut self, n: &'a str) -> Self {
        if let Some(ref mut reqs) = self.requires {
            reqs.push(n);
        } else {
            self.requires = Some(vec![n]);
        }
        self
    }

    /// Sets the requirement rules of this group. This is not to be confused with a
    /// [required group]. Requirement rules function just like [argument requirement rules], you
    /// can name other arguments or groups that must be present when one of the arguments from this
    /// group is used.
    ///
    /// **NOTE:** The names provided may be an argument, or group name
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg, ArgGroup, ErrorKind};
    /// let result = App::new("myprog")
    ///     .arg(Arg::with_name("flag")
    ///         .short("f"))
    ///     .arg(Arg::with_name("color")
    ///         .short("c"))
    ///     .arg(Arg::with_name("debug")
    ///         .short("d"))
    ///     .arg(Arg::with_name("verb")
    ///         .short("v"))
    ///     .group(ArgGroup::with_name("req_flags")
    ///         .args(&["flag", "color"])
    ///         .requires_all(&["debug", "verb"]))
    ///     .get_matches_from_safe(vec!["myprog", "-c", "-d"]);
    /// // because we used an arg from the group, and the group requires "-d" and "-v" to be used,
    /// // yet we only used "-d" it's an error
    /// assert!(result.is_err());
    /// let err = result.unwrap_err();
    /// assert_eq!(err.kind, ErrorKind::MissingRequiredArgument);
    /// ```
    /// [required group]: ./struct.ArgGroup.html#method.required
    /// [argument requirement rules]: ./struct.Arg.html#method.requires_all
    pub fn requires_all(mut self, ns: &[&'a str]) -> Self {
        for n in ns {
            self = self.requires(n);
        }
        self
    }

    /// Sets the exclusion rules of this group. Exclusion (aka conflict) rules function just like
    /// [argument exclusion rules], you can name other arguments or groups that must *not* be
    /// present when one of the arguments from this group are used.
    ///
    /// **NOTE:** The name provided may be an argument, or group name
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg, ArgGroup, ErrorKind};
    /// let result = App::new("myprog")
    ///     .arg(Arg::with_name("flag")
    ///         .short("f"))
    ///     .arg(Arg::with_name("color")
    ///         .short("c"))
    ///     .arg(Arg::with_name("debug")
    ///         .short("d"))
    ///     .group(ArgGroup::with_name("req_flags")
    ///         .args(&["flag", "color"])
    ///         .conflicts_with("debug"))
    ///     .get_matches_from_safe(vec!["myprog", "-c", "-d"]);
    /// // because we used an arg from the group, and the group conflicts with "-d", it's an error
    /// assert!(result.is_err());
    /// let err = result.unwrap_err();
    /// assert_eq!(err.kind, ErrorKind::ArgumentConflict);
    /// ```
    /// [argument exclusion rules]: ./struct.Arg.html#method.conflicts_with
    pub fn conflicts_with(mut self, n: &'a str) -> Self {
        if let Some(ref mut confs) = self.conflicts {
            confs.push(n);
        } else {
            self.conflicts = Some(vec![n]);
        }
        self
    }

    /// Sets the exclusion rules of this group. Exclusion rules function just like
    /// [argument exclusion rules], you can name other arguments or groups that must *not* be
    /// present when one of the arguments from this group are used.
    ///
    /// **NOTE:** The names provided may be an argument, or group name
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg, ArgGroup, ErrorKind};
    /// let result = App::new("myprog")
    ///     .arg(Arg::with_name("flag")
    ///         .short("f"))
    ///     .arg(Arg::with_name("color")
    ///         .short("c"))
    ///     .arg(Arg::with_name("debug")
    ///         .short("d"))
    ///     .arg(Arg::with_name("verb")
    ///         .short("v"))
    ///     .group(ArgGroup::with_name("req_flags")
    ///         .args(&["flag", "color"])
    ///         .conflicts_with_all(&["debug", "verb"]))
    ///     .get_matches_from_safe(vec!["myprog", "-c", "-v"]);
    /// // because we used an arg from the group, and the group conflicts with either "-v" or "-d"
    /// // it's an error
    /// assert!(result.is_err());
    /// let err = result.unwrap_err();
    /// assert_eq!(err.kind, ErrorKind::ArgumentConflict);
    /// ```
    /// [argument exclusion rules]: ./struct.Arg.html#method.conflicts_with_all
    pub fn conflicts_with_all(mut self, ns: &[&'a str]) -> Self {
        for n in ns {
            self = self.conflicts_with(n);
        }
        self
    }
}

impl<'a> Debug for ArgGroup<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(
            f,
            "{{\n\
             \tname: {:?},\n\
             \targs: {:?},\n\
             \trequired: {:?},\n\
             \trequires: {:?},\n\
             \tconflicts: {:?},\n\
             }}",
            self.name, self.args, self.required, self.requires, self.conflicts
        )
    }
}

impl<'a, 'z> From<&'z ArgGroup<'a>> for ArgGroup<'a> {
    fn from(g: &'z ArgGroup<'a>) -> Self {
        ArgGroup {
            name: g.name,
            required: g.required,
            args: g.args.clone(),
            requires: g.requires.clone(),
            conflicts: g.conflicts.clone(),
            multiple: g.multiple,
        }
    }
}

#[cfg(feature = "yaml")]
impl<'a> From<&'a BTreeMap<Yaml, Yaml>> for ArgGroup<'a> {
    fn from(b: &'a BTreeMap<Yaml, Yaml>) -> Self {
        // We WANT this to panic on error...so expect() is good.
        let mut a = ArgGroup::default();
        let group_settings = if b.len() == 1 {
            let name_yml = b.keys().nth(0).expect("failed to get name");
            let name_str = name_yml
                .as_str()
                .expect("failed to convert arg YAML name to str");
            a.name = name_str;
            b.get(name_yml)
                .expect("failed to get name_str")
                .as_hash()
                .expect("failed to convert to a hash")
        } else {
            b
        };

        for (k, v) in group_settings {
            a = match k.as_str().unwrap() {
                "required" => a.required(v.as_bool().unwrap()),
                "multiple" => a.multiple(v.as_bool().unwrap()),
                "args" => yaml_vec_or_str!(v, a, arg),
                "arg" => {
                    if let Some(ys) = v.as_str() {
                        a = a.arg(ys);
                    }
                    a
                }
                "requires" => yaml_vec_or_str!(v, a, requires),
                "conflicts_with" => yaml_vec_or_str!(v, a, conflicts_with),
                "name" => {
                    if let Some(ys) = v.as_str() {
                        a.name = ys;
                    }
                    a
                }
                s => panic!(
                    "Unknown ArgGroup setting '{}' in YAML file for \
                     ArgGroup '{}'",
                    s, a.name
                ),
            }
        }

        a
    }
}

#[cfg(test)]
mod test {
    use super::ArgGroup;
    #[cfg(feature = "yaml")]
    use yaml_rust::YamlLoader;

    #[test]
    fn groups() {
        let g = ArgGroup::with_name("test")
            .arg("a1")
            .arg("a4")
            .args(&["a2", "a3"])
            .required(true)
            .conflicts_with("c1")
            .conflicts_with_all(&["c2", "c3"])
            .conflicts_with("c4")
            .requires("r1")
            .requires_all(&["r2", "r3"])
            .requires("r4");

        let args = vec!["a1", "a4", "a2", "a3"];
        let reqs = vec!["r1", "r2", "r3", "r4"];
        let confs = vec!["c1", "c2", "c3", "c4"];

        assert_eq!(g.args, args);
        assert_eq!(g.requires, Some(reqs));
        assert_eq!(g.conflicts, Some(confs));
    }

    #[test]
    fn test_debug() {
        let g = ArgGroup::with_name("test")
            .arg("a1")
            .arg("a4")
            .args(&["a2", "a3"])
            .required(true)
            .conflicts_with("c1")
            .conflicts_with_all(&["c2", "c3"])
            .conflicts_with("c4")
            .requires("r1")
            .requires_all(&["r2", "r3"])
            .requires("r4");

        let args = vec!["a1", "a4", "a2", "a3"];
        let reqs = vec!["r1", "r2", "r3", "r4"];
        let confs = vec!["c1", "c2", "c3", "c4"];

        let debug_str = format!(
            "{{\n\
             \tname: \"test\",\n\
             \targs: {:?},\n\
             \trequired: {:?},\n\
             \trequires: {:?},\n\
             \tconflicts: {:?},\n\
             }}",
            args,
            true,
            Some(reqs),
            Some(confs)
        );
        assert_eq!(&*format!("{:?}", g), &*debug_str);
    }

    #[test]
    fn test_from() {
        let g = ArgGroup::with_name("test")
            .arg("a1")
            .arg("a4")
            .args(&["a2", "a3"])
            .required(true)
            .conflicts_with("c1")
            .conflicts_with_all(&["c2", "c3"])
            .conflicts_with("c4")
            .requires("r1")
            .requires_all(&["r2", "r3"])
            .requires("r4");

        let args = vec!["a1", "a4", "a2", "a3"];
        let reqs = vec!["r1", "r2", "r3", "r4"];
        let confs = vec!["c1", "c2", "c3", "c4"];

        let g2 = ArgGroup::from(&g);
        assert_eq!(g2.args, args);
        assert_eq!(g2.requires, Some(reqs));
        assert_eq!(g2.conflicts, Some(confs));
    }

    #[cfg(feature = "yaml")]
    #[cfg_attr(feature = "yaml", test)]
    fn test_yaml() {
        let g_yaml = "name: test
args:
- a1
- a4
- a2
- a3
conflicts_with:
- c1
- c2
- c3
- c4
requires:
- r1
- r2
- r3
- r4";
        let yml = &YamlLoader::load_from_str(g_yaml).expect("failed to load YAML file")[0];
        let g = ArgGroup::from_yaml(yml);
        let args = vec!["a1", "a4", "a2", "a3"];
        let reqs = vec!["r1", "r2", "r3", "r4"];
        let confs = vec!["c1", "c2", "c3", "c4"];
        assert_eq!(g.args, args);
        assert_eq!(g.requires, Some(reqs));
        assert_eq!(g.conflicts, Some(confs));
    }
}

impl<'a> Clone for ArgGroup<'a> {
    fn clone(&self) -> Self {
        ArgGroup {
            name: self.name,
            required: self.required,
            args: self.args.clone(),
            requires: self.requires.clone(),
            conflicts: self.conflicts.clone(),
            multiple: self.multiple,
        }
    }
}
