use crate::args::{Arg, ArgFlags, ArgSettings};

#[derive(Debug, Clone, Default)]
pub struct Base<'a, 'b>
where
    'a: 'b,
{
    pub name: &'a str,
    pub help: Option<&'b str>,
    pub long_help: Option<&'b str>,
    pub blacklist: Option<Vec<&'a str>>,
    pub settings: ArgFlags,
    pub r_unless: Option<Vec<&'a str>>,
    pub overrides: Option<Vec<&'a str>>,
    pub groups: Option<Vec<&'a str>>,
    pub requires: Option<Vec<(Option<&'b str>, &'a str)>>,
}

impl<'n, 'e> Base<'n, 'e> {
    pub fn new(name: &'n str) -> Self {
        Base {
            name,
            ..Default::default()
        }
    }

    pub fn set(&mut self, s: ArgSettings) {
        self.settings.set(s);
    }
    pub fn unset(&mut self, s: ArgSettings) {
        self.settings.unset(s);
    }
    pub fn is_set(&self, s: ArgSettings) -> bool {
        self.settings.is_set(s)
    }
}

impl<'n, 'e, 'z> From<&'z Arg<'n, 'e>> for Base<'n, 'e> {
    fn from(a: &'z Arg<'n, 'e>) -> Self {
        a.b.clone()
    }
}

impl<'n, 'e> PartialEq for Base<'n, 'e> {
    fn eq(&self, other: &Base<'n, 'e>) -> bool {
        self.name == other.name
    }
}
