package v1

import (
	"errors"
	"fmt"

	gogoprotoany "github.com/cosmos/gogoproto/types/any"
	"golang.org/x/sync/errgroup"

	"cosmossdk.io/core/address"
)

// NewGenesisState creates a new genesis state for the governance module
func NewGenesisState(startingProposalID uint64, params Params) *GenesisState {
	return &GenesisState{
		StartingProposalId: startingProposalID,
		Params:             &params,
	}
}

// DefaultGenesisState defines the default governance genesis state
func DefaultGenesisState() *GenesisState {
	return NewGenesisState(
		DefaultStartingProposalID,
		DefaultParams(),
	)
}

// Empty returns true if a GenesisState is empty
func (data GenesisState) Empty() bool {
	return data.StartingProposalId == 0 || data.Params == nil
}

// ValidateGenesis checks if gov genesis state is valid ranges
// It checks if params are in valid ranges
// It also makes sure that the provided proposal IDs are unique and
// that there are no duplicate deposit or vote records and no vote or deposits for non-existent proposals
func ValidateGenesis(ac address.Codec, data *GenesisState) error {
	if data.StartingProposalId == 0 {
		return errors.New("starting proposal id must be greater than 0")
	}

	var errGroup errgroup.Group

	// weed out duplicate proposals
	proposalIds := make(map[uint64]struct{})
	for _, p := range data.Proposals {
		if _, ok := proposalIds[p.Id]; ok {
			return fmt.Errorf("duplicate proposal id: %d", p.Id)
		}

		proposalIds[p.Id] = struct{}{}
	}

	// weed out duplicate deposits
	errGroup.Go(func() error {
		type depositKey struct {
			ProposalId uint64
			Depositor  string
		}
		depositIds := make(map[depositKey]struct{})
		for _, d := range data.Deposits {
			if _, ok := proposalIds[d.ProposalId]; !ok {
				return fmt.Errorf("deposit %v has non-existent proposal id: %d", d, d.ProposalId)
			}

			dk := depositKey{d.ProposalId, d.Depositor}
			if _, ok := depositIds[dk]; ok {
				return fmt.Errorf("duplicate deposit: %v", d)
			}

			depositIds[dk] = struct{}{}
		}

		return nil
	})

	// weed out duplicate votes
	errGroup.Go(func() error {
		type voteKey struct {
			ProposalId uint64
			Voter      string
		}
		voteIds := make(map[voteKey]struct{})
		for _, v := range data.Votes {
			if _, ok := proposalIds[v.ProposalId]; !ok {
				return fmt.Errorf("vote %v has non-existent proposal id: %d", v, v.ProposalId)
			}

			vk := voteKey{v.ProposalId, v.Voter}
			if _, ok := voteIds[vk]; ok {
				return fmt.Errorf("duplicate vote: %v", v)
			}

			voteIds[vk] = struct{}{}
		}

		return nil
	})

	// verify params
	errGroup.Go(func() error {
		return data.Params.ValidateBasic(ac)
	})

	return errGroup.Wait()
}

var _ gogoprotoany.UnpackInterfacesMessage = GenesisState{}

// UnpackInterfaces implements UnpackInterfacesMessage.UnpackInterfaces
func (data GenesisState) UnpackInterfaces(unpacker gogoprotoany.AnyUnpacker) error {
	for _, p := range data.Proposals {
		err := p.UnpackInterfaces(unpacker)
		if err != nil {
			return err
		}
	}
	return nil
}
