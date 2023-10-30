//! Generate the ISA-specific settings.
use std::collections::HashMap;

use crate::constant_hash::generate_table;
use cranelift_codegen_shared::constant_hash::simple_hash;

use crate::cdsl::camel_case;
use crate::cdsl::settings::{
    BoolSetting, Predicate, Preset, Setting, SettingGroup, SpecificSetting,
};
use crate::error;
use crate::srcgen::{Formatter, Match};
use crate::unique_table::UniqueSeqTable;

pub(crate) enum ParentGroup {
    None,
    Shared,
}

/// Emits the constructor of the Flags structure.
fn gen_constructor(group: &SettingGroup, parent: ParentGroup, fmt: &mut Formatter) {
    let args = match parent {
        ParentGroup::None => "builder: Builder",
        ParentGroup::Shared => "shared: &settings::Flags, builder: Builder",
    };
    fmtln!(fmt, "impl Flags {");
    fmt.indent(|fmt| {
        fmt.doc_comment(format!("Create flags {} settings group.", group.name));
        fmtln!(fmt, "#[allow(unused_variables)]");
        fmtln!(fmt, "pub fn new({}) -> Self {{", args);
        fmt.indent(|fmt| {
            fmtln!(fmt, "let bvec = builder.state_for(\"{}\");", group.name);
            fmtln!(
                fmt,
                "let mut {} = Self {{ bytes: [0; {}] }};",
                group.name,
                group.byte_size()
            );
            fmtln!(
                fmt,
                "debug_assert_eq!(bvec.len(), {});",
                group.settings_size
            );
            fmtln!(
                fmt,
                "{}.bytes[0..{}].copy_from_slice(&bvec);",
                group.name,
                group.settings_size
            );

            // Now compute the predicates.
            for p in &group.predicates {
                fmt.comment(format!("Precompute #{}.", p.number));
                fmtln!(fmt, "if {} {{", p.render(group));
                fmt.indent(|fmt| {
                    fmtln!(
                        fmt,
                        "{}.bytes[{}] |= 1 << {};",
                        group.name,
                        group.bool_start_byte_offset + p.number / 8,
                        p.number % 8
                    );
                });
                fmtln!(fmt, "}");
            }

            fmtln!(fmt, group.name);
        });
        fmtln!(fmt, "}");
    });
    fmtln!(fmt, "}");
}

/// Generates the `iter` function.
fn gen_iterator(group: &SettingGroup, fmt: &mut Formatter) {
    fmtln!(fmt, "impl Flags {");
    fmt.indent(|fmt| {
        fmt.doc_comment("Iterates the setting values.");
        fmtln!(fmt, "pub fn iter(&self) -> impl Iterator<Item = Value> {");
        fmt.indent(|fmt| {
            fmtln!(fmt, "let mut bytes = [0; {}];", group.settings_size);
            fmtln!(fmt, "bytes.copy_from_slice(&self.bytes[0..{}]);", group.settings_size);
            fmtln!(fmt, "DESCRIPTORS.iter().filter_map(move |d| {");
            fmt.indent(|fmt| {
                fmtln!(fmt, "let values = match &d.detail {");
                fmt.indent(|fmt| {
                    fmtln!(fmt, "detail::Detail::Preset => return None,");
                    fmtln!(fmt, "detail::Detail::Enum { last, enumerators } => Some(TEMPLATE.enums(*last, *enumerators)),");
                    fmtln!(fmt, "_ => None");
                });
                fmtln!(fmt, "};");
                fmtln!(fmt, "Some(Value{ name: d.name, detail: d.detail, values, value: bytes[d.offset as usize] })");
            });
            fmtln!(fmt, "})");
        });
        fmtln!(fmt, "}");
    });
    fmtln!(fmt, "}");
}

/// Emit Display and FromStr implementations for enum settings.
fn gen_to_and_from_str(name: &str, values: &[&'static str], fmt: &mut Formatter) {
    fmtln!(fmt, "impl fmt::Display for {} {{", name);
    fmt.indent(|fmt| {
        fmtln!(
            fmt,
            "fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {"
        );
        fmt.indent(|fmt| {
            fmtln!(fmt, "f.write_str(match *self {");
            fmt.indent(|fmt| {
                for v in values.iter() {
                    fmtln!(fmt, "Self::{} => \"{}\",", camel_case(v), v);
                }
            });
            fmtln!(fmt, "})");
        });
        fmtln!(fmt, "}");
    });
    fmtln!(fmt, "}");

    fmtln!(fmt, "impl str::FromStr for {} {{", name);
    fmt.indent(|fmt| {
        fmtln!(fmt, "type Err = ();");
        fmtln!(fmt, "fn from_str(s: &str) -> Result<Self, Self::Err> {");
        fmt.indent(|fmt| {
            fmtln!(fmt, "match s {");
            fmt.indent(|fmt| {
                for v in values.iter() {
                    fmtln!(fmt, "\"{}\" => Ok(Self::{}),", v, camel_case(v));
                }
                fmtln!(fmt, "_ => Err(()),");
            });
            fmtln!(fmt, "}");
        });
        fmtln!(fmt, "}");
    });
    fmtln!(fmt, "}");
}

/// Emit real enum for the Enum settings.
fn gen_enum_types(group: &SettingGroup, fmt: &mut Formatter) {
    for setting in group.settings.iter() {
        let values = match setting.specific {
            SpecificSetting::Bool(_) | SpecificSetting::Num(_) => continue,
            SpecificSetting::Enum(ref values) => values,
        };
        let name = camel_case(setting.name);

        fmt.doc_comment(format!("Values for `{}.{}`.", group.name, setting.name));
        fmtln!(fmt, "#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]");
        fmtln!(fmt, "pub enum {} {{", name);
        fmt.indent(|fmt| {
            for v in values.iter() {
                fmt.doc_comment(format!("`{}`.", v));
                fmtln!(fmt, "{},", camel_case(v));
            }
        });
        fmtln!(fmt, "}");

        gen_to_and_from_str(&name, values, fmt);
    }
}

/// Emit a getter function for `setting`.
fn gen_getter(setting: &Setting, fmt: &mut Formatter) {
    fmt.doc_comment(format!("{}\n{}", setting.description, setting.comment));
    match setting.specific {
        SpecificSetting::Bool(BoolSetting {
            predicate_number, ..
        }) => {
            fmtln!(fmt, "pub fn {}(&self) -> bool {{", setting.name);
            fmt.indent(|fmt| {
                fmtln!(fmt, "self.numbered_predicate({})", predicate_number);
            });
            fmtln!(fmt, "}");
        }
        SpecificSetting::Enum(ref values) => {
            let ty = camel_case(setting.name);
            fmtln!(fmt, "pub fn {}(&self) -> {} {{", setting.name, ty);
            fmt.indent(|fmt| {
                let mut m = Match::new(format!("self.bytes[{}]", setting.byte_offset));
                for (i, v) in values.iter().enumerate() {
                    m.arm_no_fields(format!("{}", i), format!("{}::{}", ty, camel_case(v)));
                }
                m.arm_no_fields("_", "panic!(\"Invalid enum value\")");
                fmt.add_match(m);
            });
            fmtln!(fmt, "}");
        }
        SpecificSetting::Num(_) => {
            fmtln!(fmt, "pub fn {}(&self) -> u8 {{", setting.name);
            fmt.indent(|fmt| {
                fmtln!(fmt, "self.bytes[{}]", setting.byte_offset);
            });
            fmtln!(fmt, "}");
        }
    }
}

fn gen_pred_getter(predicate: &Predicate, group: &SettingGroup, fmt: &mut Formatter) {
    fmt.doc_comment(format!("Computed predicate `{}`.", predicate.render(group)));
    fmtln!(fmt, "pub fn {}(&self) -> bool {{", predicate.name);
    fmt.indent(|fmt| {
        fmtln!(fmt, "self.numbered_predicate({})", predicate.number);
    });
    fmtln!(fmt, "}");
}

/// Emits getters for each setting value.
fn gen_getters(group: &SettingGroup, fmt: &mut Formatter) {
    fmt.doc_comment("User-defined settings.");
    fmtln!(fmt, "#[allow(dead_code)]");
    fmtln!(fmt, "impl Flags {");
    fmt.indent(|fmt| {
        fmt.doc_comment("Get a view of the boolean predicates.");
        fmtln!(
            fmt,
            "pub fn predicate_view(&self) -> crate::settings::PredicateView {"
        );
        fmt.indent(|fmt| {
            fmtln!(
                fmt,
                "crate::settings::PredicateView::new(&self.bytes[{}..])",
                group.bool_start_byte_offset
            );
        });
        fmtln!(fmt, "}");

        if !group.settings.is_empty() {
            fmt.doc_comment("Dynamic numbered predicate getter.");
            fmtln!(fmt, "fn numbered_predicate(&self, p: usize) -> bool {");
            fmt.indent(|fmt| {
                fmtln!(
                    fmt,
                    "self.bytes[{} + p / 8] & (1 << (p % 8)) != 0",
                    group.bool_start_byte_offset
                );
            });
            fmtln!(fmt, "}");
        }

        for setting in &group.settings {
            gen_getter(&setting, fmt);
        }
        for predicate in &group.predicates {
            gen_pred_getter(&predicate, &group, fmt);
        }
    });
    fmtln!(fmt, "}");
}

#[derive(Hash, PartialEq, Eq)]
enum SettingOrPreset<'a> {
    Setting(&'a Setting),
    Preset(&'a Preset),
}

impl<'a> SettingOrPreset<'a> {
    fn name(&self) -> &str {
        match *self {
            SettingOrPreset::Setting(s) => s.name,
            SettingOrPreset::Preset(p) => p.name,
        }
    }
}

/// Emits DESCRIPTORS, ENUMERATORS, HASH_TABLE and PRESETS.
fn gen_descriptors(group: &SettingGroup, fmt: &mut Formatter) {
    let mut enum_table = UniqueSeqTable::new();

    let mut descriptor_index_map: HashMap<SettingOrPreset, usize> = HashMap::new();

    // Generate descriptors.
    fmtln!(
        fmt,
        "static DESCRIPTORS: [detail::Descriptor; {}] = [",
        group.settings.len() + group.presets.len()
    );
    fmt.indent(|fmt| {
        for (idx, setting) in group.settings.iter().enumerate() {
            fmtln!(fmt, "detail::Descriptor {");
            fmt.indent(|fmt| {
                fmtln!(fmt, "name: \"{}\",", setting.name);
                fmtln!(fmt, "description: \"{}\",", setting.description);
                fmtln!(fmt, "offset: {},", setting.byte_offset);
                match setting.specific {
                    SpecificSetting::Bool(BoolSetting { bit_offset, .. }) => {
                        fmtln!(
                            fmt,
                            "detail: detail::Detail::Bool {{ bit: {} }},",
                            bit_offset
                        );
                    }
                    SpecificSetting::Enum(ref values) => {
                        let offset = enum_table.add(values);
                        fmtln!(
                            fmt,
                            "detail: detail::Detail::Enum {{ last: {}, enumerators: {} }},",
                            values.len() - 1,
                            offset
                        );
                    }
                    SpecificSetting::Num(_) => {
                        fmtln!(fmt, "detail: detail::Detail::Num,");
                    }
                }

                descriptor_index_map.insert(SettingOrPreset::Setting(setting), idx);
            });
            fmtln!(fmt, "},");
        }

        for (idx, preset) in group.presets.iter().enumerate() {
            fmtln!(fmt, "detail::Descriptor {");
            fmt.indent(|fmt| {
                fmtln!(fmt, "name: \"{}\",", preset.name);
                fmtln!(fmt, "description: \"{}\",", preset.description);
                fmtln!(fmt, "offset: {},", (idx as u8) * group.settings_size);
                fmtln!(fmt, "detail: detail::Detail::Preset,");
            });
            fmtln!(fmt, "},");

            let whole_idx = idx + group.settings.len();
            descriptor_index_map.insert(SettingOrPreset::Preset(preset), whole_idx);
        }
    });
    fmtln!(fmt, "];");

    // Generate enumerators.
    fmtln!(fmt, "static ENUMERATORS: [&str; {}] = [", enum_table.len());
    fmt.indent(|fmt| {
        for enum_val in enum_table.iter() {
            fmtln!(fmt, "\"{}\",", enum_val);
        }
    });
    fmtln!(fmt, "];");

    // Generate hash table.
    let mut hash_entries: Vec<SettingOrPreset> = Vec::new();
    hash_entries.extend(group.settings.iter().map(|x| SettingOrPreset::Setting(x)));
    hash_entries.extend(group.presets.iter().map(|x| SettingOrPreset::Preset(x)));

    let hash_table = generate_table(hash_entries.iter(), hash_entries.len(), |entry| {
        simple_hash(entry.name())
    });
    fmtln!(fmt, "static HASH_TABLE: [u16; {}] = [", hash_table.len());
    fmt.indent(|fmt| {
        for h in &hash_table {
            match *h {
                Some(setting_or_preset) => fmtln!(
                    fmt,
                    "{},",
                    &descriptor_index_map
                        .get(setting_or_preset)
                        .unwrap()
                        .to_string()
                ),
                None => fmtln!(fmt, "0xffff,"),
            }
        }
    });
    fmtln!(fmt, "];");

    // Generate presets.
    fmtln!(
        fmt,
        "static PRESETS: [(u8, u8); {}] = [",
        group.presets.len() * (group.settings_size as usize)
    );
    fmt.indent(|fmt| {
        for preset in &group.presets {
            fmt.comment(preset.name);
            for (mask, value) in preset.layout(&group) {
                fmtln!(fmt, "(0b{:08b}, 0b{:08b}),", mask, value);
            }
        }
    });
    fmtln!(fmt, "];");
}

fn gen_template(group: &SettingGroup, fmt: &mut Formatter) {
    let mut default_bytes: Vec<u8> = vec![0; group.settings_size as usize];
    for setting in &group.settings {
        *default_bytes.get_mut(setting.byte_offset as usize).unwrap() |= setting.default_byte();
    }

    let default_bytes: Vec<String> = default_bytes
        .iter()
        .map(|x| format!("{:#04x}", x))
        .collect();
    let default_bytes_str = default_bytes.join(", ");

    fmtln!(
        fmt,
        "static TEMPLATE: detail::Template = detail::Template {"
    );
    fmt.indent(|fmt| {
        fmtln!(fmt, "name: \"{}\",", group.name);
        fmtln!(fmt, "descriptors: &DESCRIPTORS,");
        fmtln!(fmt, "enumerators: &ENUMERATORS,");
        fmtln!(fmt, "hash_table: &HASH_TABLE,");
        fmtln!(fmt, "defaults: &[{}],", default_bytes_str);
        fmtln!(fmt, "presets: &PRESETS,");
    });
    fmtln!(fmt, "};");

    fmt.doc_comment(format!(
        "Create a `settings::Builder` for the {} settings group.",
        group.name
    ));
    fmtln!(fmt, "pub fn builder() -> Builder {");
    fmt.indent(|fmt| {
        fmtln!(fmt, "Builder::new(&TEMPLATE)");
    });
    fmtln!(fmt, "}");
}

fn gen_display(group: &SettingGroup, fmt: &mut Formatter) {
    fmtln!(fmt, "impl fmt::Display for Flags {");
    fmt.indent(|fmt| {
        fmtln!(
            fmt,
            "fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {"
        );
        fmt.indent(|fmt| {
            fmtln!(fmt, "writeln!(f, \"[{}]\")?;", group.name);
            fmtln!(fmt, "for d in &DESCRIPTORS {");
            fmt.indent(|fmt| {
                fmtln!(fmt, "if !d.detail.is_preset() {");
                fmt.indent(|fmt| {
                    fmtln!(fmt, "write!(f, \"{} = \", d.name)?;");
                    fmtln!(
                        fmt,
                        "TEMPLATE.format_toml_value(d.detail, self.bytes[d.offset as usize], f)?;",
                    );
                    fmtln!(fmt, "writeln!(f)?;");
                });
                fmtln!(fmt, "}");
            });
            fmtln!(fmt, "}");
            fmtln!(fmt, "Ok(())");
        });
        fmtln!(fmt, "}")
    });
    fmtln!(fmt, "}");
}

fn gen_group(group: &SettingGroup, parent: ParentGroup, fmt: &mut Formatter) {
    // Generate struct.
    fmtln!(fmt, "#[derive(Clone, Hash)]");
    fmt.doc_comment(format!("Flags group `{}`.", group.name));
    fmtln!(fmt, "pub struct Flags {");
    fmt.indent(|fmt| {
        fmtln!(fmt, "bytes: [u8; {}],", group.byte_size());
    });
    fmtln!(fmt, "}");

    gen_constructor(group, parent, fmt);
    gen_iterator(group, fmt);
    gen_enum_types(group, fmt);
    gen_getters(group, fmt);
    gen_descriptors(group, fmt);
    gen_template(group, fmt);
    gen_display(group, fmt);
}

pub(crate) fn generate(
    settings: &SettingGroup,
    parent_group: ParentGroup,
    filename: &str,
    out_dir: &str,
) -> Result<(), error::Error> {
    let mut fmt = Formatter::new();
    gen_group(&settings, parent_group, &mut fmt);
    fmt.update_file(filename, out_dir)?;
    Ok(())
}
