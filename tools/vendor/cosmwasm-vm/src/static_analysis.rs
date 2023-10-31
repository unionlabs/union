use parity_wasm::elements::{deserialize_buffer, Internal, Module};
use std::collections::HashSet;

use crate::errors::{VmError, VmResult};

pub const REQUIRED_IBC_EXPORTS: &[&str] = &[
    "ibc_channel_open",
    "ibc_channel_connect",
    "ibc_channel_close",
    "ibc_packet_receive",
    "ibc_packet_ack",
    "ibc_packet_timeout",
];

pub fn deserialize_wasm(wasm_code: &[u8]) -> VmResult<Module> {
    deserialize_buffer(wasm_code).map_err(|err| {
        VmError::static_validation_err(format!(
            "Wasm bytecode could not be deserialized. Deserialization error: \"{}\"",
            err
        ))
    })
}

/// A trait that allows accessing shared functionality of `parity_wasm::elements::Module`
/// and `wasmer::Module` in a shared fashion.
pub trait ExportInfo {
    /// Returns all exported function names with the given prefix
    fn exported_function_names(&self, prefix: Option<&str>) -> HashSet<String>;
}

impl ExportInfo for Module {
    fn exported_function_names(&self, prefix: Option<&str>) -> HashSet<String> {
        self.export_section()
            .map_or(HashSet::default(), |export_section| {
                export_section
                    .entries()
                    .iter()
                    .filter_map(|entry| match entry.internal() {
                        Internal::Function(_) => Some(entry.field()),
                        _ => None,
                    })
                    .filter(|name| {
                        if let Some(required_prefix) = prefix {
                            name.starts_with(required_prefix)
                        } else {
                            true
                        }
                    })
                    .map(|name| name.to_string())
                    .collect()
            })
    }
}

impl ExportInfo for wasmer::Module {
    fn exported_function_names(&self, prefix: Option<&str>) -> HashSet<String> {
        self.exports()
            .functions()
            .filter_map(|function_export| {
                let name = function_export.name();
                if let Some(required_prefix) = prefix {
                    if name.starts_with(required_prefix) {
                        Some(name.to_string())
                    } else {
                        None
                    }
                } else {
                    Some(name.to_string())
                }
            })
            .collect()
    }
}

/// Returns true if and only if all IBC entry points ([`REQUIRED_IBC_EXPORTS`])
/// exist as exported functions. This does not guarantee the entry points
/// are functional and for simplicity does not even check their signatures.
pub fn has_ibc_entry_points(module: &impl ExportInfo) -> bool {
    let available_exports = module.exported_function_names(None);
    REQUIRED_IBC_EXPORTS
        .iter()
        .all(|required| available_exports.contains(*required))
}

#[cfg(test)]
mod tests {
    use super::*;
    use parity_wasm::elements::Internal;
    use wasmer::{Cranelift, Store, Universal};

    static CONTRACT: &[u8] = include_bytes!("../testdata/hackatom.wasm");
    static CORRUPTED: &[u8] = include_bytes!("../testdata/corrupted.wasm");

    #[test]
    fn deserialize_wasm_works() {
        let module = deserialize_wasm(CONTRACT).unwrap();
        assert_eq!(module.version(), 1);

        let exported_functions = module
            .export_section()
            .unwrap()
            .entries()
            .iter()
            .filter(|entry| matches!(entry.internal(), Internal::Function(_)));
        assert_eq!(exported_functions.count(), 8); // 4 required exports plus "execute", "migrate", "query" and "sudo"

        let exported_memories = module
            .export_section()
            .unwrap()
            .entries()
            .iter()
            .filter(|entry| matches!(entry.internal(), Internal::Memory(_)));
        assert_eq!(exported_memories.count(), 1);
    }

    #[test]
    fn deserialize_wasm_corrupted_data() {
        match deserialize_wasm(CORRUPTED).unwrap_err() {
            VmError::StaticValidationErr { msg, .. } => {
                assert!(msg.starts_with("Wasm bytecode could not be deserialized."))
            }
            err => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    fn exported_function_names_works_for_parity_with_no_prefix() {
        let wasm = wat::parse_str(r#"(module)"#).unwrap();
        let module = deserialize_wasm(&wasm).unwrap();
        let exports = module.exported_function_names(None);
        assert_eq!(exports, HashSet::new());

        let wasm = wat::parse_str(
            r#"(module
                (memory 3)
                (export "memory" (memory 0))

                (type (func))
                (func (type 0) nop)
                (export "foo" (func 0))
                (export "bar" (func 0))
            )"#,
        )
        .unwrap();
        let module = deserialize_wasm(&wasm).unwrap();
        let exports = module.exported_function_names(None);
        assert_eq!(
            exports,
            HashSet::from_iter(vec!["foo".to_string(), "bar".to_string()])
        );
    }

    #[test]
    fn exported_function_names_works_for_parity_with_prefix() {
        let wasm = wat::parse_str(r#"(module)"#).unwrap();
        let module = deserialize_wasm(&wasm).unwrap();
        let exports = module.exported_function_names(Some("b"));
        assert_eq!(exports, HashSet::new());

        let wasm = wat::parse_str(
            r#"(module
                (memory 3)
                (export "memory" (memory 0))

                (type (func))
                (func (type 0) nop)
                (export "foo" (func 0))
                (export "bar" (func 0))
                (export "baz" (func 0))
            )"#,
        )
        .unwrap();
        let module = deserialize_wasm(&wasm).unwrap();
        let exports = module.exported_function_names(Some("b"));
        assert_eq!(
            exports,
            HashSet::from_iter(vec!["bar".to_string(), "baz".to_string()])
        );
    }

    #[test]
    fn exported_function_names_works_for_wasmer_with_no_prefix() {
        let wasm = wat::parse_str(r#"(module)"#).unwrap();
        let store = Store::new(&Universal::new(Cranelift::default()).engine());
        let module = wasmer::Module::new(&store, wasm).unwrap();
        let exports = module.exported_function_names(None);
        assert_eq!(exports, HashSet::new());

        let wasm = wat::parse_str(
            r#"(module
                (memory 3)
                (export "memory" (memory 0))

                (type (func))
                (func (type 0) nop)
                (export "foo" (func 0))
                (export "bar" (func 0))
            )"#,
        )
        .unwrap();
        let store = Store::new(&Universal::new(Cranelift::default()).engine());
        let module = wasmer::Module::new(&store, wasm).unwrap();
        let exports = module.exported_function_names(None);
        assert_eq!(
            exports,
            HashSet::from_iter(vec!["foo".to_string(), "bar".to_string()])
        );
    }

    #[test]
    fn exported_function_names_works_for_wasmer_with_prefix() {
        let wasm = wat::parse_str(r#"(module)"#).unwrap();
        let store = Store::new(&Universal::new(Cranelift::default()).engine());
        let module = wasmer::Module::new(&store, wasm).unwrap();
        let exports = module.exported_function_names(Some("b"));
        assert_eq!(exports, HashSet::new());

        let wasm = wat::parse_str(
            r#"(module
                (memory 3)
                (export "memory" (memory 0))

                (type (func))
                (func (type 0) nop)
                (export "foo" (func 0))
                (export "bar" (func 0))
                (export "baz" (func 0))
            )"#,
        )
        .unwrap();
        let store = Store::new(&Universal::new(Cranelift::default()).engine());
        let module = wasmer::Module::new(&store, wasm).unwrap();
        let exports = module.exported_function_names(Some("b"));
        assert_eq!(
            exports,
            HashSet::from_iter(vec!["bar".to_string(), "baz".to_string()])
        );
    }

    #[test]
    fn has_ibc_entry_points_works() {
        // Non-IBC contract
        let wasm = wat::parse_str(
            r#"(module
                (memory 3)
                (export "memory" (memory 0))

                (type (func))
                (func (type 0) nop)
                (export "interface_version_8" (func 0))
                (export "instantiate" (func 0))
                (export "allocate" (func 0))
                (export "deallocate" (func 0))
            )"#,
        )
        .unwrap();
        let module = deserialize_wasm(&wasm).unwrap();
        assert!(!has_ibc_entry_points(&module));

        // IBC contract
        let wasm = wat::parse_str(
            r#"(module
                (memory 3)
                (export "memory" (memory 0))

                (type (func))
                (func (type 0) nop)
                (export "interface_version_8" (func 0))
                (export "instantiate" (func 0))
                (export "execute" (func 0))
                (export "allocate" (func 0))
                (export "deallocate" (func 0))
                (export "ibc_channel_open" (func 0))
                (export "ibc_channel_connect" (func 0))
                (export "ibc_channel_close" (func 0))
                (export "ibc_packet_receive" (func 0))
                (export "ibc_packet_ack" (func 0))
                (export "ibc_packet_timeout" (func 0))
            )"#,
        )
        .unwrap();
        let module = deserialize_wasm(&wasm).unwrap();
        assert!(has_ibc_entry_points(&module));

        // Missing packet ack
        let wasm = wat::parse_str(
            r#"(module
                (memory 3)
                (export "memory" (memory 0))

                (type (func))
                (func (type 0) nop)
                (export "interface_version_8" (func 0))
                (export "instantiate" (func 0))
                (export "execute" (func 0))
                (export "allocate" (func 0))
                (export "deallocate" (func 0))
                (export "ibc_channel_open" (func 0))
                (export "ibc_channel_connect" (func 0))
                (export "ibc_channel_close" (func 0))
                (export "ibc_packet_receive" (func 0))
                (export "ibc_packet_timeout" (func 0))
            )"#,
        )
        .unwrap();
        let module = deserialize_wasm(&wasm).unwrap();
        assert!(!has_ibc_entry_points(&module));
    }
}
