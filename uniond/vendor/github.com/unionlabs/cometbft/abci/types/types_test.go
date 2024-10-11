package types_test

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	abci "github.com/cometbft/cometbft/abci/types"
	"github.com/cometbft/cometbft/crypto/merkle"
)

func TestHashAndProveResults(t *testing.T) {
	trs := []*abci.ExecTxResult{
		// Note, these tests rely on the first two entries being in this order.
		{Code: 0, Data: nil},
		{Code: 0, Data: []byte{}},

		{Code: 0, Data: []byte("one")},
		{Code: 14, Data: nil},
		{Code: 14, Data: []byte("foo")},
		{Code: 14, Data: []byte("bar")},
	}

	// Nil and []byte{} should produce the same bytes
	bz0, err := trs[0].Marshal()
	require.NoError(t, err)
	bz1, err := trs[1].Marshal()
	require.NoError(t, err)
	require.Equal(t, bz0, bz1)

	// Make sure that we can get a root hash from results and verify proofs.
	rs, err := abci.MarshalTxResults(trs)
	require.NoError(t, err)
	root := merkle.HashFromByteSlices(rs)
	assert.NotEmpty(t, root)

	_, proofs := merkle.ProofsFromByteSlices(rs)
	for i, tr := range trs {
		bz, err := tr.Marshal()
		require.NoError(t, err)

		valid := proofs[i].Verify(root, bz)
		require.NoError(t, valid, "%d", i)
	}
}

func TestHashDeterministicFieldsOnly(t *testing.T) {
	tr1 := abci.ExecTxResult{
		Code:      1,
		Data:      []byte("transaction"),
		Log:       "nondeterministic data: abc",
		Info:      "nondeterministic data: abc",
		GasWanted: 1000,
		GasUsed:   1000,
		Events:    []abci.Event{},
		Codespace: "nondeterministic.data.abc",
	}
	tr2 := abci.ExecTxResult{
		Code:      1,
		Data:      []byte("transaction"),
		Log:       "nondeterministic data: def",
		Info:      "nondeterministic data: def",
		GasWanted: 1000,
		GasUsed:   1000,
		Events:    []abci.Event{},
		Codespace: "nondeterministic.data.def",
	}
	r1, err := abci.MarshalTxResults([]*abci.ExecTxResult{&tr1})
	require.NoError(t, err)
	r2, err := abci.MarshalTxResults([]*abci.ExecTxResult{&tr2})
	require.NoError(t, err)
	require.Equal(t, merkle.HashFromByteSlices(r1), merkle.HashFromByteSlices(r2))
}
