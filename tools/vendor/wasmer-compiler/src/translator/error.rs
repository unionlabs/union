use crate::{CompileError, WasmError};
use wasmparser::BinaryReaderError;

/// Return an `Err(WasmError::Unsupported(msg))` where `msg` the string built by calling `format!`
/// on the arguments to this macro.
#[macro_export]
macro_rules! wasm_unsupported {
    ($($arg:tt)*) => { $crate::WasmError::Unsupported(format!($($arg)*)) }
}

impl From<BinaryReaderError> for WasmError {
    fn from(original: BinaryReaderError) -> Self {
        Self::InvalidWebAssembly {
            message: original.message().into(),
            offset: original.offset(),
        }
    }
}

impl From<BinaryReaderError> for CompileError {
    fn from(original: BinaryReaderError) -> Self {
        // `From` does not seem to be transitive by default, so we convert
        // BinaryReaderError -> WasmError -> CompileError
        Self::from(WasmError::from(original))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasmparser::BinaryReader;

    #[test]
    fn can_convert_binary_reader_error_to_wasm_error() {
        let mut reader = BinaryReader::new(b"\0\0\0\0");
        let binary_reader_error = reader.read_bytes(10).unwrap_err();
        match WasmError::from(binary_reader_error) {
            WasmError::InvalidWebAssembly { message, offset } => {
                assert_eq!(message, "Unexpected EOF");
                assert_eq!(offset, 0);
            }
            err => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    fn can_convert_binary_reader_error_to_compile_error() {
        let mut reader = BinaryReader::new(b"\0\0\0\0");
        let binary_reader_error = reader.read_bytes(10).unwrap_err();
        match CompileError::from(binary_reader_error) {
            CompileError::Wasm(WasmError::InvalidWebAssembly { message, offset }) => {
                assert_eq!(message, "Unexpected EOF");
                assert_eq!(offset, 0);
            }
            err => panic!("Unexpected error: {:?}", err),
        }
    }
}
