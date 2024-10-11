package keeper_test

import (
	"testing"
	"time"

	cmtproto "github.com/cometbft/cometbft/api/cometbft/types/v1"
	cmttypes "github.com/cometbft/cometbft/types"
	gogotypes "github.com/cosmos/gogoproto/types"
	"github.com/stretchr/testify/suite"

	"cosmossdk.io/core/header"
	coretesting "cosmossdk.io/core/testing"
	storetypes "cosmossdk.io/store/types"
	consensusparamkeeper "cosmossdk.io/x/consensus/keeper"
	"cosmossdk.io/x/consensus/types"

	"github.com/cosmos/cosmos-sdk/baseapp"
	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	"github.com/cosmos/cosmos-sdk/runtime"
	"github.com/cosmos/cosmos-sdk/testutil"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/address"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
)

type KeeperTestSuite struct {
	suite.Suite
	ctx                   sdk.Context
	consensusParamsKeeper *consensusparamkeeper.Keeper

	queryClient types.QueryClient
}

func getDuration(d time.Duration) *time.Duration {
	dur := d
	return &dur
}

func (s *KeeperTestSuite) SetupTest(enabledFeatures bool) {
	key := storetypes.NewKVStoreKey(types.StoreKey)
	testCtx := testutil.DefaultContextWithDB(s.T(), key, storetypes.NewTransientStoreKey("transient_test"))
	ctx := testCtx.Ctx.WithHeaderInfo(header.Info{Height: 5})
	encCfg := moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{})
	env := runtime.NewEnvironment(runtime.NewKVStoreService(key), coretesting.NewNopLogger())

	authority, err := codectestutil.CodecOptions{}.GetAddressCodec().BytesToString(address.Module("gov"))
	s.Require().NoError(err)

	keeper := consensusparamkeeper.NewKeeper(encCfg.Codec, env, authority)

	s.ctx = ctx
	s.consensusParamsKeeper = &keeper

	types.RegisterInterfaces(encCfg.InterfaceRegistry)
	queryHelper := baseapp.NewQueryServerTestHelper(ctx, encCfg.InterfaceRegistry)
	types.RegisterQueryServer(queryHelper, keeper)
	s.queryClient = types.NewQueryClient(queryHelper)
	params := cmttypes.DefaultConsensusParams()
	if enabledFeatures {
		params.Feature.VoteExtensionsEnableHeight = 5
		params.Feature.PbtsEnableHeight = 5
	}
	err = s.consensusParamsKeeper.ParamsStore.Set(ctx, params.ToProto())
	s.Require().NoError(err)
}

func TestKeeperTestSuite(t *testing.T) {
	suite.Run(t, new(KeeperTestSuite))
}

func (s *KeeperTestSuite) TestGRPCQueryConsensusParams() {
	// Create ConsensusParams with modified fields
	modifiedConsensusParams := cmttypes.DefaultConsensusParams().ToProto()
	modifiedConsensusParams.Block.MaxBytes++
	modifiedConsensusParams.Block.MaxGas = 100
	modifiedConsensusParams.Evidence.MaxAgeDuration++
	modifiedConsensusParams.Evidence.MaxAgeNumBlocks++
	modifiedConsensusParams.Evidence.MaxBytes++
	modifiedConsensusParams.Validator.PubKeyTypes = []string{cmttypes.ABCIPubKeyTypeSecp256k1}
	*modifiedConsensusParams.Synchrony.MessageDelay += time.Second
	*modifiedConsensusParams.Synchrony.Precision += 100 * time.Millisecond
	modifiedConsensusParams.Feature.VoteExtensionsEnableHeight.Value = 200
	modifiedConsensusParams.Feature.PbtsEnableHeight.Value = 100

	testCases := []struct {
		msg      string
		req      types.QueryParamsRequest
		malleate func()
		response types.QueryParamsResponse
		expPass  bool
	}{
		{
			"success",
			types.QueryParamsRequest{},
			func() {
				input := &types.MsgUpdateParams{
					Authority: s.consensusParamsKeeper.GetAuthority(),
					Block:     modifiedConsensusParams.Block,
					Validator: modifiedConsensusParams.Validator,
					Evidence:  modifiedConsensusParams.Evidence,
					Synchrony: modifiedConsensusParams.Synchrony,
					Feature:   modifiedConsensusParams.Feature,
				}
				_, err := s.consensusParamsKeeper.UpdateParams(s.ctx, input)
				s.Require().NoError(err)
			},
			types.QueryParamsResponse{
				Params: &cmtproto.ConsensusParams{
					Block:     modifiedConsensusParams.Block,
					Validator: modifiedConsensusParams.Validator,
					Evidence:  modifiedConsensusParams.Evidence,
					Version:   modifiedConsensusParams.Version,
					Synchrony: modifiedConsensusParams.Synchrony,
					Feature:   modifiedConsensusParams.Feature,
				},
			},
			true,
		},
		{
			"success with (deprecated) ABCI",
			types.QueryParamsRequest{},
			func() {
				input := &types.MsgUpdateParams{
					Authority: s.consensusParamsKeeper.GetAuthority(),
					Block:     modifiedConsensusParams.Block,
					Validator: modifiedConsensusParams.Validator,
					Evidence:  modifiedConsensusParams.Evidence,
					Synchrony: modifiedConsensusParams.Synchrony,
					Abci: &cmtproto.ABCIParams{ //nolint: staticcheck // testing backwards compatibility
						VoteExtensionsEnableHeight: 1234,
					},
				}
				_, err := s.consensusParamsKeeper.UpdateParams(s.ctx, input)
				s.Require().NoError(err)
			},
			types.QueryParamsResponse{
				Params: &cmtproto.ConsensusParams{
					Block:     modifiedConsensusParams.Block,
					Validator: modifiedConsensusParams.Validator,
					Evidence:  modifiedConsensusParams.Evidence,
					Version:   modifiedConsensusParams.Version,
					Synchrony: modifiedConsensusParams.Synchrony,
					Feature: &cmtproto.FeatureParams{
						VoteExtensionsEnableHeight: &gogotypes.Int64Value{Value: 1234},
						PbtsEnableHeight:           &gogotypes.Int64Value{Value: 0},
					},
				},
			},
			true,
		},
	}

	for _, tc := range testCases {
		s.Run(tc.msg, func() {
			s.SetupTest(false) // reset

			tc.malleate()
			res, err := s.consensusParamsKeeper.Params(s.ctx, &tc.req)

			if tc.expPass {
				s.Require().NoError(err)
				s.Require().NotNil(res)
				s.Require().Equal(tc.response.Params, res.Params)
			} else {
				s.Require().Error(err)
				s.Require().Nil(res)
			}
		})
	}
}

func (s *KeeperTestSuite) TestUpdateParams() {
	defaultConsensusParams := cmttypes.DefaultConsensusParams().ToProto()
	testCases := []struct {
		name            string
		enabledFeatures bool
		input           *types.MsgUpdateParams
		expErr          bool
		expErrMsg       string
	}{
		{
			name: "valid params",
			input: &types.MsgUpdateParams{
				Authority: s.consensusParamsKeeper.GetAuthority(),
				Block:     defaultConsensusParams.Block,
				Validator: defaultConsensusParams.Validator,
				Evidence:  defaultConsensusParams.Evidence,
			},
		},
		{
			name: "invalid  params",
			input: &types.MsgUpdateParams{
				Authority: s.consensusParamsKeeper.GetAuthority(),
				Block:     &cmtproto.BlockParams{MaxGas: -10, MaxBytes: -10},
				Validator: defaultConsensusParams.Validator,
				Evidence:  defaultConsensusParams.Evidence,
			},
			expErr:    true,
			expErrMsg: "block.MaxBytes must be -1 or greater than 0. Got -10",
		},
		{
			name: "invalid authority",
			input: &types.MsgUpdateParams{
				Authority: "invalid",
				Block:     defaultConsensusParams.Block,
				Validator: defaultConsensusParams.Validator,
				Evidence:  defaultConsensusParams.Evidence,
			},
			expErr:    true,
			expErrMsg: "invalid authority",
		},
		{
			name: "nil evidence params",
			input: &types.MsgUpdateParams{
				Authority: s.consensusParamsKeeper.GetAuthority(),
				Block:     defaultConsensusParams.Block,
				Validator: defaultConsensusParams.Validator,
				Evidence:  nil,
			},
			expErr:    true,
			expErrMsg: "all parameters must be present",
		},
		{
			name: "nil block params",
			input: &types.MsgUpdateParams{
				Authority: s.consensusParamsKeeper.GetAuthority(),
				Block:     nil,
				Validator: defaultConsensusParams.Validator,
				Evidence:  defaultConsensusParams.Evidence,
			},
			expErr:    true,
			expErrMsg: "all parameters must be present",
		},
		{
			name: "nil validator params",
			input: &types.MsgUpdateParams{
				Authority: s.consensusParamsKeeper.GetAuthority(),
				Block:     defaultConsensusParams.Block,
				Validator: nil,
				Evidence:  defaultConsensusParams.Evidence,
			},
			expErr:    true,
			expErrMsg: "all parameters must be present",
		},
		{
			name: "valid Feature update - vote extensions",
			input: &types.MsgUpdateParams{
				Authority: s.consensusParamsKeeper.GetAuthority(),
				Block:     defaultConsensusParams.Block,
				Validator: defaultConsensusParams.Validator,
				Evidence:  defaultConsensusParams.Evidence,
				Feature: &cmtproto.FeatureParams{
					VoteExtensionsEnableHeight: &gogotypes.Int64Value{Value: 300},
				},
			},
		},
		{
			name: "valid Feature update - pbts",
			input: &types.MsgUpdateParams{
				Authority: s.consensusParamsKeeper.GetAuthority(),
				Block:     defaultConsensusParams.Block,
				Validator: defaultConsensusParams.Validator,
				Evidence:  defaultConsensusParams.Evidence,
				Feature: &cmtproto.FeatureParams{
					PbtsEnableHeight: &gogotypes.Int64Value{Value: 150},
				},
			},
		},
		{
			name: "valid Feature update - vote extensions + pbts",
			input: &types.MsgUpdateParams{
				Authority: s.consensusParamsKeeper.GetAuthority(),
				Block:     defaultConsensusParams.Block,
				Validator: defaultConsensusParams.Validator,
				Evidence:  defaultConsensusParams.Evidence,
				Feature: &cmtproto.FeatureParams{
					VoteExtensionsEnableHeight: &gogotypes.Int64Value{Value: 120},
					PbtsEnableHeight:           &gogotypes.Int64Value{Value: 110},
				},
			},
		},
		{
			name:            "valid noop Feature update - vote extensions + pbts (enabled feature)",
			enabledFeatures: true,
			input: &types.MsgUpdateParams{
				Authority: s.consensusParamsKeeper.GetAuthority(),
				Block:     defaultConsensusParams.Block,
				Validator: defaultConsensusParams.Validator,
				Evidence:  defaultConsensusParams.Evidence,
				Feature: &cmtproto.FeatureParams{
					VoteExtensionsEnableHeight: &gogotypes.Int64Value{Value: 5},
					PbtsEnableHeight:           &gogotypes.Int64Value{Value: 5},
				},
			},
		},
		{
			name: "valid (deprecated) ABCI update",
			input: &types.MsgUpdateParams{
				Authority: s.consensusParamsKeeper.GetAuthority(),
				Block:     defaultConsensusParams.Block,
				Validator: defaultConsensusParams.Validator,
				Evidence:  defaultConsensusParams.Evidence,
				Abci: &cmtproto.ABCIParams{ //nolint: staticcheck // testing backwards compatibility
					VoteExtensionsEnableHeight: 90,
				},
			},
		},
		{
			name: "invalid Feature + (deprecated) ABCI vote extensions update",
			input: &types.MsgUpdateParams{
				Authority: s.consensusParamsKeeper.GetAuthority(),
				Block:     defaultConsensusParams.Block,
				Validator: defaultConsensusParams.Validator,
				Evidence:  defaultConsensusParams.Evidence,
				Abci: &cmtproto.ABCIParams{ //nolint: staticcheck // testing backwards compatibility
					VoteExtensionsEnableHeight: 3000,
				},
				Feature: &cmtproto.FeatureParams{
					VoteExtensionsEnableHeight: &gogotypes.Int64Value{Value: 3000},
				},
			},
			expErr:    true,
			expErrMsg: "abci in sections Feature and (deprecated) ABCI cannot be used simultaneously",
		},
		{
			name: "invalid vote extensions update - current height",
			input: &types.MsgUpdateParams{
				Authority: s.consensusParamsKeeper.GetAuthority(),
				Block:     defaultConsensusParams.Block,
				Validator: defaultConsensusParams.Validator,
				Evidence:  defaultConsensusParams.Evidence,
				Feature: &cmtproto.FeatureParams{
					VoteExtensionsEnableHeight: &gogotypes.Int64Value{Value: 5},
				},
			},
			expErr:    true,
			expErrMsg: "xtensions cannot be updated to a past or current height",
		},
		{
			name: "invalid pbts update - current height",
			input: &types.MsgUpdateParams{
				Authority: s.consensusParamsKeeper.GetAuthority(),
				Block:     defaultConsensusParams.Block,
				Validator: defaultConsensusParams.Validator,
				Evidence:  defaultConsensusParams.Evidence,
				Feature: &cmtproto.FeatureParams{
					PbtsEnableHeight: &gogotypes.Int64Value{Value: 5},
				},
			},
			expErr:    true,
			expErrMsg: "PBTS cannot be updated to a past or current height",
		},
		{
			name: "invalid vote extensions update - past height",
			input: &types.MsgUpdateParams{
				Authority: s.consensusParamsKeeper.GetAuthority(),
				Block:     defaultConsensusParams.Block,
				Validator: defaultConsensusParams.Validator,
				Evidence:  defaultConsensusParams.Evidence,
				Feature: &cmtproto.FeatureParams{
					VoteExtensionsEnableHeight: &gogotypes.Int64Value{Value: 4},
				},
			},
			expErr:    true,
			expErrMsg: "xtensions cannot be updated to a past or current height",
		},
		{
			name: "invalid pbts update - past height",
			input: &types.MsgUpdateParams{
				Authority: s.consensusParamsKeeper.GetAuthority(),
				Block:     defaultConsensusParams.Block,
				Validator: defaultConsensusParams.Validator,
				Evidence:  defaultConsensusParams.Evidence,
				Feature: &cmtproto.FeatureParams{
					PbtsEnableHeight: &gogotypes.Int64Value{Value: 5},
				},
			},
			expErr:    true,
			expErrMsg: "PBTS cannot be updated to a past or current height",
		},
		{
			name: "invalid vote extensions update - negative height",
			input: &types.MsgUpdateParams{
				Authority: s.consensusParamsKeeper.GetAuthority(),
				Block:     defaultConsensusParams.Block,
				Validator: defaultConsensusParams.Validator,
				Evidence:  defaultConsensusParams.Evidence,
				Feature: &cmtproto.FeatureParams{
					VoteExtensionsEnableHeight: &gogotypes.Int64Value{Value: -1},
				},
			},
			expErr:    true,
			expErrMsg: "Feature.VoteExtensionsEnabledHeight cannot be negative",
		},
		{
			name: "invalid pbts update - negative height",
			input: &types.MsgUpdateParams{
				Authority: s.consensusParamsKeeper.GetAuthority(),
				Block:     defaultConsensusParams.Block,
				Validator: defaultConsensusParams.Validator,
				Evidence:  defaultConsensusParams.Evidence,
				Feature: &cmtproto.FeatureParams{
					PbtsEnableHeight: &gogotypes.Int64Value{Value: -1},
				},
			},
			expErr:    true,
			expErrMsg: "Feature.PbtsEnableHeight cannot be negative",
		},
		{
			name:            "invalid vote extensions update - enabled feature",
			enabledFeatures: true,
			input: &types.MsgUpdateParams{
				Authority: s.consensusParamsKeeper.GetAuthority(),
				Block:     defaultConsensusParams.Block,
				Validator: defaultConsensusParams.Validator,
				Evidence:  defaultConsensusParams.Evidence,
				Feature: &cmtproto.FeatureParams{
					VoteExtensionsEnableHeight: &gogotypes.Int64Value{Value: 25},
				},
			},
			expErr:    true,
			expErrMsg: "xtensions cannot be modified once enabledenabled",
		},
		{
			name:            "invalid pbts update - enabled feature",
			enabledFeatures: true,
			input: &types.MsgUpdateParams{
				Authority: s.consensusParamsKeeper.GetAuthority(),
				Block:     defaultConsensusParams.Block,
				Validator: defaultConsensusParams.Validator,
				Evidence:  defaultConsensusParams.Evidence,
				Feature: &cmtproto.FeatureParams{
					PbtsEnableHeight: &gogotypes.Int64Value{Value: 35},
				},
			},
			expErr:    true,
			expErrMsg: "PBTS cannot be modified once enabled",
		},
		{
			name: "valid Synchrony update - precision",
			input: &types.MsgUpdateParams{
				Authority: s.consensusParamsKeeper.GetAuthority(),
				Block:     defaultConsensusParams.Block,
				Validator: defaultConsensusParams.Validator,
				Evidence:  defaultConsensusParams.Evidence,
				Synchrony: &cmtproto.SynchronyParams{
					Precision: getDuration(3 * time.Second),
				},
			},
		},
		{
			name: "valid Synchrony update - delay",
			input: &types.MsgUpdateParams{
				Authority: s.consensusParamsKeeper.GetAuthority(),
				Block:     defaultConsensusParams.Block,
				Validator: defaultConsensusParams.Validator,
				Evidence:  defaultConsensusParams.Evidence,
				Synchrony: &cmtproto.SynchronyParams{
					MessageDelay: getDuration(10 * time.Second),
				},
			},
		},
		{
			name: "valid Synchrony update - precision + delay",
			input: &types.MsgUpdateParams{
				Authority: s.consensusParamsKeeper.GetAuthority(),
				Block:     defaultConsensusParams.Block,
				Validator: defaultConsensusParams.Validator,
				Evidence:  defaultConsensusParams.Evidence,
				Synchrony: &cmtproto.SynchronyParams{
					Precision:    getDuration(4 * time.Second),
					MessageDelay: getDuration(11 * time.Second),
				},
			},
		},
		{
			name: "valid Synchrony update - 0 precision",
			input: &types.MsgUpdateParams{
				Authority: s.consensusParamsKeeper.GetAuthority(),
				Block:     defaultConsensusParams.Block,
				Validator: defaultConsensusParams.Validator,
				Evidence:  defaultConsensusParams.Evidence,
				Synchrony: &cmtproto.SynchronyParams{
					Precision: getDuration(0),
				},
			},
		},
		{
			name: "valid Synchrony update - 0 delay",
			input: &types.MsgUpdateParams{
				Authority: s.consensusParamsKeeper.GetAuthority(),
				Block:     defaultConsensusParams.Block,
				Validator: defaultConsensusParams.Validator,
				Evidence:  defaultConsensusParams.Evidence,
				Synchrony: &cmtproto.SynchronyParams{
					MessageDelay: getDuration(0),
				},
			},
		},
		{
			name: "invalid Synchrony update - 0 precision with PBTS set",
			input: &types.MsgUpdateParams{
				Authority: s.consensusParamsKeeper.GetAuthority(),
				Block:     defaultConsensusParams.Block,
				Validator: defaultConsensusParams.Validator,
				Evidence:  defaultConsensusParams.Evidence,
				Feature: &cmtproto.FeatureParams{
					PbtsEnableHeight: &gogotypes.Int64Value{Value: 20},
				},
				Synchrony: &cmtproto.SynchronyParams{
					Precision: getDuration(0),
				},
			},
			expErr:    true,
			expErrMsg: "synchrony.Precision must be greater than 0",
		},
		{
			name: "invalid Synchrony update - 0 delay with PBTS set",
			input: &types.MsgUpdateParams{
				Authority: s.consensusParamsKeeper.GetAuthority(),
				Block:     defaultConsensusParams.Block,
				Validator: defaultConsensusParams.Validator,
				Evidence:  defaultConsensusParams.Evidence,
				Feature: &cmtproto.FeatureParams{
					PbtsEnableHeight: &gogotypes.Int64Value{Value: 20},
				},
				Synchrony: &cmtproto.SynchronyParams{
					MessageDelay: getDuration(0),
				},
			},
			expErr:    true,
			expErrMsg: "synchrony.MessageDelay must be greater than 0",
		},
	}

	for _, tc := range testCases {
		s.Run(tc.name, func() {
			s.SetupTest(tc.enabledFeatures)
			_, err := s.consensusParamsKeeper.UpdateParams(s.ctx, tc.input)
			if tc.expErr {
				s.Require().Error(err)
				s.Require().Contains(err.Error(), tc.expErrMsg)
			} else {
				s.Require().NoError(err)

				res, err := s.consensusParamsKeeper.Params(s.ctx, &types.QueryParamsRequest{})
				s.Require().NoError(err)

				s.Require().Equal(tc.input.Block, res.Params.Block)
				s.Require().Equal(tc.input.Evidence, res.Params.Evidence)
				s.Require().Equal(tc.input.Validator, res.Params.Validator)
				if tc.input.Abci != nil {
					s.Require().Equal(tc.input.Abci.VoteExtensionsEnableHeight,
						res.Params.Feature.VoteExtensionsEnableHeight.GetValue())
				}
				if tc.input.Feature != nil {
					if tc.input.Feature.VoteExtensionsEnableHeight != nil {
						s.Require().Equal(tc.input.Feature.VoteExtensionsEnableHeight.GetValue(),
							res.Params.Feature.VoteExtensionsEnableHeight.GetValue())
					}
					if tc.input.Feature.PbtsEnableHeight != nil {
						s.Require().Equal(tc.input.Feature.PbtsEnableHeight.GetValue(),
							res.Params.Feature.PbtsEnableHeight.GetValue())
					}
				}
				if tc.input.Synchrony != nil {
					if tc.input.Synchrony.MessageDelay != nil {
						s.Require().Equal(tc.input.Synchrony.MessageDelay,
							res.Params.Synchrony.MessageDelay)
					}
					if tc.input.Synchrony.Precision != nil {
						s.Require().Equal(tc.input.Synchrony.Precision,
							res.Params.Synchrony.Precision)
					}
				}
			}
		})
	}
}
