package baseapp

import (
	"time"

	abci "github.com/cometbft/cometbft/abci/types"

	"cosmossdk.io/core/comet"
)

var _ comet.BlockInfo = (*cometInfo)(nil)

// CometInfo defines the properties provided by comet to the application
type cometInfo struct {
	Misbehavior     []abci.Misbehavior
	ValidatorsHash  []byte
	ProposerAddress []byte
	LastCommit      abci.CommitInfo
}

func (r cometInfo) GetEvidence() comet.EvidenceList {
	return evidenceWrapper{evidence: r.Misbehavior}
}

func (r cometInfo) GetValidatorsHash() []byte {
	return r.ValidatorsHash
}

func (r cometInfo) GetProposerAddress() []byte {
	return r.ProposerAddress
}

func (r cometInfo) GetLastCommit() comet.CommitInfo {
	return commitInfoWrapper{r.LastCommit}
}

type evidenceWrapper struct {
	evidence []abci.Misbehavior
}

func (e evidenceWrapper) Len() int {
	return len(e.evidence)
}

func (e evidenceWrapper) Get(i int) comet.Evidence {
	return misbehaviorWrapper{e.evidence[i]}
}

// commitInfoWrapper is a wrapper around abci.CommitInfo that implements CommitInfo interface
type commitInfoWrapper struct {
	abci.CommitInfo
}

var _ comet.CommitInfo = (*commitInfoWrapper)(nil)

func (c commitInfoWrapper) Round() int32 {
	return c.CommitInfo.Round
}

func (c commitInfoWrapper) Votes() comet.VoteInfos {
	return abciVoteInfoWrapper{c.CommitInfo.Votes}
}

// abciVoteInfoWrapper is a wrapper around abci.VoteInfo that implements VoteInfos interface
type abciVoteInfoWrapper struct {
	votes []abci.VoteInfo
}

var _ comet.VoteInfos = (*abciVoteInfoWrapper)(nil)

func (e abciVoteInfoWrapper) Len() int {
	return len(e.votes)
}

func (e abciVoteInfoWrapper) Get(i int) comet.VoteInfo {
	return voteInfoWrapper{e.votes[i]}
}

// voteInfoWrapper is a wrapper around abci.VoteInfo that implements VoteInfo interface
type voteInfoWrapper struct {
	abci.VoteInfo
}

var _ comet.VoteInfo = (*voteInfoWrapper)(nil)

func (v voteInfoWrapper) GetBlockIDFlag() comet.BlockIDFlag {
	return comet.BlockIDFlag(v.VoteInfo.BlockIdFlag)
}

func (v voteInfoWrapper) Validator() comet.Validator {
	return validatorWrapper{v.VoteInfo.Validator}
}

// validatorWrapper is a wrapper around abci.Validator that implements Validator interface
type validatorWrapper struct {
	abci.Validator
}

var _ comet.Validator = (*validatorWrapper)(nil)

func (v validatorWrapper) Address() []byte {
	return v.Validator.Address
}

func (v validatorWrapper) Power() int64 {
	return v.Validator.Power
}

type misbehaviorWrapper struct {
	abci.Misbehavior
}

func (m misbehaviorWrapper) Type() comet.MisbehaviorType {
	return comet.MisbehaviorType(m.Misbehavior.Type)
}

func (m misbehaviorWrapper) Height() int64 {
	return m.Misbehavior.Height
}

func (m misbehaviorWrapper) Validator() comet.Validator {
	return validatorWrapper{m.Misbehavior.Validator}
}

func (m misbehaviorWrapper) Time() time.Time {
	return m.Misbehavior.Time
}

func (m misbehaviorWrapper) TotalVotingPower() int64 {
	return m.Misbehavior.TotalVotingPower
}

type prepareProposalInfo struct {
	*abci.RequestPrepareProposal
}

var _ comet.BlockInfo = (*prepareProposalInfo)(nil)

func (r prepareProposalInfo) GetEvidence() comet.EvidenceList {
	return evidenceWrapper{r.Misbehavior}
}

func (r prepareProposalInfo) GetValidatorsHash() []byte {
	return r.NextValidatorsHash
}

func (r prepareProposalInfo) GetProposerAddress() []byte {
	return r.RequestPrepareProposal.ProposerAddress
}

func (r prepareProposalInfo) GetLastCommit() comet.CommitInfo {
	return extendedCommitInfoWrapper{r.RequestPrepareProposal.LocalLastCommit}
}

var _ comet.BlockInfo = (*prepareProposalInfo)(nil)

type extendedCommitInfoWrapper struct {
	abci.ExtendedCommitInfo
}

var _ comet.CommitInfo = (*extendedCommitInfoWrapper)(nil)

func (e extendedCommitInfoWrapper) Round() int32 {
	return e.ExtendedCommitInfo.Round
}

func (e extendedCommitInfoWrapper) Votes() comet.VoteInfos {
	return extendedVoteInfoWrapperList{e.ExtendedCommitInfo.Votes}
}

type extendedVoteInfoWrapperList struct {
	votes []abci.ExtendedVoteInfo
}

var _ comet.VoteInfos = (*extendedVoteInfoWrapperList)(nil)

func (e extendedVoteInfoWrapperList) Len() int {
	return len(e.votes)
}

func (e extendedVoteInfoWrapperList) Get(i int) comet.VoteInfo {
	return extendedVoteInfoWrapper{e.votes[i]}
}

type extendedVoteInfoWrapper struct {
	abci.ExtendedVoteInfo
}

var _ comet.VoteInfo = (*extendedVoteInfoWrapper)(nil)

func (e extendedVoteInfoWrapper) GetBlockIDFlag() comet.BlockIDFlag {
	return comet.BlockIDFlag(e.ExtendedVoteInfo.BlockIdFlag)
}

func (e extendedVoteInfoWrapper) Validator() comet.Validator {
	return validatorWrapper{e.ExtendedVoteInfo.Validator}
}
