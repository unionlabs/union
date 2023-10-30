//! Complete commands within shells

/// Complete commands within bash
pub mod bash {
    use std::ffi::OsStr;
    use std::ffi::OsString;
    use std::io::Write;

    use clap_lex::OsStrExt as _;
    use unicode_xid::UnicodeXID;

    #[derive(clap::Subcommand)]
    #[command(hide = true)]
    #[allow(missing_docs)]
    #[derive(Clone, Debug)]
    pub enum CompleteCommand {
        /// Register shell completions for this program
        Complete(CompleteArgs),
    }

    #[derive(clap::Args)]
    #[command(group = clap::ArgGroup::new("complete").multiple(true).conflicts_with("register"))]
    #[allow(missing_docs)]
    #[derive(Clone, Debug)]
    pub struct CompleteArgs {
        /// Path to write completion-registration to
        #[arg(long, required = true)]
        register: Option<std::path::PathBuf>,

        #[arg(
            long,
            required = true,
            value_name = "COMP_CWORD",
            hide_short_help = true,
            group = "complete"
        )]
        index: Option<usize>,

        #[arg(long, hide_short_help = true, group = "complete")]
        ifs: Option<String>,

        #[arg(
            long = "type",
            required = true,
            hide_short_help = true,
            group = "complete"
        )]
        comp_type: Option<CompType>,

        #[arg(long, hide_short_help = true, group = "complete")]
        space: bool,

        #[arg(
            long,
            conflicts_with = "space",
            hide_short_help = true,
            group = "complete"
        )]
        no_space: bool,

        #[arg(raw = true, hide_short_help = true, group = "complete")]
        comp_words: Vec<OsString>,
    }

    impl CompleteCommand {
        /// Process the completion request
        pub fn complete(&self, cmd: &mut clap::Command) -> std::convert::Infallible {
            self.try_complete(cmd).unwrap_or_else(|e| e.exit());
            std::process::exit(0)
        }

        /// Process the completion request
        pub fn try_complete(&self, cmd: &mut clap::Command) -> clap::error::Result<()> {
            debug!("CompleteCommand::try_complete: {self:?}");
            let CompleteCommand::Complete(args) = self;
            if let Some(out_path) = args.register.as_deref() {
                let mut buf = Vec::new();
                let name = cmd.get_name();
                let bin = cmd.get_bin_name().unwrap_or_else(|| cmd.get_name());
                register(name, [bin], bin, &Behavior::default(), &mut buf)?;
                if out_path == std::path::Path::new("-") {
                    std::io::stdout().write_all(&buf)?;
                } else if out_path.is_dir() {
                    let out_path = out_path.join(file_name(name));
                    std::fs::write(out_path, buf)?;
                } else {
                    std::fs::write(out_path, buf)?;
                }
            } else {
                let index = args.index.unwrap_or_default();
                let comp_type = args.comp_type.unwrap_or_default();
                let space = match (args.space, args.no_space) {
                    (true, false) => Some(true),
                    (false, true) => Some(false),
                    (true, true) => {
                        unreachable!("`--space` and `--no-space` set, clap should prevent this")
                    }
                    (false, false) => None,
                }
                .unwrap();
                let current_dir = std::env::current_dir().ok();
                let completions = complete(
                    cmd,
                    args.comp_words.clone(),
                    index,
                    comp_type,
                    space,
                    current_dir.as_deref(),
                )?;

                let mut buf = Vec::new();
                for (i, completion) in completions.iter().enumerate() {
                    if i != 0 {
                        write!(&mut buf, "{}", args.ifs.as_deref().unwrap_or("\n"))?;
                    }
                    write!(&mut buf, "{}", completion.to_string_lossy())?;
                }
                std::io::stdout().write_all(&buf)?;
            }

            Ok(())
        }
    }

    /// The recommended file name for the registration code
    pub fn file_name(name: &str) -> String {
        format!("{name}.bash")
    }

    /// Define the completion behavior
    pub enum Behavior {
        /// Bare bones behavior
        Minimal,
        /// Fallback to readline behavior when no matches are generated
        Readline,
        /// Customize bash's completion behavior
        Custom(String),
    }

    impl Default for Behavior {
        fn default() -> Self {
            Self::Readline
        }
    }

    /// Generate code to register the dynamic completion
    pub fn register(
        name: &str,
        executables: impl IntoIterator<Item = impl AsRef<str>>,
        completer: &str,
        behavior: &Behavior,
        buf: &mut dyn Write,
    ) -> Result<(), std::io::Error> {
        let escaped_name = name.replace('-', "_");
        debug_assert!(
            escaped_name.chars().all(|c| c.is_xid_continue()),
            "`name` must be an identifier, got `{escaped_name}`"
        );
        let mut upper_name = escaped_name.clone();
        upper_name.make_ascii_uppercase();

        let executables = executables
            .into_iter()
            .map(|s| shlex::quote(s.as_ref()).into_owned())
            .collect::<Vec<_>>()
            .join(" ");

        let options = match behavior {
            Behavior::Minimal => "-o nospace -o bashdefault",
            Behavior::Readline => "-o nospace -o default -o bashdefault",
            Behavior::Custom(c) => c.as_str(),
        };

        let completer = shlex::quote(completer);

        let script = r#"
_clap_complete_NAME() {
    local IFS=$'\013'
    local SUPPRESS_SPACE=0
    if compopt +o nospace 2> /dev/null; then
        SUPPRESS_SPACE=1
    fi
    if [[ ${SUPPRESS_SPACE} == 1 ]]; then
        SPACE_ARG="--no-space"
    else
        SPACE_ARG="--space"
    fi
    COMPREPLY=( $("COMPLETER" complete --index ${COMP_CWORD} --type ${COMP_TYPE} ${SPACE_ARG} --ifs="$IFS" -- "${COMP_WORDS[@]}") )
    if [[ $? != 0 ]]; then
        unset COMPREPLY
    elif [[ $SUPPRESS_SPACE == 1 ]] && [[ "${COMPREPLY-}" =~ [=/:]$ ]]; then
        compopt -o nospace
    fi
}
complete OPTIONS -F _clap_complete_NAME EXECUTABLES
"#
        .replace("NAME", &escaped_name)
        .replace("EXECUTABLES", &executables)
        .replace("OPTIONS", options)
        .replace("COMPLETER", &completer)
        .replace("UPPER", &upper_name);

        writeln!(buf, "{script}")?;
        Ok(())
    }

    /// Type of completion attempted that caused a completion function to be called
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    #[non_exhaustive]
    pub enum CompType {
        /// Normal completion
        Normal,
        /// List completions after successive tabs
        Successive,
        /// List alternatives on partial word completion
        Alternatives,
        /// List completions if the word is not unmodified
        Unmodified,
        /// Menu completion
        Menu,
    }

    impl clap::ValueEnum for CompType {
        fn value_variants<'a>() -> &'a [Self] {
            &[
                Self::Normal,
                Self::Successive,
                Self::Alternatives,
                Self::Unmodified,
                Self::Menu,
            ]
        }
        fn to_possible_value(&self) -> ::std::option::Option<clap::builder::PossibleValue> {
            match self {
                Self::Normal => {
                    let value = "9";
                    debug_assert_eq!(b'\t'.to_string(), value);
                    Some(
                        clap::builder::PossibleValue::new(value)
                            .alias("normal")
                            .help("Normal completion"),
                    )
                }
                Self::Successive => {
                    let value = "63";
                    debug_assert_eq!(b'?'.to_string(), value);
                    Some(
                        clap::builder::PossibleValue::new(value)
                            .alias("successive")
                            .help("List completions after successive tabs"),
                    )
                }
                Self::Alternatives => {
                    let value = "33";
                    debug_assert_eq!(b'!'.to_string(), value);
                    Some(
                        clap::builder::PossibleValue::new(value)
                            .alias("alternatives")
                            .help("List alternatives on partial word completion"),
                    )
                }
                Self::Unmodified => {
                    let value = "64";
                    debug_assert_eq!(b'@'.to_string(), value);
                    Some(
                        clap::builder::PossibleValue::new(value)
                            .alias("unmodified")
                            .help("List completions if the word is not unmodified"),
                    )
                }
                Self::Menu => {
                    let value = "37";
                    debug_assert_eq!(b'%'.to_string(), value);
                    Some(
                        clap::builder::PossibleValue::new(value)
                            .alias("menu")
                            .help("Menu completion"),
                    )
                }
            }
        }
    }

    impl Default for CompType {
        fn default() -> Self {
            Self::Normal
        }
    }

    /// Complete the command specified
    pub fn complete(
        cmd: &mut clap::Command,
        args: Vec<std::ffi::OsString>,
        arg_index: usize,
        _comp_type: CompType,
        _trailing_space: bool,
        current_dir: Option<&std::path::Path>,
    ) -> Result<Vec<std::ffi::OsString>, std::io::Error> {
        cmd.build();

        let raw_args = clap_lex::RawArgs::new(args.into_iter());
        let mut cursor = raw_args.cursor();
        let mut target_cursor = raw_args.cursor();
        raw_args.seek(
            &mut target_cursor,
            clap_lex::SeekFrom::Start(arg_index as u64),
        );
        // As we loop, `cursor` will always be pointing to the next item
        raw_args.next_os(&mut target_cursor);

        // TODO: Multicall support
        if !cmd.is_no_binary_name_set() {
            raw_args.next_os(&mut cursor);
        }

        let mut current_cmd = &*cmd;
        let mut pos_index = 1;
        let mut is_escaped = false;
        while let Some(arg) = raw_args.next(&mut cursor) {
            if cursor == target_cursor {
                return complete_arg(&arg, current_cmd, current_dir, pos_index, is_escaped);
            }

            debug!("complete::next: Begin parsing '{:?}'", arg.to_value_os(),);

            if let Ok(value) = arg.to_value() {
                if let Some(next_cmd) = current_cmd.find_subcommand(value) {
                    current_cmd = next_cmd;
                    pos_index = 0;
                    continue;
                }
            }

            if is_escaped {
                pos_index += 1;
            } else if arg.is_escape() {
                is_escaped = true;
            } else if let Some(_long) = arg.to_long() {
            } else if let Some(_short) = arg.to_short() {
            } else {
                pos_index += 1;
            }
        }

        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "No completion generated",
        ))
    }

    fn complete_arg(
        arg: &clap_lex::ParsedArg<'_>,
        cmd: &clap::Command,
        current_dir: Option<&std::path::Path>,
        pos_index: usize,
        is_escaped: bool,
    ) -> Result<Vec<std::ffi::OsString>, std::io::Error> {
        debug!(
            "complete_arg: arg={:?}, cmd={:?}, current_dir={:?}, pos_index={}, is_escaped={}",
            arg,
            cmd.get_name(),
            current_dir,
            pos_index,
            is_escaped
        );
        let mut completions = Vec::new();

        if !is_escaped {
            if let Some((flag, value)) = arg.to_long() {
                if let Ok(flag) = flag {
                    if let Some(value) = value {
                        if let Some(arg) = cmd.get_arguments().find(|a| a.get_long() == Some(flag))
                        {
                            completions.extend(
                                complete_arg_value(value.to_str().ok_or(value), arg, current_dir)
                                    .into_iter()
                                    .map(|os| {
                                        // HACK: Need better `OsStr` manipulation
                                        format!("--{}={}", flag, os.to_string_lossy()).into()
                                    }),
                            )
                        }
                    } else {
                        completions.extend(
                            crate::generator::utils::longs_and_visible_aliases(cmd)
                                .into_iter()
                                .filter_map(|f| {
                                    f.starts_with(flag).then(|| format!("--{f}").into())
                                }),
                        );
                    }
                }
            } else if arg.is_escape() || arg.is_stdio() || arg.is_empty() {
                // HACK: Assuming knowledge of is_escape / is_stdio
                completions.extend(
                    crate::generator::utils::longs_and_visible_aliases(cmd)
                        .into_iter()
                        .map(|f| format!("--{f}").into()),
                );
            }

            if arg.is_empty() || arg.is_stdio() || arg.is_short() {
                // HACK: Assuming knowledge of is_stdio
                completions.extend(
                    crate::generator::utils::shorts_and_visible_aliases(cmd)
                        .into_iter()
                        // HACK: Need better `OsStr` manipulation
                        .map(|f| format!("{}{}", arg.to_value_os().to_string_lossy(), f).into()),
                );
            }
        }

        if let Some(positional) = cmd
            .get_positionals()
            .find(|p| p.get_index() == Some(pos_index))
        {
            completions.extend(complete_arg_value(arg.to_value(), positional, current_dir));
        }

        if let Ok(value) = arg.to_value() {
            completions.extend(complete_subcommand(value, cmd));
        }

        Ok(completions)
    }

    fn complete_arg_value(
        value: Result<&str, &OsStr>,
        arg: &clap::Arg,
        current_dir: Option<&std::path::Path>,
    ) -> Vec<OsString> {
        let mut values = Vec::new();
        debug!("complete_arg_value: arg={arg:?}, value={value:?}");

        if let Some(possible_values) = crate::generator::utils::possible_values(arg) {
            if let Ok(value) = value {
                values.extend(possible_values.into_iter().filter_map(|p| {
                    let name = p.get_name();
                    name.starts_with(value).then(|| name.into())
                }));
            }
        } else {
            let value_os = match value {
                Ok(value) => OsStr::new(value),
                Err(value_os) => value_os,
            };
            match arg.get_value_hint() {
                clap::ValueHint::Other => {
                    // Should not complete
                }
                clap::ValueHint::Unknown | clap::ValueHint::AnyPath => {
                    values.extend(complete_path(value_os, current_dir, |_| true));
                }
                clap::ValueHint::FilePath => {
                    values.extend(complete_path(value_os, current_dir, |p| p.is_file()));
                }
                clap::ValueHint::DirPath => {
                    values.extend(complete_path(value_os, current_dir, |p| p.is_dir()));
                }
                clap::ValueHint::ExecutablePath => {
                    use is_executable::IsExecutable;
                    values.extend(complete_path(value_os, current_dir, |p| p.is_executable()));
                }
                clap::ValueHint::CommandName
                | clap::ValueHint::CommandString
                | clap::ValueHint::CommandWithArguments
                | clap::ValueHint::Username
                | clap::ValueHint::Hostname
                | clap::ValueHint::Url
                | clap::ValueHint::EmailAddress => {
                    // No completion implementation
                }
                _ => {
                    // Safe-ish fallback
                    values.extend(complete_path(value_os, current_dir, |_| true));
                }
            }
            values.sort();
        }

        values
    }

    fn complete_path(
        value_os: &OsStr,
        current_dir: Option<&std::path::Path>,
        is_wanted: impl Fn(&std::path::Path) -> bool,
    ) -> Vec<OsString> {
        let mut completions = Vec::new();

        let current_dir = match current_dir {
            Some(current_dir) => current_dir,
            None => {
                // Can't complete without a `current_dir`
                return Vec::new();
            }
        };
        let (existing, prefix) = value_os
            .split_once("\\")
            .unwrap_or((OsStr::new(""), value_os));
        let root = current_dir.join(existing);
        debug!("complete_path: root={root:?}, prefix={prefix:?}");
        let prefix = prefix.to_string_lossy();

        for entry in std::fs::read_dir(&root)
            .ok()
            .into_iter()
            .flatten()
            .filter_map(Result::ok)
        {
            let raw_file_name = OsString::from(entry.file_name());
            if !raw_file_name.starts_with(&prefix) {
                continue;
            }

            if entry.metadata().map(|m| m.is_dir()).unwrap_or(false) {
                let path = entry.path();
                let mut suggestion = pathdiff::diff_paths(&path, current_dir).unwrap_or(path);
                suggestion.push(""); // Ensure trailing `/`
                completions.push(suggestion.as_os_str().to_owned());
            } else {
                let path = entry.path();
                if is_wanted(&path) {
                    let suggestion = pathdiff::diff_paths(&path, current_dir).unwrap_or(path);
                    completions.push(suggestion.as_os_str().to_owned());
                }
            }
        }

        completions
    }

    fn complete_subcommand(value: &str, cmd: &clap::Command) -> Vec<OsString> {
        debug!(
            "complete_subcommand: cmd={:?}, value={:?}",
            cmd.get_name(),
            value
        );

        let mut scs = crate::generator::utils::all_subcommands(cmd)
            .into_iter()
            .filter(|x| x.0.starts_with(value))
            .map(|x| OsString::from(&x.0))
            .collect::<Vec<_>>();
        scs.sort();
        scs.dedup();
        scs
    }
}
