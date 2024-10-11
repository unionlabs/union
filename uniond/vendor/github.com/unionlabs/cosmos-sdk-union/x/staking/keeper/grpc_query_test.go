package keeper_test

import (
	gocontext "context"
	"fmt"

	"cosmossdk.io/x/staking/testutil"
	"cosmossdk.io/x/staking/types"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

func (s *KeeperTestSuite) TestGRPCQueryValidator() {
	ctx, keeper, queryClient := s.ctx, s.stakingKeeper, s.queryClient
	require := s.Require()

	validator := testutil.NewValidator(s.T(), sdk.ValAddress(PKs[0].Address().Bytes()), PKs[0])
	require.NoError(keeper.SetValidator(ctx, validator))
	var req *types.QueryValidatorRequest
	testCases := []struct {
		msg      string
		malleate func()
		expPass  bool
	}{
		{
			"empty request",
			func() {
				req = &types.QueryValidatorRequest{}
			},
			false,
		},
		{
			"with valid and not existing address",
			func() {
				req = &types.QueryValidatorRequest{
					ValidatorAddr: "cosmosvaloper15jkng8hytwt22lllv6mw4k89qkqehtahd84ptu",
				}
			},
			false,
		},
		{
			"valid request",
			func() {
				req = &types.QueryValidatorRequest{ValidatorAddr: validator.OperatorAddress}
			},
			true,
		},
	}

	for _, tc := range testCases {
		s.Run(fmt.Sprintf("Case %s", tc.msg), func() {
			tc.malleate()
			res, err := queryClient.Validator(gocontext.Background(), req)
			if tc.expPass {
				require.NoError(err)
				require.True(validator.Equal(&res.Validator))
			} else {
				require.Error(err)
				require.Nil(res)
			}
		})
	}
}

func (s *KeeperTestSuite) TestGRPCQueryValidators() {
	ctx, keeper, queryClient := s.ctx, s.stakingKeeper, s.queryClient
	require := s.Require()

	// Create and set validators
	pkValidator1 := PKs[0]
	pkValidator2 := PKs[1]
	validator1 := testutil.NewValidator(s.T(), sdk.ValAddress(pkValidator1.Address().Bytes()), pkValidator1)
	validator2 := testutil.NewValidator(s.T(), sdk.ValAddress(pkValidator2.Address().Bytes()), pkValidator2)
	require.NoError(keeper.SetValidator(ctx, validator1))
	require.NoError(keeper.SetValidator(ctx, validator2))

	// Create a map to associate consensus addresses with validators
	consensusAddrMap := make(map[string]types.Validator)
	consAddr1, err := keeper.ConsensusAddressCodec().BytesToString(pkValidator1.Address())
	require.NoError(err)
	consAddr2, err := keeper.ConsensusAddressCodec().BytesToString(pkValidator2.Address())
	require.NoError(err)
	consensusAddrMap[consAddr1] = validator1
	consensusAddrMap[consAddr2] = validator2

	var req *types.QueryValidatorsRequest
	testCases := []struct {
		msg      string
		malleate func()
	}{
		{
			"valid request with no status",
			func() {
				req = &types.QueryValidatorsRequest{}
			},
		},
		{
			"valid request with bonded status",
			func() {
				req = &types.QueryValidatorsRequest{
					Status: types.Bonded.String(),
				}
			},
		},
	}

	for _, tc := range testCases {
		s.Run(fmt.Sprintf("Case %s", tc.msg), func() {
			tc.malleate()
			res, err := queryClient.Validators(gocontext.Background(), req)
			require.NoError(err)
			require.NotNil(res)
			// Verify that the length of both lists is the same
			require.Equal(len(res.Validators), len(res.ValidatorInfo))

			// Verify that each ValidatorInfo corresponds to the correct Validator and the response match created validators
			for i, valInfo := range res.ValidatorInfo {
				val, exists := consensusAddrMap[valInfo.ConsensusAddress]
				require.True(exists, "Validator not found for consensus address")
				require.Equal(val.OperatorAddress, res.Validators[i].OperatorAddress)
			}
		})
	}
}
