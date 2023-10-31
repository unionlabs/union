#![allow(deprecated)]

// Std
use std::ops::BitOr;
#[cfg(feature = "yaml")]
use std::str::FromStr;

#[allow(unused)]
use crate::Arg;
#[allow(unused)]
use crate::Command;

// Third party
use bitflags::bitflags;

#[doc(hidden)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct AppFlags(Flags);

impl Default for AppFlags {
    fn default() -> Self {
        AppFlags(Flags::COLOR_AUTO)
    }
}

/// Application level settings, which affect how [`Command`] operates
///
/// **NOTE:** When these settings are used, they apply only to current command, and are *not*
/// propagated down or up through child or parent subcommands
///
/// [`Command`]: crate::Command
#[derive(Debug, PartialEq, Copy, Clone)]
#[non_exhaustive]
pub enum AppSettings {
    /// Deprecated, replaced with [`Command::ignore_errors`]
    ///
    /// Derive: replace `#[clap(setting = IgnoreErrors)]` with `#[clap(ignore_errors = true)]`
    ///
    /// Builder: replace `cmd.setting(IgnoreErrors)` with `cmd.ignore_errors = true`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.1.0",
            note = "Replaced with `Command::ignore_errors`

Derive: replace `#[clap(setting = IgnoreErrors)]` with `#[clap(ignore_errors = true)]`

Builder: replace `cmd.setting(IgnoreErrors)` with `cmd.ignore_errors(true)`
"
        )
    )]
    IgnoreErrors,

    /// Deprecated, replace
    /// ```rust,no_run
    /// let cmd = clap::Command::new("cmd")
    ///     .global_setting(clap::AppSettings::WaitOnError)
    ///     .arg(clap::arg!(--flag));
    /// let m = cmd.get_matches();
    /// ```
    /// with
    /// ```rust
    /// let cmd = clap::Command::new("cmd")
    ///     .arg(clap::arg!(--flag));
    /// let m = match cmd.try_get_matches() {
    ///     Ok(m) => m,
    ///     Err(err) => {
    ///         if err.use_stderr() {
    ///             let _ = err.print();
    ///
    ///             eprintln!("\nPress [ENTER] / [RETURN] to continue...");
    ///             use std::io::BufRead;
    ///             let mut s = String::new();
    ///             let i = std::io::stdin();
    ///             i.lock().read_line(&mut s).unwrap();
    ///
    ///             std::process::exit(2);
    ///         } else {
    ///             let _ = err.print();
    ///             std::process::exit(0);
    ///         }
    ///     }
    /// };
    /// ```
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.1.0",
            note = "See documentation for how to hand-implement this"
        )
    )]
    WaitOnError,

    /// Deprecated, replaced with [`Command::allow_hyphen_values`] and
    /// [`Arg::is_allow_hyphen_values_set`]
    ///
    /// Derive: replace `#[clap(setting = AllowHyphenValues)]` with `#[clap(allow_hyphen_values = true)]`
    ///
    /// Builder: replace `cmd.setting(AllowHyphenValues)` with `cmd.allow_hyphen_values(true)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.1.0",
            note = "Replaced with `Command::allow_hyphen_values` and `Arg::is_allow_hyphen_values_set`

Derive: replace `#[clap(setting = AllowHyphenValues)]` with `#[clap(allow_hyphen_values = true)]`

Builder: replace `cmd.setting(AllowHyphenValues)` with `cmd.allow_hyphen_values(true)`
"
        )
    )]
    AllowHyphenValues,

    /// Deprecated, replaced with [`Command::allow_negative_numbers`] and
    /// [`Command::is_allow_negative_numbers_set`]
    ///
    /// Derive: replace `#[clap(setting = AllowNegativeNumbers)]` with `#[clap(allow_negative_numbers = true)]`
    ///
    /// Builder: replace `cmd.setting(AllowNegativeNumbers)` with `cmd.allow_negative_numbers(true)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.1.0",
            note = "Replaced with `Command::allow_negative_numbers` and `Command::is_allow_negative_numbers_set`

Derive: replace `#[clap(setting = AllowNegativeNumbers)]` with `#[clap(allow_negative_numbers = true)]`

Builder: replace `cmd.setting(AllowNegativeNumbers)` with `cmd.allow_negative_numbers(true)`
"
        )
    )]
    AllowNegativeNumbers,

    /// Deprecated, replaced with [`ArgAction::Set`][super::ArgAction::Set]
    ///
    /// The new actions (`ArgAction::Set`, `ArgAction::SetTrue`) do this by default.
    ///
    /// See `ArgAction::StoreValue` and `ArgAction::IncOccurrence` for how to migrate
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.2.0",
            note = "Replaced with `Arg::action(ArgAction::...)`

The new actions (`ArgAction::Set`, `ArgAction::SetTrue`) do this by default.

See `ArgAction::StoreValue` and `ArgAction::IncOccurrence` for how to migrate
"
        )
    )]
    AllArgsOverrideSelf,

    /// Deprecated, replaced with [`Command::allow_missing_positional`] and
    /// [`Command::is_allow_missing_positional_set`]
    ///
    /// Derive: replace `#[clap(setting = AllowMissingPositional)]` with `#[clap(allow_missing_positional = true)]`
    ///
    /// Builder: replace `cmd.setting(AllowMissingPositional)` with `cmd.allow_missing_positional(true)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.1.0",
            note = "Replaced with `Command::allow_missing_positional` and `Command::is_allow_missing_positional_set`

Derive: replace `#[clap(setting = AllowMissingPositional)]` with `#[clap(allow_missing_positional = true)]`

Builder: replace `cmd.setting(AllowMissingPositional)` with `cmd.allow_missing_positional(true)`
"
        )
    )]
    AllowMissingPositional,

    /// Deprecated, replaced with [`Command::trailing_var_arg`] and [`Command::is_trailing_var_arg_set`]
    ///
    /// Derive: replace `#[clap(setting = TrailingVarArg)]` with `#[clap(trailing_var_arg = true)]`
    ///
    /// Builder: replace `cmd.setting(TrailingVarArg)` with `cmd.trailing_var_arg(true)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.1.0",
            note = "Replaced with `Command::trailing_var_arg` and `Command::is_trailing_var_arg_set`

Derive: replace `#[clap(setting = TrailingVarArg)]` with `#[clap(trailing_var_arg = true)]`

Builder: replace `cmd.setting(TrailingVarArg)` with `cmd.trailing_var_arg(true)`
"
        )
    )]
    TrailingVarArg,

    /// Deprecated, replaced with [`Command::dont_delimit_trailing_values`] and
    /// [`Command::is_dont_delimit_trailing_values_set`]
    ///
    /// Derive: replace `#[clap(setting = DontDelimitTrailingValues)]` with `#[clap(dont_delimit_trailing_values = true)]`
    ///
    /// Builder: replace `cmd.setting(DontDelimitTrailingValues)` with `cmd.dont_delimit_trailing_values(true)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.1.0",
            note = "Replaced with `Command::dont_delimit_trailing_values` and `Command::is_dont_delimit_trailing_values_set`

Derive: replace `#[clap(setting = DontDelimitTrailingValues)]` with `#[clap(dont_delimit_trailing_values = true)]`

Builder: replace `cmd.setting(DontDelimitTrailingValues)` with `cmd.dont_delimit_trailing_values(true)`
"
        )
    )]
    DontDelimitTrailingValues,

    /// Deprecated, replaced with [`Command::infer_long_args`]
    ///
    /// Derive: replace `#[clap(setting = InferLongArgs)]` with `#[clap(infer_long_args = true)]`
    ///
    /// Builder: replace `cmd.setting(InferLongArgs)` with `cmd.infer_long_args(true)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.1.0",
            note = "Replaced with `Command::infer_long_args`

Derive: replace `#[clap(setting = InferLongArgs)]` with `#[clap(infer_long_args = true)]`

Builder: replace `cmd.setting(InferLongArgs)` with `cmd.infer_long_args(true)`
"
        )
    )]
    InferLongArgs,

    /// Deprecated, replaced with [`Command::infer_subcommands`]
    ///
    /// Derive: replace `#[clap(setting = InferSubcommands)]` with `#[clap(infer_subcommands = true)]`
    ///
    /// Builder: replace `cmd.setting(InferSubcommands)` with `cmd.infer_subcommands(true)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.1.0",
            note = "Replaced with `Command::infer_subcommands`

Derive: replace `#[clap(setting = InferSubcommands)]` with `#[clap(infer_subcommands = true)]`

Builder: replace `cmd.setting(InferSubcommands)` with `cmd.infer_subcommands(true)`
"
        )
    )]
    InferSubcommands,

    /// Deprecated, replaced with [`Command::subcommand_required`] and
    /// [`Command::is_subcommand_required_set`]
    ///
    /// Derive: replace `#[clap(setting = SubcommandRequired)]` with `#[clap(subcommand_required = true)]`
    ///
    /// Builder: replace `cmd.setting(SubcommandRequired)` with `cmd.subcommand_required(true)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.1.0",
            note = "Replaced with `Command::subcommand_required` and `Command::is_subcommand_required_set`

Derive: replace `#[clap(setting = SubcommandRequired)]` with `#[clap(subcommand_required = true)]`

Builder: replace `cmd.setting(SubcommandRequired)` with `cmd.subcommand_required(true)`
"
        )
    )]
    SubcommandRequired,

    /// Deprecated, replaced with [`Command::subcommand_required`] combined with
    /// [`Command::arg_required_else_help`].
    ///
    /// Derive: replace `#[clap(setting = SubcommandRequiredElseHelp)]` with `#[clap(subcommand_required = true, arg_required_else_help = true)]`
    ///
    /// Builder: replace `cmd.setting(SubcommandRequiredElseHelp)` with `cmd.subcommand_required(true).arg_required_else_help(true)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.1.0",
            note = "Replaced with `Command::subcommand_required` combined with `Command::arg_required_else_help`

Derive: replace `#[clap(setting = SubcommandRequiredElseHelp)]` with `#[clap(subcommand_required = true, arg_required_else_help = true)]`

Builder: replace `cmd.setting(SubcommandRequiredElseHelp)` with `cmd.subcommand_required(true).arg_required_else_help(true)`
"
        )
    )]
    SubcommandRequiredElseHelp,

    /// Deprecated, replaced with [`Command::allow_external_subcommands`] and
    /// [`Command::is_allow_external_subcommands_set`]
    ///
    /// Derive: replace `#[clap(setting = AllowExternalSubcommands)]` with `#[clap(allow_external_subcommands = true)]`
    ///
    /// Builder: replace `cmd.setting(AllowExternalSubcommands)` with `cmd.allow_external_subcommands(true)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.1.0",
            note = "Replaced with `Command::allow_external_subcommands` and `Command::is_allow_external_subcommands_set`

Derive: replace `#[clap(setting = AllowExternalSubcommands)]` with `#[clap(allow_external_subcommands = true)]`

Builder: replace `cmd.setting(AllowExternalSubcommands)` with `cmd.allow_external_subcommands(true)`
"
        )
    )]
    AllowExternalSubcommands,

    /// Deprecated, replaced with [`Command::multicall`] and [`Command::is_multicall_set`]
    ///
    /// Derive: replace `#[clap(setting = Multicall)]` with `#[clap(multicall = true)]`
    ///
    /// Builder: replace `cmd.setting(Multicall)` with `cmd.multicall(true)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.1.0",
            note = "Replaced with `Command::multicall` and `Command::is_multicall_set`

Derive: replace `#[clap(setting = Multicall)]` with `#[clap(multicall = true)]`

Builder: replace `cmd.setting(Multicall)` with `cmd.multicall(true)`
"
        )
    )]
    Multicall,

    /// Deprecated, replaced with [`Command::allow_invalid_utf8_for_external_subcommands`] and [`Command::is_allow_invalid_utf8_for_external_subcommands_set`]
    ///
    /// Derive: replace `#[clap(setting = AllowInvalidUtf8ForExternalSubcommands)]` with `#[clap(allow_invalid_utf8_for_external_subcommands = true)]`
    ///
    /// Builder: replace `cmd.setting(AllowInvalidUtf8ForExternalSubcommands)` with `cmd.allow_invalid_utf8_for_external_subcommands(true)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.1.0",
            note = "Replaced with `Command::allow_invalid_utf8_for_external_subcommands` and `Command::is_allow_invalid_utf8_for_external_subcommands_set`

Derive: replace `#[clap(setting = AllowInvalidUtf8ForExternalSubcommands)]` with `#[clap(allow_invalid_utf8_for_external_subcommands = true)]`

Builder: replace `cmd.setting(AllowInvalidUtf8ForExternalSubcommands)` with `cmd.allow_invalid_utf8_for_external_subcommands(true)`
"
        )
    )]
    AllowInvalidUtf8ForExternalSubcommands,

    /// Deprecated, this is now the default
    ///
    /// Derive: remove `#[clap(setting = UseLongFormatForHelpSubcommand)]`
    ///
    /// Builder: remove `cmd.setting(UseLongFormatForHelpSubcommand)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.1.0",
            note = "This is now the default

Derive: remove `#[clap(setting = UseLongFormatForHelpSubcommand)]`

Builder: remove `cmd.setting(UseLongFormatForHelpSubcommand)`
"
        )
    )]
    UseLongFormatForHelpSubcommand,

    /// Deprecated, replaced with [`Command::subcommand_negates_reqs`] and
    /// [`Command::is_subcommand_negates_reqs_set`]
    ///
    /// Derive: replace `#[clap(setting = SubcommandsNegateReqs)]` with `#[clap(subcommand_negates_reqs = true)]`
    ///
    /// Builder: replace `cmd.setting(SubcommandsNegateReqs)` with `cmd.subcommand_negates_reqs(true)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.1.0",
            note = "Replaced with `Command::subcommand_negates_reqs` and `Command::is_subcommand_negates_reqs_set`

Derive: replace `#[clap(setting = SubcommandsNegateReqs)]` with `#[clap(subcommand_negates_reqs = true)]`

Builder: replace `cmd.setting(SubcommandsNegateReqs)` with `cmd.subcommand_negates_reqs(true)`
"
        )
    )]
    SubcommandsNegateReqs,

    /// Deprecated, replaced with [`Command::args_conflicts_with_subcommands`] and
    /// [`Command::is_args_conflicts_with_subcommands_set`]
    ///
    /// Derive: replace `#[clap(setting = ArgsNegateSubcommands)]` with `#[clap(args_conflicts_with_subcommands = true)]`
    ///
    /// Builder: replace `cmd.setting(ArgsNegateSubcommands)` with `cmd.args_conflicts_with_subcommands(true)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.1.0",
            note = "Replaced with `Command::args_conflicts_with_subcommands` and `Command::is_args_conflicts_with_subcommands_set`

Derive: replace `#[clap(setting = ArgsNegateSubcommands)]` with `#[clap(args_conflicts_with_subcommands = true)]`

Builder: replace `cmd.setting(ArgsNegateSubcommands)` with `cmd.args_conflicts_with_subcommands(true)`
"
        )
    )]
    ArgsNegateSubcommands,

    /// Deprecated, replaced with [`Command::subcommand_precedence_over_arg`] and
    /// [`Command::is_subcommand_precedence_over_arg_set`]
    ///
    /// Derive: replace `#[clap(setting = SubcommandPrecedenceOverArg)]` with `#[clap(subcommand_precedence_over_arg = true)]`
    ///
    /// Builder: replace `cmd.setting(SubcommandPrecedenceOverArg)` with `cmd.subcommand_precedence_over_arg(true)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.1.0",
            note = "Replaced with `Command::subcommand_precedence_over_arg` and `Command::is_subcommand_precedence_over_arg_set`

Derive: replace `#[clap(setting = SubcommandPrecedenceOverArg)]` with `#[clap(subcommand_precedence_over_arg = true)]`

Builder: replace `cmd.setting(SubcommandPrecedenceOverArg)` with `cmd.subcommand_precedence_over_arg(true)`
"
        )
    )]
    SubcommandPrecedenceOverArg,

    /// Deprecated, replaced with [`Command::arg_required_else_help`] and
    /// [`Command::is_arg_required_else_help_set`]
    ///
    /// Derive: replace `#[clap(setting = ArgRequiredElseHelp)]` with `#[clap(arg_required_else_help = true)]`
    ///
    /// Builder: replace `cmd.setting(ArgRequiredElseHelp)` with `cmd.arg_required_else_help(true)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.1.0",
            note = "Replaced with `Command::arg_required_else_help` and `Command::is_arg_required_else_help_set`

Derive: replace `#[clap(setting = ArgRequiredElseHelp)]` with `#[clap(arg_required_else_help = true)]`

Builder: replace `cmd.setting(ArgRequiredElseHelp)` with `cmd.arg_required_else_help(true)`
"
        )
    )]
    ArgRequiredElseHelp,

    /// Displays the arguments and [`subcommands`] in the help message in the order that they were
    /// declared in, and not alphabetically which is the default.
    ///
    /// To override the declaration order, see [`Arg::display_order`] and [`Command::display_order`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{Command, Arg, AppSettings};
    /// Command::new("myprog")
    ///     .global_setting(AppSettings::DeriveDisplayOrder)
    ///     .get_matches();
    /// ```
    ///
    /// [`subcommands`]: crate::Command::subcommand()
    /// [`Arg::display_order`]: crate::Arg::display_order
    /// [`Command::display_order`]: crate::Command::display_order
    DeriveDisplayOrder,

    /// Deprecated, replaced with [`Command::dont_collapse_args_in_usage`] and
    /// [`Command::is_dont_collapse_args_in_usage_set`]
    ///
    /// Derive: replace `#[clap(setting = DontCollapseArgsInUsage)]` with `#[clap(dont_collapse_args_in_usage = true)]`
    ///
    /// Builder: replace `cmd.setting(DontCollapseArgsInUsage)` with `cmd.dont_collapse_args_in_usage(true)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.1.0",
            note = "Replaced with `Command::dont_collapse_args_in_usage` and `Command::is_dont_collapse_args_in_usage_set`

Derive: replace `#[clap(setting = DontCollapseArgsInUsage)]` with `#[clap(dont_collapse_args_in_usage = true)]`

Builder: replace `cmd.setting(DontCollapseArgsInUsage)` with `cmd.dont_collapse_args_in_usage(true)`
"
        )
    )]
    DontCollapseArgsInUsage,

    /// Deprecated, replaced with [`Command::next_line_help`] and [`Command::is_next_line_help_set`]
    ///
    /// Derive: replace `#[clap(setting = NextLineHelp)]` with `#[clap(next_line_help = true)]`
    ///
    /// Builder: replace `cmd.setting(NextLineHelp)` with `cmd.next_line_help(true)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.1.0",
            note = "Replaced with `Command::next_line_help` and `Command::is_next_line_help_set`

Derive: replace `#[clap(setting = NextLineHelp)]` with `#[clap(next_line_help = true)]`

Builder: replace `cmd.setting(NextLineHelp)` with `cmd.next_line_help(true)`
"
        )
    )]
    NextLineHelp,

    /// Deprecated, replaced with [`Command::disable_colored_help`] and
    /// [`Command::is_disable_colored_help_set`]
    ///
    /// Derive: replace `#[clap(setting = DisableColoredHelp)]` with `#[clap(disable_colored_help = true)]`
    ///
    /// Builder: replace `cmd.setting(DisableColoredHelp)` with `cmd.disable_colored_help(true)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.1.0",
            note = "Replaced with `Command::disable_colored_help` and `Command::is_disable_colored_help_set`

Derive: replace `#[clap(setting = DisableColoredHelp)]` with `#[clap(disable_colored_help = true)]`

Builder: replace `cmd.setting(DisableColoredHelp)` with `cmd.disable_colored_help(true)`
"
        )
    )]
    DisableColoredHelp,

    /// Deprecated, replaced with [`Command::disable_help_flag`] and [`Command::is_disable_help_flag_set`]
    ///
    /// Derive: replace `#[clap(setting = DisableHelpFlag)]` with `#[clap(disable_help_flag = true)]`
    ///
    /// Builder: replace `cmd.setting(DisableHelpFlag)` with `cmd.disable_help_flag(true)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.1.0",
            note = "Replaced with `Command::disable_help_flag` and `Command::is_disable_help_flag_set`

Derive: replace `#[clap(setting = DisableHelpFlag)]` with `#[clap(disable_help_flag = true)]`

Builder: replace `cmd.setting(DisableHelpFlag)` with `cmd.disable_help_flag(true)`
"
        )
    )]
    DisableHelpFlag,

    /// Deprecated, replaced with [`Command::disable_help_subcommand`] and
    /// [`Command::is_disable_help_subcommand_set`]
    ///
    /// Derive: replace `#[clap(setting = DisableHelpSubcommand)]` with `#[clap(disable_help_subcommand = true)]`
    ///
    /// Builder: replace `cmd.setting(DisableHelpSubcommand)` with `cmd.disable_help_subcommand(true)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.1.0",
            note = "Replaced with `Command::disable_help_subcommand` and `Command::is_disable_help_subcommand_set`

Derive: replace `#[clap(setting = DisableHelpSubcommand)]` with `#[clap(disable_help_subcommand = true)]`

Builder: replace `cmd.setting(DisableHelpSubcommand)` with `cmd.disable_help_subcommand(true)`
"
        )
    )]
    DisableHelpSubcommand,

    /// Deprecated, replaced with [`Command::disable_version_flag`] and
    /// [`Command::is_disable_version_flag_set`]
    ///
    /// Derive: replace `#[clap(setting = DisableVersionFlag)]` with `#[clap(disable_version_flag = true)]`
    ///
    /// Builder: replace `cmd.setting(DisableVersionFlag)` with `cmd.disable_version_flag(true)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.1.0",
            note = "Replaced with `Command::disable_version_flag` and `Command::is_disable_version_flag_set`

Derive: replace `#[clap(setting = DisableVersionFlag)]` with `#[clap(disable_version_flag = true)]`

Builder: replace `cmd.setting(DisableVersionFlag)` with `cmd.disable_version_flag(true)`
"
        )
    )]
    DisableVersionFlag,

    /// Deprecated, replaced with [`Command::propagate_version`] and [`Command::is_propagate_version_set`]
    ///
    /// Derive: replace `#[clap(setting = PropagateVersion)]` with `#[clap(propagate_version = true)]`
    ///
    /// Builder: replace `cmd.setting(PropagateVersion)` with `cmd.propagate_version(true)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.1.0",
            note = "Replaced with `Command::propagate_version` and `Command::is_propagate_version_set`

Derive: replace `#[clap(setting = PropagateVersion)]` with `#[clap(propagate_version = true)]`

Builder: replace `cmd.setting(PropagateVersion)` with `cmd.propagate_version(true)`
"
        )
    )]
    PropagateVersion,

    /// Deprecated, replaced with [`Command::hide`] and [`Command::is_hide_set`]
    ///
    /// Derive: replace `#[clap(setting = Hidden)]` with `#[clap(hide = true)]`
    ///
    /// Builder: replace `cmd.setting(Hidden)` with `cmd.hide(true)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.1.0",
            note = "Replaced with `Command::hide` and `Command::is_hide_set`

Derive: replace `#[clap(setting = Hidden)]` with `#[clap(hide = true)]`

Builder: replace `cmd.setting(Hidden)` with `cmd.hide(true)`
"
        )
    )]
    Hidden,

    /// Deprecated, replaced with [`Command::hide_possible_values`] and
    /// [`Arg::is_hide_possible_values_set`]
    ///
    /// Derive: replace `#[clap(setting = HidePossibleValues)]` with `#[clap(hide_possible_values = true)]`
    ///
    /// Builder: replace `cmd.setting(HidePossibleValues)` with `cmd.hide_possible_values(true)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.1.0",
            note = "Replaced with `Command::hide_possible_values` and `Arg::is_hide_possible_values_set`

Derive: replace `#[clap(setting = HidePossibleValues)]` with `#[clap(hide_possible_values = true)]`

Builder: replace `cmd.setting(HidePossibleValues)` with `cmd.hide_possible_values(true)`
"
        )
    )]
    HidePossibleValues,

    /// Deprecated, replaced with [`Command::help_expected`]
    ///
    /// Derive: replace `#[clap(setting = HelpExpected)]` with `#[clap(help_expected = true)]`
    ///
    /// Builder: replace `cmd.setting(HelpExpected)` with `cmd.help_expected(true)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.1.0",
            note = "Replaced with `Command::help_expected`

Derive: replace `#[clap(setting = HelpExpected)]` with `#[clap(help_expected = true)]`

Builder: replace `cmd.setting(HelpExpected)` with `cmd.help_expected(true)`
"
        )
    )]
    HelpExpected,

    /// Deprecated, replaced with [`Command::no_binary_name`]
    ///
    /// Derive: replace `#[clap(setting = NoBinaryName)]` with `#[clap(no_binary_name = true)]`
    ///
    /// Builder: replace `cmd.setting(NoBinaryName)` with `cmd.no_binary_name(true)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.1.0",
            note = "Replaced with `Command::no_binary_name`

Derive: replace `#[clap(setting = NoBinaryName)]` with `#[clap(no_binary_name = true)]`

Builder: replace `cmd.setting(NoBinaryName)` with `cmd.no_binary_name(true)`
"
        )
    )]
    NoBinaryName,

    /// Deprecated, replaced with [`Arg::action`][super::Arg::action]
    ///
    /// Derive: replace `#[clap(setting = NoAutoHelp)]` with setting an explicit action on your help argument
    ///
    /// Builder: replace `cmd.setting(NoAutoHelp)` with setting an explicit action on your help argument
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.2.0",
            note = "Replaced with `Arg::action`

Derive: replace `#[clap(setting = NoAutoHelp)]` with setting an explicit action on your help argument

Builder: replace `cmd.setting(NoAutoHelp)` with setting an explicit action on your help argument
"
        )
    )]
    NoAutoHelp,

    /// Deprecated, replaced with [`Arg::action`][super::Arg::action]
    ///
    /// Derive: replace `#[clap(setting = NoAutoVersion)]` with setting an explicit action on your version argument
    ///
    /// Builder: replace `cmd.setting(NoAutoVersion)` with setting an explicit action on your version argument
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.2.0",
            note = "Replaced with `Arg::action`

Derive: replace `#[clap(setting = NoAutoVersion)]` with setting an explicit action on your version argument

Builder: replace `cmd.setting(NoAutoVersion)` with setting an explicit action on your version argument
"
        )
    )]
    NoAutoVersion,

    /// Deprecated, replaced with [`Command::allow_hyphen_values`]
    ///
    /// Derive: replace `#[clap(setting = AllowLeadingHyphen)]` with `#[clap(allow_hyphen_values = true)]`
    ///
    /// Builder: replace `cmd.setting(AllowLeadingHyphen)` with `cmd.allow_hyphen_values(true)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.0.0",
            note = "Replaced with `Command::allow_hyphen_values`

Derive: replace `#[clap(setting = AllowLeadingHyphen)]` with `#[clap(allow_hyphen_values = true)]`

Builder: replace `cmd.setting(AllowLeadingHyphen)` with `cmd.allow_hyphen_values(true)`
"
        )
    )]
    #[doc(hidden)]
    AllowLeadingHyphen,

    /// Deprecated, replaced with [`Command::allow_invalid_utf8_for_external_subcommands`] and [`Command::is_allow_invalid_utf8_for_external_subcommands_set`]
    ///
    /// Derive: replace `#[clap(setting = StrictUtf8)]` with `#[clap(allow_invalid_utf8_for_external_subcommands = true)]`
    ///
    /// Builder: replace `cmd.setting(StrictUtf8)` with `cmd.allow_invalid_utf8_for_external_subcommands(true)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.0.0",
            note = "Replaced with `Command::allow_invalid_utf8_for_external_subcommands` and `Command::is_allow_invalid_utf8_for_external_subcommands_set`

Derive: replace `#[clap(setting = StrictUtf8)]` with `#[clap(allow_invalid_utf8_for_external_subcommands = true)]`

Builder: replace `cmd.setting(StrictUtf8)` with `cmd.allow_invalid_utf8_for_external_subcommands(true)`
"
        )
    )]
    #[doc(hidden)]
    StrictUtf8,

    /// Deprecated, this is now the default
    ///
    /// Derive: remove `#[clap(setting = UnifiedHelpMessage)]`
    ///
    /// Builder: remove `cmd.setting(UnifiedHelpMessage)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.0.0",
            note = "This is now the default

Derive: remove `#[clap(setting = UnifiedHelpMessage)]`

Builder: remove `cmd.setting(UnifiedHelpMessage)`
"
        )
    )]
    #[doc(hidden)]
    UnifiedHelpMessage,

    /// Deprecated, this is now the default
    ///
    /// Derive: remove `#[clap(setting = ColoredHelp)]`
    ///
    /// Builder: remove `cmd.setting(ColoredHelp)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.0.0",
            note = "This is now the default

Derive: remove `#[clap(setting = ColoredHelp)]`

Builder: remove `cmd.setting(ColoredHelp)`
"
        )
    )]
    #[doc(hidden)]
    ColoredHelp,

    /// Deprecated, see [`Command::color`][crate::Command::color]
    ///
    /// Derive: replace `#[clap(setting = ColorAuto)]` with `#[clap(color = ColorChoice::Auto)]``
    ///
    /// Builder: replace `cmd.setting(ColorAuto)` with `cmd.color(Color::Auto)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.0.0",
            note = "Replaced with `Command::color`

Derive: replace `#[clap(setting = ColorAuto)]` with `#[clap(color = ColorChoice::Auto)]``

Builder: replace `cmd.setting(ColorAuto)` with `cmd.color(Color::Auto)`
"
        )
    )]
    #[doc(hidden)]
    ColorAuto,

    /// Deprecated, replaced with [`Command::color`][crate::Command::color]
    ///
    /// Derive: replace `#[clap(setting = ColorAlways)]` with `#[clap(color = ColorChoice::Always)]``
    ///
    /// Builder: replace `cmd.setting(ColorAlways)` with `cmd.color(Color::Always)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.0.0",
            note = "Replaced with `Command::color`

Derive: replace `#[clap(setting = ColorAlways)]` with `#[clap(color = ColorChoice::Always)]``

Builder: replace `cmd.setting(ColorAlways)` with `cmd.color(Color::Always)`
"
        )
    )]
    #[doc(hidden)]
    ColorAlways,

    /// Deprecated, replaced with [`Command::color`][crate::Command::color]
    ///
    /// Derive: replace `#[clap(setting = ColorNever)]` with `#[clap(color = ColorChoice::Never)]``
    ///
    /// Builder: replace `cmd.setting(ColorNever)` with `cmd.color(Color::Never)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.0.0",
            note = "Replaced with `Command::color`

Derive: replace `#[clap(setting = ColorNever)]` with `#[clap(color = ColorChoice::Never)]``

Builder: replace `cmd.setting(ColorNever)` with `cmd.color(Color::Never)`
"
        )
    )]
    #[doc(hidden)]
    ColorNever,

    /// Deprecated, replaced with [`Command::disable_help_flag`] and [`Command::is_disable_help_flag_set`]
    ///
    /// Derive: replace `#[clap(setting = DisableHelpFlags)]` with `#[clap(disable_help_flag = true)]`
    ///
    /// Builder: replace `cmd.setting(DisableHelpFlags)` with `cmd.disable_help_flag(true)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.0.0",
            note = "Replaced with `Command::disable_help_flag` and `Command::is_disable_help_flag_set`

Derive: replace `#[clap(setting = DisableHelpFlags)]` with `#[clap(disable_help_flag = true)]`

Builder: replace `cmd.setting(DisableHelpFlags)` with `cmd.disable_help_flag(true)`
"
        )
    )]
    #[doc(hidden)]
    DisableHelpFlags,

    /// Deprecated, replaced with [`Command::disable_version_flag`] and
    /// [`Command::is_disable_version_flag_set`]
    ///
    /// Derive: replace `#[clap(setting = DisableVersion)]` with `#[clap(disable_version_flag = true)]`
    ///
    /// Builder: replace `cmd.setting(DisableVersion)` with `cmd.disable_version_flag(true)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.0.0",
            note = "Replaced with `Command::disable_version_flag` and `Command::is_disable_version_flag_set`

Derive: replace `#[clap(setting = DisableVersion)]` with `#[clap(disable_version_flag = true)]`

Builder: replace `cmd.setting(DisableVersion)` with `cmd.disable_version_flag(true)`
"
        )
    )]
    #[doc(hidden)]
    DisableVersion,

    /// Deprecated, replaced with [`Command::propagate_version`] and [`Command::is_propagate_version_set`]
    ///
    /// Derive: replace `#[clap(setting = GlobalVersion)]` with `#[clap(propagate_version = true)]`
    ///
    /// Builder: replace `cmd.setting(GlobalVersion)` with `cmd.propagate_version(true)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.0.0",
            note = "Replaced with `Command::propagate_version` and `Command::is_propagate_version_set`

Derive: replace `#[clap(setting = GlobalVersion)]` with `#[clap(propagate_version = true)]`

Builder: replace `cmd.setting(GlobalVersion)` with `cmd.propagate_version(true)`
"
        )
    )]
    #[doc(hidden)]
    GlobalVersion,

    /// Deprecated, replaced with [`Command::hide_possible_values`] and
    /// [`Arg::is_hide_possible_values_set`]
    ///
    /// Derive: replace `#[clap(setting = HidePossibleValuesInHelp)]` with `#[clap(hide_possible_values = true)]`
    ///
    /// Builder: replace `cmd.setting(HidePossibleValuesInHelp)` with `cmd.hide_possible_values(true)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.0.0",
            note = "Replaced with `Command::hide_possible_values` and `Arg::is_hide_possible_values_set`

Derive: replace `#[clap(setting = HidePossibleValuesInHelp)]` with `#[clap(hide_possible_values = true)]`

Builder: replace `cmd.setting(HidePossibleValuesInHelp)` with `cmd.hide_possible_values(true)`
"
        )
    )]
    #[doc(hidden)]
    HidePossibleValuesInHelp,

    /// Deprecated, this is now the default
    ///
    /// Derive: remove `#[clap(setting = UnifiedHelp)]`
    ///
    /// Builder: remove `cmd.setting(UnifiedHelp)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.0.0",
            note = "This is now the default

Derive: remove `#[clap(setting = UnifiedHelp)]`

Builder: remove `cmd.setting(UnifiedHelp)`
"
        )
    )]
    #[doc(hidden)]
    UnifiedHelp,

    /// If the cmd is already built, used for caching.
    #[doc(hidden)]
    Built,

    /// If the cmd's bin name is already built, used for caching.
    #[doc(hidden)]
    BinNameBuilt,
}

bitflags! {
    struct Flags: u64 {
        const SC_NEGATE_REQS                 = 1;
        const SC_REQUIRED                    = 1 << 1;
        const ARG_REQUIRED_ELSE_HELP         = 1 << 2;
        const PROPAGATE_VERSION              = 1 << 3;
        const DISABLE_VERSION_FOR_SC         = 1 << 4;
        const WAIT_ON_ERROR                  = 1 << 6;
        const SC_REQUIRED_ELSE_HELP          = 1 << 7;
        const NO_AUTO_HELP                   = 1 << 8;
        const NO_AUTO_VERSION                = 1 << 9;
        const DISABLE_VERSION_FLAG           = 1 << 10;
        const HIDDEN                         = 1 << 11;
        const TRAILING_VARARG                = 1 << 12;
        const NO_BIN_NAME                    = 1 << 13;
        const ALLOW_UNK_SC                   = 1 << 14;
        const SC_UTF8_NONE                   = 1 << 15;
        const LEADING_HYPHEN                 = 1 << 16;
        const NO_POS_VALUES                  = 1 << 17;
        const NEXT_LINE_HELP                 = 1 << 18;
        const DERIVE_DISP_ORDER              = 1 << 19;
        const DISABLE_COLORED_HELP           = 1 << 20;
        const COLOR_ALWAYS                   = 1 << 21;
        const COLOR_AUTO                     = 1 << 22;
        const COLOR_NEVER                    = 1 << 23;
        const DONT_DELIM_TRAIL               = 1 << 24;
        const ALLOW_NEG_NUMS                 = 1 << 25;
        const DISABLE_HELP_SC                = 1 << 27;
        const DONT_COLLAPSE_ARGS             = 1 << 28;
        const ARGS_NEGATE_SCS                = 1 << 29;
        const PROPAGATE_VALS_DOWN            = 1 << 30;
        const ALLOW_MISSING_POS              = 1 << 31;
        const TRAILING_VALUES                = 1 << 32;
        const BUILT                          = 1 << 33;
        const BIN_NAME_BUILT                 = 1 << 34;
        const VALID_ARG_FOUND                = 1 << 35;
        const INFER_SUBCOMMANDS              = 1 << 36;
        const CONTAINS_LAST                  = 1 << 37;
        const ARGS_OVERRIDE_SELF             = 1 << 38;
        const HELP_REQUIRED                  = 1 << 39;
        const SUBCOMMAND_PRECEDENCE_OVER_ARG = 1 << 40;
        const DISABLE_HELP_FLAG              = 1 << 41;
        const USE_LONG_FORMAT_FOR_HELP_SC    = 1 << 42;
        const INFER_LONG_ARGS                = 1 << 43;
        const IGNORE_ERRORS                  = 1 << 44;
        const MULTICALL                      = 1 << 45;
        const NO_OP                          = 0;
    }
}

impl_settings! { AppSettings, AppFlags,
    ArgRequiredElseHelp
        => Flags::ARG_REQUIRED_ELSE_HELP,
    SubcommandPrecedenceOverArg
        => Flags::SUBCOMMAND_PRECEDENCE_OVER_ARG,
    ArgsNegateSubcommands
        => Flags::ARGS_NEGATE_SCS,
    AllowExternalSubcommands
        => Flags::ALLOW_UNK_SC,
    StrictUtf8
        => Flags::NO_OP,
    AllowInvalidUtf8ForExternalSubcommands
        => Flags::SC_UTF8_NONE,
    AllowHyphenValues
        => Flags::LEADING_HYPHEN,
    AllowLeadingHyphen
        => Flags::LEADING_HYPHEN,
    AllowNegativeNumbers
        => Flags::ALLOW_NEG_NUMS,
    AllowMissingPositional
        => Flags::ALLOW_MISSING_POS,
    UnifiedHelpMessage
        => Flags::NO_OP,
    ColoredHelp
        => Flags::NO_OP,
    ColorAlways
        => Flags::COLOR_ALWAYS,
    ColorAuto
        => Flags::COLOR_AUTO,
    ColorNever
        => Flags::COLOR_NEVER,
    DontDelimitTrailingValues
        => Flags::DONT_DELIM_TRAIL,
    DontCollapseArgsInUsage
        => Flags::DONT_COLLAPSE_ARGS,
    DeriveDisplayOrder
        => Flags::DERIVE_DISP_ORDER,
    DisableColoredHelp
        => Flags::DISABLE_COLORED_HELP,
    DisableHelpSubcommand
        => Flags::DISABLE_HELP_SC,
    DisableHelpFlag
        => Flags::DISABLE_HELP_FLAG,
    DisableHelpFlags
        => Flags::DISABLE_HELP_FLAG,
    DisableVersionFlag
        => Flags::DISABLE_VERSION_FLAG,
    DisableVersion
        => Flags::DISABLE_VERSION_FLAG,
    PropagateVersion
        => Flags::PROPAGATE_VERSION,
    GlobalVersion
        => Flags::PROPAGATE_VERSION,
    HidePossibleValues
        => Flags::NO_POS_VALUES,
    HidePossibleValuesInHelp
        => Flags::NO_POS_VALUES,
    HelpExpected
        => Flags::HELP_REQUIRED,
    Hidden
        => Flags::HIDDEN,
    Multicall
        => Flags::MULTICALL,
    NoAutoHelp
        => Flags::NO_AUTO_HELP,
    NoAutoVersion
        => Flags::NO_AUTO_VERSION,
    NoBinaryName
        => Flags::NO_BIN_NAME,
    SubcommandsNegateReqs
        => Flags::SC_NEGATE_REQS,
    SubcommandRequired
        => Flags::SC_REQUIRED,
    SubcommandRequiredElseHelp
        => Flags::SC_REQUIRED_ELSE_HELP,
    UseLongFormatForHelpSubcommand
        => Flags::USE_LONG_FORMAT_FOR_HELP_SC,
    TrailingVarArg
        => Flags::TRAILING_VARARG,
    UnifiedHelp => Flags::NO_OP,
    NextLineHelp
        => Flags::NEXT_LINE_HELP,
    IgnoreErrors
        => Flags::IGNORE_ERRORS,
    WaitOnError
        => Flags::WAIT_ON_ERROR,
    Built
        => Flags::BUILT,
    BinNameBuilt
        => Flags::BIN_NAME_BUILT,
    InferSubcommands
        => Flags::INFER_SUBCOMMANDS,
    AllArgsOverrideSelf
        => Flags::ARGS_OVERRIDE_SELF,
    InferLongArgs
        => Flags::INFER_LONG_ARGS
}

/// Deprecated in [Issue #3087](https://github.com/clap-rs/clap/issues/3087), maybe [`clap::Parser`][crate::Parser] would fit your use case?
#[cfg(feature = "yaml")]
impl FromStr for AppSettings {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        #[allow(deprecated)]
        #[allow(unreachable_patterns)]
        match &*s.to_ascii_lowercase() {
            "argrequiredelsehelp" => Ok(AppSettings::ArgRequiredElseHelp),
            "subcommandprecedenceoverarg" => Ok(AppSettings::SubcommandPrecedenceOverArg),
            "argsnegatesubcommands" => Ok(AppSettings::ArgsNegateSubcommands),
            "allowexternalsubcommands" => Ok(AppSettings::AllowExternalSubcommands),
            "strictutf8" => Ok(AppSettings::StrictUtf8),
            "allowinvalidutf8forexternalsubcommands" => {
                Ok(AppSettings::AllowInvalidUtf8ForExternalSubcommands)
            }
            "allowhyphenvalues" => Ok(AppSettings::AllowHyphenValues),
            "allowleadinghyphen" => Ok(AppSettings::AllowLeadingHyphen),
            "allownegativenumbers" => Ok(AppSettings::AllowNegativeNumbers),
            "allowmissingpositional" => Ok(AppSettings::AllowMissingPositional),
            "unifiedhelpmessage" => Ok(AppSettings::UnifiedHelpMessage),
            "coloredhelp" => Ok(AppSettings::ColoredHelp),
            "coloralways" => Ok(AppSettings::ColorAlways),
            "colorauto" => Ok(AppSettings::ColorAuto),
            "colornever" => Ok(AppSettings::ColorNever),
            "dontdelimittrailingvalues" => Ok(AppSettings::DontDelimitTrailingValues),
            "dontcollapseargsinusage" => Ok(AppSettings::DontCollapseArgsInUsage),
            "derivedisplayorder" => Ok(AppSettings::DeriveDisplayOrder),
            "disablecoloredhelp" => Ok(AppSettings::DisableColoredHelp),
            "disablehelpsubcommand" => Ok(AppSettings::DisableHelpSubcommand),
            "disablehelpflag" => Ok(AppSettings::DisableHelpFlag),
            "disablehelpflags" => Ok(AppSettings::DisableHelpFlags),
            "disableversionflag" => Ok(AppSettings::DisableVersionFlag),
            "disableversion" => Ok(AppSettings::DisableVersion),
            "propagateversion" => Ok(AppSettings::PropagateVersion),
            "propagateversion" => Ok(AppSettings::GlobalVersion),
            "hidepossiblevalues" => Ok(AppSettings::HidePossibleValues),
            "hidepossiblevaluesinhelp" => Ok(AppSettings::HidePossibleValuesInHelp),
            "helpexpected" => Ok(AppSettings::HelpExpected),
            "hidden" => Ok(AppSettings::Hidden),
            "noautohelp" => Ok(AppSettings::NoAutoHelp),
            "noautoversion" => Ok(AppSettings::NoAutoVersion),
            "nobinaryname" => Ok(AppSettings::NoBinaryName),
            "subcommandsnegatereqs" => Ok(AppSettings::SubcommandsNegateReqs),
            "subcommandrequired" => Ok(AppSettings::SubcommandRequired),
            "subcommandrequiredelsehelp" => Ok(AppSettings::SubcommandRequiredElseHelp),
            "uselongformatforhelpsubcommand" => Ok(AppSettings::UseLongFormatForHelpSubcommand),
            "trailingvararg" => Ok(AppSettings::TrailingVarArg),
            "unifiedhelp" => Ok(AppSettings::UnifiedHelp),
            "nextlinehelp" => Ok(AppSettings::NextLineHelp),
            "ignoreerrors" => Ok(AppSettings::IgnoreErrors),
            "waitonerror" => Ok(AppSettings::WaitOnError),
            "built" => Ok(AppSettings::Built),
            "binnamebuilt" => Ok(AppSettings::BinNameBuilt),
            "infersubcommands" => Ok(AppSettings::InferSubcommands),
            "allargsoverrideself" => Ok(AppSettings::AllArgsOverrideSelf),
            "inferlongargs" => Ok(AppSettings::InferLongArgs),
            _ => Err(format!("unknown AppSetting: `{}`", s)),
        }
    }
}

#[cfg(test)]
mod test {
    #[allow(clippy::cognitive_complexity)]
    #[test]
    #[cfg(feature = "yaml")]
    fn app_settings_fromstr() {
        use super::AppSettings;

        assert_eq!(
            "disablehelpflag".parse::<AppSettings>().unwrap(),
            AppSettings::DisableHelpFlag
        );
        assert_eq!(
            "argsnegatesubcommands".parse::<AppSettings>().unwrap(),
            AppSettings::ArgsNegateSubcommands
        );
        assert_eq!(
            "argrequiredelsehelp".parse::<AppSettings>().unwrap(),
            AppSettings::ArgRequiredElseHelp
        );
        assert_eq!(
            "subcommandprecedenceoverarg"
                .parse::<AppSettings>()
                .unwrap(),
            AppSettings::SubcommandPrecedenceOverArg
        );
        assert_eq!(
            "allowexternalsubcommands".parse::<AppSettings>().unwrap(),
            AppSettings::AllowExternalSubcommands
        );
        assert_eq!(
            "allowinvalidutf8forexternalsubcommands"
                .parse::<AppSettings>()
                .unwrap(),
            AppSettings::AllowInvalidUtf8ForExternalSubcommands
        );
        assert_eq!(
            "allowhyphenvalues".parse::<AppSettings>().unwrap(),
            AppSettings::AllowHyphenValues
        );
        assert_eq!(
            "allownegativenumbers".parse::<AppSettings>().unwrap(),
            AppSettings::AllowNegativeNumbers
        );
        assert_eq!(
            "disablehelpsubcommand".parse::<AppSettings>().unwrap(),
            AppSettings::DisableHelpSubcommand
        );
        assert_eq!(
            "disableversionflag".parse::<AppSettings>().unwrap(),
            AppSettings::DisableVersionFlag
        );
        assert_eq!(
            "dontcollapseargsinusage".parse::<AppSettings>().unwrap(),
            AppSettings::DontCollapseArgsInUsage
        );
        assert_eq!(
            "dontdelimittrailingvalues".parse::<AppSettings>().unwrap(),
            AppSettings::DontDelimitTrailingValues
        );
        assert_eq!(
            "derivedisplayorder".parse::<AppSettings>().unwrap(),
            AppSettings::DeriveDisplayOrder
        );
        assert_eq!(
            "disablecoloredhelp".parse::<AppSettings>().unwrap(),
            AppSettings::DisableColoredHelp
        );
        assert_eq!(
            "propagateversion".parse::<AppSettings>().unwrap(),
            AppSettings::PropagateVersion
        );
        assert_eq!(
            "hidden".parse::<AppSettings>().unwrap(),
            AppSettings::Hidden
        );
        assert_eq!(
            "hidepossiblevalues".parse::<AppSettings>().unwrap(),
            AppSettings::HidePossibleValues
        );
        assert_eq!(
            "helpexpected".parse::<AppSettings>().unwrap(),
            AppSettings::HelpExpected
        );
        assert_eq!(
            "nobinaryname".parse::<AppSettings>().unwrap(),
            AppSettings::NoBinaryName
        );
        assert_eq!(
            "nextlinehelp".parse::<AppSettings>().unwrap(),
            AppSettings::NextLineHelp
        );
        assert_eq!(
            "subcommandsnegatereqs".parse::<AppSettings>().unwrap(),
            AppSettings::SubcommandsNegateReqs
        );
        assert_eq!(
            "subcommandrequired".parse::<AppSettings>().unwrap(),
            AppSettings::SubcommandRequired
        );
        assert_eq!(
            "subcommandrequiredelsehelp".parse::<AppSettings>().unwrap(),
            AppSettings::SubcommandRequiredElseHelp
        );
        assert_eq!(
            "uselongformatforhelpsubcommand"
                .parse::<AppSettings>()
                .unwrap(),
            AppSettings::UseLongFormatForHelpSubcommand
        );
        assert_eq!(
            "trailingvararg".parse::<AppSettings>().unwrap(),
            AppSettings::TrailingVarArg
        );
        assert_eq!(
            "waitonerror".parse::<AppSettings>().unwrap(),
            AppSettings::WaitOnError
        );
        assert_eq!("built".parse::<AppSettings>().unwrap(), AppSettings::Built);
        assert_eq!(
            "binnamebuilt".parse::<AppSettings>().unwrap(),
            AppSettings::BinNameBuilt
        );
        assert_eq!(
            "infersubcommands".parse::<AppSettings>().unwrap(),
            AppSettings::InferSubcommands
        );
        assert!("hahahaha".parse::<AppSettings>().is_err());
    }
}
