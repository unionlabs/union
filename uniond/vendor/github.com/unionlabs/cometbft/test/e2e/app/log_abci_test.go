package app

import (
	"testing"

	"github.com/stretchr/testify/require"

	abci "github.com/cometbft/cometbft/abci/types"
)

// Tests for logging each type of requests.
func TestLogging(t *testing.T) {
	reqs := []*abci.Request{
		{Value: &abci.Request_Echo{Echo: &abci.EchoRequest{}}},
		{Value: &abci.Request_Flush{Flush: &abci.FlushRequest{}}},
		{Value: &abci.Request_Info{Info: &abci.InfoRequest{}}},
		{Value: &abci.Request_InitChain{InitChain: &abci.InitChainRequest{}}},
		{Value: &abci.Request_Query{Query: &abci.QueryRequest{}}},
		{Value: &abci.Request_FinalizeBlock{FinalizeBlock: &abci.FinalizeBlockRequest{}}},
		{Value: &abci.Request_CheckTx{CheckTx: &abci.CheckTxRequest{}}},
		{Value: &abci.Request_Commit{Commit: &abci.CommitRequest{}}},
		{Value: &abci.Request_ListSnapshots{ListSnapshots: &abci.ListSnapshotsRequest{}}},
		{Value: &abci.Request_OfferSnapshot{OfferSnapshot: &abci.OfferSnapshotRequest{}}},
		{Value: &abci.Request_LoadSnapshotChunk{LoadSnapshotChunk: &abci.LoadSnapshotChunkRequest{}}},
		{Value: &abci.Request_ApplySnapshotChunk{ApplySnapshotChunk: &abci.ApplySnapshotChunkRequest{}}},
		{Value: &abci.Request_PrepareProposal{PrepareProposal: &abci.PrepareProposalRequest{}}},
		{Value: &abci.Request_ProcessProposal{ProcessProposal: &abci.ProcessProposalRequest{}}},
		{Value: &abci.Request_ExtendVote{ExtendVote: &abci.ExtendVoteRequest{}}},
		{Value: &abci.Request_VerifyVoteExtension{VerifyVoteExtension: &abci.VerifyVoteExtensionRequest{}}},
	}
	for _, r := range reqs {
		s, err := GetABCIRequestString(r)
		require.NoError(t, err)
		rr, err := GetABCIRequestFromString(s)
		require.NoError(t, err)
		require.Equal(t, r, rr)
	}
}
