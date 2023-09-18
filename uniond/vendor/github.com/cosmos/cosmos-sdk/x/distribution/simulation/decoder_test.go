package simulation_test

import (
	"fmt"
	"testing"

	"github.com/stretchr/testify/require"

	"cosmossdk.io/depinject"
	"cosmossdk.io/math"
	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/crypto/keys/ed25519"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/kv"
	"github.com/cosmos/cosmos-sdk/x/distribution/simulation"
	"github.com/cosmos/cosmos-sdk/x/distribution/testutil"
	"github.com/cosmos/cosmos-sdk/x/distribution/types"
)

var (
	delPk1    = ed25519.GenPrivKey().PubKey()
	delAddr1  = sdk.AccAddress(delPk1.Address())
	valAddr1  = sdk.ValAddress(delPk1.Address())
	consAddr1 = sdk.ConsAddress(delPk1.Address().Bytes())
)

func TestDecodeDistributionStore(t *testing.T) {
	var cdc codec.Codec
	err := depinject.Inject(testutil.AppConfig, &cdc)
	require.NoError(t, err)

	dec := simulation.NewDecodeStore(cdc)

	decCoins := sdk.DecCoins{sdk.NewDecCoinFromDec(sdk.DefaultBondDenom, math.LegacyOneDec())}
	feePool := types.InitialFeePool()
	feePool.CommunityPool = decCoins
	info := types.NewDelegatorStartingInfo(2, math.LegacyOneDec(), 200)
	outstanding := types.ValidatorOutstandingRewards{Rewards: decCoins}
	commission := types.ValidatorAccumulatedCommission{Commission: decCoins}
	historicalRewards := types.NewValidatorHistoricalRewards(decCoins, 100)
	currentRewards := types.NewValidatorCurrentRewards(decCoins, 5)
	slashEvent := types.NewValidatorSlashEvent(10, math.LegacyOneDec())

	kvPairs := kv.Pairs{
		Pairs: []kv.Pair{
			{Key: types.FeePoolKey, Value: cdc.MustMarshal(&feePool)},
			{Key: types.ProposerKey, Value: consAddr1.Bytes()},
			{Key: types.GetValidatorOutstandingRewardsKey(valAddr1), Value: cdc.MustMarshal(&outstanding)},
			{Key: types.GetDelegatorWithdrawAddrKey(delAddr1), Value: delAddr1.Bytes()},
			{Key: types.GetDelegatorStartingInfoKey(valAddr1, delAddr1), Value: cdc.MustMarshal(&info)},
			{Key: types.GetValidatorHistoricalRewardsKey(valAddr1, 100), Value: cdc.MustMarshal(&historicalRewards)},
			{Key: types.GetValidatorCurrentRewardsKey(valAddr1), Value: cdc.MustMarshal(&currentRewards)},
			{Key: types.GetValidatorAccumulatedCommissionKey(valAddr1), Value: cdc.MustMarshal(&commission)},
			{Key: types.GetValidatorSlashEventKeyPrefix(valAddr1, 13), Value: cdc.MustMarshal(&slashEvent)},
			{Key: []byte{0x99}, Value: []byte{0x99}},
		},
	}

	tests := []struct {
		name        string
		expectedLog string
	}{
		{"FeePool", fmt.Sprintf("%v\n%v", feePool, feePool)},
		{"Proposer", fmt.Sprintf("%v\n%v", consAddr1, consAddr1)},
		{"ValidatorOutstandingRewards", fmt.Sprintf("%v\n%v", outstanding, outstanding)},
		{"DelegatorWithdrawAddr", fmt.Sprintf("%v\n%v", delAddr1, delAddr1)},
		{"DelegatorStartingInfo", fmt.Sprintf("%v\n%v", info, info)},
		{"ValidatorHistoricalRewards", fmt.Sprintf("%v\n%v", historicalRewards, historicalRewards)},
		{"ValidatorCurrentRewards", fmt.Sprintf("%v\n%v", currentRewards, currentRewards)},
		{"ValidatorAccumulatedCommission", fmt.Sprintf("%v\n%v", commission, commission)},
		{"ValidatorSlashEvent", fmt.Sprintf("%v\n%v", slashEvent, slashEvent)},
		{"other", ""},
	}
	for i, tt := range tests {
		i, tt := i, tt
		t.Run(tt.name, func(t *testing.T) {
			switch i {
			case len(tests) - 1:
				require.Panics(t, func() { dec(kvPairs.Pairs[i], kvPairs.Pairs[i]) }, tt.name)
			default:
				require.Equal(t, tt.expectedLog, dec(kvPairs.Pairs[i], kvPairs.Pairs[i]), tt.name)
			}
		})
	}
}
