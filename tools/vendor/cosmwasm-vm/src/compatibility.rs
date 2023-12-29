use std::collections::BTreeSet;
use std::collections::HashSet;

use wasmer::wasmparser::Import;
use wasmer::wasmparser::TypeRef;

use crate::capabilities::required_capabilities_from_module;
use crate::errors::{VmError, VmResult};
use crate::limited::LimitedDisplay;
use crate::parsed_wasm::ParsedWasm;
use crate::static_analysis::ExportInfo;

/// Lists all imports we provide upon instantiating the instance in Instance::from_module()
/// This should be updated when new imports are added
const SUPPORTED_IMPORTS: &[&str] = &[
    "env.abort",
    "env.db_read",
    "env.db_write",
    "env.db_remove",
    "env.addr_validate",
    "env.addr_canonicalize",
    "env.addr_humanize",
    "env.secp256k1_verify",
    "env.secp256k1_recover_pubkey",
    "env.ed25519_verify",
    "env.ed25519_batch_verify",
    "env.debug",
    "env.query_chain",
    #[cfg(feature = "iterator")]
    "env.db_scan",
    #[cfg(feature = "iterator")]
    "env.db_next",
    #[cfg(feature = "iterator")]
    "env.db_next_key",
    #[cfg(feature = "iterator")]
    "env.db_next_value",
];

/// Lists all entry points we expect to be present when calling a contract.
/// Other optional exports exist, e.g. "execute", "migrate" and "query".
/// The marker export interface_version_* is checked separately.
/// This is unlikely to change much, must be frozen at 1.0 to avoid breaking existing contracts
const REQUIRED_EXPORTS: &[&str] = &[
    // IO
    "allocate",
    "deallocate",
    // Required entry points
    "instantiate",
];

const INTERFACE_VERSION_PREFIX: &str = "interface_version_";
const SUPPORTED_INTERFACE_VERSIONS: &[&str] = &[
    "interface_version_8",
    #[cfg(feature = "allow_interface_version_7")]
    "interface_version_7",
];

const MEMORY_LIMIT: u32 = 512; // in pages
/// The upper limit for the `max` value of each table. CosmWasm contracts have
/// initial=max for 1 table. See
///
/// ```plain
/// $ wasm-objdump --section=table -x packages/vm/testdata/hackatom.wasm
/// Section Details:
///
/// Table[1]:
/// - table[0] type=funcref initial=161 max=161
/// ```
///
/// As of March 2023, on Juno mainnet the largest value for production contracts
/// is 485. Most are between 100 and 300.
const TABLE_SIZE_LIMIT: u32 = 2500; // entries

/// If the contract has more than this amount of imports, it will be rejected
/// during static validation before even looking into the imports. We keep this
/// number high since failing early gives less detailed error messages. Especially
/// when a user accidentally includes wasm-bindgen, they get a bunch of unsupported imports.
const MAX_IMPORTS: usize = 100;

/// Checks if the data is valid wasm and compatibility with the CosmWasm API (imports and exports)
pub fn check_wasm(wasm_code: &[u8], available_capabilities: &HashSet<String>) -> VmResult<()> {
    let module = ParsedWasm::parse(wasm_code)?;

    check_wasm_tables(&module)?;
    check_wasm_memories(&module)?;
    check_interface_version(&module)?;
    check_wasm_exports(&module)?;
    check_wasm_imports(&module, SUPPORTED_IMPORTS)?;
    check_wasm_capabilities(&module, available_capabilities)?;

    Ok(())
}

fn check_wasm_tables(module: &ParsedWasm) -> VmResult<()> {
    match module.tables.len() {
        0 => Ok(()),
        1 => {
            let limits = &module.tables[0];
            if let Some(maximum) = limits.maximum {
                if maximum > TABLE_SIZE_LIMIT {
                    return Err(VmError::static_validation_err(
                        "Wasm contract's first table section has a too large max limit",
                    ));
                }
                Ok(())
            } else {
                Err(VmError::static_validation_err(
                    "Wasm contract must not have unbound table section",
                ))
            }
        }
        _ => Err(VmError::static_validation_err(
            "Wasm contract must not have more than 1 table section",
        )),
    }
}

fn check_wasm_memories(module: &ParsedWasm) -> VmResult<()> {
    if module.memories.len() != 1 {
        return Err(VmError::static_validation_err(
            "Wasm contract must contain exactly one memory",
        ));
    }
    let memory = &module.memories[0];

    if memory.initial > MEMORY_LIMIT as u64 {
        return Err(VmError::static_validation_err(format!(
            "Wasm contract memory's minimum must not exceed {MEMORY_LIMIT} pages."
        )));
    }

    if memory.maximum.is_some() {
        return Err(VmError::static_validation_err(
            "Wasm contract memory's maximum must be unset. The host will set it for you.",
        ));
    }
    Ok(())
}

fn check_interface_version(module: &ParsedWasm) -> VmResult<()> {
    let mut interface_version_exports = module
        .exported_function_names(Some(INTERFACE_VERSION_PREFIX))
        .into_iter();
    if let Some(first_interface_version_export) = interface_version_exports.next() {
        if interface_version_exports.next().is_some() {
            Err(VmError::static_validation_err(
                "Wasm contract contains more than one marker export: interface_version_*",
            ))
        } else {
            // Exactly one interface version found
            let version_str = first_interface_version_export.as_str();
            if SUPPORTED_INTERFACE_VERSIONS
                .iter()
                .any(|&v| v == version_str)
            {
                Ok(())
            } else {
                Err(VmError::static_validation_err(
                        "Wasm contract has unknown interface_version_* marker export (see https://github.com/CosmWasm/cosmwasm/blob/main/packages/vm/README.md)",
                ))
            }
        }
    } else {
        Err(VmError::static_validation_err(
            "Wasm contract missing a required marker export: interface_version_*",
        ))
    }
}

fn check_wasm_exports(module: &ParsedWasm) -> VmResult<()> {
    let available_exports: HashSet<String> = module.exported_function_names(None);
    for required_export in REQUIRED_EXPORTS {
        if !available_exports.contains(*required_export) {
            return Err(VmError::static_validation_err(format!(
                "Wasm contract doesn't have required export: \"{required_export}\". Exports required by VM: {REQUIRED_EXPORTS:?}."
            )));
        }
    }
    Ok(())
}

/// Checks if the import requirements of the contract are satisfied.
/// When this is not the case, we either have an incompatibility between contract and VM
/// or a error in the contract.
fn check_wasm_imports(module: &ParsedWasm, supported_imports: &[&str]) -> VmResult<()> {
    if module.imports.len() > MAX_IMPORTS {
        return Err(VmError::static_validation_err(format!(
            "Import count exceeds limit. Imports: {}. Limit: {}.",
            module.imports.len(),
            MAX_IMPORTS
        )));
    }

    for required_import in &module.imports {
        let full_name = full_import_name(required_import);
        if !supported_imports.contains(&full_name.as_str()) {
            let required_import_names: BTreeSet<_> =
                module.imports.iter().map(full_import_name).collect();
            return Err(VmError::static_validation_err(format!(
                "Wasm contract requires unsupported import: \"{}\". Required imports: {}. Available imports: {:?}.",
                full_name, required_import_names.to_string_limited(200), supported_imports
            )));
        }

        match required_import.ty {
            TypeRef::Func(_) => {} // ok
            _ => return Err(VmError::static_validation_err(format!(
                "Wasm contract requires non-function import: \"{full_name}\". Right now, all supported imports are functions."
            )))
        }
    }
    Ok(())
}

fn full_import_name(ie: &Import) -> String {
    format!("{}.{}", ie.module, ie.name)
}

fn check_wasm_capabilities(
    module: &ParsedWasm,
    available_capabilities: &HashSet<String>,
) -> VmResult<()> {
    let required_capabilities = required_capabilities_from_module(module);
    if !required_capabilities.is_subset(available_capabilities) {
        // We switch to BTreeSet to get a sorted error message
        let unavailable: BTreeSet<_> = required_capabilities
            .difference(available_capabilities)
            .collect();
        return Err(VmError::static_validation_err(format!(
            "Wasm contract requires unavailable capabilities: {}",
            unavailable.to_string_limited(200)
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::VmError;

    static CONTRACT_0_7: &[u8] = include_bytes!("../testdata/hackatom_0.7.wasm");
    static CONTRACT_0_12: &[u8] = include_bytes!("../testdata/hackatom_0.12.wasm");
    static CONTRACT_0_14: &[u8] = include_bytes!("../testdata/hackatom_0.14.wasm");
    static CONTRACT_0_15: &[u8] = include_bytes!("../testdata/hackatom_0.15.wasm");
    static CONTRACT: &[u8] = include_bytes!("../testdata/hackatom.wasm");
    static CONTRACT_RUST_170: &[u8] = include_bytes!("../testdata/cyberpunk_rust170.wasm");

    fn default_capabilities() -> HashSet<String> {
        ["staking".to_string()].into_iter().collect()
    }

    #[test]
    fn check_wasm_passes_for_latest_contract() {
        // this is our reference check, must pass
        check_wasm(CONTRACT, &default_capabilities()).unwrap();
    }

    #[test]
    fn check_wasm_allows_sign_ext() {
        // See https://github.com/CosmWasm/cosmwasm/issues/1727
        check_wasm(CONTRACT_RUST_170, &default_capabilities()).unwrap();
    }

    #[test]
    fn check_wasm_old_contract() {
        match check_wasm(CONTRACT_0_15, &default_capabilities()) {
            Err(VmError::StaticValidationErr { msg, .. }) => assert_eq!(
                msg,
                "Wasm contract has unknown interface_version_* marker export (see https://github.com/CosmWasm/cosmwasm/blob/main/packages/vm/README.md)"
            ),
            Err(e) => panic!("Unexpected error {e:?}"),
            Ok(_) => panic!("This must not succeeed"),
        };

        match check_wasm(CONTRACT_0_14, &default_capabilities()) {
            Err(VmError::StaticValidationErr { msg, .. }) => assert_eq!(
                msg,
                "Wasm contract has unknown interface_version_* marker export (see https://github.com/CosmWasm/cosmwasm/blob/main/packages/vm/README.md)"
            ),
            Err(e) => panic!("Unexpected error {e:?}"),
            Ok(_) => panic!("This must not succeeed"),
        };

        match check_wasm(CONTRACT_0_12, &default_capabilities()) {
            Err(VmError::StaticValidationErr { msg, .. }) => {
                assert!(msg.contains(
                    "Wasm contract missing a required marker export: interface_version_*"
                ))
            }
            Err(e) => panic!("Unexpected error {e:?}"),
            Ok(_) => panic!("This must not succeeed"),
        };

        match check_wasm(CONTRACT_0_7, &default_capabilities()) {
            Err(VmError::StaticValidationErr { msg, .. }) => {
                assert!(msg.contains(
                    "Wasm contract missing a required marker export: interface_version_*"
                ))
            }
            Err(e) => panic!("Unexpected error {e:?}"),
            Ok(_) => panic!("This must not succeeed"),
        };
    }

    #[test]
    fn check_wasm_tables_works() {
        // No tables is fine
        let wasm = wat::parse_str("(module)").unwrap();
        assert!(ParsedWasm::parse(&wasm).unwrap().tables.is_empty());

        // One table (bound)
        let wasm = wat::parse_str("(module (table $name 123 123 funcref))").unwrap();
        check_wasm_tables(&ParsedWasm::parse(&wasm).unwrap()).unwrap();

        // One table (bound, initial > max)
        let wasm = wat::parse_str("(module (table $name 124 123 funcref))").unwrap();
        // this should be caught by the validator
        let err = &ParsedWasm::parse(&wasm).unwrap_err();
        assert!(err
            .to_string()
            .contains("size minimum must not be greater than maximum"));

        // One table (bound, max too large)
        let wasm = wat::parse_str("(module (table $name 100 9999 funcref))").unwrap();
        let err = check_wasm_tables(&ParsedWasm::parse(&wasm).unwrap()).unwrap_err();
        assert!(err
            .to_string()
            .contains("Wasm contract's first table section has a too large max limit"));

        // One table (unbound)
        let wasm = wat::parse_str("(module (table $name 100 funcref))").unwrap();
        let err = check_wasm_tables(&ParsedWasm::parse(&wasm).unwrap()).unwrap_err();
        assert!(err
            .to_string()
            .contains("Wasm contract must not have unbound table section"));
    }

    #[test]
    fn check_wasm_memories_ok() {
        let wasm = wat::parse_str("(module (memory 1))").unwrap();
        check_wasm_memories(&ParsedWasm::parse(&wasm).unwrap()).unwrap()
    }

    #[test]
    fn check_wasm_memories_no_memory() {
        let wasm = wat::parse_str("(module)").unwrap();
        match check_wasm_memories(&ParsedWasm::parse(&wasm).unwrap()) {
            Err(VmError::StaticValidationErr { msg, .. }) => {
                assert!(msg.starts_with("Wasm contract must contain exactly one memory"));
            }
            Err(e) => panic!("Unexpected error {e:?}"),
            Ok(_) => panic!("Didn't reject wasm with invalid api"),
        }
    }

    #[test]
    fn check_wasm_memories_two_memories() {
        // Generated manually because wat2wasm protects us from creating such Wasm:
        // "error: only one memory block allowed"
        let wasm = hex::decode(concat!(
            "0061736d", // magic bytes
            "01000000", // binary version (uint32)
            "05",       // section type (memory)
            "05",       // section length
            "02",       // number of memories
            "0009",     // element of type "resizable_limits", min=9, max=unset
            "0009",     // element of type "resizable_limits", min=9, max=unset
        ))
        .unwrap();

        // wrong number of memories should be caught by the validator
        match ParsedWasm::parse(&wasm) {
            Err(VmError::StaticValidationErr { msg, .. }) => {
                assert!(msg.contains("multiple memories"));
            }
            Err(e) => panic!("Unexpected error {e:?}"),
            Ok(_) => panic!("Didn't reject wasm with invalid api"),
        }
    }

    #[test]
    fn check_wasm_memories_zero_memories() {
        // Generated manually because wat2wasm would not create an empty memory section
        let wasm = hex::decode(concat!(
            "0061736d", // magic bytes
            "01000000", // binary version (uint32)
            "05",       // section type (memory)
            "01",       // section length
            "00",       // number of memories
        ))
        .unwrap();

        match check_wasm_memories(&ParsedWasm::parse(&wasm).unwrap()) {
            Err(VmError::StaticValidationErr { msg, .. }) => {
                assert!(msg.starts_with("Wasm contract must contain exactly one memory"));
            }
            Err(e) => panic!("Unexpected error {e:?}"),
            Ok(_) => panic!("Didn't reject wasm with invalid api"),
        }
    }

    #[test]
    fn check_wasm_memories_initial_size() {
        let wasm_ok = wat::parse_str("(module (memory 512))").unwrap();
        check_wasm_memories(&ParsedWasm::parse(&wasm_ok).unwrap()).unwrap();

        let wasm_too_big = wat::parse_str("(module (memory 513))").unwrap();
        match check_wasm_memories(&ParsedWasm::parse(&wasm_too_big).unwrap()) {
            Err(VmError::StaticValidationErr { msg, .. }) => {
                assert!(msg.starts_with("Wasm contract memory's minimum must not exceed 512 pages"));
            }
            Err(e) => panic!("Unexpected error {e:?}"),
            Ok(_) => panic!("Didn't reject wasm with invalid api"),
        }
    }

    #[test]
    fn check_wasm_memories_maximum_size() {
        let wasm_max = wat::parse_str("(module (memory 1 5))").unwrap();
        match check_wasm_memories(&ParsedWasm::parse(&wasm_max).unwrap()) {
            Err(VmError::StaticValidationErr { msg, .. }) => {
                assert!(msg.starts_with("Wasm contract memory's maximum must be unset"));
            }
            Err(e) => panic!("Unexpected error {e:?}"),
            Ok(_) => panic!("Didn't reject wasm with invalid api"),
        }
    }

    #[test]
    fn check_interface_version_works() {
        // valid
        let wasm = wat::parse_str(
            r#"(module
                (type (func))
                (func (type 0) nop)
                (export "add_one" (func 0))
                (export "allocate" (func 0))
                (export "interface_version_8" (func 0))
                (export "deallocate" (func 0))
                (export "instantiate" (func 0))
            )"#,
        )
        .unwrap();
        let module = ParsedWasm::parse(&wasm).unwrap();
        check_interface_version(&module).unwrap();

        #[cfg(feature = "allow_interface_version_7")]
        {
            // valid legacy version
            let wasm = wat::parse_str(
                r#"(module
                    (type (func))
                    (func (type 0) nop)
                    (export "add_one" (func 0))
                    (export "allocate" (func 0))
                    (export "interface_version_7" (func 0))
                    (export "deallocate" (func 0))
                    (export "instantiate" (func 0))
                )"#,
            )
            .unwrap();
            let module = ParsedWasm::parse(&wasm).unwrap();
            check_interface_version(&module).unwrap();
        }

        // missing
        let wasm = wat::parse_str(
            r#"(module
                (type (func))
                (func (type 0) nop)
                (export "add_one" (func 0))
                (export "allocate" (func 0))
                (export "deallocate" (func 0))
                (export "instantiate" (func 0))
            )"#,
        )
        .unwrap();
        let module = ParsedWasm::parse(&wasm).unwrap();
        match check_interface_version(&module).unwrap_err() {
            VmError::StaticValidationErr { msg, .. } => {
                assert_eq!(
                    msg,
                    "Wasm contract missing a required marker export: interface_version_*"
                );
            }
            err => panic!("Unexpected error {err:?}"),
        }

        // multiple
        let wasm = wat::parse_str(
            r#"(module
                (type (func))
                (func (type 0) nop)
                (export "add_one" (func 0))
                (export "allocate" (func 0))
                (export "interface_version_8" (func 0))
                (export "interface_version_9" (func 0))
                (export "deallocate" (func 0))
                (export "instantiate" (func 0))
            )"#,
        )
        .unwrap();
        let module = ParsedWasm::parse(&wasm).unwrap();
        match check_interface_version(&module).unwrap_err() {
            VmError::StaticValidationErr { msg, .. } => {
                assert_eq!(
                    msg,
                    "Wasm contract contains more than one marker export: interface_version_*"
                );
            }
            err => panic!("Unexpected error {err:?}"),
        }

        // CosmWasm 0.15
        let wasm = wat::parse_str(
            r#"(module
                (type (func))
                (func (type 0) nop)
                (export "add_one" (func 0))
                (export "allocate" (func 0))
                (export "interface_version_6" (func 0))
                (export "deallocate" (func 0))
                (export "instantiate" (func 0))
            )"#,
        )
        .unwrap();
        let module = ParsedWasm::parse(&wasm).unwrap();
        match check_interface_version(&module).unwrap_err() {
            VmError::StaticValidationErr { msg, .. } => {
                assert_eq!(msg, "Wasm contract has unknown interface_version_* marker export (see https://github.com/CosmWasm/cosmwasm/blob/main/packages/vm/README.md)");
            }
            err => panic!("Unexpected error {err:?}"),
        }

        // Unknown value
        let wasm = wat::parse_str(
            r#"(module
                (type (func))
                (func (type 0) nop)
                (export "add_one" (func 0))
                (export "allocate" (func 0))
                (export "interface_version_broken" (func 0))
                (export "deallocate" (func 0))
                (export "instantiate" (func 0))
            )"#,
        )
        .unwrap();
        let module = ParsedWasm::parse(&wasm).unwrap();
        match check_interface_version(&module).unwrap_err() {
            VmError::StaticValidationErr { msg, .. } => {
                assert_eq!(msg, "Wasm contract has unknown interface_version_* marker export (see https://github.com/CosmWasm/cosmwasm/blob/main/packages/vm/README.md)");
            }
            err => panic!("Unexpected error {err:?}"),
        }
    }

    #[test]
    fn check_wasm_exports_works() {
        // valid
        let wasm = wat::parse_str(
            r#"(module
                (type (func))
                (func (type 0) nop)
                (export "add_one" (func 0))
                (export "allocate" (func 0))
                (export "deallocate" (func 0))
                (export "instantiate" (func 0))
            )"#,
        )
        .unwrap();
        let module = ParsedWasm::parse(&wasm).unwrap();
        check_wasm_exports(&module).unwrap();

        // this is invalid, as it doesn't any required export
        let wasm = wat::parse_str(
            r#"(module
                (type (func))
                (func (type 0) nop)
                (export "add_one" (func 0))
            )"#,
        )
        .unwrap();
        let module = ParsedWasm::parse(&wasm).unwrap();
        match check_wasm_exports(&module) {
            Err(VmError::StaticValidationErr { msg, .. }) => {
                assert!(msg.starts_with("Wasm contract doesn't have required export: \"allocate\""));
            }
            Err(e) => panic!("Unexpected error {e:?}"),
            Ok(_) => panic!("Didn't reject wasm with invalid api"),
        }

        // this is invalid, as it doesn't contain all required exports
        let wasm = wat::parse_str(
            r#"(module
                (type (func))
                (func (type 0) nop)
                (export "add_one" (func 0))
                (export "allocate" (func 0))
            )"#,
        )
        .unwrap();
        let module = ParsedWasm::parse(&wasm).unwrap();
        match check_wasm_exports(&module) {
            Err(VmError::StaticValidationErr { msg, .. }) => {
                assert!(
                    msg.starts_with("Wasm contract doesn't have required export: \"deallocate\"")
                );
            }
            Err(e) => panic!("Unexpected error {e:?}"),
            Ok(_) => panic!("Didn't reject wasm with invalid api"),
        }
    }

    #[test]
    fn check_wasm_exports_of_old_contract() {
        let module = ParsedWasm::parse(CONTRACT_0_7).unwrap();
        match check_wasm_exports(&module) {
            Err(VmError::StaticValidationErr { msg, .. }) => {
                assert!(
                    msg.starts_with("Wasm contract doesn't have required export: \"instantiate\"")
                )
            }
            Err(e) => panic!("Unexpected error {e:?}"),
            Ok(_) => panic!("Didn't reject wasm with invalid api"),
        }
    }

    #[test]
    fn check_wasm_imports_ok() {
        let wasm = wat::parse_str(
            r#"(module
            (import "env" "db_read" (func (param i32 i32) (result i32)))
            (import "env" "db_write" (func (param i32 i32) (result i32)))
            (import "env" "db_remove" (func (param i32) (result i32)))
            (import "env" "addr_validate" (func (param i32) (result i32)))
            (import "env" "addr_canonicalize" (func (param i32 i32) (result i32)))
            (import "env" "addr_humanize" (func (param i32 i32) (result i32)))
            (import "env" "secp256k1_verify" (func (param i32 i32 i32) (result i32)))
            (import "env" "secp256k1_recover_pubkey" (func (param i32 i32 i32) (result i64)))
            (import "env" "ed25519_verify" (func (param i32 i32 i32) (result i32)))
            (import "env" "ed25519_batch_verify" (func (param i32 i32 i32) (result i32)))
        )"#,
        )
        .unwrap();
        check_wasm_imports(&ParsedWasm::parse(&wasm).unwrap(), SUPPORTED_IMPORTS).unwrap();
    }

    #[test]
    fn check_wasm_imports_exceeds_limit() {
        let wasm = wat::parse_str(
            r#"(module
            (import "env" "db_write" (func (param i32 i32) (result i32)))
            (import "env" "db_remove" (func (param i32) (result i32)))
            (import "env" "addr_validate" (func (param i32) (result i32)))
            (import "env" "addr_canonicalize" (func (param i32 i32) (result i32)))
            (import "env" "addr_humanize" (func (param i32 i32) (result i32)))
            (import "env" "secp256k1_verify" (func (param i32 i32 i32) (result i32)))
            (import "env" "secp256k1_recover_pubkey" (func (param i32 i32 i32) (result i64)))
            (import "env" "ed25519_verify" (func (param i32 i32 i32) (result i32)))
            (import "env" "ed25519_batch_verify" (func (param i32 i32 i32) (result i32)))
            (import "env" "spam01" (func (param i32 i32) (result i32)))
            (import "env" "spam02" (func (param i32 i32) (result i32)))
            (import "env" "spam03" (func (param i32 i32) (result i32)))
            (import "env" "spam04" (func (param i32 i32) (result i32)))
            (import "env" "spam05" (func (param i32 i32) (result i32)))
            (import "env" "spam06" (func (param i32 i32) (result i32)))
            (import "env" "spam07" (func (param i32 i32) (result i32)))
            (import "env" "spam08" (func (param i32 i32) (result i32)))
            (import "env" "spam09" (func (param i32 i32) (result i32)))
            (import "env" "spam10" (func (param i32 i32) (result i32)))
            (import "env" "spam11" (func (param i32 i32) (result i32)))
            (import "env" "spam12" (func (param i32 i32) (result i32)))
            (import "env" "spam13" (func (param i32 i32) (result i32)))
            (import "env" "spam14" (func (param i32 i32) (result i32)))
            (import "env" "spam15" (func (param i32 i32) (result i32)))
            (import "env" "spam16" (func (param i32 i32) (result i32)))
            (import "env" "spam17" (func (param i32 i32) (result i32)))
            (import "env" "spam18" (func (param i32 i32) (result i32)))
            (import "env" "spam19" (func (param i32 i32) (result i32)))
            (import "env" "spam20" (func (param i32 i32) (result i32)))
            (import "env" "spam21" (func (param i32 i32) (result i32)))
            (import "env" "spam22" (func (param i32 i32) (result i32)))
            (import "env" "spam23" (func (param i32 i32) (result i32)))
            (import "env" "spam24" (func (param i32 i32) (result i32)))
            (import "env" "spam25" (func (param i32 i32) (result i32)))
            (import "env" "spam26" (func (param i32 i32) (result i32)))
            (import "env" "spam27" (func (param i32 i32) (result i32)))
            (import "env" "spam28" (func (param i32 i32) (result i32)))
            (import "env" "spam29" (func (param i32 i32) (result i32)))
            (import "env" "spam30" (func (param i32 i32) (result i32)))
            (import "env" "spam31" (func (param i32 i32) (result i32)))
            (import "env" "spam32" (func (param i32 i32) (result i32)))
            (import "env" "spam33" (func (param i32 i32) (result i32)))
            (import "env" "spam34" (func (param i32 i32) (result i32)))
            (import "env" "spam35" (func (param i32 i32) (result i32)))
            (import "env" "spam36" (func (param i32 i32) (result i32)))
            (import "env" "spam37" (func (param i32 i32) (result i32)))
            (import "env" "spam38" (func (param i32 i32) (result i32)))
            (import "env" "spam39" (func (param i32 i32) (result i32)))
            (import "env" "spam40" (func (param i32 i32) (result i32)))
            (import "env" "spam41" (func (param i32 i32) (result i32)))
            (import "env" "spam42" (func (param i32 i32) (result i32)))
            (import "env" "spam43" (func (param i32 i32) (result i32)))
            (import "env" "spam44" (func (param i32 i32) (result i32)))
            (import "env" "spam45" (func (param i32 i32) (result i32)))
            (import "env" "spam46" (func (param i32 i32) (result i32)))
            (import "env" "spam47" (func (param i32 i32) (result i32)))
            (import "env" "spam48" (func (param i32 i32) (result i32)))
            (import "env" "spam49" (func (param i32 i32) (result i32)))
            (import "env" "spam50" (func (param i32 i32) (result i32)))
            (import "env" "spam51" (func (param i32 i32) (result i32)))
            (import "env" "spam52" (func (param i32 i32) (result i32)))
            (import "env" "spam53" (func (param i32 i32) (result i32)))
            (import "env" "spam54" (func (param i32 i32) (result i32)))
            (import "env" "spam55" (func (param i32 i32) (result i32)))
            (import "env" "spam56" (func (param i32 i32) (result i32)))
            (import "env" "spam57" (func (param i32 i32) (result i32)))
            (import "env" "spam58" (func (param i32 i32) (result i32)))
            (import "env" "spam59" (func (param i32 i32) (result i32)))
            (import "env" "spam60" (func (param i32 i32) (result i32)))
            (import "env" "spam61" (func (param i32 i32) (result i32)))
            (import "env" "spam62" (func (param i32 i32) (result i32)))
            (import "env" "spam63" (func (param i32 i32) (result i32)))
            (import "env" "spam64" (func (param i32 i32) (result i32)))
            (import "env" "spam65" (func (param i32 i32) (result i32)))
            (import "env" "spam66" (func (param i32 i32) (result i32)))
            (import "env" "spam67" (func (param i32 i32) (result i32)))
            (import "env" "spam68" (func (param i32 i32) (result i32)))
            (import "env" "spam69" (func (param i32 i32) (result i32)))
            (import "env" "spam70" (func (param i32 i32) (result i32)))
            (import "env" "spam71" (func (param i32 i32) (result i32)))
            (import "env" "spam72" (func (param i32 i32) (result i32)))
            (import "env" "spam73" (func (param i32 i32) (result i32)))
            (import "env" "spam74" (func (param i32 i32) (result i32)))
            (import "env" "spam75" (func (param i32 i32) (result i32)))
            (import "env" "spam76" (func (param i32 i32) (result i32)))
            (import "env" "spam77" (func (param i32 i32) (result i32)))
            (import "env" "spam78" (func (param i32 i32) (result i32)))
            (import "env" "spam79" (func (param i32 i32) (result i32)))
            (import "env" "spam80" (func (param i32 i32) (result i32)))
            (import "env" "spam81" (func (param i32 i32) (result i32)))
            (import "env" "spam82" (func (param i32 i32) (result i32)))
            (import "env" "spam83" (func (param i32 i32) (result i32)))
            (import "env" "spam84" (func (param i32 i32) (result i32)))
            (import "env" "spam85" (func (param i32 i32) (result i32)))
            (import "env" "spam86" (func (param i32 i32) (result i32)))
            (import "env" "spam87" (func (param i32 i32) (result i32)))
            (import "env" "spam88" (func (param i32 i32) (result i32)))
            (import "env" "spam89" (func (param i32 i32) (result i32)))
            (import "env" "spam90" (func (param i32 i32) (result i32)))
            (import "env" "spam91" (func (param i32 i32) (result i32)))
            (import "env" "spam92" (func (param i32 i32) (result i32)))
        )"#,
        )
        .unwrap();
        let err =
            check_wasm_imports(&ParsedWasm::parse(&wasm).unwrap(), SUPPORTED_IMPORTS).unwrap_err();
        match err {
            VmError::StaticValidationErr { msg, .. } => {
                assert_eq!(msg, "Import count exceeds limit. Imports: 101. Limit: 100.");
            }
            err => panic!("Unexpected error: {err:?}"),
        }
    }

    #[test]
    fn check_wasm_imports_missing() {
        let wasm = wat::parse_str(
            r#"(module
            (import "env" "foo" (func (param i32 i32) (result i32)))
            (import "env" "bar" (func (param i32 i32) (result i32)))
            (import "env" "spammyspam01" (func (param i32 i32) (result i32)))
            (import "env" "spammyspam02" (func (param i32 i32) (result i32)))
            (import "env" "spammyspam03" (func (param i32 i32) (result i32)))
            (import "env" "spammyspam04" (func (param i32 i32) (result i32)))
            (import "env" "spammyspam05" (func (param i32 i32) (result i32)))
            (import "env" "spammyspam06" (func (param i32 i32) (result i32)))
            (import "env" "spammyspam07" (func (param i32 i32) (result i32)))
            (import "env" "spammyspam08" (func (param i32 i32) (result i32)))
            (import "env" "spammyspam09" (func (param i32 i32) (result i32)))
            (import "env" "spammyspam10" (func (param i32 i32) (result i32)))
        )"#,
        )
        .unwrap();
        let supported_imports: &[&str] = &[
            "env.db_read",
            "env.db_write",
            "env.db_remove",
            "env.addr_canonicalize",
            "env.addr_humanize",
            "env.debug",
            "env.query_chain",
        ];
        let result = check_wasm_imports(&ParsedWasm::parse(&wasm).unwrap(), supported_imports);
        match result.unwrap_err() {
            VmError::StaticValidationErr { msg, .. } => {
                println!("{msg}");
                assert_eq!(
                    msg,
                    r#"Wasm contract requires unsupported import: "env.foo". Required imports: {"env.bar", "env.foo", "env.spammyspam01", "env.spammyspam02", "env.spammyspam03", "env.spammyspam04", "env.spammyspam05", "env.spammyspam06", "env.spammyspam07", "env.spammyspam08", ... 2 more}. Available imports: ["env.db_read", "env.db_write", "env.db_remove", "env.addr_canonicalize", "env.addr_humanize", "env.debug", "env.query_chain"]."#
                );
            }
            err => panic!("Unexpected error: {err:?}"),
        }
    }

    #[test]
    fn check_wasm_imports_of_old_contract() {
        let module = &ParsedWasm::parse(CONTRACT_0_7).unwrap();
        let result = check_wasm_imports(module, SUPPORTED_IMPORTS);
        match result.unwrap_err() {
            VmError::StaticValidationErr { msg, .. } => {
                assert!(
                    msg.starts_with("Wasm contract requires unsupported import: \"env.read_db\"")
                );
            }
            err => panic!("Unexpected error: {err:?}"),
        }
    }

    #[test]
    fn check_wasm_imports_wrong_type() {
        let wasm = wat::parse_str(r#"(module (import "env" "db_read" (memory 1 1)))"#).unwrap();
        let result = check_wasm_imports(&ParsedWasm::parse(&wasm).unwrap(), SUPPORTED_IMPORTS);
        match result.unwrap_err() {
            VmError::StaticValidationErr { msg, .. } => {
                assert!(
                    msg.starts_with("Wasm contract requires non-function import: \"env.db_read\"")
                );
            }
            err => panic!("Unexpected error: {err:?}"),
        }
    }

    #[test]
    fn check_wasm_capabilities_ok() {
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
        let module = ParsedWasm::parse(&wasm).unwrap();
        let available = [
            "water".to_string(),
            "nutrients".to_string(),
            "sun".to_string(),
            "freedom".to_string(),
        ]
        .into_iter()
        .collect();
        check_wasm_capabilities(&module, &available).unwrap();
    }

    #[test]
    fn check_wasm_capabilities_fails_for_missing() {
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
        let module = ParsedWasm::parse(&wasm).unwrap();

        // Available set 1
        let available = [
            "water".to_string(),
            "nutrients".to_string(),
            "freedom".to_string(),
        ]
        .into_iter()
        .collect();
        match check_wasm_capabilities(&module, &available).unwrap_err() {
            VmError::StaticValidationErr { msg, .. } => assert_eq!(
                msg,
                "Wasm contract requires unavailable capabilities: {\"sun\"}"
            ),
            _ => panic!("Got unexpected error"),
        }

        // Available set 2
        let available = [
            "nutrients".to_string(),
            "freedom".to_string(),
            "Water".to_string(), // capabilities are case sensitive (and lowercase by convention)
        ]
        .into_iter()
        .collect();
        match check_wasm_capabilities(&module, &available).unwrap_err() {
            VmError::StaticValidationErr { msg, .. } => assert_eq!(
                msg,
                "Wasm contract requires unavailable capabilities: {\"sun\", \"water\"}"
            ),
            _ => panic!("Got unexpected error"),
        }

        // Available set 3
        let available = ["freedom".to_string()].into_iter().collect();
        match check_wasm_capabilities(&module, &available).unwrap_err() {
            VmError::StaticValidationErr { msg, .. } => assert_eq!(
                msg,
                "Wasm contract requires unavailable capabilities: {\"nutrients\", \"sun\", \"water\"}"
            ),
            _ => panic!("Got unexpected error"),
        }

        // Available set 4
        let available = [].into_iter().collect();
        match check_wasm_capabilities(&module, &available).unwrap_err() {
            VmError::StaticValidationErr { msg, .. } => assert_eq!(
                msg,
                "Wasm contract requires unavailable capabilities: {\"nutrients\", \"sun\", \"water\"}"
            ),
            _ => panic!("Got unexpected error"),
        }
    }
}
