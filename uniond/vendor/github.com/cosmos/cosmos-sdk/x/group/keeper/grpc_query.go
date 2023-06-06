package keeper

import (
	"context"
	"math"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"

	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/types/query"
	"github.com/cosmos/cosmos-sdk/x/group"
	"github.com/cosmos/cosmos-sdk/x/group/errors"
	"github.com/cosmos/cosmos-sdk/x/group/internal/orm"
)

var _ group.QueryServer = Keeper{}

// GroupInfo queries info about a group.
func (k Keeper) GroupInfo(goCtx context.Context, request *group.QueryGroupInfoRequest) (*group.QueryGroupInfoResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)
	groupID := request.GroupId
	groupInfo, err := k.getGroupInfo(ctx, groupID)
	if err != nil {
		return nil, sdkerrors.Wrap(err, "group")
	}

	return &group.QueryGroupInfoResponse{Info: &groupInfo}, nil
}

// getGroupInfo gets the group info of the given group id.
func (k Keeper) getGroupInfo(ctx sdk.Context, id uint64) (group.GroupInfo, error) {
	var obj group.GroupInfo
	_, err := k.groupTable.GetOne(ctx.KVStore(k.key), id, &obj)
	return obj, err
}

// GroupPolicyInfo queries info about a group policy.
func (k Keeper) GroupPolicyInfo(goCtx context.Context, request *group.QueryGroupPolicyInfoRequest) (*group.QueryGroupPolicyInfoResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)
	groupPolicyInfo, err := k.getGroupPolicyInfo(ctx, request.Address)
	if err != nil {
		return nil, sdkerrors.Wrap(err, "group policy")
	}

	return &group.QueryGroupPolicyInfoResponse{Info: &groupPolicyInfo}, nil
}

// getGroupPolicyInfo gets the group policy info of the given account address.
func (k Keeper) getGroupPolicyInfo(ctx sdk.Context, accountAddress string) (group.GroupPolicyInfo, error) {
	var obj group.GroupPolicyInfo
	return obj, k.groupPolicyTable.GetOne(ctx.KVStore(k.key), orm.PrimaryKey(&group.GroupPolicyInfo{Address: accountAddress}), &obj)
}

// GroupMembers queries all members of a group.
func (k Keeper) GroupMembers(goCtx context.Context, request *group.QueryGroupMembersRequest) (*group.QueryGroupMembersResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)
	groupID := request.GroupId
	it, err := k.getGroupMembers(ctx, groupID, request.Pagination)
	if err != nil {
		return nil, err
	}

	var members []*group.GroupMember
	pageRes, err := orm.Paginate(it, request.Pagination, &members)
	if err != nil {
		return nil, err
	}

	return &group.QueryGroupMembersResponse{
		Members:    members,
		Pagination: pageRes,
	}, nil
}

// getGroupMembers returns an iterator for the given group id and page request.
func (k Keeper) getGroupMembers(ctx sdk.Context, id uint64, pageRequest *query.PageRequest) (orm.Iterator, error) {
	return k.groupMemberByGroupIndex.GetPaginated(ctx.KVStore(k.key), id, pageRequest)
}

// GroupsByAdmin queries all groups where a given address is admin.
func (k Keeper) GroupsByAdmin(goCtx context.Context, request *group.QueryGroupsByAdminRequest) (*group.QueryGroupsByAdminResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)
	addr, err := sdk.AccAddressFromBech32(request.Admin)
	if err != nil {
		return nil, err
	}
	it, err := k.getGroupsByAdmin(ctx, addr, request.Pagination)
	if err != nil {
		return nil, err
	}

	var groups []*group.GroupInfo
	pageRes, err := orm.Paginate(it, request.Pagination, &groups)
	if err != nil {
		return nil, err
	}

	return &group.QueryGroupsByAdminResponse{
		Groups:     groups,
		Pagination: pageRes,
	}, nil
}

// getGroupsByAdmin returns an iterator for the given admin account address and page request.
func (k Keeper) getGroupsByAdmin(ctx sdk.Context, admin sdk.AccAddress, pageRequest *query.PageRequest) (orm.Iterator, error) {
	return k.groupByAdminIndex.GetPaginated(ctx.KVStore(k.key), admin.Bytes(), pageRequest)
}

// GroupPoliciesByGroup queries all groups policies of a given group.
func (k Keeper) GroupPoliciesByGroup(goCtx context.Context, request *group.QueryGroupPoliciesByGroupRequest) (*group.QueryGroupPoliciesByGroupResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)
	groupID := request.GroupId
	it, err := k.getGroupPoliciesByGroup(ctx, groupID, request.Pagination)
	if err != nil {
		return nil, err
	}

	var policies []*group.GroupPolicyInfo
	pageRes, err := orm.Paginate(it, request.Pagination, &policies)
	if err != nil {
		return nil, err
	}

	return &group.QueryGroupPoliciesByGroupResponse{
		GroupPolicies: policies,
		Pagination:    pageRes,
	}, nil
}

// getGroupPoliciesByGroup returns an iterator for the given group id and page request.
func (k Keeper) getGroupPoliciesByGroup(ctx sdk.Context, id uint64, pageRequest *query.PageRequest) (orm.Iterator, error) {
	return k.groupPolicyByGroupIndex.GetPaginated(ctx.KVStore(k.key), id, pageRequest)
}

// GroupPoliciesByAdmin queries all groups policies where a given address is
// admin.
func (k Keeper) GroupPoliciesByAdmin(goCtx context.Context, request *group.QueryGroupPoliciesByAdminRequest) (*group.QueryGroupPoliciesByAdminResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)
	addr, err := sdk.AccAddressFromBech32(request.Admin)
	if err != nil {
		return nil, err
	}
	it, err := k.getGroupPoliciesByAdmin(ctx, addr, request.Pagination)
	if err != nil {
		return nil, err
	}

	var policies []*group.GroupPolicyInfo
	pageRes, err := orm.Paginate(it, request.Pagination, &policies)
	if err != nil {
		return nil, err
	}

	return &group.QueryGroupPoliciesByAdminResponse{
		GroupPolicies: policies,
		Pagination:    pageRes,
	}, nil
}

// getGroupPoliciesByAdmin returns an iterator for the given admin account address and page request.
func (k Keeper) getGroupPoliciesByAdmin(ctx sdk.Context, admin sdk.AccAddress, pageRequest *query.PageRequest) (orm.Iterator, error) {
	return k.groupPolicyByAdminIndex.GetPaginated(ctx.KVStore(k.key), admin.Bytes(), pageRequest)
}

// Proposal queries a proposal.
func (k Keeper) Proposal(goCtx context.Context, request *group.QueryProposalRequest) (*group.QueryProposalResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)
	proposalID := request.ProposalId
	proposal, err := k.getProposal(ctx, proposalID)
	if err != nil {
		return nil, err
	}

	return &group.QueryProposalResponse{Proposal: &proposal}, nil
}

// ProposalsByGroupPolicy queries all proposals of a group policy.
func (k Keeper) ProposalsByGroupPolicy(goCtx context.Context, request *group.QueryProposalsByGroupPolicyRequest) (*group.QueryProposalsByGroupPolicyResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)
	addr, err := sdk.AccAddressFromBech32(request.Address)
	if err != nil {
		return nil, err
	}
	it, err := k.getProposalsByGroupPolicy(ctx, addr, request.Pagination)
	if err != nil {
		return nil, err
	}

	var proposals []*group.Proposal
	pageRes, err := orm.Paginate(it, request.Pagination, &proposals)
	if err != nil {
		return nil, err
	}

	return &group.QueryProposalsByGroupPolicyResponse{
		Proposals:  proposals,
		Pagination: pageRes,
	}, nil
}

// getProposalsByGroupPolicy returns an iterator for the given account address and page request.
func (k Keeper) getProposalsByGroupPolicy(ctx sdk.Context, account sdk.AccAddress, pageRequest *query.PageRequest) (orm.Iterator, error) {
	return k.proposalByGroupPolicyIndex.GetPaginated(ctx.KVStore(k.key), account.Bytes(), pageRequest)
}

// getProposal gets the proposal info of the given proposal id.
func (k Keeper) getProposal(ctx sdk.Context, proposalID uint64) (group.Proposal, error) {
	var p group.Proposal
	if _, err := k.proposalTable.GetOne(ctx.KVStore(k.key), proposalID, &p); err != nil {
		return group.Proposal{}, sdkerrors.Wrap(err, "load proposal")
	}
	return p, nil
}

// VoteByProposalVoter queries a vote given a voter and a proposal ID.
func (k Keeper) VoteByProposalVoter(goCtx context.Context, request *group.QueryVoteByProposalVoterRequest) (*group.QueryVoteByProposalVoterResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)
	addr, err := sdk.AccAddressFromBech32(request.Voter)
	if err != nil {
		return nil, err
	}
	proposalID := request.ProposalId
	vote, err := k.getVote(ctx, proposalID, addr)
	if err != nil {
		return nil, err
	}
	return &group.QueryVoteByProposalVoterResponse{
		Vote: &vote,
	}, nil
}

// VotesByProposal queries all votes on a proposal.
func (k Keeper) VotesByProposal(goCtx context.Context, request *group.QueryVotesByProposalRequest) (*group.QueryVotesByProposalResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)
	proposalID := request.ProposalId
	it, err := k.getVotesByProposal(ctx, proposalID, request.Pagination)
	if err != nil {
		return nil, err
	}

	var votes []*group.Vote
	pageRes, err := orm.Paginate(it, request.Pagination, &votes)
	if err != nil {
		return nil, err
	}

	return &group.QueryVotesByProposalResponse{
		Votes:      votes,
		Pagination: pageRes,
	}, nil
}

// VotesByVoter queries all votes of a voter.
func (k Keeper) VotesByVoter(goCtx context.Context, request *group.QueryVotesByVoterRequest) (*group.QueryVotesByVoterResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)
	addr, err := sdk.AccAddressFromBech32(request.Voter)
	if err != nil {
		return nil, err
	}
	it, err := k.getVotesByVoter(ctx, addr, request.Pagination)
	if err != nil {
		return nil, err
	}

	var votes []*group.Vote
	pageRes, err := orm.Paginate(it, request.Pagination, &votes)
	if err != nil {
		return nil, err
	}

	return &group.QueryVotesByVoterResponse{
		Votes:      votes,
		Pagination: pageRes,
	}, nil
}

// GroupsByMember queries all groups where the given address is a member of.
func (k Keeper) GroupsByMember(goCtx context.Context, request *group.QueryGroupsByMemberRequest) (*group.QueryGroupsByMemberResponse, error) {
	if request == nil {
		return nil, status.Errorf(codes.InvalidArgument, "empty request")
	}

	ctx := sdk.UnwrapSDKContext(goCtx)
	member, err := sdk.AccAddressFromBech32(request.Address)
	if err != nil {
		return nil, err
	}

	iter, err := k.groupMemberByMemberIndex.GetPaginated(ctx.KVStore(k.key), member.Bytes(), request.Pagination)
	if err != nil {
		return nil, err
	}

	var members []*group.GroupMember
	pageRes, err := orm.Paginate(iter, request.Pagination, &members)
	if err != nil {
		return nil, err
	}

	var groups []*group.GroupInfo
	for _, gm := range members {
		groupInfo, err := k.getGroupInfo(ctx, gm.GroupId)
		if err != nil {
			return nil, err
		}
		groups = append(groups, &groupInfo)
	}

	return &group.QueryGroupsByMemberResponse{
		Groups:     groups,
		Pagination: pageRes,
	}, nil
}

// getVote gets the vote info for the given proposal id and voter address.
func (k Keeper) getVote(ctx sdk.Context, proposalID uint64, voter sdk.AccAddress) (group.Vote, error) {
	var v group.Vote
	return v, k.voteTable.GetOne(ctx.KVStore(k.key), orm.PrimaryKey(&group.Vote{ProposalId: proposalID, Voter: voter.String()}), &v)
}

// getVotesByProposal returns an iterator for the given proposal id and page request.
func (k Keeper) getVotesByProposal(ctx sdk.Context, proposalID uint64, pageRequest *query.PageRequest) (orm.Iterator, error) {
	return k.voteByProposalIndex.GetPaginated(ctx.KVStore(k.key), proposalID, pageRequest)
}

// getVotesByVoter returns an iterator for the given voter address and page request.
func (k Keeper) getVotesByVoter(ctx sdk.Context, voter sdk.AccAddress, pageRequest *query.PageRequest) (orm.Iterator, error) {
	return k.voteByVoterIndex.GetPaginated(ctx.KVStore(k.key), voter.Bytes(), pageRequest)
}

// TallyResult computes the live tally result of a proposal.
func (k Keeper) TallyResult(goCtx context.Context, request *group.QueryTallyResultRequest) (*group.QueryTallyResultResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)
	proposalID := request.ProposalId

	proposal, err := k.getProposal(ctx, proposalID)
	if err != nil {
		return nil, err
	}

	if proposal.Status == group.PROPOSAL_STATUS_WITHDRAWN || proposal.Status == group.PROPOSAL_STATUS_ABORTED {
		return nil, sdkerrors.Wrapf(errors.ErrInvalid, "can't get the tally of a proposal with status %s", proposal.Status)
	}

	var policyInfo group.GroupPolicyInfo
	if policyInfo, err = k.getGroupPolicyInfo(ctx, proposal.GroupPolicyAddress); err != nil {
		return nil, sdkerrors.Wrap(err, "load group policy")
	}

	tallyResult, err := k.Tally(ctx, proposal, policyInfo.GroupId)
	if err != nil {
		return nil, err
	}

	return &group.QueryTallyResultResponse{
		Tally: tallyResult,
	}, nil
}

// Groups returns all the groups present in the state.
func (k Keeper) Groups(goCtx context.Context, request *group.QueryGroupsRequest) (*group.QueryGroupsResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)

	it, err := k.groupTable.PrefixScan(ctx.KVStore(k.key), 1, math.MaxUint64)
	if err != nil {
		return nil, err
	}
	defer it.Close()

	var groups []*group.GroupInfo
	pageRes, err := orm.Paginate(it, request.Pagination, &groups)
	if err != nil {
		return nil, err
	}

	return &group.QueryGroupsResponse{
		Groups:     groups,
		Pagination: pageRes,
	}, nil
}
