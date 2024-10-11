package types_test

import (
	"bytes"
	"fmt"
	"testing"
	"time"

	"github.com/stretchr/testify/suite"

	coretesting "cosmossdk.io/core/testing"
	"cosmossdk.io/log"
	"cosmossdk.io/store"
	"cosmossdk.io/store/metrics"
	storetypes "cosmossdk.io/store/types"
	paramsmodule "cosmossdk.io/x/params"
	"cosmossdk.io/x/params/types"

	"github.com/cosmos/cosmos-sdk/codec"
	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
)

type SubspaceTestSuite struct {
	suite.Suite

	cdc   codec.Codec
	amino *codec.LegacyAmino
	ctx   sdk.Context
	ss    types.Subspace
}

func (suite *SubspaceTestSuite) SetupTest() {
	db := coretesting.NewMemDB()

	ms := store.NewCommitMultiStore(db, log.NewNopLogger(), metrics.NewNoOpMetrics())
	ms.MountStoreWithDB(key, storetypes.StoreTypeIAVL, db)
	ms.MountStoreWithDB(tkey, storetypes.StoreTypeTransient, db)
	suite.NoError(ms.LoadLatestVersion())

	encodingConfig := moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{}, paramsmodule.AppModule{})
	suite.cdc = encodingConfig.Codec
	suite.amino = encodingConfig.Amino

	ss := types.NewSubspace(suite.cdc, suite.amino, key, tkey, "testsubspace")
	suite.ctx = sdk.NewContext(ms, false, log.NewNopLogger())
	suite.ss = ss.WithKeyTable(paramKeyTable())
}

func (suite *SubspaceTestSuite) TestKeyTable() {
	suite.Require().True(suite.ss.HasKeyTable())
	suite.Require().Panics(func() {
		suite.ss.WithKeyTable(paramKeyTable())
	})
	suite.Require().NotPanics(func() {
		ss := types.NewSubspace(suite.cdc, suite.amino, key, tkey, "testsubspace2")
		_ = ss.WithKeyTable(paramKeyTable())
	})
}

func (suite *SubspaceTestSuite) TestGetSet() {
	var v time.Duration
	t := time.Hour * 48

	suite.Require().Panics(func() {
		suite.ss.Get(suite.ctx, keyUnbondingTime, &v)
	})
	suite.Require().NotEqual(t, v)
	suite.Require().NotPanics(func() {
		suite.ss.Set(suite.ctx, keyUnbondingTime, t)
	})
	suite.Require().NotPanics(func() {
		suite.ss.Get(suite.ctx, keyUnbondingTime, &v)
	})
	suite.Require().Equal(t, v)
}

func (suite *SubspaceTestSuite) TestGetIfExists() {
	var v time.Duration

	suite.Require().NotPanics(func() {
		suite.ss.GetIfExists(suite.ctx, keyUnbondingTime, &v)
	})
	suite.Require().Equal(time.Duration(0), v)
}

func (suite *SubspaceTestSuite) TestGetRaw() {
	t := time.Hour * 48

	suite.Require().NotPanics(func() {
		suite.ss.Set(suite.ctx, keyUnbondingTime, t)
	})
	suite.Require().NotPanics(func() {
		res := suite.ss.GetRaw(suite.ctx, keyUnbondingTime)
		suite.Require().Equal("2231373238303030303030303030303022", fmt.Sprintf("%X", res))
	})
}

func (suite *SubspaceTestSuite) TestIterateKeys() {
	suite.Require().NotPanics(func() {
		suite.ss.Set(suite.ctx, keyUnbondingTime, time.Second)
	})
	suite.Require().NotPanics(func() {
		suite.ss.Set(suite.ctx, keyMaxValidators, uint16(50))
	})
	suite.Require().NotPanics(func() {
		suite.ss.Set(suite.ctx, keyBondDenom, "stake")
	})

	var keys [][]byte
	suite.ss.IterateKeys(suite.ctx, func(key []byte) bool {
		keys = append(keys, key)
		return false
	})
	suite.Require().Len(keys, 3)
	suite.Require().Contains(keys, keyUnbondingTime)
	suite.Require().Contains(keys, keyMaxValidators)
	suite.Require().Contains(keys, keyBondDenom)

	var keys2 [][]byte
	suite.ss.IterateKeys(suite.ctx, func(key []byte) bool {
		if bytes.Equal(key, keyUnbondingTime) {
			return true
		}

		keys2 = append(keys2, key)
		return false
	})
	suite.Require().Len(keys2, 2)
	suite.Require().Contains(keys2, keyMaxValidators)
	suite.Require().Contains(keys2, keyBondDenom)
}

func (suite *SubspaceTestSuite) TestHas() {
	t := time.Hour * 48

	suite.Require().False(suite.ss.Has(suite.ctx, keyUnbondingTime))
	suite.Require().NotPanics(func() {
		suite.ss.Set(suite.ctx, keyUnbondingTime, t)
	})
	suite.Require().True(suite.ss.Has(suite.ctx, keyUnbondingTime))
}

func (suite *SubspaceTestSuite) TestModified() {
	t := time.Hour * 48

	suite.Require().False(suite.ss.Modified(suite.ctx, keyUnbondingTime))
	suite.Require().NotPanics(func() {
		suite.ss.Set(suite.ctx, keyUnbondingTime, t)
	})
	suite.Require().True(suite.ss.Modified(suite.ctx, keyUnbondingTime))
}

func (suite *SubspaceTestSuite) TestUpdate() {
	suite.Require().Panics(func() {
		err := suite.ss.Update(suite.ctx, []byte("invalid_key"), nil)
		suite.Require().NoError(err)
	})

	t := time.Hour * 48
	suite.Require().NotPanics(func() {
		suite.ss.Set(suite.ctx, keyUnbondingTime, t)
	})

	bad := time.Minute * 5

	bz, err := suite.amino.MarshalJSON(bad)
	suite.Require().NoError(err)
	suite.Require().Error(suite.ss.Update(suite.ctx, keyUnbondingTime, bz))

	good := time.Hour * 360
	bz, err = suite.amino.MarshalJSON(good)
	suite.Require().NoError(err)
	suite.Require().NoError(suite.ss.Update(suite.ctx, keyUnbondingTime, bz))

	var v time.Duration

	suite.Require().NotPanics(func() {
		suite.ss.Get(suite.ctx, keyUnbondingTime, &v)
	})
	suite.Require().Equal(good, v)
}

func (suite *SubspaceTestSuite) TestGetParamSet() {
	a := params{
		UnbondingTime: time.Hour * 48,
		MaxValidators: 100,
		BondDenom:     "stake",
	}
	suite.Require().NotPanics(func() {
		suite.ss.Set(suite.ctx, keyUnbondingTime, a.UnbondingTime)
		suite.ss.Set(suite.ctx, keyMaxValidators, a.MaxValidators)
		suite.ss.Set(suite.ctx, keyBondDenom, a.BondDenom)
	})

	b := params{}
	suite.Require().NotPanics(func() {
		suite.ss.GetParamSet(suite.ctx, &b)
	})
	suite.Require().Equal(a.UnbondingTime, b.UnbondingTime)
	suite.Require().Equal(a.MaxValidators, b.MaxValidators)
	suite.Require().Equal(a.BondDenom, b.BondDenom)
}

func (suite *SubspaceTestSuite) TestGetParamSetIfExists() {
	a := params{
		UnbondingTime: time.Hour * 48,
		MaxValidators: 100,
		BondDenom:     "stake",
	}
	suite.Require().NotPanics(func() {
		suite.ss.Set(suite.ctx, keyUnbondingTime, a.UnbondingTime)
		suite.ss.Set(suite.ctx, keyMaxValidators, a.MaxValidators)
		suite.ss.Set(suite.ctx, keyBondDenom, a.BondDenom)
	})

	b := paramsV2{}
	suite.Require().NotPanics(func() {
		suite.ss.GetParamSetIfExists(suite.ctx, &b)
	})
	suite.Require().Equal(a.UnbondingTime, b.UnbondingTime)
	suite.Require().Equal(a.MaxValidators, b.MaxValidators)
	suite.Require().Equal(a.BondDenom, b.BondDenom)
	suite.Require().Zero(b.MaxRedelegationEntries)
	suite.Require().False(suite.ss.Has(suite.ctx, keyMaxRedelegationEntries), "key from the new param version should not yet exist")
}

func (suite *SubspaceTestSuite) TestSetParamSet() {
	testCases := []struct {
		name string
		ps   types.ParamSet
	}{
		{"invalid unbonding time", &params{time.Hour * 1, 100, "stake"}},
		{"invalid bond denom", &params{time.Hour * 48, 100, ""}},
	}

	for _, tc := range testCases {
		suite.Run(tc.name, func() {
			suite.Require().Panics(func() {
				suite.ss.SetParamSet(suite.ctx, tc.ps)
			})
		})
	}

	a := params{
		UnbondingTime: time.Hour * 48,
		MaxValidators: 100,
		BondDenom:     "stake",
	}
	suite.Require().NotPanics(func() {
		suite.ss.SetParamSet(suite.ctx, &a)
	})

	b := params{}
	suite.Require().NotPanics(func() {
		suite.ss.GetParamSet(suite.ctx, &b)
	})
	suite.Require().Equal(a.UnbondingTime, b.UnbondingTime)
	suite.Require().Equal(a.MaxValidators, b.MaxValidators)
	suite.Require().Equal(a.BondDenom, b.BondDenom)
}

func (suite *SubspaceTestSuite) TestName() {
	suite.Require().Equal("testsubspace", suite.ss.Name())
}

func TestKeeperTestSuite(t *testing.T) {
	suite.Run(t, new(SubspaceTestSuite))
}
