package keeper_test

import (
	"context"
	"testing"

	"github.com/cometbft/cometbft/libs/log"
	"github.com/golang/mock/gomock"
	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/baseapp"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/testutil"
	simtestutil "github.com/cosmos/cosmos-sdk/testutil/sims"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
	"github.com/cosmos/cosmos-sdk/types/query"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	"github.com/cosmos/cosmos-sdk/x/group"
	groupkeeper "github.com/cosmos/cosmos-sdk/x/group/keeper"
	"github.com/cosmos/cosmos-sdk/x/group/module"
	grouptestutil "github.com/cosmos/cosmos-sdk/x/group/testutil"
)

func initKeeper(t *testing.T) (sdk.Context, groupkeeper.Keeper, []sdk.AccAddress, group.QueryClient) {
	var (
		groupKeeper       groupkeeper.Keeper
		interfaceRegistry codectypes.InterfaceRegistry
	)

	key := sdk.NewKVStoreKey(group.StoreKey)
	testCtx := testutil.DefaultContextWithDB(t, key, sdk.NewTransientStoreKey("transient_test"))
	encCfg := moduletestutil.MakeTestEncodingConfig(module.AppModuleBasic{})

	ctx := testCtx.Ctx
	bApp := baseapp.NewBaseApp(
		"group",
		log.NewNopLogger(),
		testCtx.DB,
		encCfg.TxConfig.TxDecoder(),
	)

	addrs := simtestutil.CreateIncrementalAccounts(6)
	ctrl := gomock.NewController(t)
	accountKeeper := grouptestutil.NewMockAccountKeeper(ctrl)
	accountKeeper.EXPECT().GetAccount(gomock.Any(), addrs[0]).Return(authtypes.NewBaseAccountWithAddress(addrs[0])).AnyTimes()
	accountKeeper.EXPECT().GetAccount(gomock.Any(), addrs[1]).Return(authtypes.NewBaseAccountWithAddress(addrs[1])).AnyTimes()
	accountKeeper.EXPECT().GetAccount(gomock.Any(), addrs[2]).Return(authtypes.NewBaseAccountWithAddress(addrs[2])).AnyTimes()
	accountKeeper.EXPECT().GetAccount(gomock.Any(), addrs[3]).Return(authtypes.NewBaseAccountWithAddress(addrs[3])).AnyTimes()
	accountKeeper.EXPECT().GetAccount(gomock.Any(), addrs[4]).Return(authtypes.NewBaseAccountWithAddress(addrs[4])).AnyTimes()
	accountKeeper.EXPECT().GetAccount(gomock.Any(), addrs[5]).Return(authtypes.NewBaseAccountWithAddress(addrs[5])).AnyTimes()

	groupKeeper = groupkeeper.NewKeeper(key, encCfg.Codec, bApp.MsgServiceRouter(), accountKeeper, group.DefaultConfig())

	queryHelper := baseapp.NewQueryServerTestHelper(ctx, interfaceRegistry)
	group.RegisterQueryServer(queryHelper, groupKeeper)
	queryClient := group.NewQueryClient(queryHelper)

	return ctx, groupKeeper, addrs, queryClient
}

func TestQueryGroupsByMember(t *testing.T) {
	ctx, groupKeeper, addrs, queryClient := initKeeper(t)

	// Initial group, group policy and balance setup
	members := []group.MemberRequest{
		{Address: addrs[2].String(), Weight: "1"}, {Address: addrs[3].String(), Weight: "2"},
	}

	_, err := groupKeeper.CreateGroup(ctx, &group.MsgCreateGroup{
		Admin:   addrs[0].String(),
		Members: members,
	})
	require.NoError(t, err)

	members = []group.MemberRequest{
		{Address: addrs[3].String(), Weight: "1"}, {Address: addrs[4].String(), Weight: "2"},
	}
	_, err = groupKeeper.CreateGroup(ctx, &group.MsgCreateGroup{
		Admin:   addrs[1].String(),
		Members: members,
	})
	require.NoError(t, err)

	// not part of any group
	resp, err := queryClient.GroupsByMember(context.Background(), &group.QueryGroupsByMemberRequest{
		Address: addrs[5].String(),
	})
	require.NoError(t, err)
	require.Len(t, resp.Groups, 0)

	// expect one group
	resp, err = queryClient.GroupsByMember(context.Background(), &group.QueryGroupsByMemberRequest{
		Address: addrs[4].String(),
	})
	require.NoError(t, err)
	require.Len(t, resp.Groups, 1)

	// expect two groups
	resp, err = queryClient.GroupsByMember(context.Background(), &group.QueryGroupsByMemberRequest{
		Address: addrs[3].String(),
	})
	require.NoError(t, err)
	require.Len(t, resp.Groups, 2)
}

func TestQueryGroups(t *testing.T) {
	ctx, groupKeeper, addrs, queryClient := initKeeper(t)

	// Initial group, group policy and balance setup
	members := []group.MemberRequest{
		{Address: addrs[1].String(), Weight: "1"},
		{Address: addrs[3].String(), Weight: "2"},
	}

	_, err := groupKeeper.CreateGroup(ctx, &group.MsgCreateGroup{
		Admin:   addrs[0].String(),
		Members: members,
	})
	require.NoError(t, err)

	members = []group.MemberRequest{
		{Address: addrs[3].String(), Weight: "1"},
	}
	_, err = groupKeeper.CreateGroup(ctx, &group.MsgCreateGroup{
		Admin:   addrs[2].String(),
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
			resp, err := queryClient.Groups(context.Background(), &group.QueryGroupsRequest{
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
