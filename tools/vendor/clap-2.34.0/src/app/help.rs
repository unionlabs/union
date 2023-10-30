// Std
use std::{
    borrow::Cow,
    cmp,
    collections::BTreeMap,
    fmt::Display,
    io::{self, Cursor, Read, Write},
    usize,
};

// Third Party
#[cfg(feature = "wrap_help")]
use term_size;
#[cfg(feature = "wrap_help")]
use textwrap;
use unicode_width::UnicodeWidthStr;

// Internal
use crate::{
    app::{parser::Parser, usage, App, AppSettings},
    args::{AnyArg, ArgSettings, DispOrder},
    errors::{Error, Result as ClapResult},
    fmt::{Colorizer, ColorizerOption, Format},
    map::VecMap,
    INTERNAL_ERROR_MSG,
};

#[cfg(not(feature = "wrap_help"))]
mod term_size {
    pub fn dimensions() -> Option<(usize, usize)> {
        None
    }
}

fn str_width(s: &str) -> usize {
    UnicodeWidthStr::width(s)
}

const TAB: &str = "    ";

// These are just convenient traits to make the code easier to read.
trait ArgWithDisplay<'b, 'c>: AnyArg<'b, 'c> + Display {}
impl<'b, 'c, T> ArgWithDisplay<'b, 'c> for T where T: AnyArg<'b, 'c> + Display {}

trait ArgWithOrder<'b, 'c>: ArgWithDisplay<'b, 'c> + DispOrder {
    fn as_base(&self) -> &ArgWithDisplay<'b, 'c>;
}
impl<'b, 'c, T> ArgWithOrder<'b, 'c> for T
where
    T: ArgWithDisplay<'b, 'c> + DispOrder,
{
    fn as_base(&self) -> &ArgWithDisplay<'b, 'c> {
        self
    }
}

fn as_arg_trait<'a, 'b, T: ArgWithOrder<'a, 'b>>(x: &T) -> &ArgWithOrder<'a, 'b> {
    x
}

impl<'b, 'c> DispOrder for App<'b, 'c> {
    fn disp_ord(&self) -> usize {
        999
    }
}

macro_rules! color {
    ($_self:ident, $s:expr, $c:ident) => {
        if $_self.color {
            write!($_self.writer, "{}", $_self.cizer.$c($s))
        } else {
            write!($_self.writer, "{}", $s)
        }
    };
    ($_self:ident, $fmt_s:expr, $v:expr, $c:ident) => {
        if $_self.color {
            write!($_self.writer, "{}", $_self.cizer.$c(format!($fmt_s, $v)))
        } else {
            write!($_self.writer, $fmt_s, $v)
        }
    };
}

/// `clap` Help Writer.
///
/// Wraps a writer stream providing different methods to generate help for `clap` objects.
pub struct Help<'a> {
    writer: &'a mut Write,
    next_line_help: bool,
    hide_pv: bool,
    term_w: usize,
    color: bool,
    cizer: Colorizer,
    longest: usize,
    force_next_line: bool,
    use_long: bool,
}

// Public Functions
impl<'a> Help<'a> {
    /// Create a new `Help` instance.
    #[cfg_attr(feature = "cargo-clippy", allow(clippy::too_many_arguments))]
    pub fn new(
        w: &'a mut Write,
        next_line_help: bool,
        hide_pv: bool,
        color: bool,
        cizer: Colorizer,
        term_w: Option<usize>,
        max_w: Option<usize>,
        use_long: bool,
    ) -> Self {
        debugln!("Help::new;");
        Help {
            writer: w,
            next_line_help,
            hide_pv,
            term_w: match term_w {
                Some(width) => {
                    if width == 0 {
                        usize::MAX
                    } else {
                        width
                    }
                }
                None => cmp::min(
                    term_size::dimensions().map_or(120, |(w, _)| w),
                    match max_w {
                        None | Some(0) => usize::MAX,
                        Some(mw) => mw,
                    },
                ),
            },
            color,
            cizer,
            longest: 0,
            force_next_line: false,
            use_long,
        }
    }

    /// Reads help settings from an App
    /// and write its help to the wrapped stream.
    pub fn write_app_help(w: &'a mut Write, app: &App, use_long: bool) -> ClapResult<()> {
        debugln!("Help::write_app_help;");
        Self::write_parser_help(w, &app.p, use_long)
    }

    /// Reads help settings from a Parser
    /// and write its help to the wrapped stream.
    pub fn write_parser_help(w: &'a mut Write, parser: &Parser, use_long: bool) -> ClapResult<()> {
        debugln!("Help::write_parser_help;");
        Self::_write_parser_help(w, parser, false, use_long)
    }

    /// Reads help settings from a Parser
    /// and write its help to the wrapped stream which will be stderr. This method prevents
    /// formatting when required.
    pub fn write_parser_help_to_stderr(w: &'a mut Write, parser: &Parser) -> ClapResult<()> {
        debugln!("Help::write_parser_help;");
        Self::_write_parser_help(w, parser, true, false)
    }

    #[doc(hidden)]
    pub fn _write_parser_help(
        w: &'a mut Write,
        parser: &Parser,
        stderr: bool,
        use_long: bool,
    ) -> ClapResult<()> {
        debugln!("Help::write_parser_help;");
        let nlh = parser.is_set(AppSettings::NextLineHelp);
        let hide_v = parser.is_set(AppSettings::HidePossibleValuesInHelp);
        let color = parser.is_set(AppSettings::ColoredHelp);
        let cizer = Colorizer::new(ColorizerOption {
            use_stderr: stderr,
            when: parser.color(),
        });
        Self::new(
            w,
            nlh,
            hide_v,
            color,
            cizer,
            parser.meta.term_w,
            parser.meta.max_w,
            use_long,
        )
        .write_help(parser)
    }

    /// Writes the parser help to the wrapped stream.
    pub fn write_help(&mut self, parser: &Parser) -> ClapResult<()> {
        debugln!("Help::write_help;");
        if let Some(h) = parser.meta.help_str {
            write!(self.writer, "{}", h).map_err(Error::from)?;
        } else if let Some(tmpl) = parser.meta.template {
            self.write_templated_help(parser, tmpl)?;
        } else {
            self.write_default_help(parser)?;
        }
        Ok(())
    }
}

// Methods to write AnyArg help.
impl<'a> Help<'a> {
    /// Writes help for each argument in the order they were declared to the wrapped stream.
    fn write_args_unsorted<'b: 'd, 'c: 'd, 'd, I: 'd>(&mut self, args: I) -> io::Result<()>
    where
        I: Iterator<Item = &'d ArgWithOrder<'b, 'c>>,
    {
        debugln!("Help::write_args_unsorted;");
        // The shortest an arg can legally be is 2 (i.e. '-x')
        self.longest = 2;
        let mut arg_v = Vec::with_capacity(10);
        let use_long = self.use_long;
        for arg in args.filter(|arg| should_show_arg(use_long, *arg)) {
            if arg.longest_filter() {
                self.longest = cmp::max(self.longest, str_width(arg.to_string().as_str()));
            }
            arg_v.push(arg)
        }
        let mut first = true;
        for arg in arg_v {
            if first {
                first = false;
            } else {
                self.writer.write_all(b"\n")?;
            }
            self.write_arg(arg.as_base())?;
        }
        Ok(())
    }

    /// Sorts arguments by length and display order and write their help to the wrapped stream.
    fn write_args<'b: 'd, 'c: 'd, 'd, I: 'd>(&mut self, args: I) -> io::Result<()>
    where
        I: Iterator<Item = &'d ArgWithOrder<'b, 'c>>,
    {
        debugln!("Help::write_args;");
        // The shortest an arg can legally be is 2 (i.e. '-x')
        self.longest = 2;
        let mut ord_m = VecMap::new();
        let use_long = self.use_long;
        // Determine the longest
        for arg in args.filter(|arg| {
            // If it's NextLineHelp, but we don't care to compute how long because it may be
            // NextLineHelp on purpose *because* it's so long and would throw off all other
            // args alignment
            should_show_arg(use_long, *arg)
        }) {
            if arg.longest_filter() {
                debugln!("Help::write_args: Current Longest...{}", self.longest);
                self.longest = cmp::max(self.longest, str_width(arg.to_string().as_str()));
                debugln!("Help::write_args: New Longest...{}", self.longest);
            }
            let btm = ord_m.entry(arg.disp_ord()).or_insert(BTreeMap::new());
            btm.insert(arg.name(), arg);
        }
        let mut first = true;
        for btm in ord_m.values() {
            for arg in btm.values() {
                if first {
                    first = false;
                } else {
                    self.writer.write_all(b"\n")?;
                }
                self.write_arg(arg.as_base())?;
            }
        }
        Ok(())
    }

    /// Writes help for an argument to the wrapped stream.
    fn write_arg<'b, 'c>(&mut self, arg: &ArgWithDisplay<'b, 'c>) -> io::Result<()> {
        debugln!("Help::write_arg;");
        self.short(arg)?;
        self.long(arg)?;
        let spec_vals = self.val(arg)?;
        self.help(arg, &*spec_vals)?;
        Ok(())
    }

    /// Writes argument's short command to the wrapped stream.
    fn short<'b, 'c>(&mut self, arg: &ArgWithDisplay<'b, 'c>) -> io::Result<()> {
        debugln!("Help::short;");
        write!(self.writer, "{}", TAB)?;
        if let Some(s) = arg.short() {
            color!(self, "-{}", s, good)
        } else if arg.has_switch() {
            write!(self.writer, "{}", TAB)
        } else {
            Ok(())
        }
    }

    /// Writes argument's long command to the wrapped stream.
    fn long<'b, 'c>(&mut self, arg: &ArgWithDisplay<'b, 'c>) -> io::Result<()> {
        debugln!("Help::long;");
        if !arg.has_switch() {
            return Ok(());
        }
        if arg.takes_value() {
            if let Some(l) = arg.long() {
                if arg.short().is_some() {
                    write!(self.writer, ", ")?;
                }
                color!(self, "--{}", l, good)?
            }

            let sep = if arg.is_set(ArgSettings::RequireEquals) {
                "="
            } else {
                " "
            };
            write!(self.writer, "{}", sep)?;
        } else if let Some(l) = arg.long() {
            if arg.short().is_some() {
                write!(self.writer, ", ")?;
            }
            color!(self, "--{}", l, good)?;
        }
        Ok(())
    }

    /// Writes argument's possible values to the wrapped stream.
    fn val<'b, 'c>(&mut self, arg: &ArgWithDisplay<'b, 'c>) -> Result<String, io::Error> {
        debugln!("Help::val: arg={}", arg);
        if arg.takes_value() {
            let delim = if arg.is_set(ArgSettings::RequireDelimiter) {
                arg.val_delim().expect(INTERNAL_ERROR_MSG)
            } else {
                ' '
            };
            if let Some(vec) = arg.val_names() {
                let mut it = vec.iter().peekable();
                while let Some((_, val)) = it.next() {
                    color!(self, "<{}>", val, good)?;
                    if it.peek().is_some() {
                        write!(self.writer, "{}", delim)?;
                    }
                }
                let num = vec.len();
                if arg.is_set(ArgSettings::Multiple) && num == 1 {
                    color!(self, "...", good)?;
                }
            } else if let Some(num) = arg.num_vals() {
                let mut it = (0..num).peekable();
                while let Some(_) = it.next() {
                    color!(self, "<{}>", arg.name(), good)?;
                    if it.peek().is_some() {
                        write!(self.writer, "{}", delim)?;
                    }
                }
                if arg.is_set(ArgSettings::Multiple) && num == 1 {
                    color!(self, "...", good)?;
                }
            } else if arg.has_switch() {
                color!(self, "<{}>", arg.name(), good)?;
                if arg.is_set(ArgSettings::Multiple) {
                    color!(self, "...", good)?;
                }
            } else {
                color!(self, "{}", arg, good)?;
            }
        }

        let spec_vals = self.spec_vals(arg);
        let h = arg.help().unwrap_or("");
        let h_w = str_width(h) + str_width(&*spec_vals);
        let nlh = self.next_line_help || arg.is_set(ArgSettings::NextLineHelp);
        let taken = self.longest + 12;
        self.force_next_line = !nlh
            && self.term_w >= taken
            && (taken as f32 / self.term_w as f32) > 0.40
            && h_w > (self.term_w - taken);

        debug!("Help::val: Has switch...");
        if arg.has_switch() {
            sdebugln!("Yes");
            debugln!("Help::val: force_next_line...{:?}", self.force_next_line);
            debugln!("Help::val: nlh...{:?}", nlh);
            debugln!("Help::val: taken...{}", taken);
            debugln!(
                "Help::val: help_width > (width - taken)...{} > ({} - {})",
                h_w,
                self.term_w,
                taken
            );
            debugln!("Help::val: longest...{}", self.longest);
            debug!("Help::val: next_line...");
            if !(nlh || self.force_next_line) {
                sdebugln!("No");
                let self_len = str_width(arg.to_string().as_str());
                // subtract ourself
                let mut spcs = self.longest - self_len;
                // Since we're writing spaces from the tab point we first need to know if we
                // had a long and short, or just short
                if arg.long().is_some() {
                    // Only account 4 after the val
                    spcs += 4;
                } else {
                    // Only account for ', --' + 4 after the val
                    spcs += 8;
                }

                write_nspaces!(self.writer, spcs);
            } else {
                sdebugln!("Yes");
            }
        } else if !(nlh || self.force_next_line) {
            sdebugln!("No, and not next_line");
            write_nspaces!(
                self.writer,
                self.longest + 4 - (str_width(arg.to_string().as_str()))
            );
        } else {
            sdebugln!("No");
        }
        Ok(spec_vals)
    }

    fn write_before_after_help(&mut self, h: &str) -> io::Result<()> {
        debugln!("Help::write_before_after_help;");
        let mut help = String::from(h);
        // determine if our help fits or needs to wrap
        debugln!(
            "Help::write_before_after_help: Term width...{}",
            self.term_w
        );
        let too_long = str_width(h) >= self.term_w;

        debug!("Help::write_before_after_help: Too long...");
        if too_long || h.contains("{n}") {
            sdebugln!("Yes");
            debugln!("Help::write_before_after_help: help: {}", help);
            debugln!(
                "Help::write_before_after_help: help width: {}",
                str_width(&*help)
            );
            // Determine how many newlines we need to insert
            debugln!(
                "Help::write_before_after_help: Usable space: {}",
                self.term_w
            );
            help = wrap_help(&help.replace("{n}", "\n"), self.term_w);
        } else {
            sdebugln!("No");
        }
        write!(self.writer, "{}", help)?;
        Ok(())
    }

    /// Writes argument's help to the wrapped stream.
    fn help<'b, 'c>(&mut self, arg: &ArgWithDisplay<'b, 'c>, spec_vals: &str) -> io::Result<()> {
        debugln!("Help::help;");
        let h = if self.use_long && arg.name() != "" {
            arg.long_help().unwrap_or_else(|| arg.help().unwrap_or(""))
        } else {
            arg.help().unwrap_or_else(|| arg.long_help().unwrap_or(""))
        };
        let mut help = String::from(h) + spec_vals;
        let nlh = self.next_line_help
            || arg.is_set(ArgSettings::NextLineHelp)
            || (self.use_long && arg.name() != "");
        debugln!("Help::help: Next Line...{:?}", nlh);

        let spcs = if nlh || self.force_next_line {
            12 // "tab" * 3
        } else {
            self.longest + 12
        };

        let too_long = spcs + str_width(h) + str_width(&*spec_vals) >= self.term_w;

        // Is help on next line, if so then indent
        if nlh || self.force_next_line {
            write!(self.writer, "\n{}{}{}", TAB, TAB, TAB)?;
        }

        debug!("Help::help: Too long...");
        if too_long && spcs <= self.term_w || h.contains("{n}") {
            sdebugln!("Yes");
            debugln!("Help::help: help...{}", help);
            debugln!("Help::help: help width...{}", str_width(&*help));
            // Determine how many newlines we need to insert
            let avail_chars = self.term_w - spcs;
            debugln!("Help::help: Usable space...{}", avail_chars);
            help = wrap_help(&help.replace("{n}", "\n"), avail_chars);
        } else {
            sdebugln!("No");
        }
        if let Some(part) = help.lines().next() {
            write!(self.writer, "{}", part)?;
        }
        for part in help.lines().skip(1) {
            writeln!(self.writer)?;
            if nlh || self.force_next_line {
                write!(self.writer, "{}{}{}", TAB, TAB, TAB)?;
            } else if arg.has_switch() {
                write_nspaces!(self.writer, self.longest + 12);
            } else {
                write_nspaces!(self.writer, self.longest + 8);
            }
            write!(self.writer, "{}", part)?;
        }
        if !help.contains('\n') && (nlh || self.force_next_line) {
            writeln!(self.writer)?;
        }
        Ok(())
    }

    fn spec_vals(&self, a: &ArgWithDisplay) -> String {
        debugln!("Help::spec_vals: a={}", a);
        let mut spec_vals = vec![];
        if let Some(ref env) = a.env() {
            debugln!(
                "Help::spec_vals: Found environment variable...[{:?}:{:?}]",
                env.0,
                env.1
            );
            let env_val = if !a.is_set(ArgSettings::HideEnvValues) {
                format!(
                    "={}",
                    env.1.map_or(Cow::Borrowed(""), |val| val.to_string_lossy())
                )
            } else {
                String::new()
            };
            let env_info = format!(" [env: {}{}]", env.0.to_string_lossy(), env_val);
            spec_vals.push(env_info);
        }
        if !a.is_set(ArgSettings::HideDefaultValue) {
            if let Some(pv) = a.default_val() {
                debugln!("Help::spec_vals: Found default value...[{:?}]", pv);
                spec_vals.push(format!(
                    " [default: {}]",
                    if self.color {
                        self.cizer.good(pv.to_string_lossy())
                    } else {
                        Format::None(pv.to_string_lossy())
                    }
                ));
            }
        }
        if let Some(ref aliases) = a.aliases() {
            debugln!("Help::spec_vals: Found aliases...{:?}", aliases);
            spec_vals.push(format!(
                " [aliases: {}]",
                if self.color {
                    aliases
                        .iter()
                        .map(|v| format!("{}", self.cizer.good(v)))
                        .collect::<Vec<_>>()
                        .join(", ")
                } else {
                    aliases.join(", ")
                }
            ));
        }
        if !self.hide_pv && !a.is_set(ArgSettings::HidePossibleValues) {
            if let Some(pv) = a.possible_vals() {
                debugln!("Help::spec_vals: Found possible vals...{:?}", pv);
                spec_vals.push(if self.color {
                    format!(
                        " [possible values: {}]",
                        pv.iter()
                            .map(|v| format!("{}", self.cizer.good(v)))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                } else {
                    format!(" [possible values: {}]", pv.join(", "))
                });
            }
        }
        spec_vals.join(" ")
    }
}

fn should_show_arg(use_long: bool, arg: &ArgWithOrder) -> bool {
    if arg.is_set(ArgSettings::Hidden) {
        return false;
    }

    (!arg.is_set(ArgSettings::HiddenLongHelp) && use_long)
        || (!arg.is_set(ArgSettings::HiddenShortHelp) && !use_long)
        || arg.is_set(ArgSettings::NextLineHelp)
}

// Methods to write Parser help.
impl<'a> Help<'a> {
    /// Writes help for all arguments (options, flags, args, subcommands)
    /// including titles of a Parser Object to the wrapped stream.
    pub fn write_all_args(&mut self, parser: &Parser) -> ClapResult<()> {
        debugln!("Help::write_all_args;");
        let flags = parser.has_flags();
        let pos = parser
            .positionals()
            .filter(|arg| !arg.is_set(ArgSettings::Hidden))
            .count()
            > 0;
        let opts = parser.has_opts();
        let subcmds = parser.has_visible_subcommands();

        let unified_help = parser.is_set(AppSettings::UnifiedHelpMessage);

        let mut first = true;

        if unified_help && (flags || opts) {
            let opts_flags = parser
                .flags()
                .map(as_arg_trait)
                .chain(parser.opts().map(as_arg_trait));
            color!(self, "OPTIONS:\n", warning)?;
            self.write_args(opts_flags)?;
            first = false;
        } else {
            if flags {
                color!(self, "FLAGS:\n", warning)?;
                self.write_args(parser.flags().map(as_arg_trait))?;
                first = false;
            }
            if opts {
                if !first {
                    self.writer.write_all(b"\n\n")?;
                }
                color!(self, "OPTIONS:\n", warning)?;
                self.write_args(parser.opts().map(as_arg_trait))?;
                first = false;
            }
        }

        if pos {
            if !first {
                self.writer.write_all(b"\n\n")?;
            }
            color!(self, "ARGS:\n", warning)?;
            self.write_args_unsorted(parser.positionals().map(as_arg_trait))?;
            first = false;
        }

        if subcmds {
            if !first {
                self.writer.write_all(b"\n\n")?;
            }
            color!(self, "SUBCOMMANDS:\n", warning)?;
            self.write_subcommands(parser)?;
        }

        Ok(())
    }

    /// Writes help for subcommands of a Parser Object to the wrapped stream.
    fn write_subcommands(&mut self, parser: &Parser) -> io::Result<()> {
        debugln!("Help::write_subcommands;");
        // The shortest an arg can legally be is 2 (i.e. '-x')
        self.longest = 2;
        let mut ord_m = VecMap::new();
        for sc in parser
            .subcommands
            .iter()
            .filter(|s| !s.p.is_set(AppSettings::Hidden))
        {
            let btm = ord_m.entry(sc.p.meta.disp_ord).or_insert(BTreeMap::new());
            self.longest = cmp::max(self.longest, str_width(sc.p.meta.name.as_str()));
            //self.longest = cmp::max(self.longest, sc.p.meta.name.len());
            btm.insert(sc.p.meta.name.clone(), sc.clone());
        }

        let mut first = true;
        for btm in ord_m.values() {
            for sc in btm.values() {
                if first {
                    first = false;
                } else {
                    self.writer.write_all(b"\n")?;
                }
                self.write_arg(sc)?;
            }
        }
        Ok(())
    }

    /// Writes version of a Parser Object to the wrapped stream.
    fn write_version(&mut self, parser: &Parser) -> io::Result<()> {
        debugln!("Help::write_version;");
        write!(self.writer, "{}", parser.meta.version.unwrap_or(""))?;
        Ok(())
    }

    /// Writes binary name of a Parser Object to the wrapped stream.
    fn write_bin_name(&mut self, parser: &Parser) -> io::Result<()> {
        debugln!("Help::write_bin_name;");
        macro_rules! write_name {
            () => {{
                let mut name = parser.meta.name.clone();
                name = name.replace("{n}", "\n");
                color!(self, wrap_help(&name, self.term_w), good)?;
            }};
        }
        if let Some(bn) = parser.meta.bin_name.as_ref() {
            if bn.contains(' ') {
                // Incase we're dealing with subcommands i.e. git mv is translated to git-mv
                color!(self, bn.replace(" ", "-"), good)?
            } else {
                write_name!();
            }
        } else {
            write_name!();
        }
        Ok(())
    }

    /// Writes default help for a Parser Object to the wrapped stream.
    pub fn write_default_help(&mut self, parser: &Parser) -> ClapResult<()> {
        debugln!("Help::write_default_help;");
        if let Some(h) = parser.meta.pre_help {
            self.write_before_after_help(h)?;
            self.writer.write_all(b"\n\n")?;
        }

        macro_rules! write_thing {
            ($thing:expr) => {{
                let mut owned_thing = $thing.to_owned();
                owned_thing = owned_thing.replace("{n}", "\n");
                write!(self.writer, "{}\n", wrap_help(&owned_thing, self.term_w))?
            }};
        }
        // Print the version
        self.write_bin_name(parser)?;
        self.writer.write_all(b" ")?;
        self.write_version(parser)?;
        self.writer.write_all(b"\n")?;
        if let Some(author) = parser.meta.author {
            write_thing!(author)
        }
        // if self.use_long {
        //     if let Some(about) = parser.meta.long_about {
        //         debugln!("Help::write_default_help: writing long about");
        //         write_thing!(about)
        //     } else if let Some(about) = parser.meta.about {
        //         debugln!("Help::write_default_help: writing about");
        //         write_thing!(about)
        //     }
        // } else
        if let Some(about) = parser.meta.long_about {
            debugln!("Help::write_default_help: writing long about");
            write_thing!(about)
        } else if let Some(about) = parser.meta.about {
            debugln!("Help::write_default_help: writing about");
            write_thing!(about)
        }

        color!(self, "\nUSAGE:", warning)?;
        write!(
            self.writer,
            "\n{}{}\n\n",
            TAB,
            usage::create_usage_no_title(parser, &[])
        )?;

        let flags = parser.has_flags();
        let pos = parser.has_positionals();
        let opts = parser.has_opts();
        let subcmds = parser.has_subcommands();

        if flags || opts || pos || subcmds {
            self.write_all_args(parser)?;
        }

        if let Some(h) = parser.meta.more_help {
            if flags || opts || pos || subcmds {
                self.writer.write_all(b"\n\n")?;
            }
            self.write_before_after_help(h)?;
        }

        self.writer.flush().map_err(Error::from)
    }
}

/// Possible results for a copying function that stops when a given
/// byte was found.
enum CopyUntilResult {
    DelimiterFound(usize),
    DelimiterNotFound(usize),
    ReaderEmpty,
    ReadError(io::Error),
    WriteError(io::Error),
}

/// Copies the contents of a reader into a writer until a delimiter byte is found.
/// On success, the total number of bytes that were
/// copied from reader to writer is returned.
fn copy_until<R: Read, W: Write>(r: &mut R, w: &mut W, delimiter_byte: u8) -> CopyUntilResult {
    debugln!("copy_until;");

    let mut count = 0;
    for wb in r.bytes() {
        match wb {
            Ok(b) => {
                if b == delimiter_byte {
                    return CopyUntilResult::DelimiterFound(count);
                }
                match w.write(&[b]) {
                    Ok(c) => count += c,
                    Err(e) => return CopyUntilResult::WriteError(e),
                }
            }
            Err(e) => return CopyUntilResult::ReadError(e),
        }
    }
    if count > 0 {
        CopyUntilResult::DelimiterNotFound(count)
    } else {
        CopyUntilResult::ReaderEmpty
    }
}

/// Copies the contents of a reader into a writer until a {tag} is found,
/// copying the tag content to a buffer and returning its size.
/// In addition to errors, there are three possible outputs:
///   - `None`: The reader was consumed.
///   - `Some(Ok(0))`: No tag was captured but the reader still contains data.
///   - `Some(Ok(length>0))`: a tag with `length` was captured to the `tag_buffer`.
fn copy_and_capture<R: Read, W: Write>(
    r: &mut R,
    w: &mut W,
    tag_buffer: &mut Cursor<Vec<u8>>,
) -> Option<io::Result<usize>> {
    use self::CopyUntilResult::*;
    debugln!("copy_and_capture;");

    // Find the opening byte.
    match copy_until(r, w, b'{') {
        // The end of the reader was reached without finding the opening tag.
        // (either with or without having copied data to the writer)
        // Return None indicating that we are done.
        ReaderEmpty | DelimiterNotFound(_) => None,

        // Something went wrong.
        ReadError(e) | WriteError(e) => Some(Err(e)),

        // The opening byte was found.
        // (either with or without having copied data to the writer)
        DelimiterFound(_) => {
            // Lets reset the buffer first and find out how long it is.
            tag_buffer.set_position(0);
            let buffer_size = tag_buffer.get_ref().len();

            // Find the closing byte,limiting the reader to the length of the buffer.
            let mut rb = r.take(buffer_size as u64);
            match copy_until(&mut rb, tag_buffer, b'}') {
                // We were already at the end of the reader.
                // Return None indicating that we are done.
                ReaderEmpty => None,

                // The closing tag was found.
                // Return the tag_length.
                DelimiterFound(tag_length) => Some(Ok(tag_length)),

                // The end of the reader was found without finding the closing tag.
                // Write the opening byte and captured text to the writer.
                // Return 0 indicating that nothing was captured but the reader still contains data.
                DelimiterNotFound(not_tag_length) => match w.write(b"{") {
                    Err(e) => Some(Err(e)),
                    _ => match w.write(&tag_buffer.get_ref()[0..not_tag_length]) {
                        Err(e) => Some(Err(e)),
                        _ => Some(Ok(0)),
                    },
                },

                ReadError(e) | WriteError(e) => Some(Err(e)),
            }
        }
    }
}

// Methods to write Parser help using templates.
impl<'a> Help<'a> {
    /// Write help to stream for the parser in the format defined by the template.
    ///
    /// Tags arg given inside curly brackets:
    /// Valid tags are:
    ///     * `{bin}`         - Binary name.
    ///     * `{version}`     - Version number.
    ///     * `{author}`      - Author information.
    ///     * `{usage}`       - Automatically generated or given usage string.
    ///     * `{all-args}`    - Help for all arguments (options, flags, positionals arguments,
    ///                         and subcommands) including titles.
    ///     * `{unified}`     - Unified help for options and flags.
    ///     * `{flags}`       - Help for flags.
    ///     * `{options}`     - Help for options.
    ///     * `{positionals}` - Help for positionals arguments.
    ///     * `{subcommands}` - Help for subcommands.
    ///     * `{after-help}`  - Info to be displayed after the help message.
    ///     * `{before-help}` - Info to be displayed before the help message.
    ///
    /// The template system is, on purpose, very simple. Therefore the tags have to written
    /// in the lowercase and without spacing.
    fn write_templated_help(&mut self, parser: &Parser, template: &str) -> ClapResult<()> {
        debugln!("Help::write_templated_help;");
        let mut tmplr = Cursor::new(&template);
        let mut tag_buf = Cursor::new(vec![0u8; 15]);

        // The strategy is to copy the template from the reader to wrapped stream
        // until a tag is found. Depending on its value, the appropriate content is copied
        // to the wrapped stream.
        // The copy from template is then resumed, repeating this sequence until reading
        // the complete template.

        loop {
            let tag_length = match copy_and_capture(&mut tmplr, &mut self.writer, &mut tag_buf) {
                None => return Ok(()),
                Some(Err(e)) => return Err(Error::from(e)),
                Some(Ok(val)) if val > 0 => val,
                _ => continue,
            };

            debugln!("Help::write_template_help:iter: tag_buf={};", unsafe {
                String::from_utf8_unchecked(
                    tag_buf.get_ref()[0..tag_length]
                        .iter()
                        .map(|&i| i)
                        .collect::<Vec<_>>(),
                )
            });
            match &tag_buf.get_ref()[0..tag_length] {
                b"?" => {
                    self.writer.write_all(b"Could not decode tag name")?;
                }
                b"bin" => {
                    self.write_bin_name(parser)?;
                }
                b"version" => {
                    write!(
                        self.writer,
                        "{}",
                        parser.meta.version.unwrap_or("unknown version")
                    )?;
                }
                b"author" => {
                    write!(
                        self.writer,
                        "{}",
                        parser.meta.author.unwrap_or("unknown author")
                    )?;
                }
                b"about" => {
                    write!(
                        self.writer,
                        "{}",
                        parser.meta.about.unwrap_or("unknown about")
                    )?;
                }
                b"long-about" => {
                    write!(
                        self.writer,
                        "{}",
                        parser.meta.long_about.unwrap_or("unknown about")
                    )?;
                }
                b"usage" => {
                    write!(self.writer, "{}", usage::create_usage_no_title(parser, &[]))?;
                }
                b"all-args" => {
                    self.write_all_args(parser)?;
                }
                b"unified" => {
                    let opts_flags = parser
                        .flags()
                        .map(as_arg_trait)
                        .chain(parser.opts().map(as_arg_trait));
                    self.write_args(opts_flags)?;
                }
                b"flags" => {
                    self.write_args(parser.flags().map(as_arg_trait))?;
                }
                b"options" => {
                    self.write_args(parser.opts().map(as_arg_trait))?;
                }
                b"positionals" => {
                    self.write_args(parser.positionals().map(as_arg_trait))?;
                }
                b"subcommands" => {
                    self.write_subcommands(parser)?;
                }
                b"after-help" => {
                    write!(
                        self.writer,
                        "{}",
                        parser.meta.more_help.unwrap_or("unknown after-help")
                    )?;
                }
                b"before-help" => {
                    write!(
                        self.writer,
                        "{}",
                        parser.meta.pre_help.unwrap_or("unknown before-help")
                    )?;
                }
                // Unknown tag, write it back.
                r => {
                    self.writer.write_all(b"{")?;
                    self.writer.write_all(r)?;
                    self.writer.write_all(b"}")?;
                }
            }
        }
    }
}

fn wrap_help(help: &str, avail_chars: usize) -> String {
    let wrapper = textwrap::Wrapper::new(avail_chars).break_words(false);
    help.lines()
        .map(|line| wrapper.fill(line))
        .collect::<Vec<String>>()
        .join("\n")
}

#[cfg(test)]
mod test {
    use super::wrap_help;

    #[test]
    fn wrap_help_last_word() {
        let help = String::from("foo bar baz");
        assert_eq!(wrap_help(&help, 5), "foo\nbar\nbaz");
    }
}
