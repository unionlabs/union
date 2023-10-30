use std::collections::HashSet;

use crate::static_analysis::ExportInfo;

const REQUIRES_PREFIX: &str = "requires_";

/// Takes a comma-separated string, splits it by commas, removes empty elements and returns a set of capabilities.
/// This can be used e.g. to initialize the cache.
pub fn capabilities_from_csv(csv: &str) -> HashSet<String> {
    csv.split(',')
        .map(|x| x.trim().to_string())
        .filter(|f| !f.is_empty())
        .collect()
}

/// Implementation for check_wasm, based on static analysis of the bytecode.
/// This is used for code upload, to perform check before compiling the Wasm.
pub fn required_capabilities_from_module(module: &impl ExportInfo) -> HashSet<String> {
    module
        .exported_function_names(Some(REQUIRES_PREFIX))
        .into_iter()
        .filter_map(|name| {
            if name.len() > REQUIRES_PREFIX.len() {
                let (_, required_capability) = name.split_at(REQUIRES_PREFIX.len());
                Some(required_capability.to_string())
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::static_analysis::deserialize_wasm;

    #[test]
    fn capabilities_from_csv_works() {
        let set = capabilities_from_csv("foo, bar,baz ");
        assert_eq!(set.len(), 3);
        assert!(set.contains("foo"));
        assert!(set.contains("bar"));
        assert!(set.contains("baz"));
    }

    #[test]
    fn capabilities_from_csv_skips_empty() {
        let set = capabilities_from_csv("");
        assert_eq!(set.len(), 0);
        let set = capabilities_from_csv("a,,b");
        assert_eq!(set.len(), 2);
        assert!(set.contains("a"));
        assert!(set.contains("b"));
        let set = capabilities_from_csv("a,b,");
        assert_eq!(set.len(), 2);
        assert!(set.contains("a"));
        assert!(set.contains("b"));
    }

    #[test]
    fn required_capabilities_from_module_works() {
        let wasm = wat::parse_str(
            r#"(module
            (type (func))
            (func (type 0) nop)
            (export "requires_water" (func 0))
            (export "requires_" (func 0))
            (export "requires_nutrients" (func 0))
            (export "require_milk" (func 0))
            (export "REQUIRES_air" (func 0))
            (export "requires_sun" (func 0))
            )"#,
        )
        .unwrap();
        let module = deserialize_wasm(&wasm).unwrap();

        let required_capabilities = required_capabilities_from_module(&module);
        assert_eq!(required_capabilities.len(), 3);
        assert!(required_capabilities.contains("nutrients"));
        assert!(required_capabilities.contains("sun"));
        assert!(required_capabilities.contains("water"));
    }

    #[test]
    fn required_capabilities_from_module_works_without_exports_section() {
        let wasm = wat::parse_str(r#"(module)"#).unwrap();
        let module = deserialize_wasm(&wasm).unwrap();
        let required_capabilities = required_capabilities_from_module(&module);
        assert_eq!(required_capabilities.len(), 0);
    }
}
