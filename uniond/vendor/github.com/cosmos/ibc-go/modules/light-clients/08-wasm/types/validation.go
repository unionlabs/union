package types

import (
	"strings"

	errorsmod "cosmossdk.io/errors"

	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

const maxWasmSize = 3 * 1024 * 1024

// ValidateWasmCode valides that the size of the wasm code is in the allowed range
// and that the contents are of a wasm binary.
func ValidateWasmCode(code []byte) error {
	if len(code) == 0 {
		return ErrWasmEmptyCode
	}
	if len(code) > maxWasmSize {
		return ErrWasmCodeTooLarge
	}

	return nil
}

// MaxWasmByteSize returns the maximum allowed number of bytes for wasm bytecode
func MaxWasmByteSize() uint64 {
	return maxWasmSize
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

	if !strings.HasPrefix(clientID, exported.Wasm) {
		return errorsmod.Wrapf(host.ErrInvalidID, "client identifier %s does not contain %s prefix", clientID, exported.Wasm)
	}

	return nil
}
