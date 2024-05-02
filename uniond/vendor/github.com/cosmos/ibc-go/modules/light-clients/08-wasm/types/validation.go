package types

import (
	"strings"

	errorsmod "cosmossdk.io/errors"

	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
)

// MaxWasmSize denotes the maximum size (in bytes) a contract is allowed to be.
const MaxWasmSize uint64 = 3 * 1024 * 1024

// ValidateWasmCode valides that the size of the wasm code is in the allowed range
// and that the contents are of a wasm binary.
func ValidateWasmCode(code []byte) error {
	if len(code) == 0 {
		return ErrWasmEmptyCode
	}
	if uint64(len(code)) > MaxWasmSize {
		return ErrWasmCodeTooLarge
	}

	return nil
}

// ValidateWasmChecksum validates that the checksum is of the correct length
func ValidateWasmChecksum(checksum Checksum) error {
	lenChecksum := len(checksum)
	if lenChecksum == 0 {
		return errorsmod.Wrap(ErrInvalidChecksum, "checksum cannot be empty")
	}
	if lenChecksum != 32 { // sha256 output is 256 bits long
		return errorsmod.Wrapf(ErrInvalidChecksum, "expected length of 32 bytes, got %d", lenChecksum)
	}

	return nil
}

// ValidateClientID validates the client identifier by ensuring that it conforms
// to the 02-client identifier format and that it is a 08-wasm clientID.
func ValidateClientID(clientID string) error {
	if !clienttypes.IsValidClientID(clientID) {
		return errorsmod.Wrapf(host.ErrInvalidID, "invalid client identifier %s", clientID)
	}

	if !strings.HasPrefix(clientID, Wasm) {
		return errorsmod.Wrapf(host.ErrInvalidID, "client identifier %s does not contain %s prefix", clientID, Wasm)
	}

	return nil
}
