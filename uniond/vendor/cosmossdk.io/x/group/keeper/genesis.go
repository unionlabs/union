package keeper

import (
	"context"
	"encoding/json"

	"cosmossdk.io/errors"
	"cosmossdk.io/x/group"

	"github.com/cosmos/cosmos-sdk/codec"
)

// InitGenesis initializes the group module's genesis state.
func (k Keeper) InitGenesis(ctx context.Context, cdc codec.JSONCodec, data json.RawMessage) error {
	var genesisState group.GenesisState
	cdc.MustUnmarshalJSON(data, &genesisState)

	store := k.KVStoreService.OpenKVStore(ctx)

	if err := k.groupTable.Import(store, genesisState.Groups, genesisState.GroupSeq); err != nil {
		return errors.Wrap(err, "groups")
	}

	if err := k.groupMemberTable.Import(store, genesisState.GroupMembers, 0); err != nil {
		return errors.Wrap(err, "group members")
	}

	if err := k.groupPolicyTable.Import(store, genesisState.GroupPolicies, 0); err != nil {
		return errors.Wrap(err, "group policies")
	}

	if err := k.groupPolicySeq.InitVal(store, genesisState.GroupPolicySeq); err != nil {
		return errors.Wrap(err, "group policy account seq")
	}

	if err := k.proposalTable.Import(store, genesisState.Proposals, genesisState.ProposalSeq); err != nil {
		return errors.Wrap(err, "proposals")
	}

	if err := k.voteTable.Import(store, genesisState.Votes, 0); err != nil {
		return errors.Wrap(err, "votes")
	}

	return nil
}

// ExportGenesis returns the group module's exported genesis.
func (k Keeper) ExportGenesis(ctx context.Context, _ codec.JSONCodec) (*group.GenesisState, error) {
	genesisState := group.NewGenesisState()

	var groups []*group.GroupInfo

	store := k.KVStoreService.OpenKVStore(ctx)

	groupSeq, err := k.groupTable.Export(store, &groups)
	if err != nil {
		return nil, errors.Wrap(err, "groups")
	}
	genesisState.Groups = groups
	genesisState.GroupSeq = groupSeq

	var groupMembers []*group.GroupMember
	_, err = k.groupMemberTable.Export(store, &groupMembers)
	if err != nil {
		return nil, errors.Wrap(err, "group members")
	}
	genesisState.GroupMembers = groupMembers

	var groupPolicies []*group.GroupPolicyInfo
	_, err = k.groupPolicyTable.Export(store, &groupPolicies)
	if err != nil {
		return nil, errors.Wrap(err, "group policies")
	}
	genesisState.GroupPolicies = groupPolicies
	genesisState.GroupPolicySeq = k.groupPolicySeq.CurVal(store)

	var proposals []*group.Proposal
	proposalSeq, err := k.proposalTable.Export(store, &proposals)
	if err != nil {
		return nil, errors.Wrap(err, "proposals")
	}
	genesisState.Proposals = proposals
	genesisState.ProposalSeq = proposalSeq

	var votes []*group.Vote
	_, err = k.voteTable.Export(store, &votes)
	if err != nil {
		return nil, errors.Wrap(err, "votes")
	}
	genesisState.Votes = votes

	return genesisState, nil
}
