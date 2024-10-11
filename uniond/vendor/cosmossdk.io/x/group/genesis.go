package group

import (
	"fmt"

	gogoprotoany "github.com/cosmos/gogoproto/types/any"

	errorsmod "cosmossdk.io/errors"

	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

// NewGenesisState creates a new genesis state with default values.
func NewGenesisState() *GenesisState {
	return &GenesisState{}
}

// Validate performs basic genesis state validation returning an error upon any
// failure.
func (s GenesisState) Validate() error {
	groups := make(map[uint64]GroupInfo)
	groupPolicies := make(map[string]GroupPolicyInfo)
	groupMembers := make(map[uint64]GroupMember)
	proposals := make(map[uint64]Proposal)

	for _, g := range s.Groups {
		if err := g.ValidateBasic(); err != nil {
			return errorsmod.Wrap(err, "Group validation failed")
		}
		groups[g.Id] = *g
	}

	for _, g := range s.GroupPolicies {

		// check that group with group policy's GroupId exists
		if _, exists := groups[g.GroupId]; !exists {
			return errorsmod.Wrap(sdkerrors.ErrNotFound, fmt.Sprintf("group with GroupId %d doesn't exist", g.GroupId))
		}

		if err := g.ValidateBasic(); err != nil {
			return errorsmod.Wrap(err, "GroupPolicy validation failed")
		}
		groupPolicies[g.Address] = *g
	}

	for _, g := range s.GroupMembers {

		// check that group with group member's GroupId exists
		if _, exists := groups[g.GroupId]; !exists {
			return errorsmod.Wrap(sdkerrors.ErrNotFound, fmt.Sprintf("group member with GroupId %d doesn't exist", g.GroupId))
		}

		if err := g.ValidateBasic(); err != nil {
			return errorsmod.Wrap(err, "GroupMember validation failed")
		}
		groupMembers[g.GroupId] = *g
	}

	for _, p := range s.Proposals {

		// check that group policy with proposal address exists
		if _, exists := groupPolicies[p.GroupPolicyAddress]; !exists {
			return errorsmod.Wrap(sdkerrors.ErrNotFound, fmt.Sprintf("group policy account with address %s doesn't correspond to proposal address", p.GroupPolicyAddress))
		}

		if err := p.ValidateBasic(); err != nil {
			return errorsmod.Wrap(err, "Proposal validation failed")
		}
		proposals[p.Id] = *p
	}

	for _, v := range s.Votes {

		if err := v.ValidateBasic(); err != nil {
			return errorsmod.Wrap(err, "Vote validation failed")
		}

		// check that proposal exists
		if _, exists := proposals[v.ProposalId]; !exists {
			return errorsmod.Wrap(sdkerrors.ErrNotFound, fmt.Sprintf("proposal with ProposalId %d doesn't exist", v.ProposalId))
		}
	}
	return nil
}

// UnpackInterfaces implements UnpackInterfacesMessage.UnpackInterfaces
func (s GenesisState) UnpackInterfaces(unpacker gogoprotoany.AnyUnpacker) error {
	for _, g := range s.GroupPolicies {
		err := g.UnpackInterfaces(unpacker)
		if err != nil {
			return err
		}
	}
	for _, p := range s.Proposals {
		err := p.UnpackInterfaces(unpacker)
		if err != nil {
			return err
		}
	}
	return nil
}
