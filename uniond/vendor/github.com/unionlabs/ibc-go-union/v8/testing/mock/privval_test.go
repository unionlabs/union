package mock_test

import (
	"testing"

	"github.com/stretchr/testify/require"

	tmproto "github.com/cometbft/cometbft/api/cometbft/types/v1"
	tmtypes "github.com/cometbft/cometbft/types"

	"github.com/cosmos/ibc-go/v8/testing/mock"
)

const chainID = "testChain"

func TestGetPubKey(t *testing.T) {
	pv := mock.NewPV()
	pk, err := pv.GetPubKey()
	require.NoError(t, err)
	require.Equal(t, "ed25519", pk.Type())
}

func TestSignVote(t *testing.T) {
	pv := mock.NewPV()
	pk, _ := pv.GetPubKey()

	vote := &tmproto.Vote{Height: 2}
	err := pv.SignVote(chainID, vote)
	require.NoError(t, err)

	msg := tmtypes.VoteSignBytes(chainID, vote)
	ok := pk.VerifySignature(msg, vote.Signature)
	require.True(t, ok)
}

func TestSignProposal(t *testing.T) {
	pv := mock.NewPV()
	pk, _ := pv.GetPubKey()

	proposal := &tmproto.Proposal{Round: 2}
	err := pv.SignProposal(chainID, proposal)
	require.NoError(t, err)

	msg := tmtypes.ProposalSignBytes(chainID, proposal)
	ok := pk.VerifySignature(msg, proposal.Signature)
	require.True(t, ok)
}
