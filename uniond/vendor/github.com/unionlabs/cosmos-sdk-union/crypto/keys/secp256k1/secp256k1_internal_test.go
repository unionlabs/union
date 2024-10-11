package secp256k1

import (
	"bytes"
	"math/big"
	"testing"

	secp "github.com/decred/dcrd/dcrec/secp256k1/v4"
	"github.com/stretchr/testify/require"
)

func Test_genPrivKey(t *testing.T) {
	empty := make([]byte, 32)
	oneB := big.NewInt(1).Bytes()
	onePadded := make([]byte, 32)
	copy(onePadded[32-len(oneB):32], oneB)
	t.Logf("one padded: %v, len=%v", onePadded, len(onePadded))

	validOne := append(empty, onePadded...)
	tests := []struct {
		name        string
		notSoRand   []byte
		shouldPanic bool
	}{
		{"empty bytes (panics because 1st 32 bytes are zero and 0 is not a valid field element)", empty, true},
		{"curve order: N", secp.S256().N.Bytes(), true},
		{"valid because 0 < 1 < N", validOne, false},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if tt.shouldPanic {
				require.Panics(t, func() {
					genPrivKeyLegacy(bytes.NewReader(tt.notSoRand))
				})
				return
			}
			got := genPrivKeyLegacy(bytes.NewReader(tt.notSoRand))
			fe := new(big.Int).SetBytes(got)
			require.True(t, fe.Cmp(secp.S256().N) < 0)
			require.True(t, fe.Sign() > 0)
		})
	}
}
