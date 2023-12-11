use wasmer_types::{CompileError, WasmError};
use wasmparser::BinaryReaderError;

/// Return an `Err(WasmError::Unsupported(msg))` where `msg` the string built by calling `format!`
/// on the arguments to this macro.
#[macro_export]
macro_rules! wasm_unsupported {
    ($($arg:tt)*) => { wasmer_types::WasmError::Unsupported(format!($($arg)*)) }
}

///
pub fn from_binaryreadererror_wasmerror(original: BinaryReaderError) -> WasmError {
    WasmError::InvalidWebAssembly {
        message: original.message().into(),
        offset: original.offset(),
    }
}

///
#[allow(dead_code)]
pub fn from_binaryreadererror_compileerror(original: BinaryReaderError) -> CompileError {
    // BinaryReaderError -> WasmError -> CompileError
    CompileError::Wasm(from_binaryreadererror_wasmerror(original))
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasmparser::BinaryReader;

    #[test]
    fn can_convert_binary_reader_error_to_wasm_error() {
        let mut reader = BinaryReader::new(b"\0\0\0\0");
        let binary_reader_error = reader.read_bytes(10).unwrap_err();
        match from_binaryreadererror_wasmerror(binary_reader_error) {
            WasmError::InvalidWebAssembly { message, offset } => {
                assert_eq!(message, "unexpected end-of-file");
                assert_eq!(offset, 0);
            }
            err => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    fn can_convert_binary_reader_error_to_compile_error() {
        let mut reader = BinaryReader::new(b"\0\0\0\0");
        let binary_reader_error = reader.read_bytes(10).unwrap_err();
        match from_binaryreadererror_compileerror(binary_reader_error) {
            CompileError::Wasm(WasmError::InvalidWebAssembly { message, offset }) => {
                assert_eq!(message, "unexpected end-of-file");
                assert_eq!(offset, 0);
            }
            err => panic!("Unexpected error: {:?}", err),
        }
    }
}
