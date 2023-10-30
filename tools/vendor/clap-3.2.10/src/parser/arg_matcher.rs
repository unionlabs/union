// Std
use std::collections::HashMap;
use std::ffi::OsString;
use std::mem;
use std::ops::Deref;

// Internal
use crate::builder::{Arg, ArgPredicate, Command};
use crate::parser::AnyValue;
use crate::parser::Identifier;
use crate::parser::PendingArg;
use crate::parser::{ArgMatches, MatchedArg, SubCommand, ValueSource};
use crate::util::Id;
use crate::INTERNAL_ERROR_MSG;

#[derive(Debug, Default)]
pub(crate) struct ArgMatcher {
    matches: ArgMatches,
    pending: Option<PendingArg>,
}

impl ArgMatcher {
    pub(crate) fn new(_cmd: &Command) -> Self {
        ArgMatcher {
            matches: ArgMatches {
                #[cfg(debug_assertions)]
                valid_args: {
                    let args = _cmd.get_arguments().map(|a| a.id.clone());
                    let groups = _cmd.get_groups().map(|g| g.id.clone());
                    args.chain(groups).collect()
                },
                #[cfg(debug_assertions)]
                valid_subcommands: _cmd.get_subcommands().map(|sc| sc.get_id()).collect(),
                // HACK: Allow an external subcommand's ArgMatches be a stand-in for any ArgMatches
                // since users can't detect it and avoid the asserts.
                //
                // See clap-rs/clap#3263
                #[cfg(debug_assertions)]
                #[cfg(not(feature = "unstable-v4"))]
                disable_asserts: _cmd.is_allow_external_subcommands_set(),
                #[cfg(debug_assertions)]
                #[cfg(feature = "unstable-v4")]
                disable_asserts: false,
                ..Default::default()
            },
            pending: None,
        }
    }

    pub(crate) fn into_inner(self) -> ArgMatches {
        self.matches
    }

    pub(crate) fn propagate_globals(&mut self, global_arg_vec: &[Id]) {
        debug!(
            "ArgMatcher::get_global_values: global_arg_vec={:?}",
            global_arg_vec
        );
        let mut vals_map = HashMap::new();
        self.fill_in_global_values(global_arg_vec, &mut vals_map);
    }

    fn fill_in_global_values(
        &mut self,
        global_arg_vec: &[Id],
        vals_map: &mut HashMap<Id, MatchedArg>,
    ) {
        for global_arg in global_arg_vec {
            if let Some(ma) = self.get(global_arg) {
                // We have to check if the parent's global arg wasn't used but still exists
                // such as from a default value.
                //
                // For example, `myprog subcommand --global-arg=value` where `--global-arg` defines
                // a default value of `other` myprog would have an existing MatchedArg for
                // `--global-arg` where the value is `other`
                let to_update = if let Some(parent_ma) = vals_map.get(global_arg) {
                    if parent_ma.source() > ma.source() {
                        parent_ma
                    } else {
                        ma
                    }
                } else {
                    ma
                }
                .clone();
                vals_map.insert(global_arg.clone(), to_update);
            }
        }
        if let Some(ref mut sc) = self.matches.subcommand {
            let mut am = ArgMatcher {
                matches: mem::take(&mut sc.matches),
                pending: None,
            };
            am.fill_in_global_values(global_arg_vec, vals_map);
            mem::swap(&mut am.matches, &mut sc.matches);
        }

        for (name, matched_arg) in vals_map.iter_mut() {
            self.matches.args.insert(name.clone(), matched_arg.clone());
        }
    }

    pub(crate) fn get(&self, arg: &Id) -> Option<&MatchedArg> {
        self.matches.args.get(arg)
    }

    pub(crate) fn get_mut(&mut self, arg: &Id) -> Option<&mut MatchedArg> {
        self.matches.args.get_mut(arg)
    }

    pub(crate) fn remove(&mut self, arg: &Id) {
        self.matches.args.swap_remove(arg);
    }

    pub(crate) fn contains(&self, arg: &Id) -> bool {
        self.matches.args.contains_key(arg)
    }

    pub(crate) fn arg_ids(&self) -> indexmap::map::Keys<Id, MatchedArg> {
        self.matches.args.keys()
    }

    pub(crate) fn entry(&mut self, arg: &Id) -> indexmap::map::Entry<Id, MatchedArg> {
        self.matches.args.entry(arg.clone())
    }

    pub(crate) fn subcommand(&mut self, sc: SubCommand) {
        self.matches.subcommand = Some(Box::new(sc));
    }

    pub(crate) fn subcommand_name(&self) -> Option<&str> {
        self.matches.subcommand_name()
    }

    pub(crate) fn iter(&self) -> indexmap::map::Iter<Id, MatchedArg> {
        self.matches.args.iter()
    }

    pub(crate) fn check_explicit<'a>(&self, arg: &Id, predicate: ArgPredicate<'a>) -> bool {
        self.get(arg).map_or(false, |a| a.check_explicit(predicate))
    }

    pub(crate) fn start_custom_arg(&mut self, arg: &Arg, source: ValueSource) {
        let id = &arg.id;
        debug!(
            "ArgMatcher::start_custom_arg: id={:?}, source={:?}",
            id, source
        );
        let ma = self.entry(id).or_insert(MatchedArg::new_arg(arg));
        debug_assert_eq!(ma.type_id(), Some(arg.get_value_parser().type_id()));
        ma.set_source(source);
        ma.new_val_group();
    }

    pub(crate) fn start_custom_group(&mut self, id: &Id, source: ValueSource) {
        debug!(
            "ArgMatcher::start_custom_arg: id={:?}, source={:?}",
            id, source
        );
        let ma = self.entry(id).or_insert(MatchedArg::new_group());
        debug_assert_eq!(ma.type_id(), None);
        ma.set_source(source);
        ma.new_val_group();
    }

    pub(crate) fn start_occurrence_of_arg(&mut self, arg: &Arg) {
        let id = &arg.id;
        debug!("ArgMatcher::start_occurrence_of_arg: id={:?}", id);
        let ma = self.entry(id).or_insert(MatchedArg::new_arg(arg));
        debug_assert_eq!(ma.type_id(), Some(arg.get_value_parser().type_id()));
        ma.set_source(ValueSource::CommandLine);
        #[allow(deprecated)]
        ma.inc_occurrences();
        ma.new_val_group();
    }

    pub(crate) fn start_occurrence_of_group(&mut self, id: &Id) {
        debug!("ArgMatcher::start_occurrence_of_group: id={:?}", id);
        let ma = self.entry(id).or_insert(MatchedArg::new_group());
        debug_assert_eq!(ma.type_id(), None);
        ma.set_source(ValueSource::CommandLine);
        #[allow(deprecated)]
        ma.inc_occurrences();
        ma.new_val_group();
    }

    pub(crate) fn start_occurrence_of_external(&mut self, cmd: &crate::Command) {
        let id = &Id::empty_hash();
        debug!("ArgMatcher::start_occurrence_of_external: id={:?}", id,);
        let ma = self.entry(id).or_insert(MatchedArg::new_external(cmd));
        debug_assert_eq!(
            ma.type_id(),
            Some(
                cmd.get_external_subcommand_value_parser()
                    .expect(INTERNAL_ERROR_MSG)
                    .type_id()
            )
        );
        ma.set_source(ValueSource::CommandLine);
        #[allow(deprecated)]
        ma.inc_occurrences();
        ma.new_val_group();
    }

    pub(crate) fn add_val_to(&mut self, arg: &Id, val: AnyValue, raw_val: OsString) {
        let ma = self.get_mut(arg).expect(INTERNAL_ERROR_MSG);
        ma.append_val(val, raw_val);
    }

    pub(crate) fn add_index_to(&mut self, arg: &Id, idx: usize) {
        let ma = self.get_mut(arg).expect(INTERNAL_ERROR_MSG);
        ma.push_index(idx);
    }

    pub(crate) fn needs_more_vals(&self, o: &Arg) -> bool {
        let num_resolved = self.get(&o.id).map(|ma| ma.num_vals()).unwrap_or(0);
        let num_pending = self
            .pending
            .as_ref()
            .and_then(|p| (p.id == o.id).then(|| p.raw_vals.len()))
            .unwrap_or(0);
        let current_num = num_resolved + num_pending;
        debug!(
            "ArgMatcher::needs_more_vals: o={}, resolved={}, pending={}",
            o.name, num_resolved, num_pending
        );
        if current_num == 0 {
            true
        } else if let Some(num) = o.num_vals {
            debug!("ArgMatcher::needs_more_vals: num_vals...{}", num);
            #[allow(deprecated)]
            if o.is_multiple_occurrences_set() {
                (current_num % num) != 0
            } else {
                num != current_num
            }
        } else if let Some(num) = o.max_vals {
            debug!("ArgMatcher::needs_more_vals: max_vals...{}", num);
            current_num < num
        } else if o.min_vals.is_some() {
            debug!("ArgMatcher::needs_more_vals: min_vals...true");
            true
        } else {
            o.is_multiple_values_set()
        }
    }

    pub(crate) fn pending_arg_id(&self) -> Option<&Id> {
        self.pending.as_ref().map(|p| &p.id)
    }

    pub(crate) fn pending_values_mut(
        &mut self,
        id: &Id,
        ident: Option<Identifier>,
    ) -> &mut Vec<OsString> {
        let pending = self.pending.get_or_insert_with(|| PendingArg {
            id: id.clone(),
            ident,
            raw_vals: Default::default(),
        });
        debug_assert_eq!(pending.id, *id, "{}", INTERNAL_ERROR_MSG);
        if ident.is_some() {
            debug_assert_eq!(pending.ident, ident, "{}", INTERNAL_ERROR_MSG);
        }
        &mut pending.raw_vals
    }

    pub(crate) fn take_pending(&mut self) -> Option<PendingArg> {
        self.pending.take()
    }
}

impl Deref for ArgMatcher {
    type Target = ArgMatches;

    fn deref(&self) -> &Self::Target {
        &self.matches
    }
}
