package keeper

import (
	"context"
	"fmt"
	"time"

	"cosmossdk.io/core/appmodule"
	errorsmod "cosmossdk.io/errors"
	"cosmossdk.io/x/group"
	"cosmossdk.io/x/group/errors"
	"cosmossdk.io/x/group/internal/orm"

	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

const (
	// Group Table
	GroupTablePrefix        byte = 0x0
	GroupTableSeqPrefix     byte = 0x1
	GroupByAdminIndexPrefix byte = 0x2

	// Group Member Table
	GroupMemberTablePrefix         byte = 0x10
	GroupMemberByGroupIndexPrefix  byte = 0x11
	GroupMemberByMemberIndexPrefix byte = 0x12

	// Group Policy Table
	GroupPolicyTablePrefix        byte = 0x20
	GroupPolicyTableSeqPrefix     byte = 0x21
	GroupPolicyByGroupIndexPrefix byte = 0x22
	GroupPolicyByAdminIndexPrefix byte = 0x23

	// Proposal Table
	ProposalTablePrefix              byte = 0x30
	ProposalTableSeqPrefix           byte = 0x31
	ProposalByGroupPolicyIndexPrefix byte = 0x32
	ProposalsByVotingPeriodEndPrefix byte = 0x33

	// Vote Table
	VoteTablePrefix           byte = 0x40
	VoteByProposalIndexPrefix byte = 0x41
	VoteByVoterIndexPrefix    byte = 0x42
)

type Keeper struct {
	appmodule.Environment
	accKeeper group.AccountKeeper

	// Group Table
	groupTable        orm.AutoUInt64Table
	groupByAdminIndex orm.Index

	// Group Member Table
	groupMemberTable         orm.PrimaryKeyTable
	groupMemberByGroupIndex  orm.Index
	groupMemberByMemberIndex orm.Index

	// Group Policy Table
	groupPolicySeq          orm.Sequence
	groupPolicyTable        orm.PrimaryKeyTable
	groupPolicyByGroupIndex orm.Index
	groupPolicyByAdminIndex orm.Index

	// Proposal Table
	proposalTable              orm.AutoUInt64Table
	proposalByGroupPolicyIndex orm.Index
	proposalsByVotingPeriodEnd orm.Index

	// Vote Table
	voteTable           orm.PrimaryKeyTable
	voteByProposalIndex orm.Index
	voteByVoterIndex    orm.Index

	config group.Config

	cdc codec.Codec
}

// NewKeeper creates a new group keeper.
func NewKeeper(env appmodule.Environment, cdc codec.Codec, accKeeper group.AccountKeeper, config group.Config) Keeper {
	k := Keeper{
		Environment: env,
		accKeeper:   accKeeper,
		cdc:         cdc,
	}

	/*
		Example of group params:
		config.MaxExecutionPeriod = "1209600s" 	// example execution period in seconds
		config.MaxMetadataLen = 1000 			// example metadata length in bytes
		config.MaxProposalTitleLen = 255 		// example max title length in characters
		config.MaxProposalSummaryLen = 10200 	// example max summary length in characters
	*/

	defaultConfig := group.DefaultConfig()
	// Set the max execution period if not set by app developer.
	if config.MaxExecutionPeriod <= 0 {
		config.MaxExecutionPeriod = defaultConfig.MaxExecutionPeriod
	}
	// If MaxMetadataLen not set by app developer, set to default value.
	if config.MaxMetadataLen <= 0 {
		config.MaxMetadataLen = defaultConfig.MaxMetadataLen
	}
	// If MaxProposalTitleLen not set by app developer, set to default value.
	if config.MaxProposalTitleLen <= 0 {
		config.MaxProposalTitleLen = defaultConfig.MaxProposalTitleLen
	}
	// If MaxProposalSummaryLen not set by app developer, set to default value.
	if config.MaxProposalSummaryLen <= 0 {
		config.MaxProposalSummaryLen = defaultConfig.MaxProposalSummaryLen
	}
	k.config = config

	groupTable, err := orm.NewAutoUInt64Table([2]byte{GroupTablePrefix}, GroupTableSeqPrefix, &group.GroupInfo{}, cdc, k.accKeeper.AddressCodec())
	if err != nil {
		panic(err.Error())
	}
	k.groupByAdminIndex, err = orm.NewIndex(groupTable, GroupByAdminIndexPrefix, func(val interface{}) ([]interface{}, error) {
		addr, err := accKeeper.AddressCodec().StringToBytes(val.(*group.GroupInfo).Admin)
		if err != nil {
			return nil, err
		}
		return []interface{}{addr}, nil
	}, []byte{})
	if err != nil {
		panic(err.Error())
	}
	k.groupTable = *groupTable

	// Group Member Table
	groupMemberTable, err := orm.NewPrimaryKeyTable([2]byte{GroupMemberTablePrefix}, &group.GroupMember{}, cdc, k.accKeeper.AddressCodec())
	if err != nil {
		panic(err.Error())
	}
	k.groupMemberByGroupIndex, err = orm.NewIndex(groupMemberTable, GroupMemberByGroupIndexPrefix, func(val interface{}) ([]interface{}, error) {
		group := val.(*group.GroupMember).GroupId
		return []interface{}{group}, nil
	}, group.GroupMember{}.GroupId)
	if err != nil {
		panic(err.Error())
	}
	k.groupMemberByMemberIndex, err = orm.NewIndex(groupMemberTable, GroupMemberByMemberIndexPrefix, func(val interface{}) ([]interface{}, error) {
		memberAddr := val.(*group.GroupMember).Member.Address
		addr, err := accKeeper.AddressCodec().StringToBytes(memberAddr)
		if err != nil {
			return nil, err
		}
		return []interface{}{addr}, nil
	}, []byte{})
	if err != nil {
		panic(err.Error())
	}
	k.groupMemberTable = *groupMemberTable

	// Group Policy Table
	k.groupPolicySeq = orm.NewSequence(GroupPolicyTableSeqPrefix)
	groupPolicyTable, err := orm.NewPrimaryKeyTable([2]byte{GroupPolicyTablePrefix}, &group.GroupPolicyInfo{}, cdc, k.accKeeper.AddressCodec())
	if err != nil {
		panic(err.Error())
	}
	k.groupPolicyByGroupIndex, err = orm.NewIndex(groupPolicyTable, GroupPolicyByGroupIndexPrefix, func(value interface{}) ([]interface{}, error) {
		return []interface{}{value.(*group.GroupPolicyInfo).GroupId}, nil
	}, group.GroupPolicyInfo{}.GroupId)
	if err != nil {
		panic(err.Error())
	}
	k.groupPolicyByAdminIndex, err = orm.NewIndex(groupPolicyTable, GroupPolicyByAdminIndexPrefix, func(value interface{}) ([]interface{}, error) {
		admin := value.(*group.GroupPolicyInfo).Admin
		addr, err := accKeeper.AddressCodec().StringToBytes(admin)
		if err != nil {
			return nil, err
		}
		return []interface{}{addr}, nil
	}, []byte{})
	if err != nil {
		panic(err.Error())
	}
	k.groupPolicyTable = *groupPolicyTable

	// Proposal Table
	proposalTable, err := orm.NewAutoUInt64Table([2]byte{ProposalTablePrefix}, ProposalTableSeqPrefix, &group.Proposal{}, cdc, k.accKeeper.AddressCodec())
	if err != nil {
		panic(err.Error())
	}
	k.proposalByGroupPolicyIndex, err = orm.NewIndex(proposalTable, ProposalByGroupPolicyIndexPrefix, func(value interface{}) ([]interface{}, error) {
		account := value.(*group.Proposal).GroupPolicyAddress
		addr, err := accKeeper.AddressCodec().StringToBytes(account)
		if err != nil {
			return nil, err
		}
		return []interface{}{addr}, nil
	}, []byte{})
	if err != nil {
		panic(err.Error())
	}
	k.proposalsByVotingPeriodEnd, err = orm.NewIndex(proposalTable, ProposalsByVotingPeriodEndPrefix, func(value interface{}) ([]interface{}, error) {
		votingPeriodEnd := value.(*group.Proposal).VotingPeriodEnd
		return []interface{}{sdk.FormatTimeBytes(votingPeriodEnd)}, nil
	}, []byte{})
	if err != nil {
		panic(err.Error())
	}
	k.proposalTable = *proposalTable

	// Vote Table
	voteTable, err := orm.NewPrimaryKeyTable([2]byte{VoteTablePrefix}, &group.Vote{}, cdc, k.accKeeper.AddressCodec())
	if err != nil {
		panic(err.Error())
	}
	k.voteByProposalIndex, err = orm.NewIndex(voteTable, VoteByProposalIndexPrefix, func(value interface{}) ([]interface{}, error) {
		return []interface{}{value.(*group.Vote).ProposalId}, nil
	}, group.Vote{}.ProposalId)
	if err != nil {
		panic(err.Error())
	}
	k.voteByVoterIndex, err = orm.NewIndex(voteTable, VoteByVoterIndexPrefix, func(value interface{}) ([]interface{}, error) {
		addr, err := accKeeper.AddressCodec().StringToBytes(value.(*group.Vote).Voter)
		if err != nil {
			return nil, err
		}
		return []interface{}{addr}, nil
	}, []byte{})
	if err != nil {
		panic(err.Error())
	}
	k.voteTable = *voteTable

	return k
}

// GetGroupSequence returns the current value of the group table sequence
func (k Keeper) GetGroupSequence(ctx context.Context) uint64 {
	return k.groupTable.Sequence().CurVal(k.KVStoreService.OpenKVStore(ctx))
}

// GetGroupPolicySeq returns the current value of the group policy table sequence
func (k Keeper) GetGroupPolicySeq(ctx sdk.Context) uint64 {
	return k.groupPolicySeq.CurVal(k.KVStoreService.OpenKVStore(ctx))
}

// proposalsByVPEnd returns all proposals whose voting_period_end is after the `endTime` time argument.
func (k Keeper) proposalsByVPEnd(ctx context.Context, endTime time.Time) (proposals []group.Proposal, err error) {
	timeBytes := sdk.FormatTimeBytes(endTime)
	it, err := k.proposalsByVotingPeriodEnd.PrefixScan(k.KVStoreService.OpenKVStore(ctx), nil, timeBytes)
	if err != nil {
		return proposals, err
	}
	defer it.Close()

	for {
		// Important: this following line cannot be outside of the for loop.
		// It seems that when one unmarshals into the same `group.Proposal`
		// reference, then gogoproto somehow "adds" the new bytes to the old
		// object for some fields. When running simulations, for proposals with
		// each 1-2 proposers, after a couple of loop iterations we got to a
		// proposal with 60k+ proposers.
		// So we're declaring a local variable that gets GCed.
		//
		// Also see `x/group/types/proposal_test.go`, TestGogoUnmarshalProposal().
		var proposal group.Proposal
		_, err := it.LoadNext(&proposal)
		if errors.ErrORMIteratorDone.Is(err) {
			break
		}
		if err != nil {
			return proposals, err
		}
		proposals = append(proposals, proposal)
	}

	return proposals, nil
}

// pruneProposal deletes a proposal from state.
func (k Keeper) pruneProposal(ctx context.Context, proposalID uint64) error {
	err := k.proposalTable.Delete(k.KVStoreService.OpenKVStore(ctx), proposalID)
	if err != nil {
		return err
	}

	k.Logger.Debug(fmt.Sprintf("Pruned proposal %d", proposalID))
	return nil
}

// abortProposals iterates through all proposals by group policy index
// and marks submitted proposals as aborted.
func (k Keeper) abortProposals(ctx context.Context, groupPolicyAddr sdk.AccAddress) error {
	proposals, err := k.proposalsByGroupPolicy(ctx, groupPolicyAddr)
	if err != nil {
		return err
	}

	for _, proposalInfo := range proposals {
		// Mark all proposals still in the voting phase as aborted.
		if proposalInfo.Status == group.PROPOSAL_STATUS_SUBMITTED {
			proposalInfo.Status = group.PROPOSAL_STATUS_ABORTED

			if err := k.proposalTable.Update(k.KVStoreService.OpenKVStore(ctx), proposalInfo.Id, &proposalInfo); err != nil {
				return err
			}
		}
	}
	return nil
}

// proposalsByGroupPolicy returns all proposals for a given group policy.
func (k Keeper) proposalsByGroupPolicy(ctx context.Context, groupPolicyAddr sdk.AccAddress) ([]group.Proposal, error) {
	proposalIt, err := k.proposalByGroupPolicyIndex.Get(k.KVStoreService.OpenKVStore(ctx), groupPolicyAddr.Bytes())
	if err != nil {
		return nil, err
	}
	defer proposalIt.Close()

	var proposals []group.Proposal
	for {
		var proposalInfo group.Proposal
		_, err = proposalIt.LoadNext(&proposalInfo)
		if errors.ErrORMIteratorDone.Is(err) {
			break
		}
		if err != nil {
			return proposals, err
		}

		proposals = append(proposals, proposalInfo)
	}
	return proposals, nil
}

// pruneVotes prunes all votes for a proposal from state.
func (k Keeper) pruneVotes(ctx context.Context, proposalID uint64) error {
	votes, err := k.votesByProposal(ctx, proposalID)
	if err != nil {
		return err
	}

	for _, v := range votes {
		err = k.voteTable.Delete(k.KVStoreService.OpenKVStore(ctx), &v)
		if err != nil {
			return err
		}
	}

	return nil
}

// votesByProposal returns all votes for a given proposal.
func (k Keeper) votesByProposal(ctx context.Context, proposalID uint64) ([]group.Vote, error) {
	it, err := k.voteByProposalIndex.Get(k.KVStoreService.OpenKVStore(ctx), proposalID)
	if err != nil {
		return nil, err
	}
	defer it.Close()

	var votes []group.Vote
	for {
		var vote group.Vote
		_, err = it.LoadNext(&vote)
		if errors.ErrORMIteratorDone.Is(err) {
			break
		}
		if err != nil {
			return votes, err
		}
		votes = append(votes, vote)
	}
	return votes, nil
}

// PruneProposals prunes all proposals that are expired, i.e. whose
// `voting_period + max_execution_period` is greater than the current block
// time.
func (k Keeper) PruneProposals(ctx context.Context) error {
	endTime := k.HeaderService.HeaderInfo(ctx).Time.Add(-k.config.MaxExecutionPeriod)
	proposals, err := k.proposalsByVPEnd(ctx, endTime)
	if err != nil {
		return nil
	}
	for _, proposal := range proposals {
		err := k.pruneProposal(ctx, proposal.Id)
		if err != nil {
			return err
		}
		// Emit event for proposal finalized with its result
		if err := k.EventService.EventManager(ctx).Emit(
			&group.EventProposalPruned{
				ProposalId:  proposal.Id,
				Status:      proposal.Status,
				TallyResult: &proposal.FinalTallyResult,
			},
		); err != nil {
			return err
		}
	}

	return nil
}

// TallyProposalsAtVPEnd iterates over all proposals whose voting period
// has ended, tallies their votes, prunes them, and updates the proposal's
// `FinalTallyResult` field.
func (k Keeper) TallyProposalsAtVPEnd(ctx context.Context) error {
	proposals, err := k.proposalsByVPEnd(ctx, k.HeaderService.HeaderInfo(ctx).Time)
	if err != nil {
		return nil
	}
	for _, proposal := range proposals {
		policyInfo, err := k.getGroupPolicyInfo(ctx, proposal.GroupPolicyAddress)
		if err != nil {
			return errorsmod.Wrap(err, "group policy")
		}

		electorate, err := k.getGroupInfo(ctx, policyInfo.GroupId)
		if err != nil {
			return errorsmod.Wrap(err, "group")
		}

		proposalID := proposal.Id
		if proposal.Status == group.PROPOSAL_STATUS_ABORTED || proposal.Status == group.PROPOSAL_STATUS_WITHDRAWN {
			if err := k.pruneProposal(ctx, proposalID); err != nil {
				return err
			}
			if err := k.pruneVotes(ctx, proposalID); err != nil {
				return err
			}
			// Emit event for proposal finalized with its result
			if err := k.EventService.EventManager(ctx).Emit(
				&group.EventProposalPruned{
					ProposalId: proposal.Id,
					Status:     proposal.Status,
				},
			); err != nil {
				return err
			}
		} else if proposal.Status == group.PROPOSAL_STATUS_SUBMITTED {
			if err := k.doTallyAndUpdate(ctx, &proposal, electorate, policyInfo); err != nil {
				return errorsmod.Wrap(err, "doTallyAndUpdate")
			}

			if err := k.proposalTable.Update(k.KVStoreService.OpenKVStore(ctx), proposal.Id, &proposal); err != nil {
				return errorsmod.Wrap(err, "proposal update")
			}
		}
		// Note: We do nothing if the proposal has been marked as ACCEPTED or
		// REJECTED.
	}
	return nil
}

// assertMetadataLength returns an error if given metadata length
// is greater than defined MaxMetadataLen in the module configuration
func (k Keeper) assertMetadataLength(metadata, description string) error {
	if uint64(len(metadata)) > k.config.MaxMetadataLen {
		return errors.ErrMetadataTooLong.Wrap(description)
	}
	return nil
}

// assertSummaryLength returns an error if given summary length
// is greater than defined MaxProposalSummaryLen in the module configuration
func (k Keeper) assertSummaryLength(summary string) error {
	if uint64(len(summary)) > k.config.MaxProposalSummaryLen {
		return errors.ErrSummaryTooLong
	}
	return nil
}

// assertTitleLength returns an error if given summary length
// is greater than defined MaxProposalTitleLen in the module configuration
func (k Keeper) assertTitleLength(title string) error {
	if uint64(len(title)) > k.config.MaxProposalTitleLen {
		return errors.ErrTitleTooLong
	}
	return nil
}
