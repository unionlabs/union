use std::io::Write;

use syn::ext::IdentExt;

use crate::bindgen::cdecl;
use crate::bindgen::config::{Config, Language};
use crate::bindgen::ir::{AnnotationSet, Cfg, ConditionWrite};
use crate::bindgen::ir::{Documentation, Path, ToCondition, Type};
use crate::bindgen::writer::{Source, SourceWriter};

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub ty: Type,
    pub cfg: Option<Cfg>,
    pub annotations: AnnotationSet,
    pub documentation: Documentation,
}

impl Field {
    pub fn from_name_and_type(name: String, ty: Type) -> Field {
        Field {
            name,
            ty,
            cfg: None,
            annotations: AnnotationSet::new(),
            documentation: Documentation::none(),
        }
    }

    pub fn load(field: &syn::Field, self_path: &Path) -> Result<Option<Field>, String> {
        Ok(if let Some(mut ty) = Type::load(&field.ty)? {
            ty.replace_self_with(self_path);
            Some(Field {
                name: field
                    .ident
                    .as_ref()
                    .ok_or_else(|| "field is missing identifier".to_string())?
                    .unraw()
                    .to_string(),
                ty,
                cfg: Cfg::load(&field.attrs),
                annotations: AnnotationSet::load(&field.attrs)?,
                documentation: Documentation::load(&field.attrs),
            })
        } else {
            None
        })
    }
}

impl Source for Field {
    fn write<F: Write>(&self, config: &Config, out: &mut SourceWriter<F>) {
        // Cython doesn't support conditional fields.
        let condition = self.cfg.to_condition(config);
        if config.language != Language::Cython {
            condition.write_before(config, out);
        }

        self.documentation.write(config, out);
        cdecl::write_field(out, &self.ty, &self.name, config);
        // Cython extern declarations don't manage layouts, layouts are defined entierly by the
        // corresponding C code. So we can omit bitfield sizes which are not supported by Cython.
        if config.language != Language::Cython {
            if let Some(bitfield) = self.annotations.atom("bitfield") {
                write!(out, ": {}", bitfield.unwrap_or_default());
            }
        }

        if config.language != Language::Cython {
            condition.write_after(config, out);
            // FIXME(#634): `write_vertical_source_list` should support
            // configuring list elements natively. For now we print a newline
            // here to avoid printing `#endif;` with semicolon.
            if condition.is_some() {
                out.new_line();
            }
        }
    }
}
