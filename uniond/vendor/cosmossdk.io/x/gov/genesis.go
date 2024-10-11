package gov

import (
	"context"
	"fmt"

	"cosmossdk.io/collections"
	"cosmossdk.io/x/gov/keeper"
	"cosmossdk.io/x/gov/types"
	v1 "cosmossdk.io/x/gov/types/v1"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

// InitGenesis - store genesis parameters
func InitGenesis(ctx context.Context, ak types.AccountKeeper, bk types.BankKeeper, k *keeper.Keeper, data *v1.GenesisState) error {
	err := k.ProposalID.Set(ctx, data.StartingProposalId)
	if err != nil {
		return err
	}

	err = k.Params.Set(ctx, *data.Params)
	if err != nil {
		return err
	}

	err = k.Constitution.Set(ctx, data.Constitution)
	if err != nil {
		return err
	}

	// check if the deposits pool account exists
	moduleAcc := k.GetGovernanceAccount(ctx)
	if moduleAcc == nil {
		return fmt.Errorf("%s module account has not been set", types.ModuleName)
	}

	var totalDeposits sdk.Coins
	for _, deposit := range data.Deposits {
		err := k.SetDeposit(ctx, *deposit)
		if err != nil {
			return err
		}
		totalDeposits = totalDeposits.Add(deposit.Amount...)
	}

	for _, vote := range data.Votes {
		addr, err := ak.AddressCodec().StringToBytes(vote.Voter)
		if err != nil {
			return err
		}
		err = k.Votes.Set(ctx, collections.Join(vote.ProposalId, sdk.AccAddress(addr)), *vote)
		if err != nil {
			return err
		}
	}

	for _, proposal := range data.Proposals {
		switch proposal.Status {
		case v1.StatusDepositPeriod:
			err := k.InactiveProposalsQueue.Set(ctx, collections.Join(*proposal.DepositEndTime, proposal.Id), proposal.Id)
			if err != nil {
				return err
			}
		case v1.StatusVotingPeriod:
			err := k.ActiveProposalsQueue.Set(ctx, collections.Join(*proposal.VotingEndTime, proposal.Id), proposal.Id)
			if err != nil {
				return err
			}
		}
		if err := k.Proposals.Set(ctx, proposal.Id, *proposal); err != nil {
			return err
		}
	}

	// if account has zero balance it probably means it's not set, so we set it
	balance := bk.GetAllBalances(ctx, moduleAcc.GetAddress())
	if balance.IsZero() {
		ak.SetModuleAccount(ctx, moduleAcc)
	}

	// check if total deposits equals balance, if it doesn't return an error
	if !balance.Equal(totalDeposits) {
		return fmt.Errorf("expected module account was %s but we got %s", balance.String(), totalDeposits.String())
	}
	return nil
}

// ExportGenesis - output genesis parameters
func ExportGenesis(ctx context.Context, k *keeper.Keeper) (*v1.GenesisState, error) {
	startingProposalID, err := k.ProposalID.Peek(ctx)
	if err != nil {
		return nil, err
	}

	var proposals v1.Proposals
	err = k.Proposals.Walk(ctx, nil, func(_ uint64, value v1.Proposal) (stop bool, err error) {
		proposals = append(proposals, &value)
		return false, nil
	})
	if err != nil {
		return nil, err
	}

	constitution, err := k.Constitution.Get(ctx)
	if err != nil {
		return nil, err
	}

	params, err := k.Params.Get(ctx)
	if err != nil {
		return nil, err
	}

	var proposalsDeposits v1.Deposits
	err = k.Deposits.Walk(ctx, nil, func(_ collections.Pair[uint64, sdk.AccAddress], value v1.Deposit) (stop bool, err error) {
		proposalsDeposits = append(proposalsDeposits, &value)
		return false, nil
	})
	if err != nil {
		return nil, err
	}

	// export proposals votes
	var proposalsVotes v1.Votes
	err = k.Votes.Walk(ctx, nil, func(_ collections.Pair[uint64, sdk.AccAddress], value v1.Vote) (stop bool, err error) {
		proposalsVotes = append(proposalsVotes, &value)
		return false, nil
	})
	if err != nil {
		return nil, err
	}

	return &v1.GenesisState{
		StartingProposalId: startingProposalID,
		Deposits:           proposalsDeposits,
		Votes:              proposalsVotes,
		Proposals:          proposals,
		Params:             &params,
		Constitution:       constitution,
	}, nil
}
