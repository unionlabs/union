package simulation_test

import (
	"fmt"
	"testing"
	"time"

	gogotypes "github.com/cosmos/gogoproto/types"
	"github.com/stretchr/testify/require"

	"cosmossdk.io/depinject"
	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/crypto/keys/ed25519"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/kv"
	"github.com/cosmos/cosmos-sdk/x/slashing/simulation"
	"github.com/cosmos/cosmos-sdk/x/slashing/testutil"
	"github.com/cosmos/cosmos-sdk/x/slashing/types"
)

// nolint:deadcode,unused,varcheck
var (
	delPk1    = ed25519.GenPrivKey().PubKey()
	delAddr1  = sdk.AccAddress(delPk1.Address())
	valAddr1  = sdk.ValAddress(delPk1.Address())
	consAddr1 = sdk.ConsAddress(delPk1.Address().Bytes())
)

func TestDecodeStore(t *testing.T) {
	var cdc codec.Codec
	depinject.Inject(testutil.AppConfig, &cdc)
	dec := simulation.NewDecodeStore(cdc)

	info := types.NewValidatorSigningInfo(consAddr1, 0, 1, time.Now().UTC(), false, 0)
	missed := gogotypes.BoolValue{Value: true}
	bz, err := cdc.MarshalInterface(delPk1)
	require.NoError(t, err)

	kvPairs := kv.Pairs{
		Pairs: []kv.Pair{
			{Key: types.ValidatorSigningInfoKey(consAddr1), Value: cdc.MustMarshal(&info)},
			{Key: types.ValidatorMissedBlockBitArrayKey(consAddr1, 6), Value: cdc.MustMarshal(&missed)},
			{Key: types.AddrPubkeyRelationKey(delAddr1), Value: bz},
			{Key: []byte{0x99}, Value: []byte{0x99}}, // This test should panic
		},
	}

	tests := []struct {
		name        string
		expectedLog string
		panics      bool
	}{
		{"ValidatorSigningInfo", fmt.Sprintf("%v\n%v", info, info), false},
		{"ValidatorMissedBlockBitArray", fmt.Sprintf("missedA: %v\nmissedB: %v", missed.Value, missed.Value), false},
		{"AddrPubkeyRelation", fmt.Sprintf("PubKeyA: %s\nPubKeyB: %s", delPk1, delPk1), false},
		{"other", "", true},
	}
	for i, tt := range tests {
		i, tt := i, tt
		t.Run(tt.name, func(t *testing.T) {
			if tt.panics {
				require.Panics(t, func() { dec(kvPairs.Pairs[i], kvPairs.Pairs[i]) }, tt.name)
			} else {
				require.Equal(t, tt.expectedLog, dec(kvPairs.Pairs[i], kvPairs.Pairs[i]), tt.name)
			}
		})
	}
}
