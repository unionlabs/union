// Std
use std::{
    ffi::{OsStr, OsString},
    fmt as std_fmt,
    rc::Rc,
};

// Internal
use crate::{
    args::settings::ArgSettings,
    map::{self, VecMap},
    INTERNAL_ERROR_MSG,
};

#[doc(hidden)]
pub trait AnyArg<'n, 'e>: std_fmt::Display {
    fn name(&self) -> &'n str;
    fn overrides(&self) -> Option<&[&'e str]>;
    fn aliases(&self) -> Option<Vec<&'e str>>;
    fn requires(&self) -> Option<&[(Option<&'e str>, &'n str)]>;
    fn blacklist(&self) -> Option<&[&'e str]>;
    fn required_unless(&self) -> Option<&[&'e str]>;
    fn is_set(&self, setting: ArgSettings) -> bool;
    fn set(&mut self, setting: ArgSettings);
    fn has_switch(&self) -> bool;
    fn max_vals(&self) -> Option<u64>;
    fn min_vals(&self) -> Option<u64>;
    fn num_vals(&self) -> Option<u64>;
    fn possible_vals(&self) -> Option<&[&'e str]>;
    #[cfg_attr(feature = "cargo-clippy", allow(clippy::type_complexity))]
    fn validator(&self) -> Option<&Rc<Fn(String) -> Result<(), String>>>;
    #[cfg_attr(feature = "cargo-clippy", allow(clippy::type_complexity))]
    fn validator_os(&self) -> Option<&Rc<Fn(&OsStr) -> Result<(), OsString>>>;
    fn short(&self) -> Option<char>;
    fn long(&self) -> Option<&'e str>;
    fn val_delim(&self) -> Option<char>;
    fn takes_value(&self) -> bool;
    fn val_names(&self) -> Option<&VecMap<&'e str>>;
    fn help(&self) -> Option<&'e str>;
    fn long_help(&self) -> Option<&'e str>;
    fn default_val(&self) -> Option<&'e OsStr>;
    fn default_vals_ifs(&self) -> Option<map::Values<(&'n str, Option<&'e OsStr>, &'e OsStr)>>;
    fn env<'s>(&'s self) -> Option<(&'n OsStr, Option<&'s OsString>)>;
    fn longest_filter(&self) -> bool;
    fn val_terminator(&self) -> Option<&'e str>;
}

pub trait DispOrder {
    fn disp_ord(&self) -> usize;
}

impl<'n, 'e, 'z, T: ?Sized> AnyArg<'n, 'e> for &'z T
where
    T: AnyArg<'n, 'e> + 'z,
{
    fn name(&self) -> &'n str {
        (*self).name()
    }
    fn overrides(&self) -> Option<&[&'e str]> {
        (*self).overrides()
    }
    fn aliases(&self) -> Option<Vec<&'e str>> {
        (*self).aliases()
    }
    fn requires(&self) -> Option<&[(Option<&'e str>, &'n str)]> {
        (*self).requires()
    }
    fn blacklist(&self) -> Option<&[&'e str]> {
        (*self).blacklist()
    }
    fn required_unless(&self) -> Option<&[&'e str]> {
        (*self).required_unless()
    }
    fn is_set(&self, a: ArgSettings) -> bool {
        (*self).is_set(a)
    }
    fn set(&mut self, _: ArgSettings) {
        panic!("{}", INTERNAL_ERROR_MSG)
    }
    fn has_switch(&self) -> bool {
        (*self).has_switch()
    }
    fn max_vals(&self) -> Option<u64> {
        (*self).max_vals()
    }
    fn min_vals(&self) -> Option<u64> {
        (*self).min_vals()
    }
    fn num_vals(&self) -> Option<u64> {
        (*self).num_vals()
    }
    fn possible_vals(&self) -> Option<&[&'e str]> {
        (*self).possible_vals()
    }
    #[cfg_attr(feature = "cargo-clippy", allow(clippy::type_complexity))]
    fn validator(&self) -> Option<&Rc<Fn(String) -> Result<(), String>>> {
        (*self).validator()
    }
    #[cfg_attr(feature = "cargo-clippy", allow(clippy::type_complexity))]
    fn validator_os(&self) -> Option<&Rc<Fn(&OsStr) -> Result<(), OsString>>> {
        (*self).validator_os()
    }
    fn short(&self) -> Option<char> {
        (*self).short()
    }
    fn long(&self) -> Option<&'e str> {
        (*self).long()
    }
    fn val_delim(&self) -> Option<char> {
        (*self).val_delim()
    }
    fn takes_value(&self) -> bool {
        (*self).takes_value()
    }
    fn val_names(&self) -> Option<&VecMap<&'e str>> {
        (*self).val_names()
    }
    fn help(&self) -> Option<&'e str> {
        (*self).help()
    }
    fn long_help(&self) -> Option<&'e str> {
        (*self).long_help()
    }
    fn default_val(&self) -> Option<&'e OsStr> {
        (*self).default_val()
    }
    fn default_vals_ifs(&self) -> Option<map::Values<(&'n str, Option<&'e OsStr>, &'e OsStr)>> {
        (*self).default_vals_ifs()
    }
    fn env<'s>(&'s self) -> Option<(&'n OsStr, Option<&'s OsString>)> {
        (*self).env()
    }
    fn longest_filter(&self) -> bool {
        (*self).longest_filter()
    }
    fn val_terminator(&self) -> Option<&'e str> {
        (*self).val_terminator()
    }
}
