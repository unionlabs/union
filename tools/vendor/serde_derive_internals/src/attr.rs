use internals::respan::respan;
use internals::symbol::*;
use internals::{ungroup, Ctxt};
use proc_macro2::{Spacing, Span, TokenStream, TokenTree};
use quote::ToTokens;
use std::borrow::Cow;
use std::collections::BTreeSet;
use syn;
use syn::parse::{self, Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::Ident;
use syn::Meta::{List, NameValue, Path};
use syn::NestedMeta::{Lit, Meta};

// This module handles parsing of `#[serde(...)]` attributes. The entrypoints
// are `attr::Container::from_ast`, `attr::Variant::from_ast`, and
// `attr::Field::from_ast`. Each returns an instance of the corresponding
// struct. Note that none of them return a Result. Unrecognized, malformed, or
// duplicated attributes result in a span_err but otherwise are ignored. The
// user will see errors simultaneously for all bad attributes in the crate
// rather than just the first.

pub use internals::case::RenameRule;

struct Attr<'c, T> {
    cx: &'c Ctxt,
    name: Symbol,
    tokens: TokenStream,
    value: Option<T>,
}

impl<'c, T> Attr<'c, T> {
    fn none(cx: &'c Ctxt, name: Symbol) -> Self {
        Attr {
            cx,
            name,
            tokens: TokenStream::new(),
            value: None,
        }
    }

    fn set<A: ToTokens>(&mut self, obj: A, value: T) {
        let tokens = obj.into_token_stream();

        if self.value.is_some() {
            self.cx
                .error_spanned_by(tokens, format!("duplicate serde attribute `{}`", self.name));
        } else {
            self.tokens = tokens;
            self.value = Some(value);
        }
    }

    fn set_opt<A: ToTokens>(&mut self, obj: A, value: Option<T>) {
        if let Some(value) = value {
            self.set(obj, value);
        }
    }

    fn set_if_none(&mut self, value: T) {
        if self.value.is_none() {
            self.value = Some(value);
        }
    }

    fn get(self) -> Option<T> {
        self.value
    }

    fn get_with_tokens(self) -> Option<(TokenStream, T)> {
        match self.value {
            Some(v) => Some((self.tokens, v)),
            None => None,
        }
    }
}

struct BoolAttr<'c>(Attr<'c, ()>);

impl<'c> BoolAttr<'c> {
    fn none(cx: &'c Ctxt, name: Symbol) -> Self {
        BoolAttr(Attr::none(cx, name))
    }

    fn set_true<A: ToTokens>(&mut self, obj: A) {
        self.0.set(obj, ());
    }

    fn get(&self) -> bool {
        self.0.value.is_some()
    }
}

struct VecAttr<'c, T> {
    cx: &'c Ctxt,
    name: Symbol,
    first_dup_tokens: TokenStream,
    values: Vec<T>,
}

impl<'c, T> VecAttr<'c, T> {
    fn none(cx: &'c Ctxt, name: Symbol) -> Self {
        VecAttr {
            cx,
            name,
            first_dup_tokens: TokenStream::new(),
            values: Vec::new(),
        }
    }

    fn insert<A: ToTokens>(&mut self, obj: A, value: T) {
        if self.values.len() == 1 {
            self.first_dup_tokens = obj.into_token_stream();
        }
        self.values.push(value);
    }

    fn at_most_one(mut self) -> Result<Option<T>, ()> {
        if self.values.len() > 1 {
            let dup_token = self.first_dup_tokens;
            self.cx.error_spanned_by(
                dup_token,
                format!("duplicate serde attribute `{}`", self.name),
            );
            Err(())
        } else {
            Ok(self.values.pop())
        }
    }

    fn get(self) -> Vec<T> {
        self.values
    }
}

pub struct Name {
    serialize: String,
    serialize_renamed: bool,
    deserialize: String,
    deserialize_renamed: bool,
    deserialize_aliases: Vec<String>,
}

#[allow(deprecated)]
fn unraw(ident: &Ident) -> String {
    // str::trim_start_matches was added in 1.30, trim_left_matches deprecated
    // in 1.33. We currently support rustc back to 1.15 so we need to continue
    // to use the deprecated one.
    ident.to_string().trim_left_matches("r#").to_owned()
}

impl Name {
    fn from_attrs(
        source_name: String,
        ser_name: Attr<String>,
        de_name: Attr<String>,
        de_aliases: Option<VecAttr<String>>,
    ) -> Name {
        let deserialize_aliases = match de_aliases {
            Some(de_aliases) => {
                let mut alias_list = BTreeSet::new();
                for alias_name in de_aliases.get() {
                    alias_list.insert(alias_name);
                }
                alias_list.into_iter().collect()
            }
            None => Vec::new(),
        };

        let ser_name = ser_name.get();
        let ser_renamed = ser_name.is_some();
        let de_name = de_name.get();
        let de_renamed = de_name.is_some();
        Name {
            serialize: ser_name.unwrap_or_else(|| source_name.clone()),
            serialize_renamed: ser_renamed,
            deserialize: de_name.unwrap_or(source_name),
            deserialize_renamed: de_renamed,
            deserialize_aliases,
        }
    }

    /// Return the container name for the container when serializing.
    pub fn serialize_name(&self) -> String {
        self.serialize.clone()
    }

    /// Return the container name for the container when deserializing.
    pub fn deserialize_name(&self) -> String {
        self.deserialize.clone()
    }

    fn deserialize_aliases(&self) -> Vec<String> {
        let mut aliases = self.deserialize_aliases.clone();
        let main_name = self.deserialize_name();
        if !aliases.contains(&main_name) {
            aliases.push(main_name);
        }
        aliases
    }
}

pub struct RenameAllRules {
    serialize: RenameRule,
    deserialize: RenameRule,
}

/// Represents struct or enum attribute information.
pub struct Container {
    name: Name,
    transparent: bool,
    deny_unknown_fields: bool,
    default: Default,
    rename_all_rules: RenameAllRules,
    ser_bound: Option<Vec<syn::WherePredicate>>,
    de_bound: Option<Vec<syn::WherePredicate>>,
    tag: TagType,
    type_from: Option<syn::Type>,
    type_try_from: Option<syn::Type>,
    type_into: Option<syn::Type>,
    remote: Option<syn::Path>,
    identifier: Identifier,
    has_flatten: bool,
    serde_path: Option<syn::Path>,
    is_packed: bool,
    /// Error message generated when type can't be deserialized
    expecting: Option<String>,
}

/// Styles of representing an enum.
pub enum TagType {
    /// The default.
    ///
    /// ```json
    /// {"variant1": {"key1": "value1", "key2": "value2"}}
    /// ```
    External,

    /// `#[serde(tag = "type")]`
    ///
    /// ```json
    /// {"type": "variant1", "key1": "value1", "key2": "value2"}
    /// ```
    Internal { tag: String },

    /// `#[serde(tag = "t", content = "c")]`
    ///
    /// ```json
    /// {"t": "variant1", "c": {"key1": "value1", "key2": "value2"}}
    /// ```
    Adjacent { tag: String, content: String },

    /// `#[serde(untagged)]`
    ///
    /// ```json
    /// {"key1": "value1", "key2": "value2"}
    /// ```
    None,
}

/// Whether this enum represents the fields of a struct or the variants of an
/// enum.
#[derive(Copy, Clone)]
pub enum Identifier {
    /// It does not.
    No,

    /// This enum represents the fields of a struct. All of the variants must be
    /// unit variants, except possibly one which is annotated with
    /// `#[serde(other)]` and is a newtype variant.
    Field,

    /// This enum represents the variants of an enum. All of the variants must
    /// be unit variants.
    Variant,
}

impl Identifier {
    #[cfg(feature = "deserialize_in_place")]
    pub fn is_some(self) -> bool {
        match self {
            Identifier::No => false,
            Identifier::Field | Identifier::Variant => true,
        }
    }
}

impl Container {
    /// Extract out the `#[serde(...)]` attributes from an item.
    pub fn from_ast(cx: &Ctxt, item: &syn::DeriveInput) -> Self {
        let mut ser_name = Attr::none(cx, RENAME);
        let mut de_name = Attr::none(cx, RENAME);
        let mut transparent = BoolAttr::none(cx, TRANSPARENT);
        let mut deny_unknown_fields = BoolAttr::none(cx, DENY_UNKNOWN_FIELDS);
        let mut default = Attr::none(cx, DEFAULT);
        let mut rename_all_ser_rule = Attr::none(cx, RENAME_ALL);
        let mut rename_all_de_rule = Attr::none(cx, RENAME_ALL);
        let mut ser_bound = Attr::none(cx, BOUND);
        let mut de_bound = Attr::none(cx, BOUND);
        let mut untagged = BoolAttr::none(cx, UNTAGGED);
        let mut internal_tag = Attr::none(cx, TAG);
        let mut content = Attr::none(cx, CONTENT);
        let mut type_from = Attr::none(cx, FROM);
        let mut type_try_from = Attr::none(cx, TRY_FROM);
        let mut type_into = Attr::none(cx, INTO);
        let mut remote = Attr::none(cx, REMOTE);
        let mut field_identifier = BoolAttr::none(cx, FIELD_IDENTIFIER);
        let mut variant_identifier = BoolAttr::none(cx, VARIANT_IDENTIFIER);
        let mut serde_path = Attr::none(cx, CRATE);
        let mut expecting = Attr::none(cx, EXPECTING);

        for meta_item in item
            .attrs
            .iter()
            .flat_map(|attr| get_serde_meta_items(cx, attr))
            .flatten()
        {
            match &meta_item {
                // Parse `#[serde(rename = "foo")]`
                Meta(NameValue(m)) if m.path == RENAME => {
                    if let Ok(s) = get_lit_str(cx, RENAME, &m.lit) {
                        ser_name.set(&m.path, s.value());
                        de_name.set(&m.path, s.value());
                    }
                }

                // Parse `#[serde(rename(serialize = "foo", deserialize = "bar"))]`
                Meta(List(m)) if m.path == RENAME => {
                    if let Ok((ser, de)) = get_renames(cx, &m.nested) {
                        ser_name.set_opt(&m.path, ser.map(syn::LitStr::value));
                        de_name.set_opt(&m.path, de.map(syn::LitStr::value));
                    }
                }

                // Parse `#[serde(rename_all = "foo")]`
                Meta(NameValue(m)) if m.path == RENAME_ALL => {
                    if let Ok(s) = get_lit_str(cx, RENAME_ALL, &m.lit) {
                        match RenameRule::from_str(&s.value()) {
                            Ok(rename_rule) => {
                                rename_all_ser_rule.set(&m.path, rename_rule);
                                rename_all_de_rule.set(&m.path, rename_rule);
                            }
                            Err(err) => cx.error_spanned_by(s, err),
                        }
                    }
                }

                // Parse `#[serde(rename_all(serialize = "foo", deserialize = "bar"))]`
                Meta(List(m)) if m.path == RENAME_ALL => {
                    if let Ok((ser, de)) = get_renames(cx, &m.nested) {
                        if let Some(ser) = ser {
                            match RenameRule::from_str(&ser.value()) {
                                Ok(rename_rule) => rename_all_ser_rule.set(&m.path, rename_rule),
                                Err(err) => cx.error_spanned_by(ser, err),
                            }
                        }
                        if let Some(de) = de {
                            match RenameRule::from_str(&de.value()) {
                                Ok(rename_rule) => rename_all_de_rule.set(&m.path, rename_rule),
                                Err(err) => cx.error_spanned_by(de, err),
                            }
                        }
                    }
                }

                // Parse `#[serde(transparent)]`
                Meta(Path(word)) if word == TRANSPARENT => {
                    transparent.set_true(word);
                }

                // Parse `#[serde(deny_unknown_fields)]`
                Meta(Path(word)) if word == DENY_UNKNOWN_FIELDS => {
                    deny_unknown_fields.set_true(word);
                }

                // Parse `#[serde(default)]`
                Meta(Path(word)) if word == DEFAULT => match &item.data {
                    syn::Data::Struct(syn::DataStruct { fields, .. }) => match fields {
                        syn::Fields::Named(_) => {
                            default.set(word, Default::Default);
                        }
                        syn::Fields::Unnamed(_) | syn::Fields::Unit => cx.error_spanned_by(
                            fields,
                            "#[serde(default)] can only be used on structs with named fields",
                        ),
                    },
                    syn::Data::Enum(syn::DataEnum { enum_token, .. }) => cx.error_spanned_by(
                        enum_token,
                        "#[serde(default)] can only be used on structs with named fields",
                    ),
                    syn::Data::Union(syn::DataUnion { union_token, .. }) => cx.error_spanned_by(
                        union_token,
                        "#[serde(default)] can only be used on structs with named fields",
                    ),
                },

                // Parse `#[serde(default = "...")]`
                Meta(NameValue(m)) if m.path == DEFAULT => {
                    if let Ok(path) = parse_lit_into_expr_path(cx, DEFAULT, &m.lit) {
                        match &item.data {
                            syn::Data::Struct(syn::DataStruct { fields, .. }) => {
                                match fields {
                                    syn::Fields::Named(_) => {
                                        default.set(&m.path, Default::Path(path));
                                    }
                                    syn::Fields::Unnamed(_) | syn::Fields::Unit => cx
                                        .error_spanned_by(
                                            fields,
                                            "#[serde(default = \"...\")] can only be used on structs with named fields",
                                        ),
                                }
                            }
                            syn::Data::Enum(syn::DataEnum { enum_token, .. }) => cx
                                .error_spanned_by(
                                    enum_token,
                                    "#[serde(default = \"...\")] can only be used on structs with named fields",
                                ),
                            syn::Data::Union(syn::DataUnion {
                                union_token, ..
                            }) => cx.error_spanned_by(
                                union_token,
                                "#[serde(default = \"...\")] can only be used on structs with named fields",
                            ),
                        }
                    }
                }

                // Parse `#[serde(bound = "T: SomeBound")]`
                Meta(NameValue(m)) if m.path == BOUND => {
                    if let Ok(where_predicates) = parse_lit_into_where(cx, BOUND, BOUND, &m.lit) {
                        ser_bound.set(&m.path, where_predicates.clone());
                        de_bound.set(&m.path, where_predicates);
                    }
                }

                // Parse `#[serde(bound(serialize = "...", deserialize = "..."))]`
                Meta(List(m)) if m.path == BOUND => {
                    if let Ok((ser, de)) = get_where_predicates(cx, &m.nested) {
                        ser_bound.set_opt(&m.path, ser);
                        de_bound.set_opt(&m.path, de);
                    }
                }

                // Parse `#[serde(untagged)]`
                Meta(Path(word)) if word == UNTAGGED => match item.data {
                    syn::Data::Enum(_) => {
                        untagged.set_true(word);
                    }
                    syn::Data::Struct(syn::DataStruct { struct_token, .. }) => {
                        cx.error_spanned_by(
                            struct_token,
                            "#[serde(untagged)] can only be used on enums",
                        );
                    }
                    syn::Data::Union(syn::DataUnion { union_token, .. }) => {
                        cx.error_spanned_by(
                            union_token,
                            "#[serde(untagged)] can only be used on enums",
                        );
                    }
                },

                // Parse `#[serde(tag = "type")]`
                Meta(NameValue(m)) if m.path == TAG => {
                    if let Ok(s) = get_lit_str(cx, TAG, &m.lit) {
                        match &item.data {
                            syn::Data::Enum(_) => {
                                internal_tag.set(&m.path, s.value());
                            }
                            syn::Data::Struct(syn::DataStruct { fields, .. }) => match fields {
                                syn::Fields::Named(_) => {
                                    internal_tag.set(&m.path, s.value());
                                }
                                syn::Fields::Unnamed(_) | syn::Fields::Unit => {
                                    cx.error_spanned_by(
                                            fields,
                                            "#[serde(tag = \"...\")] can only be used on enums and structs with named fields",
                                        );
                                }
                            },
                            syn::Data::Union(syn::DataUnion { union_token, .. }) => {
                                cx.error_spanned_by(
                                    union_token,
                                    "#[serde(tag = \"...\")] can only be used on enums and structs with named fields",
                                );
                            }
                        }
                    }
                }

                // Parse `#[serde(content = "c")]`
                Meta(NameValue(m)) if m.path == CONTENT => {
                    if let Ok(s) = get_lit_str(cx, CONTENT, &m.lit) {
                        match &item.data {
                            syn::Data::Enum(_) => {
                                content.set(&m.path, s.value());
                            }
                            syn::Data::Struct(syn::DataStruct { struct_token, .. }) => {
                                cx.error_spanned_by(
                                    struct_token,
                                    "#[serde(content = \"...\")] can only be used on enums",
                                );
                            }
                            syn::Data::Union(syn::DataUnion { union_token, .. }) => {
                                cx.error_spanned_by(
                                    union_token,
                                    "#[serde(content = \"...\")] can only be used on enums",
                                );
                            }
                        }
                    }
                }

                // Parse `#[serde(from = "Type")]
                Meta(NameValue(m)) if m.path == FROM => {
                    if let Ok(from_ty) = parse_lit_into_ty(cx, FROM, &m.lit) {
                        type_from.set_opt(&m.path, Some(from_ty));
                    }
                }

                // Parse `#[serde(try_from = "Type")]
                Meta(NameValue(m)) if m.path == TRY_FROM => {
                    if let Ok(try_from_ty) = parse_lit_into_ty(cx, TRY_FROM, &m.lit) {
                        type_try_from.set_opt(&m.path, Some(try_from_ty));
                    }
                }

                // Parse `#[serde(into = "Type")]
                Meta(NameValue(m)) if m.path == INTO => {
                    if let Ok(into_ty) = parse_lit_into_ty(cx, INTO, &m.lit) {
                        type_into.set_opt(&m.path, Some(into_ty));
                    }
                }

                // Parse `#[serde(remote = "...")]`
                Meta(NameValue(m)) if m.path == REMOTE => {
                    if let Ok(path) = parse_lit_into_path(cx, REMOTE, &m.lit) {
                        if is_primitive_path(&path, "Self") {
                            remote.set(&m.path, item.ident.clone().into());
                        } else {
                            remote.set(&m.path, path);
                        }
                    }
                }

                // Parse `#[serde(field_identifier)]`
                Meta(Path(word)) if word == FIELD_IDENTIFIER => {
                    field_identifier.set_true(word);
                }

                // Parse `#[serde(variant_identifier)]`
                Meta(Path(word)) if word == VARIANT_IDENTIFIER => {
                    variant_identifier.set_true(word);
                }

                // Parse `#[serde(crate = "foo")]`
                Meta(NameValue(m)) if m.path == CRATE => {
                    if let Ok(path) = parse_lit_into_path(cx, CRATE, &m.lit) {
                        serde_path.set(&m.path, path)
                    }
                }

                // Parse `#[serde(expecting = "a message")]`
                Meta(NameValue(m)) if m.path == EXPECTING => {
                    if let Ok(s) = get_lit_str(cx, EXPECTING, &m.lit) {
                        expecting.set(&m.path, s.value());
                    }
                }

                Meta(meta_item) => {
                    let path = meta_item
                        .path()
                        .into_token_stream()
                        .to_string()
                        .replace(' ', "");
                    cx.error_spanned_by(
                        meta_item.path(),
                        format!("unknown serde container attribute `{}`", path),
                    );
                }

                Lit(lit) => {
                    cx.error_spanned_by(lit, "unexpected literal in serde container attribute");
                }
            }
        }

        let mut is_packed = false;
        for attr in &item.attrs {
            if attr.path.is_ident("repr") {
                let _ = attr.parse_args_with(|input: ParseStream| {
                    while let Some(token) = input.parse()? {
                        if let TokenTree::Ident(ident) = token {
                            is_packed |= ident == "packed";
                        }
                    }
                    Ok(())
                });
            }
        }

        Container {
            name: Name::from_attrs(unraw(&item.ident), ser_name, de_name, None),
            transparent: transparent.get(),
            deny_unknown_fields: deny_unknown_fields.get(),
            default: default.get().unwrap_or(Default::None),
            rename_all_rules: RenameAllRules {
                serialize: rename_all_ser_rule.get().unwrap_or(RenameRule::None),
                deserialize: rename_all_de_rule.get().unwrap_or(RenameRule::None),
            },
            ser_bound: ser_bound.get(),
            de_bound: de_bound.get(),
            tag: decide_tag(cx, item, untagged, internal_tag, content),
            type_from: type_from.get(),
            type_try_from: type_try_from.get(),
            type_into: type_into.get(),
            remote: remote.get(),
            identifier: decide_identifier(cx, item, field_identifier, variant_identifier),
            has_flatten: false,
            serde_path: serde_path.get(),
            is_packed,
            expecting: expecting.get(),
        }
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn rename_all_rules(&self) -> &RenameAllRules {
        &self.rename_all_rules
    }

    pub fn transparent(&self) -> bool {
        self.transparent
    }

    pub fn deny_unknown_fields(&self) -> bool {
        self.deny_unknown_fields
    }

    pub fn default(&self) -> &Default {
        &self.default
    }

    pub fn ser_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.ser_bound.as_ref().map(|vec| &vec[..])
    }

    pub fn de_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.de_bound.as_ref().map(|vec| &vec[..])
    }

    pub fn tag(&self) -> &TagType {
        &self.tag
    }

    pub fn type_from(&self) -> Option<&syn::Type> {
        self.type_from.as_ref()
    }

    pub fn type_try_from(&self) -> Option<&syn::Type> {
        self.type_try_from.as_ref()
    }

    pub fn type_into(&self) -> Option<&syn::Type> {
        self.type_into.as_ref()
    }

    pub fn remote(&self) -> Option<&syn::Path> {
        self.remote.as_ref()
    }

    pub fn is_packed(&self) -> bool {
        self.is_packed
    }

    pub fn identifier(&self) -> Identifier {
        self.identifier
    }

    pub fn has_flatten(&self) -> bool {
        self.has_flatten
    }

    pub fn mark_has_flatten(&mut self) {
        self.has_flatten = true;
    }

    pub fn custom_serde_path(&self) -> Option<&syn::Path> {
        self.serde_path.as_ref()
    }

    pub fn serde_path(&self) -> Cow<syn::Path> {
        self.custom_serde_path()
            .map_or_else(|| Cow::Owned(parse_quote!(_serde)), Cow::Borrowed)
    }

    /// Error message generated when type can't be deserialized.
    /// If `None`, default message will be used
    pub fn expecting(&self) -> Option<&str> {
        self.expecting.as_ref().map(String::as_ref)
    }
}

fn decide_tag(
    cx: &Ctxt,
    item: &syn::DeriveInput,
    untagged: BoolAttr,
    internal_tag: Attr<String>,
    content: Attr<String>,
) -> TagType {
    match (
        untagged.0.get_with_tokens(),
        internal_tag.get_with_tokens(),
        content.get_with_tokens(),
    ) {
        (None, None, None) => TagType::External,
        (Some(_), None, None) => TagType::None,
        (None, Some((_, tag)), None) => {
            // Check that there are no tuple variants.
            if let syn::Data::Enum(data) = &item.data {
                for variant in &data.variants {
                    match &variant.fields {
                        syn::Fields::Named(_) | syn::Fields::Unit => {}
                        syn::Fields::Unnamed(fields) => {
                            if fields.unnamed.len() != 1 {
                                cx.error_spanned_by(
                                    variant,
                                    "#[serde(tag = \"...\")] cannot be used with tuple variants",
                                );
                                break;
                            }
                        }
                    }
                }
            }
            TagType::Internal { tag }
        }
        (Some((untagged_tokens, _)), Some((tag_tokens, _)), None) => {
            cx.error_spanned_by(
                untagged_tokens,
                "enum cannot be both untagged and internally tagged",
            );
            cx.error_spanned_by(
                tag_tokens,
                "enum cannot be both untagged and internally tagged",
            );
            TagType::External // doesn't matter, will error
        }
        (None, None, Some((content_tokens, _))) => {
            cx.error_spanned_by(
                content_tokens,
                "#[serde(tag = \"...\", content = \"...\")] must be used together",
            );
            TagType::External
        }
        (Some((untagged_tokens, _)), None, Some((content_tokens, _))) => {
            cx.error_spanned_by(
                untagged_tokens,
                "untagged enum cannot have #[serde(content = \"...\")]",
            );
            cx.error_spanned_by(
                content_tokens,
                "untagged enum cannot have #[serde(content = \"...\")]",
            );
            TagType::External
        }
        (None, Some((_, tag)), Some((_, content))) => TagType::Adjacent { tag, content },
        (Some((untagged_tokens, _)), Some((tag_tokens, _)), Some((content_tokens, _))) => {
            cx.error_spanned_by(
                untagged_tokens,
                "untagged enum cannot have #[serde(tag = \"...\", content = \"...\")]",
            );
            cx.error_spanned_by(
                tag_tokens,
                "untagged enum cannot have #[serde(tag = \"...\", content = \"...\")]",
            );
            cx.error_spanned_by(
                content_tokens,
                "untagged enum cannot have #[serde(tag = \"...\", content = \"...\")]",
            );
            TagType::External
        }
    }
}

fn decide_identifier(
    cx: &Ctxt,
    item: &syn::DeriveInput,
    field_identifier: BoolAttr,
    variant_identifier: BoolAttr,
) -> Identifier {
    match (
        &item.data,
        field_identifier.0.get_with_tokens(),
        variant_identifier.0.get_with_tokens(),
    ) {
        (_, None, None) => Identifier::No,
        (_, Some((field_identifier_tokens, _)), Some((variant_identifier_tokens, _))) => {
            cx.error_spanned_by(
                field_identifier_tokens,
                "#[serde(field_identifier)] and #[serde(variant_identifier)] cannot both be set",
            );
            cx.error_spanned_by(
                variant_identifier_tokens,
                "#[serde(field_identifier)] and #[serde(variant_identifier)] cannot both be set",
            );
            Identifier::No
        }
        (syn::Data::Enum(_), Some(_), None) => Identifier::Field,
        (syn::Data::Enum(_), None, Some(_)) => Identifier::Variant,
        (syn::Data::Struct(syn::DataStruct { struct_token, .. }), Some(_), None) => {
            cx.error_spanned_by(
                struct_token,
                "#[serde(field_identifier)] can only be used on an enum",
            );
            Identifier::No
        }
        (syn::Data::Union(syn::DataUnion { union_token, .. }), Some(_), None) => {
            cx.error_spanned_by(
                union_token,
                "#[serde(field_identifier)] can only be used on an enum",
            );
            Identifier::No
        }
        (syn::Data::Struct(syn::DataStruct { struct_token, .. }), None, Some(_)) => {
            cx.error_spanned_by(
                struct_token,
                "#[serde(variant_identifier)] can only be used on an enum",
            );
            Identifier::No
        }
        (syn::Data::Union(syn::DataUnion { union_token, .. }), None, Some(_)) => {
            cx.error_spanned_by(
                union_token,
                "#[serde(variant_identifier)] can only be used on an enum",
            );
            Identifier::No
        }
    }
}

/// Represents variant attribute information
pub struct Variant {
    name: Name,
    rename_all_rules: RenameAllRules,
    ser_bound: Option<Vec<syn::WherePredicate>>,
    de_bound: Option<Vec<syn::WherePredicate>>,
    skip_deserializing: bool,
    skip_serializing: bool,
    other: bool,
    serialize_with: Option<syn::ExprPath>,
    deserialize_with: Option<syn::ExprPath>,
    borrow: Option<syn::Meta>,
}

impl Variant {
    pub fn from_ast(cx: &Ctxt, variant: &syn::Variant) -> Self {
        let mut ser_name = Attr::none(cx, RENAME);
        let mut de_name = Attr::none(cx, RENAME);
        let mut de_aliases = VecAttr::none(cx, RENAME);
        let mut skip_deserializing = BoolAttr::none(cx, SKIP_DESERIALIZING);
        let mut skip_serializing = BoolAttr::none(cx, SKIP_SERIALIZING);
        let mut rename_all_ser_rule = Attr::none(cx, RENAME_ALL);
        let mut rename_all_de_rule = Attr::none(cx, RENAME_ALL);
        let mut ser_bound = Attr::none(cx, BOUND);
        let mut de_bound = Attr::none(cx, BOUND);
        let mut other = BoolAttr::none(cx, OTHER);
        let mut serialize_with = Attr::none(cx, SERIALIZE_WITH);
        let mut deserialize_with = Attr::none(cx, DESERIALIZE_WITH);
        let mut borrow = Attr::none(cx, BORROW);

        for meta_item in variant
            .attrs
            .iter()
            .flat_map(|attr| get_serde_meta_items(cx, attr))
            .flatten()
        {
            match &meta_item {
                // Parse `#[serde(rename = "foo")]`
                Meta(NameValue(m)) if m.path == RENAME => {
                    if let Ok(s) = get_lit_str(cx, RENAME, &m.lit) {
                        ser_name.set(&m.path, s.value());
                        de_name.set_if_none(s.value());
                        de_aliases.insert(&m.path, s.value());
                    }
                }

                // Parse `#[serde(rename(serialize = "foo", deserialize = "bar"))]`
                Meta(List(m)) if m.path == RENAME => {
                    if let Ok((ser, de)) = get_multiple_renames(cx, &m.nested) {
                        ser_name.set_opt(&m.path, ser.map(syn::LitStr::value));
                        for de_value in de {
                            de_name.set_if_none(de_value.value());
                            de_aliases.insert(&m.path, de_value.value());
                        }
                    }
                }

                // Parse `#[serde(alias = "foo")]`
                Meta(NameValue(m)) if m.path == ALIAS => {
                    if let Ok(s) = get_lit_str(cx, ALIAS, &m.lit) {
                        de_aliases.insert(&m.path, s.value());
                    }
                }

                // Parse `#[serde(rename_all = "foo")]`
                Meta(NameValue(m)) if m.path == RENAME_ALL => {
                    if let Ok(s) = get_lit_str(cx, RENAME_ALL, &m.lit) {
                        match RenameRule::from_str(&s.value()) {
                            Ok(rename_rule) => {
                                rename_all_ser_rule.set(&m.path, rename_rule);
                                rename_all_de_rule.set(&m.path, rename_rule);
                            }
                            Err(err) => cx.error_spanned_by(s, err),
                        }
                    }
                }

                // Parse `#[serde(rename_all(serialize = "foo", deserialize = "bar"))]`
                Meta(List(m)) if m.path == RENAME_ALL => {
                    if let Ok((ser, de)) = get_renames(cx, &m.nested) {
                        if let Some(ser) = ser {
                            match RenameRule::from_str(&ser.value()) {
                                Ok(rename_rule) => rename_all_ser_rule.set(&m.path, rename_rule),
                                Err(err) => cx.error_spanned_by(ser, err),
                            }
                        }
                        if let Some(de) = de {
                            match RenameRule::from_str(&de.value()) {
                                Ok(rename_rule) => rename_all_de_rule.set(&m.path, rename_rule),
                                Err(err) => cx.error_spanned_by(de, err),
                            }
                        }
                    }
                }

                // Parse `#[serde(skip)]`
                Meta(Path(word)) if word == SKIP => {
                    skip_serializing.set_true(word);
                    skip_deserializing.set_true(word);
                }

                // Parse `#[serde(skip_deserializing)]`
                Meta(Path(word)) if word == SKIP_DESERIALIZING => {
                    skip_deserializing.set_true(word);
                }

                // Parse `#[serde(skip_serializing)]`
                Meta(Path(word)) if word == SKIP_SERIALIZING => {
                    skip_serializing.set_true(word);
                }

                // Parse `#[serde(other)]`
                Meta(Path(word)) if word == OTHER => {
                    other.set_true(word);
                }

                // Parse `#[serde(bound = "T: SomeBound")]`
                Meta(NameValue(m)) if m.path == BOUND => {
                    if let Ok(where_predicates) = parse_lit_into_where(cx, BOUND, BOUND, &m.lit) {
                        ser_bound.set(&m.path, where_predicates.clone());
                        de_bound.set(&m.path, where_predicates);
                    }
                }

                // Parse `#[serde(bound(serialize = "...", deserialize = "..."))]`
                Meta(List(m)) if m.path == BOUND => {
                    if let Ok((ser, de)) = get_where_predicates(cx, &m.nested) {
                        ser_bound.set_opt(&m.path, ser);
                        de_bound.set_opt(&m.path, de);
                    }
                }

                // Parse `#[serde(with = "...")]`
                Meta(NameValue(m)) if m.path == WITH => {
                    if let Ok(path) = parse_lit_into_expr_path(cx, WITH, &m.lit) {
                        let mut ser_path = path.clone();
                        ser_path
                            .path
                            .segments
                            .push(Ident::new("serialize", Span::call_site()).into());
                        serialize_with.set(&m.path, ser_path);
                        let mut de_path = path;
                        de_path
                            .path
                            .segments
                            .push(Ident::new("deserialize", Span::call_site()).into());
                        deserialize_with.set(&m.path, de_path);
                    }
                }

                // Parse `#[serde(serialize_with = "...")]`
                Meta(NameValue(m)) if m.path == SERIALIZE_WITH => {
                    if let Ok(path) = parse_lit_into_expr_path(cx, SERIALIZE_WITH, &m.lit) {
                        serialize_with.set(&m.path, path);
                    }
                }

                // Parse `#[serde(deserialize_with = "...")]`
                Meta(NameValue(m)) if m.path == DESERIALIZE_WITH => {
                    if let Ok(path) = parse_lit_into_expr_path(cx, DESERIALIZE_WITH, &m.lit) {
                        deserialize_with.set(&m.path, path);
                    }
                }

                // Defer `#[serde(borrow)]` and `#[serde(borrow = "'a + 'b")]`
                Meta(m) if m.path() == BORROW => match &variant.fields {
                    syn::Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                        borrow.set(m.path(), m.clone());
                    }
                    _ => {
                        cx.error_spanned_by(
                            variant,
                            "#[serde(borrow)] may only be used on newtype variants",
                        );
                    }
                },

                Meta(meta_item) => {
                    let path = meta_item
                        .path()
                        .into_token_stream()
                        .to_string()
                        .replace(' ', "");
                    cx.error_spanned_by(
                        meta_item.path(),
                        format!("unknown serde variant attribute `{}`", path),
                    );
                }

                Lit(lit) => {
                    cx.error_spanned_by(lit, "unexpected literal in serde variant attribute");
                }
            }
        }

        Variant {
            name: Name::from_attrs(unraw(&variant.ident), ser_name, de_name, Some(de_aliases)),
            rename_all_rules: RenameAllRules {
                serialize: rename_all_ser_rule.get().unwrap_or(RenameRule::None),
                deserialize: rename_all_de_rule.get().unwrap_or(RenameRule::None),
            },
            ser_bound: ser_bound.get(),
            de_bound: de_bound.get(),
            skip_deserializing: skip_deserializing.get(),
            skip_serializing: skip_serializing.get(),
            other: other.get(),
            serialize_with: serialize_with.get(),
            deserialize_with: deserialize_with.get(),
            borrow: borrow.get(),
        }
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn aliases(&self) -> Vec<String> {
        self.name.deserialize_aliases()
    }

    pub fn rename_by_rules(&mut self, rules: &RenameAllRules) {
        if !self.name.serialize_renamed {
            self.name.serialize = rules.serialize.apply_to_variant(&self.name.serialize);
        }
        if !self.name.deserialize_renamed {
            self.name.deserialize = rules.deserialize.apply_to_variant(&self.name.deserialize);
        }
    }

    pub fn rename_all_rules(&self) -> &RenameAllRules {
        &self.rename_all_rules
    }

    pub fn ser_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.ser_bound.as_ref().map(|vec| &vec[..])
    }

    pub fn de_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.de_bound.as_ref().map(|vec| &vec[..])
    }

    pub fn skip_deserializing(&self) -> bool {
        self.skip_deserializing
    }

    pub fn skip_serializing(&self) -> bool {
        self.skip_serializing
    }

    pub fn other(&self) -> bool {
        self.other
    }

    pub fn serialize_with(&self) -> Option<&syn::ExprPath> {
        self.serialize_with.as_ref()
    }

    pub fn deserialize_with(&self) -> Option<&syn::ExprPath> {
        self.deserialize_with.as_ref()
    }
}

/// Represents field attribute information
pub struct Field {
    name: Name,
    skip_serializing: bool,
    skip_deserializing: bool,
    skip_serializing_if: Option<syn::ExprPath>,
    default: Default,
    serialize_with: Option<syn::ExprPath>,
    deserialize_with: Option<syn::ExprPath>,
    ser_bound: Option<Vec<syn::WherePredicate>>,
    de_bound: Option<Vec<syn::WherePredicate>>,
    borrowed_lifetimes: BTreeSet<syn::Lifetime>,
    getter: Option<syn::ExprPath>,
    flatten: bool,
    transparent: bool,
}

/// Represents the default to use for a field when deserializing.
pub enum Default {
    /// Field must always be specified because it does not have a default.
    None,
    /// The default is given by `std::default::Default::default()`.
    Default,
    /// The default is given by this function.
    Path(syn::ExprPath),
}

impl Default {
    pub fn is_none(&self) -> bool {
        match self {
            Default::None => true,
            Default::Default | Default::Path(_) => false,
        }
    }
}

impl Field {
    /// Extract out the `#[serde(...)]` attributes from a struct field.
    pub fn from_ast(
        cx: &Ctxt,
        index: usize,
        field: &syn::Field,
        attrs: Option<&Variant>,
        container_default: &Default,
    ) -> Self {
        let mut ser_name = Attr::none(cx, RENAME);
        let mut de_name = Attr::none(cx, RENAME);
        let mut de_aliases = VecAttr::none(cx, RENAME);
        let mut skip_serializing = BoolAttr::none(cx, SKIP_SERIALIZING);
        let mut skip_deserializing = BoolAttr::none(cx, SKIP_DESERIALIZING);
        let mut skip_serializing_if = Attr::none(cx, SKIP_SERIALIZING_IF);
        let mut default = Attr::none(cx, DEFAULT);
        let mut serialize_with = Attr::none(cx, SERIALIZE_WITH);
        let mut deserialize_with = Attr::none(cx, DESERIALIZE_WITH);
        let mut ser_bound = Attr::none(cx, BOUND);
        let mut de_bound = Attr::none(cx, BOUND);
        let mut borrowed_lifetimes = Attr::none(cx, BORROW);
        let mut getter = Attr::none(cx, GETTER);
        let mut flatten = BoolAttr::none(cx, FLATTEN);

        let ident = match &field.ident {
            Some(ident) => unraw(ident),
            None => index.to_string(),
        };

        let variant_borrow = attrs
            .and_then(|variant| variant.borrow.as_ref())
            .map(|borrow| Meta(borrow.clone()));

        for meta_item in field
            .attrs
            .iter()
            .flat_map(|attr| get_serde_meta_items(cx, attr))
            .flatten()
            .chain(variant_borrow)
        {
            match &meta_item {
                // Parse `#[serde(rename = "foo")]`
                Meta(NameValue(m)) if m.path == RENAME => {
                    if let Ok(s) = get_lit_str(cx, RENAME, &m.lit) {
                        ser_name.set(&m.path, s.value());
                        de_name.set_if_none(s.value());
                        de_aliases.insert(&m.path, s.value());
                    }
                }

                // Parse `#[serde(rename(serialize = "foo", deserialize = "bar"))]`
                Meta(List(m)) if m.path == RENAME => {
                    if let Ok((ser, de)) = get_multiple_renames(cx, &m.nested) {
                        ser_name.set_opt(&m.path, ser.map(syn::LitStr::value));
                        for de_value in de {
                            de_name.set_if_none(de_value.value());
                            de_aliases.insert(&m.path, de_value.value());
                        }
                    }
                }

                // Parse `#[serde(alias = "foo")]`
                Meta(NameValue(m)) if m.path == ALIAS => {
                    if let Ok(s) = get_lit_str(cx, ALIAS, &m.lit) {
                        de_aliases.insert(&m.path, s.value());
                    }
                }

                // Parse `#[serde(default)]`
                Meta(Path(word)) if word == DEFAULT => {
                    default.set(word, Default::Default);
                }

                // Parse `#[serde(default = "...")]`
                Meta(NameValue(m)) if m.path == DEFAULT => {
                    if let Ok(path) = parse_lit_into_expr_path(cx, DEFAULT, &m.lit) {
                        default.set(&m.path, Default::Path(path));
                    }
                }

                // Parse `#[serde(skip_serializing)]`
                Meta(Path(word)) if word == SKIP_SERIALIZING => {
                    skip_serializing.set_true(word);
                }

                // Parse `#[serde(skip_deserializing)]`
                Meta(Path(word)) if word == SKIP_DESERIALIZING => {
                    skip_deserializing.set_true(word);
                }

                // Parse `#[serde(skip)]`
                Meta(Path(word)) if word == SKIP => {
                    skip_serializing.set_true(word);
                    skip_deserializing.set_true(word);
                }

                // Parse `#[serde(skip_serializing_if = "...")]`
                Meta(NameValue(m)) if m.path == SKIP_SERIALIZING_IF => {
                    if let Ok(path) = parse_lit_into_expr_path(cx, SKIP_SERIALIZING_IF, &m.lit) {
                        skip_serializing_if.set(&m.path, path);
                    }
                }

                // Parse `#[serde(serialize_with = "...")]`
                Meta(NameValue(m)) if m.path == SERIALIZE_WITH => {
                    if let Ok(path) = parse_lit_into_expr_path(cx, SERIALIZE_WITH, &m.lit) {
                        serialize_with.set(&m.path, path);
                    }
                }

                // Parse `#[serde(deserialize_with = "...")]`
                Meta(NameValue(m)) if m.path == DESERIALIZE_WITH => {
                    if let Ok(path) = parse_lit_into_expr_path(cx, DESERIALIZE_WITH, &m.lit) {
                        deserialize_with.set(&m.path, path);
                    }
                }

                // Parse `#[serde(with = "...")]`
                Meta(NameValue(m)) if m.path == WITH => {
                    if let Ok(path) = parse_lit_into_expr_path(cx, WITH, &m.lit) {
                        let mut ser_path = path.clone();
                        ser_path
                            .path
                            .segments
                            .push(Ident::new("serialize", Span::call_site()).into());
                        serialize_with.set(&m.path, ser_path);
                        let mut de_path = path;
                        de_path
                            .path
                            .segments
                            .push(Ident::new("deserialize", Span::call_site()).into());
                        deserialize_with.set(&m.path, de_path);
                    }
                }

                // Parse `#[serde(bound = "T: SomeBound")]`
                Meta(NameValue(m)) if m.path == BOUND => {
                    if let Ok(where_predicates) = parse_lit_into_where(cx, BOUND, BOUND, &m.lit) {
                        ser_bound.set(&m.path, where_predicates.clone());
                        de_bound.set(&m.path, where_predicates);
                    }
                }

                // Parse `#[serde(bound(serialize = "...", deserialize = "..."))]`
                Meta(List(m)) if m.path == BOUND => {
                    if let Ok((ser, de)) = get_where_predicates(cx, &m.nested) {
                        ser_bound.set_opt(&m.path, ser);
                        de_bound.set_opt(&m.path, de);
                    }
                }

                // Parse `#[serde(borrow)]`
                Meta(Path(word)) if word == BORROW => {
                    if let Ok(borrowable) = borrowable_lifetimes(cx, &ident, field) {
                        borrowed_lifetimes.set(word, borrowable);
                    }
                }

                // Parse `#[serde(borrow = "'a + 'b")]`
                Meta(NameValue(m)) if m.path == BORROW => {
                    if let Ok(lifetimes) = parse_lit_into_lifetimes(cx, BORROW, &m.lit) {
                        if let Ok(borrowable) = borrowable_lifetimes(cx, &ident, field) {
                            for lifetime in &lifetimes {
                                if !borrowable.contains(lifetime) {
                                    cx.error_spanned_by(
                                        field,
                                        format!(
                                            "field `{}` does not have lifetime {}",
                                            ident, lifetime
                                        ),
                                    );
                                }
                            }
                            borrowed_lifetimes.set(&m.path, lifetimes);
                        }
                    }
                }

                // Parse `#[serde(getter = "...")]`
                Meta(NameValue(m)) if m.path == GETTER => {
                    if let Ok(path) = parse_lit_into_expr_path(cx, GETTER, &m.lit) {
                        getter.set(&m.path, path);
                    }
                }

                // Parse `#[serde(flatten)]`
                Meta(Path(word)) if word == FLATTEN => {
                    flatten.set_true(word);
                }

                Meta(meta_item) => {
                    let path = meta_item
                        .path()
                        .into_token_stream()
                        .to_string()
                        .replace(' ', "");
                    cx.error_spanned_by(
                        meta_item.path(),
                        format!("unknown serde field attribute `{}`", path),
                    );
                }

                Lit(lit) => {
                    cx.error_spanned_by(lit, "unexpected literal in serde field attribute");
                }
            }
        }

        // Is skip_deserializing, initialize the field to Default::default() unless a
        // different default is specified by `#[serde(default = "...")]` on
        // ourselves or our container (e.g. the struct we are in).
        if let Default::None = *container_default {
            if skip_deserializing.0.value.is_some() {
                default.set_if_none(Default::Default);
            }
        }

        let mut borrowed_lifetimes = borrowed_lifetimes.get().unwrap_or_default();
        if !borrowed_lifetimes.is_empty() {
            // Cow<str> and Cow<[u8]> never borrow by default:
            //
            //     impl<'de, 'a, T: ?Sized> Deserialize<'de> for Cow<'a, T>
            //
            // A #[serde(borrow)] attribute enables borrowing that corresponds
            // roughly to these impls:
            //
            //     impl<'de: 'a, 'a> Deserialize<'de> for Cow<'a, str>
            //     impl<'de: 'a, 'a> Deserialize<'de> for Cow<'a, [u8]>
            if is_cow(&field.ty, is_str) {
                let mut path = syn::Path {
                    leading_colon: None,
                    segments: Punctuated::new(),
                };
                let span = Span::call_site();
                path.segments.push(Ident::new("_serde", span).into());
                path.segments.push(Ident::new("__private", span).into());
                path.segments.push(Ident::new("de", span).into());
                path.segments
                    .push(Ident::new("borrow_cow_str", span).into());
                let expr = syn::ExprPath {
                    attrs: Vec::new(),
                    qself: None,
                    path,
                };
                deserialize_with.set_if_none(expr);
            } else if is_cow(&field.ty, is_slice_u8) {
                let mut path = syn::Path {
                    leading_colon: None,
                    segments: Punctuated::new(),
                };
                let span = Span::call_site();
                path.segments.push(Ident::new("_serde", span).into());
                path.segments.push(Ident::new("__private", span).into());
                path.segments.push(Ident::new("de", span).into());
                path.segments
                    .push(Ident::new("borrow_cow_bytes", span).into());
                let expr = syn::ExprPath {
                    attrs: Vec::new(),
                    qself: None,
                    path,
                };
                deserialize_with.set_if_none(expr);
            }
        } else if is_implicitly_borrowed(&field.ty) {
            // Types &str and &[u8] are always implicitly borrowed. No need for
            // a #[serde(borrow)].
            collect_lifetimes(&field.ty, &mut borrowed_lifetimes);
        }

        Field {
            name: Name::from_attrs(ident, ser_name, de_name, Some(de_aliases)),
            skip_serializing: skip_serializing.get(),
            skip_deserializing: skip_deserializing.get(),
            skip_serializing_if: skip_serializing_if.get(),
            default: default.get().unwrap_or(Default::None),
            serialize_with: serialize_with.get(),
            deserialize_with: deserialize_with.get(),
            ser_bound: ser_bound.get(),
            de_bound: de_bound.get(),
            borrowed_lifetimes,
            getter: getter.get(),
            flatten: flatten.get(),
            transparent: false,
        }
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn aliases(&self) -> Vec<String> {
        self.name.deserialize_aliases()
    }

    pub fn rename_by_rules(&mut self, rules: &RenameAllRules) {
        if !self.name.serialize_renamed {
            self.name.serialize = rules.serialize.apply_to_field(&self.name.serialize);
        }
        if !self.name.deserialize_renamed {
            self.name.deserialize = rules.deserialize.apply_to_field(&self.name.deserialize);
        }
    }

    pub fn skip_serializing(&self) -> bool {
        self.skip_serializing
    }

    pub fn skip_deserializing(&self) -> bool {
        self.skip_deserializing
    }

    pub fn skip_serializing_if(&self) -> Option<&syn::ExprPath> {
        self.skip_serializing_if.as_ref()
    }

    pub fn default(&self) -> &Default {
        &self.default
    }

    pub fn serialize_with(&self) -> Option<&syn::ExprPath> {
        self.serialize_with.as_ref()
    }

    pub fn deserialize_with(&self) -> Option<&syn::ExprPath> {
        self.deserialize_with.as_ref()
    }

    pub fn ser_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.ser_bound.as_ref().map(|vec| &vec[..])
    }

    pub fn de_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.de_bound.as_ref().map(|vec| &vec[..])
    }

    pub fn borrowed_lifetimes(&self) -> &BTreeSet<syn::Lifetime> {
        &self.borrowed_lifetimes
    }

    pub fn getter(&self) -> Option<&syn::ExprPath> {
        self.getter.as_ref()
    }

    pub fn flatten(&self) -> bool {
        self.flatten
    }

    pub fn transparent(&self) -> bool {
        self.transparent
    }

    pub fn mark_transparent(&mut self) {
        self.transparent = true;
    }
}

type SerAndDe<T> = (Option<T>, Option<T>);

fn get_ser_and_de<'a, 'b, T, F>(
    cx: &'b Ctxt,
    attr_name: Symbol,
    metas: &'a Punctuated<syn::NestedMeta, Token![,]>,
    f: F,
) -> Result<(VecAttr<'b, T>, VecAttr<'b, T>), ()>
where
    T: 'a,
    F: Fn(&Ctxt, Symbol, Symbol, &'a syn::Lit) -> Result<T, ()>,
{
    let mut ser_meta = VecAttr::none(cx, attr_name);
    let mut de_meta = VecAttr::none(cx, attr_name);

    for meta in metas {
        match meta {
            Meta(NameValue(meta)) if meta.path == SERIALIZE => {
                if let Ok(v) = f(cx, attr_name, SERIALIZE, &meta.lit) {
                    ser_meta.insert(&meta.path, v);
                }
            }

            Meta(NameValue(meta)) if meta.path == DESERIALIZE => {
                if let Ok(v) = f(cx, attr_name, DESERIALIZE, &meta.lit) {
                    de_meta.insert(&meta.path, v);
                }
            }

            _ => {
                cx.error_spanned_by(
                    meta,
                    format!(
                        "malformed {0} attribute, expected `{0}(serialize = ..., deserialize = ...)`",
                        attr_name
                    ),
                );
                return Err(());
            }
        }
    }

    Ok((ser_meta, de_meta))
}

fn get_renames<'a>(
    cx: &Ctxt,
    items: &'a Punctuated<syn::NestedMeta, Token![,]>,
) -> Result<SerAndDe<&'a syn::LitStr>, ()> {
    let (ser, de) = get_ser_and_de(cx, RENAME, items, get_lit_str2)?;
    Ok((ser.at_most_one()?, de.at_most_one()?))
}

fn get_multiple_renames<'a>(
    cx: &Ctxt,
    items: &'a Punctuated<syn::NestedMeta, Token![,]>,
) -> Result<(Option<&'a syn::LitStr>, Vec<&'a syn::LitStr>), ()> {
    let (ser, de) = get_ser_and_de(cx, RENAME, items, get_lit_str2)?;
    Ok((ser.at_most_one()?, de.get()))
}

fn get_where_predicates(
    cx: &Ctxt,
    items: &Punctuated<syn::NestedMeta, Token![,]>,
) -> Result<SerAndDe<Vec<syn::WherePredicate>>, ()> {
    let (ser, de) = get_ser_and_de(cx, BOUND, items, parse_lit_into_where)?;
    Ok((ser.at_most_one()?, de.at_most_one()?))
}

pub fn get_serde_meta_items(cx: &Ctxt, attr: &syn::Attribute) -> Result<Vec<syn::NestedMeta>, ()> {
    if attr.path != SERDE {
        return Ok(Vec::new());
    }

    match attr.parse_meta() {
        Ok(List(meta)) => Ok(meta.nested.into_iter().collect()),
        Ok(other) => {
            cx.error_spanned_by(other, "expected #[serde(...)]");
            Err(())
        }
        Err(err) => {
            cx.syn_error(err);
            Err(())
        }
    }
}

fn get_lit_str<'a>(cx: &Ctxt, attr_name: Symbol, lit: &'a syn::Lit) -> Result<&'a syn::LitStr, ()> {
    get_lit_str2(cx, attr_name, attr_name, lit)
}

fn get_lit_str2<'a>(
    cx: &Ctxt,
    attr_name: Symbol,
    meta_item_name: Symbol,
    lit: &'a syn::Lit,
) -> Result<&'a syn::LitStr, ()> {
    if let syn::Lit::Str(lit) = lit {
        Ok(lit)
    } else {
        cx.error_spanned_by(
            lit,
            format!(
                "expected serde {} attribute to be a string: `{} = \"...\"`",
                attr_name, meta_item_name
            ),
        );
        Err(())
    }
}

fn parse_lit_into_path(cx: &Ctxt, attr_name: Symbol, lit: &syn::Lit) -> Result<syn::Path, ()> {
    let string = get_lit_str(cx, attr_name, lit)?;
    parse_lit_str(string).map_err(|_| {
        cx.error_spanned_by(lit, format!("failed to parse path: {:?}", string.value()))
    })
}

fn parse_lit_into_expr_path(
    cx: &Ctxt,
    attr_name: Symbol,
    lit: &syn::Lit,
) -> Result<syn::ExprPath, ()> {
    let string = get_lit_str(cx, attr_name, lit)?;
    parse_lit_str(string).map_err(|_| {
        cx.error_spanned_by(lit, format!("failed to parse path: {:?}", string.value()))
    })
}

fn parse_lit_into_where(
    cx: &Ctxt,
    attr_name: Symbol,
    meta_item_name: Symbol,
    lit: &syn::Lit,
) -> Result<Vec<syn::WherePredicate>, ()> {
    let string = get_lit_str2(cx, attr_name, meta_item_name, lit)?;
    if string.value().is_empty() {
        return Ok(Vec::new());
    }

    let where_string = syn::LitStr::new(&format!("where {}", string.value()), string.span());

    parse_lit_str::<syn::WhereClause>(&where_string)
        .map(|wh| wh.predicates.into_iter().collect())
        .map_err(|err| cx.error_spanned_by(lit, err))
}

fn parse_lit_into_ty(cx: &Ctxt, attr_name: Symbol, lit: &syn::Lit) -> Result<syn::Type, ()> {
    let string = get_lit_str(cx, attr_name, lit)?;

    parse_lit_str(string).map_err(|_| {
        cx.error_spanned_by(
            lit,
            format!("failed to parse type: {} = {:?}", attr_name, string.value()),
        )
    })
}

// Parses a string literal like "'a + 'b + 'c" containing a nonempty list of
// lifetimes separated by `+`.
fn parse_lit_into_lifetimes(
    cx: &Ctxt,
    attr_name: Symbol,
    lit: &syn::Lit,
) -> Result<BTreeSet<syn::Lifetime>, ()> {
    let string = get_lit_str(cx, attr_name, lit)?;
    if string.value().is_empty() {
        cx.error_spanned_by(lit, "at least one lifetime must be borrowed");
        return Err(());
    }

    struct BorrowedLifetimes(Punctuated<syn::Lifetime, Token![+]>);

    impl Parse for BorrowedLifetimes {
        fn parse(input: ParseStream) -> parse::Result<Self> {
            Punctuated::parse_separated_nonempty(input).map(BorrowedLifetimes)
        }
    }

    if let Ok(BorrowedLifetimes(lifetimes)) = parse_lit_str(string) {
        let mut set = BTreeSet::new();
        for lifetime in lifetimes {
            if !set.insert(lifetime.clone()) {
                cx.error_spanned_by(lit, format!("duplicate borrowed lifetime `{}`", lifetime));
            }
        }
        return Ok(set);
    }

    cx.error_spanned_by(
        lit,
        format!("failed to parse borrowed lifetimes: {:?}", string.value()),
    );
    Err(())
}

fn is_implicitly_borrowed(ty: &syn::Type) -> bool {
    is_implicitly_borrowed_reference(ty) || is_option(ty, is_implicitly_borrowed_reference)
}

fn is_implicitly_borrowed_reference(ty: &syn::Type) -> bool {
    is_reference(ty, is_str) || is_reference(ty, is_slice_u8)
}

// Whether the type looks like it might be `std::borrow::Cow<T>` where elem="T".
// This can have false negatives and false positives.
//
// False negative:
//
//     use std::borrow::Cow as Pig;
//
//     #[derive(Deserialize)]
//     struct S<'a> {
//         #[serde(borrow)]
//         pig: Pig<'a, str>,
//     }
//
// False positive:
//
//     type str = [i16];
//
//     #[derive(Deserialize)]
//     struct S<'a> {
//         #[serde(borrow)]
//         cow: Cow<'a, str>,
//     }
fn is_cow(ty: &syn::Type, elem: fn(&syn::Type) -> bool) -> bool {
    let path = match ungroup(ty) {
        syn::Type::Path(ty) => &ty.path,
        _ => {
            return false;
        }
    };
    let seg = match path.segments.last() {
        Some(seg) => seg,
        None => {
            return false;
        }
    };
    let args = match &seg.arguments {
        syn::PathArguments::AngleBracketed(bracketed) => &bracketed.args,
        _ => {
            return false;
        }
    };
    seg.ident == "Cow"
        && args.len() == 2
        && match (&args[0], &args[1]) {
            (syn::GenericArgument::Lifetime(_), syn::GenericArgument::Type(arg)) => elem(arg),
            _ => false,
        }
}

fn is_option(ty: &syn::Type, elem: fn(&syn::Type) -> bool) -> bool {
    let path = match ungroup(ty) {
        syn::Type::Path(ty) => &ty.path,
        _ => {
            return false;
        }
    };
    let seg = match path.segments.last() {
        Some(seg) => seg,
        None => {
            return false;
        }
    };
    let args = match &seg.arguments {
        syn::PathArguments::AngleBracketed(bracketed) => &bracketed.args,
        _ => {
            return false;
        }
    };
    seg.ident == "Option"
        && args.len() == 1
        && match &args[0] {
            syn::GenericArgument::Type(arg) => elem(arg),
            _ => false,
        }
}

// Whether the type looks like it might be `&T` where elem="T". This can have
// false negatives and false positives.
//
// False negative:
//
//     type Yarn = str;
//
//     #[derive(Deserialize)]
//     struct S<'a> {
//         r: &'a Yarn,
//     }
//
// False positive:
//
//     type str = [i16];
//
//     #[derive(Deserialize)]
//     struct S<'a> {
//         r: &'a str,
//     }
fn is_reference(ty: &syn::Type, elem: fn(&syn::Type) -> bool) -> bool {
    match ungroup(ty) {
        syn::Type::Reference(ty) => ty.mutability.is_none() && elem(&ty.elem),
        _ => false,
    }
}

fn is_str(ty: &syn::Type) -> bool {
    is_primitive_type(ty, "str")
}

fn is_slice_u8(ty: &syn::Type) -> bool {
    match ungroup(ty) {
        syn::Type::Slice(ty) => is_primitive_type(&ty.elem, "u8"),
        _ => false,
    }
}

fn is_primitive_type(ty: &syn::Type, primitive: &str) -> bool {
    match ungroup(ty) {
        syn::Type::Path(ty) => ty.qself.is_none() && is_primitive_path(&ty.path, primitive),
        _ => false,
    }
}

fn is_primitive_path(path: &syn::Path, primitive: &str) -> bool {
    path.leading_colon.is_none()
        && path.segments.len() == 1
        && path.segments[0].ident == primitive
        && path.segments[0].arguments.is_empty()
}

// All lifetimes that this type could borrow from a Deserializer.
//
// For example a type `S<'a, 'b>` could borrow `'a` and `'b`. On the other hand
// a type `for<'a> fn(&'a str)` could not borrow `'a` from the Deserializer.
//
// This is used when there is an explicit or implicit `#[serde(borrow)]`
// attribute on the field so there must be at least one borrowable lifetime.
fn borrowable_lifetimes(
    cx: &Ctxt,
    name: &str,
    field: &syn::Field,
) -> Result<BTreeSet<syn::Lifetime>, ()> {
    let mut lifetimes = BTreeSet::new();
    collect_lifetimes(&field.ty, &mut lifetimes);
    if lifetimes.is_empty() {
        cx.error_spanned_by(
            field,
            format!("field `{}` has no lifetimes to borrow", name),
        );
        Err(())
    } else {
        Ok(lifetimes)
    }
}

fn collect_lifetimes(ty: &syn::Type, out: &mut BTreeSet<syn::Lifetime>) {
    match ty {
        syn::Type::Slice(ty) => {
            collect_lifetimes(&ty.elem, out);
        }
        syn::Type::Array(ty) => {
            collect_lifetimes(&ty.elem, out);
        }
        syn::Type::Ptr(ty) => {
            collect_lifetimes(&ty.elem, out);
        }
        syn::Type::Reference(ty) => {
            out.extend(ty.lifetime.iter().cloned());
            collect_lifetimes(&ty.elem, out);
        }
        syn::Type::Tuple(ty) => {
            for elem in &ty.elems {
                collect_lifetimes(elem, out);
            }
        }
        syn::Type::Path(ty) => {
            if let Some(qself) = &ty.qself {
                collect_lifetimes(&qself.ty, out);
            }
            for seg in &ty.path.segments {
                if let syn::PathArguments::AngleBracketed(bracketed) = &seg.arguments {
                    for arg in &bracketed.args {
                        match arg {
                            syn::GenericArgument::Lifetime(lifetime) => {
                                out.insert(lifetime.clone());
                            }
                            syn::GenericArgument::Type(ty) => {
                                collect_lifetimes(ty, out);
                            }
                            syn::GenericArgument::Binding(binding) => {
                                collect_lifetimes(&binding.ty, out);
                            }
                            syn::GenericArgument::Constraint(_)
                            | syn::GenericArgument::Const(_) => {}
                        }
                    }
                }
            }
        }
        syn::Type::Paren(ty) => {
            collect_lifetimes(&ty.elem, out);
        }
        syn::Type::Group(ty) => {
            collect_lifetimes(&ty.elem, out);
        }
        syn::Type::Macro(ty) => {
            collect_lifetimes_from_tokens(ty.mac.tokens.clone(), out);
        }
        syn::Type::BareFn(_)
        | syn::Type::Never(_)
        | syn::Type::TraitObject(_)
        | syn::Type::ImplTrait(_)
        | syn::Type::Infer(_)
        | syn::Type::Verbatim(_) => {}

        #[cfg(test)]
        syn::Type::__TestExhaustive(_) => unimplemented!(),
        #[cfg(not(test))]
        _ => {}
    }
}

fn collect_lifetimes_from_tokens(tokens: TokenStream, out: &mut BTreeSet<syn::Lifetime>) {
    let mut iter = tokens.into_iter();
    while let Some(tt) = iter.next() {
        match &tt {
            TokenTree::Punct(op) if op.as_char() == '\'' && op.spacing() == Spacing::Joint => {
                if let Some(TokenTree::Ident(ident)) = iter.next() {
                    out.insert(syn::Lifetime {
                        apostrophe: op.span(),
                        ident,
                    });
                }
            }
            TokenTree::Group(group) => {
                let tokens = group.stream();
                collect_lifetimes_from_tokens(tokens, out);
            }
            _ => {}
        }
    }
}

fn parse_lit_str<T>(s: &syn::LitStr) -> parse::Result<T>
where
    T: Parse,
{
    let tokens = spanned_tokens(s)?;
    syn::parse2(tokens)
}

fn spanned_tokens(s: &syn::LitStr) -> parse::Result<TokenStream> {
    let stream = syn::parse_str(&s.value())?;
    Ok(respan(stream, s.span()))
}
