//go:build linux
// +build linux

package keyring

import (
	"errors"
	"io"
	"strings"
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/codec"
)

func TestNewKeyctlKeyring(t *testing.T) {
	cdc := getCodec()

	tests := []struct {
		name        string
		appName     string
		backend     string
		dir         string
		userInput   io.Reader
		cdc         codec.Codec
		expectedErr error
	}{
		{
			name:        "keyctl backend",
			appName:     "cosmos",
			backend:     BackendKeyctl,
			dir:         t.TempDir(),
			userInput:   strings.NewReader(""),
			cdc:         cdc,
			expectedErr: nil,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			kr, err := New(tt.appName, tt.backend, tt.dir, tt.userInput, tt.cdc)
			if tt.expectedErr == nil {
				require.NoError(t, err)
			} else {
				require.Error(t, err)
				require.Nil(t, kr)
				require.True(t, errors.Is(err, tt.expectedErr))
			}
		})
	}
}
