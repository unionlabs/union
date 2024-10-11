package keeper_test

import (
	"context"
	"testing"
	"time"

	"github.com/golang/mock/gomock"
	"github.com/stretchr/testify/require"

	"cosmossdk.io/log"
	storetypes "cosmossdk.io/store/types"
	"cosmossdk.io/x/group"
	groupkeeper "cosmossdk.io/x/group/keeper"
	"cosmossdk.io/x/group/module"
	grouptestutil "cosmossdk.io/x/group/testutil"

	"github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/codec/address"
	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/runtime"
	"github.com/cosmos/cosmos-sdk/testutil"
	simtestutil "github.com/cosmos/cosmos-sdk/testutil/sims"
	"github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
	"github.com/cosmos/cosmos-sdk/types/query"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
)

type fixture struct {
	ctx          types.Context
	keeper       groupkeeper.Keeper
	queryClient  group.QueryClient
	addrs        []string
	defaultGroup *group.MsgCreateGroupWithPolicyResponse
}

func initKeeper(t *testing.T) *fixture {
	t.Helper()
	var (
		groupKeeper       groupkeeper.Keeper
		interfaceRegistry codectypes.InterfaceRegistry
	)

	key := storetypes.NewKVStoreKey(group.StoreKey)
	testCtx := testutil.DefaultContextWithDB(t, key, storetypes.NewTransientStoreKey("transient_test"))
	encCfg := moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{}, module.AppModule{})

	ctx := testCtx.Ctx

	bApp := baseapp.NewBaseApp(
		"group",
		log.NewNopLogger(),
		testCtx.DB,
		encCfg.TxConfig.TxDecoder(),
	)

	addressCodec := address.NewBech32Codec("cosmos")
	accAddrs := simtestutil.CreateIncrementalAccounts(6)
	addrs := make([]string, len(accAddrs))

	ctrl := gomock.NewController(t)
	accountKeeper := grouptestutil.NewMockAccountKeeper(ctrl)
	var err error
	for i, addr := range accAddrs {
		accountKeeper.EXPECT().GetAccount(gomock.Any(), addr).Return(authtypes.NewBaseAccountWithAddress(addr)).AnyTimes()
		addrs[i], err = addressCodec.BytesToString(addr)
		require.NoError(t, err)
	}
	accountKeeper.EXPECT().AddressCodec().Return(addressCodec).AnyTimes()

	// group policy expected calls
	accountKeeper.EXPECT().GetAccount(gomock.Any(), gomock.Any()).Return(nil).AnyTimes()
	accountKeeper.EXPECT().NewAccount(gomock.Any(), gomock.Any()).Return(nil).AnyTimes()
	accountKeeper.EXPECT().SetAccount(gomock.Any(), gomock.Any()).AnyTimes()

	env := runtime.NewEnvironment(runtime.NewKVStoreService(key), log.NewNopLogger(), runtime.EnvWithQueryRouterService(bApp.GRPCQueryRouter()), runtime.EnvWithMsgRouterService(bApp.MsgServiceRouter()))

	groupKeeper = groupkeeper.NewKeeper(env, encCfg.Codec, accountKeeper, group.DefaultConfig())
	queryHelper := baseapp.NewQueryServerTestHelper(ctx, interfaceRegistry)
	group.RegisterQueryServer(queryHelper, groupKeeper)
	queryClient := group.NewQueryClient(queryHelper)

	msgGroupAndPolicy := &group.MsgCreateGroupWithPolicy{
		Admin: addrs[0],
		Members: []group.MemberRequest{
			{Address: addrs[1], Weight: "1"},
			{Address: addrs[3], Weight: "2"},
		},
	}
	err = msgGroupAndPolicy.SetDecisionPolicy(group.NewThresholdDecisionPolicy("2", time.Second, 20))
	require.NoError(t, err)

	resp, err := groupKeeper.CreateGroupWithPolicy(ctx, msgGroupAndPolicy)
	require.NoError(t, err)

	return &fixture{
		ctx:          ctx,
		keeper:       groupKeeper,
		queryClient:  queryClient,
		addrs:        addrs,
		defaultGroup: resp,
	}
}

func TestQueryGroupInfo(t *testing.T) {
	fixture := initKeeper(t)

	testCases := []struct {
		name      string
		req       group.QueryGroupInfoRequest
		expErrMsg string
	}{
		{
			name:      "invalid req",
			expErrMsg: "group: not found",
		},
		{
			name:      "unknown id",
			req:       group.QueryGroupInfoRequest{GroupId: 20},
			expErrMsg: "group: not found",
		},
		{
			name:      "valid id",
			req:       group.QueryGroupInfoRequest{GroupId: 1},
			expErrMsg: "",
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			_, err := fixture.queryClient.GroupInfo(fixture.ctx, &tc.req)
			if tc.expErrMsg != "" {
				require.Error(t, err)
				require.Contains(t, err.Error(), tc.expErrMsg)
			} else {
				require.NoError(t, err)
			}
		})
	}
}

func TestQueryGroupPolicyInfo(t *testing.T) {
	fixture := initKeeper(t)

	testCases := []struct {
		name      string
		req       group.QueryGroupPolicyInfoRequest
		expErrMsg string
	}{
		{
			name:      "valid address",
			req:       group.QueryGroupPolicyInfoRequest{Address: fixture.defaultGroup.GroupPolicyAddress},
			expErrMsg: "",
		},
		{
			name:      "unexisting address",
			req:       group.QueryGroupPolicyInfoRequest{Address: fixture.addrs[5]},
			expErrMsg: "group policy: not found",
		},
		{
			name:      "invalid address",
			req:       group.QueryGroupPolicyInfoRequest{Address: "invalid address"},
			expErrMsg: "decoding bech32 failed",
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			_, err := fixture.queryClient.GroupPolicyInfo(fixture.ctx, &tc.req)
			if tc.expErrMsg != "" {
				require.Error(t, err)
				require.Contains(t, err.Error(), tc.expErrMsg)
			} else {
				require.NoError(t, err)
			}
		})
	}
}

func TestQueryGroupMembers(t *testing.T) {
	fixture := initKeeper(t)

	testCases := []struct {
		name      string
		req       group.QueryGroupMembersRequest
		postRun   func(resp *group.QueryGroupMembersResponse)
		expErrMsg string
	}{
		{
			name: "valid group",
			req:  group.QueryGroupMembersRequest{GroupId: 1},
			postRun: func(resp *group.QueryGroupMembersResponse) {
				require.Len(t, resp.Members, 2)
			},
			expErrMsg: "",
		},
		{
			name: "unexisting group",
			req:  group.QueryGroupMembersRequest{GroupId: 20},
			postRun: func(resp *group.QueryGroupMembersResponse) {
				require.Len(t, resp.Members, 0)
			},
			expErrMsg: "",
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			resp, err := fixture.queryClient.GroupMembers(fixture.ctx, &tc.req)
			if tc.expErrMsg != "" {
				require.Error(t, err)
				require.Contains(t, err.Error(), tc.expErrMsg)
			} else {
				require.NoError(t, err)
			}

			if tc.postRun != nil {
				tc.postRun(resp)
			}
		})
	}
}

func TestQueryGroupsByAdmin(t *testing.T) {
	fixture := initKeeper(t)

	testCases := []struct {
		name      string
		req       group.QueryGroupsByAdminRequest
		postRun   func(resp *group.QueryGroupsByAdminResponse)
		expErrMsg string
	}{
		{
			name:      "valid admin",
			req:       group.QueryGroupsByAdminRequest{Admin: fixture.addrs[0]},
			postRun:   func(resp *group.QueryGroupsByAdminResponse) { require.Len(t, resp.Groups, 1) },
			expErrMsg: "",
		},
		{
			name:      "unexisting address",
			req:       group.QueryGroupsByAdminRequest{Admin: fixture.addrs[5]},
			postRun:   func(resp *group.QueryGroupsByAdminResponse) { require.Len(t, resp.Groups, 0) },
			expErrMsg: "",
		},
		{
			name:      "invalid address",
			req:       group.QueryGroupsByAdminRequest{Admin: "invalid address"},
			expErrMsg: "decoding bech32 failed",
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			resp, err := fixture.queryClient.GroupsByAdmin(fixture.ctx, &tc.req)
			if tc.expErrMsg != "" {
				require.Error(t, err)
				require.Contains(t, err.Error(), tc.expErrMsg)
			} else {
				require.NoError(t, err)
			}

			if tc.postRun != nil {
				tc.postRun(resp)
			}
		})
	}
}

func TestQueryGroupPoliciesByGroup(t *testing.T) {
	fixture := initKeeper(t)

	testCases := []struct {
		name      string
		req       group.QueryGroupPoliciesByGroupRequest
		postRun   func(resp *group.QueryGroupPoliciesByGroupResponse)
		expErrMsg string
	}{
		{
			name:      "valid group",
			req:       group.QueryGroupPoliciesByGroupRequest{GroupId: 1},
			postRun:   func(resp *group.QueryGroupPoliciesByGroupResponse) { require.Len(t, resp.GroupPolicies, 1) },
			expErrMsg: "",
		},
		{
			name:      "unexisting group",
			req:       group.QueryGroupPoliciesByGroupRequest{GroupId: 20},
			postRun:   func(resp *group.QueryGroupPoliciesByGroupResponse) { require.Len(t, resp.GroupPolicies, 0) },
			expErrMsg: "",
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			resp, err := fixture.keeper.GroupPoliciesByGroup(fixture.ctx, &tc.req)
			if tc.expErrMsg != "" {
				require.Error(t, err)
				require.Contains(t, err.Error(), tc.expErrMsg)
			} else {
				require.NoError(t, err)
			}

			if tc.postRun != nil {
				tc.postRun(resp)
			}
		})
	}
}

func TestQueryGroupPoliciesByAdmin(t *testing.T) {
	fixture := initKeeper(t)

	testCases := []struct {
		name      string
		req       group.QueryGroupPoliciesByAdminRequest
		postRun   func(resp *group.QueryGroupPoliciesByAdminResponse)
		expErrMsg string
	}{
		{
			name:      "valid admin",
			req:       group.QueryGroupPoliciesByAdminRequest{Admin: fixture.addrs[0]},
			postRun:   func(resp *group.QueryGroupPoliciesByAdminResponse) { require.Len(t, resp.GroupPolicies, 1) },
			expErrMsg: "",
		},
		{
			name:      "unexisting address",
			req:       group.QueryGroupPoliciesByAdminRequest{Admin: fixture.addrs[5]},
			postRun:   func(resp *group.QueryGroupPoliciesByAdminResponse) { require.Len(t, resp.GroupPolicies, 0) },
			expErrMsg: "",
		},
		{
			name:      "invalid address",
			req:       group.QueryGroupPoliciesByAdminRequest{Admin: "invalid address"},
			expErrMsg: "decoding bech32 failed",
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			resp, err := fixture.keeper.GroupPoliciesByAdmin(fixture.ctx, &tc.req)
			if tc.expErrMsg != "" {
				require.Error(t, err)
				require.Contains(t, err.Error(), tc.expErrMsg)
			} else {
				require.NoError(t, err)
			}

			if tc.postRun != nil {
				tc.postRun(resp)
			}
		})
	}
}

func TestQueryGroupsByMember(t *testing.T) {
	fixture := initKeeper(t)

	members := []group.MemberRequest{
		{Address: fixture.addrs[3], Weight: "1"}, {Address: fixture.addrs[4], Weight: "2"},
	}
	_, err := fixture.keeper.CreateGroup(fixture.ctx, &group.MsgCreateGroup{
		Admin:   fixture.addrs[1],
		Members: members,
	})
	require.NoError(t, err)

	// not part of any group
	resp, err := fixture.queryClient.GroupsByMember(context.Background(), &group.QueryGroupsByMemberRequest{
		Address: fixture.addrs[5],
	})
	require.NoError(t, err)
	require.Len(t, resp.Groups, 0)

	// expect one group
	resp, err = fixture.queryClient.GroupsByMember(context.Background(), &group.QueryGroupsByMemberRequest{
		Address: fixture.addrs[4],
	})
	require.NoError(t, err)
	require.Len(t, resp.Groups, 1)

	// expect two groups
	resp, err = fixture.queryClient.GroupsByMember(context.Background(), &group.QueryGroupsByMemberRequest{
		Address: fixture.addrs[3],
	})
	require.NoError(t, err)
	require.Len(t, resp.Groups, 2)
}

func TestQueryGroups(t *testing.T) {
	fixture := initKeeper(t)

	members := []group.MemberRequest{
		{Address: fixture.addrs[3], Weight: "1"},
	}
	_, err := fixture.keeper.CreateGroup(fixture.ctx, &group.MsgCreateGroup{
		Admin:   fixture.addrs[2],
		Members: members,
	})
	require.NoError(t, err)

	testCases := []struct {
		name         string
		expErr       bool
		expLen       int
		itemsPerPage uint64
	}{
		{
			name:         "success case, without pagination",
			expErr:       false,
			expLen:       2,
			itemsPerPage: 10,
		},
		{
			name:         "success case, with pagination",
			expErr:       false,
			expLen:       1,
			itemsPerPage: 1,
		},
		{
			name:   "success without pagination",
			expErr: false,
			expLen: 2,
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			resp, err := fixture.queryClient.Groups(context.Background(), &group.QueryGroupsRequest{
				Pagination: &query.PageRequest{
					Limit: tc.itemsPerPage,
				},
			})

			if tc.expErr {
				require.Error(t, err)
			} else {
				require.NoError(t, err)
				require.Equal(t, len(resp.Groups), tc.expLen)
			}
		})
	}
}
