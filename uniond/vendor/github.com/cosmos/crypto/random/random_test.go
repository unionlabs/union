package random_test

import (
	"github.com/cosmos/crypto/random"
	"testing"

	"github.com/stretchr/testify/require"
)

// the purpose of this test is primarily to ensure that the randomness
// generation won't error.
func TestRandomConsistency(t *testing.T) {
	x1 := random.CRandBytes(256)
	x2 := random.CRandBytes(256)
	x3 := random.CRandBytes(256)
	x4 := random.CRandBytes(256)
	x5 := random.CRandBytes(256)
	require.NotEqual(t, x1, x2)
	require.NotEqual(t, x3, x4)
	require.NotEqual(t, x4, x5)
	require.NotEqual(t, x1, x5)
}
