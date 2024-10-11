package types_test

import (
	"encoding/hex"
	"math/big"
	"testing"

	"github.com/stretchr/testify/require"

	"cosmossdk.io/math"
	"cosmossdk.io/x/staking/types"

	"github.com/cosmos/cosmos-sdk/codec/address"
	"github.com/cosmos/cosmos-sdk/crypto/keys/ed25519"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

var (
	keysPK1   = ed25519.GenPrivKeyFromSecret([]byte{1}).PubKey()
	keysAddr1 = keysPK1.Address()
)

func TestGetValidatorPowerRank(t *testing.T) {
	valAddr1 := sdk.ValAddress(keysAddr1)
	val1 := newValidator(t, valAddr1, keysPK1)
	val1.Tokens = math.ZeroInt()
	val2, val3, val4 := val1, val1, val1
	val2.Tokens = sdk.TokensFromConsensusPower(1, sdk.DefaultPowerReduction)
	val3.Tokens = sdk.TokensFromConsensusPower(10, sdk.DefaultPowerReduction)
	x := new(big.Int).Exp(big.NewInt(2), big.NewInt(40), big.NewInt(0))
	val4.Tokens = sdk.TokensFromConsensusPower(x.Int64(), sdk.DefaultPowerReduction)

	tests := []struct {
		validator types.Validator
		wantHex   string
	}{
		{val1, "230000000000000000149c288ede7df62742fc3b7d0962045a8cef0f79f6"},
		{val2, "230000000000000001149c288ede7df62742fc3b7d0962045a8cef0f79f6"},
		{val3, "23000000000000000a149c288ede7df62742fc3b7d0962045a8cef0f79f6"},
		{val4, "230000010000000000149c288ede7df62742fc3b7d0962045a8cef0f79f6"},
	}
	for i, tt := range tests {
		got := hex.EncodeToString(types.GetValidatorsByPowerIndexKey(tt.validator, sdk.DefaultPowerReduction, address.NewBech32Codec("cosmosvaloper")))

		require.Equal(t, tt.wantHex, got, "Keys did not match on test case %d", i)
	}
}
