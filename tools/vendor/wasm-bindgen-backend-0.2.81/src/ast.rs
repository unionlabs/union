//! A representation of the Abstract Syntax Tree of a Rust program,
//! with all the added metadata necessary to generate WASM bindings
//! for it.

use crate::Diagnostic;
use proc_macro2::{Ident, Span};
use std::hash::{Hash, Hasher};
use syn;
use wasm_bindgen_shared as shared;

/// An abstract syntax tree representing a rust program. Contains
/// extra information for joining up this rust code with javascript.
#[cfg_attr(feature = "extra-traits", derive(Debug))]
#[derive(Default, Clone)]
pub struct Program {
    /// rust -> js interfaces
    pub exports: Vec<Export>,
    /// js -> rust interfaces
    pub imports: Vec<Import>,
    /// rust enums
    pub enums: Vec<Enum>,
    /// rust structs
    pub structs: Vec<Struct>,
    /// custom typescript sections to be included in the definition file
    pub typescript_custom_sections: Vec<String>,
    /// Inline JS snippets
    pub inline_js: Vec<String>,
}

impl Program {
    /// Returns true if the Program is empty
    pub fn is_empty(&self) -> bool {
        self.exports.is_empty()
            && self.imports.is_empty()
            && self.enums.is_empty()
            && self.structs.is_empty()
            && self.typescript_custom_sections.is_empty()
            && self.inline_js.is_empty()
    }
}

/// A rust to js interface. Allows interaction with rust objects/functions
/// from javascript.
#[cfg_attr(feature = "extra-traits", derive(Debug))]
#[derive(Clone)]
pub struct Export {
    /// Comments extracted from the rust source.
    pub comments: Vec<String>,
    /// The rust function
    pub function: Function,
    /// The class name in JS this is attached to
    pub js_class: Option<String>,
    /// The kind (static, named, regular)
    pub method_kind: MethodKind,
    /// The type of `self` (either `self`, `&self`, or `&mut self`)
    pub method_self: Option<MethodSelf>,
    /// The struct name, in Rust, this is attached to
    pub rust_class: Option<Ident>,
    /// The name of the rust function/method on the rust side.
    pub rust_name: Ident,
    /// Whether or not this function should be flagged as the wasm start
    /// function.
    pub start: bool,
}

/// The 3 types variations of `self`.
#[cfg_attr(feature = "extra-traits", derive(Debug, PartialEq, Eq))]
#[derive(Clone)]
pub enum MethodSelf {
    /// `self`
    ByValue,
    /// `&mut self`
    RefMutable,
    /// `&self`
    RefShared,
}

/// Things imported from a JS module (in an `extern` block)
#[cfg_attr(feature = "extra-traits", derive(Debug))]
#[derive(Clone)]
pub struct Import {
    /// The type of module being imported from
    pub module: ImportModule,
    /// The namespace to access the item through, if any
    pub js_namespace: Option<Vec<String>>,
    /// The type of item being imported
    pub kind: ImportKind,
}

/// The possible types of module to import from
#[cfg_attr(feature = "extra-traits", derive(Debug))]
#[derive(Clone)]
pub enum ImportModule {
    /// No module / import from global scope
    None,
    /// Import from the named module, with relative paths interpreted
    Named(String, Span),
    /// Import from the named module, without interpreting paths
    RawNamed(String, Span),
    /// Import from an inline JS snippet
    Inline(usize, Span),
}

impl Hash for ImportModule {
    fn hash<H: Hasher>(&self, h: &mut H) {
        match self {
            ImportModule::None => {
                0u8.hash(h);
            }
            ImportModule::Named(name, _) => {
                1u8.hash(h);
                name.hash(h);
            }
            ImportModule::Inline(idx, _) => {
                2u8.hash(h);
                idx.hash(h);
            }
            ImportModule::RawNamed(name, _) => {
                3u8.hash(h);
                name.hash(h);
            }
        }
    }
}

/// The type of item being imported
#[cfg_attr(feature = "extra-traits", derive(Debug))]
#[derive(Clone)]
pub enum ImportKind {
    /// Importing a function
    Function(ImportFunction),
    /// Importing a static value
    Static(ImportStatic),
    /// Importing a type/class
    Type(ImportType),
    /// Importing a JS enum
    Enum(ImportEnum),
}

/// A function being imported from JS
#[cfg_attr(feature = "extra-traits", derive(Debug))]
#[derive(Clone)]
pub struct ImportFunction {
    /// The full signature of the function
    pub function: Function,
    /// The name rust code will use
    pub rust_name: Ident,
    /// The type being returned
    pub js_ret: Option<syn::Type>,
    /// Whether to catch JS exceptions
    pub catch: bool,
    /// Whether the function is variadic on the JS side
    pub variadic: bool,
    /// Whether the function should use structural type checking
    pub structural: bool,
    /// Causes the Builder (See cli-support::js::binding::Builder) to error out if
    /// it finds itself generating code for a function with this signature
    pub assert_no_shim: bool,
    /// The kind of function being imported
    pub kind: ImportFunctionKind,
    /// The shim name to use in the generated code. The 'shim' is a function that appears in
    /// the generated JS as a wrapper around the actual function to import, performing any
    /// necessary conversions (EG adding a try/catch to change a thrown error into a Result)
    pub shim: Ident,
    /// The doc comment on this import, if one is provided
    pub doc_comment: Option<String>,
}

/// The type of a function being imported
#[cfg_attr(feature = "extra-traits", derive(Debug, PartialEq, Eq))]
#[derive(Clone)]
pub enum ImportFunctionKind {
    /// A class method
    Method {
        /// The name of the class for this method, in JS
        class: String,
        /// The type of the class for this method, in Rust
        ty: syn::Type,
        /// The kind of method this is
        kind: MethodKind,
    },
    /// A standard function
    Normal,
}

/// The type of a method
#[cfg_attr(feature = "extra-traits", derive(Debug, PartialEq, Eq))]
#[derive(Clone)]
pub enum MethodKind {
    /// A class constructor
    Constructor,
    /// Any other kind of method
    Operation(Operation),
}

/// The operation performed by a class method
#[cfg_attr(feature = "extra-traits", derive(Debug, PartialEq, Eq))]
#[derive(Clone)]
pub struct Operation {
    /// Whether this method is static
    pub is_static: bool,
    /// The internal kind of this Operation
    pub kind: OperationKind,
}

/// The kind of operation performed by a method
#[cfg_attr(feature = "extra-traits", derive(Debug, PartialEq, Eq))]
#[derive(Clone)]
pub enum OperationKind {
    /// A standard method, nothing special
    Regular,
    /// A method for getting the value of the provided Ident
    Getter(Option<Ident>),
    /// A method for setting the value of the provided Ident
    Setter(Option<Ident>),
    /// A dynamically intercepted getter
    IndexingGetter,
    /// A dynamically intercepted setter
    IndexingSetter,
    /// A dynamically intercepted deleter
    IndexingDeleter,
}

/// The type of a static being imported
#[cfg_attr(feature = "extra-traits", derive(Debug, PartialEq, Eq))]
#[derive(Clone)]
pub struct ImportStatic {
    /// The visibility of this static in Rust
    pub vis: syn::Visibility,
    /// The type of static being imported
    pub ty: syn::Type,
    /// The name of the shim function used to access this static
    pub shim: Ident,
    /// The name of this static on the Rust side
    pub rust_name: Ident,
    /// The name of this static on the JS side
    pub js_name: String,
}

/// The metadata for a type being imported
#[cfg_attr(feature = "extra-traits", derive(Debug, PartialEq, Eq))]
#[derive(Clone)]
pub struct ImportType {
    /// The visibility of this type in Rust
    pub vis: syn::Visibility,
    /// The name of this type on the Rust side
    pub rust_name: Ident,
    /// The name of this type on the JS side
    pub js_name: String,
    /// The custom attributes to apply to this type
    pub attrs: Vec<syn::Attribute>,
    /// The TS definition to generate for this type
    pub typescript_type: Option<String>,
    /// The doc comment applied to this type, if one exists
    pub doc_comment: Option<String>,
    /// The name of the shim to check instanceof for this type
    pub instanceof_shim: String,
    /// The name of the remote function to use for the generated is_type_of
    pub is_type_of: Option<syn::Expr>,
    /// The list of classes this extends, if any
    pub extends: Vec<syn::Path>,
    /// A custom prefix to add and attempt to fall back to, if the type isn't found
    pub vendor_prefixes: Vec<Ident>,
    /// If present, don't generate a `Deref` impl
    pub no_deref: bool,
}

/// The metadata for an Enum being imported
#[cfg_attr(feature = "extra-traits", derive(Debug, PartialEq, Eq))]
#[derive(Clone)]
pub struct ImportEnum {
    /// The Rust enum's visibility
    pub vis: syn::Visibility,
    /// The Rust enum's identifiers
    pub name: Ident,
    /// The Rust identifiers for the variants
    pub variants: Vec<Ident>,
    /// The JS string values of the variants
    pub variant_values: Vec<String>,
    /// Attributes to apply to the Rust enum
    pub rust_attrs: Vec<syn::Attribute>,
}

/// Information about a function being imported or exported
#[cfg_attr(feature = "extra-traits", derive(Debug))]
#[derive(Clone)]
pub struct Function {
    /// The name of the function
    pub name: String,
    /// The span of the function's name in Rust code
    pub name_span: Span,
    /// Whether the function has a js_name attribute
    pub renamed_via_js_name: bool,
    /// The arguments to the function
    pub arguments: Vec<syn::PatType>,
    /// The return type of the function, if provided
    pub ret: Option<syn::Type>,
    /// Any custom attributes being applied to the function
    pub rust_attrs: Vec<syn::Attribute>,
    /// The visibility of this function in Rust
    pub rust_vis: syn::Visibility,
    /// Whether this is an `async` function
    pub r#async: bool,
    /// Whether to generate a typescript definition for this function
    pub generate_typescript: bool,
}

/// Information about a Struct being exported
#[cfg_attr(feature = "extra-traits", derive(Debug, PartialEq, Eq))]
#[derive(Clone)]
pub struct Struct {
    /// The name of the struct in Rust code
    pub rust_name: Ident,
    /// The name of the struct in JS code
    pub js_name: String,
    /// All the fields of this struct to export
    pub fields: Vec<StructField>,
    /// The doc comments on this struct, if provided
    pub comments: Vec<String>,
    /// Whether this struct is inspectable (provides toJSON/toString properties to JS)
    pub is_inspectable: bool,
    /// Whether to generate a typescript definition for this struct
    pub generate_typescript: bool,
}

/// The field of a struct
#[cfg_attr(feature = "extra-traits", derive(Debug, PartialEq, Eq))]
#[derive(Clone)]
pub struct StructField {
    /// The name of the field in Rust code
    pub rust_name: syn::Member,
    /// The name of the field in JS code
    pub js_name: String,
    /// The name of the struct this field is part of
    pub struct_name: Ident,
    /// Whether this value is read-only to JS
    pub readonly: bool,
    /// The type of this field
    pub ty: syn::Type,
    /// The name of the getter shim for this field
    pub getter: Ident,
    /// The name of the setter shim for this field
    pub setter: Ident,
    /// The doc comments on this field, if any
    pub comments: Vec<String>,
    /// Whether to generate a typescript definition for this field
    pub generate_typescript: bool,
    /// Whether to use .clone() in the auto-generated getter for this field
    pub getter_with_clone: bool,
}

/// Information about an Enum being exported
#[cfg_attr(feature = "extra-traits", derive(Debug, PartialEq, Eq))]
#[derive(Clone)]
pub struct Enum {
    /// The name of this enum in Rust code
    pub rust_name: Ident,
    /// The name of this enum in JS code
    pub js_name: String,
    /// The variants provided by this enum
    pub variants: Vec<Variant>,
    /// The doc comments on this enum, if any
    pub comments: Vec<String>,
    /// The value to use for a `none` variant of the enum
    pub hole: u32,
    /// Whether to generate a typescript definition for this enum
    pub generate_typescript: bool,
}

/// The variant of an enum
#[cfg_attr(feature = "extra-traits", derive(Debug, PartialEq, Eq))]
#[derive(Clone)]
pub struct Variant {
    /// The name of this variant
    pub name: Ident,
    /// The backing value of this variant
    pub value: u32,
    /// The doc comments on this variant, if any
    pub comments: Vec<String>,
}

/// Unused, the type of an argument to / return from a function
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TypeKind {
    /// A by-reference arg, EG `&T`
    ByRef,
    /// A by-mutable-reference arg, EG `&mut T`
    ByMutRef,
    /// A by-value arg, EG `T`
    ByValue,
}

/// Unused, the location of a type for a function argument (import/export, argument/ret)
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TypeLocation {
    /// An imported argument (JS side type)
    ImportArgument,
    /// An imported return
    ImportRet,
    /// An exported argument (Rust side type)
    ExportArgument,
    /// An exported return
    ExportRet,
}

impl Export {
    /// Mangles a rust -> javascript export, so that the created Ident will be unique over function
    /// name and class name, if the function belongs to a javascript class.
    pub(crate) fn rust_symbol(&self) -> Ident {
        let mut generated_name = String::from("__wasm_bindgen_generated");
        if let Some(class) = &self.js_class {
            generated_name.push_str("_");
            generated_name.push_str(class);
        }
        generated_name.push_str("_");
        generated_name.push_str(&self.function.name.to_string());
        Ident::new(&generated_name, Span::call_site())
    }

    /// This is the name of the shim function that gets exported and takes the raw
    /// ABI form of its arguments and converts them back into their normal,
    /// "high level" form before calling the actual function.
    pub(crate) fn export_name(&self) -> String {
        let fn_name = self.function.name.to_string();
        match &self.js_class {
            Some(class) => shared::struct_function_export_name(class, &fn_name),
            None => shared::free_function_export_name(&fn_name),
        }
    }
}

impl ImportKind {
    /// Whether this type can be inside an `impl` block.
    pub fn fits_on_impl(&self) -> bool {
        match *self {
            ImportKind::Function(_) => true,
            ImportKind::Static(_) => false,
            ImportKind::Type(_) => false,
            ImportKind::Enum(_) => false,
        }
    }
}

impl Function {
    /// If the rust object has a `fn xxx(&self) -> MyType` method, get the name for a getter in
    /// javascript (in this case `xxx`, so you can write `val = obj.xxx`)
    pub fn infer_getter_property(&self) -> &str {
        &self.name
    }

    /// If the rust object has a `fn set_xxx(&mut self, MyType)` style method, get the name
    /// for a setter in javascript (in this case `xxx`, so you can write `obj.xxx = val`)
    pub fn infer_setter_property(&self) -> Result<String, Diagnostic> {
        let name = self.name.to_string();

        // Otherwise we infer names based on the Rust function name.
        if !name.starts_with("set_") {
            bail_span!(
                syn::token::Pub(self.name_span),
                "setters must start with `set_`, found: {}",
                name,
            );
        }
        Ok(name[4..].to_string())
    }
}
