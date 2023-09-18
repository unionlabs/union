package simulation_test

import (
	"fmt"
	"testing"
	"time"

	"cosmossdk.io/math"
	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/codec"
	cryptocodec "github.com/cosmos/cosmos-sdk/crypto/codec"
	"github.com/cosmos/cosmos-sdk/crypto/keys/ed25519"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/kv"
	"github.com/cosmos/cosmos-sdk/types/module/testutil"
	"github.com/cosmos/cosmos-sdk/x/staking/simulation"
	"github.com/cosmos/cosmos-sdk/x/staking/types"
)

var (
	delPk1   = ed25519.GenPrivKey().PubKey()
	delAddr1 = sdk.AccAddress(delPk1.Address())
	valAddr1 = sdk.ValAddress(delPk1.Address())
)

func makeTestCodec() (cdc *codec.LegacyAmino) {
	cdc = codec.NewLegacyAmino()
	sdk.RegisterLegacyAminoCodec(cdc)
	cryptocodec.RegisterCrypto(cdc)
	types.RegisterLegacyAminoCodec(cdc)
	return
}

func TestDecodeStore(t *testing.T) {
	cdc := testutil.MakeTestEncodingConfig().Codec
	dec := simulation.NewDecodeStore(cdc)
	bondTime := time.Now().UTC()

	val, err := types.NewValidator(valAddr1, delPk1, types.NewDescription("test", "test", "test", "test", "test"))
	require.NoError(t, err)
	del := types.NewDelegation(delAddr1, valAddr1, math.LegacyOneDec())
	ubd := types.NewUnbondingDelegation(delAddr1, valAddr1, 15, bondTime, math.OneInt(), 1)
	red := types.NewRedelegation(delAddr1, valAddr1, valAddr1, 12, bondTime, math.OneInt(), math.LegacyOneDec(), 0)

	kvPairs := kv.Pairs{
		Pairs: []kv.Pair{
			{Key: types.LastTotalPowerKey, Value: cdc.MustMarshal(&sdk.IntProto{Int: math.OneInt()})},
			{Key: types.GetValidatorKey(valAddr1), Value: cdc.MustMarshal(&val)},
			{Key: types.LastValidatorPowerKey, Value: valAddr1.Bytes()},
			{Key: types.GetDelegationKey(delAddr1, valAddr1), Value: cdc.MustMarshal(&del)},
			{Key: types.GetUBDKey(delAddr1, valAddr1), Value: cdc.MustMarshal(&ubd)},
			{Key: types.GetREDKey(delAddr1, valAddr1, valAddr1), Value: cdc.MustMarshal(&red)},
			{Key: []byte{0x99}, Value: []byte{0x99}},
		},
	}

	tests := []struct {
		name        string
		expectedLog string
	}{
		{"LastTotalPower", fmt.Sprintf("%v\n%v", math.OneInt(), math.OneInt())},
		{"Validator", fmt.Sprintf("%v\n%v", val, val)},
		{"LastValidatorPower/ValidatorsByConsAddr/ValidatorsByPowerIndex", fmt.Sprintf("%v\n%v", valAddr1, valAddr1)},
		{"Delegation", fmt.Sprintf("%v\n%v", del, del)},
		{"UnbondingDelegation", fmt.Sprintf("%v\n%v", ubd, ubd)},
		{"Redelegation", fmt.Sprintf("%v\n%v", red, red)},
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
