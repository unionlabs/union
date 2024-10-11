package mock

import (
	"testing"

	cmtproto "github.com/cometbft/cometbft/api/cometbft/types/v1"
	"github.com/stretchr/testify/require"
)

func TestGetPubKey(t *testing.T) {
	pv := NewPV()
	pb, err := pv.GetPubKey()
	require.NoError(t, err)
	require.NotNil(t, pb)
}

func TestSignVote(t *testing.T) {
	pv := NewPV()
	v := cmtproto.Vote{}
	err := pv.SignVote("chain-id", &v, false)
	require.NoError(t, err)
	require.NotNil(t, v.Signature)
}

func TestSignProposal(t *testing.T) {
	pv := NewPV()
	p := cmtproto.Proposal{}
	err := pv.SignProposal("chain-id", &p)
	require.NoError(t, err)
	require.NotNil(t, p.Signature)
}
