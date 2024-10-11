package types

import (
	"context"
	"errors"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

var _ GovHooks = MultiGovHooks{}

// combine multiple governance hooks, all hook functions are run in array sequence
type MultiGovHooks []GovHooks

func NewMultiGovHooks(hooks ...GovHooks) MultiGovHooks {
	return hooks
}

func (h MultiGovHooks) AfterProposalSubmission(ctx context.Context, proposalID uint64) error {
	var errs error
	for i := range h {
		errs = errors.Join(errs, h[i].AfterProposalSubmission(ctx, proposalID))
	}

	return errs
}

func (h MultiGovHooks) AfterProposalDeposit(ctx context.Context, proposalID uint64, depositorAddr sdk.AccAddress) error {
	var errs error
	for i := range h {
		errs = errors.Join(errs, h[i].AfterProposalDeposit(ctx, proposalID, depositorAddr))
	}
	return errs
}

func (h MultiGovHooks) AfterProposalVote(ctx context.Context, proposalID uint64, voterAddr sdk.AccAddress) error {
	var errs error
	for i := range h {
		errs = errors.Join(errs, h[i].AfterProposalVote(ctx, proposalID, voterAddr))
	}
	return errs
}

func (h MultiGovHooks) AfterProposalFailedMinDeposit(ctx context.Context, proposalID uint64) error {
	var errs error
	for i := range h {
		errs = errors.Join(errs, h[i].AfterProposalFailedMinDeposit(ctx, proposalID))
	}
	return errs
}

func (h MultiGovHooks) AfterProposalVotingPeriodEnded(ctx context.Context, proposalID uint64) error {
	var errs error
	for i := range h {
		errs = errors.Join(errs, h[i].AfterProposalVotingPeriodEnded(ctx, proposalID))
	}
	return errs
}
