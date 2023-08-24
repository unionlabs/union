package keeper

import (
	"context"
	"encoding/binary"
	"fmt"

	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	"github.com/cosmos/cosmos-sdk/x/group"
	"github.com/cosmos/cosmos-sdk/x/group/errors"
	"github.com/cosmos/cosmos-sdk/x/group/internal/math"
	"github.com/cosmos/cosmos-sdk/x/group/internal/orm"
)

var _ group.MsgServer = Keeper{}

// TODO: Revisit this once we have proper gas fee framework.
// Tracking issues https://github.com/cosmos/cosmos-sdk/issues/9054, https://github.com/cosmos/cosmos-sdk/discussions/9072
const gasCostPerIteration = uint64(20)

func (k Keeper) CreateGroup(goCtx context.Context, req *group.MsgCreateGroup) (*group.MsgCreateGroupResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)
	metadata := req.Metadata
	members := group.MemberRequests{Members: req.Members}
	admin := req.Admin

	if err := members.ValidateBasic(); err != nil {
		return nil, err
	}

	if err := k.assertMetadataLength(metadata, "group metadata"); err != nil {
		return nil, err
	}

	totalWeight := math.NewDecFromInt64(0)
	for i := range members.Members {
		m := members.Members[i]
		if err := k.assertMetadataLength(m.Metadata, "member metadata"); err != nil {
			return nil, err
		}

		// Members of a group must have a positive weight.
		weight, err := math.NewPositiveDecFromString(m.Weight)
		if err != nil {
			return nil, err
		}

		// Adding up members weights to compute group total weight.
		totalWeight, err = totalWeight.Add(weight)
		if err != nil {
			return nil, err
		}
	}

	// Create a new group in the groupTable.
	groupInfo := &group.GroupInfo{
		Id:          k.groupTable.Sequence().PeekNextVal(ctx.KVStore(k.key)),
		Admin:       admin,
		Metadata:    metadata,
		Version:     1,
		TotalWeight: totalWeight.String(),
		CreatedAt:   ctx.BlockTime(),
	}
	groupID, err := k.groupTable.Create(ctx.KVStore(k.key), groupInfo)
	if err != nil {
		return nil, sdkerrors.Wrap(err, "could not create group")
	}

	// Create new group members in the groupMemberTable.
	for i := range members.Members {
		m := members.Members[i]
		err := k.groupMemberTable.Create(ctx.KVStore(k.key), &group.GroupMember{
			GroupId: groupID,
			Member: &group.Member{
				Address:  m.Address,
				Weight:   m.Weight,
				Metadata: m.Metadata,
				AddedAt:  ctx.BlockTime(),
			},
		})
		if err != nil {
			return nil, sdkerrors.Wrapf(err, "could not store member %d", i)
		}
	}

	err = ctx.EventManager().EmitTypedEvent(&group.EventCreateGroup{GroupId: groupID})
	if err != nil {
		return nil, err
	}

	return &group.MsgCreateGroupResponse{GroupId: groupID}, nil
}

func (k Keeper) UpdateGroupMembers(goCtx context.Context, req *group.MsgUpdateGroupMembers) (*group.MsgUpdateGroupMembersResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)
	action := func(g *group.GroupInfo) error {
		totalWeight, err := math.NewNonNegativeDecFromString(g.TotalWeight)
		if err != nil {
			return sdkerrors.Wrap(err, "group total weight")
		}
		for i := range req.MemberUpdates {
			if err := k.assertMetadataLength(req.MemberUpdates[i].Metadata, "group member metadata"); err != nil {
				return err
			}
			groupMember := group.GroupMember{
				GroupId: req.GroupId,
				Member: &group.Member{
					Address:  req.MemberUpdates[i].Address,
					Weight:   req.MemberUpdates[i].Weight,
					Metadata: req.MemberUpdates[i].Metadata,
				},
			}

			// Checking if the group member is already part of the group
			var found bool
			var prevGroupMember group.GroupMember
			switch err := k.groupMemberTable.GetOne(ctx.KVStore(k.key), orm.PrimaryKey(&groupMember), &prevGroupMember); {
			case err == nil:
				found = true
			case sdkerrors.ErrNotFound.Is(err):
				found = false
			default:
				return sdkerrors.Wrap(err, "get group member")
			}

			newMemberWeight, err := math.NewNonNegativeDecFromString(groupMember.Member.Weight)
			if err != nil {
				return err
			}

			// Handle delete for members with zero weight.
			if newMemberWeight.IsZero() {
				// We can't delete a group member that doesn't already exist.
				if !found {
					return sdkerrors.Wrap(sdkerrors.ErrNotFound, "unknown member")
				}

				previousMemberWeight, err := math.NewPositiveDecFromString(prevGroupMember.Member.Weight)
				if err != nil {
					return err
				}

				// Subtract the weight of the group member to delete from the group total weight.
				totalWeight, err = math.SubNonNegative(totalWeight, previousMemberWeight)
				if err != nil {
					return err
				}

				// Delete group member in the groupMemberTable.
				if err := k.groupMemberTable.Delete(ctx.KVStore(k.key), &groupMember); err != nil {
					return sdkerrors.Wrap(err, "delete member")
				}
				continue
			}
			// If group member already exists, handle update
			if found {
				previousMemberWeight, err := math.NewPositiveDecFromString(prevGroupMember.Member.Weight)
				if err != nil {
					return err
				}
				// Subtract previous weight from the group total weight.
				totalWeight, err = math.SubNonNegative(totalWeight, previousMemberWeight)
				if err != nil {
					return err
				}
				// Save updated group member in the groupMemberTable.
				groupMember.Member.AddedAt = prevGroupMember.Member.AddedAt
				if err := k.groupMemberTable.Update(ctx.KVStore(k.key), &groupMember); err != nil {
					return sdkerrors.Wrap(err, "add member")
				}
			} else { // else handle create.
				groupMember.Member.AddedAt = ctx.BlockTime()
				if err := k.groupMemberTable.Create(ctx.KVStore(k.key), &groupMember); err != nil {
					return sdkerrors.Wrap(err, "add member")
				}
			}
			// In both cases (handle + update), we need to add the new member's weight to the group total weight.
			totalWeight, err = totalWeight.Add(newMemberWeight)
			if err != nil {
				return err
			}
		}
		// Update group in the groupTable.
		g.TotalWeight = totalWeight.String()
		g.Version++

		if err := k.validateDecisionPolicies(ctx, *g); err != nil {
			return err
		}

		return k.groupTable.Update(ctx.KVStore(k.key), g.Id, g)
	}

	err := k.doUpdateGroup(ctx, req, action, "members updated")
	if err != nil {
		return nil, err
	}

	return &group.MsgUpdateGroupMembersResponse{}, nil
}

func (k Keeper) UpdateGroupAdmin(goCtx context.Context, req *group.MsgUpdateGroupAdmin) (*group.MsgUpdateGroupAdminResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)
	action := func(g *group.GroupInfo) error {
		g.Admin = req.NewAdmin
		g.Version++

		return k.groupTable.Update(ctx.KVStore(k.key), g.Id, g)
	}

	err := k.doUpdateGroup(ctx, req, action, "admin updated")
	if err != nil {
		return nil, err
	}

	return &group.MsgUpdateGroupAdminResponse{}, nil
}

func (k Keeper) UpdateGroupMetadata(goCtx context.Context, req *group.MsgUpdateGroupMetadata) (*group.MsgUpdateGroupMetadataResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)
	action := func(g *group.GroupInfo) error {
		g.Metadata = req.Metadata
		g.Version++
		return k.groupTable.Update(ctx.KVStore(k.key), g.Id, g)
	}

	if err := k.assertMetadataLength(req.Metadata, "group metadata"); err != nil {
		return nil, err
	}

	err := k.doUpdateGroup(ctx, req, action, "metadata updated")
	if err != nil {
		return nil, err
	}

	return &group.MsgUpdateGroupMetadataResponse{}, nil
}

func (k Keeper) CreateGroupWithPolicy(goCtx context.Context, req *group.MsgCreateGroupWithPolicy) (*group.MsgCreateGroupWithPolicyResponse, error) {
	groupRes, err := k.CreateGroup(goCtx, &group.MsgCreateGroup{
		Admin:    req.Admin,
		Members:  req.Members,
		Metadata: req.GroupMetadata,
	})
	if err != nil {
		return nil, sdkerrors.Wrap(err, "group response")
	}
	groupID := groupRes.GroupId

	var groupPolicyAddr sdk.AccAddress
	groupPolicyRes, err := k.CreateGroupPolicy(goCtx, &group.MsgCreateGroupPolicy{
		Admin:          req.Admin,
		GroupId:        groupID,
		Metadata:       req.GroupPolicyMetadata,
		DecisionPolicy: req.DecisionPolicy,
	})
	if err != nil {
		return nil, sdkerrors.Wrap(err, "group policy response")
	}
	policyAddr := groupPolicyRes.Address

	groupPolicyAddr, err = sdk.AccAddressFromBech32(policyAddr)
	if err != nil {
		return nil, sdkerrors.Wrap(err, "group policy address")
	}
	groupPolicyAddress := groupPolicyAddr.String()

	if req.GroupPolicyAsAdmin {
		updateAdminReq := &group.MsgUpdateGroupAdmin{
			GroupId:  groupID,
			Admin:    req.Admin,
			NewAdmin: groupPolicyAddress,
		}
		_, err = k.UpdateGroupAdmin(goCtx, updateAdminReq)
		if err != nil {
			return nil, err
		}

		updatePolicyAddressReq := &group.MsgUpdateGroupPolicyAdmin{
			Admin:              req.Admin,
			GroupPolicyAddress: groupPolicyAddress,
			NewAdmin:           groupPolicyAddress,
		}
		_, err = k.UpdateGroupPolicyAdmin(goCtx, updatePolicyAddressReq)
		if err != nil {
			return nil, err
		}
	}

	return &group.MsgCreateGroupWithPolicyResponse{GroupId: groupID, GroupPolicyAddress: groupPolicyAddress}, nil
}

func (k Keeper) CreateGroupPolicy(goCtx context.Context, req *group.MsgCreateGroupPolicy) (*group.MsgCreateGroupPolicyResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)
	admin, err := sdk.AccAddressFromBech32(req.GetAdmin())
	if err != nil {
		return nil, sdkerrors.Wrap(err, "request admin")
	}
	policy, err := req.GetDecisionPolicy()
	if err != nil {
		return nil, sdkerrors.Wrap(err, "request decision policy")
	}
	groupID := req.GetGroupID()
	metadata := req.GetMetadata()

	if err := k.assertMetadataLength(metadata, "group policy metadata"); err != nil {
		return nil, err
	}

	g, err := k.getGroupInfo(ctx, groupID)
	if err != nil {
		return nil, err
	}
	groupAdmin, err := sdk.AccAddressFromBech32(g.Admin)
	if err != nil {
		return nil, sdkerrors.Wrap(err, "group admin")
	}
	// Only current group admin is authorized to create a group policy for this
	if !groupAdmin.Equals(admin) {
		return nil, sdkerrors.Wrap(sdkerrors.ErrUnauthorized, "not group admin")
	}

	err = policy.Validate(g, k.config)
	if err != nil {
		return nil, err
	}

	// Generate account address of group policy.
	var accountAddr sdk.AccAddress
	// loop here in the rare case where a ADR-028-derived address creates a
	// collision with an existing address.
	for {
		nextAccVal := k.groupPolicySeq.NextVal(ctx.KVStore(k.key))
		derivationKey := make([]byte, 8)
		binary.BigEndian.PutUint64(derivationKey, nextAccVal)

		ac, err := authtypes.NewModuleCredential(group.ModuleName, []byte{GroupPolicyTablePrefix}, derivationKey)
		if err != nil {
			return nil, err
		}
		accountAddr = sdk.AccAddress(ac.Address())
		if k.accKeeper.GetAccount(ctx, accountAddr) != nil {
			// handle a rare collision, in which case we just go on to the
			// next sequence value and derive a new address.
			continue
		}

		// group policy accounts are unclaimable base accounts
		account, err := authtypes.NewBaseAccountWithPubKey(ac)
		if err != nil {
			return nil, sdkerrors.Wrap(err, "could not create group policy account")
		}

		acc := k.accKeeper.NewAccount(ctx, account)
		k.accKeeper.SetAccount(ctx, acc)

		break
	}

	groupPolicy, err := group.NewGroupPolicyInfo(
		accountAddr,
		groupID,
		admin,
		metadata,
		1,
		policy,
		ctx.BlockTime(),
	)
	if err != nil {
		return nil, err
	}

	if err := k.groupPolicyTable.Create(ctx.KVStore(k.key), &groupPolicy); err != nil {
		return nil, sdkerrors.Wrap(err, "could not create group policy")
	}

	err = ctx.EventManager().EmitTypedEvent(&group.EventCreateGroupPolicy{Address: accountAddr.String()})
	if err != nil {
		return nil, err
	}

	return &group.MsgCreateGroupPolicyResponse{Address: accountAddr.String()}, nil
}

func (k Keeper) UpdateGroupPolicyAdmin(goCtx context.Context, req *group.MsgUpdateGroupPolicyAdmin) (*group.MsgUpdateGroupPolicyAdminResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)
	action := func(groupPolicy *group.GroupPolicyInfo) error {
		groupPolicy.Admin = req.NewAdmin
		groupPolicy.Version++
		return k.groupPolicyTable.Update(ctx.KVStore(k.key), groupPolicy)
	}

	err := k.doUpdateGroupPolicy(ctx, req.GroupPolicyAddress, req.Admin, action, "group policy admin updated")
	if err != nil {
		return nil, err
	}

	return &group.MsgUpdateGroupPolicyAdminResponse{}, nil
}

func (k Keeper) UpdateGroupPolicyDecisionPolicy(goCtx context.Context, req *group.MsgUpdateGroupPolicyDecisionPolicy) (*group.MsgUpdateGroupPolicyDecisionPolicyResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)
	policy, err := req.GetDecisionPolicy()
	if err != nil {
		return nil, err
	}

	action := func(groupPolicy *group.GroupPolicyInfo) error {
		g, err := k.getGroupInfo(ctx, groupPolicy.GroupId)
		if err != nil {
			return err
		}

		err = policy.Validate(g, k.config)
		if err != nil {
			return err
		}

		err = groupPolicy.SetDecisionPolicy(policy)
		if err != nil {
			return err
		}

		groupPolicy.Version++
		return k.groupPolicyTable.Update(ctx.KVStore(k.key), groupPolicy)
	}

	err = k.doUpdateGroupPolicy(ctx, req.GroupPolicyAddress, req.Admin, action, "group policy's decision policy updated")
	if err != nil {
		return nil, err
	}

	return &group.MsgUpdateGroupPolicyDecisionPolicyResponse{}, nil
}

func (k Keeper) UpdateGroupPolicyMetadata(goCtx context.Context, req *group.MsgUpdateGroupPolicyMetadata) (*group.MsgUpdateGroupPolicyMetadataResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)
	metadata := req.GetMetadata()

	action := func(groupPolicy *group.GroupPolicyInfo) error {
		groupPolicy.Metadata = metadata
		groupPolicy.Version++
		return k.groupPolicyTable.Update(ctx.KVStore(k.key), groupPolicy)
	}

	if err := k.assertMetadataLength(metadata, "group policy metadata"); err != nil {
		return nil, err
	}

	err := k.doUpdateGroupPolicy(ctx, req.GroupPolicyAddress, req.Admin, action, "group policy metadata updated")
	if err != nil {
		return nil, err
	}

	return &group.MsgUpdateGroupPolicyMetadataResponse{}, nil
}

func (k Keeper) SubmitProposal(goCtx context.Context, req *group.MsgSubmitProposal) (*group.MsgSubmitProposalResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)
	groupPolicyAddr, err := sdk.AccAddressFromBech32(req.GroupPolicyAddress)
	if err != nil {
		return nil, sdkerrors.Wrap(err, "request account address of group policy")
	}
	metadata := req.Metadata
	proposers := req.Proposers
	msgs, err := req.GetMsgs()
	if err != nil {
		return nil, sdkerrors.Wrap(err, "request msgs")
	}

	if err := k.assertMetadataLength(metadata, "metadata"); err != nil {
		return nil, err
	}

	if err := k.assertMetadataLength(req.Summary, "proposal summary"); err != nil {
		return nil, err
	}

	if err := k.assertMetadataLength(req.Title, "proposal Title"); err != nil {
		return nil, err
	}

	policyAcc, err := k.getGroupPolicyInfo(ctx, req.GroupPolicyAddress)
	if err != nil {
		return nil, sdkerrors.Wrap(err, "load group policy")
	}

	g, err := k.getGroupInfo(ctx, policyAcc.GroupId)
	if err != nil {
		return nil, sdkerrors.Wrap(err, "get group by groupId of group policy")
	}

	// Only members of the group can submit a new proposal.
	for i := range proposers {
		if !k.groupMemberTable.Has(ctx.KVStore(k.key), orm.PrimaryKey(&group.GroupMember{GroupId: g.Id, Member: &group.Member{Address: proposers[i]}})) {
			return nil, sdkerrors.Wrapf(errors.ErrUnauthorized, "not in group: %s", proposers[i])
		}
	}

	// Check that if the messages require signers, they are all equal to the given account address of group policy.
	if err := ensureMsgAuthZ(msgs, groupPolicyAddr); err != nil {
		return nil, err
	}

	policy, err := policyAcc.GetDecisionPolicy()
	if err != nil {
		return nil, sdkerrors.Wrap(err, "proposal group policy decision policy")
	}

	// Prevent proposal that can not succeed.
	err = policy.Validate(g, k.config)
	if err != nil {
		return nil, err
	}

	m := &group.Proposal{
		Id:                 k.proposalTable.Sequence().PeekNextVal(ctx.KVStore(k.key)),
		GroupPolicyAddress: req.GroupPolicyAddress,
		Metadata:           metadata,
		Proposers:          proposers,
		SubmitTime:         ctx.BlockTime(),
		GroupVersion:       g.Version,
		GroupPolicyVersion: policyAcc.Version,
		Status:             group.PROPOSAL_STATUS_SUBMITTED,
		ExecutorResult:     group.PROPOSAL_EXECUTOR_RESULT_NOT_RUN,
		VotingPeriodEnd:    ctx.BlockTime().Add(policy.GetVotingPeriod()), // The voting window begins as soon as the proposal is submitted.
		FinalTallyResult:   group.DefaultTallyResult(),
		Title:              req.Title,
		Summary:            req.Summary,
	}

	if err := m.SetMsgs(msgs); err != nil {
		return nil, sdkerrors.Wrap(err, "create proposal")
	}

	id, err := k.proposalTable.Create(ctx.KVStore(k.key), m)
	if err != nil {
		return nil, sdkerrors.Wrap(err, "create proposal")
	}

	err = ctx.EventManager().EmitTypedEvent(&group.EventSubmitProposal{ProposalId: id})
	if err != nil {
		return nil, err
	}

	// Try to execute proposal immediately
	if req.Exec == group.Exec_EXEC_TRY {
		// Consider proposers as Yes votes
		for i := range proposers {
			ctx.GasMeter().ConsumeGas(gasCostPerIteration, "vote on proposal")
			_, err = k.Vote(sdk.WrapSDKContext(ctx), &group.MsgVote{
				ProposalId: id,
				Voter:      proposers[i],
				Option:     group.VOTE_OPTION_YES,
			})
			if err != nil {
				return &group.MsgSubmitProposalResponse{ProposalId: id}, sdkerrors.Wrapf(err, "the proposal was created but failed on vote for voter %s", proposers[i])
			}
		}

		// Then try to execute the proposal
		_, err = k.Exec(sdk.WrapSDKContext(ctx), &group.MsgExec{
			ProposalId: id,
			// We consider the first proposer as the MsgExecRequest signer
			// but that could be revisited (eg using the group policy)
			Executor: proposers[0],
		})
		if err != nil {
			return &group.MsgSubmitProposalResponse{ProposalId: id}, sdkerrors.Wrap(err, "the proposal was created but failed on exec")
		}
	}

	return &group.MsgSubmitProposalResponse{ProposalId: id}, nil
}

func (k Keeper) WithdrawProposal(goCtx context.Context, req *group.MsgWithdrawProposal) (*group.MsgWithdrawProposalResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)
	id := req.ProposalId
	address := req.Address

	proposal, err := k.getProposal(ctx, id)
	if err != nil {
		return nil, err
	}

	// Ensure the proposal can be withdrawn.
	if proposal.Status != group.PROPOSAL_STATUS_SUBMITTED {
		return nil, sdkerrors.Wrapf(errors.ErrInvalid, "cannot withdraw a proposal with the status of %s", proposal.Status.String())
	}

	var policyInfo group.GroupPolicyInfo
	if policyInfo, err = k.getGroupPolicyInfo(ctx, proposal.GroupPolicyAddress); err != nil {
		return nil, sdkerrors.Wrap(err, "load group policy")
	}

	// check address is the group policy admin he is in proposers list..
	if address != policyInfo.Admin && !isProposer(proposal, address) {
		return nil, sdkerrors.Wrapf(errors.ErrUnauthorized, "given address is neither group policy admin nor in proposers: %s", address)
	}

	proposal.Status = group.PROPOSAL_STATUS_WITHDRAWN
	if err := k.proposalTable.Update(ctx.KVStore(k.key), id, &proposal); err != nil {
		return nil, err
	}

	err = ctx.EventManager().EmitTypedEvent(&group.EventWithdrawProposal{ProposalId: id})
	if err != nil {
		return nil, err
	}

	return &group.MsgWithdrawProposalResponse{}, nil
}

func (k Keeper) Vote(goCtx context.Context, req *group.MsgVote) (*group.MsgVoteResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)
	id := req.ProposalId
	voteOption := req.Option
	metadata := req.Metadata

	if err := k.assertMetadataLength(metadata, "metadata"); err != nil {
		return nil, err
	}

	proposal, err := k.getProposal(ctx, id)
	if err != nil {
		return nil, err
	}
	// Ensure that we can still accept votes for this proposal.
	if proposal.Status != group.PROPOSAL_STATUS_SUBMITTED {
		return nil, sdkerrors.Wrap(errors.ErrInvalid, "proposal not open for voting")
	}
	if ctx.BlockTime().After(proposal.VotingPeriodEnd) {
		return nil, sdkerrors.Wrap(errors.ErrExpired, "voting period has ended already")
	}

	policyInfo, err := k.getGroupPolicyInfo(ctx, proposal.GroupPolicyAddress)
	if err != nil {
		return nil, sdkerrors.Wrap(err, "load group policy")
	}

	electorate, err := k.getGroupInfo(ctx, policyInfo.GroupId)
	if err != nil {
		return nil, err
	}

	// Count and store votes.
	voterAddr := req.Voter
	voter := group.GroupMember{GroupId: electorate.Id, Member: &group.Member{Address: voterAddr}}
	if err := k.groupMemberTable.GetOne(ctx.KVStore(k.key), orm.PrimaryKey(&voter), &voter); err != nil {
		return nil, sdkerrors.Wrapf(err, "voter address: %s", voterAddr)
	}
	newVote := group.Vote{
		ProposalId: id,
		Voter:      voterAddr,
		Option:     voteOption,
		Metadata:   metadata,
		SubmitTime: ctx.BlockTime(),
	}

	// The ORM will return an error if the vote already exists,
	// making sure than a voter hasn't already voted.
	if err := k.voteTable.Create(ctx.KVStore(k.key), &newVote); err != nil {
		return nil, sdkerrors.Wrap(err, "store vote")
	}

	err = ctx.EventManager().EmitTypedEvent(&group.EventVote{ProposalId: id})
	if err != nil {
		return nil, err
	}

	// Try to execute proposal immediately
	if req.Exec == group.Exec_EXEC_TRY {
		_, err = k.Exec(sdk.WrapSDKContext(ctx), &group.MsgExec{
			ProposalId: id,
			Executor:   voterAddr,
		})
		if err != nil {
			return nil, err
		}
	}

	return &group.MsgVoteResponse{}, nil
}

// doTallyAndUpdate performs a tally, and, if the tally result is final, then:
// - updates the proposal's `Status` and `FinalTallyResult` fields,
// - prune all the votes.
func (k Keeper) doTallyAndUpdate(ctx sdk.Context, p *group.Proposal, electorate group.GroupInfo, policyInfo group.GroupPolicyInfo) error {
	policy, err := policyInfo.GetDecisionPolicy()
	if err != nil {
		return err
	}

	tallyResult, err := k.Tally(ctx, *p, policyInfo.GroupId)
	if err != nil {
		return err
	}

	result, err := policy.Allow(tallyResult, electorate.TotalWeight)
	if err != nil {
		return sdkerrors.Wrap(err, "policy allow")
	}

	// If the result was final (i.e. enough votes to pass) or if the voting
	// period ended, then we consider the proposal as final.
	if isFinal := result.Final || ctx.BlockTime().After(p.VotingPeriodEnd); isFinal {
		if err := k.pruneVotes(ctx, p.Id); err != nil {
			return err
		}
		p.FinalTallyResult = tallyResult
		if result.Allow {
			p.Status = group.PROPOSAL_STATUS_ACCEPTED
		} else {
			p.Status = group.PROPOSAL_STATUS_REJECTED
		}

	}

	return nil
}

// Exec executes the messages from a proposal.
func (k Keeper) Exec(goCtx context.Context, req *group.MsgExec) (*group.MsgExecResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)
	id := req.ProposalId

	proposal, err := k.getProposal(ctx, id)
	if err != nil {
		return nil, err
	}

	if proposal.Status != group.PROPOSAL_STATUS_SUBMITTED && proposal.Status != group.PROPOSAL_STATUS_ACCEPTED {
		return nil, sdkerrors.Wrapf(errors.ErrInvalid, "not possible to exec with proposal status %s", proposal.Status.String())
	}

	policyInfo, err := k.getGroupPolicyInfo(ctx, proposal.GroupPolicyAddress)
	if err != nil {
		return nil, sdkerrors.Wrap(err, "load group policy")
	}

	// If proposal is still in SUBMITTED phase, it means that the voting period
	// didn't end yet, and tallying hasn't been done. In this case, we need to
	// tally first.
	if proposal.Status == group.PROPOSAL_STATUS_SUBMITTED {
		electorate, err := k.getGroupInfo(ctx, policyInfo.GroupId)
		if err != nil {
			return nil, sdkerrors.Wrap(err, "load group")
		}

		if err := k.doTallyAndUpdate(ctx, &proposal, electorate, policyInfo); err != nil {
			return nil, err
		}
	}

	// Execute proposal payload.
	var logs string
	if proposal.Status == group.PROPOSAL_STATUS_ACCEPTED && proposal.ExecutorResult != group.PROPOSAL_EXECUTOR_RESULT_SUCCESS {
		// Caching context so that we don't update the store in case of failure.
		cacheCtx, flush := ctx.CacheContext()

		addr, err := sdk.AccAddressFromBech32(policyInfo.Address)
		if err != nil {
			return nil, err
		}

		decisionPolicy := policyInfo.DecisionPolicy.GetCachedValue().(group.DecisionPolicy)
		if results, err := k.doExecuteMsgs(cacheCtx, k.router, proposal, addr, decisionPolicy); err != nil {
			proposal.ExecutorResult = group.PROPOSAL_EXECUTOR_RESULT_FAILURE
			logs = fmt.Sprintf("proposal execution failed on proposal %d, because of error %s", id, err.Error())
			k.Logger(ctx).Info("proposal execution failed", "cause", err, "proposalID", id)
		} else {
			proposal.ExecutorResult = group.PROPOSAL_EXECUTOR_RESULT_SUCCESS
			flush()

			for _, res := range results {
				// NOTE: The sdk msg handler creates a new EventManager, so events must be correctly propagated back to the current context
				ctx.EventManager().EmitEvents(res.GetEvents())
			}
		}
	}

	// Update proposal in proposalTable
	// If proposal has successfully run, delete it from state.
	if proposal.ExecutorResult == group.PROPOSAL_EXECUTOR_RESULT_SUCCESS {
		if err := k.pruneProposal(ctx, proposal.Id); err != nil {
			return nil, err
		}

		// Emit event for proposal finalized with its result
		if err := ctx.EventManager().EmitTypedEvent(
			&group.EventProposalPruned{
				ProposalId:  proposal.Id,
				Status:      proposal.Status,
				TallyResult: &proposal.FinalTallyResult,
			}); err != nil {
			return nil, err
		}
	} else {
		store := ctx.KVStore(k.key)
		if err := k.proposalTable.Update(store, id, &proposal); err != nil {
			return nil, err
		}
	}

	err = ctx.EventManager().EmitTypedEvent(&group.EventExec{
		ProposalId: id,
		Logs:       logs,
		Result:     proposal.ExecutorResult,
	})
	if err != nil {
		return nil, err
	}

	return &group.MsgExecResponse{
		Result: proposal.ExecutorResult,
	}, nil
}

// LeaveGroup implements the MsgServer/LeaveGroup method.
func (k Keeper) LeaveGroup(goCtx context.Context, req *group.MsgLeaveGroup) (*group.MsgLeaveGroupResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)
	_, err := sdk.AccAddressFromBech32(req.Address)
	if err != nil {
		return nil, err
	}

	groupInfo, err := k.getGroupInfo(ctx, req.GroupId)
	if err != nil {
		return nil, sdkerrors.Wrap(err, "group")
	}

	groupWeight, err := math.NewNonNegativeDecFromString(groupInfo.TotalWeight)
	if err != nil {
		return nil, err
	}

	gm, err := k.getGroupMember(ctx, &group.GroupMember{
		GroupId: req.GroupId,
		Member:  &group.Member{Address: req.Address},
	})
	if err != nil {
		return nil, err
	}

	memberWeight, err := math.NewPositiveDecFromString(gm.Member.Weight)
	if err != nil {
		return nil, err
	}

	updatedWeight, err := math.SubNonNegative(groupWeight, memberWeight)
	if err != nil {
		return nil, err
	}

	// delete group member in the groupMemberTable.
	if err := k.groupMemberTable.Delete(ctx.KVStore(k.key), gm); err != nil {
		return nil, sdkerrors.Wrap(err, "group member")
	}

	// update group weight
	groupInfo.TotalWeight = updatedWeight.String()
	groupInfo.Version++

	if err := k.validateDecisionPolicies(ctx, groupInfo); err != nil {
		return nil, err
	}

	if err := k.groupTable.Update(ctx.KVStore(k.key), groupInfo.Id, &groupInfo); err != nil {
		return nil, err
	}

	ctx.EventManager().EmitTypedEvent(&group.EventLeaveGroup{
		GroupId: req.GroupId,
		Address: req.Address,
	})

	return &group.MsgLeaveGroupResponse{}, nil
}

func (k Keeper) getGroupMember(ctx sdk.Context, member *group.GroupMember) (*group.GroupMember, error) {
	var groupMember group.GroupMember
	switch err := k.groupMemberTable.GetOne(ctx.KVStore(k.key),
		orm.PrimaryKey(member), &groupMember); {
	case err == nil:
		break
	case sdkerrors.ErrNotFound.Is(err):
		return nil, sdkerrors.ErrNotFound.Wrapf("%s is not part of group %d", member.Member.Address, member.GroupId)
	default:
		return nil, err
	}

	return &groupMember, nil
}

type authNGroupReq interface {
	GetGroupID() uint64
	GetAdmin() string
}

type (
	actionFn            func(m *group.GroupInfo) error
	groupPolicyActionFn func(m *group.GroupPolicyInfo) error
)

// doUpdateGroupPolicy first makes sure that the group policy admin initiated the group policy update,
// before performing the group policy update and emitting an event.
func (k Keeper) doUpdateGroupPolicy(ctx sdk.Context, groupPolicy string, admin string, action groupPolicyActionFn, note string) error {
	groupPolicyInfo, err := k.getGroupPolicyInfo(ctx, groupPolicy)
	if err != nil {
		return sdkerrors.Wrap(err, "load group policy")
	}

	groupPolicyAddr, err := sdk.AccAddressFromBech32(groupPolicy)
	if err != nil {
		return sdkerrors.Wrap(err, "group policy address")
	}

	groupPolicyAdmin, err := sdk.AccAddressFromBech32(admin)
	if err != nil {
		return sdkerrors.Wrap(err, "group policy admin")
	}

	// Only current group policy admin is authorized to update a group policy.
	if groupPolicyAdmin.String() != groupPolicyInfo.Admin {
		return sdkerrors.Wrap(sdkerrors.ErrUnauthorized, "not group policy admin")
	}

	if err := action(&groupPolicyInfo); err != nil {
		return sdkerrors.Wrap(err, note)
	}

	if err = k.abortProposals(ctx, groupPolicyAddr); err != nil {
		return err
	}

	if err = ctx.EventManager().EmitTypedEvent(&group.EventUpdateGroupPolicy{Address: groupPolicyInfo.Address}); err != nil {
		return err
	}

	return nil
}

// doUpdateGroup first makes sure that the group admin initiated the group update,
// before performing the group update and emitting an event.
func (k Keeper) doUpdateGroup(ctx sdk.Context, req authNGroupReq, action actionFn, note string) error {
	err := k.doAuthenticated(ctx, req, action, note)
	if err != nil {
		return err
	}

	err = ctx.EventManager().EmitTypedEvent(&group.EventUpdateGroup{GroupId: req.GetGroupID()})
	if err != nil {
		return err
	}

	return nil
}

// doAuthenticated makes sure that the group admin initiated the request,
// and perform the provided action on the group.
func (k Keeper) doAuthenticated(ctx sdk.Context, req authNGroupReq, action actionFn, errNote string) error {
	group, err := k.getGroupInfo(ctx, req.GetGroupID())
	if err != nil {
		return err
	}
	admin, err := sdk.AccAddressFromBech32(group.Admin)
	if err != nil {
		return sdkerrors.Wrap(err, "group admin")
	}
	reqAdmin, err := sdk.AccAddressFromBech32(req.GetAdmin())
	if err != nil {
		return sdkerrors.Wrap(err, "request admin")
	}
	if !admin.Equals(reqAdmin) {
		return sdkerrors.Wrapf(sdkerrors.ErrUnauthorized, "not group admin; got %s, expected %s", req.GetAdmin(), group.Admin)
	}
	if err := action(&group); err != nil {
		return sdkerrors.Wrap(err, errNote)
	}
	return nil
}

// assertMetadataLength returns an error if given metadata length
// is greater than a pre-defined maxMetadataLen.
func (k Keeper) assertMetadataLength(metadata string, description string) error {
	if metadata != "" && uint64(len(metadata)) > k.config.MaxMetadataLen {
		return sdkerrors.Wrapf(errors.ErrMaxLimit, description)
	}
	return nil
}

// validateDecisionPolicies loops through all decision policies from the group,
// and calls each of their Validate() method.
func (k Keeper) validateDecisionPolicies(ctx sdk.Context, g group.GroupInfo) error {
	it, err := k.groupPolicyByGroupIndex.Get(ctx.KVStore(k.key), g.Id)
	if err != nil {
		return err
	}
	defer it.Close()

	for {
		var groupPolicy group.GroupPolicyInfo
		_, err = it.LoadNext(&groupPolicy)
		if errors.ErrORMIteratorDone.Is(err) {
			break
		}
		if err != nil {
			return err
		}

		err = groupPolicy.DecisionPolicy.GetCachedValue().(group.DecisionPolicy).Validate(g, k.config)
		if err != nil {
			return err
		}
	}

	return nil
}

// isProposer checks that an address is a proposer of a given proposal.
func isProposer(proposal group.Proposal, address string) bool {
	for _, proposer := range proposal.Proposers {
		if proposer == address {
			return true
		}
	}

	return false
}
