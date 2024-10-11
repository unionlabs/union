package types

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	abci "github.com/cometbft/cometbft/abci/types"
)

func TestABCIResults(t *testing.T) {
	a := &abci.ExecTxResult{Code: 0, Data: nil}
	b := &abci.ExecTxResult{Code: 0, Data: []byte{}}
	c := &abci.ExecTxResult{Code: 0, Data: []byte("one")}
	d := &abci.ExecTxResult{Code: 14, Data: nil}
	e := &abci.ExecTxResult{Code: 14, Data: []byte("foo")}
	f := &abci.ExecTxResult{Code: 14, Data: []byte("bar")}

	// Nil and []byte{} should produce the same bytes
	bzA, err := a.Marshal()
	require.NoError(t, err)
	bzB, err := b.Marshal()
	require.NoError(t, err)

	require.Equal(t, bzA, bzB)

	// a and b should be the same, don't go in results.
	results := ABCIResults{a, c, d, e, f}

	// Make sure each result serializes differently
	last := []byte{}
	assert.Equal(t, last, bzA) // first one is empty
	for i, res := range results[1:] {
		bz, err := res.Marshal()
		require.NoError(t, err)

		assert.NotEqual(t, last, bz, "%d", i)
		last = bz
	}

	// Make sure that we can get a root hash from results and verify proofs.
	root := results.Hash()
	assert.NotEmpty(t, root)

	for i, res := range results {
		bz, err := res.Marshal()
		require.NoError(t, err)

		proof := results.ProveResult(i)
		valid := proof.Verify(root, bz)
		require.NoError(t, valid, "%d", i)
	}
}
