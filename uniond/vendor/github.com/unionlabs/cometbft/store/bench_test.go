package store

import (
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cometbft/cometbft/internal/test"
	"github.com/cometbft/cometbft/types"
	cmttime "github.com/cometbft/cometbft/types/time"
)

// TestLoadBlockExtendedCommit tests loading the extended commit for a previously
// saved block. The load method should return nil when only a commit was saved and
// return the extended commit otherwise.
func BenchmarkRepeatedLoadSeenCommitSameBlock(b *testing.B) {
	state, bs, _, _, cleanup, _ := makeStateAndBlockStoreAndIndexers()
	defer cleanup()
	h := bs.Height() + 1
	block := state.MakeBlock(h, test.MakeNTxs(h, 10), new(types.Commit), nil, state.Validators.GetProposer().Address)
	seenCommit := makeTestExtCommitWithNumSigs(block.Header.Height, cmttime.Now(), 100).ToCommit()
	ps, err := block.MakePartSet(types.BlockPartSizeBytes)
	require.NoError(b, err)
	bs.SaveBlock(block, ps, seenCommit)

	// sanity check
	res := bs.LoadSeenCommit(block.Height)
	require.Equal(b, seenCommit, res)

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		res := bs.LoadSeenCommit(block.Height)
		require.NotNil(b, res)
	}
}
