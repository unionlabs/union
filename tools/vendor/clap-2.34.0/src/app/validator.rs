// std
#[allow(deprecated, unused_imports)]
use std::{ascii::AsciiExt, fmt::Display};

// Internal
use crate::{
    app::{
        parser::{ParseResult, Parser},
        settings::AppSettings as AS,
        usage,
    },
    args::{settings::ArgSettings, AnyArg, ArgMatcher, MatchedArg},
    errors::{Error, ErrorKind, Result as ClapResult},
    fmt::{Colorizer, ColorizerOption},
    INTERNAL_ERROR_MSG, INVALID_UTF8,
};

pub struct Validator<'a, 'b, 'z>(&'z mut Parser<'a, 'b>)
where
    'a: 'b,
    'b: 'z;

impl<'a, 'b, 'z> Validator<'a, 'b, 'z> {
    pub fn new(p: &'z mut Parser<'a, 'b>) -> Self {
        Validator(p)
    }

    pub fn validate(
        &mut self,
        needs_val_of: ParseResult<'a>,
        subcmd_name: Option<String>,
        matcher: &mut ArgMatcher<'a>,
    ) -> ClapResult<()> {
        debugln!("Validator::validate;");
        let mut reqs_validated = false;
        self.0.add_env(matcher)?;
        self.0.add_defaults(matcher)?;
        if let ParseResult::Opt(a) = needs_val_of {
            debugln!("Validator::validate: needs_val_of={:?}", a);
            let o = {
                self.0
                    .opts
                    .iter()
                    .find(|o| o.b.name == a)
                    .expect(INTERNAL_ERROR_MSG)
                    .clone()
            };
            self.validate_required(matcher)?;
            reqs_validated = true;
            let should_err = if let Some(v) = matcher.0.args.get(&*o.b.name) {
                v.vals.is_empty() && !(o.v.min_vals.is_some() && o.v.min_vals.unwrap() == 0)
            } else {
                true
            };
            if should_err {
                return Err(Error::empty_value(
                    &o,
                    &*usage::create_error_usage(self.0, matcher, None),
                    self.0.color(),
                ));
            }
        }

        if matcher.is_empty()
            && matcher.subcommand_name().is_none()
            && self.0.is_set(AS::ArgRequiredElseHelp)
        {
            let mut out = vec![];
            self.0.write_help_err(&mut out)?;
            return Err(Error {
                message: String::from_utf8_lossy(&*out).into_owned(),
                kind: ErrorKind::MissingArgumentOrSubcommand,
                info: None,
            });
        }
        self.validate_blacklist(matcher)?;
        if !(reqs_validated || self.0.is_set(AS::SubcommandsNegateReqs) && subcmd_name.is_some()) {
            self.validate_required(matcher)?;
        }
        self.validate_matched_args(matcher)?;
        matcher.usage(usage::create_usage_with_title(self.0, &[]));

        Ok(())
    }

    fn validate_arg_values<A>(
        &self,
        arg: &A,
        ma: &MatchedArg,
        matcher: &ArgMatcher<'a>,
    ) -> ClapResult<()>
    where
        A: AnyArg<'a, 'b> + Display,
    {
        debugln!("Validator::validate_arg_values: arg={:?}", arg.name());
        for val in &ma.vals {
            if self.0.is_set(AS::StrictUtf8) && val.to_str().is_none() {
                debugln!(
                    "Validator::validate_arg_values: invalid UTF-8 found in val {:?}",
                    val
                );
                return Err(Error::invalid_utf8(
                    &*usage::create_error_usage(self.0, matcher, None),
                    self.0.color(),
                ));
            }
            if let Some(p_vals) = arg.possible_vals() {
                debugln!("Validator::validate_arg_values: possible_vals={:?}", p_vals);
                let val_str = val.to_string_lossy();
                let ok = if arg.is_set(ArgSettings::CaseInsensitive) {
                    p_vals.iter().any(|pv| pv.eq_ignore_ascii_case(&*val_str))
                } else {
                    p_vals.contains(&&*val_str)
                };
                if !ok {
                    return Err(Error::invalid_value(
                        val_str,
                        p_vals,
                        arg,
                        &*usage::create_error_usage(self.0, matcher, None),
                        self.0.color(),
                    ));
                }
            }
            if !arg.is_set(ArgSettings::EmptyValues)
                && val.is_empty()
                && matcher.contains(&*arg.name())
            {
                debugln!("Validator::validate_arg_values: illegal empty val found");
                return Err(Error::empty_value(
                    arg,
                    &*usage::create_error_usage(self.0, matcher, None),
                    self.0.color(),
                ));
            }
            if let Some(vtor) = arg.validator() {
                debug!("Validator::validate_arg_values: checking validator...");
                if let Err(e) = vtor(val.to_string_lossy().into_owned()) {
                    sdebugln!("error");
                    return Err(Error::value_validation(Some(arg), e, self.0.color()));
                } else {
                    sdebugln!("good");
                }
            }
            if let Some(vtor) = arg.validator_os() {
                debug!("Validator::validate_arg_values: checking validator_os...");
                if let Err(e) = vtor(val) {
                    sdebugln!("error");
                    return Err(Error::value_validation(
                        Some(arg),
                        (*e).to_string_lossy().to_string(),
                        self.0.color(),
                    ));
                } else {
                    sdebugln!("good");
                }
            }
        }
        Ok(())
    }

    fn build_err(&self, name: &str, matcher: &ArgMatcher) -> ClapResult<()> {
        debugln!("build_err!: name={}", name);
        let mut c_with = find_from!(self.0, &name, blacklist, matcher);
        c_with = c_with.or_else(|| {
            self.0
                .find_any_arg(name)
                .and_then(|aa| aa.blacklist())
                .and_then(|bl| bl.iter().find(|arg| matcher.contains(arg)))
                .and_then(|an| self.0.find_any_arg(an))
                .map(|aa| format!("{}", aa))
        });
        debugln!("build_err!: '{:?}' conflicts with '{}'", c_with, &name);
        //        matcher.remove(&name);
        let usg = usage::create_error_usage(self.0, matcher, None);
        if let Some(f) = find_by_name!(self.0, name, flags, iter) {
            debugln!("build_err!: It was a flag...");
            Err(Error::argument_conflict(f, c_with, &*usg, self.0.color()))
        } else if let Some(o) = find_by_name!(self.0, name, opts, iter) {
            debugln!("build_err!: It was an option...");
            Err(Error::argument_conflict(o, c_with, &*usg, self.0.color()))
        } else {
            match find_by_name!(self.0, name, positionals, values) {
                Some(p) => {
                    debugln!("build_err!: It was a positional...");
                    Err(Error::argument_conflict(p, c_with, &*usg, self.0.color()))
                }
                None => panic!("{}", INTERNAL_ERROR_MSG),
            }
        }
    }

    fn validate_blacklist(&self, matcher: &mut ArgMatcher) -> ClapResult<()> {
        debugln!("Validator::validate_blacklist;");
        let mut conflicts: Vec<&str> = vec![];
        for (&name, _) in matcher.iter() {
            debugln!("Validator::validate_blacklist:iter:{};", name);
            if let Some(grps) = self.0.groups_for_arg(name) {
                for grp in &grps {
                    if let Some(g) = self.0.groups.iter().find(|g| &g.name == grp) {
                        if !g.multiple {
                            for arg in &g.args {
                                if arg == &name {
                                    continue;
                                }
                                conflicts.push(arg);
                            }
                        }
                        if let Some(ref gc) = g.conflicts {
                            conflicts.extend(&*gc);
                        }
                    }
                }
            }
            if let Some(arg) = find_any_by_name!(self.0, name) {
                if let Some(bl) = arg.blacklist() {
                    for conf in bl {
                        if matcher.get(conf).is_some() {
                            conflicts.push(conf);
                        }
                    }
                }
            } else {
                debugln!("Validator::validate_blacklist:iter:{}:group;", name);
                let args = self.0.arg_names_in_group(name);
                for arg in &args {
                    debugln!(
                        "Validator::validate_blacklist:iter:{}:group:iter:{};",
                        name,
                        arg
                    );
                    if let Some(bl) = find_any_by_name!(self.0, *arg).unwrap().blacklist() {
                        for conf in bl {
                            if matcher.get(conf).is_some() {
                                conflicts.push(conf);
                            }
                        }
                    }
                }
            }
        }

        for name in &conflicts {
            debugln!(
                "Validator::validate_blacklist:iter:{}: Checking blacklisted arg",
                name
            );
            let mut should_err = false;
            if self.0.groups.iter().any(|g| &g.name == name) {
                debugln!(
                    "Validator::validate_blacklist:iter:{}: groups contains it...",
                    name
                );
                for n in self.0.arg_names_in_group(name) {
                    debugln!(
                        "Validator::validate_blacklist:iter:{}:iter:{}: looking in group...",
                        name,
                        n
                    );
                    if matcher.contains(n) {
                        debugln!(
                            "Validator::validate_blacklist:iter:{}:iter:{}: matcher contains it...",
                            name,
                            n
                        );
                        return self.build_err(n, matcher);
                    }
                }
            } else if let Some(ma) = matcher.get(name) {
                debugln!(
                    "Validator::validate_blacklist:iter:{}: matcher contains it...",
                    name
                );
                should_err = ma.occurs > 0;
            }
            if should_err {
                return self.build_err(*name, matcher);
            }
        }
        Ok(())
    }

    fn validate_matched_args(&self, matcher: &mut ArgMatcher<'a>) -> ClapResult<()> {
        debugln!("Validator::validate_matched_args;");
        for (name, ma) in matcher.iter() {
            debugln!(
                "Validator::validate_matched_args:iter:{}: vals={:#?}",
                name,
                ma.vals
            );
            if let Some(opt) = find_by_name!(self.0, *name, opts, iter) {
                self.validate_arg_num_vals(opt, ma, matcher)?;
                self.validate_arg_values(opt, ma, matcher)?;
                self.validate_arg_requires(opt, ma, matcher)?;
                self.validate_arg_num_occurs(opt, ma, matcher)?;
            } else if let Some(flag) = find_by_name!(self.0, *name, flags, iter) {
                self.validate_arg_requires(flag, ma, matcher)?;
                self.validate_arg_num_occurs(flag, ma, matcher)?;
            } else if let Some(pos) = find_by_name!(self.0, *name, positionals, values) {
                self.validate_arg_num_vals(pos, ma, matcher)?;
                self.validate_arg_num_occurs(pos, ma, matcher)?;
                self.validate_arg_values(pos, ma, matcher)?;
                self.validate_arg_requires(pos, ma, matcher)?;
            } else {
                let grp = self
                    .0
                    .groups
                    .iter()
                    .find(|g| &g.name == name)
                    .expect(INTERNAL_ERROR_MSG);
                if let Some(ref g_reqs) = grp.requires {
                    if g_reqs.iter().any(|&n| !matcher.contains(n)) {
                        return self.missing_required_error(matcher, None);
                    }
                }
            }
        }
        Ok(())
    }

    fn validate_arg_num_occurs<A>(
        &self,
        a: &A,
        ma: &MatchedArg,
        matcher: &ArgMatcher,
    ) -> ClapResult<()>
    where
        A: AnyArg<'a, 'b> + Display,
    {
        debugln!("Validator::validate_arg_num_occurs: a={};", a.name());
        if ma.occurs > 1 && !a.is_set(ArgSettings::Multiple) {
            // Not the first time, and we don't allow multiples
            return Err(Error::unexpected_multiple_usage(
                a,
                &*usage::create_error_usage(self.0, matcher, None),
                self.0.color(),
            ));
        }
        Ok(())
    }

    fn validate_arg_num_vals<A>(
        &self,
        a: &A,
        ma: &MatchedArg,
        matcher: &ArgMatcher,
    ) -> ClapResult<()>
    where
        A: AnyArg<'a, 'b> + Display,
    {
        debugln!("Validator::validate_arg_num_vals:{}", a.name());
        if let Some(num) = a.num_vals() {
            debugln!("Validator::validate_arg_num_vals: num_vals set...{}", num);
            let should_err = if a.is_set(ArgSettings::Multiple) {
                ((ma.vals.len() as u64) % num) != 0
            } else {
                num != (ma.vals.len() as u64)
            };
            if should_err {
                debugln!("Validator::validate_arg_num_vals: Sending error WrongNumberOfValues");
                return Err(Error::wrong_number_of_values(
                    a,
                    num,
                    if a.is_set(ArgSettings::Multiple) {
                        ma.vals.len() % num as usize
                    } else {
                        ma.vals.len()
                    },
                    if ma.vals.len() == 1
                        || (a.is_set(ArgSettings::Multiple) && (ma.vals.len() % num as usize) == 1)
                    {
                        "as"
                    } else {
                        "ere"
                    },
                    &*usage::create_error_usage(self.0, matcher, None),
                    self.0.color(),
                ));
            }
        }
        if let Some(num) = a.max_vals() {
            debugln!("Validator::validate_arg_num_vals: max_vals set...{}", num);
            if (ma.vals.len() as u64) > num {
                debugln!("Validator::validate_arg_num_vals: Sending error TooManyValues");
                return Err(Error::too_many_values(
                    ma.vals
                        .iter()
                        .last()
                        .expect(INTERNAL_ERROR_MSG)
                        .to_str()
                        .expect(INVALID_UTF8),
                    a,
                    &*usage::create_error_usage(self.0, matcher, None),
                    self.0.color(),
                ));
            }
        }
        let min_vals_zero = if let Some(num) = a.min_vals() {
            debugln!("Validator::validate_arg_num_vals: min_vals set: {}", num);
            if (ma.vals.len() as u64) < num && num != 0 {
                debugln!("Validator::validate_arg_num_vals: Sending error TooFewValues");
                return Err(Error::too_few_values(
                    a,
                    num,
                    ma.vals.len(),
                    &*usage::create_error_usage(self.0, matcher, None),
                    self.0.color(),
                ));
            }
            num == 0
        } else {
            false
        };
        // Issue 665 (https://github.com/clap-rs/clap/issues/665)
        // Issue 1105 (https://github.com/clap-rs/clap/issues/1105)
        if a.takes_value() && !min_vals_zero && ma.vals.is_empty() {
            return Err(Error::empty_value(
                a,
                &*usage::create_error_usage(self.0, matcher, None),
                self.0.color(),
            ));
        }
        Ok(())
    }

    fn validate_arg_requires<A>(
        &self,
        a: &A,
        ma: &MatchedArg,
        matcher: &ArgMatcher,
    ) -> ClapResult<()>
    where
        A: AnyArg<'a, 'b> + Display,
    {
        debugln!("Validator::validate_arg_requires:{};", a.name());
        if let Some(a_reqs) = a.requires() {
            for &(val, name) in a_reqs.iter().filter(|&&(val, _)| val.is_some()) {
                let missing_req =
                    |v| v == val.expect(INTERNAL_ERROR_MSG) && !matcher.contains(name);
                if ma.vals.iter().any(missing_req) {
                    return self.missing_required_error(matcher, None);
                }
            }
            for &(_, name) in a_reqs.iter().filter(|&&(val, _)| val.is_none()) {
                if !matcher.contains(name) {
                    return self.missing_required_error(matcher, Some(name));
                }
            }
        }
        Ok(())
    }

    fn validate_required(&mut self, matcher: &ArgMatcher) -> ClapResult<()> {
        debugln!(
            "Validator::validate_required: required={:?};",
            self.0.required
        );

        let mut should_err = false;
        let mut to_rem = Vec::new();
        for name in &self.0.required {
            debugln!("Validator::validate_required:iter:{}:", name);
            if matcher.contains(name) {
                continue;
            }
            if to_rem.contains(name) {
                continue;
            } else if let Some(a) = find_any_by_name!(self.0, *name) {
                if self.is_missing_required_ok(a, matcher) {
                    to_rem.push(a.name());
                    if let Some(reqs) = a.requires() {
                        for r in reqs
                            .iter()
                            .filter(|&&(val, _)| val.is_none())
                            .map(|&(_, name)| name)
                        {
                            to_rem.push(r);
                        }
                    }
                    continue;
                }
            }
            should_err = true;
            break;
        }
        if should_err {
            for r in &to_rem {
                'inner: for i in (0..self.0.required.len()).rev() {
                    if &self.0.required[i] == r {
                        self.0.required.swap_remove(i);
                        break 'inner;
                    }
                }
            }
            return self.missing_required_error(matcher, None);
        }

        // Validate the conditionally required args
        for &(a, v, r) in &self.0.r_ifs {
            if let Some(ma) = matcher.get(a) {
                if matcher.get(r).is_none() && ma.vals.iter().any(|val| val == v) {
                    return self.missing_required_error(matcher, Some(r));
                }
            }
        }
        Ok(())
    }

    fn validate_arg_conflicts(&self, a: &AnyArg, matcher: &ArgMatcher) -> Option<bool> {
        debugln!("Validator::validate_arg_conflicts: a={:?};", a.name());
        a.blacklist().map(|bl| {
            bl.iter().any(|conf| {
                matcher.contains(conf)
                    || self
                        .0
                        .groups
                        .iter()
                        .find(|g| &g.name == conf)
                        .map_or(false, |g| g.args.iter().any(|arg| matcher.contains(arg)))
            })
        })
    }

    fn validate_required_unless(&self, a: &AnyArg, matcher: &ArgMatcher) -> Option<bool> {
        debugln!("Validator::validate_required_unless: a={:?};", a.name());
        macro_rules! check {
            ($how:ident, $_self:expr, $a:ident, $m:ident) => {{
                $a.required_unless().map(|ru| {
                    ru.iter().$how(|n| {
                        $m.contains(n) || {
                            if let Some(grp) = $_self.groups.iter().find(|g| &g.name == n) {
                                grp.args.iter().any(|arg| $m.contains(arg))
                            } else {
                                false
                            }
                        }
                    })
                })
            }};
        }
        if a.is_set(ArgSettings::RequiredUnlessAll) {
            check!(all, self.0, a, matcher)
        } else {
            check!(any, self.0, a, matcher)
        }
    }

    fn missing_required_error(&self, matcher: &ArgMatcher, extra: Option<&str>) -> ClapResult<()> {
        debugln!("Validator::missing_required_error: extra={:?}", extra);
        let c = Colorizer::new(ColorizerOption {
            use_stderr: true,
            when: self.0.color(),
        });
        let mut reqs = self.0.required.iter().map(|&r| &*r).collect::<Vec<_>>();
        if let Some(r) = extra {
            reqs.push(r);
        }
        reqs.retain(|n| !matcher.contains(n));
        reqs.dedup();
        debugln!("Validator::missing_required_error: reqs={:#?}", reqs);
        let req_args =
            usage::get_required_usage_from(self.0, &reqs[..], Some(matcher), extra, true)
                .iter()
                .fold(String::new(), |acc, s| {
                    acc + &format!("\n    {}", c.error(s))[..]
                });
        debugln!(
            "Validator::missing_required_error: req_args={:#?}",
            req_args
        );
        Err(Error::missing_required_argument(
            &*req_args,
            &*usage::create_error_usage(self.0, matcher, extra),
            self.0.color(),
        ))
    }

    #[inline]
    fn is_missing_required_ok(&self, a: &AnyArg, matcher: &ArgMatcher) -> bool {
        debugln!("Validator::is_missing_required_ok: a={}", a.name());
        self.validate_arg_conflicts(a, matcher).unwrap_or(false)
            || self.validate_required_unless(a, matcher).unwrap_or(false)
    }
}
