/// Check if the provided bytes are wasm-like
pub fn is_wasm(bytes: impl AsRef<[u8]>) -> bool {
    bytes.as_ref().starts_with(b"\0asm")
}
