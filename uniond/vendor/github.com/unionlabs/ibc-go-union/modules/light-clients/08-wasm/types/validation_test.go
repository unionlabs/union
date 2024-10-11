package types_test

import (
	"crypto/rand"
	"testing"

	"github.com/stretchr/testify/require"

	errorsmod "cosmossdk.io/errors"

	wasmtesting "github.com/cosmos/ibc-go/modules/light-clients/08-wasm/testing"
	"github.com/cosmos/ibc-go/modules/light-clients/08-wasm/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
)

func TestValidateWasmCode(t *testing.T) {
	var code []byte

	testCases := []struct {
		name     string
		malleate func()
		expError error
	}{
		{
			"success",
			func() {
				code = wasmtesting.Code
			},
			nil,
		},
		{
			"failure: empty byte slice",
			func() {
				code = []byte{}
			},
			types.ErrWasmEmptyCode,
		},
		{
			"failure: byte slice too large",
			func() {
				expLength := types.MaxWasmByteSize() + 1
				code = make([]byte, expLength)
				length, err := rand.Read(code)
				require.NoError(t, err, t.Name())
				require.Equal(t, expLength, uint64(length), t.Name())
			},
			types.ErrWasmCodeTooLarge,
		},
	}

	for _, tc := range testCases {
		tc.malleate()

		err := types.ValidateWasmCode(code)

		if tc.expError == nil {
			require.NoError(t, err, tc.name)
		} else {
			require.ErrorIs(t, err, tc.expError, tc.name)
		}
	}
}

func TestValidateWasmChecksum(t *testing.T) {
	var checksum types.Checksum

	testCases := []struct {
		name     string
		malleate func()
		expError error
	}{
		{
			"success",
			func() {
				hash, err := types.CreateChecksum(wasmtesting.Code)
				require.NoError(t, err, t.Name())
				checksum = hash
			},
			nil,
		},
		{
			"failure: nil byte slice",
			func() {
				checksum = nil
			},
			errorsmod.Wrap(types.ErrInvalidChecksum, "checksum cannot be empty"),
		},
		{
			"failure: empty byte slice",
			func() {
				checksum = []byte{}
			},
			errorsmod.Wrap(types.ErrInvalidChecksum, "checksum cannot be empty"),
		},
		{
			"failure: byte slice size is not 32",
			func() {
				checksum = []byte{1}
			},
			errorsmod.Wrapf(types.ErrInvalidChecksum, "expected length of 32 bytes, got %d", 1),
		},
	}

	for _, tc := range testCases {
		tc.malleate()

		err := types.ValidateWasmChecksum(checksum)

		if tc.expError == nil {
			require.NoError(t, err, tc.name)
		} else {
			require.ErrorContains(t, err, tc.expError.Error(), tc.name)
		}
	}
}

func TestValidateClientID(t *testing.T) {
	var clientID string

	testCases := []struct {
		name     string
		malleate func()
		expError error
	}{
		{
			"success: valid wasm client identifier",
			func() {
				clientID = defaultWasmClientID
			},
			nil,
		},
		{
			"failure: empty clientID",
			func() {
				clientID = ""
			},
			errorsmod.Wrapf(host.ErrInvalidID, "invalid client identifier %s", clientID),
		},
		{
			"failure: clientID is not a wasm client identifier",
			func() {
				clientID = ibctesting.FirstClientID
			},
			errorsmod.Wrapf(host.ErrInvalidID, "client identifier %s does not contain %s prefix", ibctesting.FirstClientID, types.Wasm),
		},
	}

	for _, tc := range testCases {
		tc.malleate()

		err := types.ValidateClientID(clientID)

		if tc.expError == nil {
			require.NoError(t, err, tc.name)
		} else {
			require.ErrorContains(t, err, tc.expError.Error(), tc.name)
		}
	}
}
