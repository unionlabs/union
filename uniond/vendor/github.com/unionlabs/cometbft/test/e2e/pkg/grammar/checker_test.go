package grammar

import (
	"fmt"
	"testing"

	"github.com/stretchr/testify/require"

	abci "github.com/cometbft/cometbft/abci/types"
)

var (
	initChain       = &abci.Request{Value: &abci.Request_InitChain{InitChain: &abci.InitChainRequest{}}}
	finalizeBlock   = &abci.Request{Value: &abci.Request_FinalizeBlock{FinalizeBlock: &abci.FinalizeBlockRequest{}}}
	commit          = &abci.Request{Value: &abci.Request_Commit{Commit: &abci.CommitRequest{}}}
	offerSnapshot   = &abci.Request{Value: &abci.Request_OfferSnapshot{OfferSnapshot: &abci.OfferSnapshotRequest{}}}
	applyChunk      = &abci.Request{Value: &abci.Request_ApplySnapshotChunk{ApplySnapshotChunk: &abci.ApplySnapshotChunkRequest{}}}
	prepareProposal = &abci.Request{Value: &abci.Request_PrepareProposal{PrepareProposal: &abci.PrepareProposalRequest{}}}
	processProposal = &abci.Request{Value: &abci.Request_ProcessProposal{ProcessProposal: &abci.ProcessProposalRequest{}}}
	extendVote      = &abci.Request{Value: &abci.Request_ExtendVote{ExtendVote: &abci.ExtendVoteRequest{}}}
	gotVote         = &abci.Request{Value: &abci.Request_VerifyVoteExtension{VerifyVoteExtension: &abci.VerifyVoteExtensionRequest{}}}
)

const CleanStart = true

type ABCIExecution struct {
	abciCalls []*abci.Request
	isValid   bool
}

// consensus-exec part of executions
// consensus-exec = (inf)consensus-height
// it is part of each executions.
var consExecPart = []ABCIExecution{
	// consensus-height = finalizeBlock commit
	{[]*abci.Request{finalizeBlock, commit}, true},
	{[]*abci.Request{commit}, false},
	// consensus-height = *consensus-round finalizeBlock commit
	// consensus-height = *consensus-round finalizeBlock commit
	// consensus-round = proposer
	// proposer = *gotVote
	{[]*abci.Request{gotVote, finalizeBlock, commit}, true},
	{[]*abci.Request{gotVote, gotVote, finalizeBlock, commit}, true},
	// proposer = [prepare-proposal [process-proposal]]
	{[]*abci.Request{prepareProposal, processProposal, finalizeBlock, commit}, true},
	{[]*abci.Request{prepareProposal, finalizeBlock, commit}, true},
	// proposer = [extend]
	{[]*abci.Request{extendVote, finalizeBlock, commit}, true},
	{[]*abci.Request{gotVote, extendVote, finalizeBlock, commit}, true},
	{[]*abci.Request{gotVote, gotVote, extendVote, finalizeBlock, commit}, true},
	{[]*abci.Request{extendVote, gotVote, finalizeBlock, commit}, true},
	{[]*abci.Request{extendVote, gotVote, gotVote, finalizeBlock, commit}, true},
	{[]*abci.Request{gotVote, gotVote, extendVote, gotVote, gotVote, finalizeBlock, commit}, true},
	// proposer = *gotVote [prepare-proposal [process-proposal]]
	{[]*abci.Request{gotVote, prepareProposal, finalizeBlock, commit}, true},
	{[]*abci.Request{gotVote, gotVote, prepareProposal, finalizeBlock, commit}, true},
	{[]*abci.Request{gotVote, prepareProposal, processProposal, finalizeBlock, commit}, true},
	{[]*abci.Request{gotVote, gotVote, prepareProposal, processProposal, finalizeBlock, commit}, true},
	// proposer = *gotVote [extend]
	// same as just [extend]
	// proposer = [prepare-proposal [process-proposal]] [extend]
	{[]*abci.Request{prepareProposal, extendVote, finalizeBlock, commit}, true},
	{[]*abci.Request{prepareProposal, gotVote, extendVote, finalizeBlock, commit}, true},
	{[]*abci.Request{prepareProposal, extendVote, gotVote, finalizeBlock, commit}, true},
	{[]*abci.Request{prepareProposal, gotVote, extendVote, gotVote, finalizeBlock, commit}, true},
	{[]*abci.Request{prepareProposal, processProposal, extendVote, finalizeBlock, commit}, true},
	{[]*abci.Request{prepareProposal, processProposal, gotVote, extendVote, finalizeBlock, commit}, true},
	{[]*abci.Request{prepareProposal, processProposal, extendVote, gotVote, finalizeBlock, commit}, true},
	{[]*abci.Request{prepareProposal, processProposal, gotVote, extendVote, gotVote, finalizeBlock, commit}, true},
	// proposer = *gotVote [prepare-proposal [process-proposal]] [extend]
	{[]*abci.Request{gotVote, prepareProposal, extendVote, finalizeBlock, commit}, true},
	{[]*abci.Request{gotVote, gotVote, prepareProposal, gotVote, extendVote, finalizeBlock, commit}, true},
	{[]*abci.Request{gotVote, prepareProposal, extendVote, gotVote, finalizeBlock, commit}, true},
	{[]*abci.Request{gotVote, gotVote, prepareProposal, gotVote, extendVote, gotVote, finalizeBlock, commit}, true},
	{[]*abci.Request{gotVote, prepareProposal, processProposal, extendVote, finalizeBlock, commit}, true},
	{[]*abci.Request{gotVote, gotVote, prepareProposal, processProposal, gotVote, extendVote, finalizeBlock, commit}, true},
	{[]*abci.Request{gotVote, prepareProposal, processProposal, extendVote, gotVote, finalizeBlock, commit}, true},
	{[]*abci.Request{gotVote, gotVote, prepareProposal, processProposal, gotVote, extendVote, gotVote, finalizeBlock, commit}, true},

	// consensus-round = non-proposer
	// non-proposer = *gotVote
	// same as for proposer

	// non-proposer = [process-proposal]
	{[]*abci.Request{processProposal, finalizeBlock, commit}, true},
	// non-proposer = [extend]
	// same as for proposer

	// non-proposer = *gotVote [process-proposal]
	{[]*abci.Request{gotVote, processProposal, finalizeBlock, commit}, true},
	{[]*abci.Request{gotVote, gotVote, processProposal, finalizeBlock, commit}, true},
	// non-proposer = *gotVote [extend]
	// same as just [extend]

	// non-proposer = [process-proposal] [extend]
	{[]*abci.Request{processProposal, extendVote, finalizeBlock, commit}, true},
	{[]*abci.Request{processProposal, gotVote, extendVote, finalizeBlock, commit}, true},
	{[]*abci.Request{processProposal, extendVote, gotVote, finalizeBlock, commit}, true},
	{[]*abci.Request{processProposal, gotVote, extendVote, gotVote, finalizeBlock, commit}, true},

	// non-proposer = *gotVote [prepare-proposal [process-proposal]] [extend]
	{[]*abci.Request{gotVote, processProposal, extendVote, finalizeBlock, commit}, true},
	{[]*abci.Request{gotVote, gotVote, processProposal, gotVote, extendVote, finalizeBlock, commit}, true},
	{[]*abci.Request{gotVote, processProposal, extendVote, gotVote, finalizeBlock, commit}, true},
	{[]*abci.Request{gotVote, gotVote, processProposal, gotVote, extendVote, gotVote, finalizeBlock, commit}, true},

	{[]*abci.Request{prepareProposal, processProposal, processProposal, prepareProposal, processProposal, processProposal, processProposal, finalizeBlock, commit}, true},
}

func TestVerifyCleanStart(t *testing.T) {
	// Parts of executions specific for clean-start execution
	specificCleanStartPart := []ABCIExecution{
		// start = clean-start
		// clean-start = init-chain consensus-exec
		{[]*abci.Request{initChain}, true},
		// clean-start = state-sync consensus-exec
		// state-sync = success-sync
		{[]*abci.Request{offerSnapshot, applyChunk}, true},
		{[]*abci.Request{offerSnapshot, applyChunk, applyChunk}, true},
		{[]*abci.Request{applyChunk}, false},
		{[]*abci.Request{offerSnapshot}, false},
		// state-sync = *state-sync-attempt success-sync
		{[]*abci.Request{offerSnapshot, applyChunk, offerSnapshot, applyChunk}, true},
		{[]*abci.Request{offerSnapshot, applyChunk, applyChunk, applyChunk, offerSnapshot, applyChunk}, true},
		{[]*abci.Request{applyChunk, offerSnapshot, applyChunk}, false},
		{[]*abci.Request{offerSnapshot, offerSnapshot, applyChunk}, true},
		// extra invalid executions
		{[]*abci.Request{initChain, offerSnapshot, applyChunk}, false},
		{[]*abci.Request{}, false},
	}
	for i, part1 := range specificCleanStartPart {
		for j, part2 := range consExecPart {
			checker := NewGrammarChecker(DefaultConfig())
			execution := append(part1.abciCalls, part2.abciCalls...)
			valid := part1.isValid && part2.isValid
			result, err := checker.Verify(execution, CleanStart)
			if result == valid {
				continue
			}
			if err == nil {
				err = fmt.Errorf("grammar parsed an incorrect execution: %v", checker.getExecutionString(execution))
			}
			t.Errorf("Test %v:%v returned %v, expected %v\n%v\n", i, j, result, valid, err)
		}
	}
}

func TestVerifyRecovery(t *testing.T) {
	// Parts of executions specific for recovery execution
	specificRecoveryPart := []ABCIExecution{
		// start = recovery
		// recovery = init-chain consensus-exec
		{[]*abci.Request{initChain}, true},
		// recovery = consensus-exec
		{[]*abci.Request{}, true},
	}
	for i, part1 := range specificRecoveryPart {
		for j, part2 := range consExecPart {
			checker := NewGrammarChecker(DefaultConfig())
			execution := append(part1.abciCalls, part2.abciCalls...)
			valid := part1.isValid && part2.isValid
			result, err := checker.Verify(execution, !CleanStart)
			if result == valid {
				continue
			}
			if err == nil {
				err = fmt.Errorf("grammar parsed an incorrect execution: %v", checker.getExecutionString(execution))
			}
			t.Errorf("Test %v:%v returned %v, expected %v\n%v\n", i, j, result, valid, err)
		}
	}
}

func TestFilterLastHeight(t *testing.T) {
	reqs := []*abci.Request{initChain, finalizeBlock}
	checker := NewGrammarChecker(DefaultConfig())
	r, n := checker.filterLastHeight(reqs)
	require.Equal(t, len(r), 0)
	require.Equal(t, n, 2)
	reqs = append(reqs, commit)
	r, n = checker.filterLastHeight(reqs)
	require.Equal(t, len(r), len(reqs))
	require.Zero(t, n)
	reqs = append(reqs, []*abci.Request{prepareProposal, processProposal}...)
	r, n = checker.filterLastHeight(reqs)
	require.Equal(t, len(r), 3)
	require.Equal(t, n, 2)
}
