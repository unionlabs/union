package test

import (
	"testing"
	"time"

	"github.com/stretchr/testify/require"

	"github.com/cometbft/cometbft/crypto"
	"github.com/cometbft/cometbft/crypto/tmhash"
	"github.com/cometbft/cometbft/types"
	"github.com/cometbft/cometbft/version"
)

const (
	DefaultTestChainID = "test-chain"
)

var (
	DefaultTestTime = time.Date(2020, 1, 1, 0, 0, 0, 0, time.UTC)
)

func RandomAddress() []byte {
	return crypto.CRandBytes(crypto.AddressSize)
}

func RandomHash() []byte {
	return crypto.CRandBytes(tmhash.Size)
}

func MakeBlockID() types.BlockID {
	return MakeBlockIDWithHash(RandomHash())
}

func MakeBlockIDWithHash(hash []byte) types.BlockID {
	return types.BlockID{
		Hash: hash,
		PartSetHeader: types.PartSetHeader{
			Total: 100,
			Hash:  RandomHash(),
		},
	}
}

// MakeHeader fills the rest of the contents of the header such that it passes
// validate basic
func MakeHeader(t *testing.T, h *types.Header) *types.Header {
	t.Helper()
	if h.Version.Block == 0 {
		h.Version.Block = version.BlockProtocol
	}
	if h.Height == 0 {
		h.Height = 1
	}
	if h.LastBlockID.IsZero() {
		h.LastBlockID = MakeBlockID()
	}
	if h.ChainID == "" {
		h.ChainID = DefaultTestChainID
	}
	if len(h.LastCommitHash) == 0 {
		h.LastCommitHash = RandomHash()
	}
	if len(h.DataHash) == 0 {
		h.DataHash = RandomHash()
	}
	if len(h.ValidatorsHash) == 0 {
		h.ValidatorsHash = RandomHash()
	}
	if len(h.NextValidatorsHash) == 0 {
		h.NextValidatorsHash = RandomHash()
	}
	if len(h.ConsensusHash) == 0 {
		h.ConsensusHash = RandomHash()
	}
	if len(h.AppHash) == 0 {
		h.AppHash = RandomHash()
	}
	if len(h.LastResultsHash) == 0 {
		h.LastResultsHash = RandomHash()
	}
	if len(h.EvidenceHash) == 0 {
		h.EvidenceHash = RandomHash()
	}
	if len(h.ProposerAddress) == 0 {
		h.ProposerAddress = RandomAddress()
	}

	require.NoError(t, h.ValidateBasic())

	return h
}
