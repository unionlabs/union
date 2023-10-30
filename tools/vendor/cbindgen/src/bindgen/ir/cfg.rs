/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::fmt;
use std::io::Write;

use crate::bindgen::cargo::cargo_metadata::Dependency;
use crate::bindgen::config::{Config, Language};
use crate::bindgen::writer::SourceWriter;

#[derive(PartialEq, Eq)]
enum DefineKey<'a> {
    Boolean(&'a str),
    Named(&'a str, &'a str),
}

impl<'a> DefineKey<'a> {
    fn load(key: &str) -> DefineKey {
        // TODO: dirty parser
        if !key.contains('=') {
            return DefineKey::Boolean(key);
        }

        let mut splits = key.trim().split('=');

        let name = match splits.next() {
            Some(n) => n.trim(),
            None => return DefineKey::Boolean(key),
        };

        let value = match splits.next() {
            Some(v) => v.trim(),
            None => return DefineKey::Boolean(key),
        };

        if splits.next().is_some() {
            return DefineKey::Boolean(key);
        }

        DefineKey::Named(name, value)
    }
}

#[derive(Debug, Clone)]
pub enum Cfg {
    Boolean(String),
    Named(String, String),
    Any(Vec<Cfg>),
    All(Vec<Cfg>),
    Not(Box<Cfg>),
}

impl fmt::Display for Cfg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Cfg::Boolean(key) => write!(f, "{}", key),
            Cfg::Named(key, value) => write!(f, "{} = {:?}", key, value),
            Cfg::Any(cfgs) => {
                write!(f, "any(")?;
                for (index, cfg) in cfgs.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", cfg)?;
                }
                write!(f, ")")
            }
            Cfg::All(cfgs) => {
                write!(f, "all(")?;
                for (index, cfg) in cfgs.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", cfg)?;
                }
                write!(f, ")")
            }
            Cfg::Not(cfg) => write!(f, "not({})", cfg),
        }
    }
}

impl Cfg {
    pub fn join(cfgs: &[Cfg]) -> Option<Cfg> {
        if cfgs.is_empty() {
            None
        } else {
            Some(Cfg::All(cfgs.to_owned()))
        }
    }

    pub fn append(parent: Option<&Cfg>, child: Option<Cfg>) -> Option<Cfg> {
        match (parent, child) {
            (None, None) => None,
            (None, Some(child)) => Some(child),
            (Some(parent), None) => Some(parent.clone()),
            (Some(parent), Some(child)) => Some(Cfg::All(vec![parent.clone(), child])),
        }
    }

    pub fn load(attrs: &[syn::Attribute]) -> Option<Cfg> {
        let mut configs = Vec::new();

        for attr in attrs {
            if let Ok(syn::Meta::List(syn::MetaList { path, nested, .. })) = attr.parse_meta() {
                if !path.is_ident("cfg") || nested.len() != 1 {
                    continue;
                }

                if let Some(config) = Cfg::load_single(nested.first().unwrap()) {
                    configs.push(config);
                }
            }
        }

        match configs.len() {
            0 => None,
            1 => Some(configs.pop().unwrap()),
            _ => Some(Cfg::All(configs)),
        }
    }

    pub fn load_metadata(dependency: &Dependency) -> Option<Cfg> {
        let target = dependency.target.as_ref()?;
        match syn::parse_str::<syn::Meta>(target) {
            Ok(target) => {
                // Parsing succeeded using the #[cfg] syntax
                if let syn::Meta::List(syn::MetaList { path, nested, .. }) = target {
                    if !path.is_ident("cfg") || nested.len() != 1 {
                        return None;
                    }
                    Cfg::load_single(nested.first().unwrap())
                } else {
                    None
                }
            }
            Err(_) => {
                // Parsing failed using #[cfg], this may be a literal target
                // name
                Cfg::load_single(&syn::NestedMeta::Lit(syn::Lit::Str(syn::LitStr::new(
                    target,
                    proc_macro2::Span::call_site(),
                ))))
            }
        }
    }

    fn load_single(item: &syn::NestedMeta) -> Option<Cfg> {
        Some(match *item {
            syn::NestedMeta::Meta(syn::Meta::Path(ref path)) => {
                Cfg::Boolean(format!("{}", path.segments.first().unwrap().ident))
            }
            syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue {
                ref path,
                lit: syn::Lit::Str(ref value),
                ..
            })) => Cfg::Named(
                format!("{}", path.segments.first().unwrap().ident),
                value.value(),
            ),
            syn::NestedMeta::Meta(syn::Meta::List(syn::MetaList {
                ref path,
                ref nested,
                ..
            })) => {
                if path.is_ident("any") {
                    Cfg::Any(Cfg::load_list(nested.iter())?)
                } else if path.is_ident("all") {
                    Cfg::All(Cfg::load_list(nested.iter())?)
                } else if path.is_ident("not") {
                    if nested.len() != 1 {
                        return None;
                    }

                    Cfg::Not(Box::new(Cfg::load_single(&nested[0])?))
                } else {
                    return None;
                }
            }
            _ => return None,
        })
    }

    fn load_list<'a, I: Iterator<Item = &'a syn::NestedMeta>>(attrs: I) -> Option<Vec<Cfg>> {
        let mut configs = Vec::new();

        for attr in attrs {
            configs.push(Cfg::load_single(attr)?);
        }

        if configs.is_empty() {
            None
        } else {
            Some(configs)
        }
    }
}

pub trait ToCondition: Sized {
    fn to_condition(&self, config: &Config) -> Option<Condition>;
}

impl ToCondition for Option<Cfg> {
    fn to_condition(&self, config: &Config) -> Option<Condition> {
        self.as_ref()?.to_condition(config)
    }
}

impl ToCondition for Cfg {
    fn to_condition(&self, config: &Config) -> Option<Condition> {
        match *self {
            Cfg::Boolean(ref cfg_name) => {
                let define = config
                    .defines
                    .iter()
                    .find(|(key, ..)| DefineKey::Boolean(cfg_name) == DefineKey::load(key));
                if let Some((_, define)) = define {
                    Some(Condition::Define(define.to_owned()))
                } else {
                    warn!(
                        "Missing `[defines]` entry for `{}` in cbindgen config.",
                        self,
                    );
                    None
                }
            }
            Cfg::Named(ref cfg_name, ref cfg_value) => {
                let define = config.defines.iter().find(|(key, ..)| {
                    DefineKey::Named(cfg_name, cfg_value) == DefineKey::load(key)
                });
                if let Some((_, define)) = define {
                    Some(Condition::Define(define.to_owned()))
                } else {
                    warn!(
                        "Missing `[defines]` entry for `{}` in cbindgen config.",
                        self,
                    );
                    None
                }
            }
            Cfg::Any(ref children) => {
                let conditions: Vec<_> = children
                    .iter()
                    .filter_map(|x| x.to_condition(config))
                    .collect();
                match conditions.len() {
                    0 => None,
                    1 => conditions.into_iter().next(),
                    _ => Some(Condition::Any(conditions)),
                }
            }
            Cfg::All(ref children) => {
                let cfgs: Vec<_> = children
                    .iter()
                    .filter_map(|x| x.to_condition(config))
                    .collect();
                match cfgs.len() {
                    0 => None,
                    1 => cfgs.into_iter().next(),
                    _ => Some(Condition::All(cfgs)),
                }
            }
            Cfg::Not(ref child) => child
                .to_condition(config)
                .map(|cfg| Condition::Not(Box::new(cfg))),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Condition {
    Define(String),
    Any(Vec<Condition>),
    All(Vec<Condition>),
    Not(Box<Condition>),
}

impl Condition {
    fn write<F: Write>(&self, config: &Config, out: &mut SourceWriter<F>) {
        match *self {
            Condition::Define(ref define) => {
                if config.language == Language::Cython {
                    write!(out, "{}", define);
                } else {
                    out.write("defined(");
                    write!(out, "{}", define);
                    out.write(")");
                }
            }
            Condition::Any(ref conditions) => {
                out.write("(");
                for (i, condition) in conditions.iter().enumerate() {
                    if i != 0 {
                        out.write(if config.language == Language::Cython {
                            " or "
                        } else {
                            " || "
                        });
                    }
                    condition.write(config, out);
                }
                out.write(")");
            }
            Condition::All(ref conditions) => {
                out.write("(");
                for (i, condition) in conditions.iter().enumerate() {
                    if i != 0 {
                        out.write(if config.language == Language::Cython {
                            " and "
                        } else {
                            " && "
                        });
                    }
                    condition.write(config, out);
                }
                out.write(")");
            }
            Condition::Not(ref condition) => {
                out.write(if config.language == Language::Cython {
                    "not "
                } else {
                    "!"
                });
                condition.write(config, out);
            }
        }
    }
}

pub trait ConditionWrite {
    fn write_before<F: Write>(&self, config: &Config, out: &mut SourceWriter<F>);
    fn write_after<F: Write>(&self, config: &Config, out: &mut SourceWriter<F>);
}

impl ConditionWrite for Option<Condition> {
    fn write_before<F: Write>(&self, config: &Config, out: &mut SourceWriter<F>) {
        if let Some(ref cfg) = *self {
            if config.language == Language::Cython {
                out.write("IF ");
                cfg.write(config, out);
                out.open_brace();
            } else {
                out.push_set_spaces(0);
                out.write("#if ");
                cfg.write(config, out);
                out.pop_set_spaces();
                out.new_line();
            }
        }
    }

    fn write_after<F: Write>(&self, config: &Config, out: &mut SourceWriter<F>) {
        if self.is_some() {
            if config.language == Language::Cython {
                out.close_brace(false);
            } else {
                out.new_line();
                out.push_set_spaces(0);
                out.write("#endif");
                out.pop_set_spaces();
            }
        }
    }
}
