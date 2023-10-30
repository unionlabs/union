/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::io::Write;

use syn::ext::IdentExt;

use crate::bindgen::config::{Config, Language, LayoutConfig};
use crate::bindgen::declarationtyperesolver::DeclarationTypeResolver;
use crate::bindgen::dependencies::Dependencies;
use crate::bindgen::ir::{
    AnnotationSet, Cfg, ConditionWrite, Constant, Documentation, Field, GenericArgument,
    GenericParams, Item, ItemContainer, Path, Repr, ReprAlign, ReprStyle, ToCondition, Type,
    Typedef,
};
use crate::bindgen::library::Library;
use crate::bindgen::mangle;
use crate::bindgen::monomorph::Monomorphs;
use crate::bindgen::rename::{IdentifierType, RenameRule};
use crate::bindgen::reserved;
use crate::bindgen::utilities::IterHelpers;
use crate::bindgen::writer::{ListType, Source, SourceWriter};

#[derive(Debug, Clone)]
pub struct Struct {
    pub path: Path,
    pub export_name: String,
    pub generic_params: GenericParams,
    pub fields: Vec<Field>,
    /// Whether there's a tag field on the body of this struct. When this is
    /// true, is_enum_variant_body is also guaranteed to be true.
    pub has_tag_field: bool,
    /// Whether this is an enum variant body.
    pub is_enum_variant_body: bool,
    pub alignment: Option<ReprAlign>,
    pub is_transparent: bool,
    pub cfg: Option<Cfg>,
    pub annotations: AnnotationSet,
    pub documentation: Documentation,
    pub associated_constants: Vec<Constant>,
}

impl Struct {
    /// Whether this struct can derive operator== / operator!=.
    pub fn can_derive_eq(&self) -> bool {
        !self.fields.is_empty() && self.fields.iter().all(|x| x.ty.can_cmp_eq())
    }

    pub fn add_associated_constant(&mut self, c: Constant) {
        self.associated_constants.push(c);
    }

    pub fn load(
        layout_config: &LayoutConfig,
        item: &syn::ItemStruct,
        mod_cfg: Option<&Cfg>,
    ) -> Result<Self, String> {
        let repr = Repr::load(&item.attrs)?;
        let is_transparent = match repr.style {
            ReprStyle::C => false,
            ReprStyle::Transparent => true,
            _ => {
                return Err("Struct is not marked #[repr(C)] or #[repr(transparent)].".to_owned());
            }
        };

        let path = Path::new(item.ident.unraw().to_string());

        // Ensure we can safely represent the struct given the configuration.
        if let Some(align) = repr.align {
            layout_config.ensure_safe_to_represent(&align)?;
        }

        let fields = match item.fields {
            syn::Fields::Unit => Vec::new(),
            syn::Fields::Named(ref fields) => fields
                .named
                .iter()
                .try_skip_map(|field| Field::load(field, &path))?,
            syn::Fields::Unnamed(ref fields) => {
                let mut out = Vec::new();
                let mut current = 0;
                for field in fields.unnamed.iter() {
                    if let Some(mut ty) = Type::load(&field.ty)? {
                        ty.replace_self_with(&path);
                        out.push(Field {
                            name: format!("{}", current),
                            ty,
                            cfg: Cfg::load(&field.attrs),
                            annotations: AnnotationSet::load(&field.attrs)?,
                            documentation: Documentation::load(&field.attrs),
                        });
                        current += 1;
                    }
                }
                out
            }
        };

        let has_tag_field = false;
        let is_enum_variant_body = false;

        Ok(Struct::new(
            path,
            GenericParams::load(&item.generics)?,
            fields,
            has_tag_field,
            is_enum_variant_body,
            repr.align,
            is_transparent,
            Cfg::append(mod_cfg, Cfg::load(&item.attrs)),
            AnnotationSet::load(&item.attrs)?,
            Documentation::load(&item.attrs),
        ))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        path: Path,
        generic_params: GenericParams,
        fields: Vec<Field>,
        has_tag_field: bool,
        is_enum_variant_body: bool,
        alignment: Option<ReprAlign>,
        is_transparent: bool,
        cfg: Option<Cfg>,
        annotations: AnnotationSet,
        documentation: Documentation,
    ) -> Self {
        let export_name = path.name().to_owned();
        Self {
            path,
            export_name,
            generic_params,
            fields,
            has_tag_field,
            is_enum_variant_body,
            alignment,
            is_transparent,
            cfg,
            annotations,
            documentation,
            associated_constants: vec![],
        }
    }

    pub fn simplify_standard_types(&mut self, config: &Config) {
        for field in &mut self.fields {
            field.ty.simplify_standard_types(config);
        }
    }

    pub fn is_generic(&self) -> bool {
        self.generic_params.len() > 0
    }

    pub fn add_monomorphs(&self, library: &Library, out: &mut Monomorphs) {
        // Generic structs can instantiate monomorphs only once they've been
        // instantiated. See `instantiate_monomorph` for more details.
        if self.is_generic() {
            return;
        }

        for field in &self.fields {
            field.ty.add_monomorphs(library, out);
        }
    }

    pub fn mangle_paths(&mut self, monomorphs: &Monomorphs) {
        for field in &mut self.fields {
            field.ty.mangle_paths(monomorphs);
        }
    }

    pub fn specialize(
        &self,
        generic_values: &[GenericArgument],
        mappings: &[(&Path, &GenericArgument)],
        config: &Config,
    ) -> Self {
        let mangled_path = mangle::mangle_path(&self.path, generic_values, &config.export.mangle);
        Struct::new(
            mangled_path,
            GenericParams::default(),
            self.fields
                .iter()
                .map(|field| Field {
                    name: field.name.clone(),
                    ty: field.ty.specialize(mappings),
                    cfg: field.cfg.clone(),
                    annotations: field.annotations.clone(),
                    documentation: field.documentation.clone(),
                })
                .collect(),
            self.has_tag_field,
            self.is_enum_variant_body,
            self.alignment,
            self.is_transparent,
            self.cfg.clone(),
            self.annotations.clone(),
            self.documentation.clone(),
        )
    }

    fn emit_bitflags_binop<F: Write>(
        &self,
        constexpr_prefix: &str,
        operator: char,
        other: &str,
        out: &mut SourceWriter<F>,
    ) {
        out.new_line();
        write!(
            out,
            "{}{} operator{}(const {}& {}) const",
            constexpr_prefix,
            self.export_name(),
            operator,
            self.export_name(),
            other
        );
        out.open_brace();
        write!(
            out,
            "return {} {{ static_cast<decltype(bits)>(this->bits {} {}.bits) }};",
            self.export_name(),
            operator,
            other
        );
        out.close_brace(false);

        out.new_line();
        write!(
            out,
            "{}& operator{}=(const {}& {})",
            self.export_name(),
            operator,
            self.export_name(),
            other
        );
        out.open_brace();
        write!(out, "*this = (*this {} {});", operator, other);
        out.new_line();
        write!(out, "return *this;");
        out.close_brace(false);
    }
}

impl Item for Struct {
    fn path(&self) -> &Path {
        &self.path
    }

    fn export_name(&self) -> &str {
        &self.export_name
    }

    fn cfg(&self) -> Option<&Cfg> {
        self.cfg.as_ref()
    }

    fn annotations(&self) -> &AnnotationSet {
        &self.annotations
    }

    fn annotations_mut(&mut self) -> &mut AnnotationSet {
        &mut self.annotations
    }

    fn container(&self) -> ItemContainer {
        ItemContainer::Struct(self.clone())
    }

    fn collect_declaration_types(&self, resolver: &mut DeclarationTypeResolver) {
        if self.is_transparent {
            resolver.add_none(&self.path);
        } else {
            resolver.add_struct(&self.path);
        }
    }

    fn resolve_declaration_types(&mut self, resolver: &DeclarationTypeResolver) {
        for field in &mut self.fields {
            field.ty.resolve_declaration_types(resolver);
        }
    }

    fn rename_for_config(&mut self, config: &Config) {
        // Rename the name of the struct
        if !(self.has_tag_field && config.language == Language::Cxx) {
            config.export.rename(&mut self.export_name);
        }

        // Rename the types used in fields
        {
            let fields = self.fields.iter_mut().skip(self.has_tag_field as usize);
            for field in fields {
                field.ty.rename_for_config(config, &self.generic_params);
            }
        }

        // Apply renaming rules to fields in the following order
        //   1. `cbindgen::field-names` annotation
        //   2. `cbindgen::rename-all` annotation
        //   3. config struct rename rule
        // If the struct is a tuple struct and we have not renamed the
        // fields, then prefix each of them with an underscore.
        // If any field is a reserved keyword, then postfix it with an
        // underscore.

        // Scope for mutable borrow of fields
        {
            let names = self.fields.iter_mut().map(|field| &mut field.name);

            let field_rules = self
                .annotations
                .parse_atom::<RenameRule>("rename-all")
                .unwrap_or(config.structure.rename_fields);

            if let Some(o) = self.annotations.list("field-names") {
                for (dest, src) in names.zip(o) {
                    *dest = src;
                }
            } else if let Some(r) = field_rules.not_none() {
                for name in names {
                    *name = r.apply(name, IdentifierType::StructMember).into_owned();
                }
            } else {
                // If we don't have any rules for a tuple struct, prefix them with
                // an underscore so it still compiles.
                for name in names {
                    if name.starts_with(|c: char| c.is_ascii_digit()) {
                        name.insert(0, '_');
                    }
                }
            }
        }

        for field in &mut self.fields {
            reserved::escape(&mut field.name);
        }

        for c in self.associated_constants.iter_mut() {
            c.rename_for_config(config);
        }
    }

    fn add_dependencies(&self, library: &Library, out: &mut Dependencies) {
        let mut fields = self.fields.iter();

        // If there is a tag field, skip it
        if self.has_tag_field {
            fields.next();
        }

        for field in fields {
            field
                .ty
                .add_dependencies_ignoring_generics(&self.generic_params, library, out);
        }

        for c in &self.associated_constants {
            c.add_dependencies(library, out);
        }
    }

    fn instantiate_monomorph(
        &self,
        generic_values: &[GenericArgument],
        library: &Library,
        out: &mut Monomorphs,
    ) {
        let mappings = self.generic_params.call(self.path.name(), generic_values);
        let monomorph = self.specialize(generic_values, &mappings, library.get_config());
        out.insert_struct(library, self, monomorph, generic_values.to_owned());
    }
}

impl Source for Struct {
    fn write<F: Write>(&self, config: &Config, out: &mut SourceWriter<F>) {
        if self.is_transparent {
            let typedef = Typedef {
                path: self.path.clone(),
                export_name: self.export_name.to_owned(),
                generic_params: self.generic_params.clone(),
                aliased: self.fields[0].ty.clone(),
                cfg: self.cfg.clone(),
                annotations: self.annotations.clone(),
                documentation: self.documentation.clone(),
            };
            typedef.write(config, out);
            for constant in &self.associated_constants {
                out.new_line();
                constant.write(config, out, Some(self));
            }
            return;
        }

        let condition = self.cfg.to_condition(config);
        condition.write_before(config, out);

        self.documentation.write(config, out);

        if !self.is_enum_variant_body {
            self.generic_params.write(config, out);
        }

        // The following results in
        // C++ or C with Tag as style:
        //   struct Name {
        // C with Type only style:
        //   typedef struct {
        // C with Both as style:
        //   typedef struct Name {
        match config.language {
            Language::C if config.style.generate_typedef() => out.write("typedef "),
            Language::C | Language::Cxx => {}
            Language::Cython => out.write(config.style.cython_def()),
        }

        // Cython extern declarations don't manage layouts, layouts are defined entierly by the
        // corresponding C code. So this `packed` is only for documentation, and missing
        // `aligned(n)` is also not a problem.
        if config.language == Language::Cython {
            if let Some(align) = self.alignment {
                match align {
                    ReprAlign::Packed => out.write("packed "),
                    ReprAlign::Align(_) => {} // Not supported
                }
            }
        }

        out.write("struct");

        if config.language != Language::Cython {
            if let Some(align) = self.alignment {
                match align {
                    ReprAlign::Packed => {
                        if let Some(ref anno) = config.layout.packed {
                            write!(out, " {}", anno);
                        }
                    }
                    ReprAlign::Align(n) => {
                        if let Some(ref anno) = config.layout.aligned_n {
                            write!(out, " {}({})", anno, n);
                        }
                    }
                }
            }
        }

        if self.annotations.must_use(config) {
            if let Some(ref anno) = config.structure.must_use {
                write!(out, " {}", anno);
            }
        }

        if config.language != Language::C || config.style.generate_tag() {
            write!(out, " {}", self.export_name());
        }

        out.open_brace();

        // Emit the pre_body section, if relevant
        if let Some(body) = config.export.pre_body(&self.path) {
            out.write_raw_block(body);
            out.new_line();
        }

        out.write_vertical_source_list(&self.fields, ListType::Cap(";"));
        if config.language == Language::Cython && self.fields.is_empty() {
            out.write("pass");
        }

        if config.language == Language::Cxx {
            let mut wrote_start_newline = false;

            if config.structure.derive_constructor(&self.annotations) && !self.fields.is_empty() {
                if !wrote_start_newline {
                    wrote_start_newline = true;
                    out.new_line();
                }

                out.new_line();

                let arg_renamer = |name: &str| {
                    config
                        .function
                        .rename_args
                        .apply(name, IdentifierType::FunctionArg)
                        .into_owned()
                };
                write!(out, "{}(", self.export_name());
                let vec: Vec<_> = self
                    .fields
                    .iter()
                    .map(|field| {
                        Field::from_name_and_type(
                            // const-ref args to constructor
                            format!("const& {}", arg_renamer(&field.name)),
                            field.ty.clone(),
                        )
                    })
                    .collect();
                out.write_vertical_source_list(&vec[..], ListType::Join(","));
                write!(out, ")");
                out.new_line();
                write!(out, "  : ");
                let vec: Vec<_> = self
                    .fields
                    .iter()
                    .map(|field| format!("{}({})", field.name, arg_renamer(&field.name)))
                    .collect();
                out.write_vertical_source_list(&vec[..], ListType::Join(","));
                out.new_line();
                write!(out, "{{}}");
                out.new_line();
            }

            let other = config
                .function
                .rename_args
                .apply("other", IdentifierType::FunctionArg);

            if self
                .annotations
                .bool("internal-derive-bitflags")
                .unwrap_or(false)
            {
                if !wrote_start_newline {
                    wrote_start_newline = true;
                    out.new_line();
                }
                let constexpr_prefix = if config.constant.allow_constexpr {
                    "constexpr "
                } else {
                    ""
                };

                out.new_line();
                write!(out, "{}explicit operator bool() const", constexpr_prefix);
                out.open_brace();
                write!(out, "return !!bits;");
                out.close_brace(false);

                out.new_line();
                write!(
                    out,
                    "{}{} operator~() const",
                    constexpr_prefix,
                    self.export_name()
                );
                out.open_brace();
                write!(
                    out,
                    "return {} {{ static_cast<decltype(bits)>(~bits) }};",
                    self.export_name()
                );
                out.close_brace(false);
                self.emit_bitflags_binop(constexpr_prefix, '|', &other, out);
                self.emit_bitflags_binop(constexpr_prefix, '&', &other, out);
                self.emit_bitflags_binop(constexpr_prefix, '^', &other, out);
            }

            // Generate a serializer function that allows dumping this struct
            // to an std::ostream. It's defined as a friend function inside the
            // struct definition, and doesn't need the `inline` keyword even
            // though it's implemented right in the generated header file.
            if config.structure.derive_ostream(&self.annotations) {
                if !wrote_start_newline {
                    wrote_start_newline = true;
                    out.new_line();
                }

                out.new_line();
                let stream = config
                    .function
                    .rename_args
                    .apply("stream", IdentifierType::FunctionArg);
                let instance = config
                    .function
                    .rename_args
                    .apply("instance", IdentifierType::FunctionArg);
                write!(
                    out,
                    "friend std::ostream& operator<<(std::ostream& {}, const {}& {})",
                    stream,
                    self.export_name(),
                    instance,
                );
                out.open_brace();
                write!(out, "return {} << \"{{ \"", stream);
                let vec: Vec<_> = self
                    .fields
                    .iter()
                    .map(|x| format!(" << \"{}=\" << {}.{}", x.name, instance, x.name))
                    .collect();
                out.write_vertical_source_list(&vec[..], ListType::Join(" << \", \""));
                out.write(" << \" }\";");
                out.close_brace(false);
            }

            let skip_fields = self.has_tag_field as usize;

            macro_rules! emit_op {
                ($op_name:expr, $op:expr, $conjuc:expr) => {{
                    if !wrote_start_newline {
                        #[allow(unused_assignments)]
                        {
                            wrote_start_newline = true;
                        }
                        out.new_line();
                    }

                    out.new_line();

                    if let Some(Some(attrs)) =
                        self.annotations.atom(concat!($op_name, "-attributes"))
                    {
                        write!(out, "{} ", attrs);
                    }

                    write!(
                        out,
                        "bool operator{}(const {}& {}) const",
                        $op,
                        self.export_name(),
                        other
                    );
                    out.open_brace();
                    out.write("return ");
                    let vec: Vec<_> = self
                        .fields
                        .iter()
                        .skip(skip_fields)
                        .map(|field| format!("{} {} {}.{}", field.name, $op, other, field.name))
                        .collect();
                    out.write_vertical_source_list(
                        &vec[..],
                        ListType::Join(&format!(" {}", $conjuc)),
                    );
                    out.write(";");
                    out.close_brace(false);
                }};
            }

            if config.structure.derive_eq(&self.annotations) && self.can_derive_eq() {
                emit_op!("eq", "==", "&&");
            }
            if config.structure.derive_neq(&self.annotations) && self.can_derive_eq() {
                emit_op!("neq", "!=", "||");
            }
            if config.structure.derive_lt(&self.annotations)
                && self.fields.len() == 1
                && self.fields[0].ty.can_cmp_order()
            {
                emit_op!("lt", "<", "&&");
            }
            if config.structure.derive_lte(&self.annotations)
                && self.fields.len() == 1
                && self.fields[0].ty.can_cmp_order()
            {
                emit_op!("lte", "<=", "&&");
            }
            if config.structure.derive_gt(&self.annotations)
                && self.fields.len() == 1
                && self.fields[0].ty.can_cmp_order()
            {
                emit_op!("gt", ">", "&&");
            }
            if config.structure.derive_gte(&self.annotations)
                && self.fields.len() == 1
                && self.fields[0].ty.can_cmp_order()
            {
                emit_op!("gte", ">=", "&&");
            }
        }

        // Emit the post_body section, if relevant
        if let Some(body) = config.export.post_body(&self.path) {
            out.new_line();
            out.write_raw_block(body);
        }

        if config.language == Language::Cxx
            && config.structure.associated_constants_in_body
            && config.constant.allow_static_const
        {
            for constant in &self.associated_constants {
                out.new_line();
                constant.write_declaration(config, out, self);
            }
        }

        if config.language == Language::C && config.style.generate_typedef() {
            out.close_brace(false);
            write!(out, " {};", self.export_name());
        } else {
            out.close_brace(true);
        }

        for constant in &self.associated_constants {
            out.new_line();
            constant.write(config, out, Some(self));
        }

        condition.write_after(config, out);
    }
}
