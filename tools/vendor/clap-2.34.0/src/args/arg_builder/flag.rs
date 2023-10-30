// Std
use std::{
    convert::From,
    ffi::{OsStr, OsString},
    fmt::{Display, Formatter, Result},
    mem,
    rc::Rc,
    result::Result as StdResult,
};

// Internal
use crate::{
    args::{AnyArg, Arg, ArgSettings, Base, DispOrder, Switched},
    map::{self, VecMap},
};

#[derive(Default, Clone, Debug)]
#[doc(hidden)]
pub struct FlagBuilder<'n, 'e>
where
    'n: 'e,
{
    pub b: Base<'n, 'e>,
    pub s: Switched<'e>,
}

impl<'n, 'e> FlagBuilder<'n, 'e> {
    pub fn new(name: &'n str) -> Self {
        FlagBuilder {
            b: Base::new(name),
            ..Default::default()
        }
    }
}

impl<'a, 'b, 'z> From<&'z Arg<'a, 'b>> for FlagBuilder<'a, 'b> {
    fn from(a: &'z Arg<'a, 'b>) -> Self {
        FlagBuilder {
            b: Base::from(a),
            s: Switched::from(a),
        }
    }
}

impl<'a, 'b> From<Arg<'a, 'b>> for FlagBuilder<'a, 'b> {
    fn from(mut a: Arg<'a, 'b>) -> Self {
        FlagBuilder {
            b: mem::take(&mut a.b),
            s: mem::take(&mut a.s),
        }
    }
}

impl<'n, 'e> Display for FlagBuilder<'n, 'e> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        if let Some(l) = self.s.long {
            write!(f, "--{}", l)?;
        } else {
            write!(f, "-{}", self.s.short.unwrap())?;
        }

        Ok(())
    }
}

impl<'n, 'e> AnyArg<'n, 'e> for FlagBuilder<'n, 'e> {
    fn name(&self) -> &'n str {
        self.b.name
    }
    fn overrides(&self) -> Option<&[&'e str]> {
        self.b.overrides.as_ref().map(|o| &o[..])
    }
    fn requires(&self) -> Option<&[(Option<&'e str>, &'n str)]> {
        self.b.requires.as_ref().map(|o| &o[..])
    }
    fn blacklist(&self) -> Option<&[&'e str]> {
        self.b.blacklist.as_ref().map(|o| &o[..])
    }
    fn required_unless(&self) -> Option<&[&'e str]> {
        self.b.r_unless.as_ref().map(|o| &o[..])
    }
    fn is_set(&self, s: ArgSettings) -> bool {
        self.b.settings.is_set(s)
    }
    fn has_switch(&self) -> bool {
        true
    }
    fn takes_value(&self) -> bool {
        false
    }
    fn set(&mut self, s: ArgSettings) {
        self.b.settings.set(s)
    }
    fn max_vals(&self) -> Option<u64> {
        None
    }
    fn val_names(&self) -> Option<&VecMap<&'e str>> {
        None
    }
    fn num_vals(&self) -> Option<u64> {
        None
    }
    fn possible_vals(&self) -> Option<&[&'e str]> {
        None
    }
    #[cfg_attr(feature = "cargo-clippy", allow(clippy::type_complexity))]
    fn validator(&self) -> Option<&Rc<Fn(String) -> StdResult<(), String>>> {
        None
    }
    #[cfg_attr(feature = "cargo-clippy", allow(clippy::type_complexity))]
    fn validator_os(&self) -> Option<&Rc<Fn(&OsStr) -> StdResult<(), OsString>>> {
        None
    }
    fn min_vals(&self) -> Option<u64> {
        None
    }
    fn short(&self) -> Option<char> {
        self.s.short
    }
    fn long(&self) -> Option<&'e str> {
        self.s.long
    }
    fn val_delim(&self) -> Option<char> {
        None
    }
    fn help(&self) -> Option<&'e str> {
        self.b.help
    }
    fn long_help(&self) -> Option<&'e str> {
        self.b.long_help
    }
    fn val_terminator(&self) -> Option<&'e str> {
        None
    }
    fn default_val(&self) -> Option<&'e OsStr> {
        None
    }
    fn default_vals_ifs(&self) -> Option<map::Values<(&'n str, Option<&'e OsStr>, &'e OsStr)>> {
        None
    }
    fn env<'s>(&'s self) -> Option<(&'n OsStr, Option<&'s OsString>)> {
        None
    }
    fn longest_filter(&self) -> bool {
        self.s.long.is_some()
    }
    fn aliases(&self) -> Option<Vec<&'e str>> {
        if let Some(ref aliases) = self.s.aliases {
            let vis_aliases: Vec<_> = aliases
                .iter()
                .filter_map(|&(n, v)| if v { Some(n) } else { None })
                .collect();
            if vis_aliases.is_empty() {
                None
            } else {
                Some(vis_aliases)
            }
        } else {
            None
        }
    }
}

impl<'n, 'e> DispOrder for FlagBuilder<'n, 'e> {
    fn disp_ord(&self) -> usize {
        self.s.disp_ord
    }
}

impl<'n, 'e> PartialEq for FlagBuilder<'n, 'e> {
    fn eq(&self, other: &FlagBuilder<'n, 'e>) -> bool {
        self.b == other.b
    }
}

#[cfg(test)]
mod test {
    use super::FlagBuilder;
    use crate::args::settings::ArgSettings;

    #[test]
    fn flagbuilder_display() {
        let mut f = FlagBuilder::new("flg");
        f.b.settings.set(ArgSettings::Multiple);
        f.s.long = Some("flag");

        assert_eq!(&*format!("{}", f), "--flag");

        let mut f2 = FlagBuilder::new("flg");
        f2.s.short = Some('f');

        assert_eq!(&*format!("{}", f2), "-f");
    }

    #[test]
    fn flagbuilder_display_single_alias() {
        let mut f = FlagBuilder::new("flg");
        f.s.long = Some("flag");
        f.s.aliases = Some(vec![("als", true)]);

        assert_eq!(&*format!("{}", f), "--flag");
    }

    #[test]
    fn flagbuilder_display_multiple_aliases() {
        let mut f = FlagBuilder::new("flg");
        f.s.short = Some('f');
        f.s.aliases = Some(vec![
            ("alias_not_visible", false),
            ("f2", true),
            ("f3", true),
            ("f4", true),
        ]);
        assert_eq!(&*format!("{}", f), "-f");
    }
}
