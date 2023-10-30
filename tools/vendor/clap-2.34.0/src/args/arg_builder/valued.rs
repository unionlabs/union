use std::{
    ffi::{OsStr, OsString},
    rc::Rc,
};

use crate::{map::VecMap, Arg};

#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct Valued<'a, 'b>
where
    'a: 'b,
{
    pub possible_vals: Option<Vec<&'b str>>,
    pub val_names: Option<VecMap<&'b str>>,
    pub num_vals: Option<u64>,
    pub max_vals: Option<u64>,
    pub min_vals: Option<u64>,
    #[cfg_attr(feature = "cargo-clippy", allow(clippy::type_complexity))]
    pub validator: Option<Rc<Fn(String) -> Result<(), String>>>,
    #[cfg_attr(feature = "cargo-clippy", allow(clippy::type_complexity))]
    pub validator_os: Option<Rc<Fn(&OsStr) -> Result<(), OsString>>>,
    pub val_delim: Option<char>,
    pub default_val: Option<&'b OsStr>,
    #[cfg_attr(feature = "cargo-clippy", allow(clippy::type_complexity))]
    pub default_vals_ifs: Option<VecMap<(&'a str, Option<&'b OsStr>, &'b OsStr)>>,
    pub env: Option<(&'a OsStr, Option<OsString>)>,
    pub terminator: Option<&'b str>,
}

impl<'n, 'e> Default for Valued<'n, 'e> {
    fn default() -> Self {
        Valued {
            possible_vals: None,
            num_vals: None,
            min_vals: None,
            max_vals: None,
            val_names: None,
            validator: None,
            validator_os: None,
            val_delim: None,
            default_val: None,
            default_vals_ifs: None,
            env: None,
            terminator: None,
        }
    }
}

impl<'n, 'e> Valued<'n, 'e> {
    pub fn fill_in(&mut self) {
        if let Some(ref vec) = self.val_names {
            if vec.len() > 1 {
                self.num_vals = Some(vec.len() as u64);
            }
        }
    }
}

impl<'n, 'e, 'z> From<&'z Arg<'n, 'e>> for Valued<'n, 'e> {
    fn from(a: &'z Arg<'n, 'e>) -> Self {
        let mut v = a.v.clone();
        if let Some(ref vec) = a.v.val_names {
            if vec.len() > 1 {
                v.num_vals = Some(vec.len() as u64);
            }
        }
        v
    }
}
