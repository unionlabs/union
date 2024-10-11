package keeper_test

import (
	"bytes"
	"context"
	"errors"
	"sort"
	"strings"
	"time"

	"github.com/golang/mock/gomock"

	"cosmossdk.io/core/header"
	banktypes "cosmossdk.io/x/bank/types"
	"cosmossdk.io/x/group"
	"cosmossdk.io/x/group/internal/math"
	"cosmossdk.io/x/group/keeper"
	minttypes "cosmossdk.io/x/mint/types"

	"github.com/cosmos/cosmos-sdk/codec/address"
	simtestutil "github.com/cosmos/cosmos-sdk/testutil/sims"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/query"
)

var EventProposalPruned = "cosmos.group.v1.EventProposalPruned"

func (s *TestSuite) TestCreateGroupWithLotsOfMembers() {
	for i := 50; i < 70; i++ {
		membersResp := s.createGroupAndGetMembers(i)
		s.Require().Equal(len(membersResp), i)
	}
}

func (s *TestSuite) createGroupAndGetMembers(numMembers int) []*group.GroupMember {
	addressPool := simtestutil.CreateIncrementalAccounts(numMembers)
	members := make([]group.MemberRequest, numMembers)
	for i := 0; i < len(members); i++ {
		addr, err := s.accountKeeper.AddressCodec().BytesToString(addressPool[i])
		s.Require().NoError(err)
		members[i] = group.MemberRequest{
			Address: addr,
			Weight:  "1",
		}
		s.accountKeeper.EXPECT().AddressCodec().Return(address.NewBech32Codec("cosmos")).AnyTimes()
	}

	g, err := s.groupKeeper.CreateGroup(s.ctx, &group.MsgCreateGroup{
		Admin:   members[0].Address,
		Members: members,
	})
	s.Require().NoErrorf(err, "failed to create group with %d members", len(members))
	s.T().Logf("group %d created with %d members", g.GroupId, len(members))

	groupMemberResp, err := s.groupKeeper.GroupMembers(s.ctx, &group.QueryGroupMembersRequest{GroupId: g.GroupId})
	s.Require().NoError(err)

	s.T().Logf("got %d members from group %d", len(groupMemberResp.Members), g.GroupId)

	return groupMemberResp.Members
}

func (s *TestSuite) TestCreateGroup() {
	members := []group.MemberRequest{{
		Address: s.addrsStr[4],
		Weight:  "1",
	}, {
		Address: s.addrsStr[5],
		Weight:  "2",
	}}

	expGroups := []*group.GroupInfo{
		{
			Id:          s.groupID,
			Version:     1,
			Admin:       s.addrsStr[0],
			TotalWeight: "3",
			CreatedAt:   s.blockTime,
		},
		{
			Id:          2,
			Version:     1,
			Admin:       s.addrsStr[0],
			TotalWeight: "3",
			CreatedAt:   s.blockTime,
		},
	}

	specs := map[string]struct {
		req       *group.MsgCreateGroup
		expErr    bool
		expErrMsg string
		expGroups []*group.GroupInfo
	}{
		"all good": {
			req: &group.MsgCreateGroup{
				Admin:   s.addrsStr[0],
				Members: members,
			},
			expGroups: expGroups,
		},
		"group metadata: metadata too long": {
			req: &group.MsgCreateGroup{
				Admin:    s.addrsStr[0],
				Members:  members,
				Metadata: strings.Repeat("a", 256),
			},
			expErr:    true,
			expErrMsg: "group metadata: metadata too long",
		},
		"invalid member address": {
			req: &group.MsgCreateGroup{
				Admin: s.addrsStr[0],
				Members: []group.MemberRequest{{
					Address: "invalid",
					Weight:  "1",
				}},
			},
			expErr:    true,
			expErrMsg: "member address invalid",
		},
		"member metadata too long": {
			req: &group.MsgCreateGroup{
				Admin: s.addrsStr[0],
				Members: []group.MemberRequest{{
					Address:  s.addrsStr[2],
					Weight:   "1",
					Metadata: strings.Repeat("a", 256),
				}},
			},
			expErr:    true,
			expErrMsg: "metadata too long",
		},
		"zero member weight": {
			req: &group.MsgCreateGroup{
				Admin: s.addrsStr[0],
				Members: []group.MemberRequest{{
					Address: s.addrsStr[2],
					Weight:  "0",
				}},
			},
			expErr:    true,
			expErrMsg: "expected a positive decimal",
		},
		"invalid member weight - Inf": {
			req: &group.MsgCreateGroup{
				Admin: s.addrsStr[0],
				Members: []group.MemberRequest{{
					Address: s.addrsStr[2],
					Weight:  "inf",
				}},
			},
			expErr:    true,
			expErrMsg: "expected a finite decimal",
		},
		"invalid member weight - NaN": {
			req: &group.MsgCreateGroup{
				Admin: s.addrsStr[0],
				Members: []group.MemberRequest{{
					Address: s.addrsStr[2],
					Weight:  "NaN",
				}},
			},
			expErr:    true,
			expErrMsg: "expected a finite decimal",
		},
	}

	var seq uint32 = 1
	for msg, spec := range specs {
		s.Run(msg, func() {
			blockTime := sdk.UnwrapSDKContext(s.ctx).HeaderInfo().Time
			res, err := s.groupKeeper.CreateGroup(s.ctx, spec.req)
			if spec.expErr {
				s.Require().Error(err)
				s.Require().Contains(err.Error(), spec.expErrMsg)
				_, err := s.groupKeeper.GroupInfo(s.ctx, &group.QueryGroupInfoRequest{GroupId: uint64(seq + 1)})
				s.Require().Error(err)
				return
			}

			s.Require().NoError(err)
			id := res.GroupId

			seq++
			s.Assert().Equal(uint64(seq), id)

			// then all data persisted
			loadedGroupRes, err := s.groupKeeper.GroupInfo(s.ctx, &group.QueryGroupInfoRequest{GroupId: id})
			s.Require().NoError(err)
			s.Assert().Equal(spec.req.Admin, loadedGroupRes.Info.Admin)
			s.Assert().Equal(spec.req.Metadata, loadedGroupRes.Info.Metadata)
			s.Assert().Equal(id, loadedGroupRes.Info.Id)
			s.Assert().Equal(uint64(1), loadedGroupRes.Info.Version)

			// and members are stored as well
			membersRes, err := s.groupKeeper.GroupMembers(s.ctx, &group.QueryGroupMembersRequest{GroupId: id})
			s.Require().NoError(err)
			loadedMembers := membersRes.Members
			s.Require().Equal(len(members), len(loadedMembers))
			// we reorder members by address to be able to compare them
			sort.Slice(members, func(i, j int) bool {
				addri, err := sdk.AccAddressFromBech32(members[i].Address)
				s.Require().NoError(err)
				addrj, err := sdk.AccAddressFromBech32(members[j].Address)
				s.Require().NoError(err)
				return bytes.Compare(addri, addrj) < 0
			})
			for i := range loadedMembers {
				s.Assert().Equal(members[i].Metadata, loadedMembers[i].Member.Metadata)
				s.Assert().Equal(members[i].Address, loadedMembers[i].Member.Address)
				s.Assert().Equal(members[i].Weight, loadedMembers[i].Member.Weight)
				s.Assert().Equal(blockTime, loadedMembers[i].Member.AddedAt)
				s.Assert().Equal(id, loadedMembers[i].GroupId)
			}

			// query groups by admin
			groupsRes, err := s.groupKeeper.GroupsByAdmin(s.ctx, &group.QueryGroupsByAdminRequest{Admin: s.addrsStr[0]})
			s.Require().NoError(err)
			loadedGroups := groupsRes.Groups
			s.Require().Equal(len(spec.expGroups), len(loadedGroups))
			for i := range loadedGroups {
				s.Assert().Equal(spec.expGroups[i].Metadata, loadedGroups[i].Metadata)
				s.Assert().Equal(spec.expGroups[i].Admin, loadedGroups[i].Admin)
				s.Assert().Equal(spec.expGroups[i].TotalWeight, loadedGroups[i].TotalWeight)
				s.Assert().Equal(spec.expGroups[i].Id, loadedGroups[i].Id)
				s.Assert().Equal(spec.expGroups[i].Version, loadedGroups[i].Version)
				s.Assert().Equal(spec.expGroups[i].CreatedAt, loadedGroups[i].CreatedAt)
			}
		})
	}
}

func (s *TestSuite) TestUpdateGroupMembers() {
	member1 := s.addrsStr[4]
	member2 := s.addrsStr[5]
	members := []group.MemberRequest{{
		Address: member1,
		Weight:  "1",
	}}

	myAdmin := s.addrsStr[3]
	groupRes, err := s.groupKeeper.CreateGroup(s.ctx, &group.MsgCreateGroup{
		Admin:   myAdmin,
		Members: members,
	})
	s.Require().NoError(err)
	groupID := groupRes.GroupId

	specs := map[string]struct {
		req        *group.MsgUpdateGroupMembers
		expErr     bool
		expErrMsg  string
		expGroup   *group.GroupInfo
		expMembers []*group.GroupMember
	}{
		"empty group id": {
			req: &group.MsgUpdateGroupMembers{
				GroupId: 0,
				Admin:   myAdmin,
				MemberUpdates: []group.MemberRequest{{
					Address: member2,
					Weight:  "2",
				}},
			},
			expErr:    true,
			expErrMsg: "value is empty",
		},
		"no new members": {
			req: &group.MsgUpdateGroupMembers{
				GroupId:       groupID,
				Admin:         myAdmin,
				MemberUpdates: []group.MemberRequest{},
			},
			expErr:    true,
			expErrMsg: "value is empty",
		},
		"invalid member": {
			req: &group.MsgUpdateGroupMembers{
				GroupId: groupID,
				Admin:   myAdmin,
				MemberUpdates: []group.MemberRequest{
					{},
				},
			},
			expErr:    true,
			expErrMsg: "empty address string is not allowed",
		},
		"invalid member metadata too long": {
			req: &group.MsgUpdateGroupMembers{
				GroupId: groupID,
				Admin:   myAdmin,
				MemberUpdates: []group.MemberRequest{
					{
						Address:  member2,
						Weight:   "2",
						Metadata: strings.Repeat("a", 10240),
					},
				},
			},
			expErr:    true,
			expErrMsg: "members updated: group member metadata: metadata too lon",
		},
		"add new member": {
			req: &group.MsgUpdateGroupMembers{
				GroupId: groupID,
				Admin:   myAdmin,
				MemberUpdates: []group.MemberRequest{{
					Address: member2,
					Weight:  "2",
				}},
			},
			expGroup: &group.GroupInfo{
				Id:          groupID,
				Admin:       myAdmin,
				TotalWeight: "3",
				Version:     2,
				CreatedAt:   s.blockTime,
			},
			expMembers: []*group.GroupMember{
				{
					Member: &group.Member{
						Address: member2,
						Weight:  "2",
						AddedAt: s.sdkCtx.HeaderInfo().Time,
					},
					GroupId: groupID,
				},
				{
					Member: &group.Member{
						Address: member1,
						Weight:  "1",
						AddedAt: s.blockTime,
					},
					GroupId: groupID,
				},
			},
		},
		"update member": {
			req: &group.MsgUpdateGroupMembers{
				GroupId: groupID,
				Admin:   myAdmin,
				MemberUpdates: []group.MemberRequest{{
					Address: member1,
					Weight:  "2",
				}},
			},
			expGroup: &group.GroupInfo{
				Id:          groupID,
				Admin:       myAdmin,
				TotalWeight: "2",
				Version:     2,
				CreatedAt:   s.blockTime,
			},
			expMembers: []*group.GroupMember{
				{
					GroupId: groupID,
					Member: &group.Member{
						Address: member1,
						Weight:  "2",
						AddedAt: s.blockTime,
					},
				},
			},
		},
		"update member with same data": {
			req: &group.MsgUpdateGroupMembers{
				GroupId: groupID,
				Admin:   myAdmin,
				MemberUpdates: []group.MemberRequest{{
					Address: member1,
					Weight:  "1",
				}},
			},
			expGroup: &group.GroupInfo{
				Id:          groupID,
				Admin:       myAdmin,
				TotalWeight: "1",
				Version:     2,
				CreatedAt:   s.blockTime,
			},
			expMembers: []*group.GroupMember{
				{
					GroupId: groupID,
					Member: &group.Member{
						Address: member1,
						Weight:  "1",
						AddedAt: s.blockTime,
					},
				},
			},
		},
		"replace member": {
			req: &group.MsgUpdateGroupMembers{
				GroupId: groupID,
				Admin:   myAdmin,
				MemberUpdates: []group.MemberRequest{
					{
						Address: member1,
						Weight:  "0",
					},
					{
						Address: member2,
						Weight:  "1",
					},
				},
			},
			expGroup: &group.GroupInfo{
				Id:          groupID,
				Admin:       myAdmin,
				TotalWeight: "1",
				Version:     2,
				CreatedAt:   s.blockTime,
			},
			expMembers: []*group.GroupMember{{
				GroupId: groupID,
				Member: &group.Member{
					Address: member2,
					Weight:  "1",
					AddedAt: s.sdkCtx.HeaderInfo().Time,
				},
			}},
		},
		"remove existing member": {
			req: &group.MsgUpdateGroupMembers{
				GroupId: groupID,
				Admin:   myAdmin,
				MemberUpdates: []group.MemberRequest{{
					Address: member1,
					Weight:  "0",
				}},
			},
			expGroup: &group.GroupInfo{
				Id:          groupID,
				Admin:       myAdmin,
				TotalWeight: "0",
				Version:     2,
				CreatedAt:   s.blockTime,
			},
			expMembers: []*group.GroupMember{},
		},
		"remove unknown member": {
			req: &group.MsgUpdateGroupMembers{
				GroupId: groupID,
				Admin:   myAdmin,
				MemberUpdates: []group.MemberRequest{{
					Address: s.addrsStr[3],
					Weight:  "0",
				}},
			},
			expErr: true,
			expGroup: &group.GroupInfo{
				Id:          groupID,
				Admin:       myAdmin,
				TotalWeight: "1",
				Version:     1,
				CreatedAt:   s.blockTime,
			},
			expMembers: []*group.GroupMember{{
				GroupId: groupID,
				Member: &group.Member{
					Address: member1,
					Weight:  "1",
				},
			}},
		},
		"with wrong admin": {
			req: &group.MsgUpdateGroupMembers{
				GroupId: groupID,
				Admin:   s.addrsStr[2],
				MemberUpdates: []group.MemberRequest{{
					Address: member1,
					Weight:  "2",
				}},
			},
			expErr:    true,
			expErrMsg: "not group admin",
			expGroup: &group.GroupInfo{
				Id:          groupID,
				Admin:       myAdmin,
				TotalWeight: "1",
				Version:     1,
				CreatedAt:   s.blockTime,
			},
			expMembers: []*group.GroupMember{{
				GroupId: groupID,
				Member: &group.Member{
					Address: member1,
					Weight:  "1",
				},
			}},
		},
		"with unknown groupID": {
			req: &group.MsgUpdateGroupMembers{
				GroupId: 999,
				Admin:   myAdmin,
				MemberUpdates: []group.MemberRequest{{
					Address: member1,
					Weight:  "2",
				}},
			},
			expErr:    true,
			expErrMsg: "not found",
			expGroup: &group.GroupInfo{
				Id:          groupID,
				Admin:       myAdmin,
				TotalWeight: "1",
				Version:     1,
				CreatedAt:   s.blockTime,
			},
			expMembers: []*group.GroupMember{{
				GroupId: groupID,
				Member: &group.Member{
					Address: member1,
					Weight:  "1",
				},
			}},
		},
	}
	for msg, spec := range specs {
		s.Run(msg, func() {
			sdkCtx, _ := s.sdkCtx.CacheContext()
			_, err := s.groupKeeper.UpdateGroupMembers(sdkCtx, spec.req)
			if spec.expErr {
				s.Require().Error(err)
				s.Require().Contains(err.Error(), spec.expErrMsg)
				return
			}
			s.Require().NoError(err)

			// then
			res, err := s.groupKeeper.GroupInfo(sdkCtx, &group.QueryGroupInfoRequest{GroupId: groupID})
			s.Require().NoError(err)
			s.Assert().Equal(spec.expGroup, res.Info)

			// and members persisted
			membersRes, err := s.groupKeeper.GroupMembers(sdkCtx, &group.QueryGroupMembersRequest{GroupId: groupID})
			s.Require().NoError(err)
			loadedMembers := membersRes.Members
			s.Require().Equal(len(spec.expMembers), len(loadedMembers))
			// we reorder group members by address to be able to compare them
			sort.Slice(spec.expMembers, func(i, j int) bool {
				addri, err := sdk.AccAddressFromBech32(spec.expMembers[i].Member.Address)
				s.Require().NoError(err)
				addrj, err := sdk.AccAddressFromBech32(spec.expMembers[j].Member.Address)
				s.Require().NoError(err)
				return bytes.Compare(addri, addrj) < 0
			})
			for i := range loadedMembers {
				s.Assert().Equal(spec.expMembers[i].Member.Metadata, loadedMembers[i].Member.Metadata)
				s.Assert().Equal(spec.expMembers[i].Member.Address, loadedMembers[i].Member.Address)
				s.Assert().Equal(spec.expMembers[i].Member.Weight, loadedMembers[i].Member.Weight)
				s.Assert().Equal(spec.expMembers[i].Member.AddedAt, loadedMembers[i].Member.AddedAt)
				s.Assert().Equal(spec.expMembers[i].GroupId, loadedMembers[i].GroupId)
			}

			events := sdkCtx.EventManager().ABCIEvents()
			s.Require().Len(events, 1) // EventUpdateGroup
		})
	}
}

func (s *TestSuite) TestUpdateGroupAdmin() {
	members := []group.MemberRequest{{
		Address: s.addrsStr[0],
		Weight:  "1",
	}}
	oldAdmin := s.addrsStr[1]
	newAdmin := s.addrsStr[2]
	groupRes, err := s.groupKeeper.CreateGroup(s.ctx, &group.MsgCreateGroup{
		Admin:   oldAdmin,
		Members: members,
	})
	s.Require().NoError(err)
	groupID := groupRes.GroupId
	specs := map[string]struct {
		req       *group.MsgUpdateGroupAdmin
		expStored *group.GroupInfo
		expErr    bool
		expErrMsg string
	}{
		"with no groupID": {
			req: &group.MsgUpdateGroupAdmin{
				GroupId:  0,
				Admin:    oldAdmin,
				NewAdmin: newAdmin,
			},
			expErr:    true,
			expErrMsg: "value is empty",
		},
		"with identical admin and new admin": {
			req: &group.MsgUpdateGroupAdmin{
				GroupId:  groupID,
				Admin:    oldAdmin,
				NewAdmin: oldAdmin,
			},
			expErr:    true,
			expErrMsg: "new and old admin are the same",
		},
		"with correct admin": {
			req: &group.MsgUpdateGroupAdmin{
				GroupId:  groupID,
				Admin:    oldAdmin,
				NewAdmin: newAdmin,
			},
			expStored: &group.GroupInfo{
				Id:          groupID,
				Admin:       newAdmin,
				TotalWeight: "1",
				Version:     2,
				CreatedAt:   s.blockTime,
			},
		},
		"with wrong admin": {
			req: &group.MsgUpdateGroupAdmin{
				GroupId:  groupID,
				Admin:    s.addrsStr[3],
				NewAdmin: newAdmin,
			},
			expErr:    true,
			expErrMsg: "not group admin",
			expStored: &group.GroupInfo{
				Id:          groupID,
				Admin:       oldAdmin,
				TotalWeight: "1",
				Version:     1,
				CreatedAt:   s.blockTime,
			},
		},
		"with unknown groupID": {
			req: &group.MsgUpdateGroupAdmin{
				GroupId:  999,
				Admin:    oldAdmin,
				NewAdmin: newAdmin,
			},
			expErr:    true,
			expErrMsg: "not found",
			expStored: &group.GroupInfo{
				Id:          groupID,
				Admin:       oldAdmin,
				TotalWeight: "1",
				Version:     1,
				CreatedAt:   s.blockTime,
			},
		},
		"with invalid new admin address": {
			req: &group.MsgUpdateGroupAdmin{
				GroupId:  groupID,
				Admin:    oldAdmin,
				NewAdmin: "%s",
			},
			expErr:    true,
			expErrMsg: "new admin address",
		},
	}
	for msg, spec := range specs {
		s.Run(msg, func() {
			_, err := s.groupKeeper.UpdateGroupAdmin(s.ctx, spec.req)
			if spec.expErr {
				s.Require().Error(err)
				s.Require().Contains(err.Error(), spec.expErrMsg)
				return
			}
			s.Require().NoError(err)

			// then
			res, err := s.groupKeeper.GroupInfo(s.ctx, &group.QueryGroupInfoRequest{GroupId: groupID})
			s.Require().NoError(err)
			s.Assert().Equal(spec.expStored, res.Info)
		})
	}
}

func (s *TestSuite) TestUpdateGroupMetadata() {
	oldAdmin := s.addrsStr[0]
	groupID := s.groupID

	specs := map[string]struct {
		req       *group.MsgUpdateGroupMetadata
		expErr    bool
		expStored *group.GroupInfo
	}{
		"with correct admin": {
			req: &group.MsgUpdateGroupMetadata{
				GroupId: groupID,
				Admin:   oldAdmin,
			},
			expStored: &group.GroupInfo{
				Id:          groupID,
				Admin:       oldAdmin,
				TotalWeight: "3",
				Version:     2,
				CreatedAt:   s.blockTime,
			},
		},
		"with wrong admin": {
			req: &group.MsgUpdateGroupMetadata{
				GroupId: groupID,
				Admin:   s.addrsStr[2],
			},
			expErr: true,
			expStored: &group.GroupInfo{
				Id:          groupID,
				Admin:       oldAdmin,
				TotalWeight: "1",
				Version:     1,
				CreatedAt:   s.blockTime,
			},
		},
		"with unknown groupid": {
			req: &group.MsgUpdateGroupMetadata{
				GroupId: 999,
				Admin:   oldAdmin,
			},
			expErr: true,
			expStored: &group.GroupInfo{
				Id:          groupID,
				Admin:       oldAdmin,
				TotalWeight: "1",
				Version:     1,
				CreatedAt:   s.blockTime,
			},
		},
	}
	for msg, spec := range specs {
		s.Run(msg, func() {
			sdkCtx, _ := s.sdkCtx.CacheContext()
			_, err := s.groupKeeper.UpdateGroupMetadata(sdkCtx, spec.req)
			if spec.expErr {
				s.Require().Error(err)
				return
			}
			s.Require().NoError(err)

			// then
			res, err := s.groupKeeper.GroupInfo(sdkCtx, &group.QueryGroupInfoRequest{GroupId: groupID})
			s.Require().NoError(err)
			s.Assert().Equal(spec.expStored, res.Info)

			events := sdkCtx.EventManager().ABCIEvents()
			s.Require().Len(events, 1) // EventUpdateGroup
		})
	}
}

func (s *TestSuite) TestCreateGroupWithPolicy() {
	s.setNextAccount()

	members := []group.MemberRequest{{
		Address: s.addrsStr[4],
		Weight:  "1",
	}, {
		Address: s.addrsStr[5],
		Weight:  "2",
	}}

	specs := map[string]struct {
		req       *group.MsgCreateGroupWithPolicy
		policy    group.DecisionPolicy
		malleate  func()
		expErr    bool
		expErrMsg string
	}{
		"all good": {
			req: &group.MsgCreateGroupWithPolicy{
				Admin:              s.addrsStr[0],
				Members:            members,
				GroupPolicyAsAdmin: false,
			},
			malleate: func() {
				s.setNextAccount()
			},
			policy: group.NewThresholdDecisionPolicy(
				"1",
				time.Second,
				0,
			),
		},
		"group policy as admin is true": {
			req: &group.MsgCreateGroupWithPolicy{
				Admin:              s.addrsStr[0],
				Members:            members,
				GroupPolicyAsAdmin: true,
			},
			malleate: func() {
				s.setNextAccount()
			},
			policy: group.NewThresholdDecisionPolicy(
				"1",
				time.Second,
				0,
			),
		},
		"group metadata too long": {
			req: &group.MsgCreateGroupWithPolicy{
				Admin:              s.addrsStr[0],
				Members:            members,
				GroupPolicyAsAdmin: false,
				GroupMetadata:      strings.Repeat("a", 256),
			},
			policy: group.NewThresholdDecisionPolicy(
				"1",
				time.Second,
				0,
			),
			expErr:    true,
			expErrMsg: "group response: group metadata: metadata too long",
		},
		"group policy metadata: metadata too long": {
			req: &group.MsgCreateGroupWithPolicy{
				Admin:               s.addrsStr[0],
				Members:             members,
				GroupPolicyAsAdmin:  false,
				GroupPolicyMetadata: strings.Repeat("a", 256),
			},
			policy: group.NewThresholdDecisionPolicy(
				"1",
				time.Second,
				0,
			),
			expErr:    true,
			expErrMsg: "group policy metadata: metadata too long",
		},
		"member metadata too long": {
			req: &group.MsgCreateGroupWithPolicy{
				Admin: s.addrsStr[0],
				Members: []group.MemberRequest{{
					Address:  s.addrsStr[2],
					Weight:   "1",
					Metadata: strings.Repeat("a", 256),
				}},
				GroupPolicyAsAdmin: false,
			},
			policy: group.NewThresholdDecisionPolicy(
				"1",
				time.Second,
				0,
			),
			expErr:    true,
			expErrMsg: "group response: member metadata: metadata too long",
		},
		"zero member weight": {
			req: &group.MsgCreateGroupWithPolicy{
				Admin: s.addrsStr[0],
				Members: []group.MemberRequest{{
					Address: s.addrsStr[2],
					Weight:  "0",
				}},
				GroupPolicyAsAdmin: false,
			},
			policy: group.NewThresholdDecisionPolicy(
				"1",
				time.Second,
				0,
			),
			expErr:    true,
			expErrMsg: "expected a positive decimal",
		},
		"invalid member address": {
			req: &group.MsgCreateGroupWithPolicy{
				Admin: s.addrsStr[0],
				Members: []group.MemberRequest{{
					Address: "invalid",
					Weight:  "1",
				}},
				GroupPolicyAsAdmin: false,
			},
			policy: group.NewThresholdDecisionPolicy(
				"1",
				time.Second,
				0,
			),
			expErr:    true,
			expErrMsg: "decoding bech32 failed",
		},
		"decision policy threshold > total group weight": {
			req: &group.MsgCreateGroupWithPolicy{
				Admin:              s.addrsStr[0],
				Members:            members,
				GroupPolicyAsAdmin: false,
			},
			malleate: func() {
				s.setNextAccount()
			},
			policy: group.NewThresholdDecisionPolicy(
				"10",
				time.Second,
				0,
			),
			expErr: false,
		},
	}

	for msg, spec := range specs {
		s.Run(msg, func() {
			s.setNextAccount()
			err := spec.req.SetDecisionPolicy(spec.policy)
			s.Require().NoError(err)

			blockTime := sdk.UnwrapSDKContext(s.ctx).HeaderInfo().Time
			res, err := s.groupKeeper.CreateGroupWithPolicy(s.ctx, spec.req)
			if spec.expErr {
				s.Require().Error(err)
				s.Require().Contains(err.Error(), spec.expErrMsg)
				return
			}
			s.Require().NoError(err)
			id := res.GroupId
			groupPolicyAddr := res.GroupPolicyAddress

			// then all data persisted in group
			loadedGroupRes, err := s.groupKeeper.GroupInfo(s.ctx, &group.QueryGroupInfoRequest{GroupId: id})
			s.Require().NoError(err)
			s.Assert().Equal(spec.req.GroupMetadata, loadedGroupRes.Info.Metadata)
			s.Assert().Equal(id, loadedGroupRes.Info.Id)
			if spec.req.GroupPolicyAsAdmin {
				s.Assert().NotEqual(spec.req.Admin, loadedGroupRes.Info.Admin)
				s.Assert().Equal(groupPolicyAddr, loadedGroupRes.Info.Admin)
			} else {
				s.Assert().Equal(spec.req.Admin, loadedGroupRes.Info.Admin)
			}

			// and members are stored as well
			membersRes, err := s.groupKeeper.GroupMembers(s.ctx, &group.QueryGroupMembersRequest{GroupId: id})
			s.Require().NoError(err)
			loadedMembers := membersRes.Members
			s.Require().Equal(len(members), len(loadedMembers))
			// we reorder members by address to be able to compare them
			sort.Slice(members, func(i, j int) bool {
				addri, err := sdk.AccAddressFromBech32(members[i].Address)
				s.Require().NoError(err)
				addrj, err := sdk.AccAddressFromBech32(members[j].Address)
				s.Require().NoError(err)
				return bytes.Compare(addri, addrj) < 0
			})
			for i := range loadedMembers {
				s.Assert().Equal(members[i].Metadata, loadedMembers[i].Member.Metadata)
				s.Assert().Equal(members[i].Address, loadedMembers[i].Member.Address)
				s.Assert().Equal(members[i].Weight, loadedMembers[i].Member.Weight)
				s.Assert().Equal(blockTime, loadedMembers[i].Member.AddedAt)
				s.Assert().Equal(id, loadedMembers[i].GroupId)
			}

			// then all data persisted in group policy
			groupPolicyRes, err := s.groupKeeper.GroupPolicyInfo(s.ctx, &group.QueryGroupPolicyInfoRequest{Address: groupPolicyAddr})
			s.Require().NoError(err)

			groupPolicy := groupPolicyRes.Info
			s.Assert().Equal(groupPolicyAddr, groupPolicy.Address)
			s.Assert().Equal(id, groupPolicy.GroupId)
			s.Assert().Equal(spec.req.GroupPolicyMetadata, groupPolicy.Metadata)
			dp, err := groupPolicy.GetDecisionPolicy()
			s.Assert().NoError(err)
			s.Assert().Equal(spec.policy.(*group.ThresholdDecisionPolicy), dp)
			if spec.req.GroupPolicyAsAdmin {
				s.Assert().NotEqual(spec.req.Admin, groupPolicy.Admin)
				s.Assert().Equal(groupPolicyAddr, groupPolicy.Admin)
			} else {
				s.Assert().Equal(spec.req.Admin, groupPolicy.Admin)
			}
		})
	}
}

func (s *TestSuite) TestCreateGroupPolicy() {
	s.setNextAccount()
	groupRes, err := s.groupKeeper.CreateGroup(s.ctx, &group.MsgCreateGroup{
		Admin:   s.addrsStr[0],
		Members: nil,
	})
	s.Require().NoError(err)
	myGroupID := groupRes.GroupId

	specs := map[string]struct {
		req       *group.MsgCreateGroupPolicy
		policy    group.DecisionPolicy
		expErr    bool
		expErrMsg string
	}{
		"all good": {
			req: &group.MsgCreateGroupPolicy{
				Admin:   s.addrsStr[0],
				GroupId: myGroupID,
			},
			policy: group.NewThresholdDecisionPolicy(
				"1",
				time.Second,
				0,
			),
		},
		"all good with percentage decision policy": {
			req: &group.MsgCreateGroupPolicy{
				Admin:   s.addrsStr[0],
				GroupId: myGroupID,
			},
			policy: group.NewPercentageDecisionPolicy(
				"0.5",
				time.Second,
				0,
			),
		},
		"decision policy threshold > total group weight": {
			req: &group.MsgCreateGroupPolicy{
				Admin:   s.addrsStr[0],
				GroupId: myGroupID,
			},
			policy: group.NewThresholdDecisionPolicy(
				"10",
				time.Second,
				0,
			),
		},
		"group id does not exists": {
			req: &group.MsgCreateGroupPolicy{
				Admin:   s.addrsStr[0],
				GroupId: 9999,
			},
			policy: group.NewThresholdDecisionPolicy(
				"1",
				time.Second,
				0,
			),
			expErr:    true,
			expErrMsg: "not found",
		},
		"admin not group admin": {
			req: &group.MsgCreateGroupPolicy{
				Admin:   s.addrsStr[3],
				GroupId: myGroupID,
			},
			policy: group.NewThresholdDecisionPolicy(
				"1",
				time.Second,
				0,
			),
			expErr:    true,
			expErrMsg: "not group admin",
		},
		"metadata too long": {
			req: &group.MsgCreateGroupPolicy{
				Admin:    s.addrsStr[0],
				GroupId:  myGroupID,
				Metadata: strings.Repeat("a", 256),
			},
			policy: group.NewThresholdDecisionPolicy(
				"1",
				time.Second,
				0,
			),
			expErr:    true,
			expErrMsg: "group policy metadata: metadata too long",
		},
		"percentage decision policy with negative value": {
			req: &group.MsgCreateGroupPolicy{
				Admin:   s.addrsStr[0],
				GroupId: myGroupID,
			},
			policy: group.NewPercentageDecisionPolicy(
				"-0.5",
				time.Second,
				0,
			),
			expErr:    true,
			expErrMsg: "expected a positive decimal",
		},
		"percentage decision policy with value greater than 1": {
			req: &group.MsgCreateGroupPolicy{
				Admin:   s.addrsStr[0],
				GroupId: myGroupID,
			},
			policy: group.NewPercentageDecisionPolicy(
				"2",
				time.Second,
				0,
			),
			expErr:    true,
			expErrMsg: "percentage must be > 0 and <= 1",
		},
	}
	for msg, spec := range specs {
		s.Run(msg, func() {
			err := spec.req.SetDecisionPolicy(spec.policy)
			s.Require().NoError(err)

			s.setNextAccount()

			res, err := s.groupKeeper.CreateGroupPolicy(s.ctx, spec.req)
			if spec.expErr {
				s.Require().Error(err)
				s.Require().Contains(err.Error(), spec.expErrMsg)
				return
			}
			s.Require().NoError(err)
			addr := res.Address

			// then all data persisted
			groupPolicyRes, err := s.groupKeeper.GroupPolicyInfo(s.ctx, &group.QueryGroupPolicyInfoRequest{Address: addr})
			s.Require().NoError(err)

			groupPolicy := groupPolicyRes.Info
			s.Assert().Equal(addr, groupPolicy.Address)
			s.Assert().Equal(myGroupID, groupPolicy.GroupId)
			s.Assert().Equal(spec.req.Admin, groupPolicy.Admin)
			s.Assert().Equal(spec.req.Metadata, groupPolicy.Metadata)
			s.Assert().Equal(uint64(1), groupPolicy.Version)
			percentageDecisionPolicy, ok := spec.policy.(*group.PercentageDecisionPolicy)
			if ok {
				dp, err := groupPolicy.GetDecisionPolicy()
				s.Assert().NoError(err)
				s.Assert().Equal(percentageDecisionPolicy, dp)
			} else {
				dp, err := groupPolicy.GetDecisionPolicy()
				s.Assert().NoError(err)
				s.Assert().Equal(spec.policy.(*group.ThresholdDecisionPolicy), dp)
			}
		})
	}
}

func (s *TestSuite) TestUpdateGroupPolicyAdmin() {
	addrs := s.addrs
	addr1 := addrs[0]
	addr2 := addrs[1]

	admin := addr1
	adminAddr, err := s.accountKeeper.AddressCodec().BytesToString(admin)
	s.Require().NoError(err)
	newAdmin, err := s.accountKeeper.AddressCodec().BytesToString(addr2)
	s.Require().NoError(err)

	policy := group.NewThresholdDecisionPolicy(
		"1",
		time.Second,
		0,
	)
	s.setNextAccount()
	groupPolicyAddr, myGroupID := s.createGroupAndGroupPolicy(admin, nil, policy)

	specs := map[string]struct {
		req            *group.MsgUpdateGroupPolicyAdmin
		expGroupPolicy *group.GroupPolicyInfo
		expErr         bool
		expErrMsg      string
	}{
		"with wrong admin": {
			req: &group.MsgUpdateGroupPolicyAdmin{
				Admin:              s.addrsStr[4],
				GroupPolicyAddress: groupPolicyAddr,
				NewAdmin:           newAdmin,
			},
			expGroupPolicy: &group.GroupPolicyInfo{
				Admin:          adminAddr,
				Address:        groupPolicyAddr,
				GroupId:        myGroupID,
				Version:        2,
				DecisionPolicy: nil,
				CreatedAt:      s.blockTime,
			},
			expErr:    true,
			expErrMsg: "not group policy admin: unauthorized",
		},
		"with wrong group policy": {
			req: &group.MsgUpdateGroupPolicyAdmin{
				Admin:              adminAddr,
				GroupPolicyAddress: s.addrsStr[4],
				NewAdmin:           newAdmin,
			},
			expGroupPolicy: &group.GroupPolicyInfo{
				Admin:          adminAddr,
				Address:        groupPolicyAddr,
				GroupId:        myGroupID,
				Version:        2,
				DecisionPolicy: nil,
				CreatedAt:      s.blockTime,
			},
			expErr:    true,
			expErrMsg: "load group policy: not found",
		},
		"correct data": {
			req: &group.MsgUpdateGroupPolicyAdmin{
				Admin:              adminAddr,
				GroupPolicyAddress: groupPolicyAddr,
				NewAdmin:           newAdmin,
			},
			expGroupPolicy: &group.GroupPolicyInfo{
				Admin:          newAdmin,
				Address:        groupPolicyAddr,
				GroupId:        myGroupID,
				Version:        2,
				DecisionPolicy: nil,
				CreatedAt:      s.blockTime,
			},
			expErr: false,
		},
		"with invalid new admin address": {
			req: &group.MsgUpdateGroupPolicyAdmin{
				Admin:              adminAddr,
				GroupPolicyAddress: groupPolicyAddr,
				NewAdmin:           "%s",
			},
			expGroupPolicy: &group.GroupPolicyInfo{
				Admin:          adminAddr,
				Address:        groupPolicyAddr,
				GroupId:        myGroupID,
				Version:        2,
				DecisionPolicy: nil,
				CreatedAt:      s.blockTime,
			},
			expErr:    true,
			expErrMsg: "new admin address",
		},
	}
	for msg, spec := range specs {

		err := spec.expGroupPolicy.SetDecisionPolicy(policy)
		s.Require().NoError(err)

		s.Run(msg, func() {
			_, err := s.groupKeeper.UpdateGroupPolicyAdmin(s.ctx, spec.req)
			if spec.expErr {
				s.Require().Error(err)
				s.Require().Contains(err.Error(), spec.expErrMsg)
				return
			}
			s.Require().NoError(err)
			res, err := s.groupKeeper.GroupPolicyInfo(s.ctx, &group.QueryGroupPolicyInfoRequest{
				Address: groupPolicyAddr,
			})
			s.Require().NoError(err)
			s.Assert().Equal(spec.expGroupPolicy, res.Info)
		})
	}
}

func (s *TestSuite) TestUpdateGroupPolicyDecisionPolicy() {
	addrs := s.addrs

	admin := addrs[0]
	adminAddr, err := s.accountKeeper.AddressCodec().BytesToString(admin)
	s.Require().NoError(err)

	policy := group.NewThresholdDecisionPolicy(
		"1",
		time.Second,
		0,
	)

	s.setNextAccount()
	groupPolicyAddr, myGroupID := s.createGroupAndGroupPolicy(admin, nil, policy)

	specs := map[string]struct {
		preRun         func(admin sdk.AccAddress) (policyAddr string, groupId uint64)
		req            *group.MsgUpdateGroupPolicyDecisionPolicy
		policy         group.DecisionPolicy
		expGroupPolicy *group.GroupPolicyInfo
		expErr         bool
		expErrMsg      string
	}{
		"with wrong admin": {
			req: &group.MsgUpdateGroupPolicyDecisionPolicy{
				Admin:              s.addrsStr[4],
				GroupPolicyAddress: groupPolicyAddr,
			},
			policy:         policy,
			expGroupPolicy: &group.GroupPolicyInfo{},
			expErr:         true,
			expErrMsg:      "not group policy admin: unauthorized",
		},
		"with wrong group policy": {
			req: &group.MsgUpdateGroupPolicyDecisionPolicy{
				Admin:              adminAddr,
				GroupPolicyAddress: s.addrsStr[4],
			},
			policy:         policy,
			expGroupPolicy: &group.GroupPolicyInfo{},
			expErr:         true,
			expErrMsg:      "load group policy: not found",
		},
		"invalid percentage decision policy with negative value": {
			req: &group.MsgUpdateGroupPolicyDecisionPolicy{
				Admin:              adminAddr,
				GroupPolicyAddress: groupPolicyAddr,
			},
			policy: group.NewPercentageDecisionPolicy(
				"-0.5",
				time.Duration(1)*time.Second,
				0,
			),
			expGroupPolicy: &group.GroupPolicyInfo{
				Admin:          adminAddr,
				Address:        groupPolicyAddr,
				GroupId:        myGroupID,
				Version:        2,
				DecisionPolicy: nil,
				CreatedAt:      s.blockTime,
			},
			expErr:    true,
			expErrMsg: "expected a positive decimal",
		},
		"invalid percentage decision policy with value greater than 1": {
			req: &group.MsgUpdateGroupPolicyDecisionPolicy{
				Admin:              adminAddr,
				GroupPolicyAddress: groupPolicyAddr,
			},
			policy: group.NewPercentageDecisionPolicy(
				"2",
				time.Duration(1)*time.Second,
				0,
			),
			expGroupPolicy: &group.GroupPolicyInfo{
				Admin:          adminAddr,
				Address:        groupPolicyAddr,
				GroupId:        myGroupID,
				Version:        2,
				DecisionPolicy: nil,
				CreatedAt:      s.blockTime,
			},
			expErr:    true,
			expErrMsg: "percentage must be > 0 and <= 1",
		},
		"correct data": {
			req: &group.MsgUpdateGroupPolicyDecisionPolicy{
				Admin:              adminAddr,
				GroupPolicyAddress: groupPolicyAddr,
			},
			policy: group.NewThresholdDecisionPolicy(
				"2",
				time.Duration(2)*time.Second,
				0,
			),
			expGroupPolicy: &group.GroupPolicyInfo{
				Admin:          adminAddr,
				Address:        groupPolicyAddr,
				GroupId:        myGroupID,
				Version:        2,
				DecisionPolicy: nil,
				CreatedAt:      s.blockTime,
			},
			expErr: false,
		},
		"correct data with percentage decision policy": {
			preRun: func(admin sdk.AccAddress) (string, uint64) {
				s.setNextAccount()
				return s.createGroupAndGroupPolicy(admin, nil, policy)
			},
			req: &group.MsgUpdateGroupPolicyDecisionPolicy{
				Admin:              adminAddr,
				GroupPolicyAddress: groupPolicyAddr,
			},
			policy: group.NewPercentageDecisionPolicy(
				"0.5",
				time.Duration(2)*time.Second,
				0,
			),
			expGroupPolicy: &group.GroupPolicyInfo{
				Admin:          adminAddr,
				DecisionPolicy: nil,
				Version:        2,
				CreatedAt:      s.blockTime,
			},
			expErr: false,
		},
	}
	for msg, spec := range specs {

		policyAddr := groupPolicyAddr
		err := spec.expGroupPolicy.SetDecisionPolicy(spec.policy)
		s.Require().NoError(err)
		if spec.preRun != nil {
			policyAddr1, groupID := spec.preRun(admin)
			policyAddr = policyAddr1

			// update the expected info with new group policy details
			spec.expGroupPolicy.Address = policyAddr1
			spec.expGroupPolicy.GroupId = groupID

			// update req with new group policy addr
			spec.req.GroupPolicyAddress = policyAddr1
		}

		err = spec.req.SetDecisionPolicy(spec.policy)
		s.Require().NoError(err)

		s.Run(msg, func() {
			_, err := s.groupKeeper.UpdateGroupPolicyDecisionPolicy(s.ctx, spec.req)
			if spec.expErr {
				s.Require().Error(err)
				s.Require().Contains(err.Error(), spec.expErrMsg)
				return
			}
			s.Require().NoError(err)
			res, err := s.groupKeeper.GroupPolicyInfo(s.ctx, &group.QueryGroupPolicyInfoRequest{
				Address: policyAddr,
			})
			s.Require().NoError(err)
			s.Assert().Equal(spec.expGroupPolicy, res.Info)
		})
	}
}

func (s *TestSuite) TestUpdateGroupPolicyMetadata() {
	admin := s.addrs[0]
	adminAddr, err := s.accountKeeper.AddressCodec().BytesToString(admin)
	s.Require().NoError(err)

	policy := group.NewThresholdDecisionPolicy(
		"1",
		time.Second,
		0,
	)

	s.setNextAccount()
	groupPolicyAddr, myGroupID := s.createGroupAndGroupPolicy(admin, nil, policy)

	specs := map[string]struct {
		req            *group.MsgUpdateGroupPolicyMetadata
		expGroupPolicy *group.GroupPolicyInfo
		expErr         bool
		expErrMsg      string
	}{
		"with wrong admin": {
			req: &group.MsgUpdateGroupPolicyMetadata{
				Admin:              s.addrsStr[4],
				GroupPolicyAddress: groupPolicyAddr,
			},
			expGroupPolicy: &group.GroupPolicyInfo{},
			expErr:         true,
			expErrMsg:      "not group policy admin: unauthorized",
		},
		"with wrong group policy": {
			req: &group.MsgUpdateGroupPolicyMetadata{
				Admin:              adminAddr,
				GroupPolicyAddress: s.addrsStr[4],
			},
			expGroupPolicy: &group.GroupPolicyInfo{},
			expErr:         true,
			expErrMsg:      "load group policy: not found",
		},
		"with metadata too long": {
			req: &group.MsgUpdateGroupPolicyMetadata{
				Admin:              adminAddr,
				GroupPolicyAddress: groupPolicyAddr,
				Metadata:           strings.Repeat("a", 1001),
			},
			expGroupPolicy: &group.GroupPolicyInfo{},
			expErr:         true,
			expErrMsg:      "group policy metadata: metadata too long",
		},
		"correct data": {
			req: &group.MsgUpdateGroupPolicyMetadata{
				Admin:              adminAddr,
				GroupPolicyAddress: groupPolicyAddr,
			},
			expGroupPolicy: &group.GroupPolicyInfo{
				Admin:          adminAddr,
				Address:        groupPolicyAddr,
				GroupId:        myGroupID,
				Version:        2,
				DecisionPolicy: nil,
				CreatedAt:      s.blockTime,
			},
			expErr: false,
		},
	}
	for msg, spec := range specs {

		err := spec.expGroupPolicy.SetDecisionPolicy(policy)
		s.Require().NoError(err)

		s.Run(msg, func() {
			_, err := s.groupKeeper.UpdateGroupPolicyMetadata(s.ctx, spec.req)
			if spec.expErr {
				s.Require().Error(err)
				s.Require().Contains(err.Error(), spec.expErrMsg)
				return
			}
			s.Require().NoError(err)

			res, err := s.groupKeeper.GroupPolicyInfo(s.ctx, &group.QueryGroupPolicyInfoRequest{
				Address: groupPolicyAddr,
			})
			s.Require().NoError(err)
			s.Assert().Equal(spec.expGroupPolicy, res.Info)

			// check events
			var hasUpdateGroupPolicyEvent bool
			events := s.ctx.(sdk.Context).EventManager().ABCIEvents()
			for _, event := range events {
				event, err := sdk.ParseTypedEvent(event)
				s.Require().NoError(err)

				if e, ok := event.(*group.EventUpdateGroupPolicy); ok {
					s.Require().Equal(e.Address, groupPolicyAddr)
					hasUpdateGroupPolicyEvent = true
					break
				}
			}

			s.Require().True(hasUpdateGroupPolicyEvent)
		})
	}
}

func (s *TestSuite) TestGroupPoliciesByAdminOrGroup() {
	addrs := s.addrs

	admin, err := s.accountKeeper.AddressCodec().BytesToString(addrs[1])
	s.Require().NoError(err)

	groupRes, err := s.groupKeeper.CreateGroup(s.ctx, &group.MsgCreateGroup{
		Admin:   admin,
		Members: nil,
	})
	s.Require().NoError(err)
	myGroupID := groupRes.GroupId

	policies := []group.DecisionPolicy{
		group.NewThresholdDecisionPolicy(
			"1",
			time.Second,
			0,
		),
		group.NewThresholdDecisionPolicy(
			"10",
			time.Second,
			0,
		),
		group.NewPercentageDecisionPolicy(
			"0.5",
			time.Second,
			0,
		),
	}

	count := 3
	expectAccs := make([]*group.GroupPolicyInfo, count)
	for i := range expectAccs {
		req := &group.MsgCreateGroupPolicy{
			Admin:   admin,
			GroupId: myGroupID,
		}
		err := req.SetDecisionPolicy(policies[i])
		s.Require().NoError(err)

		s.setNextAccount()
		res, err := s.groupKeeper.CreateGroupPolicy(s.ctx, req)
		s.Require().NoError(err)

		expectAcc := &group.GroupPolicyInfo{
			Address:   res.Address,
			Admin:     admin,
			GroupId:   myGroupID,
			Version:   uint64(1),
			CreatedAt: s.blockTime,
		}
		err = expectAcc.SetDecisionPolicy(policies[i])
		s.Require().NoError(err)
		expectAccs[i] = expectAcc
	}
	sort.Slice(expectAccs, func(i, j int) bool { return expectAccs[i].Address < expectAccs[j].Address })

	// query group policy by group
	policiesByGroupRes, err := s.groupKeeper.GroupPoliciesByGroup(s.ctx, &group.QueryGroupPoliciesByGroupRequest{
		GroupId: myGroupID,
	})
	s.Require().NoError(err)
	policyAccs := policiesByGroupRes.GroupPolicies
	s.Require().Equal(len(policyAccs), count)
	// we reorder policyAccs by address to be able to compare them
	sort.Slice(policyAccs, func(i, j int) bool { return policyAccs[i].Address < policyAccs[j].Address })
	for i := range policyAccs {
		s.Assert().Equal(policyAccs[i].Address, expectAccs[i].Address)
		s.Assert().Equal(policyAccs[i].GroupId, expectAccs[i].GroupId)
		s.Assert().Equal(policyAccs[i].Admin, expectAccs[i].Admin)
		s.Assert().Equal(policyAccs[i].Metadata, expectAccs[i].Metadata)
		s.Assert().Equal(policyAccs[i].Version, expectAccs[i].Version)
		s.Assert().Equal(policyAccs[i].CreatedAt, expectAccs[i].CreatedAt)
		dp1, err := policyAccs[i].GetDecisionPolicy()
		s.Assert().NoError(err)
		dp2, err := expectAccs[i].GetDecisionPolicy()
		s.Assert().NoError(err)
		s.Assert().Equal(dp1, dp2)
	}

	// no group policy
	noPolicies, err := s.groupKeeper.GroupPoliciesByAdmin(s.ctx, &group.QueryGroupPoliciesByAdminRequest{
		Admin: s.addrsStr[2],
	})
	s.Require().NoError(err)
	policyAccs = noPolicies.GroupPolicies
	s.Require().Equal(len(policyAccs), 0)

	// query group policy by admin
	policiesByAdminRes, err := s.groupKeeper.GroupPoliciesByAdmin(s.ctx, &group.QueryGroupPoliciesByAdminRequest{
		Admin: admin,
	})
	s.Require().NoError(err)
	policyAccs = policiesByAdminRes.GroupPolicies
	s.Require().Equal(len(policyAccs), count)
	// we reorder policyAccs by address to be able to compare them
	sort.Slice(policyAccs, func(i, j int) bool { return policyAccs[i].Address < policyAccs[j].Address })
	for i := range policyAccs {
		s.Assert().Equal(policyAccs[i].Address, expectAccs[i].Address)
		s.Assert().Equal(policyAccs[i].GroupId, expectAccs[i].GroupId)
		s.Assert().Equal(policyAccs[i].Admin, expectAccs[i].Admin)
		s.Assert().Equal(policyAccs[i].Metadata, expectAccs[i].Metadata)
		s.Assert().Equal(policyAccs[i].Version, expectAccs[i].Version)
		s.Assert().Equal(policyAccs[i].CreatedAt, expectAccs[i].CreatedAt)
		dp1, err := policyAccs[i].GetDecisionPolicy()
		s.Assert().NoError(err)
		dp2, err := expectAccs[i].GetDecisionPolicy()
		s.Assert().NoError(err)
		s.Assert().Equal(dp1, dp2)
	}
}

func (s *TestSuite) TestSubmitProposal() {
	addrs := s.addrs
	addr2 := addrs[1] // Has weight 2

	myGroupID := s.groupID
	accountAddr, err := s.accountKeeper.AddressCodec().BytesToString(s.groupPolicyAddr)
	s.Require().NoError(err)

	// Create a new group policy to test TRY_EXEC
	policyReq := &group.MsgCreateGroupPolicy{
		Admin:   s.addrsStr[0],
		GroupId: myGroupID,
	}
	noMinExecPeriodPolicy := group.NewThresholdDecisionPolicy(
		"2",
		time.Second,
		0, // no MinExecutionPeriod to test TRY_EXEC
	)
	err = policyReq.SetDecisionPolicy(noMinExecPeriodPolicy)
	s.Require().NoError(err)
	s.setNextAccount()
	res, err := s.groupKeeper.CreateGroupPolicy(s.ctx, policyReq)
	s.Require().NoError(err)

	noMinExecPeriodPolicyAddr, err := s.accountKeeper.AddressCodec().StringToBytes(res.Address)
	s.Require().NoError(err)

	// Create a new group policy with super high threshold
	bigThresholdPolicy := group.NewThresholdDecisionPolicy(
		"100",
		time.Second,
		minExecutionPeriod,
	)
	s.setNextAccount()
	err = policyReq.SetDecisionPolicy(bigThresholdPolicy)
	s.Require().NoError(err)
	bigThresholdRes, err := s.groupKeeper.CreateGroupPolicy(s.ctx, policyReq)
	s.Require().NoError(err)
	bigThresholdAddr := bigThresholdRes.Address

	msgSend := &banktypes.MsgSend{
		FromAddress: res.Address,
		ToAddress:   s.addrsStr[1],
		Amount:      sdk.Coins{sdk.NewInt64Coin("test", 100)},
	}
	defaultProposal := group.Proposal{
		GroupPolicyAddress: accountAddr,
		Status:             group.PROPOSAL_STATUS_SUBMITTED,
		FinalTallyResult: group.TallyResult{
			YesCount:        "0",
			NoCount:         "0",
			AbstainCount:    "0",
			NoWithVetoCount: "0",
		},
		ExecutorResult: group.PROPOSAL_EXECUTOR_RESULT_NOT_RUN,
	}
	specs := map[string]struct {
		req         *group.MsgSubmitProposal
		msgs        []sdk.Msg
		expProposal group.Proposal
		expErr      bool
		expErrMsg   string
		postRun     func(sdkCtx sdk.Context)
		preRun      func(msg []sdk.Msg)
	}{
		"all good with minimal fields set": {
			req: &group.MsgSubmitProposal{
				GroupPolicyAddress: accountAddr,
				Proposers:          []string{s.addrsStr[1]},
			},
			expProposal: defaultProposal,
			postRun:     func(sdkCtx sdk.Context) {},
		},
		"all good with good msg payload": {
			req: &group.MsgSubmitProposal{
				GroupPolicyAddress: accountAddr,
				Proposers:          []string{s.addrsStr[1]},
			},
			msgs: []sdk.Msg{&banktypes.MsgSend{
				FromAddress: accountAddr,
				ToAddress:   s.addrsStr[1],
				Amount:      sdk.Coins{sdk.NewInt64Coin("token", 100)},
			}},
			expProposal: defaultProposal,
			postRun:     func(sdkCtx sdk.Context) {},
		},
		"title != metadata.title": {
			req: &group.MsgSubmitProposal{
				GroupPolicyAddress: accountAddr,
				Proposers:          []string{s.addrsStr[1]},
				Metadata:           "{\"title\":\"title\",\"summary\":\"description\"}",
				Title:              "title2",
				Summary:            "description",
			},
			expErr:    true,
			expErrMsg: "metadata title 'title' must equal proposal title 'title2'",
			postRun:   func(sdkCtx sdk.Context) {},
		},
		"summary != metadata.summary": {
			req: &group.MsgSubmitProposal{
				GroupPolicyAddress: accountAddr,
				Proposers:          []string{s.addrsStr[1]},
				Metadata:           "{\"title\":\"title\",\"summary\":\"description of proposal\"}",
				Title:              "title",
				Summary:            "description",
			},
			expErr:    true,
			expErrMsg: "metadata summary 'description of proposal' must equal proposal summary 'description'",
			postRun:   func(sdkCtx sdk.Context) {},
		},
		"metadata too long": {
			req: &group.MsgSubmitProposal{
				GroupPolicyAddress: accountAddr,
				Proposers:          []string{s.addrsStr[1]},
				Metadata:           strings.Repeat("a", 256),
			},
			expErr:    true,
			expErrMsg: "metadata: metadata too long",
			postRun:   func(sdkCtx sdk.Context) {},
		},
		"summary too long": {
			req: &group.MsgSubmitProposal{
				GroupPolicyAddress: accountAddr,
				Proposers:          []string{s.addrsStr[1]},
				Metadata:           "{\"title\":\"title\",\"summary\":\"description\"}",
				Summary:            strings.Repeat("a", 256*40),
			},
			expErr:    true,
			expErrMsg: "summary too long",
			postRun:   func(sdkCtx sdk.Context) {},
		},
		"group policy required": {
			req: &group.MsgSubmitProposal{
				Proposers: []string{s.addrsStr[1]},
			},
			expErr:    true,
			expErrMsg: "empty address string is not allowed",
			postRun:   func(sdkCtx sdk.Context) {},
		},
		"existing group policy required": {
			req: &group.MsgSubmitProposal{
				GroupPolicyAddress: s.addrsStr[0],
				Proposers:          []string{s.addrsStr[1]},
			},
			expErr:    true,
			expErrMsg: "not found",
			postRun:   func(sdkCtx sdk.Context) {},
		},
		"decision policy threshold > total group weight": {
			req: &group.MsgSubmitProposal{
				GroupPolicyAddress: bigThresholdAddr,
				Proposers:          []string{s.addrsStr[1]},
			},
			expErr: false,
			expProposal: group.Proposal{
				GroupPolicyAddress: bigThresholdAddr,
				Status:             group.PROPOSAL_STATUS_SUBMITTED,
				FinalTallyResult:   group.DefaultTallyResult(),
				ExecutorResult:     group.PROPOSAL_EXECUTOR_RESULT_NOT_RUN,
			},
			postRun: func(sdkCtx sdk.Context) {},
		},
		"only group members can create a proposal": {
			req: &group.MsgSubmitProposal{
				GroupPolicyAddress: accountAddr,
				Proposers:          []string{s.addrsStr[3]},
			},
			expErr:    true,
			expErrMsg: "not in group",
			postRun:   func(sdkCtx sdk.Context) {},
		},
		"all proposers must be in group": {
			req: &group.MsgSubmitProposal{
				GroupPolicyAddress: accountAddr,
				Proposers:          []string{s.addrsStr[1], s.addrsStr[3]},
			},
			expErr:    true,
			expErrMsg: "not in group",
			postRun:   func(sdkCtx sdk.Context) {},
		},
		"admin that is not a group member can not create proposal": {
			req: &group.MsgSubmitProposal{
				GroupPolicyAddress: accountAddr,
				Proposers:          []string{s.addrsStr[0]},
			},
			expErr:    true,
			expErrMsg: "not in group",
			postRun:   func(sdkCtx sdk.Context) {},
		},
		"reject msgs that are not authz by group policy": {
			req: &group.MsgSubmitProposal{
				GroupPolicyAddress: accountAddr,
				Proposers:          []string{s.addrsStr[1]},
			},
			msgs:      []sdk.Msg{&testdata.TestMsg{Signers: []string{s.addrsStr[0]}}},
			expErr:    true,
			expErrMsg: "msg does not have group policy authorization",
			postRun:   func(sdkCtx sdk.Context) {},
		},
		"with try exec": {
			preRun: func(msgs []sdk.Msg) {
				for i := 0; i < len(msgs); i++ {
					s.bankKeeper.EXPECT().Send(gomock.Any(), msgs[i]).Return(nil, nil)
				}
			},
			req: &group.MsgSubmitProposal{
				GroupPolicyAddress: res.Address,
				Proposers:          []string{s.addrsStr[1]},
				Exec:               group.Exec_EXEC_TRY,
			},
			msgs: []sdk.Msg{msgSend},
			expProposal: group.Proposal{
				GroupPolicyAddress: res.Address,
				Status:             group.PROPOSAL_STATUS_ACCEPTED,
				FinalTallyResult: group.TallyResult{
					YesCount:        "2",
					NoCount:         "0",
					AbstainCount:    "0",
					NoWithVetoCount: "0",
				},
				ExecutorResult: group.PROPOSAL_EXECUTOR_RESULT_SUCCESS,
			},
			postRun: func(sdkCtx sdk.Context) {
				s.bankKeeper.EXPECT().GetAllBalances(sdkCtx, noMinExecPeriodPolicyAddr).Return(sdk.NewCoins(sdk.NewInt64Coin("test", 9900)))
				s.bankKeeper.EXPECT().GetAllBalances(sdkCtx, addr2).Return(sdk.NewCoins(sdk.NewInt64Coin("test", 100)))

				fromBalances := s.bankKeeper.GetAllBalances(sdkCtx, noMinExecPeriodPolicyAddr)
				s.Require().Contains(fromBalances, sdk.NewInt64Coin("test", 9900))
				toBalances := s.bankKeeper.GetAllBalances(sdkCtx, addr2)
				s.Require().Contains(toBalances, sdk.NewInt64Coin("test", 100))
				events := sdkCtx.EventManager().Events()
				s.Require().True(eventTypeFound(events, EventProposalPruned))
			},
		},
		"with try exec, not enough yes votes for proposal to pass": {
			req: &group.MsgSubmitProposal{
				GroupPolicyAddress: res.Address,
				Proposers:          []string{s.addrsStr[4]},
				Exec:               group.Exec_EXEC_TRY,
			},
			msgs: []sdk.Msg{msgSend},
			expProposal: group.Proposal{
				GroupPolicyAddress: res.Address,
				Status:             group.PROPOSAL_STATUS_SUBMITTED,
				FinalTallyResult: group.TallyResult{
					YesCount:        "0", // Since tally doesn't pass Allow(), we consider the proposal not final
					NoCount:         "0",
					AbstainCount:    "0",
					NoWithVetoCount: "0",
				},
				ExecutorResult: group.PROPOSAL_EXECUTOR_RESULT_NOT_RUN,
			},
			postRun: func(sdkCtx sdk.Context) {},
		},
	}
	for msg, spec := range specs {
		s.Run(msg, func() {
			err := spec.req.SetMsgs(spec.msgs)
			s.Require().NoError(err)

			if spec.preRun != nil {
				spec.preRun(spec.msgs)
			}

			res, err := s.groupKeeper.SubmitProposal(s.ctx, spec.req)
			if spec.expErr {
				s.Require().Error(err)
				s.Require().Contains(err.Error(), spec.expErrMsg)
				return
			}
			s.Require().NoError(err)
			id := res.ProposalId

			if !(spec.expProposal.ExecutorResult == group.PROPOSAL_EXECUTOR_RESULT_SUCCESS) {
				// then all data persisted
				proposalRes, err := s.groupKeeper.Proposal(s.ctx, &group.QueryProposalRequest{ProposalId: id})
				s.Require().NoError(err)
				proposal := proposalRes.Proposal

				s.Assert().Equal(spec.expProposal.GroupPolicyAddress, proposal.GroupPolicyAddress)
				s.Assert().Equal(spec.req.Metadata, proposal.Metadata)
				s.Assert().Equal(spec.req.Proposers, proposal.Proposers)
				s.Assert().Equal(s.blockTime, proposal.SubmitTime)
				s.Assert().Equal(uint64(1), proposal.GroupVersion)
				s.Assert().Equal(uint64(1), proposal.GroupPolicyVersion)
				s.Assert().Equal(spec.expProposal.Status, proposal.Status)
				s.Assert().Equal(spec.expProposal.FinalTallyResult, proposal.FinalTallyResult)
				s.Assert().Equal(spec.expProposal.ExecutorResult, proposal.ExecutorResult)
				s.Assert().Equal(s.blockTime.Add(time.Second), proposal.VotingPeriodEnd)

				msgs, err := proposal.GetMsgs()
				s.Assert().NoError(err)
				if spec.msgs == nil { // then empty list is ok
					s.Assert().Len(msgs, 0)
				} else {
					s.Assert().Equal(spec.msgs, msgs)
				}
			}

			spec.postRun(s.sdkCtx)
		})
	}
}

func (s *TestSuite) TestWithdrawProposal() {
	msgSend := &banktypes.MsgSend{
		FromAddress: s.groupPolicyStrAddr,
		ToAddress:   s.addrsStr[1],
		Amount:      sdk.Coins{sdk.NewInt64Coin("test", 100)},
	}

	proposers := []string{s.addrsStr[1]}
	proposalID := submitProposal(s.ctx, s, []sdk.Msg{msgSend}, proposers)

	specs := map[string]struct {
		preRun     func(sdkCtx sdk.Context) uint64
		proposalID uint64
		admin      string
		expErrMsg  string
		postRun    func(sdkCtx sdk.Context)
	}{
		"wrong admin": {
			preRun: func(sdkCtx sdk.Context) uint64 {
				return submitProposal(s.ctx, s, []sdk.Msg{msgSend}, proposers)
			},
			admin:     s.addrsStr[4],
			expErrMsg: "unauthorized",
			postRun:   func(sdkCtx sdk.Context) {},
		},
		"wrong proposal id": {
			preRun: func(sdkCtx sdk.Context) uint64 {
				return 1111
			},
			admin:     proposers[0],
			expErrMsg: "not found",
			postRun:   func(sdkCtx sdk.Context) {},
		},
		"happy case with proposer": {
			preRun: func(sdkCtx sdk.Context) uint64 {
				return submitProposal(s.ctx, s, []sdk.Msg{msgSend}, proposers)
			},
			proposalID: proposalID,
			admin:      proposers[0],
			postRun:    func(sdkCtx sdk.Context) {},
		},
		"already closed proposal": {
			preRun: func(sdkCtx sdk.Context) uint64 {
				pID := submitProposal(s.ctx, s, []sdk.Msg{msgSend}, proposers)
				_, err := s.groupKeeper.WithdrawProposal(s.ctx, &group.MsgWithdrawProposal{
					ProposalId: pID,
					Address:    proposers[0],
				})
				s.Require().NoError(err)
				return pID
			},
			proposalID: proposalID,
			admin:      proposers[0],
			expErrMsg:  "cannot withdraw a proposal with the status of PROPOSAL_STATUS_WITHDRAWN",
			postRun:    func(sdkCtx sdk.Context) {},
		},
		"happy case with group admin address": {
			preRun: func(sdkCtx sdk.Context) uint64 {
				return submitProposal(s.ctx, s, []sdk.Msg{msgSend}, proposers)
			},
			proposalID: proposalID,
			admin:      proposers[0],
			postRun: func(sdkCtx sdk.Context) {
				resp, err := s.groupKeeper.Proposal(s.ctx, &group.QueryProposalRequest{ProposalId: proposalID})
				s.Require().NoError(err)
				vpe := resp.Proposal.VotingPeriodEnd
				timeDiff := vpe.Sub(s.sdkCtx.HeaderInfo().Time)
				ctxVPE := sdkCtx.WithHeaderInfo(header.Info{Time: s.sdkCtx.HeaderInfo().Time.Add(timeDiff).Add(time.Second * 1)})
				s.Require().NoError(s.groupKeeper.TallyProposalsAtVPEnd(ctxVPE))
				events := ctxVPE.EventManager().Events()

				s.Require().True(eventTypeFound(events, EventProposalPruned))
			},
		},
	}
	for msg, spec := range specs {

		s.Run(msg, func() {
			pID := spec.preRun(s.sdkCtx)

			_, err := s.groupKeeper.WithdrawProposal(s.ctx, &group.MsgWithdrawProposal{
				ProposalId: pID,
				Address:    spec.admin,
			})

			if spec.expErrMsg != "" {
				s.Require().Error(err)
				s.Require().Contains(err.Error(), spec.expErrMsg)
				return
			}

			s.Require().NoError(err)
			resp, err := s.groupKeeper.Proposal(s.ctx, &group.QueryProposalRequest{ProposalId: pID})
			s.Require().NoError(err)
			s.Require().Equal(resp.GetProposal().Status, group.PROPOSAL_STATUS_WITHDRAWN)
		})
		spec.postRun(s.sdkCtx)
	}
}

func (s *TestSuite) TestVote() {
	addr5 := s.addrs[4]
	members := []group.MemberRequest{
		{Address: s.addrsStr[3], Weight: "1"},
		{Address: s.addrsStr[2], Weight: "2"},
	}

	groupRes, err := s.groupKeeper.CreateGroup(s.ctx, &group.MsgCreateGroup{
		Admin:   s.addrsStr[0],
		Members: members,
	})
	s.Require().NoError(err)
	myGroupID := groupRes.GroupId

	policy := group.NewThresholdDecisionPolicy(
		"2",
		time.Duration(2),
		0,
	)
	policyReq := &group.MsgCreateGroupPolicy{
		Admin:   s.addrsStr[0],
		GroupId: myGroupID,
	}
	err = policyReq.SetDecisionPolicy(policy)
	s.Require().NoError(err)

	s.setNextAccount()
	policyRes, err := s.groupKeeper.CreateGroupPolicy(s.ctx, policyReq)
	s.Require().NoError(err)
	accountAddr := policyRes.Address
	// module account will be created and returned
	groupPolicy, err := s.accountKeeper.AddressCodec().StringToBytes(accountAddr)
	s.Require().NoError(err)
	s.Require().NotNil(groupPolicy)

	s.bankKeeper.EXPECT().SendCoinsFromModuleToAccount(s.sdkCtx, minttypes.ModuleName, groupPolicy, sdk.Coins{sdk.NewInt64Coin("test", 10000)}).Return(nil).AnyTimes()
	s.Require().NoError(s.bankKeeper.SendCoinsFromModuleToAccount(s.sdkCtx, minttypes.ModuleName, groupPolicy, sdk.Coins{sdk.NewInt64Coin("test", 10000)}))

	req := &group.MsgSubmitProposal{
		GroupPolicyAddress: accountAddr,
		Proposers:          []string{s.addrsStr[3]},
		Messages:           nil,
	}
	msg := &banktypes.MsgSend{
		FromAddress: accountAddr,
		ToAddress:   s.addrsStr[4],
		Amount:      sdk.Coins{sdk.NewInt64Coin("test", 100)},
	}
	err = req.SetMsgs([]sdk.Msg{msg})
	s.Require().NoError(err)

	proposalRes, err := s.groupKeeper.SubmitProposal(s.ctx, req)
	s.Require().NoError(err)
	myProposalID := proposalRes.ProposalId

	// no group policy
	proposalsRes, err := s.groupKeeper.ProposalsByGroupPolicy(s.ctx, &group.QueryProposalsByGroupPolicyRequest{
		Address: s.addrsStr[2],
	})
	s.Require().NoError(err)
	proposals := proposalsRes.Proposals
	s.Require().Equal(len(proposals), 0)

	// proposals by group policy (request with pagination)
	proposalsRes, err = s.groupKeeper.ProposalsByGroupPolicy(s.ctx, &group.QueryProposalsByGroupPolicyRequest{
		Address: accountAddr,
		Pagination: &query.PageRequest{
			Limit: 2,
		},
	})
	s.Require().NoError(err)
	proposals = proposalsRes.Proposals
	s.Require().Equal(len(proposals), 1)

	// proposals by group policy
	proposalsRes, err = s.groupKeeper.ProposalsByGroupPolicy(s.ctx, &group.QueryProposalsByGroupPolicyRequest{
		Address: accountAddr,
	})
	s.Require().NoError(err)
	proposals = proposalsRes.Proposals
	s.Require().Equal(len(proposals), 1)
	s.Assert().Equal(req.GroupPolicyAddress, proposals[0].GroupPolicyAddress)
	s.Assert().Equal(req.Metadata, proposals[0].Metadata)
	s.Assert().Equal(req.Proposers, proposals[0].Proposers)
	s.Assert().Equal(s.blockTime, proposals[0].SubmitTime)
	s.Assert().Equal(uint64(1), proposals[0].GroupVersion)
	s.Assert().Equal(uint64(1), proposals[0].GroupPolicyVersion)
	s.Assert().Equal(group.PROPOSAL_STATUS_SUBMITTED, proposals[0].Status)
	s.Assert().Equal(group.DefaultTallyResult(), proposals[0].FinalTallyResult)

	specs := map[string]struct {
		srcCtx            sdk.Context
		expTallyResult    group.TallyResult // expected after tallying
		isFinal           bool              // is the tally result final?
		req               *group.MsgVote
		doBefore          func(ctx context.Context)
		postRun           func(sdkCtx sdk.Context)
		expProposalStatus group.ProposalStatus         // expected after tallying
		expExecutorResult group.ProposalExecutorResult // expected after tallying
		expErr            bool
		expErrMsg         string
	}{
		"vote yes": {
			req: &group.MsgVote{
				ProposalId: myProposalID,
				Voter:      s.addrsStr[3],
				Option:     group.VOTE_OPTION_YES,
			},
			expTallyResult: group.TallyResult{
				YesCount:        "1",
				NoCount:         "0",
				AbstainCount:    "0",
				NoWithVetoCount: "0",
			},
			expProposalStatus: group.PROPOSAL_STATUS_SUBMITTED,
			expExecutorResult: group.PROPOSAL_EXECUTOR_RESULT_NOT_RUN,
			postRun:           func(sdkCtx sdk.Context) {},
		},
		"with try exec": {
			req: &group.MsgVote{
				ProposalId: myProposalID,
				Voter:      s.addrsStr[2],
				Option:     group.VOTE_OPTION_YES,
				Exec:       group.Exec_EXEC_TRY,
			},
			expTallyResult: group.TallyResult{
				YesCount:        "2",
				NoCount:         "0",
				AbstainCount:    "0",
				NoWithVetoCount: "0",
			},
			isFinal:           true,
			expProposalStatus: group.PROPOSAL_STATUS_ACCEPTED,
			expExecutorResult: group.PROPOSAL_EXECUTOR_RESULT_SUCCESS,
			doBefore: func(ctx context.Context) {
				s.bankKeeper.EXPECT().Send(gomock.Any(), msg).Return(nil, nil)
			},
			postRun: func(sdkCtx sdk.Context) {
				s.bankKeeper.EXPECT().GetAllBalances(gomock.Any(), groupPolicy).Return(sdk.NewCoins(sdk.NewInt64Coin("test", 9900)))
				s.bankKeeper.EXPECT().GetAllBalances(gomock.Any(), addr5).Return(sdk.NewCoins(sdk.NewInt64Coin("test", 100)))

				fromBalances := s.bankKeeper.GetAllBalances(sdkCtx, groupPolicy)
				s.Require().Contains(fromBalances, sdk.NewInt64Coin("test", 9900))
				toBalances := s.bankKeeper.GetAllBalances(sdkCtx, addr5)
				s.Require().Contains(toBalances, sdk.NewInt64Coin("test", 100))
			},
		},
		"with try exec, not enough yes votes for proposal to pass": {
			req: &group.MsgVote{
				ProposalId: myProposalID,
				Voter:      s.addrsStr[3],
				Option:     group.VOTE_OPTION_YES,
				Exec:       group.Exec_EXEC_TRY,
			},
			expTallyResult: group.TallyResult{
				YesCount:        "1",
				NoCount:         "0",
				AbstainCount:    "0",
				NoWithVetoCount: "0",
			},
			expProposalStatus: group.PROPOSAL_STATUS_SUBMITTED,
			expExecutorResult: group.PROPOSAL_EXECUTOR_RESULT_NOT_RUN,
			postRun:           func(sdkCtx sdk.Context) {},
		},
		"vote no": {
			req: &group.MsgVote{
				ProposalId: myProposalID,
				Voter:      s.addrsStr[3],
				Option:     group.VOTE_OPTION_NO,
			},
			expTallyResult: group.TallyResult{
				YesCount:        "0",
				NoCount:         "1",
				AbstainCount:    "0",
				NoWithVetoCount: "0",
			},
			expProposalStatus: group.PROPOSAL_STATUS_SUBMITTED,
			expExecutorResult: group.PROPOSAL_EXECUTOR_RESULT_NOT_RUN,
			postRun:           func(sdkCtx sdk.Context) {},
		},
		"vote abstain": {
			req: &group.MsgVote{
				ProposalId: myProposalID,
				Voter:      s.addrsStr[3],
				Option:     group.VOTE_OPTION_ABSTAIN,
			},
			expTallyResult: group.TallyResult{
				YesCount:        "0",
				NoCount:         "0",
				AbstainCount:    "1",
				NoWithVetoCount: "0",
			},
			expProposalStatus: group.PROPOSAL_STATUS_SUBMITTED,
			expExecutorResult: group.PROPOSAL_EXECUTOR_RESULT_NOT_RUN,
			postRun:           func(sdkCtx sdk.Context) {},
		},
		"vote veto": {
			req: &group.MsgVote{
				ProposalId: myProposalID,
				Voter:      s.addrsStr[3],
				Option:     group.VOTE_OPTION_NO_WITH_VETO,
			},
			expTallyResult: group.TallyResult{
				YesCount:        "0",
				NoCount:         "0",
				AbstainCount:    "0",
				NoWithVetoCount: "1",
			},
			expProposalStatus: group.PROPOSAL_STATUS_SUBMITTED,
			expExecutorResult: group.PROPOSAL_EXECUTOR_RESULT_NOT_RUN,
			postRun:           func(sdkCtx sdk.Context) {},
		},
		"apply decision policy early": {
			req: &group.MsgVote{
				ProposalId: myProposalID,
				Voter:      s.addrsStr[2],
				Option:     group.VOTE_OPTION_YES,
			},
			expTallyResult: group.TallyResult{
				YesCount:        "2",
				NoCount:         "0",
				AbstainCount:    "0",
				NoWithVetoCount: "0",
			},
			expProposalStatus: group.PROPOSAL_STATUS_ACCEPTED,
			expExecutorResult: group.PROPOSAL_EXECUTOR_RESULT_NOT_RUN,
			postRun:           func(sdkCtx sdk.Context) {},
		},
		"reject new votes when final decision is made already": {
			req: &group.MsgVote{
				ProposalId: myProposalID,
				Voter:      s.addrsStr[3],
				Option:     group.VOTE_OPTION_YES,
			},
			doBefore: func(ctx context.Context) {
				_, err := s.groupKeeper.Vote(ctx, &group.MsgVote{
					ProposalId: myProposalID,
					Voter:      s.addrsStr[2],
					Option:     group.VOTE_OPTION_NO_WITH_VETO,
					Exec:       1, // Execute the proposal so that its status is final
				})
				s.Require().NoError(err)
			},
			expErr:    true,
			expErrMsg: "proposal not open for voting",
			postRun:   func(sdkCtx sdk.Context) {},
		},
		"metadata too long": {
			req: &group.MsgVote{
				ProposalId: myProposalID,
				Voter:      s.addrsStr[3],
				Option:     group.VOTE_OPTION_NO,
				Metadata:   strings.Repeat("a", 256),
			},
			expErr:    true,
			expErrMsg: "metadata: metadata too long",
			postRun:   func(sdkCtx sdk.Context) {},
		},
		"existing proposal required": {
			req: &group.MsgVote{
				ProposalId: 999,
				Voter:      s.addrsStr[3],
				Option:     group.VOTE_OPTION_NO,
			},
			expErr:    true,
			expErrMsg: "load proposal: not found",
			postRun:   func(sdkCtx sdk.Context) {},
		},
		"empty vote option": {
			req: &group.MsgVote{
				ProposalId: myProposalID,
				Voter:      s.addrsStr[3],
			},
			expErr:    true,
			expErrMsg: "vote option: value is empty",
			postRun:   func(sdkCtx sdk.Context) {},
		},
		"invalid vote option": {
			req: &group.MsgVote{
				ProposalId: myProposalID,
				Voter:      s.addrsStr[3],
				Option:     5,
			},
			expErr:    true,
			expErrMsg: "ote option: invalid value",
			postRun:   func(sdkCtx sdk.Context) {},
		},
		"voter must be in group": {
			req: &group.MsgVote{
				ProposalId: myProposalID,
				Voter:      s.addrsStr[1],
				Option:     group.VOTE_OPTION_NO,
			},
			expErr:    true,
			expErrMsg: "not found",
			postRun:   func(sdkCtx sdk.Context) {},
		},
		"admin that is not a group member can not vote": {
			req: &group.MsgVote{
				ProposalId: myProposalID,
				Voter:      s.addrsStr[0],
				Option:     group.VOTE_OPTION_NO,
			},
			expErr:    true,
			expErrMsg: "not found",
			postRun:   func(sdkCtx sdk.Context) {},
		},
		"on voting period end": {
			req: &group.MsgVote{
				ProposalId: myProposalID,
				Voter:      s.addrsStr[3],
				Option:     group.VOTE_OPTION_NO,
			},
			srcCtx:    s.sdkCtx.WithHeaderInfo(header.Info{Time: s.sdkCtx.HeaderInfo().Time.Add(time.Second)}),
			expErr:    true,
			expErrMsg: "voting period has ended already: expired",
			postRun:   func(sdkCtx sdk.Context) {},
		},
		"vote closed already": {
			req: &group.MsgVote{
				ProposalId: myProposalID,
				Voter:      s.addrsStr[3],
				Option:     group.VOTE_OPTION_NO,
			},
			doBefore: func(ctx context.Context) {
				s.bankKeeper.EXPECT().Send(gomock.Any(), msg).Return(nil, nil)

				_, err := s.groupKeeper.Vote(ctx, &group.MsgVote{
					ProposalId: myProposalID,
					Voter:      s.addrsStr[2],
					Option:     group.VOTE_OPTION_YES,
					Exec:       1, // Execute to close the proposal.
				})
				s.Require().NoError(err)
			},
			expErr:    true,
			expErrMsg: "load proposal: not found",
			postRun:   func(sdkCtx sdk.Context) {},
		},
		"voted already": {
			req: &group.MsgVote{
				ProposalId: myProposalID,
				Voter:      s.addrsStr[3],
				Option:     group.VOTE_OPTION_NO,
			},
			doBefore: func(ctx context.Context) {
				_, err := s.groupKeeper.Vote(ctx, &group.MsgVote{
					ProposalId: myProposalID,
					Voter:      s.addrsStr[3],
					Option:     group.VOTE_OPTION_YES,
				})
				s.Require().NoError(err)
			},
			expErr:    true,
			expErrMsg: "store vote: unique constraint violation",
			postRun:   func(sdkCtx sdk.Context) {},
		},
	}
	for msg, spec := range specs {
		s.Run(msg, func() {
			sdkCtx := s.sdkCtx
			if !spec.srcCtx.IsZero() {
				sdkCtx = spec.srcCtx
			}
			sdkCtx, _ = sdkCtx.CacheContext()
			if spec.doBefore != nil {
				spec.doBefore(sdkCtx)
			}
			_, err := s.groupKeeper.Vote(sdkCtx, spec.req)
			if spec.expErr {
				s.Require().Error(err)
				s.Require().Contains(err.Error(), spec.expErrMsg)
				return
			}
			s.Require().NoError(err)

			if !(spec.expExecutorResult == group.PROPOSAL_EXECUTOR_RESULT_SUCCESS) {
				// vote is stored and all data persisted
				res, err := s.groupKeeper.VoteByProposalVoter(sdkCtx, &group.QueryVoteByProposalVoterRequest{
					ProposalId: spec.req.ProposalId,
					Voter:      spec.req.Voter,
				})
				s.Require().NoError(err)
				loaded := res.Vote
				s.Assert().Equal(spec.req.ProposalId, loaded.ProposalId)
				s.Assert().Equal(spec.req.Voter, loaded.Voter)
				s.Assert().Equal(spec.req.Option, loaded.Option)
				s.Assert().Equal(spec.req.Metadata, loaded.Metadata)
				s.Assert().Equal(s.blockTime, loaded.SubmitTime)

				// query votes by proposal
				votesByProposalRes, err := s.groupKeeper.VotesByProposal(sdkCtx, &group.QueryVotesByProposalRequest{
					ProposalId: spec.req.ProposalId,
				})
				s.Require().NoError(err)
				votesByProposal := votesByProposalRes.Votes
				s.Require().Equal(1, len(votesByProposal))
				vote := votesByProposal[0]
				s.Assert().Equal(spec.req.ProposalId, vote.ProposalId)
				s.Assert().Equal(spec.req.Voter, vote.Voter)
				s.Assert().Equal(spec.req.Option, vote.Option)
				s.Assert().Equal(spec.req.Metadata, vote.Metadata)
				s.Assert().Equal(s.blockTime, vote.SubmitTime)

				// query votes by voter
				voter := spec.req.Voter
				votesByVoterRes, err := s.groupKeeper.VotesByVoter(sdkCtx, &group.QueryVotesByVoterRequest{
					Voter: voter,
				})
				s.Require().NoError(err)
				votesByVoter := votesByVoterRes.Votes
				s.Require().Equal(1, len(votesByVoter))
				s.Assert().Equal(spec.req.ProposalId, votesByVoter[0].ProposalId)
				s.Assert().Equal(voter, votesByVoter[0].Voter)
				s.Assert().Equal(spec.req.Option, votesByVoter[0].Option)
				s.Assert().Equal(spec.req.Metadata, votesByVoter[0].Metadata)
				s.Assert().Equal(s.blockTime, votesByVoter[0].SubmitTime)

				proposalRes, err := s.groupKeeper.Proposal(sdkCtx, &group.QueryProposalRequest{
					ProposalId: spec.req.ProposalId,
				})
				s.Require().NoError(err)

				proposal := proposalRes.Proposal
				if spec.isFinal {
					s.Assert().Equal(spec.expTallyResult, proposal.FinalTallyResult)
					s.Assert().Equal(spec.expProposalStatus, proposal.Status)
					s.Assert().Equal(spec.expExecutorResult, proposal.ExecutorResult)
				} else {
					s.Assert().Equal(group.DefaultTallyResult(), proposal.FinalTallyResult) // Make sure proposal isn't mutated.

					// do a round of tallying
					tallyResult, err := s.groupKeeper.Tally(sdkCtx, *proposal, myGroupID)
					s.Require().NoError(err)

					s.Assert().Equal(spec.expTallyResult, tallyResult)
				}
			}

			spec.postRun(sdkCtx)
		})
	}

	s.T().Log("test tally result should not take into account the member who left the group")
	members = []group.MemberRequest{
		{Address: s.addrsStr[1], Weight: "3"},
		{Address: s.addrsStr[2], Weight: "2"},
		{Address: s.addrsStr[3], Weight: "1"},
	}
	reqCreate := &group.MsgCreateGroupWithPolicy{
		Admin:         s.addrsStr[0],
		Members:       members,
		GroupMetadata: "metadata",
	}

	policy = group.NewThresholdDecisionPolicy(
		"4",
		time.Duration(10),
		0,
	)
	s.Require().NoError(reqCreate.SetDecisionPolicy(policy))
	s.setNextAccount()

	result, err := s.groupKeeper.CreateGroupWithPolicy(s.ctx, reqCreate)
	s.Require().NoError(err)
	s.Require().NotNil(result)

	policyAddr := result.GroupPolicyAddress
	groupID := result.GroupId
	reqProposal := &group.MsgSubmitProposal{
		GroupPolicyAddress: policyAddr,
		Proposers:          []string{s.addrsStr[3]},
	}
	s.Require().NoError(reqProposal.SetMsgs([]sdk.Msg{&banktypes.MsgSend{
		FromAddress: policyAddr,
		ToAddress:   s.addrsStr[4],
		Amount:      sdk.Coins{sdk.NewInt64Coin("test", 100)},
	}}))

	resSubmitProposal, err := s.groupKeeper.SubmitProposal(s.ctx, reqProposal)
	s.Require().NoError(err)
	s.Require().NotNil(resSubmitProposal)
	proposalID := resSubmitProposal.ProposalId

	for _, voter := range []string{s.addrsStr[3], s.addrsStr[2], s.addrsStr[1]} {
		_, err := s.groupKeeper.Vote(s.ctx,
			&group.MsgVote{ProposalId: proposalID, Voter: voter, Option: group.VOTE_OPTION_YES},
		)
		s.Require().NoError(err)
	}

	qProposals, err := s.groupKeeper.Proposal(s.ctx, &group.QueryProposalRequest{
		ProposalId: proposalID,
	})
	s.Require().NoError(err)

	tallyResult, err := s.groupKeeper.Tally(s.sdkCtx, *qProposals.Proposal, groupID)
	s.Require().NoError(err)

	_, err = s.groupKeeper.LeaveGroup(s.ctx, &group.MsgLeaveGroup{Address: s.addrsStr[3], GroupId: groupID})
	s.Require().NoError(err)

	tallyResult1, err := s.groupKeeper.Tally(s.sdkCtx, *qProposals.Proposal, groupID)
	s.Require().NoError(err)
	s.Require().NotEqual(tallyResult.String(), tallyResult1.String())
}

func (s *TestSuite) TestExecProposal() {
	addrs := s.addrs
	addr2 := addrs[1]

	msgSend1 := &banktypes.MsgSend{
		FromAddress: s.groupPolicyStrAddr,
		ToAddress:   s.addrsStr[1],
		Amount:      sdk.Coins{sdk.NewInt64Coin("test", 100)},
	}
	msgSend2 := &banktypes.MsgSend{
		FromAddress: s.groupPolicyStrAddr,
		ToAddress:   s.addrsStr[1],
		Amount:      sdk.Coins{sdk.NewInt64Coin("test", 10001)},
	}
	proposers := []string{s.addrsStr[1]}

	specs := map[string]struct {
		srcBlockTime      time.Time
		setupProposal     func(ctx context.Context) uint64
		expErr            bool
		expErrMsg         string
		expProposalStatus group.ProposalStatus
		expExecutorResult group.ProposalExecutorResult
		expBalance        bool
		expFromBalances   sdk.Coin
		expToBalances     sdk.Coin
		postRun           func(sdkCtx sdk.Context)
	}{
		"proposal executed when accepted": {
			setupProposal: func(ctx context.Context) uint64 {
				s.bankKeeper.EXPECT().Send(gomock.Any(), msgSend1).Return(nil, nil)
				msgs := []sdk.Msg{msgSend1}
				return submitProposalAndVote(ctx, s, msgs, proposers, group.VOTE_OPTION_YES)
			},
			srcBlockTime:      s.blockTime.Add(minExecutionPeriod), // After min execution period end
			expProposalStatus: group.PROPOSAL_STATUS_ACCEPTED,
			expExecutorResult: group.PROPOSAL_EXECUTOR_RESULT_SUCCESS,
			expBalance:        true,
			expFromBalances:   sdk.NewInt64Coin("test", 9900),
			expToBalances:     sdk.NewInt64Coin("test", 100),
			postRun: func(sdkCtx sdk.Context) {
				events := sdkCtx.EventManager().Events()
				s.Require().True(eventTypeFound(events, EventProposalPruned))
			},
		},
		"proposal with multiple messages executed when accepted": {
			setupProposal: func(ctx context.Context) uint64 {
				msgs := []sdk.Msg{msgSend1, msgSend1}
				s.bankKeeper.EXPECT().Send(gomock.Any(), msgSend1).Return(nil, nil).MaxTimes(2)

				return submitProposalAndVote(ctx, s, msgs, proposers, group.VOTE_OPTION_YES)
			},
			srcBlockTime:      s.blockTime.Add(minExecutionPeriod), // After min execution period end
			expProposalStatus: group.PROPOSAL_STATUS_ACCEPTED,
			expExecutorResult: group.PROPOSAL_EXECUTOR_RESULT_SUCCESS,
			expBalance:        true,
			expFromBalances:   sdk.NewInt64Coin("test", 9800),
			expToBalances:     sdk.NewInt64Coin("test", 200),
			postRun: func(sdkCtx sdk.Context) {
				events := sdkCtx.EventManager().Events()
				s.Require().True(eventTypeFound(events, EventProposalPruned))
			},
		},
		"proposal not executed when rejected": {
			setupProposal: func(ctx context.Context) uint64 {
				msgs := []sdk.Msg{msgSend1}
				return submitProposalAndVote(ctx, s, msgs, proposers, group.VOTE_OPTION_NO)
			},
			srcBlockTime:      s.blockTime.Add(minExecutionPeriod), // After min execution period end
			expProposalStatus: group.PROPOSAL_STATUS_REJECTED,
			expExecutorResult: group.PROPOSAL_EXECUTOR_RESULT_NOT_RUN,
			postRun: func(sdkCtx sdk.Context) {
				events := sdkCtx.EventManager().Events()
				s.Require().False(eventTypeFound(events, EventProposalPruned))
			},
		},
		"open proposal must not fail": {
			setupProposal: func(ctx context.Context) uint64 {
				return submitProposal(ctx, s, []sdk.Msg{msgSend1}, proposers)
			},
			expProposalStatus: group.PROPOSAL_STATUS_SUBMITTED,
			expExecutorResult: group.PROPOSAL_EXECUTOR_RESULT_NOT_RUN,
			postRun: func(sdkCtx sdk.Context) {
				events := sdkCtx.EventManager().Events()
				s.Require().False(eventTypeFound(events, EventProposalPruned))
			},
		},
		"invalid proposal id": {
			setupProposal: func(ctx context.Context) uint64 {
				return 0
			},
			expErr:    true,
			expErrMsg: "proposal id: value is empty",
		},
		"existing proposal required": {
			setupProposal: func(ctx context.Context) uint64 {
				return 9999
			},
			expErr:    true,
			expErrMsg: "load proposal: not found",
		},
		"Decision policy also applied on exactly voting period end": {
			setupProposal: func(ctx context.Context) uint64 {
				msgs := []sdk.Msg{msgSend1}
				return submitProposalAndVote(ctx, s, msgs, proposers, group.VOTE_OPTION_NO)
			},
			srcBlockTime:      s.blockTime.Add(time.Second), // Voting period is 1s
			expProposalStatus: group.PROPOSAL_STATUS_REJECTED,
			expExecutorResult: group.PROPOSAL_EXECUTOR_RESULT_NOT_RUN,
			postRun:           func(sdkCtx sdk.Context) {},
		},
		"Decision policy also applied after voting period end": {
			setupProposal: func(ctx context.Context) uint64 {
				msgs := []sdk.Msg{msgSend1}
				return submitProposalAndVote(ctx, s, msgs, proposers, group.VOTE_OPTION_NO)
			},
			srcBlockTime:      s.blockTime.Add(time.Second).Add(time.Millisecond), // Voting period is 1s
			expProposalStatus: group.PROPOSAL_STATUS_REJECTED,
			expExecutorResult: group.PROPOSAL_EXECUTOR_RESULT_NOT_RUN,
			postRun:           func(sdkCtx sdk.Context) {},
		},
		"exec proposal before MinExecutionPeriod should fail": {
			setupProposal: func(ctx context.Context) uint64 {
				msgs := []sdk.Msg{msgSend1}
				return submitProposalAndVote(ctx, s, msgs, proposers, group.VOTE_OPTION_YES)
			},
			srcBlockTime:      s.blockTime.Add(4 * time.Second), // min execution date is 5s later after s.blockTime
			expProposalStatus: group.PROPOSAL_STATUS_ACCEPTED,
			expExecutorResult: group.PROPOSAL_EXECUTOR_RESULT_FAILURE, // Because MinExecutionPeriod has not passed
			postRun:           func(sdkCtx sdk.Context) {},
		},
		"exec proposal at exactly MinExecutionPeriod should pass": {
			setupProposal: func(ctx context.Context) uint64 {
				msgs := []sdk.Msg{msgSend1}
				s.bankKeeper.EXPECT().Send(gomock.Any(), msgSend1).Return(nil, nil)
				return submitProposalAndVote(ctx, s, msgs, proposers, group.VOTE_OPTION_YES)
			},
			srcBlockTime:      s.blockTime.Add(5 * time.Second), // min execution date is 5s later after s.blockTime
			expProposalStatus: group.PROPOSAL_STATUS_ACCEPTED,
			expExecutorResult: group.PROPOSAL_EXECUTOR_RESULT_SUCCESS,
			postRun: func(sdkCtx sdk.Context) {
				events := sdkCtx.EventManager().Events()
				s.Require().True(eventTypeFound(events, EventProposalPruned))
			},
		},
		"prevent double execution when successful": {
			setupProposal: func(ctx context.Context) uint64 {
				myProposalID := submitProposalAndVote(ctx, s, []sdk.Msg{msgSend1}, proposers, group.VOTE_OPTION_YES)
				s.bankKeeper.EXPECT().Send(gomock.Any(), msgSend1).Return(nil, nil)

				// Wait after min execution period end before Exec
				sdkCtx := sdk.UnwrapSDKContext(ctx)
				sdkCtx = sdkCtx.WithHeaderInfo(header.Info{Time: sdkCtx.HeaderInfo().Time.Add(minExecutionPeriod)}) // MinExecutionPeriod is 5s
				_, err := s.groupKeeper.Exec(sdkCtx, &group.MsgExec{Executor: s.addrsStr[0], ProposalId: myProposalID})
				s.Require().NoError(err)
				return myProposalID
			},
			srcBlockTime:      s.blockTime.Add(minExecutionPeriod), // After min execution period end
			expErr:            true,                                // since proposal is pruned after a successful MsgExec
			expErrMsg:         "load proposal: not found",
			expProposalStatus: group.PROPOSAL_STATUS_ACCEPTED,
			expExecutorResult: group.PROPOSAL_EXECUTOR_RESULT_SUCCESS,
			expBalance:        true,
			expFromBalances:   sdk.NewInt64Coin("test", 9900),
			expToBalances:     sdk.NewInt64Coin("test", 100),
			postRun:           func(sdkCtx sdk.Context) {},
		},
		"rollback all msg updates on failure": {
			setupProposal: func(ctx context.Context) uint64 {
				msgs := []sdk.Msg{msgSend1, msgSend2}
				s.bankKeeper.EXPECT().Send(gomock.Any(), msgSend1).Return(nil, nil)
				s.bankKeeper.EXPECT().Send(gomock.Any(), msgSend2).Return(nil, errors.New("error"))

				return submitProposalAndVote(ctx, s, msgs, proposers, group.VOTE_OPTION_YES)
			},
			srcBlockTime:      s.blockTime.Add(minExecutionPeriod), // After min execution period end
			expProposalStatus: group.PROPOSAL_STATUS_ACCEPTED,
			expExecutorResult: group.PROPOSAL_EXECUTOR_RESULT_FAILURE,
			postRun:           func(sdkCtx sdk.Context) {},
		},
		"executable when failed before": {
			setupProposal: func(ctx context.Context) uint64 {
				msgs := []sdk.Msg{msgSend2}
				myProposalID := submitProposalAndVote(ctx, s, msgs, proposers, group.VOTE_OPTION_YES)

				// Wait after min execution period end before Exec
				sdkCtx := sdk.UnwrapSDKContext(ctx)
				sdkCtx = sdkCtx.WithHeaderInfo(header.Info{Time: sdkCtx.HeaderInfo().Time.Add(minExecutionPeriod)}) // MinExecutionPeriod is 5s
				s.bankKeeper.EXPECT().Send(gomock.Any(), msgSend2).Return(nil, errors.New("error"))
				_, err := s.groupKeeper.Exec(sdkCtx, &group.MsgExec{Executor: s.addrsStr[0], ProposalId: myProposalID})
				s.bankKeeper.EXPECT().Send(gomock.Any(), msgSend2).Return(nil, nil)

				s.Require().NoError(err)
				s.Require().NoError(s.bankKeeper.SendCoinsFromModuleToAccount(s.sdkCtx, minttypes.ModuleName, s.groupPolicyAddr, sdk.Coins{sdk.NewInt64Coin("test", 10000)}))

				return myProposalID
			},
			srcBlockTime:      s.blockTime.Add(minExecutionPeriod), // After min execution period end
			expProposalStatus: group.PROPOSAL_STATUS_ACCEPTED,
			expExecutorResult: group.PROPOSAL_EXECUTOR_RESULT_SUCCESS,
			postRun:           func(sdkCtx sdk.Context) {},
		},
	}
	for msg, spec := range specs {
		s.Run(msg, func() {
			sdkCtx, _ := s.sdkCtx.CacheContext()
			proposalID := spec.setupProposal(sdkCtx)

			if !spec.srcBlockTime.IsZero() {
				sdkCtx = sdkCtx.WithHeaderInfo(header.Info{Time: spec.srcBlockTime})
			}

			_, err := s.groupKeeper.Exec(sdkCtx, &group.MsgExec{Executor: s.addrsStr[0], ProposalId: proposalID})
			if spec.expErr {
				s.Require().Error(err)
				s.Require().Contains(err.Error(), spec.expErrMsg)
				return
			}
			s.Require().NoError(err)

			if !(spec.expExecutorResult == group.PROPOSAL_EXECUTOR_RESULT_SUCCESS) {

				// and proposal is updated
				res, err := s.groupKeeper.Proposal(sdkCtx, &group.QueryProposalRequest{ProposalId: proposalID})
				s.Require().NoError(err)
				proposal := res.Proposal

				exp := group.ProposalStatus_name[int32(spec.expProposalStatus)]
				got := group.ProposalStatus_name[int32(proposal.Status)]
				s.Assert().Equal(exp, got)

				exp = group.ProposalExecutorResult_name[int32(spec.expExecutorResult)]
				got = group.ProposalExecutorResult_name[int32(proposal.ExecutorResult)]
				s.Assert().Equal(exp, got)
			}

			if spec.expBalance {
				s.bankKeeper.EXPECT().GetAllBalances(sdkCtx, s.groupPolicyAddr).Return(sdk.Coins{spec.expFromBalances})
				s.bankKeeper.EXPECT().GetAllBalances(sdkCtx, addr2).Return(sdk.Coins{spec.expToBalances})

				fromBalances := s.bankKeeper.GetAllBalances(sdkCtx, s.groupPolicyAddr)
				s.Require().Contains(fromBalances, spec.expFromBalances)
				toBalances := s.bankKeeper.GetAllBalances(sdkCtx, addr2)
				s.Require().Contains(toBalances, spec.expToBalances)
			}
			spec.postRun(sdkCtx)
		})
	}
}

func (s *TestSuite) TestExecPrunedProposalsAndVotes() {
	proposers := []string{s.addrsStr[1]}
	specs := map[string]struct {
		srcBlockTime      time.Time
		setupProposal     func(ctx context.Context) uint64
		expErr            bool
		expErrMsg         string
		expExecutorResult group.ProposalExecutorResult
	}{
		"proposal pruned after executor result success": {
			setupProposal: func(ctx context.Context) uint64 {
				msgSend1 := &banktypes.MsgSend{
					FromAddress: s.groupPolicyStrAddr,
					ToAddress:   s.addrsStr[1],
					Amount:      sdk.Coins{sdk.NewInt64Coin("test", 101)},
				}
				msgs := []sdk.Msg{msgSend1}
				s.bankKeeper.EXPECT().Send(gomock.Any(), msgSend1).Return(nil, nil)
				return submitProposalAndVote(ctx, s, msgs, proposers, group.VOTE_OPTION_YES)
			},
			expErrMsg:         "load proposal: not found",
			expExecutorResult: group.PROPOSAL_EXECUTOR_RESULT_SUCCESS,
		},
		"proposal with multiple messages pruned when executed with result success": {
			setupProposal: func(ctx context.Context) uint64 {
				msgSend1 := &banktypes.MsgSend{
					FromAddress: s.groupPolicyStrAddr,
					ToAddress:   s.addrsStr[1],
					Amount:      sdk.Coins{sdk.NewInt64Coin("test", 102)},
				}
				s.bankKeeper.EXPECT().Send(gomock.Any(), msgSend1).Return(nil, nil).MaxTimes(2)

				msgs := []sdk.Msg{msgSend1, msgSend1}
				return submitProposalAndVote(ctx, s, msgs, proposers, group.VOTE_OPTION_YES)
			},
			expErrMsg:         "load proposal: not found",
			expExecutorResult: group.PROPOSAL_EXECUTOR_RESULT_SUCCESS,
		},
		"proposal not pruned when not executed and rejected": {
			setupProposal: func(ctx context.Context) uint64 {
				msgSend1 := &banktypes.MsgSend{
					FromAddress: s.groupPolicyStrAddr,
					ToAddress:   s.addrsStr[1],
					Amount:      sdk.Coins{sdk.NewInt64Coin("test", 103)},
				}
				msgs := []sdk.Msg{msgSend1}
				return submitProposalAndVote(ctx, s, msgs, proposers, group.VOTE_OPTION_NO)
			},
			expExecutorResult: group.PROPOSAL_EXECUTOR_RESULT_NOT_RUN,
		},
		"open proposal is not pruned which must not fail ": {
			setupProposal: func(ctx context.Context) uint64 {
				msgSend1 := &banktypes.MsgSend{
					FromAddress: s.groupPolicyStrAddr,
					ToAddress:   s.addrsStr[1],
					Amount:      sdk.Coins{sdk.NewInt64Coin("test", 104)},
				}
				return submitProposal(ctx, s, []sdk.Msg{msgSend1}, proposers)
			},
			expExecutorResult: group.PROPOSAL_EXECUTOR_RESULT_NOT_RUN,
		},
		"proposal not pruned with group modified before tally": {
			setupProposal: func(ctx context.Context) uint64 {
				msgSend1 := &banktypes.MsgSend{
					FromAddress: s.groupPolicyStrAddr,
					ToAddress:   s.addrsStr[1],
					Amount:      sdk.Coins{sdk.NewInt64Coin("test", 105)},
				}
				myProposalID := submitProposal(ctx, s, []sdk.Msg{msgSend1}, proposers)

				// then modify group
				_, err := s.groupKeeper.UpdateGroupMetadata(ctx, &group.MsgUpdateGroupMetadata{
					Admin:   s.addrsStr[0],
					GroupId: s.groupID,
				})
				s.Require().NoError(err)
				return myProposalID
			},
			expExecutorResult: group.PROPOSAL_EXECUTOR_RESULT_NOT_RUN,
		},
		"proposal not pruned with group policy modified before tally": {
			setupProposal: func(ctx context.Context) uint64 {
				msgSend1 := &banktypes.MsgSend{
					FromAddress: s.groupPolicyStrAddr,
					ToAddress:   s.addrsStr[1],
					Amount:      sdk.Coins{sdk.NewInt64Coin("test", 106)},
				}

				myProposalID := submitProposal(ctx, s, []sdk.Msg{msgSend1}, proposers)
				_, err := s.groupKeeper.UpdateGroupPolicyMetadata(ctx, &group.MsgUpdateGroupPolicyMetadata{
					Admin:              s.addrsStr[0],
					GroupPolicyAddress: s.groupPolicyStrAddr,
				})
				s.Require().NoError(err)
				return myProposalID
			},
			expErr:            true, // since proposal status will be `aborted` when group policy is modified
			expErrMsg:         "not possible to exec with proposal status",
			expExecutorResult: group.PROPOSAL_EXECUTOR_RESULT_NOT_RUN,
		},
		"proposal exists when rollback all msg updates on failure": {
			setupProposal: func(ctx context.Context) uint64 {
				msgSend1 := &banktypes.MsgSend{
					FromAddress: s.groupPolicyStrAddr,
					ToAddress:   s.addrsStr[1],
					Amount:      sdk.Coins{sdk.NewInt64Coin("test", 107)},
				}

				msgSend2 := &banktypes.MsgSend{
					FromAddress: s.groupPolicyStrAddr,
					ToAddress:   s.addrsStr[1],
					Amount:      sdk.Coins{sdk.NewInt64Coin("test", 10002)},
				}

				msgs := []sdk.Msg{msgSend1, msgSend2}
				s.bankKeeper.EXPECT().Send(gomock.Any(), msgSend1).Return(nil, errors.New("error"))

				return submitProposalAndVote(ctx, s, msgs, proposers, group.VOTE_OPTION_YES)
			},
			expExecutorResult: group.PROPOSAL_EXECUTOR_RESULT_FAILURE,
		},
		"pruned when proposal is executable when failed before": {
			setupProposal: func(ctx context.Context) uint64 {
				msgSend2 := &banktypes.MsgSend{
					FromAddress: s.groupPolicyStrAddr,
					ToAddress:   s.addrsStr[1],
					Amount:      sdk.Coins{sdk.NewInt64Coin("test", 10003)},
				}

				msgs := []sdk.Msg{msgSend2}

				myProposalID := submitProposalAndVote(ctx, s, msgs, proposers, group.VOTE_OPTION_YES)

				s.bankKeeper.EXPECT().Send(gomock.Any(), msgSend2).Return(nil, errors.New("error"))

				// Wait for min execution period end
				sdkCtx := sdk.UnwrapSDKContext(ctx)
				sdkCtx = sdkCtx.WithHeaderInfo(header.Info{Time: sdkCtx.HeaderInfo().Time.Add(minExecutionPeriod)})
				_, err := s.groupKeeper.Exec(sdkCtx, &group.MsgExec{Executor: s.addrsStr[0], ProposalId: myProposalID})
				s.bankKeeper.EXPECT().Send(gomock.Any(), msgSend2).Return(nil, nil)

				s.Require().NoError(err)
				return myProposalID
			},
			expErrMsg:         "load proposal: not found",
			expExecutorResult: group.PROPOSAL_EXECUTOR_RESULT_SUCCESS,
		},
	}
	for msg, spec := range specs {
		s.Run(msg, func() {
			sdkCtx, _ := s.sdkCtx.CacheContext()
			proposalID := spec.setupProposal(sdkCtx)

			if !spec.srcBlockTime.IsZero() {
				sdkCtx = sdkCtx.WithHeaderInfo(header.Info{Time: spec.srcBlockTime})
			}

			// Wait for min execution period end
			sdkCtx = sdkCtx.WithHeaderInfo(header.Info{Time: sdkCtx.HeaderInfo().Time.Add(minExecutionPeriod)})
			_, err := s.groupKeeper.Exec(sdkCtx, &group.MsgExec{Executor: s.addrsStr[0], ProposalId: proposalID})
			if spec.expErr {
				s.Require().Error(err)
				s.Require().Contains(err.Error(), spec.expErrMsg)
				return
			}
			s.Require().NoError(err)

			if spec.expExecutorResult == group.PROPOSAL_EXECUTOR_RESULT_SUCCESS {
				// Make sure proposal is deleted from state
				_, err := s.groupKeeper.Proposal(sdkCtx, &group.QueryProposalRequest{ProposalId: proposalID})
				s.Require().Contains(err.Error(), spec.expErrMsg)
				res, err := s.groupKeeper.VotesByProposal(sdkCtx, &group.QueryVotesByProposalRequest{ProposalId: proposalID})
				s.Require().NoError(err)
				s.Require().Empty(res.GetVotes())
				events := sdkCtx.EventManager().Events()
				s.Require().True(eventTypeFound(events, EventProposalPruned))

			} else {
				// Check that proposal and votes exists
				res, err := s.groupKeeper.Proposal(sdkCtx, &group.QueryProposalRequest{ProposalId: proposalID})
				s.Require().NoError(err)
				_, err = s.groupKeeper.VotesByProposal(sdkCtx, &group.QueryVotesByProposalRequest{ProposalId: res.Proposal.Id})
				s.Require().NoError(err)
				s.Require().Equal("", spec.expErrMsg)

				exp := group.ProposalExecutorResult_name[int32(spec.expExecutorResult)]
				got := group.ProposalExecutorResult_name[int32(res.Proposal.ExecutorResult)]
				s.Assert().Equal(exp, got)
			}
		})
	}
}

func (s *TestSuite) TestLeaveGroup() {
	addrs := simtestutil.CreateIncrementalAccounts(7)

	admin1 := addrs[0]
	member1, err := s.accountKeeper.AddressCodec().BytesToString(addrs[1])
	s.Require().NoError(err)
	member2, err := s.accountKeeper.AddressCodec().BytesToString(addrs[2])
	s.Require().NoError(err)
	member3, err := s.accountKeeper.AddressCodec().BytesToString(addrs[3])
	s.Require().NoError(err)
	member4, err := s.accountKeeper.AddressCodec().BytesToString(addrs[4])
	s.Require().NoError(err)
	admin2 := addrs[5]
	admin3 := addrs[6]

	members := []group.MemberRequest{
		{
			Address:  member1,
			Weight:   "1",
			Metadata: "metadata",
		},
		{
			Address:  member2,
			Weight:   "2",
			Metadata: "metadata",
		},
		{
			Address:  member3,
			Weight:   "3",
			Metadata: "metadata",
		},
	}
	policy := group.NewThresholdDecisionPolicy(
		"3",
		time.Hour,
		time.Hour,
	)
	s.setNextAccount()
	_, groupID1 := s.createGroupAndGroupPolicy(admin1, members, policy)

	members = []group.MemberRequest{
		{
			Address:  member1,
			Weight:   "1",
			Metadata: "metadata",
		},
	}

	s.setNextAccount()
	_, groupID2 := s.createGroupAndGroupPolicy(admin2, members, nil)

	members = []group.MemberRequest{
		{
			Address:  member1,
			Weight:   "1",
			Metadata: "metadata",
		},
		{
			Address:  member2,
			Weight:   "2",
			Metadata: "metadata",
		},
	}
	policy = &group.PercentageDecisionPolicy{
		Percentage: "0.5",
		Windows:    &group.DecisionPolicyWindows{VotingPeriod: time.Hour},
	}

	s.setNextAccount()

	_, groupID3 := s.createGroupAndGroupPolicy(admin3, members, policy)
	testCases := []struct {
		name           string
		req            *group.MsgLeaveGroup
		expErr         bool
		expErrMsg      string
		expMembersSize int
		memberWeight   math.Dec
	}{
		{
			"group not found",
			&group.MsgLeaveGroup{
				GroupId: 100000,
				Address: member1,
			},
			true,
			"group: not found",
			0,
			math.NewDecFromInt64(0),
		},
		{
			"member address invalid",
			&group.MsgLeaveGroup{
				GroupId: groupID1,
				Address: "invalid",
			},
			true,
			"decoding bech32 failed",
			0,
			math.NewDecFromInt64(0),
		},
		{
			"member not part of group",
			&group.MsgLeaveGroup{
				GroupId: groupID1,
				Address: member4,
			},
			true,
			"not part of group",
			0,
			math.NewDecFromInt64(0),
		},
		{
			"valid testcase: decision policy is not present (and group total weight can be 0)",
			&group.MsgLeaveGroup{
				GroupId: groupID2,
				Address: member1,
			},
			false,
			"",
			0,
			math.NewDecFromInt64(1),
		},
		{
			"valid testcase: threshold decision policy",
			&group.MsgLeaveGroup{
				GroupId: groupID1,
				Address: member3,
			},
			false,
			"",
			2,
			math.NewDecFromInt64(3),
		},
		{
			"valid request: can leave group policy threshold more than group weight",
			&group.MsgLeaveGroup{
				GroupId: groupID1,
				Address: member2,
			},
			false,
			"",
			1,
			math.NewDecFromInt64(2),
		},
		{
			"valid request: can leave group (percentage decision policy)",
			&group.MsgLeaveGroup{
				GroupId: groupID3,
				Address: member2,
			},
			false,
			"",
			1,
			math.NewDecFromInt64(2),
		},
	}

	for _, tc := range testCases {
		s.Run(tc.name, func() {
			var groupWeight1 math.Dec
			if !tc.expErr {
				groupRes, err := s.groupKeeper.GroupInfo(s.ctx, &group.QueryGroupInfoRequest{GroupId: tc.req.GroupId})
				s.Require().NoError(err)
				groupWeight1, err = math.NewNonNegativeDecFromString(groupRes.Info.TotalWeight)
				s.Require().NoError(err)
			}

			res, err := s.groupKeeper.LeaveGroup(s.ctx, tc.req)
			if tc.expErr {
				s.Require().Error(err)
				s.Require().Contains(err.Error(), tc.expErrMsg)
			} else {
				s.Require().NoError(err)
				s.Require().NotNil(res)
				res, err := s.groupKeeper.GroupMembers(s.ctx, &group.QueryGroupMembersRequest{
					GroupId: tc.req.GroupId,
				})
				s.Require().NoError(err)
				s.Require().Len(res.Members, tc.expMembersSize)

				groupRes, err := s.groupKeeper.GroupInfo(s.ctx, &group.QueryGroupInfoRequest{GroupId: tc.req.GroupId})
				s.Require().NoError(err)
				groupWeight2, err := math.NewNonNegativeDecFromString(groupRes.Info.TotalWeight)
				s.Require().NoError(err)

				rWeight, err := groupWeight1.Sub(tc.memberWeight)
				s.Require().NoError(err)
				s.Require().Equal(rWeight.Cmp(groupWeight2), 0)
			}
		})
	}
}

func (s *TestSuite) TestExecProposalsWhenMemberLeavesOrIsUpdated() {
	proposers := []string{s.addrsStr[1]}

	specs := map[string]struct {
		votes         []group.VoteOption
		members       []group.MemberRequest
		setupProposal func(ctx context.Context, groupPolicyAddr string) uint64
		malleate      func(ctx context.Context, k keeper.Keeper, groupPolicyAddr string, groupID uint64) error
		expErrMsg     string
	}{
		"member leaves while all others vote yes: proposal accepted": {
			members: []group.MemberRequest{
				{Address: s.addrsStr[4], Weight: "1"},
				{Address: s.addrsStr[1], Weight: "2"},
				{Address: s.addrsStr[3], Weight: "1"},
				{Address: s.addrsStr[5], Weight: "2"},
				{Address: s.addrsStr[2], Weight: "2"},
			},
			votes: []group.VoteOption{
				group.VOTE_OPTION_YES, group.VOTE_OPTION_YES,
				group.VOTE_OPTION_YES, group.VOTE_OPTION_YES,
				group.VOTE_OPTION_YES,
			},
			setupProposal: func(ctx context.Context, groupPolicyAddr string) uint64 {
				msgSend1 := &banktypes.MsgSend{
					FromAddress: groupPolicyAddr,
					ToAddress:   s.addrsStr[1],
					Amount:      sdk.Coins{sdk.NewInt64Coin("test", 100)},
				}

				// the proposal will pass and be executed
				s.bankKeeper.EXPECT().Send(gomock.Any(), msgSend1).Return(nil, nil).MaxTimes(1)

				msgs := []sdk.Msg{msgSend1}
				proposalReq := &group.MsgSubmitProposal{
					GroupPolicyAddress: groupPolicyAddr,
					Proposers:          proposers,
				}
				err := proposalReq.SetMsgs(msgs)
				s.Require().NoError(err)

				proposalRes, err := s.groupKeeper.SubmitProposal(ctx, proposalReq)
				s.Require().NoError(err)

				return proposalRes.ProposalId
			},
			malleate: func(ctx context.Context, k keeper.Keeper, _ string, groupID uint64) error {
				_, err := k.LeaveGroup(ctx, &group.MsgLeaveGroup{GroupId: groupID, Address: s.addrsStr[5]})
				return err
			},
		},
		"member leaves while all others vote yes and no: proposal rejected": {
			members: []group.MemberRequest{
				{Address: s.addrsStr[4], Weight: "2"},
				{Address: s.addrsStr[1], Weight: "2"},
				{Address: s.addrsStr[3], Weight: "2"},
				{Address: s.addrsStr[2], Weight: "2"},
			},
			votes: []group.VoteOption{
				group.VOTE_OPTION_NO, group.VOTE_OPTION_NO,
				group.VOTE_OPTION_YES, group.VOTE_OPTION_YES,
			},
			setupProposal: func(ctx context.Context, groupPolicyAddr string) uint64 {
				msgSend1 := &banktypes.MsgSend{
					FromAddress: groupPolicyAddr,
					ToAddress:   s.addrsStr[1],
					Amount:      sdk.Coins{sdk.NewInt64Coin("test", 100)},
				}
				msgs := []sdk.Msg{msgSend1, msgSend1}

				proposalReq := &group.MsgSubmitProposal{
					GroupPolicyAddress: groupPolicyAddr,
					Proposers:          proposers,
				}
				err := proposalReq.SetMsgs(msgs)
				s.Require().NoError(err)

				proposalRes, err := s.groupKeeper.SubmitProposal(ctx, proposalReq)
				s.Require().NoError(err)

				return proposalRes.ProposalId
			},
			malleate: func(ctx context.Context, k keeper.Keeper, _ string, groupID uint64) error {
				_, err := k.LeaveGroup(ctx, &group.MsgLeaveGroup{GroupId: groupID, Address: s.addrsStr[3]})
				return err
			},
		},
		"member that leaves does affect the threshold policy outcome": {
			members: []group.MemberRequest{
				{Address: s.addrsStr[3], Weight: "6"},
				{Address: s.addrsStr[1], Weight: "1"},
				{Address: s.addrsStr[5], Weight: "1"},
				{Address: s.addrsStr[2], Weight: "1"},
			},
			votes: []group.VoteOption{
				group.VOTE_OPTION_YES, group.VOTE_OPTION_NO,
				group.VOTE_OPTION_YES, group.VOTE_OPTION_YES,
			},
			setupProposal: func(ctx context.Context, addr string) uint64 {
				msgSend1 := &banktypes.MsgSend{
					FromAddress: addr,
					ToAddress:   s.addrsStr[1],
					Amount:      sdk.Coins{sdk.NewInt64Coin("test", 100)},
				}
				msgs := []sdk.Msg{msgSend1, msgSend1}

				proposalReq := &group.MsgSubmitProposal{
					GroupPolicyAddress: addr,
					Proposers:          proposers,
				}
				err := proposalReq.SetMsgs(msgs)
				s.Require().NoError(err)

				proposalRes, err := s.groupKeeper.SubmitProposal(ctx, proposalReq)
				s.Require().NoError(err)

				return proposalRes.ProposalId
			},
			malleate: func(ctx context.Context, k keeper.Keeper, _ string, groupID uint64) error {
				_, err := k.LeaveGroup(ctx, &group.MsgLeaveGroup{GroupId: groupID, Address: s.addrsStr[3]})
				return err
			},
		},
		"update group policy voids the proposal": {
			members: []group.MemberRequest{
				{Address: s.addrsStr[3], Weight: "2"},
				{Address: s.addrsStr[2], Weight: "2"},
				{Address: s.addrsStr[1], Weight: "2"},
				{Address: s.addrsStr[4], Weight: "2"},
			},
			votes: []group.VoteOption{
				group.VOTE_OPTION_YES, group.VOTE_OPTION_NO,
				group.VOTE_OPTION_YES, group.VOTE_OPTION_NO,
			},
			setupProposal: func(ctx context.Context, groupPolicyAddr string) uint64 {
				msgSend1 := &banktypes.MsgSend{
					FromAddress: groupPolicyAddr,
					ToAddress:   s.addrsStr[1],
					Amount:      sdk.Coins{sdk.NewInt64Coin("test", 100)},
				}
				msgs := []sdk.Msg{msgSend1, msgSend1}
				proposalReq := &group.MsgSubmitProposal{
					GroupPolicyAddress: groupPolicyAddr,
					Proposers:          proposers,
				}
				err := proposalReq.SetMsgs(msgs)
				s.Require().NoError(err)

				proposalRes, err := s.groupKeeper.SubmitProposal(ctx, proposalReq)
				s.Require().NoError(err)

				return proposalRes.ProposalId
			},
			malleate: func(ctx context.Context, k keeper.Keeper, groupPolicyAddr string, groupID uint64) error {
				newGroupPolicy := &group.MsgUpdateGroupPolicyDecisionPolicy{
					Admin:              s.addrsStr[0],
					GroupPolicyAddress: groupPolicyAddr,
				}
				err := newGroupPolicy.SetDecisionPolicy(group.NewThresholdDecisionPolicy("10", time.Second, minExecutionPeriod))
				if err != nil {
					return err
				}
				_, err = k.UpdateGroupPolicyDecisionPolicy(ctx, newGroupPolicy)
				return err
			},
			expErrMsg: "PROPOSAL_STATUS_ABORTED",
		},
	}
	for msg, spec := range specs {
		s.Run(msg, func() {
			sdkCtx, _ := s.sdkCtx.CacheContext()

			s.setNextAccount()
			groupRes, err := s.groupKeeper.CreateGroup(s.ctx, &group.MsgCreateGroup{
				Admin:   s.addrsStr[0],
				Members: spec.members,
			})
			s.Require().NoError(err)
			groupID := groupRes.GroupId

			policy := group.NewThresholdDecisionPolicy("4", time.Second, minExecutionPeriod)
			policyReq := &group.MsgCreateGroupPolicy{
				Admin:   s.addrsStr[0],
				GroupId: groupID,
			}
			err = policyReq.SetDecisionPolicy(policy)
			s.Require().NoError(err)

			s.setNextAccount()

			s.groupKeeper.GetGroupSequence(s.ctx)
			policyRes, err := s.groupKeeper.CreateGroupPolicy(s.ctx, policyReq)
			s.Require().NoError(err)

			// Setup and submit proposal
			proposalID := spec.setupProposal(sdkCtx, policyRes.Address)

			// vote on the proposals
			for i, vote := range spec.votes {
				_, err := s.groupKeeper.Vote(sdkCtx, &group.MsgVote{
					ProposalId: proposalID,
					Voter:      spec.members[i].Address,
					Option:     vote,
				})
				s.Require().NoError(err)
			}

			err = spec.malleate(sdkCtx, s.groupKeeper, policyRes.Address, groupID)
			s.Require().NoError(err)

			// travel in time
			sdkCtx = sdkCtx.WithHeaderInfo(header.Info{Time: s.blockTime.Add(minExecutionPeriod + 1)})
			_, err = s.groupKeeper.Exec(sdkCtx, &group.MsgExec{Executor: s.addrsStr[1], ProposalId: proposalID})
			if spec.expErrMsg != "" {
				s.Require().Contains(err.Error(), spec.expErrMsg)
				return
			}
			s.Require().NoError(err)
		})
	}
}

func eventTypeFound(events []sdk.Event, eventType string) bool {
	eventTypeFound := false
	for _, e := range events {
		if e.Type == eventType {
			eventTypeFound = true
			break
		}
	}
	return eventTypeFound
}
