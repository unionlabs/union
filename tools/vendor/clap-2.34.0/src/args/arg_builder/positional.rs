// Std
use std::{
    borrow::Cow,
    ffi::{OsStr, OsString},
    fmt::{Display, Formatter, Result},
    mem,
    rc::Rc,
    result::Result as StdResult,
};

// Internal
use crate::{
    args::{AnyArg, Arg, ArgSettings, Base, DispOrder, Valued},
    map::{self, VecMap},
    INTERNAL_ERROR_MSG,
};

#[allow(missing_debug_implementations)]
#[doc(hidden)]
#[derive(Clone, Default)]
pub struct PosBuilder<'n, 'e>
where
    'n: 'e,
{
    pub b: Base<'n, 'e>,
    pub v: Valued<'n, 'e>,
    pub index: u64,
}

impl<'n, 'e> PosBuilder<'n, 'e> {
    pub fn new(name: &'n str, idx: u64) -> Self {
        PosBuilder {
            b: Base::new(name),
            index: idx,
            ..Default::default()
        }
    }

    pub fn from_arg_ref(a: &Arg<'n, 'e>, idx: u64) -> Self {
        let mut pb = PosBuilder {
            b: Base::from(a),
            v: Valued::from(a),
            index: idx,
        };
        if a.v.max_vals.is_some()
            || a.v.min_vals.is_some()
            || (a.v.num_vals.is_some() && a.v.num_vals.unwrap() > 1)
        {
            pb.b.settings.set(ArgSettings::Multiple);
        }
        pb
    }

    pub fn from_arg(mut a: Arg<'n, 'e>, idx: u64) -> Self {
        if a.v.max_vals.is_some()
            || a.v.min_vals.is_some()
            || (a.v.num_vals.is_some() && a.v.num_vals.unwrap() > 1)
        {
            a.b.settings.set(ArgSettings::Multiple);
        }
        PosBuilder {
            b: mem::take(&mut a.b),
            v: mem::take(&mut a.v),
            index: idx,
        }
    }

    pub fn multiple_str(&self) -> &str {
        let mult_vals = self
            .v
            .val_names
            .as_ref()
            .map_or(true, |names| names.len() < 2);
        if self.is_set(ArgSettings::Multiple) && mult_vals {
            "..."
        } else {
            ""
        }
    }

    pub fn name_no_brackets(&self) -> Cow<str> {
        debugln!("PosBuilder::name_no_brackets;");
        let mut delim = String::new();
        delim.push(if self.is_set(ArgSettings::RequireDelimiter) {
            self.v.val_delim.expect(INTERNAL_ERROR_MSG)
        } else {
            ' '
        });
        if let Some(ref names) = self.v.val_names {
            debugln!("PosBuilder:name_no_brackets: val_names={:#?}", names);
            if names.len() > 1 {
                Cow::Owned(
                    names
                        .values()
                        .map(|n| format!("<{}>", n))
                        .collect::<Vec<_>>()
                        .join(&*delim),
                )
            } else {
                Cow::Borrowed(names.values().next().expect(INTERNAL_ERROR_MSG))
            }
        } else {
            debugln!("PosBuilder:name_no_brackets: just name");
            Cow::Borrowed(self.b.name)
        }
    }
}

impl<'n, 'e> Display for PosBuilder<'n, 'e> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let mut delim = String::new();
        delim.push(if self.is_set(ArgSettings::RequireDelimiter) {
            self.v.val_delim.expect(INTERNAL_ERROR_MSG)
        } else {
            ' '
        });
        if let Some(ref names) = self.v.val_names {
            write!(
                f,
                "{}",
                names
                    .values()
                    .map(|n| format!("<{}>", n))
                    .collect::<Vec<_>>()
                    .join(&*delim)
            )?;
        } else {
            write!(f, "<{}>", self.b.name)?;
        }
        if self.b.settings.is_set(ArgSettings::Multiple)
            && (self.v.val_names.is_none() || self.v.val_names.as_ref().unwrap().len() == 1)
        {
            write!(f, "...")?;
        }

        Ok(())
    }
}

impl<'n, 'e> AnyArg<'n, 'e> for PosBuilder<'n, 'e> {
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
    fn val_names(&self) -> Option<&VecMap<&'e str>> {
        self.v.val_names.as_ref()
    }
    fn is_set(&self, s: ArgSettings) -> bool {
        self.b.settings.is_set(s)
    }
    fn set(&mut self, s: ArgSettings) {
        self.b.settings.set(s)
    }
    fn has_switch(&self) -> bool {
        false
    }
    fn max_vals(&self) -> Option<u64> {
        self.v.max_vals
    }
    fn val_terminator(&self) -> Option<&'e str> {
        self.v.terminator
    }
    fn num_vals(&self) -> Option<u64> {
        self.v.num_vals
    }
    fn possible_vals(&self) -> Option<&[&'e str]> {
        self.v.possible_vals.as_ref().map(|o| &o[..])
    }
    #[cfg_attr(feature = "cargo-clippy", allow(clippy::type_complexity))]
    fn validator(&self) -> Option<&Rc<Fn(String) -> StdResult<(), String>>> {
        self.v.validator.as_ref()
    }
    #[cfg_attr(feature = "cargo-clippy", allow(clippy::type_complexity))]
    fn validator_os(&self) -> Option<&Rc<Fn(&OsStr) -> StdResult<(), OsString>>> {
        self.v.validator_os.as_ref()
    }
    fn min_vals(&self) -> Option<u64> {
        self.v.min_vals
    }
    fn short(&self) -> Option<char> {
        None
    }
    fn long(&self) -> Option<&'e str> {
        None
    }
    fn val_delim(&self) -> Option<char> {
        self.v.val_delim
    }
    fn takes_value(&self) -> bool {
        true
    }
    fn help(&self) -> Option<&'e str> {
        self.b.help
    }
    fn long_help(&self) -> Option<&'e str> {
        self.b.long_help
    }
    fn default_vals_ifs(&self) -> Option<map::Values<(&'n str, Option<&'e OsStr>, &'e OsStr)>> {
        self.v.default_vals_ifs.as_ref().map(|vm| vm.values())
    }
    fn default_val(&self) -> Option<&'e OsStr> {
        self.v.default_val
    }
    fn env<'s>(&'s self) -> Option<(&'n OsStr, Option<&'s OsString>)> {
        self.v
            .env
            .as_ref()
            .map(|&(key, ref value)| (key, value.as_ref()))
    }
    fn longest_filter(&self) -> bool {
        true
    }
    fn aliases(&self) -> Option<Vec<&'e str>> {
        None
    }
}

impl<'n, 'e> DispOrder for PosBuilder<'n, 'e> {
    fn disp_ord(&self) -> usize {
        self.index as usize
    }
}

impl<'n, 'e> PartialEq for PosBuilder<'n, 'e> {
    fn eq(&self, other: &PosBuilder<'n, 'e>) -> bool {
        self.b == other.b
    }
}

#[cfg(test)]
mod test {
    use super::PosBuilder;
    use crate::{args::settings::ArgSettings, map::VecMap};

    #[test]
    fn display_mult() {
        let mut p = PosBuilder::new("pos", 1);
        p.b.settings.set(ArgSettings::Multiple);

        assert_eq!(&*format!("{}", p), "<pos>...");
    }

    #[test]
    fn display_required() {
        let mut p2 = PosBuilder::new("pos", 1);
        p2.b.settings.set(ArgSettings::Required);

        assert_eq!(&*format!("{}", p2), "<pos>");
    }

    #[test]
    fn display_val_names() {
        let mut p2 = PosBuilder::new("pos", 1);
        let mut vm = VecMap::new();
        vm.insert(0, "file1");
        vm.insert(1, "file2");
        p2.v.val_names = Some(vm);

        assert_eq!(&*format!("{}", p2), "<file1> <file2>");
    }

    #[test]
    fn display_val_names_req() {
        let mut p2 = PosBuilder::new("pos", 1);
        p2.b.settings.set(ArgSettings::Required);
        let mut vm = VecMap::new();
        vm.insert(0, "file1");
        vm.insert(1, "file2");
        p2.v.val_names = Some(vm);

        assert_eq!(&*format!("{}", p2), "<file1> <file2>");
    }
}
