package keeper_test

import (
	"testing"

	"github.com/stretchr/testify/suite"

	coretesting "cosmossdk.io/core/testing"
	"cosmossdk.io/log"
	"cosmossdk.io/store"
	"cosmossdk.io/store/metrics"
	storetypes "cosmossdk.io/store/types"
	"cosmossdk.io/x/group"
	"cosmossdk.io/x/group/internal/orm"
	"cosmossdk.io/x/group/keeper"

	"github.com/cosmos/cosmos-sdk/codec"
	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	"github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/runtime"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

type invariantTestSuite struct {
	suite.Suite

	ctx sdk.Context
	cdc *codec.ProtoCodec
	key *storetypes.KVStoreKey
}

func TestInvariantTestSuite(t *testing.T) {
	suite.Run(t, new(invariantTestSuite))
}

func (s *invariantTestSuite) SetupSuite() {
	interfaceRegistry := types.NewInterfaceRegistry()
	group.RegisterInterfaces(interfaceRegistry)
	cdc := codec.NewProtoCodec(interfaceRegistry)
	key := storetypes.NewKVStoreKey(group.ModuleName)
	db := coretesting.NewMemDB()
	cms := store.NewCommitMultiStore(db, log.NewNopLogger(), metrics.NewNoOpMetrics())
	cms.MountStoreWithDB(key, storetypes.StoreTypeIAVL, db)
	_ = cms.LoadLatestVersion()
	sdkCtx := sdk.NewContext(cms, false, log.NewNopLogger())

	s.ctx = sdkCtx
	s.cdc = cdc
	s.key = key
}

func (s *invariantTestSuite) TestGroupTotalWeightInvariant() {
	sdkCtx, _ := s.ctx.CacheContext()
	curCtx, cdc, key := sdkCtx, s.cdc, s.key
	addressCodec := codectestutil.CodecOptions{}.GetAddressCodec()

	// Group Table
	groupTable, err := orm.NewAutoUInt64Table([2]byte{keeper.GroupTablePrefix}, keeper.GroupTableSeqPrefix, &group.GroupInfo{}, cdc, addressCodec)
	s.Require().NoError(err)

	// Group Member Table
	groupMemberTable, err := orm.NewPrimaryKeyTable([2]byte{keeper.GroupMemberTablePrefix}, &group.GroupMember{}, cdc, addressCodec)
	s.Require().NoError(err)

	groupMemberByGroupIndex, err := orm.NewIndex(groupMemberTable, keeper.GroupMemberByGroupIndexPrefix, func(val interface{}) ([]interface{}, error) {
		group := val.(*group.GroupMember).GroupId
		return []interface{}{group}, nil
	}, group.GroupMember{}.GroupId)
	s.Require().NoError(err)

	_, _, addr1 := testdata.KeyTestPubAddr()
	_, _, addr2 := testdata.KeyTestPubAddr()

	addr1Str, err := addressCodec.BytesToString(addr1)
	s.Require().NoError(err)
	addr2Str, err := addressCodec.BytesToString(addr2)
	s.Require().NoError(err)

	specs := map[string]struct {
		groupsInfo   *group.GroupInfo
		groupMembers []*group.GroupMember
		expBroken    bool
	}{
		"invariant not broken": {
			groupsInfo: &group.GroupInfo{
				Id:          1,
				Admin:       addr1Str,
				Version:     1,
				TotalWeight: "3",
			},
			groupMembers: []*group.GroupMember{
				{
					GroupId: 1,
					Member: &group.Member{
						Address: addr1Str,
						Weight:  "1",
					},
				},
				{
					GroupId: 1,
					Member: &group.Member{
						Address: addr2Str,
						Weight:  "2",
					},
				},
			},
			expBroken: false,
		},

		"group's TotalWeight must be equal to sum of its members weight ": {
			groupsInfo: &group.GroupInfo{
				Id:          1,
				Admin:       addr1Str,
				Version:     1,
				TotalWeight: "3",
			},
			groupMembers: []*group.GroupMember{
				{
					GroupId: 1,
					Member: &group.Member{
						Address: addr1Str,
						Weight:  "2",
					},
				},
				{
					GroupId: 1,
					Member: &group.Member{
						Address: addr2Str,
						Weight:  "2",
					},
				},
			},
			expBroken: true,
		},
	}

	for _, spec := range specs {
		cacheCurCtx, _ := curCtx.CacheContext()
		groupsInfo := spec.groupsInfo
		groupMembers := spec.groupMembers
		storeService := runtime.NewKVStoreService(key)
		kvStore := storeService.OpenKVStore(cacheCurCtx)
		_, err := groupTable.Create(kvStore, groupsInfo)
		s.Require().NoError(err)

		for i := 0; i < len(groupMembers); i++ {
			err := groupMemberTable.Create(kvStore, groupMembers[i])
			s.Require().NoError(err)
		}

		_, broken := keeper.GroupTotalWeightInvariantHelper(cacheCurCtx, storeService, *groupTable, groupMemberByGroupIndex)
		s.Require().Equal(spec.expBroken, broken)

	}
}
