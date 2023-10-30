use std::cmp::Ordering;

use clap_lex::RawOsStr;

use crate::builder::arg::ArgProvider;
use crate::mkeymap::KeyType;
use crate::ArgAction;
use crate::{Arg, Command, ValueHint};

pub(crate) fn assert_app(cmd: &Command) {
    debug!("Command::_debug_asserts");

    let mut short_flags = vec![];
    let mut long_flags = vec![];

    // Invalid version flag settings
    if cmd.get_version().is_none() && cmd.get_long_version().is_none() {
        // PropagateVersion is meaningless if there is no version
        assert!(
            !cmd.is_propagate_version_set(),
            "Command {}: No version information via Command::version or Command::long_version to propagate",
            cmd.get_name(),
        );

        // Used `Command::mut_arg("version", ..) but did not provide any version information to display
        let version_needed = cmd
            .get_arguments()
            .filter(|x| {
                let action_set = matches!(x.get_action(), ArgAction::Version);
                #[cfg(not(feature = "unstable-v4"))]
                let provider_set = matches!(x.provider, ArgProvider::GeneratedMutated);
                #[cfg(feature = "unstable-v4")]
                let provider_set = matches!(
                    x.provider,
                    ArgProvider::User | ArgProvider::GeneratedMutated
                );
                action_set && provider_set
            })
            .map(|x| x.get_id())
            .collect::<Vec<_>>();

        assert_eq!(version_needed, Vec::<&str>::new(), "Command {}: `ArgAction::Version` used without providing Command::version or Command::long_version"
            ,cmd.get_name()
        );
    }

    for sc in cmd.get_subcommands() {
        if let Some(s) = sc.get_short_flag().as_ref() {
            short_flags.push(Flag::Command(format!("-{}", s), sc.get_name()));
        }

        for short_alias in sc.get_all_short_flag_aliases() {
            short_flags.push(Flag::Command(format!("-{}", short_alias), sc.get_name()));
        }

        if let Some(l) = sc.get_long_flag().as_ref() {
            #[cfg(feature = "unstable-v4")]
            {
                assert!(!l.starts_with('-'), "Command {}: long_flag {:?} must not start with a `-`, that will be handled by the parser", sc.get_name(), l);
            }
            long_flags.push(Flag::Command(format!("--{}", l), sc.get_name()));
        }

        for long_alias in sc.get_all_long_flag_aliases() {
            long_flags.push(Flag::Command(format!("--{}", long_alias), sc.get_name()));
        }
    }

    for arg in cmd.get_arguments() {
        assert_arg(arg);

        assert!(
            !cmd.is_multicall_set(),
            "Command {}: Arguments like {} cannot be set on a multicall command",
            cmd.get_name(),
            arg.name
        );

        if let Some(s) = arg.short.as_ref() {
            short_flags.push(Flag::Arg(format!("-{}", s), &*arg.name));
        }

        for (short_alias, _) in &arg.short_aliases {
            short_flags.push(Flag::Arg(format!("-{}", short_alias), arg.name));
        }

        if let Some(l) = arg.long.as_ref() {
            #[cfg(feature = "unstable-v4")]
            {
                assert!(!l.starts_with('-'), "Argument {}: long {:?} must not start with a `-`, that will be handled by the parser", arg.name, l);
            }
            long_flags.push(Flag::Arg(format!("--{}", l), &*arg.name));
        }

        for (long_alias, _) in &arg.aliases {
            long_flags.push(Flag::Arg(format!("--{}", long_alias), arg.name));
        }

        // Name conflicts
        assert!(
            cmd.two_args_of(|x| x.id == arg.id).is_none(),
            "Command {}: Argument names must be unique, but '{}' is in use by more than one argument or group",
            cmd.get_name(),
            arg.name,
        );

        // Long conflicts
        if let Some(l) = arg.long {
            if let Some((first, second)) = cmd.two_args_of(|x| x.long == Some(l)) {
                panic!(
                    "Command {}: Long option names must be unique for each argument, \
                        but '--{}' is in use by both '{}' and '{}'",
                    cmd.get_name(),
                    l,
                    first.name,
                    second.name
                )
            }
        }

        // Short conflicts
        if let Some(s) = arg.short {
            if let Some((first, second)) = cmd.two_args_of(|x| x.short == Some(s)) {
                panic!(
                    "Command {}: Short option names must be unique for each argument, \
                        but '-{}' is in use by both '{}' and '{}'",
                    cmd.get_name(),
                    s,
                    first.name,
                    second.name
                )
            }
        }

        // Index conflicts
        if let Some(idx) = arg.index {
            if let Some((first, second)) =
                cmd.two_args_of(|x| x.is_positional() && x.index == Some(idx))
            {
                panic!(
                    "Command {}: Argument '{}' has the same index as '{}' \
                    and they are both positional arguments\n\n\t \
                    Use Arg::multiple_values(true) to allow one \
                    positional argument to take multiple values",
                    cmd.get_name(),
                    first.name,
                    second.name
                )
            }
        }

        // requires, r_if, r_unless
        for req in &arg.requires {
            assert!(
                cmd.id_exists(&req.1),
                "Command {}: Argument or group '{:?}' specified in 'requires*' for '{}' does not exist",
                cmd.get_name(),
                req.1,
                arg.name,
            );
        }

        for req in &arg.r_ifs {
            #[cfg(feature = "unstable-v4")]
            {
                assert!(
                    !arg.is_required_set(),
                    "Argument {}: `required` conflicts with `required_if_eq*`",
                    arg.name
                );
            }
            assert!(
                cmd.id_exists(&req.0),
                "Command {}: Argument or group '{:?}' specified in 'required_if_eq*' for '{}' does not exist",
                    cmd.get_name(),
                req.0,
                arg.name
            );
        }

        for req in &arg.r_ifs_all {
            #[cfg(feature = "unstable-v4")]
            {
                assert!(
                    !arg.is_required_set(),
                    "Argument {}: `required` conflicts with `required_if_eq_all`",
                    arg.name
                );
            }
            assert!(
                cmd.id_exists(&req.0),
                "Command {}: Argument or group '{:?}' specified in 'required_if_eq_all' for '{}' does not exist",
                    cmd.get_name(),
                req.0,
                arg.name
            );
        }

        for req in &arg.r_unless {
            #[cfg(feature = "unstable-v4")]
            {
                assert!(
                    !arg.is_required_set(),
                    "Argument {}: `required` conflicts with `required_unless*`",
                    arg.name
                );
            }
            assert!(
                cmd.id_exists(req),
                "Command {}: Argument or group '{:?}' specified in 'required_unless*' for '{}' does not exist",
                    cmd.get_name(),
                req,
                arg.name,
            );
        }

        for req in &arg.r_unless_all {
            #[cfg(feature = "unstable-v4")]
            {
                assert!(
                    !arg.is_required_set(),
                    "Argument {}: `required` conflicts with `required_unless*`",
                    arg.name
                );
            }
            assert!(
                cmd.id_exists(req),
                "Command {}: Argument or group '{:?}' specified in 'required_unless*' for '{}' does not exist",
                    cmd.get_name(),
                req,
                arg.name,
            );
        }

        // blacklist
        for req in &arg.blacklist {
            assert!(
                cmd.id_exists(req),
                "Command {}: Argument or group '{:?}' specified in 'conflicts_with*' for '{}' does not exist",
                    cmd.get_name(),
                req,
                arg.name,
            );
        }

        if arg.is_last_set() {
            assert!(
                arg.long.is_none(),
                "Command {}: Flags or Options cannot have last(true) set. '{}' has both a long and last(true) set.",
                    cmd.get_name(),
                arg.name
            );
            assert!(
                arg.short.is_none(),
                "Command {}: Flags or Options cannot have last(true) set. '{}' has both a short and last(true) set.",
                    cmd.get_name(),
                arg.name
            );
        }

        assert!(
            !(arg.is_required_set() && arg.is_global_set()),
            "Command {}: Global arguments cannot be required.\n\n\t'{}' is marked as both global and required",
                    cmd.get_name(),
            arg.name
        );

        // validators
        assert!(
            arg.validator.is_none() || arg.validator_os.is_none(),
            "Command {}: Argument '{}' has both `validator` and `validator_os` set which is not allowed",
                    cmd.get_name(),
            arg.name
        );

        if arg.get_value_hint() == ValueHint::CommandWithArguments {
            assert!(
                arg.is_positional(),
                "Command {}: Argument '{}' has hint CommandWithArguments and must be positional.",
                cmd.get_name(),
                arg.name
            );

            assert!(
                cmd.is_trailing_var_arg_set(),
                "Command {}: Positional argument '{}' has hint CommandWithArguments, so Command must have TrailingVarArg set.",
                    cmd.get_name(),
                arg.name
            );
        }
    }

    for group in cmd.get_groups() {
        // Name conflicts
        assert!(
            cmd.get_groups().filter(|x| x.id == group.id).count() < 2,
            "Command {}: Argument group name must be unique\n\n\t'{}' is already in use",
            cmd.get_name(),
            group.name,
        );

        // Groups should not have naming conflicts with Args
        assert!(
            !cmd.get_arguments().any(|x| x.id == group.id),
            "Command {}: Argument group name '{}' must not conflict with argument name",
            cmd.get_name(),
            group.name,
        );

        for arg in &group.args {
            // Args listed inside groups should exist
            assert!(
                cmd.get_arguments().any(|x| x.id == *arg),
                "Command {}: Argument group '{}' contains non-existent argument '{:?}'",
                cmd.get_name(),
                group.name,
                arg
            );
        }
    }

    // Conflicts between flags and subcommands

    long_flags.sort_unstable();
    short_flags.sort_unstable();

    detect_duplicate_flags(&long_flags, "long");
    detect_duplicate_flags(&short_flags, "short");

    _verify_positionals(cmd);

    if let Some(help_template) = cmd.get_help_template() {
        assert!(
            !help_template.contains("{flags}"),
            "Command {}: {}",
                    cmd.get_name(),
            "`{flags}` template variable was removed in clap3, they are now included in `{options}`",
        );
        assert!(
            !help_template.contains("{unified}"),
            "Command {}: {}",
            cmd.get_name(),
            "`{unified}` template variable was removed in clap3, use `{options}` instead"
        );
    }

    cmd._panic_on_missing_help(cmd.is_help_expected_set());
    assert_app_flags(cmd);
}

#[derive(Eq)]
enum Flag<'a> {
    Command(String, &'a str),
    Arg(String, &'a str),
}

impl PartialEq for Flag<'_> {
    fn eq(&self, other: &Flag) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl PartialOrd for Flag<'_> {
    fn partial_cmp(&self, other: &Flag) -> Option<Ordering> {
        use Flag::*;

        match (self, other) {
            (Command(s1, _), Command(s2, _))
            | (Arg(s1, _), Arg(s2, _))
            | (Command(s1, _), Arg(s2, _))
            | (Arg(s1, _), Command(s2, _)) => {
                if s1 == s2 {
                    Some(Ordering::Equal)
                } else {
                    s1.partial_cmp(s2)
                }
            }
        }
    }
}

impl Ord for Flag<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

fn detect_duplicate_flags(flags: &[Flag], short_or_long: &str) {
    use Flag::*;

    for (one, two) in find_duplicates(flags) {
        match (one, two) {
            (Command(flag, one), Command(_, another)) if one != another => panic!(
                "the '{}' {} flag is specified for both '{}' and '{}' subcommands",
                flag, short_or_long, one, another
            ),

            (Arg(flag, one), Arg(_, another)) if one != another => panic!(
                "{} option names must be unique, but '{}' is in use by both '{}' and '{}'",
                short_or_long, flag, one, another
            ),

            (Arg(flag, arg), Command(_, sub)) | (Command(flag, sub), Arg(_, arg)) => panic!(
                "the '{}' {} flag for the '{}' argument conflicts with the short flag \
                     for '{}' subcommand",
                flag, short_or_long, arg, sub
            ),

            _ => {}
        }
    }
}

/// Find duplicates in a sorted array.
///
/// The algorithm is simple: the array is sorted, duplicates
/// must be placed next to each other, we can check only adjacent elements.
fn find_duplicates<T: PartialEq>(slice: &[T]) -> impl Iterator<Item = (&T, &T)> {
    slice.windows(2).filter_map(|w| {
        if w[0] == w[1] {
            Some((&w[0], &w[1]))
        } else {
            None
        }
    })
}

fn assert_app_flags(cmd: &Command) {
    macro_rules! checker {
        ($a:ident requires $($b:ident)|+) => {
            if cmd.$a() {
                let mut s = String::new();

                $(
                    if !cmd.$b() {
                        s.push_str(&format!("  AppSettings::{} is required when AppSettings::{} is set.\n", std::stringify!($b), std::stringify!($a)));
                    }
                )+

                if !s.is_empty() {
                    panic!("{}", s)
                }
            }
        };
        ($a:ident conflicts $($b:ident)|+) => {
            if cmd.$a() {
                let mut s = String::new();

                $(
                    if cmd.$b() {
                        s.push_str(&format!("  AppSettings::{} conflicts with AppSettings::{}.\n", std::stringify!($b), std::stringify!($a)));
                    }
                )+

                if !s.is_empty() {
                    panic!("{}\n{}", cmd.get_name(), s)
                }
            }
        };
    }

    checker!(is_allow_invalid_utf8_for_external_subcommands_set requires is_allow_external_subcommands_set);
    checker!(is_multicall_set conflicts is_no_binary_name_set);
}

#[cfg(debug_assertions)]
fn _verify_positionals(cmd: &Command) -> bool {
    debug!("Command::_verify_positionals");
    // Because you must wait until all arguments have been supplied, this is the first chance
    // to make assertions on positional argument indexes
    //
    // First we verify that the index highest supplied index, is equal to the number of
    // positional arguments to verify there are no gaps (i.e. supplying an index of 1 and 3
    // but no 2)

    let highest_idx = cmd
        .get_keymap()
        .keys()
        .filter_map(|x| {
            if let KeyType::Position(n) = x {
                Some(*n)
            } else {
                None
            }
        })
        .max()
        .unwrap_or(0);

    let num_p = cmd.get_keymap().keys().filter(|x| x.is_position()).count();

    assert!(
        highest_idx == num_p,
        "Found positional argument whose index is {} but there \
             are only {} positional arguments defined",
        highest_idx,
        num_p
    );

    // Next we verify that only the highest index has takes multiple arguments (if any)
    let only_highest = |a: &Arg| a.is_multiple() && (a.index.unwrap_or(0) != highest_idx);
    if cmd.get_positionals().any(only_highest) {
        // First we make sure if there is a positional that allows multiple values
        // the one before it (second to last) has one of these:
        //  * a value terminator
        //  * ArgSettings::Last
        //  * The last arg is Required

        // We can't pass the closure (it.next()) to the macro directly because each call to
        // find() (iterator, not macro) gets called repeatedly.
        let last = &cmd.get_keymap()[&KeyType::Position(highest_idx)];
        let second_to_last = &cmd.get_keymap()[&KeyType::Position(highest_idx - 1)];

        // Either the final positional is required
        // Or the second to last has a terminator or .last(true) set
        let ok = last.is_required_set()
            || (second_to_last.terminator.is_some() || second_to_last.is_last_set())
            || last.is_last_set();
        assert!(
            ok,
            "When using a positional argument with .multiple_values(true) that is *not the \
                 last* positional argument, the last positional argument (i.e. the one \
                 with the highest index) *must* have .required(true) or .last(true) set."
        );

        // We make sure if the second to last is Multiple the last is ArgSettings::Last
        let ok = second_to_last.is_multiple() || last.is_last_set();
        assert!(
            ok,
            "Only the last positional argument, or second to last positional \
                 argument may be set to .multiple_values(true)"
        );

        // Next we check how many have both Multiple and not a specific number of values set
        let count = cmd
            .get_positionals()
            .filter(|p| {
                #[allow(deprecated)]
                {
                    p.is_multiple_occurrences_set()
                        || (p.is_multiple_values_set() && p.num_vals.is_none())
                }
            })
            .count();
        let ok = count <= 1
            || (last.is_last_set()
                && last.is_multiple()
                && second_to_last.is_multiple()
                && count == 2);
        assert!(
            ok,
            "Only one positional argument with .multiple_values(true) set is allowed per \
                 command, unless the second one also has .last(true) set"
        );
    }

    let mut found = false;

    if cmd.is_allow_missing_positional_set() {
        // Check that if a required positional argument is found, all positions with a lower
        // index are also required.
        let mut foundx2 = false;

        for p in cmd.get_positionals() {
            if foundx2 && !p.is_required_set() {
                assert!(
                    p.is_required_set(),
                    "Found non-required positional argument with a lower \
                         index than a required positional argument by two or more: {:?} \
                         index {:?}",
                    p.name,
                    p.index
                );
            } else if p.is_required_set() && !p.is_last_set() {
                // Args that .last(true) don't count since they can be required and have
                // positionals with a lower index that aren't required
                // Imagine: prog <req1> [opt1] -- <req2>
                // Both of these are valid invocations:
                //      $ prog r1 -- r2
                //      $ prog r1 o1 -- r2
                if found {
                    foundx2 = true;
                    continue;
                }
                found = true;
                continue;
            } else {
                found = false;
            }
        }
    } else {
        // Check that if a required positional argument is found, all positions with a lower
        // index are also required
        for p in (1..=num_p).rev().filter_map(|n| cmd.get_keymap().get(&n)) {
            if found {
                assert!(
                    p.is_required_set(),
                    "Found non-required positional argument with a lower \
                         index than a required positional argument: {:?} index {:?}",
                    p.name,
                    p.index
                );
            } else if p.is_required_set() && !p.is_last_set() {
                // Args that .last(true) don't count since they can be required and have
                // positionals with a lower index that aren't required
                // Imagine: prog <req1> [opt1] -- <req2>
                // Both of these are valid invocations:
                //      $ prog r1 -- r2
                //      $ prog r1 o1 -- r2
                found = true;
                continue;
            }
        }
    }
    assert!(
        cmd.get_positionals().filter(|p| p.is_last_set()).count() < 2,
        "Only one positional argument may have last(true) set. Found two."
    );
    if cmd
        .get_positionals()
        .any(|p| p.is_last_set() && p.is_required_set())
        && cmd.has_subcommands()
        && !cmd.is_subcommand_negates_reqs_set()
    {
        panic!(
            "Having a required positional argument with .last(true) set *and* child \
                 subcommands without setting SubcommandsNegateReqs isn't compatible."
        );
    }

    true
}

fn assert_arg(arg: &Arg) {
    debug!("Arg::_debug_asserts:{}", arg.name);

    // Self conflict
    // TODO: this check should be recursive
    assert!(
        !arg.blacklist.iter().any(|x| *x == arg.id),
        "Argument '{}' cannot conflict with itself",
        arg.name,
    );

    assert_eq!(
        arg.get_action().takes_values(),
        arg.is_takes_value_set(),
        "Argument `{}`'s selected action {:?} contradicts `takes_value`",
        arg.name,
        arg.get_action()
    );
    if let Some(action_type_id) = arg.get_action().value_type_id() {
        assert_eq!(
            action_type_id,
            arg.get_value_parser().type_id(),
            "Argument `{}`'s selected action {:?} contradicts `value_parser` ({:?})",
            arg.name,
            arg.get_action(),
            arg.get_value_parser()
        );
    }

    if arg.get_value_hint() != ValueHint::Unknown {
        assert!(
            arg.is_takes_value_set(),
            "Argument '{}' has value hint but takes no value",
            arg.name
        );

        if arg.get_value_hint() == ValueHint::CommandWithArguments {
            assert!(
                arg.is_multiple_values_set(),
                "Argument '{}' uses hint CommandWithArguments and must accept multiple values",
                arg.name
            )
        }
    }

    if arg.index.is_some() {
        assert!(
            arg.is_positional(),
            "Argument '{}' is a positional argument and can't have short or long name versions",
            arg.name
        );
        assert!(
            arg.is_takes_value_set(),
            "Argument '{}` is positional, it must take a value",
            arg.name
        );
    }

    #[cfg(feature = "unstable-v4")]
    {
        let num_vals = arg.get_num_vals().unwrap_or(usize::MAX);
        let num_val_names = arg.get_value_names().unwrap_or(&[]).len();
        if num_vals < num_val_names {
            panic!(
                "Argument {}: Too many value names ({}) compared to number_of_values ({})",
                arg.name, num_val_names, num_vals
            );
        }
    }

    assert_arg_flags(arg);

    assert_defaults(arg, "default_value", arg.default_vals.iter().copied());
    assert_defaults(
        arg,
        "default_missing_value",
        arg.default_missing_vals.iter().copied(),
    );
    assert_defaults(
        arg,
        "default_value_if",
        arg.default_vals_ifs
            .iter()
            .filter_map(|(_, _, default)| *default),
    );
}

fn assert_arg_flags(arg: &Arg) {
    macro_rules! checker {
        ($a:ident requires $($b:ident)|+) => {
            if arg.$a() {
                let mut s = String::new();

                $(
                    if !arg.$b() {
                        s.push_str(&format!("  Arg::{} is required when Arg::{} is set.\n", std::stringify!($b), std::stringify!($a)));
                    }
                )+

                if !s.is_empty() {
                    panic!("Argument {:?}\n{}", arg.get_id(), s)
                }
            }
        }
    }

    checker!(is_require_value_delimiter_set requires is_takes_value_set);
    checker!(is_require_value_delimiter_set requires is_use_value_delimiter_set);
    checker!(is_hide_possible_values_set requires is_takes_value_set);
    checker!(is_allow_hyphen_values_set requires is_takes_value_set);
    checker!(is_require_equals_set requires is_takes_value_set);
    checker!(is_last_set requires is_takes_value_set);
    checker!(is_hide_default_value_set requires is_takes_value_set);
    checker!(is_multiple_values_set requires is_takes_value_set);
    checker!(is_ignore_case_set requires is_takes_value_set);
    {
        #![allow(deprecated)]
        checker!(is_forbid_empty_values_set requires is_takes_value_set);
        checker!(is_allow_invalid_utf8_set requires is_takes_value_set);
    }
}

fn assert_defaults<'d>(
    arg: &Arg,
    field: &'static str,
    defaults: impl IntoIterator<Item = &'d std::ffi::OsStr>,
) {
    for default_os in defaults {
        if let Some(default_s) = default_os.to_str() {
            if !arg.possible_vals.is_empty() {
                if let Some(delim) = arg.get_value_delimiter() {
                    for part in default_s.split(delim) {
                        assert!(
                            arg.possible_vals.iter().any(|possible_val| {
                                possible_val.matches(part, arg.is_ignore_case_set())
                            }),
                            "Argument `{}`'s {}={} doesn't match possible values",
                            arg.name,
                            field,
                            part
                        )
                    }
                } else {
                    assert!(
                        arg.possible_vals.iter().any(|possible_val| {
                            possible_val.matches(default_s, arg.is_ignore_case_set())
                        }),
                        "Argument `{}`'s {}={} doesn't match possible values",
                        arg.name,
                        field,
                        default_s
                    );
                }
            }

            if let Some(validator) = arg.validator.as_ref() {
                let mut validator = validator.lock().unwrap();
                if let Some(delim) = arg.get_value_delimiter() {
                    for part in default_s.split(delim) {
                        if let Err(err) = validator(part) {
                            panic!(
                                "Argument `{}`'s {}={} failed validation: {}",
                                arg.name, field, part, err
                            );
                        }
                    }
                } else if let Err(err) = validator(default_s) {
                    panic!(
                        "Argument `{}`'s {}={} failed validation: {}",
                        arg.name, field, default_s, err
                    );
                }
            }
        }

        if let Some(validator) = arg.validator_os.as_ref() {
            let mut validator = validator.lock().unwrap();
            if let Some(delim) = arg.get_value_delimiter() {
                let default_os = RawOsStr::new(default_os);
                for part in default_os.split(delim) {
                    if let Err(err) = validator(&part.to_os_str()) {
                        panic!(
                            "Argument `{}`'s {}={:?} failed validation: {}",
                            arg.name, field, part, err
                        );
                    }
                }
            } else if let Err(err) = validator(default_os) {
                panic!(
                    "Argument `{}`'s {}={:?} failed validation: {}",
                    arg.name, field, default_os, err
                );
            }
        }

        let value_parser = arg.get_value_parser();
        let assert_cmd = Command::new("assert");
        if let Some(delim) = arg.get_value_delimiter() {
            let default_os = RawOsStr::new(default_os);
            for part in default_os.split(delim) {
                if let Err(err) = value_parser.parse_ref(&assert_cmd, Some(arg), &part.to_os_str())
                {
                    panic!(
                        "Argument `{}`'s {}={:?} failed validation: {}",
                        arg.name,
                        field,
                        part.to_str_lossy(),
                        err
                    );
                }
            }
        } else if let Err(err) = value_parser.parse_ref(&assert_cmd, Some(arg), default_os) {
            panic!(
                "Argument `{}`'s {}={:?} failed validation: {}",
                arg.name, field, default_os, err
            );
        }
    }
}
