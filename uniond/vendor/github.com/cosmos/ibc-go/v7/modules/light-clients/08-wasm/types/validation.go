package types

const MaxWasmSize = 3 * 1024 * 1024

// ValidateWasmCode valides that the size of the wasm code is in the allowed range.
func ValidateWasmCode(code []byte) (bool, error) {
	if len(code) == 0 {
		return false, ErrWasmEmptyCode
	}
	if len(code) > MaxWasmSize {
		return false, ErrWasmCodeTooLarge
	}

	return true, nil
}
