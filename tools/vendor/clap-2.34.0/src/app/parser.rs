// Std
#[cfg(not(any(target_os = "windows", target_arch = "wasm32")))]
use std::os::unix::ffi::OsStrExt;
use std::{
    cell::Cell,
    ffi::{OsStr, OsString},
    fmt::Display,
    fs::File,
    io::{self, BufWriter, Write},
    iter::Peekable,
    path::PathBuf,
    slice::Iter,
};

// Internal
#[cfg(all(feature = "debug", any(target_os = "windows", target_arch = "wasm32")))]
use crate::osstringext::OsStrExt3;
#[cfg(any(target_os = "windows", target_arch = "wasm32"))]
use crate::osstringext::OsStrExt3;
use crate::{
    app::{
        help::Help, meta::AppMeta, settings::AppFlags, settings::AppSettings as AS, usage,
        validator::Validator, App,
    },
    args::{
        settings::ArgSettings, AnyArg, Arg, ArgGroup, ArgMatcher, Base, FlagBuilder, OptBuilder,
        PosBuilder, Switched,
    },
    completions::{ComplGen, Shell},
    errors::Result as ClapResult,
    errors::{Error, ErrorKind},
    fmt::ColorWhen,
    map::{self, VecMap},
    osstringext::OsStrExt2,
    suggestions, SubCommand, INTERNAL_ERROR_MSG, INVALID_UTF8,
};

#[derive(Debug, PartialEq, Copy, Clone)]
#[doc(hidden)]
pub enum ParseResult<'a> {
    Flag,
    Opt(&'a str),
    Pos(&'a str),
    MaybeHyphenValue,
    MaybeNegNum,
    NotFound,
    ValuesDone,
}

#[allow(missing_debug_implementations)]
#[doc(hidden)]
#[derive(Clone, Default)]
pub struct Parser<'a, 'b>
where
    'a: 'b,
{
    pub meta: AppMeta<'b>,
    settings: AppFlags,
    pub g_settings: AppFlags,
    pub flags: Vec<FlagBuilder<'a, 'b>>,
    pub opts: Vec<OptBuilder<'a, 'b>>,
    pub positionals: VecMap<PosBuilder<'a, 'b>>,
    pub subcommands: Vec<App<'a, 'b>>,
    pub groups: Vec<ArgGroup<'a>>,
    pub global_args: Vec<Arg<'a, 'b>>,
    pub required: Vec<&'a str>,
    pub r_ifs: Vec<(&'a str, &'b str, &'a str)>,
    pub overrides: Vec<(&'b str, &'a str)>,
    help_short: Option<char>,
    version_short: Option<char>,
    cache: Option<&'a str>,
    pub help_message: Option<&'a str>,
    pub version_message: Option<&'a str>,
    cur_idx: Cell<usize>,
}

impl<'a, 'b> Parser<'a, 'b>
where
    'a: 'b,
{
    pub fn with_name(n: String) -> Self {
        Parser {
            meta: AppMeta::with_name(n),
            g_settings: AppFlags::zeroed(),
            cur_idx: Cell::new(0),
            ..Default::default()
        }
    }

    pub fn help_short(&mut self, s: &str) {
        let c = s
            .trim_left_matches(|c| c == '-')
            .chars()
            .next()
            .unwrap_or('h');
        self.help_short = Some(c);
    }

    pub fn version_short(&mut self, s: &str) {
        let c = s
            .trim_left_matches(|c| c == '-')
            .chars()
            .next()
            .unwrap_or('V');
        self.version_short = Some(c);
    }

    pub fn gen_completions_to<W: Write>(&mut self, for_shell: Shell, buf: &mut W) {
        if !self.is_set(AS::Propagated) {
            self.propagate_help_version();
            self.build_bin_names();
            self.propagate_globals();
            self.propagate_settings();
            self.set(AS::Propagated);
        }

        ComplGen::new(self).generate(for_shell, buf)
    }

    pub fn gen_completions(&mut self, for_shell: Shell, od: OsString) {
        use std::error::Error;

        let out_dir = PathBuf::from(od);
        let name = &*self.meta.bin_name.as_ref().unwrap().clone();
        let file_name = match for_shell {
            Shell::Bash => format!("{}.bash", name),
            Shell::Fish => format!("{}.fish", name),
            Shell::Zsh => format!("_{}", name),
            Shell::PowerShell => format!("_{}.ps1", name),
            Shell::Elvish => format!("{}.elv", name),
        };

        let mut file = match File::create(out_dir.join(file_name)) {
            Err(why) => panic!("couldn't create completion file: {}", why.description()),
            Ok(file) => file,
        };
        self.gen_completions_to(for_shell, &mut file)
    }

    #[inline]
    fn app_debug_asserts(&self) -> bool {
        assert!(self.verify_positionals());
        let should_err = self.groups.iter().all(|g| {
            g.args.iter().all(|arg| {
                self.flags.iter().any(|f| &f.b.name == arg)
                    || self.opts.iter().any(|o| &o.b.name == arg)
                    || self.positionals.values().any(|p| &p.b.name == arg)
                    || self.groups.iter().any(|g| &g.name == arg)
            })
        });
        let g = self.groups.iter().find(|g| {
            g.args.iter().any(|arg| {
                !(self.flags.iter().any(|f| &f.b.name == arg)
                    || self.opts.iter().any(|o| &o.b.name == arg)
                    || self.positionals.values().any(|p| &p.b.name == arg)
                    || self.groups.iter().any(|g| &g.name == arg))
            })
        });
        assert!(
            should_err,
            "The group '{}' contains the arg '{}' that doesn't actually exist.",
            g.unwrap().name,
            g.unwrap()
                .args
                .iter()
                .find(|arg| !(self.flags.iter().any(|f| &&f.b.name == arg)
                    || self.opts.iter().any(|o| &&o.b.name == arg)
                    || self.positionals.values().any(|p| &&p.b.name == arg)
                    || self.groups.iter().any(|g| &&g.name == arg)))
                .unwrap()
        );
        true
    }

    #[inline]
    fn debug_asserts(&self, a: &Arg) -> bool {
        assert!(
            !arg_names!(self).any(|name| name == a.b.name),
            "Non-unique argument name: {} is already in use",
            a.b.name
        );
        if let Some(l) = a.s.long {
            assert!(
                !self.contains_long(l),
                "Argument long must be unique\n\n\t--{} is already in use",
                l
            );
        }
        if let Some(s) = a.s.short {
            assert!(
                !self.contains_short(s),
                "Argument short must be unique\n\n\t-{} is already in use",
                s
            );
        }
        let i = if a.index.is_none() {
            self.positionals.len() + 1
        } else {
            a.index.unwrap() as usize
        };
        assert!(
            !self.positionals.contains_key(i),
            "Argument \"{}\" has the same index as another positional \
             argument\n\n\tPerhaps try .multiple(true) to allow one positional argument \
             to take multiple values",
            a.b.name
        );
        assert!(
            !(a.is_set(ArgSettings::Required) && a.is_set(ArgSettings::Global)),
            "Global arguments cannot be required.\n\n\t'{}' is marked as \
             global and required",
            a.b.name
        );
        if a.b.is_set(ArgSettings::Last) {
            assert!(
                !self
                    .positionals
                    .values()
                    .any(|p| p.b.is_set(ArgSettings::Last)),
                "Only one positional argument may have last(true) set. Found two."
            );
            assert!(a.s.long.is_none(),
                    "Flags or Options may not have last(true) set. {} has both a long and last(true) set.",
                    a.b.name);
            assert!(a.s.short.is_none(),
                    "Flags or Options may not have last(true) set. {} has both a short and last(true) set.",
                    a.b.name);
        }
        true
    }

    #[inline]
    fn add_conditional_reqs(&mut self, a: &Arg<'a, 'b>) {
        if let Some(ref r_ifs) = a.r_ifs {
            for &(arg, val) in r_ifs {
                self.r_ifs.push((arg, val, a.b.name));
            }
        }
    }

    #[inline]
    fn add_arg_groups(&mut self, a: &Arg<'a, 'b>) {
        if let Some(ref grps) = a.b.groups {
            for g in grps {
                let mut found = false;
                if let Some(ref mut ag) = self.groups.iter_mut().find(|grp| &grp.name == g) {
                    ag.args.push(a.b.name);
                    found = true;
                }
                if !found {
                    let mut ag = ArgGroup::with_name(g);
                    ag.args.push(a.b.name);
                    self.groups.push(ag);
                }
            }
        }
    }

    #[inline]
    fn add_reqs(&mut self, a: &Arg<'a, 'b>) {
        if a.is_set(ArgSettings::Required) {
            // If the arg is required, add all it's requirements to master required list
            self.required.push(a.b.name);
            if let Some(ref areqs) = a.b.requires {
                for name in areqs
                    .iter()
                    .filter(|&&(val, _)| val.is_none())
                    .map(|&(_, name)| name)
                {
                    self.required.push(name);
                }
            }
        }
    }

    #[inline]
    fn implied_settings(&mut self, a: &Arg<'a, 'b>) {
        if a.is_set(ArgSettings::Last) {
            // if an arg has `Last` set, we need to imply DontCollapseArgsInUsage so that args
            // in the usage string don't get confused or left out.
            self.set(AS::DontCollapseArgsInUsage);
            self.set(AS::ContainsLast);
        }
        if let Some(l) = a.s.long {
            if l == "version" {
                self.unset(AS::NeedsLongVersion);
            } else if l == "help" {
                self.unset(AS::NeedsLongHelp);
            }
        }
    }

    // actually adds the arguments
    pub fn add_arg(&mut self, a: Arg<'a, 'b>) {
        // if it's global we have to clone anyways
        if a.is_set(ArgSettings::Global) {
            return self.add_arg_ref(&a);
        }
        debug_assert!(self.debug_asserts(&a));
        self.add_conditional_reqs(&a);
        self.add_arg_groups(&a);
        self.add_reqs(&a);
        self.implied_settings(&a);
        if a.index.is_some() || (a.s.short.is_none() && a.s.long.is_none()) {
            let i = if a.index.is_none() {
                self.positionals.len() + 1
            } else {
                a.index.unwrap() as usize
            };
            self.positionals
                .insert(i, PosBuilder::from_arg(a, i as u64));
        } else if a.is_set(ArgSettings::TakesValue) {
            let mut ob = OptBuilder::from(a);
            ob.s.unified_ord = self.flags.len() + self.opts.len();
            self.opts.push(ob);
        } else {
            let mut fb = FlagBuilder::from(a);
            fb.s.unified_ord = self.flags.len() + self.opts.len();
            self.flags.push(fb);
        }
    }
    // actually adds the arguments but from a borrow (which means we have to do some cloning)
    pub fn add_arg_ref(&mut self, a: &Arg<'a, 'b>) {
        debug_assert!(self.debug_asserts(a));
        self.add_conditional_reqs(a);
        self.add_arg_groups(a);
        self.add_reqs(a);
        self.implied_settings(a);
        if a.index.is_some() || (a.s.short.is_none() && a.s.long.is_none()) {
            let i = if a.index.is_none() {
                self.positionals.len() + 1
            } else {
                a.index.unwrap() as usize
            };
            let pb = PosBuilder::from_arg_ref(a, i as u64);
            self.positionals.insert(i, pb);
        } else if a.is_set(ArgSettings::TakesValue) {
            let mut ob = OptBuilder::from(a);
            ob.s.unified_ord = self.flags.len() + self.opts.len();
            self.opts.push(ob);
        } else {
            let mut fb = FlagBuilder::from(a);
            fb.s.unified_ord = self.flags.len() + self.opts.len();
            self.flags.push(fb);
        }
        if a.is_set(ArgSettings::Global) {
            self.global_args.push(a.into());
        }
    }

    pub fn add_group(&mut self, group: ArgGroup<'a>) {
        if group.required {
            self.required.push(group.name);
            if let Some(ref reqs) = group.requires {
                self.required.extend_from_slice(reqs);
            }
            //            if let Some(ref bl) = group.conflicts {
            //                self.blacklist.extend_from_slice(bl);
            //            }
        }
        if self.groups.iter().any(|g| g.name == group.name) {
            let grp = self
                .groups
                .iter_mut()
                .find(|g| g.name == group.name)
                .expect(INTERNAL_ERROR_MSG);
            grp.args.extend_from_slice(&group.args);
            grp.requires = group.requires.clone();
            grp.conflicts = group.conflicts.clone();
            grp.required = group.required;
        } else {
            self.groups.push(group);
        }
    }

    pub fn add_subcommand(&mut self, mut subcmd: App<'a, 'b>) {
        debugln!(
            "Parser::add_subcommand: term_w={:?}, name={}",
            self.meta.term_w,
            subcmd.p.meta.name
        );
        subcmd.p.meta.term_w = self.meta.term_w;
        if subcmd.p.meta.name == "help" {
            self.unset(AS::NeedsSubcommandHelp);
        }

        self.subcommands.push(subcmd);
    }

    pub fn propagate_settings(&mut self) {
        debugln!(
            "Parser::propagate_settings: self={}, g_settings={:#?}",
            self.meta.name,
            self.g_settings
        );
        for sc in &mut self.subcommands {
            debugln!(
                "Parser::propagate_settings: sc={}, settings={:#?}, g_settings={:#?}",
                sc.p.meta.name,
                sc.p.settings,
                sc.p.g_settings
            );
            // We have to create a new scope in order to tell rustc the borrow of `sc` is
            // done and to recursively call this method
            {
                let vsc = self.settings.is_set(AS::VersionlessSubcommands);
                let gv = self.settings.is_set(AS::GlobalVersion);

                if vsc {
                    sc.p.set(AS::DisableVersion);
                }
                if gv && sc.p.meta.version.is_none() && self.meta.version.is_some() {
                    sc.p.set(AS::GlobalVersion);
                    sc.p.meta.version = Some(self.meta.version.unwrap());
                }
                sc.p.settings = sc.p.settings | self.g_settings;
                sc.p.g_settings = sc.p.g_settings | self.g_settings;
                sc.p.meta.term_w = self.meta.term_w;
                sc.p.meta.max_w = self.meta.max_w;
            }
            sc.p.propagate_settings();
        }
    }

    pub fn derive_display_order(&mut self) {
        if self.is_set(AS::DeriveDisplayOrder) {
            let unified = self.is_set(AS::UnifiedHelpMessage);
            for (i, o) in self
                .opts
                .iter_mut()
                .enumerate()
                .filter(|&(_, ref o)| o.s.disp_ord == 999)
            {
                o.s.disp_ord = if unified { o.s.unified_ord } else { i };
            }
            for (i, f) in self
                .flags
                .iter_mut()
                .enumerate()
                .filter(|&(_, ref f)| f.s.disp_ord == 999)
            {
                f.s.disp_ord = if unified { f.s.unified_ord } else { i };
            }
            for (i, sc) in &mut self
                .subcommands
                .iter_mut()
                .enumerate()
                .filter(|&(_, ref sc)| sc.p.meta.disp_ord == 999)
            {
                sc.p.meta.disp_ord = i;
            }
        }
        for sc in &mut self.subcommands {
            sc.p.derive_display_order();
        }
    }

    pub fn required(&self) -> Iter<&str> {
        self.required.iter()
    }

    #[inline]
    pub fn has_args(&self) -> bool {
        !(self.flags.is_empty() && self.opts.is_empty() && self.positionals.is_empty())
    }

    #[inline]
    pub fn has_opts(&self) -> bool {
        !self.opts.is_empty()
    }

    #[inline]
    pub fn has_flags(&self) -> bool {
        !self.flags.is_empty()
    }

    #[inline]
    pub fn has_positionals(&self) -> bool {
        !self.positionals.is_empty()
    }

    #[inline]
    pub fn has_subcommands(&self) -> bool {
        !self.subcommands.is_empty()
    }

    #[inline]
    pub fn has_visible_opts(&self) -> bool {
        if self.opts.is_empty() {
            return false;
        }
        self.opts.iter().any(|o| !o.is_set(ArgSettings::Hidden))
    }

    #[inline]
    pub fn has_visible_flags(&self) -> bool {
        if self.flags.is_empty() {
            return false;
        }
        self.flags.iter().any(|f| !f.is_set(ArgSettings::Hidden))
    }

    #[inline]
    pub fn has_visible_positionals(&self) -> bool {
        if self.positionals.is_empty() {
            return false;
        }
        self.positionals
            .values()
            .any(|p| !p.is_set(ArgSettings::Hidden))
    }

    #[inline]
    pub fn has_visible_subcommands(&self) -> bool {
        self.has_subcommands()
            && self
                .subcommands
                .iter()
                .filter(|sc| sc.p.meta.name != "help")
                .any(|sc| !sc.p.is_set(AS::Hidden))
    }

    #[inline]
    pub fn is_set(&self, s: AS) -> bool {
        self.settings.is_set(s)
    }

    #[inline]
    pub fn set(&mut self, s: AS) {
        self.settings.set(s)
    }

    #[inline]
    pub fn unset(&mut self, s: AS) {
        self.settings.unset(s)
    }

    pub fn verify_positionals(&self) -> bool {
        // Because you must wait until all arguments have been supplied, this is the first chance
        // to make assertions on positional argument indexes
        //
        // First we verify that the index highest supplied index, is equal to the number of
        // positional arguments to verify there are no gaps (i.e. supplying an index of 1 and 3
        // but no 2)
        if let Some((idx, p)) = self.positionals.iter().rev().next() {
            assert!(
                !(idx != self.positionals.len()),
                "Found positional argument \"{}\" whose index is {} but there \
                 are only {} positional arguments defined",
                p.b.name,
                idx,
                self.positionals.len()
            );
        }

        // Next we verify that only the highest index has a .multiple(true) (if any)
        if self.positionals.values().any(|a| {
            a.b.is_set(ArgSettings::Multiple) && (a.index as usize != self.positionals.len())
        }) {
            let mut it = self.positionals.values().rev();
            let last = it.next().unwrap();
            let second_to_last = it.next().unwrap();
            // Either the final positional is required
            // Or the second to last has a terminator or .last(true) set
            let ok = last.is_set(ArgSettings::Required)
                || (second_to_last.v.terminator.is_some()
                    || second_to_last.b.is_set(ArgSettings::Last))
                || last.is_set(ArgSettings::Last);
            assert!(
                ok,
                "When using a positional argument with .multiple(true) that is *not the \
                 last* positional argument, the last positional argument (i.e the one \
                 with the highest index) *must* have .required(true) or .last(true) set."
            );
            let ok = second_to_last.is_set(ArgSettings::Multiple) || last.is_set(ArgSettings::Last);
            assert!(
                ok,
                "Only the last positional argument, or second to last positional \
                 argument may be set to .multiple(true)"
            );

            let count = self
                .positionals
                .values()
                .filter(|p| p.b.settings.is_set(ArgSettings::Multiple) && p.v.num_vals.is_none())
                .count();
            let ok = count <= 1
                || (last.is_set(ArgSettings::Last)
                    && last.is_set(ArgSettings::Multiple)
                    && second_to_last.is_set(ArgSettings::Multiple)
                    && count == 2);
            assert!(
                ok,
                "Only one positional argument with .multiple(true) set is allowed per \
                 command, unless the second one also has .last(true) set"
            );
        }

        let mut found = false;
        if self.is_set(AS::AllowMissingPositional) {
            // Check that if a required positional argument is found, all positions with a lower
            // index are also required.
            let mut foundx2 = false;
            for p in self.positionals.values().rev() {
                if foundx2 && !p.b.settings.is_set(ArgSettings::Required) {
                    assert!(
                        p.b.is_set(ArgSettings::Required),
                        "Found positional argument which is not required with a lower \
                         index than a required positional argument by two or more: {:?} \
                         index {}",
                        p.b.name,
                        p.index
                    );
                } else if p.b.is_set(ArgSettings::Required) && !p.b.is_set(ArgSettings::Last) {
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
            for p in self.positionals.values().rev() {
                if found {
                    assert!(
                        p.b.is_set(ArgSettings::Required),
                        "Found positional argument which is not required with a lower \
                         index than a required positional argument: {:?} index {}",
                        p.b.name,
                        p.index
                    );
                } else if p.b.is_set(ArgSettings::Required) && !p.b.is_set(ArgSettings::Last) {
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
        if self
            .positionals
            .values()
            .any(|p| p.b.is_set(ArgSettings::Last) && p.b.is_set(ArgSettings::Required))
            && self.has_subcommands()
            && !self.is_set(AS::SubcommandsNegateReqs)
        {
            panic!(
                "Having a required positional argument with .last(true) set *and* child \
                 subcommands without setting SubcommandsNegateReqs isn't compatible."
            );
        }

        true
    }

    pub fn propagate_globals(&mut self) {
        for sc in &mut self.subcommands {
            // We have to create a new scope in order to tell rustc the borrow of `sc` is
            // done and to recursively call this method
            {
                for a in &self.global_args {
                    sc.p.add_arg_ref(a);
                }
            }
            sc.p.propagate_globals();
        }
    }

    // Checks if the arg matches a subcommand name, or any of it's aliases (if defined)
    fn possible_subcommand(&self, arg_os: &OsStr) -> (bool, Option<&str>) {
        debugln!("Parser::possible_subcommand: arg={:?}", arg_os);
        fn starts(h: &str, n: &OsStr) -> bool {
            let n_bytes = n.as_bytes();
            let h_bytes = OsStr::new(h).as_bytes();

            h_bytes.starts_with(n_bytes)
        }

        if self.is_set(AS::ArgsNegateSubcommands) && self.is_set(AS::ValidArgFound) {
            return (false, None);
        }
        if !self.is_set(AS::InferSubcommands) {
            if let Some(sc) = find_subcmd!(self, arg_os) {
                return (true, Some(&sc.p.meta.name));
            }
        } else {
            let v = self
                .subcommands
                .iter()
                .filter(|s| {
                    starts(&s.p.meta.name[..], &*arg_os)
                        || (s.p.meta.aliases.is_some()
                            && s.p
                                .meta
                                .aliases
                                .as_ref()
                                .unwrap()
                                .iter()
                                .filter(|&&(a, _)| starts(a, &*arg_os))
                                .count()
                                == 1)
                })
                .map(|sc| &sc.p.meta.name)
                .collect::<Vec<_>>();

            for sc in &v {
                if OsStr::new(sc) == arg_os {
                    return (true, Some(sc));
                }
            }

            if v.len() == 1 {
                return (true, Some(v[0]));
            }
        }
        (false, None)
    }

    fn parse_help_subcommand<I, T>(&self, it: &mut I) -> ClapResult<ParseResult<'a>>
    where
        I: Iterator<Item = T>,
        T: Into<OsString>,
    {
        debugln!("Parser::parse_help_subcommand;");
        let cmds: Vec<OsString> = it.map(|c| c.into()).collect();
        let mut help_help = false;
        let mut bin_name = self
            .meta
            .bin_name
            .as_ref()
            .unwrap_or(&self.meta.name)
            .clone();
        let mut sc = {
            let mut sc: &Parser = self;
            for (i, cmd) in cmds.iter().enumerate() {
                if &*cmd.to_string_lossy() == "help" {
                    // cmd help help
                    help_help = true;
                }
                if let Some(c) = sc
                    .subcommands
                    .iter()
                    .find(|s| &*s.p.meta.name == cmd)
                    .map(|sc| &sc.p)
                {
                    sc = c;
                    if i == cmds.len() - 1 {
                        break;
                    }
                } else if let Some(c) = sc
                    .subcommands
                    .iter()
                    .find(|s| {
                        if let Some(ref als) = s.p.meta.aliases {
                            als.iter().any(|&(a, _)| a == &*cmd.to_string_lossy())
                        } else {
                            false
                        }
                    })
                    .map(|sc| &sc.p)
                {
                    sc = c;
                    if i == cmds.len() - 1 {
                        break;
                    }
                } else {
                    return Err(Error::unrecognized_subcommand(
                        cmd.to_string_lossy().into_owned(),
                        self.meta.bin_name.as_ref().unwrap_or(&self.meta.name),
                        self.color(),
                    ));
                }
                bin_name = format!("{} {}", bin_name, &*sc.meta.name);
            }
            sc.clone()
        };
        if help_help {
            let mut pb = PosBuilder::new("subcommand", 1);
            pb.b.help = Some("The subcommand whose help message to display");
            pb.set(ArgSettings::Multiple);
            sc.positionals.insert(1, pb);
            sc.settings = sc.settings | self.g_settings;
        } else {
            sc.create_help_and_version();
        }
        if sc.meta.bin_name != self.meta.bin_name {
            sc.meta.bin_name = Some(format!("{} {}", bin_name, sc.meta.name));
        }
        Err(sc._help(false))
    }

    // allow wrong self convention due to self.valid_neg_num = true and it's a private method
    #[cfg_attr(feature = "lints", allow(wrong_self_convention))]
    fn is_new_arg(&mut self, arg_os: &OsStr, needs_val_of: ParseResult) -> bool {
        debugln!("Parser::is_new_arg:{:?}:{:?}", arg_os, needs_val_of);
        let app_wide_settings = if self.is_set(AS::AllowLeadingHyphen) {
            true
        } else if self.is_set(AS::AllowNegativeNumbers) {
            let a = arg_os.to_string_lossy();
            if a.parse::<i64>().is_ok() || a.parse::<f64>().is_ok() {
                self.set(AS::ValidNegNumFound);
                true
            } else {
                false
            }
        } else {
            false
        };
        let arg_allows_tac = match needs_val_of {
            ParseResult::Opt(name) => {
                let o = self
                    .opts
                    .iter()
                    .find(|o| o.b.name == name)
                    .expect(INTERNAL_ERROR_MSG);
                o.is_set(ArgSettings::AllowLeadingHyphen) || app_wide_settings
            }
            ParseResult::Pos(name) => {
                let p = self
                    .positionals
                    .values()
                    .find(|p| p.b.name == name)
                    .expect(INTERNAL_ERROR_MSG);
                p.is_set(ArgSettings::AllowLeadingHyphen) || app_wide_settings
            }
            ParseResult::ValuesDone => return true,
            _ => false,
        };
        debugln!("Parser::is_new_arg: arg_allows_tac={:?}", arg_allows_tac);

        // Is this a new argument, or values from a previous option?
        let mut ret = if arg_os.starts_with(b"--") {
            debugln!("Parser::is_new_arg: -- found");
            if arg_os.len() == 2 && !arg_allows_tac {
                return true; // We have to return true so override everything else
            } else if arg_allows_tac {
                return false;
            }
            true
        } else if arg_os.starts_with(b"-") {
            debugln!("Parser::is_new_arg: - found");
            // a singe '-' by itself is a value and typically means "stdin" on unix systems
            arg_os.len() != 1
        } else {
            debugln!("Parser::is_new_arg: probably value");
            false
        };

        ret = ret && !arg_allows_tac;

        debugln!("Parser::is_new_arg: starts_new_arg={:?}", ret);
        ret
    }

    // The actual parsing function
    #[cfg_attr(
        feature = "cargo-clippy",
        allow(clippy::while_let_on_iterator, clippy::nonminimal_bool)
    )]
    pub fn get_matches_with<I, T>(
        &mut self,
        matcher: &mut ArgMatcher<'a>,
        it: &mut Peekable<I>,
    ) -> ClapResult<()>
    where
        I: Iterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        debugln!("Parser::get_matches_with;");
        // Verify all positional assertions pass
        debug_assert!(self.app_debug_asserts());
        if self.positionals.values().any(|a| {
            a.b.is_set(ArgSettings::Multiple) && (a.index as usize != self.positionals.len())
        }) && self
            .positionals
            .values()
            .last()
            .map_or(false, |p| !p.is_set(ArgSettings::Last))
        {
            self.settings.set(AS::LowIndexMultiplePositional);
        }
        let has_args = self.has_args();

        // Next we create the `--help` and `--version` arguments and add them if
        // necessary
        self.create_help_and_version();

        let mut subcmd_name: Option<String> = None;
        let mut needs_val_of: ParseResult<'a> = ParseResult::NotFound;
        let mut pos_counter = 1;
        let mut sc_is_external = false;
        while let Some(arg) = it.next() {
            let arg_os = arg.into();
            debugln!(
                "Parser::get_matches_with: Begin parsing '{:?}' ({:?})",
                arg_os,
                &*arg_os.as_bytes()
            );

            self.unset(AS::ValidNegNumFound);
            // Is this a new argument, or values from a previous option?
            let starts_new_arg = self.is_new_arg(&arg_os, needs_val_of);
            if !self.is_set(AS::TrailingValues)
                && arg_os.starts_with(b"--")
                && arg_os.len() == 2
                && starts_new_arg
            {
                debugln!("Parser::get_matches_with: setting TrailingVals=true");
                self.set(AS::TrailingValues);
                continue;
            }

            // Has the user already passed '--'? Meaning only positional args follow
            if !self.is_set(AS::TrailingValues) {
                // Does the arg match a subcommand name, or any of it's aliases (if defined)
                {
                    match needs_val_of {
                        ParseResult::Opt(_) | ParseResult::Pos(_) => (),
                        _ => {
                            let (is_match, sc_name) = self.possible_subcommand(&arg_os);
                            debugln!(
                                "Parser::get_matches_with: possible_sc={:?}, sc={:?}",
                                is_match,
                                sc_name
                            );
                            if is_match {
                                let sc_name = sc_name.expect(INTERNAL_ERROR_MSG);
                                if sc_name == "help" && self.is_set(AS::NeedsSubcommandHelp) {
                                    self.parse_help_subcommand(it)?;
                                }
                                subcmd_name = Some(sc_name.to_owned());
                                break;
                            }
                        }
                    }
                }

                if starts_new_arg {
                    let check_all = self.is_set(AS::AllArgsOverrideSelf);
                    {
                        let any_arg = find_any_by_name!(self, self.cache.unwrap_or(""));
                        matcher.process_arg_overrides(
                            any_arg,
                            &mut self.overrides,
                            &mut self.required,
                            check_all,
                        );
                    }

                    if arg_os.starts_with(b"--") {
                        needs_val_of = self.parse_long_arg(matcher, &arg_os, it)?;
                        debugln!(
                            "Parser:get_matches_with: After parse_long_arg {:?}",
                            needs_val_of
                        );
                        match needs_val_of {
                            ParseResult::Flag | ParseResult::Opt(..) | ParseResult::ValuesDone => {
                                continue
                            }
                            _ => (),
                        }
                    } else if arg_os.starts_with(b"-") && arg_os.len() != 1 {
                        // Try to parse short args like normal, if AllowLeadingHyphen or
                        // AllowNegativeNumbers is set, parse_short_arg will *not* throw
                        // an error, and instead return Ok(None)
                        needs_val_of = self.parse_short_arg(matcher, &arg_os)?;
                        // If it's None, we then check if one of those two AppSettings was set
                        debugln!(
                            "Parser:get_matches_with: After parse_short_arg {:?}",
                            needs_val_of
                        );
                        match needs_val_of {
                            ParseResult::MaybeNegNum => {
                                if !(arg_os.to_string_lossy().parse::<i64>().is_ok()
                                    || arg_os.to_string_lossy().parse::<f64>().is_ok())
                                {
                                    return Err(Error::unknown_argument(
                                        &*arg_os.to_string_lossy(),
                                        "",
                                        &*usage::create_error_usage(self, matcher, None),
                                        self.color(),
                                    ));
                                }
                            }
                            ParseResult::Opt(..) | ParseResult::Flag | ParseResult::ValuesDone => {
                                continue
                            }
                            _ => (),
                        }
                    }
                } else if let ParseResult::Opt(name) = needs_val_of {
                    // Check to see if parsing a value from a previous arg
                    let arg = self
                        .opts
                        .iter()
                        .find(|o| o.b.name == name)
                        .expect(INTERNAL_ERROR_MSG);
                    // get the OptBuilder so we can check the settings
                    needs_val_of = self.add_val_to_arg(arg, &arg_os, matcher)?;
                    // get the next value from the iterator
                    continue;
                }
            }

            if !(self.is_set(AS::ArgsNegateSubcommands) && self.is_set(AS::ValidArgFound))
                && !self.is_set(AS::InferSubcommands)
                && !self.is_set(AS::AllowExternalSubcommands)
            {
                if let Some(cdate) =
                    suggestions::did_you_mean(&*arg_os.to_string_lossy(), sc_names!(self))
                {
                    return Err(Error::invalid_subcommand(
                        arg_os.to_string_lossy().into_owned(),
                        cdate,
                        self.meta.bin_name.as_ref().unwrap_or(&self.meta.name),
                        &*usage::create_error_usage(self, matcher, None),
                        self.color(),
                    ));
                }
            }

            let low_index_mults = self.is_set(AS::LowIndexMultiplePositional)
                && pos_counter == (self.positionals.len() - 1);
            let missing_pos = self.is_set(AS::AllowMissingPositional)
                && (pos_counter == (self.positionals.len() - 1)
                    && !self.is_set(AS::TrailingValues));
            debugln!(
                "Parser::get_matches_with: Positional counter...{}",
                pos_counter
            );
            debugln!(
                "Parser::get_matches_with: Low index multiples...{:?}",
                low_index_mults
            );
            if low_index_mults || missing_pos {
                if let Some(na) = it.peek() {
                    let n = (*na).clone().into();
                    needs_val_of = if needs_val_of != ParseResult::ValuesDone {
                        if let Some(p) = self.positionals.get(pos_counter) {
                            ParseResult::Pos(p.b.name)
                        } else {
                            ParseResult::ValuesDone
                        }
                    } else {
                        ParseResult::ValuesDone
                    };
                    let sc_match = { self.possible_subcommand(&n).0 };
                    if self.is_new_arg(&n, needs_val_of)
                        || sc_match
                        || suggestions::did_you_mean(&n.to_string_lossy(), sc_names!(self))
                            .is_some()
                    {
                        debugln!("Parser::get_matches_with: Bumping the positional counter...");
                        pos_counter += 1;
                    }
                } else {
                    debugln!("Parser::get_matches_with: Bumping the positional counter...");
                    pos_counter += 1;
                }
            } else if (self.is_set(AS::AllowMissingPositional) && self.is_set(AS::TrailingValues))
                || (self.is_set(AS::ContainsLast) && self.is_set(AS::TrailingValues))
            {
                // Came to -- and one postional has .last(true) set, so we go immediately
                // to the last (highest index) positional
                debugln!("Parser::get_matches_with: .last(true) and --, setting last pos");
                pos_counter = self.positionals.len();
            }
            if let Some(p) = self.positionals.get(pos_counter) {
                if p.is_set(ArgSettings::Last) && !self.is_set(AS::TrailingValues) {
                    return Err(Error::unknown_argument(
                        &*arg_os.to_string_lossy(),
                        "",
                        &*usage::create_error_usage(self, matcher, None),
                        self.color(),
                    ));
                }
                if !self.is_set(AS::TrailingValues)
                    && (self.is_set(AS::TrailingVarArg) && pos_counter == self.positionals.len())
                {
                    self.settings.set(AS::TrailingValues);
                }
                if self.cache.map_or(true, |name| name != p.b.name) {
                    let check_all = self.is_set(AS::AllArgsOverrideSelf);
                    {
                        let any_arg = find_any_by_name!(self, self.cache.unwrap_or(""));
                        matcher.process_arg_overrides(
                            any_arg,
                            &mut self.overrides,
                            &mut self.required,
                            check_all,
                        );
                    }
                    self.cache = Some(p.b.name);
                }
                let _ = self.add_val_to_arg(p, &arg_os, matcher)?;

                matcher.inc_occurrence_of(p.b.name);
                let _ = self
                    .groups_for_arg(p.b.name)
                    .map(|vec| matcher.inc_occurrences_of(&*vec));

                self.settings.set(AS::ValidArgFound);
                // Only increment the positional counter if it doesn't allow multiples
                if !p.b.settings.is_set(ArgSettings::Multiple) {
                    pos_counter += 1;
                }
                self.settings.set(AS::ValidArgFound);
            } else if self.is_set(AS::AllowExternalSubcommands) {
                // Get external subcommand name
                let sc_name = match arg_os.to_str() {
                    Some(s) => s.to_string(),
                    None => {
                        if !self.is_set(AS::StrictUtf8) {
                            return Err(Error::invalid_utf8(
                                &*usage::create_error_usage(self, matcher, None),
                                self.color(),
                            ));
                        }
                        arg_os.to_string_lossy().into_owned()
                    }
                };

                // Collect the external subcommand args
                let mut sc_m = ArgMatcher::new();
                // Due to borrow rules, this has to be a while let...
                #[cfg_attr(feature = "cargo-clippy", allow(clippy::while_let_on_iterator))]
                while let Some(v) = it.next() {
                    let a = v.into();
                    if a.to_str().is_none() && !self.is_set(AS::StrictUtf8) {
                        return Err(Error::invalid_utf8(
                            &*usage::create_error_usage(self, matcher, None),
                            self.color(),
                        ));
                    }
                    sc_m.add_val_to("", &a);
                }

                matcher.subcommand(SubCommand {
                    name: sc_name,
                    matches: sc_m.into(),
                });
                sc_is_external = true;
            } else if !((self.is_set(AS::AllowLeadingHyphen)
                || self.is_set(AS::AllowNegativeNumbers))
                && arg_os.starts_with(b"-"))
                && !self.is_set(AS::InferSubcommands)
            {
                return Err(Error::unknown_argument(
                    &*arg_os.to_string_lossy(),
                    "",
                    &*usage::create_error_usage(self, matcher, None),
                    self.color(),
                ));
            } else if !has_args || self.is_set(AS::InferSubcommands) && self.has_subcommands() {
                if let Some(cdate) =
                    suggestions::did_you_mean(&*arg_os.to_string_lossy(), sc_names!(self))
                {
                    return Err(Error::invalid_subcommand(
                        arg_os.to_string_lossy().into_owned(),
                        cdate,
                        self.meta.bin_name.as_ref().unwrap_or(&self.meta.name),
                        &*usage::create_error_usage(self, matcher, None),
                        self.color(),
                    ));
                } else {
                    return Err(Error::unrecognized_subcommand(
                        arg_os.to_string_lossy().into_owned(),
                        self.meta.bin_name.as_ref().unwrap_or(&self.meta.name),
                        self.color(),
                    ));
                }
            } else {
                return Err(Error::unknown_argument(
                    &*arg_os.to_string_lossy(),
                    "",
                    &*usage::create_error_usage(self, matcher, None),
                    self.color(),
                ));
            }
        }

        if !sc_is_external {
            if let Some(ref pos_sc_name) = subcmd_name {
                let sc_name = {
                    find_subcmd!(self, pos_sc_name)
                        .expect(INTERNAL_ERROR_MSG)
                        .p
                        .meta
                        .name
                        .clone()
                };
                self.parse_subcommand(&*sc_name, matcher, it)?;
            } else if self.is_set(AS::SubcommandRequired) {
                let bn = self.meta.bin_name.as_ref().unwrap_or(&self.meta.name);
                return Err(Error::missing_subcommand(
                    bn,
                    &usage::create_error_usage(self, matcher, None),
                    self.color(),
                ));
            } else if self.is_set(AS::SubcommandRequiredElseHelp) {
                debugln!("Parser::get_matches_with: SubcommandRequiredElseHelp=true");
                let mut out = vec![];
                self.write_help_err(&mut out)?;
                return Err(Error {
                    message: String::from_utf8_lossy(&*out).into_owned(),
                    kind: ErrorKind::MissingArgumentOrSubcommand,
                    info: None,
                });
            }
        }

        // In case the last arg was new, we  need to process it's overrides
        let check_all = self.is_set(AS::AllArgsOverrideSelf);
        {
            let any_arg = find_any_by_name!(self, self.cache.unwrap_or(""));
            matcher.process_arg_overrides(
                any_arg,
                &mut self.overrides,
                &mut self.required,
                check_all,
            );
        }

        self.remove_overrides(matcher);

        Validator::new(self).validate(needs_val_of, subcmd_name, matcher)
    }

    fn remove_overrides(&mut self, matcher: &mut ArgMatcher) {
        debugln!("Parser::remove_overrides:{:?};", self.overrides);
        for &(overr, name) in &self.overrides {
            debugln!("Parser::remove_overrides:iter:({},{});", overr, name);
            if matcher.is_present(overr) {
                debugln!(
                    "Parser::remove_overrides:iter:({},{}): removing {};",
                    overr,
                    name,
                    name
                );
                matcher.remove(name);
                for i in (0..self.required.len()).rev() {
                    debugln!(
                        "Parser::remove_overrides:iter:({},{}): removing required {};",
                        overr,
                        name,
                        name
                    );
                    if self.required[i] == name {
                        self.required.swap_remove(i);
                        break;
                    }
                }
            }
        }
    }

    fn propagate_help_version(&mut self) {
        debugln!("Parser::propagate_help_version;");
        self.create_help_and_version();
        for sc in &mut self.subcommands {
            sc.p.propagate_help_version();
        }
    }

    fn build_bin_names(&mut self) {
        debugln!("Parser::build_bin_names;");
        for sc in &mut self.subcommands {
            debug!("Parser::build_bin_names:iter: bin_name set...");
            if sc.p.meta.bin_name.is_none() {
                sdebugln!("No");
                let bin_name = format!(
                    "{}{}{}",
                    self.meta
                        .bin_name
                        .as_ref()
                        .unwrap_or(&self.meta.name.clone()),
                    if self.meta.bin_name.is_some() {
                        " "
                    } else {
                        ""
                    },
                    &*sc.p.meta.name
                );
                debugln!(
                    "Parser::build_bin_names:iter: Setting bin_name of {} to {}",
                    self.meta.name,
                    bin_name
                );
                sc.p.meta.bin_name = Some(bin_name);
            } else {
                sdebugln!("yes ({:?})", sc.p.meta.bin_name);
            }
            debugln!(
                "Parser::build_bin_names:iter: Calling build_bin_names from...{}",
                sc.p.meta.name
            );
            sc.p.build_bin_names();
        }
    }

    fn parse_subcommand<I, T>(
        &mut self,
        sc_name: &str,
        matcher: &mut ArgMatcher<'a>,
        it: &mut Peekable<I>,
    ) -> ClapResult<()>
    where
        I: Iterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        use std::fmt::Write;
        debugln!("Parser::parse_subcommand;");
        let mut mid_string = String::new();
        if !self.is_set(AS::SubcommandsNegateReqs) {
            let mut hs: Vec<&str> = self.required.iter().map(|n| &**n).collect();
            for k in matcher.arg_names() {
                hs.push(k);
            }
            let reqs = usage::get_required_usage_from(self, &hs, Some(matcher), None, false);

            for s in &reqs {
                write!(&mut mid_string, " {}", s).expect(INTERNAL_ERROR_MSG);
            }
        }
        mid_string.push(' ');
        if let Some(ref mut sc) = self
            .subcommands
            .iter_mut()
            .find(|s| s.p.meta.name == sc_name)
        {
            let mut sc_matcher = ArgMatcher::new();
            // bin_name should be parent's bin_name + [<reqs>] + the sc's name separated by
            // a space
            sc.p.meta.usage = Some(format!(
                "{}{}{}",
                self.meta.bin_name.as_ref().unwrap_or(&String::new()),
                if self.meta.bin_name.is_some() {
                    &*mid_string
                } else {
                    ""
                },
                &*sc.p.meta.name
            ));
            sc.p.meta.bin_name = Some(format!(
                "{}{}{}",
                self.meta.bin_name.as_ref().unwrap_or(&String::new()),
                if self.meta.bin_name.is_some() {
                    " "
                } else {
                    ""
                },
                &*sc.p.meta.name
            ));
            debugln!(
                "Parser::parse_subcommand: About to parse sc={}",
                sc.p.meta.name
            );
            debugln!("Parser::parse_subcommand: sc settings={:#?}", sc.p.settings);
            sc.p.get_matches_with(&mut sc_matcher, it)?;
            matcher.subcommand(SubCommand {
                name: sc.p.meta.name.clone(),
                matches: sc_matcher.into(),
            });
        }
        Ok(())
    }

    pub fn groups_for_arg(&self, name: &str) -> Option<Vec<&'a str>> {
        debugln!("Parser::groups_for_arg: name={}", name);

        if self.groups.is_empty() {
            debugln!("Parser::groups_for_arg: No groups defined");
            return None;
        }
        let mut res = vec![];
        debugln!("Parser::groups_for_arg: Searching through groups...");
        for grp in &self.groups {
            for a in &grp.args {
                if a == &name {
                    sdebugln!("\tFound '{}'", grp.name);
                    res.push(&*grp.name);
                }
            }
        }
        if res.is_empty() {
            return None;
        }

        Some(res)
    }

    pub fn args_in_group(&self, group: &str) -> Vec<String> {
        debug_assert!(self.app_debug_asserts());

        let mut g_vec = vec![];
        let mut args = vec![];

        for n in &self
            .groups
            .iter()
            .find(|g| g.name == group)
            .expect(INTERNAL_ERROR_MSG)
            .args
        {
            if let Some(f) = self.flags.iter().find(|f| &f.b.name == n) {
                args.push(f.to_string());
            } else if let Some(f) = self.opts.iter().find(|o| &o.b.name == n) {
                args.push(f.to_string());
            } else if let Some(p) = self.positionals.values().find(|p| &p.b.name == n) {
                args.push(p.b.name.to_owned());
            } else {
                g_vec.push(*n);
            }
        }

        for av in g_vec.iter().map(|g| self.args_in_group(g)) {
            args.extend(av);
        }
        args.dedup();
        args.iter().map(ToOwned::to_owned).collect()
    }

    pub fn arg_names_in_group(&self, group: &str) -> Vec<&'a str> {
        let mut g_vec = vec![];
        let mut args = vec![];

        for n in &self
            .groups
            .iter()
            .find(|g| g.name == group)
            .expect(INTERNAL_ERROR_MSG)
            .args
        {
            if self.groups.iter().any(|g| g.name == *n) {
                args.extend(self.arg_names_in_group(n));
                g_vec.push(*n);
            } else if !args.contains(n) {
                args.push(*n);
            }
        }

        args.iter().copied().collect()
    }

    pub fn create_help_and_version(&mut self) {
        debugln!("Parser::create_help_and_version;");
        // name is "hclap_help" because flags are sorted by name
        if !self.is_set(AS::DisableHelpFlags) && !self.contains_long("help") {
            debugln!("Parser::create_help_and_version: Building --help");
            if self.help_short.is_none() && !self.contains_short('h') {
                self.help_short = Some('h');
            }
            let arg = FlagBuilder {
                b: Base {
                    name: "hclap_help",
                    help: self.help_message.or(Some("Prints help information")),
                    ..Default::default()
                },
                s: Switched {
                    short: self.help_short,
                    long: Some("help"),
                    ..Default::default()
                },
            };
            self.flags.push(arg);
        }
        if !self.is_set(AS::DisableVersion) && !self.contains_long("version") {
            debugln!("Parser::create_help_and_version: Building --version");
            if self.version_short.is_none() && !self.contains_short('V') {
                self.version_short = Some('V');
            }
            // name is "vclap_version" because flags are sorted by name
            let arg = FlagBuilder {
                b: Base {
                    name: "vclap_version",
                    help: self.version_message.or(Some("Prints version information")),
                    ..Default::default()
                },
                s: Switched {
                    short: self.version_short,
                    long: Some("version"),
                    ..Default::default()
                },
            };
            self.flags.push(arg);
        }
        if !self.subcommands.is_empty()
            && !self.is_set(AS::DisableHelpSubcommand)
            && self.is_set(AS::NeedsSubcommandHelp)
        {
            debugln!("Parser::create_help_and_version: Building help");
            self.subcommands.push(
                App::new("help")
                    .about("Prints this message or the help of the given subcommand(s)"),
            );
        }
    }

    // Retrieves the names of all args the user has supplied thus far, except required ones
    // because those will be listed in self.required
    fn check_for_help_and_version_str(&self, arg: &OsStr) -> ClapResult<()> {
        debugln!("Parser::check_for_help_and_version_str;");
        debug!(
            "Parser::check_for_help_and_version_str: Checking if --{} is help or version...",
            arg.to_str().unwrap()
        );
        if arg == "help" && self.is_set(AS::NeedsLongHelp) {
            sdebugln!("Help");
            return Err(self._help(true));
        }
        if arg == "version" && self.is_set(AS::NeedsLongVersion) {
            sdebugln!("Version");
            return Err(self._version(true));
        }
        sdebugln!("Neither");

        Ok(())
    }

    fn check_for_help_and_version_char(&self, arg: char) -> ClapResult<()> {
        debugln!("Parser::check_for_help_and_version_char;");
        debug!(
            "Parser::check_for_help_and_version_char: Checking if -{} is help or version...",
            arg
        );
        if let Some(h) = self.help_short {
            if arg == h && self.is_set(AS::NeedsLongHelp) {
                sdebugln!("Help");
                return Err(self._help(false));
            }
        }
        if let Some(v) = self.version_short {
            if arg == v && self.is_set(AS::NeedsLongVersion) {
                sdebugln!("Version");
                return Err(self._version(false));
            }
        }
        sdebugln!("Neither");
        Ok(())
    }

    fn use_long_help(&self) -> bool {
        // In this case, both must be checked. This allows the retention of
        // original formatting, but also ensures that the actual -h or --help
        // specified by the user is sent through. If HiddenShortHelp is not included,
        // then items specified with hidden_short_help will also be hidden.
        let should_long = |v: &Base| {
            v.long_help.is_some()
                || v.is_set(ArgSettings::HiddenLongHelp)
                || v.is_set(ArgSettings::HiddenShortHelp)
        };

        self.meta.long_about.is_some()
            || self.flags.iter().any(|f| should_long(&f.b))
            || self.opts.iter().any(|o| should_long(&o.b))
            || self.positionals.values().any(|p| should_long(&p.b))
            || self
                .subcommands
                .iter()
                .any(|s| s.p.meta.long_about.is_some())
    }

    fn _help(&self, mut use_long: bool) -> Error {
        debugln!("Parser::_help: use_long={:?}", use_long);
        use_long = use_long && self.use_long_help();
        let mut buf = vec![];
        match Help::write_parser_help(&mut buf, self, use_long) {
            Err(e) => e,
            _ => Error {
                message: String::from_utf8(buf).unwrap_or_default(),
                kind: ErrorKind::HelpDisplayed,
                info: None,
            },
        }
    }

    fn _version(&self, use_long: bool) -> Error {
        debugln!("Parser::_version: ");
        let out = io::stdout();
        let mut buf_w = BufWriter::new(out.lock());
        match self.print_version(&mut buf_w, use_long) {
            Err(e) => e,
            _ => Error {
                message: String::new(),
                kind: ErrorKind::VersionDisplayed,
                info: None,
            },
        }
    }

    fn parse_long_arg<I, T>(
        &mut self,
        matcher: &mut ArgMatcher<'a>,
        full_arg: &OsStr,
        it: &mut Peekable<I>,
    ) -> ClapResult<ParseResult<'a>>
    where
        I: Iterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        // maybe here lifetime should be 'a
        debugln!("Parser::parse_long_arg;");

        // Update the current index
        self.cur_idx.set(self.cur_idx.get() + 1);

        let mut val = None;
        debug!("Parser::parse_long_arg: Does it contain '='...");
        let arg = if full_arg.contains_byte(b'=') {
            let (p0, p1) = full_arg.trim_left_matches(b'-').split_at_byte(b'=');
            sdebugln!("Yes '{:?}'", p1);
            val = Some(p1);
            p0
        } else {
            sdebugln!("No");
            full_arg.trim_left_matches(b'-')
        };

        if let Some(opt) = find_opt_by_long!(@os self, arg) {
            debugln!(
                "Parser::parse_long_arg: Found valid opt '{}'",
                opt.to_string()
            );
            self.settings.set(AS::ValidArgFound);
            let ret = self.parse_opt(val, opt, val.is_some(), matcher)?;
            if self.cache.map_or(true, |name| name != opt.b.name) {
                self.cache = Some(opt.b.name);
            }

            return Ok(ret);
        } else if let Some(flag) = find_flag_by_long!(@os self, arg) {
            debugln!(
                "Parser::parse_long_arg: Found valid flag '{}'",
                flag.to_string()
            );
            self.settings.set(AS::ValidArgFound);
            // Only flags could be help or version, and we need to check the raw long
            // so this is the first point to check
            self.check_for_help_and_version_str(arg)?;

            self.parse_flag(flag, matcher)?;

            // Handle conflicts, requirements, etc.
            if self.cache.map_or(true, |name| name != flag.b.name) {
                self.cache = Some(flag.b.name);
            }

            return Ok(ParseResult::Flag);
        } else if self.is_set(AS::AllowLeadingHyphen) {
            return Ok(ParseResult::MaybeHyphenValue);
        } else if self.is_set(AS::ValidNegNumFound) {
            return Ok(ParseResult::MaybeNegNum);
        }

        debugln!("Parser::parse_long_arg: Didn't match anything");

        let args_rest: Vec<_> = it.map(|x| x.into()).collect();
        let args_rest2: Vec<_> = args_rest
            .iter()
            .map(|x| x.to_str().expect(INVALID_UTF8))
            .collect();
        self.did_you_mean_error(arg.to_str().expect(INVALID_UTF8), matcher, &args_rest2[..])
            .map(|_| ParseResult::NotFound)
    }

    fn parse_short_arg(
        &mut self,
        matcher: &mut ArgMatcher<'a>,
        full_arg: &OsStr,
    ) -> ClapResult<ParseResult<'a>> {
        debugln!("Parser::parse_short_arg: full_arg={:?}", full_arg);
        let arg_os = full_arg.trim_left_matches(b'-');
        let arg = arg_os.to_string_lossy();

        // If AllowLeadingHyphen is set, we want to ensure `-val` gets parsed as `-val` and not
        // `-v` `-a` `-l` assuming `v` `a` and `l` are all, or mostly, valid shorts.
        if self.is_set(AS::AllowLeadingHyphen) {
            if arg.chars().any(|c| !self.contains_short(c)) {
                debugln!(
                    "Parser::parse_short_arg: LeadingHyphenAllowed yet -{} isn't valid",
                    arg
                );
                return Ok(ParseResult::MaybeHyphenValue);
            }
        } else if self.is_set(AS::ValidNegNumFound) {
            // TODO: Add docs about having AllowNegativeNumbers and `-2` as a valid short
            // May be better to move this to *after* not finding a valid flag/opt?
            debugln!("Parser::parse_short_arg: Valid negative num...");
            return Ok(ParseResult::MaybeNegNum);
        }

        let mut ret = ParseResult::NotFound;
        for c in arg.chars() {
            debugln!("Parser::parse_short_arg:iter:{}", c);

            // update each index because `-abcd` is four indices to clap
            self.cur_idx.set(self.cur_idx.get() + 1);

            // Check for matching short options, and return the name if there is no trailing
            // concatenated value: -oval
            // Option: -o
            // Value: val
            if let Some(opt) = find_opt_by_short!(self, c) {
                debugln!("Parser::parse_short_arg:iter:{}: Found valid opt", c);
                self.settings.set(AS::ValidArgFound);
                // Check for trailing concatenated value
                let p: Vec<_> = arg.splitn(2, c).collect();
                debugln!(
                    "Parser::parse_short_arg:iter:{}: p[0]={:?}, p[1]={:?}",
                    c,
                    p[0].as_bytes(),
                    p[1].as_bytes()
                );
                let i = p[0].as_bytes().len() + 1;
                let val = if !p[1].as_bytes().is_empty() {
                    debugln!(
                        "Parser::parse_short_arg:iter:{}: val={:?} (bytes), val={:?} (ascii)",
                        c,
                        arg_os.split_at(i).1.as_bytes(),
                        arg_os.split_at(i).1
                    );
                    Some(arg_os.split_at(i).1)
                } else {
                    None
                };

                // Default to "we're expecting a value later"
                let ret = self.parse_opt(val, opt, false, matcher)?;

                if self.cache.map_or(true, |name| name != opt.b.name) {
                    self.cache = Some(opt.b.name);
                }

                return Ok(ret);
            } else if let Some(flag) = find_flag_by_short!(self, c) {
                debugln!("Parser::parse_short_arg:iter:{}: Found valid flag", c);
                self.settings.set(AS::ValidArgFound);
                // Only flags can be help or version
                self.check_for_help_and_version_char(c)?;
                ret = self.parse_flag(flag, matcher)?;

                // Handle conflicts, requirements, overrides, etc.
                // Must be called here due to mutabililty
                if self.cache.map_or(true, |name| name != flag.b.name) {
                    self.cache = Some(flag.b.name);
                }
            } else {
                let arg = format!("-{}", c);
                return Err(Error::unknown_argument(
                    &*arg,
                    "",
                    &*usage::create_error_usage(self, matcher, None),
                    self.color(),
                ));
            }
        }
        Ok(ret)
    }

    fn parse_opt(
        &self,
        val: Option<&OsStr>,
        opt: &OptBuilder<'a, 'b>,
        had_eq: bool,
        matcher: &mut ArgMatcher<'a>,
    ) -> ClapResult<ParseResult<'a>> {
        debugln!("Parser::parse_opt; opt={}, val={:?}", opt.b.name, val);
        debugln!("Parser::parse_opt; opt.settings={:?}", opt.b.settings);
        let mut has_eq = false;
        let no_val = val.is_none();
        let empty_vals = opt.is_set(ArgSettings::EmptyValues);
        let min_vals_zero = opt.v.min_vals.unwrap_or(1) == 0;
        let needs_eq = opt.is_set(ArgSettings::RequireEquals);

        debug!("Parser::parse_opt; Checking for val...");
        if let Some(fv) = val {
            has_eq = fv.starts_with(&[b'=']) || had_eq;
            let v = fv.trim_left_matches(b'=');
            if !empty_vals && (v.is_empty() || (needs_eq && !has_eq)) {
                sdebugln!("Found Empty - Error");
                return Err(Error::empty_value(
                    opt,
                    &*usage::create_error_usage(self, matcher, None),
                    self.color(),
                ));
            }
            sdebugln!("Found - {:?}, len: {}", v, v.len());
            debugln!(
                "Parser::parse_opt: {:?} contains '='...{:?}",
                fv,
                fv.starts_with(&[b'='])
            );
            self.add_val_to_arg(opt, v, matcher)?;
        } else if needs_eq && !(empty_vals || min_vals_zero) {
            sdebugln!("None, but requires equals...Error");
            return Err(Error::empty_value(
                opt,
                &*usage::create_error_usage(self, matcher, None),
                self.color(),
            ));
        } else {
            sdebugln!("None");
        }

        matcher.inc_occurrence_of(opt.b.name);
        // Increment or create the group "args"
        if let Some(vec) = self.groups_for_arg(opt.b.name) {
            matcher.inc_occurrences_of(&*vec);
        }

        let needs_delim = opt.is_set(ArgSettings::RequireDelimiter);
        let mult = opt.is_set(ArgSettings::Multiple);
        if no_val && min_vals_zero && !has_eq && needs_eq {
            debugln!("Parser::parse_opt: More arg vals not required...");
            return Ok(ParseResult::ValuesDone);
        } else if no_val || (mult && !needs_delim) && !has_eq && matcher.needs_more_vals(opt) {
            debugln!("Parser::parse_opt: More arg vals required...");
            return Ok(ParseResult::Opt(opt.b.name));
        }
        debugln!("Parser::parse_opt: More arg vals not required...");
        Ok(ParseResult::ValuesDone)
    }

    fn add_val_to_arg<A>(
        &self,
        arg: &A,
        val: &OsStr,
        matcher: &mut ArgMatcher<'a>,
    ) -> ClapResult<ParseResult<'a>>
    where
        A: AnyArg<'a, 'b> + Display,
    {
        debugln!("Parser::add_val_to_arg; arg={}, val={:?}", arg.name(), val);
        debugln!(
            "Parser::add_val_to_arg; trailing_vals={:?}, DontDelimTrailingVals={:?}",
            self.is_set(AS::TrailingValues),
            self.is_set(AS::DontDelimitTrailingValues)
        );
        if !(self.is_set(AS::TrailingValues) && self.is_set(AS::DontDelimitTrailingValues)) {
            if let Some(delim) = arg.val_delim() {
                if val.is_empty() {
                    Ok(self.add_single_val_to_arg(arg, val, matcher)?)
                } else {
                    let mut iret = ParseResult::ValuesDone;
                    for v in val.split(delim as u32 as u8) {
                        iret = self.add_single_val_to_arg(arg, v, matcher)?;
                    }
                    // If there was a delimiter used, we're not looking for more values
                    if val.contains_byte(delim as u32 as u8)
                        || arg.is_set(ArgSettings::RequireDelimiter)
                    {
                        iret = ParseResult::ValuesDone;
                    }
                    Ok(iret)
                }
            } else {
                self.add_single_val_to_arg(arg, val, matcher)
            }
        } else {
            self.add_single_val_to_arg(arg, val, matcher)
        }
    }

    fn add_single_val_to_arg<A>(
        &self,
        arg: &A,
        v: &OsStr,
        matcher: &mut ArgMatcher<'a>,
    ) -> ClapResult<ParseResult<'a>>
    where
        A: AnyArg<'a, 'b> + Display,
    {
        debugln!("Parser::add_single_val_to_arg;");
        debugln!("Parser::add_single_val_to_arg: adding val...{:?}", v);

        // update the current index because each value is a distinct index to clap
        self.cur_idx.set(self.cur_idx.get() + 1);

        // @TODO @docs @p4: docs for indices should probably note that a terminator isn't a value
        // and therefore not reported in indices
        if let Some(t) = arg.val_terminator() {
            if t == v {
                return Ok(ParseResult::ValuesDone);
            }
        }

        matcher.add_val_to(arg.name(), v);
        matcher.add_index_to(arg.name(), self.cur_idx.get());

        // Increment or create the group "args"
        if let Some(grps) = self.groups_for_arg(arg.name()) {
            for grp in grps {
                matcher.add_val_to(&*grp, v);
            }
        }

        if matcher.needs_more_vals(arg) {
            return Ok(ParseResult::Opt(arg.name()));
        }
        Ok(ParseResult::ValuesDone)
    }

    fn parse_flag(
        &self,
        flag: &FlagBuilder<'a, 'b>,
        matcher: &mut ArgMatcher<'a>,
    ) -> ClapResult<ParseResult<'a>> {
        debugln!("Parser::parse_flag;");

        matcher.inc_occurrence_of(flag.b.name);
        matcher.add_index_to(flag.b.name, self.cur_idx.get());

        // Increment or create the group "args"
        if let Some(vec) = self.groups_for_arg(flag.b.name) {
            matcher.inc_occurrences_of(&*vec);
        }

        Ok(ParseResult::Flag)
    }

    fn did_you_mean_error(
        &self,
        arg: &str,
        matcher: &mut ArgMatcher<'a>,
        args_rest: &[&str],
    ) -> ClapResult<()> {
        // Didn't match a flag or option
        let suffix =
            suggestions::did_you_mean_flag_suffix(arg, args_rest, longs!(self), &self.subcommands);

        // Add the arg to the matches to build a proper usage string
        if let Some(name) = suffix.1 {
            if let Some(opt) = find_opt_by_long!(self, name) {
                if let Some(grps) = self.groups_for_arg(&*opt.b.name) {
                    matcher.inc_occurrences_of(&*grps);
                }
                matcher.insert(&*opt.b.name);
            } else if let Some(flg) = find_flag_by_long!(self, name) {
                if let Some(grps) = self.groups_for_arg(&*flg.b.name) {
                    matcher.inc_occurrences_of(&*grps);
                }
                matcher.insert(&*flg.b.name);
            }
        }

        let used_arg = format!("--{}", arg);
        Err(Error::unknown_argument(
            &*used_arg,
            &*suffix.0,
            &*usage::create_error_usage(self, matcher, None),
            self.color(),
        ))
    }

    // Prints the version to the user and exits if quit=true
    fn print_version<W: Write>(&self, w: &mut W, use_long: bool) -> ClapResult<()> {
        self.write_version(w, use_long)?;
        w.flush().map_err(Error::from)
    }

    pub fn write_version<W: Write>(&self, w: &mut W, use_long: bool) -> io::Result<()> {
        let ver = if use_long {
            self.meta
                .long_version
                .unwrap_or_else(|| self.meta.version.unwrap_or(""))
        } else {
            self.meta
                .version
                .unwrap_or_else(|| self.meta.long_version.unwrap_or(""))
        };
        if let Some(bn) = self.meta.bin_name.as_ref() {
            if bn.contains(' ') {
                // Incase we're dealing with subcommands i.e. git mv is translated to git-mv
                write!(w, "{} {}", bn.replace(" ", "-"), ver)
            } else {
                write!(w, "{} {}", &self.meta.name[..], ver)
            }
        } else {
            write!(w, "{} {}", &self.meta.name[..], ver)
        }
    }

    pub fn print_help(&self) -> ClapResult<()> {
        let out = io::stdout();
        let mut buf_w = BufWriter::new(out.lock());
        self.write_help(&mut buf_w)
    }

    pub fn write_help<W: Write>(&self, w: &mut W) -> ClapResult<()> {
        Help::write_parser_help(w, self, false)
    }

    pub fn write_long_help<W: Write>(&self, w: &mut W) -> ClapResult<()> {
        Help::write_parser_help(w, self, true)
    }

    pub fn write_help_err<W: Write>(&self, w: &mut W) -> ClapResult<()> {
        Help::write_parser_help_to_stderr(w, self)
    }

    pub fn add_defaults(&mut self, matcher: &mut ArgMatcher<'a>) -> ClapResult<()> {
        debugln!("Parser::add_defaults;");
        macro_rules! add_val {
            (@default $_self:ident, $a:ident, $m:ident) => {
                if let Some(ref val) = $a.v.default_val {
                    debugln!("Parser::add_defaults:iter:{}: has default vals", $a.b.name);
                    if $m.get($a.b.name).map(|ma| ma.vals.len()).map(|len| len == 0).unwrap_or(false) {
                        debugln!("Parser::add_defaults:iter:{}: has no user defined vals", $a.b.name);
                        $_self.add_val_to_arg($a, OsStr::new(val), $m)?;

                        if $_self.cache.map_or(true, |name| name != $a.name()) {
                            $_self.cache = Some($a.name());
                        }
                    } else if $m.get($a.b.name).is_some() {
                        debugln!("Parser::add_defaults:iter:{}: has user defined vals", $a.b.name);
                    } else {
                        debugln!("Parser::add_defaults:iter:{}: wasn't used", $a.b.name);

                        $_self.add_val_to_arg($a, OsStr::new(val), $m)?;

                        if $_self.cache.map_or(true, |name| name != $a.name()) {
                            $_self.cache = Some($a.name());
                        }
                    }
                } else {
                    debugln!("Parser::add_defaults:iter:{}: doesn't have default vals", $a.b.name);
                }
            };
            ($_self:ident, $a:ident, $m:ident) => {
                if let Some(ref vm) = $a.v.default_vals_ifs {
                    sdebugln!(" has conditional defaults");
                    let mut done = false;
                    if $m.get($a.b.name).is_none() {
                        for &(arg, val, default) in vm.values() {
                            let add = if let Some(a) = $m.get(arg) {
                                if let Some(v) = val {
                                    a.vals.iter().any(|value| v == value)
                                } else {
                                    true
                                }
                            } else {
                                false
                            };
                            if add {
                                $_self.add_val_to_arg($a, OsStr::new(default), $m)?;
                                if $_self.cache.map_or(true, |name| name != $a.name()) {
                                    $_self.cache = Some($a.name());
                                }
                                done = true;
                                break;
                            }
                        }
                    }

                    if done {
                        continue; // outer loop (outside macro)
                    }
                } else {
                    sdebugln!(" doesn't have conditional defaults");
                }
                add_val!(@default $_self, $a, $m)
            };
        }

        for o in &self.opts {
            debug!("Parser::add_defaults:iter:{}:", o.b.name);
            add_val!(self, o, matcher);
        }
        for p in self.positionals.values() {
            debug!("Parser::add_defaults:iter:{}:", p.b.name);
            add_val!(self, p, matcher);
        }
        Ok(())
    }

    pub fn add_env(&mut self, matcher: &mut ArgMatcher<'a>) -> ClapResult<()> {
        macro_rules! add_val {
            ($_self:ident, $a:ident, $m:ident) => {
                if let Some(ref val) = $a.v.env {
                    if $m
                        .get($a.b.name)
                        .map(|ma| ma.vals.len())
                        .map(|len| len == 0)
                        .unwrap_or(false)
                    {
                        if let Some(ref val) = val.1 {
                            $_self.add_val_to_arg($a, OsStr::new(val), $m)?;

                            if $_self.cache.map_or(true, |name| name != $a.name()) {
                                $_self.cache = Some($a.name());
                            }
                        }
                    } else {
                        if let Some(ref val) = val.1 {
                            $_self.add_val_to_arg($a, OsStr::new(val), $m)?;

                            if $_self.cache.map_or(true, |name| name != $a.name()) {
                                $_self.cache = Some($a.name());
                            }
                        }
                    }
                }
            };
        }

        for o in &self.opts {
            add_val!(self, o, matcher);
        }
        for p in self.positionals.values() {
            add_val!(self, p, matcher);
        }
        Ok(())
    }

    pub fn flags(&self) -> Iter<FlagBuilder<'a, 'b>> {
        self.flags.iter()
    }

    pub fn opts(&self) -> Iter<OptBuilder<'a, 'b>> {
        self.opts.iter()
    }

    pub fn positionals(&self) -> map::Values<PosBuilder<'a, 'b>> {
        self.positionals.values()
    }

    pub fn subcommands(&self) -> Iter<App> {
        self.subcommands.iter()
    }

    // Should we color the output? None=determined by output location, true=yes, false=no
    #[doc(hidden)]
    pub fn color(&self) -> ColorWhen {
        debugln!("Parser::color;");
        debug!("Parser::color: Color setting...");
        if self.is_set(AS::ColorNever) {
            sdebugln!("Never");
            ColorWhen::Never
        } else if self.is_set(AS::ColorAlways) {
            sdebugln!("Always");
            ColorWhen::Always
        } else {
            sdebugln!("Auto");
            ColorWhen::Auto
        }
    }

    pub fn find_any_arg(&self, name: &str) -> Option<&AnyArg<'a, 'b>> {
        if let Some(f) = find_by_name!(self, name, flags, iter) {
            return Some(f);
        }
        if let Some(o) = find_by_name!(self, name, opts, iter) {
            return Some(o);
        }
        if let Some(p) = find_by_name!(self, name, positionals, values) {
            return Some(p);
        }
        None
    }

    /// Check is a given string matches the binary name for this parser
    fn is_bin_name(&self, value: &str) -> bool {
        self.meta
            .bin_name
            .as_ref()
            .map(|name| value == name)
            .unwrap_or(false)
    }

    /// Check is a given string is an alias for this parser
    fn is_alias(&self, value: &str) -> bool {
        self.meta
            .aliases
            .as_ref()
            .map(|aliases| {
                for alias in aliases {
                    if alias.0 == value {
                        return true;
                    }
                }
                false
            })
            .unwrap_or(false)
    }

    // Only used for completion scripts due to bin_name messiness
    #[cfg_attr(feature = "lints", allow(block_in_if_condition_stmt))]
    pub fn find_subcommand(&'b self, sc: &str) -> Option<&'b App<'a, 'b>> {
        debugln!("Parser::find_subcommand: sc={}", sc);
        debugln!(
            "Parser::find_subcommand: Currently in Parser...{}",
            self.meta.bin_name.as_ref().unwrap()
        );
        for s in &self.subcommands {
            if s.p.is_bin_name(sc) {
                return Some(s);
            }
            // XXX: why do we split here?
            // isn't `sc` supposed to be single word already?
            let last = sc.split(' ').rev().next().expect(INTERNAL_ERROR_MSG);
            if s.p.is_alias(last) {
                return Some(s);
            }

            if let Some(app) = s.p.find_subcommand(sc) {
                return Some(app);
            }
        }
        None
    }

    #[inline]
    fn contains_long(&self, l: &str) -> bool {
        longs!(self).any(|al| al == &l)
    }

    #[inline]
    fn contains_short(&self, s: char) -> bool {
        shorts!(self).any(|arg_s| arg_s == &s)
    }
}
