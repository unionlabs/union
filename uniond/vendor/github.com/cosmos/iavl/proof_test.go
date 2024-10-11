// nolint: errcheck
package iavl

import (
	"bytes"
	"sort"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	iavlrand "github.com/cosmos/iavl/internal/rand"
)

func TestTreeGetProof(t *testing.T) {
	require := require.New(t)
	tree := getTestTree(0)
	for _, ikey := range []byte{0x11, 0x32, 0x50, 0x72, 0x99} {
		key := []byte{ikey}
		tree.Set(key, []byte(iavlrand.RandStr(8)))
	}

	key := []byte{0x32}
	proof, err := tree.GetMembershipProof(key)
	require.NoError(err)
	require.NotNil(proof)

	res, err := tree.VerifyMembership(proof, key)
	require.NoError(err, "%+v", err)
	require.True(res)

	key = []byte{0x1}
	proof, err = tree.GetNonMembershipProof(key)
	require.NoError(err)
	require.NotNil(proof)

	res, err = tree.VerifyNonMembership(proof, key)
	require.NoError(err, "%+v", err)
	require.True(res)
}

func TestTreeKeyExistsProof(t *testing.T) {
	tree := getTestTree(0)

	// should get error
	_, err := tree.GetProof([]byte("foo"))
	assert.Error(t, err)

	// insert lots of info and store the bytes
	allkeys := make([][]byte, 200)
	for i := 0; i < 200; i++ {
		key := iavlrand.RandStr(20)
		value := "value_for_" + key
		tree.Set([]byte(key), []byte(value))
		allkeys[i] = []byte(key)
	}
	sortByteSlices(allkeys) // Sort all keys

	// query random key fails
	_, err = tree.GetMembershipProof([]byte("foo"))
	require.Error(t, err)

	// valid proof for real keys
	for _, key := range allkeys {
		proof, err := tree.GetMembershipProof(key)
		require.NoError(t, err)
		require.Equal(t,
			append([]byte("value_for_"), key...),
			proof.GetExist().Value,
		)

		res, err := tree.VerifyMembership(proof, key)
		require.NoError(t, err)
		require.True(t, res)
	}
}

//----------------------------------------

// Contract: !bytes.Equal(input, output) && len(input) >= len(output)
func MutateByteSlice(bytez []byte) []byte {
	// If bytez is empty, panic
	if len(bytez) == 0 {
		panic("Cannot mutate an empty bytez")
	}

	// Copy bytez
	mBytez := make([]byte, len(bytez))
	copy(mBytez, bytez)
	bytez = mBytez

	// Try a random mutation
	switch iavlrand.RandInt() % 2 {
	case 0: // Mutate a single byte
		bytez[iavlrand.RandInt()%len(bytez)] += byte(iavlrand.RandInt()%255 + 1)
	case 1: // Remove an arbitrary byte
		pos := iavlrand.RandInt() % len(bytez)
		bytez = append(bytez[:pos], bytez[pos+1:]...)
	}
	return bytez
}

func sortByteSlices(src [][]byte) [][]byte {
	bzz := byteslices(src)
	sort.Sort(bzz)
	return bzz
}

type byteslices [][]byte

func (bz byteslices) Len() int {
	return len(bz)
}

func (bz byteslices) Less(i, j int) bool {
	switch bytes.Compare(bz[i], bz[j]) {
	case -1:
		return true
	case 0, 1:
		return false
	default:
		panic("should not happen")
	}
}

func (bz byteslices) Swap(i, j int) {
	bz[j], bz[i] = bz[i], bz[j]
}
