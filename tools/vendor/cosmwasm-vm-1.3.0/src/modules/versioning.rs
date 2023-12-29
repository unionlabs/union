use crate::wasm_backend::compile;

/// This header prefix contains the module type (wasmer-universal) and
/// the magic value WASMER\0\0.
/// The full header also contains a little endian encoded uint32 version number
/// and a length that we do not check.
const EXPECTED_MODULE_HEADER_PREFIX: &[u8] = b"wasmer-universalWASMER\0\0";

const ENGINE_TYPE_LEN: usize = 16; // https://github.com/wasmerio/wasmer/blob/2.2.0-rc1/lib/engine-universal/src/artifact.rs#L48
const METADATA_HEADER_LEN: usize = 16; // https://github.com/wasmerio/wasmer/blob/2.2.0-rc1/lib/engine/src/artifact.rs#L251-L252

fn current_wasmer_module_header() -> Vec<u8> {
    // echo "(module)" > my.wat && wat2wasm my.wat && hexdump -C my.wasm
    const WASM: &[u8] = b"\x00\x61\x73\x6d\x01\x00\x00\x00";
    let module = compile(WASM, None, &[]).unwrap();
    let mut bytes = module.serialize().unwrap_or_default();

    bytes.truncate(ENGINE_TYPE_LEN + METADATA_HEADER_LEN);
    bytes
}

/// Obtains the module version from Wasmer that is currently used.
/// As long as the overall format does not change, this returns a
/// counter (1 for Wasmer 2.2.0). When the format changes in an
/// unexpected way (e.g. a different engine is used or the meta
/// format changes), this panics. That way we can ensure an
/// incompatible module format can be found early in the development
/// cycle.
pub fn current_wasmer_module_version() -> u32 {
    let header = current_wasmer_module_header();
    if !header.starts_with(EXPECTED_MODULE_HEADER_PREFIX) {
        panic!("Wasmer module format changed. Please update the expected version accordingly and bump MODULE_SERIALIZATION_VERSION.");
    }

    let metadata = &header[header.len() - METADATA_HEADER_LEN..];
    u32::from_le_bytes((metadata[8..12]).try_into().unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_wasmer_module_header_works() {
        let header = current_wasmer_module_header();
        assert!(header.starts_with(EXPECTED_MODULE_HEADER_PREFIX));
    }

    #[test]
    fn current_wasmer_module_version_works() {
        let version = current_wasmer_module_version();
        assert_eq!(version, 1);
    }
}
