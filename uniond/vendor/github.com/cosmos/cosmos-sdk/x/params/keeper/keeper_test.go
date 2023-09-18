package keeper_test

import (
	"reflect"
	"testing"

	"cosmossdk.io/math"
	"github.com/stretchr/testify/require"
	"github.com/stretchr/testify/suite"

	"github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/store/prefix"
	"github.com/cosmos/cosmos-sdk/testutil"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
	"github.com/cosmos/cosmos-sdk/x/params"
	"github.com/cosmos/cosmos-sdk/x/params/keeper"
	"github.com/cosmos/cosmos-sdk/x/params/types"
	"github.com/cosmos/cosmos-sdk/x/params/types/proposal"
)

type KeeperTestSuite struct {
	suite.Suite

	ctx          sdk.Context
	paramsKeeper keeper.Keeper
	queryClient  proposal.QueryClient
}

func (suite *KeeperTestSuite) SetupTest() {
	encodingCfg := moduletestutil.MakeTestEncodingConfig(params.AppModuleBasic{})
	key := sdk.NewKVStoreKey(types.StoreKey)
	tkey := sdk.NewTransientStoreKey("params_transient_test")

	suite.ctx = testutil.DefaultContext(key, tkey)
	suite.paramsKeeper = keeper.NewKeeper(encodingCfg.Codec, encodingCfg.Amino, key, tkey)
	suite.paramsKeeper.Subspace("bank")
	suite.paramsKeeper.Subspace("staking")

	queryHelper := baseapp.NewQueryServerTestHelper(suite.ctx, encodingCfg.InterfaceRegistry)
	proposal.RegisterQueryServer(queryHelper, suite.paramsKeeper)

	suite.queryClient = proposal.NewQueryClient(queryHelper)
}

func TestKeeperTestSuite(t *testing.T) {
	suite.Run(t, new(KeeperTestSuite))
}

func validateNoOp(_ interface{}) error { return nil }

func TestKeeper(t *testing.T) {
	kvs := []struct {
		key   string
		param int64
	}{
		{"key1", 10},
		{"key2", 55},
		{"key3", 182},
		{"key4", 17582},
		{"key5", 2768554},
		{"key6", 1157279},
		{"key7", 9058701},
	}

	table := types.NewKeyTable(
		types.NewParamSetPair([]byte("key1"), int64(0), validateNoOp),
		types.NewParamSetPair([]byte("key2"), int64(0), validateNoOp),
		types.NewParamSetPair([]byte("key3"), int64(0), validateNoOp),
		types.NewParamSetPair([]byte("key4"), int64(0), validateNoOp),
		types.NewParamSetPair([]byte("key5"), int64(0), validateNoOp),
		types.NewParamSetPair([]byte("key6"), int64(0), validateNoOp),
		types.NewParamSetPair([]byte("key7"), int64(0), validateNoOp),
		types.NewParamSetPair([]byte("extra1"), bool(false), validateNoOp),
		types.NewParamSetPair([]byte("extra2"), string(""), validateNoOp),
	)

	cdc, ctx, skey, _, keeper := testComponents()

	store := prefix.NewStore(ctx.KVStore(skey), []byte("test/"))
	space := keeper.Subspace("test")
	require.False(t, space.HasKeyTable())
	space = space.WithKeyTable(table)
	require.True(t, space.HasKeyTable())

	// Set params
	for i, kv := range kvs {
		kv := kv
		require.NotPanics(t, func() { space.Set(ctx, []byte(kv.key), kv.param) }, "space.Set panics, tc #%d", i)
	}

	// Test space.Get
	for i, kv := range kvs {
		i, kv := i, kv
		var param int64
		require.NotPanics(t, func() { space.Get(ctx, []byte(kv.key), &param) }, "space.Get panics, tc #%d", i)
		require.Equal(t, kv.param, param, "stored param not equal, tc #%d", i)
	}

	// Test space.GetRaw
	for i, kv := range kvs {
		var param int64
		bz := space.GetRaw(ctx, []byte(kv.key))
		err := cdc.UnmarshalJSON(bz, &param)
		require.Nil(t, err, "err is not nil, tc #%d", i)
		require.Equal(t, kv.param, param, "stored param not equal, tc #%d", i)
	}

	// Test store.Get equals space.Get
	for i, kv := range kvs {
		var param int64
		bz := store.Get([]byte(kv.key))
		require.NotNil(t, bz, "KVStore.Get returns nil, tc #%d", i)
		err := cdc.UnmarshalJSON(bz, &param)
		require.NoError(t, err, "UnmarshalJSON returns error, tc #%d", i)
		require.Equal(t, kv.param, param, "stored param not equal, tc #%d", i)
	}

	// Test invalid space.Get
	for i, kv := range kvs {
		kv := kv
		var param bool
		require.Panics(t, func() { space.Get(ctx, []byte(kv.key), &param) }, "invalid space.Get not panics, tc #%d", i)
	}

	// Test invalid space.Set
	for i, kv := range kvs {
		kv := kv
		require.Panics(t, func() { space.Set(ctx, []byte(kv.key), true) }, "invalid space.Set not panics, tc #%d", i)
	}

	// Test GetSubspace
	for i, kv := range kvs {
		i, kv := i, kv
		var gparam, param int64
		gspace, ok := keeper.GetSubspace("test")
		require.True(t, ok, "cannot retrieve subspace, tc #%d", i)

		require.NotPanics(t, func() { gspace.Get(ctx, []byte(kv.key), &gparam) })
		require.NotPanics(t, func() { space.Get(ctx, []byte(kv.key), &param) })
		require.Equal(t, gparam, param, "GetSubspace().Get not equal with space.Get, tc #%d", i)

		require.NotPanics(t, func() { gspace.Set(ctx, []byte(kv.key), int64(i)) })
		require.NotPanics(t, func() { space.Get(ctx, []byte(kv.key), &param) })
		require.Equal(t, int64(i), param, "GetSubspace().Set not equal with space.Get, tc #%d", i)
	}
}

func indirect(ptr interface{}) interface{} {
	return reflect.ValueOf(ptr).Elem().Interface()
}

func TestGetSubspaces(t *testing.T) {
	_, _, _, _, keeper := testComponents()

	table := types.NewKeyTable(
		types.NewParamSetPair([]byte("string"), "", validateNoOp),
		types.NewParamSetPair([]byte("bool"), false, validateNoOp),
	)

	_ = keeper.Subspace("key1").WithKeyTable(table)
	_ = keeper.Subspace("key2").WithKeyTable(table)

	spaces := keeper.GetSubspaces()
	require.Len(t, spaces, 2)

	var names []string
	for _, ss := range spaces {
		names = append(names, ss.Name())
	}

	require.Contains(t, names, "key1")
	require.Contains(t, names, "key2")
}

func TestSubspace(t *testing.T) {
	cdc, ctx, key, _, keeper := testComponents()

	kvs := []struct {
		key   string
		param interface{}
		zero  interface{}
		ptr   interface{}
	}{
		{"string", "test", "", new(string)},
		{"bool", true, false, new(bool)},
		{"int16", int16(1), int16(0), new(int16)},
		{"int32", int32(1), int32(0), new(int32)},
		{"int64", int64(1), int64(0), new(int64)},
		{"uint16", uint16(1), uint16(0), new(uint16)},
		{"uint32", uint32(1), uint32(0), new(uint32)},
		{"uint64", uint64(1), uint64(0), new(uint64)},
		{"int", sdk.NewInt(1), *new(math.Int), new(math.Int)},
		{"uint", sdk.NewUint(1), *new(sdk.Uint), new(sdk.Uint)},
		{"dec", math.LegacyNewDec(1), *new(sdk.Dec), new(sdk.Dec)},
		{"struct", s{1}, s{0}, new(s)},
	}

	table := types.NewKeyTable(
		types.NewParamSetPair([]byte("string"), "", validateNoOp),
		types.NewParamSetPair([]byte("bool"), false, validateNoOp),
		types.NewParamSetPair([]byte("int16"), int16(0), validateNoOp),
		types.NewParamSetPair([]byte("int32"), int32(0), validateNoOp),
		types.NewParamSetPair([]byte("int64"), int64(0), validateNoOp),
		types.NewParamSetPair([]byte("uint16"), uint16(0), validateNoOp),
		types.NewParamSetPair([]byte("uint32"), uint32(0), validateNoOp),
		types.NewParamSetPair([]byte("uint64"), uint64(0), validateNoOp),
		types.NewParamSetPair([]byte("int"), math.Int{}, validateNoOp),
		types.NewParamSetPair([]byte("uint"), sdk.Uint{}, validateNoOp),
		types.NewParamSetPair([]byte("dec"), sdk.Dec{}, validateNoOp),
		types.NewParamSetPair([]byte("struct"), s{}, validateNoOp),
	)

	store := prefix.NewStore(ctx.KVStore(key), []byte("test/"))
	space := keeper.Subspace("test").WithKeyTable(table)

	// Test space.Set, space.Modified
	for i, kv := range kvs {
		i, kv := i, kv
		require.False(t, space.Modified(ctx, []byte(kv.key)), "space.Modified returns true before setting, tc #%d", i)
		require.NotPanics(t, func() { space.Set(ctx, []byte(kv.key), kv.param) }, "space.Set panics, tc #%d", i)
		require.True(t, space.Modified(ctx, []byte(kv.key)), "space.Modified returns false after setting, tc #%d", i)
	}

	// Test space.Get, space.GetIfExists
	for i, kv := range kvs {
		i, kv := i, kv
		require.NotPanics(t, func() { space.GetIfExists(ctx, []byte("invalid"), kv.ptr) }, "space.GetIfExists panics when no value exists, tc #%d", i)
		require.Equal(t, kv.zero, indirect(kv.ptr), "space.GetIfExists unmarshalls when no value exists, tc #%d", i)
		require.Panics(t, func() { space.Get(ctx, []byte("invalid"), kv.ptr) }, "invalid space.Get not panics when no value exists, tc #%d", i)
		require.Equal(t, kv.zero, indirect(kv.ptr), "invalid space.Get unmarshalls when no value exists, tc #%d", i)

		require.NotPanics(t, func() { space.GetIfExists(ctx, []byte(kv.key), kv.ptr) }, "space.GetIfExists panics, tc #%d", i)
		require.Equal(t, kv.param, indirect(kv.ptr), "stored param not equal, tc #%d", i)
		require.NotPanics(t, func() { space.Get(ctx, []byte(kv.key), kv.ptr) }, "space.Get panics, tc #%d", i)
		require.Equal(t, kv.param, indirect(kv.ptr), "stored param not equal, tc #%d", i)

		require.Panics(t, func() { space.Get(ctx, []byte("invalid"), kv.ptr) }, "invalid space.Get not panics when no value exists, tc #%d", i)
		require.Equal(t, kv.param, indirect(kv.ptr), "invalid space.Get unmarshalls when no value existt, tc #%d", i)

		require.Panics(t, func() { space.Get(ctx, []byte(kv.key), nil) }, "invalid space.Get not panics when the pointer is nil, tc #%d", i)
		require.Panics(t, func() { space.Get(ctx, []byte(kv.key), new(invalid)) }, "invalid space.Get not panics when the pointer is different type, tc #%d", i)
	}

	// Test store.Get equals space.Get
	for i, kv := range kvs {
		bz := store.Get([]byte(kv.key))
		require.NotNil(t, bz, "store.Get() returns nil, tc #%d", i)
		err := cdc.UnmarshalJSON(bz, kv.ptr)
		require.NoError(t, err, "cdc.UnmarshalJSON() returns error, tc #%d", i)
		require.Equal(t, kv.param, indirect(kv.ptr), "stored param not equal, tc #%d", i)
	}
}

type paramJSON struct {
	Param1 int64  `json:"param1,omitempty" yaml:"param1,omitempty"`
	Param2 string `json:"param2,omitempty" yaml:"param2,omitempty"`
}

func TestJSONUpdate(t *testing.T) {
	_, ctx, _, _, keeper := testComponents()

	key := []byte("key")

	space := keeper.Subspace("test").WithKeyTable(types.NewKeyTable(types.NewParamSetPair(key, paramJSON{}, validateNoOp)))

	var param paramJSON

	err := space.Update(ctx, key, []byte(`{"param1": "10241024"}`))
	require.NoError(t, err)
	space.Get(ctx, key, &param)
	require.Equal(t, paramJSON{10241024, ""}, param)

	err = space.Update(ctx, key, []byte(`{"param2": "helloworld"}`))
	require.NoError(t, err)
	space.Get(ctx, key, &param)
	require.Equal(t, paramJSON{10241024, "helloworld"}, param)

	err = space.Update(ctx, key, []byte(`{"param1": "20482048"}`))
	require.NoError(t, err)
	space.Get(ctx, key, &param)
	require.Equal(t, paramJSON{20482048, "helloworld"}, param)

	err = space.Update(ctx, key, []byte(`{"param1": "40964096", "param2": "goodbyeworld"}`))
	require.NoError(t, err)
	space.Get(ctx, key, &param)
	require.Equal(t, paramJSON{40964096, "goodbyeworld"}, param)
}
