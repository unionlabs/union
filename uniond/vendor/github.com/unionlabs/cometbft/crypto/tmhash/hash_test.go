package tmhash_test

import (
	"crypto/sha256"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/cometbft/cometbft/crypto/tmhash"
)

func TestHash(t *testing.T) {
	testVector := []byte("abc")
	hasher := tmhash.New()
	_, err := hasher.Write(testVector)
	require.NoError(t, err)
	bz := hasher.Sum(nil)

	bz2 := tmhash.Sum(testVector)

	hasher = sha256.New()
	_, err = hasher.Write(testVector)
	require.NoError(t, err)
	bz3 := hasher.Sum(nil)

	assert.Equal(t, bz, bz2)
	assert.Equal(t, bz, bz3)
}

func TestHashTruncated(t *testing.T) {
	testVector := []byte("abc")
	hasher := tmhash.NewTruncated()
	_, err := hasher.Write(testVector)
	require.NoError(t, err)
	bz := hasher.Sum(nil)

	bz2 := tmhash.SumTruncated(testVector)

	hasher = sha256.New()
	_, err = hasher.Write(testVector)
	require.NoError(t, err)
	bz3 := hasher.Sum(nil)
	bz3 = bz3[:tmhash.TruncatedSize]

	assert.Equal(t, bz, bz2)
	assert.Equal(t, bz, bz3)
}
