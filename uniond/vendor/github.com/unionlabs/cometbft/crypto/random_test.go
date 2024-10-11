package crypto_test

import (
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cometbft/cometbft/crypto"
)

// the purpose of this test is primarily to ensure that the randomness
// generation won't error.
func TestRandomConsistency(t *testing.T) {
	x1 := crypto.CRandBytes(256)
	x2 := crypto.CRandBytes(256)
	x3 := crypto.CRandBytes(256)
	x4 := crypto.CRandBytes(256)
	x5 := crypto.CRandBytes(256)
	require.NotEqual(t, x1, x2)
	require.NotEqual(t, x3, x4)
	require.NotEqual(t, x4, x5)
	require.NotEqual(t, x1, x5)
}
