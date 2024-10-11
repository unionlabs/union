package common

import (
	"fmt"

	"cosmossdk.io/x/distribution/types"

	"github.com/cosmos/cosmos-sdk/client"
)

// QueryDelegationRewards queries a delegation rewards between a delegator and a
// validator.
func QueryDelegationRewards(clientCtx client.Context, delAddr, valAddr string) ([]byte, int64, error) {
	delegatorAddr, err := clientCtx.AddressCodec.StringToBytes(delAddr)
	if err != nil {
		return nil, 0, err
	}

	validatorAddr, err := clientCtx.ValidatorAddressCodec.StringToBytes(valAddr)
	if err != nil {
		return nil, 0, err
	}

	params := types.NewQueryDelegationRewardsParams(delegatorAddr, validatorAddr)
	bz, err := clientCtx.LegacyAmino.MarshalJSON(params)
	if err != nil {
		return nil, 0, fmt.Errorf("failed to marshal params: %w", err)
	}

	route := fmt.Sprintf("custom/%s/%s", types.ModuleName, types.QueryDelegationRewards)
	return clientCtx.QueryWithData(route, bz)
}
