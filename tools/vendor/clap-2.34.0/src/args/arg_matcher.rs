// Std
use std::{
    collections::{
        hash_map::{Entry, Iter},
        HashMap,
    },
    ffi::OsStr,
    mem,
    ops::Deref,
};

// Internal
use crate::args::{settings::ArgSettings, AnyArg, ArgMatches, MatchedArg, SubCommand};

#[doc(hidden)]
#[allow(missing_debug_implementations)]
pub struct ArgMatcher<'a>(pub ArgMatches<'a>);

impl<'a> Default for ArgMatcher<'a> {
    fn default() -> Self {
        ArgMatcher(ArgMatches::default())
    }
}

impl<'a> ArgMatcher<'a> {
    pub fn new() -> Self {
        ArgMatcher::default()
    }

    pub fn process_arg_overrides<'b>(
        &mut self,
        a: Option<&AnyArg<'a, 'b>>,
        overrides: &mut Vec<(&'b str, &'a str)>,
        required: &mut Vec<&'a str>,
        check_all: bool,
    ) {
        debugln!(
            "ArgMatcher::process_arg_overrides:{:?};",
            a.map_or(None, |a| Some(a.name()))
        );
        if let Some(aa) = a {
            let mut self_done = false;
            if let Some(a_overrides) = aa.overrides() {
                for overr in a_overrides {
                    debugln!("ArgMatcher::process_arg_overrides:iter:{};", overr);
                    if overr == &aa.name() {
                        self_done = true;
                        self.handle_self_overrides(a);
                    } else if self.is_present(overr) {
                        debugln!(
                            "ArgMatcher::process_arg_overrides:iter:{}: removing from matches;",
                            overr
                        );
                        self.remove(overr);
                        for i in (0..required.len()).rev() {
                            if &required[i] == overr {
                                debugln!(
                                    "ArgMatcher::process_arg_overrides:iter:{}: removing required;",
                                    overr
                                );
                                required.swap_remove(i);
                                break;
                            }
                        }
                        overrides.push((overr, aa.name()));
                    } else {
                        overrides.push((overr, aa.name()));
                    }
                }
            }
            if check_all && !self_done {
                self.handle_self_overrides(a);
            }
        }
    }

    pub fn handle_self_overrides<'b>(&mut self, a: Option<&AnyArg<'a, 'b>>) {
        debugln!(
            "ArgMatcher::handle_self_overrides:{:?};",
            a.map_or(None, |a| Some(a.name()))
        );
        if let Some(aa) = a {
            if !aa.has_switch() || aa.is_set(ArgSettings::Multiple) {
                // positional args can't override self or else we would never advance to the next

                // Also flags with --multiple set are ignored otherwise we could never have more
                // than one
                return;
            }
            if let Some(ma) = self.get_mut(aa.name()) {
                if ma.vals.len() > 1 {
                    // swap_remove(0) would be O(1) but does not preserve order, which
                    // we need
                    ma.vals.remove(0);
                    ma.occurs = 1;
                } else if !aa.takes_value() && ma.occurs > 1 {
                    ma.occurs = 1;
                }
            }
        }
    }

    pub fn is_present(&self, name: &str) -> bool {
        self.0.is_present(name)
    }

    pub fn propagate_globals(&mut self, global_arg_vec: &[&'a str]) {
        debugln!(
            "ArgMatcher::get_global_values: global_arg_vec={:?}",
            global_arg_vec
        );
        let mut vals_map = HashMap::new();
        self.fill_in_global_values(global_arg_vec, &mut vals_map);
    }

    fn fill_in_global_values(
        &mut self,
        global_arg_vec: &[&'a str],
        vals_map: &mut HashMap<&'a str, MatchedArg>,
    ) {
        for global_arg in global_arg_vec {
            if let Some(ma) = self.get(global_arg) {
                // We have to check if the parent's global arg wasn't used but still exists
                // such as from a default value.
                //
                // For example, `myprog subcommand --global-arg=value` where --global-arg defines
                // a default value of `other` myprog would have an existing MatchedArg for
                // --global-arg where the value is `other`, however the occurs will be 0.
                let to_update = if let Some(parent_ma) = vals_map.get(global_arg) {
                    if parent_ma.occurs > 0 && ma.occurs == 0 {
                        parent_ma.clone()
                    } else {
                        ma.clone()
                    }
                } else {
                    ma.clone()
                };
                vals_map.insert(global_arg, to_update);
            }
        }
        if let Some(ref mut sc) = self.0.subcommand {
            let mut am = ArgMatcher(mem::replace(&mut sc.matches, ArgMatches::new()));
            am.fill_in_global_values(global_arg_vec, vals_map);
            mem::swap(&mut am.0, &mut sc.matches);
        }

        for (name, matched_arg) in vals_map.iter_mut() {
            self.0.args.insert(name, matched_arg.clone());
        }
    }

    pub fn get_mut(&mut self, arg: &str) -> Option<&mut MatchedArg> {
        self.0.args.get_mut(arg)
    }

    pub fn get(&self, arg: &str) -> Option<&MatchedArg> {
        self.0.args.get(arg)
    }

    pub fn remove(&mut self, arg: &str) {
        self.0.args.remove(arg);
    }

    pub fn remove_all(&mut self, args: &[&str]) {
        for &arg in args {
            self.0.args.remove(arg);
        }
    }

    pub fn insert(&mut self, name: &'a str) {
        self.0.args.insert(name, MatchedArg::new());
    }

    pub fn contains(&self, arg: &str) -> bool {
        self.0.args.contains_key(arg)
    }

    pub fn is_empty(&self) -> bool {
        self.0.args.is_empty()
    }

    pub fn usage(&mut self, usage: String) {
        self.0.usage = Some(usage);
    }

    pub fn arg_names(&'a self) -> Vec<&'a str> {
        self.0.args.keys().map(Deref::deref).collect()
    }

    pub fn entry(&mut self, arg: &'a str) -> Entry<&'a str, MatchedArg> {
        self.0.args.entry(arg)
    }

    pub fn subcommand(&mut self, sc: SubCommand<'a>) {
        self.0.subcommand = Some(Box::new(sc));
    }

    pub fn subcommand_name(&self) -> Option<&str> {
        self.0.subcommand_name()
    }

    pub fn iter(&self) -> Iter<&str, MatchedArg> {
        self.0.args.iter()
    }

    pub fn inc_occurrence_of(&mut self, arg: &'a str) {
        debugln!("ArgMatcher::inc_occurrence_of: arg={}", arg);
        if let Some(a) = self.get_mut(arg) {
            a.occurs += 1;
            return;
        }
        debugln!("ArgMatcher::inc_occurrence_of: first instance");
        self.insert(arg);
    }

    pub fn inc_occurrences_of(&mut self, args: &[&'a str]) {
        debugln!("ArgMatcher::inc_occurrences_of: args={:?}", args);
        for arg in args {
            self.inc_occurrence_of(arg);
        }
    }

    pub fn add_val_to(&mut self, arg: &'a str, val: &OsStr) {
        let ma = self.entry(arg).or_insert(MatchedArg {
            occurs: 0,
            indices: Vec::with_capacity(1),
            vals: Vec::with_capacity(1),
        });
        ma.vals.push(val.to_owned());
    }

    pub fn add_index_to(&mut self, arg: &'a str, idx: usize) {
        let ma = self.entry(arg).or_insert(MatchedArg {
            occurs: 0,
            indices: Vec::with_capacity(1),
            vals: Vec::new(),
        });
        ma.indices.push(idx);
    }

    pub fn needs_more_vals<'b, A>(&self, o: &A) -> bool
    where
        A: AnyArg<'a, 'b>,
    {
        debugln!("ArgMatcher::needs_more_vals: o={}", o.name());
        if let Some(ma) = self.get(o.name()) {
            if let Some(num) = o.num_vals() {
                debugln!("ArgMatcher::needs_more_vals: num_vals...{}", num);
                return if o.is_set(ArgSettings::Multiple) {
                    ((ma.vals.len() as u64) % num) != 0
                } else {
                    num != (ma.vals.len() as u64)
                };
            } else if let Some(num) = o.max_vals() {
                debugln!("ArgMatcher::needs_more_vals: max_vals...{}", num);
                return (ma.vals.len() as u64) <= num;
            } else if o.min_vals().is_some() {
                debugln!("ArgMatcher::needs_more_vals: min_vals...true");
                return true;
            }
            return o.is_set(ArgSettings::Multiple);
        }
        true
    }
}

// Not changing to From just to not deal with possible breaking changes on v2 since v3 is coming
// in the future anyways
#[cfg_attr(feature = "cargo-clippy", allow(clippy::from_over_into))]
impl<'a> Into<ArgMatches<'a>> for ArgMatcher<'a> {
    fn into(self) -> ArgMatches<'a> {
        self.0
    }
}
