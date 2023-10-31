#![doc(html_root_url = "https://docs.rs/wasm-bindgen-shared/0.2")]

#[cfg(test)]
mod schema_hash_approval;

// This gets changed whenever our schema changes.
// At this time versions of wasm-bindgen and wasm-bindgen-cli are required to have the exact same
// SCHEMA_VERSION in order to work together.
pub const SCHEMA_VERSION: &str = "0.2.87";

#[macro_export]
macro_rules! shared_api {
    ($mac:ident) => {
        $mac! {
        struct Program<'a> {
            exports: Vec<Export<'a>>,
            enums: Vec<Enum<'a>>,
            imports: Vec<Import<'a>>,
            structs: Vec<Struct<'a>>,
            typescript_custom_sections: Vec<&'a str>,
            local_modules: Vec<LocalModule<'a>>,
            inline_js: Vec<&'a str>,
            unique_crate_identifier: &'a str,
            package_json: Option<&'a str>,
            linked_modules: Vec<LinkedModule<'a>>,
        }

        struct Import<'a> {
            module: Option<ImportModule<'a>>,
            js_namespace: Option<Vec<String>>,
            kind: ImportKind<'a>,
        }

        struct LinkedModule<'a> {
            module: ImportModule<'a>,
            link_function_name: &'a str,
        }

        enum ImportModule<'a> {
            Named(&'a str),
            RawNamed(&'a str),
            Inline(u32),
        }

        enum ImportKind<'a> {
            Function(ImportFunction<'a>),
            Static(ImportStatic<'a>),
            Type(ImportType<'a>),
            Enum(ImportEnum),
        }

        struct ImportFunction<'a> {
            shim: &'a str,
            catch: bool,
            variadic: bool,
            assert_no_shim: bool,
            method: Option<MethodData<'a>>,
            structural: bool,
            function: Function<'a>,
        }

        struct MethodData<'a> {
            class: &'a str,
            kind: MethodKind<'a>,
        }

        enum MethodKind<'a> {
            Constructor,
            Operation(Operation<'a>),
        }

        struct Operation<'a> {
            is_static: bool,
            kind: OperationKind<'a>,
        }

        enum OperationKind<'a> {
            Regular,
            Getter(&'a str),
            Setter(&'a str),
            IndexingGetter,
            IndexingSetter,
            IndexingDeleter,
        }

        struct ImportStatic<'a> {
            name: &'a str,
            shim: &'a str,
        }

        struct ImportType<'a> {
            name: &'a str,
            instanceof_shim: &'a str,
            vendor_prefixes: Vec<&'a str>,
        }

        struct ImportEnum {}

        struct Export<'a> {
            class: Option<&'a str>,
            comments: Vec<&'a str>,
            consumed: bool,
            function: Function<'a>,
            method_kind: MethodKind<'a>,
            start: bool,
        }

        struct Enum<'a> {
            name: &'a str,
            variants: Vec<EnumVariant<'a>>,
            comments: Vec<&'a str>,
            generate_typescript: bool,
        }

        struct EnumVariant<'a> {
            name: &'a str,
            value: u32,
            comments: Vec<&'a str>,
        }

        struct Function<'a> {
            arg_names: Vec<String>,
            asyncness: bool,
            name: &'a str,
            generate_typescript: bool,
            generate_jsdoc: bool,
            variadic: bool,
        }

        struct Struct<'a> {
            name: &'a str,
            fields: Vec<StructField<'a>>,
            comments: Vec<&'a str>,
            is_inspectable: bool,
            generate_typescript: bool,
        }

        struct StructField<'a> {
            name: &'a str,
            readonly: bool,
            comments: Vec<&'a str>,
            generate_typescript: bool,
            generate_jsdoc: bool,
        }

        struct LocalModule<'a> {
            identifier: &'a str,
            contents: &'a str,
        }
        }
    }; // end of mac case
} // end of mac definition

pub fn new_function(struct_name: &str) -> String {
    let mut name = "__wbg_".to_string();
    name.extend(struct_name.chars().flat_map(|s| s.to_lowercase()));
    name.push_str("_new");
    name
}

pub fn free_function(struct_name: &str) -> String {
    let mut name = "__wbg_".to_string();
    name.extend(struct_name.chars().flat_map(|s| s.to_lowercase()));
    name.push_str("_free");
    name
}

pub fn free_function_export_name(function_name: &str) -> String {
    function_name.to_string()
}

pub fn struct_function_export_name(struct_: &str, f: &str) -> String {
    let mut name = struct_
        .chars()
        .flat_map(|s| s.to_lowercase())
        .collect::<String>();
    name.push('_');
    name.push_str(f);
    name
}

pub fn struct_field_get(struct_: &str, f: &str) -> String {
    let mut name = String::from("__wbg_get_");
    name.extend(struct_.chars().flat_map(|s| s.to_lowercase()));
    name.push('_');
    name.push_str(f);
    name
}

pub fn struct_field_set(struct_: &str, f: &str) -> String {
    let mut name = String::from("__wbg_set_");
    name.extend(struct_.chars().flat_map(|s| s.to_lowercase()));
    name.push('_');
    name.push_str(f);
    name
}

pub fn version() -> String {
    let mut v = env!("CARGO_PKG_VERSION").to_string();
    if let Some(s) = option_env!("WBG_VERSION") {
        v.push_str(" (");
        v.push_str(s);
        v.push(')');
    }
    v
}
