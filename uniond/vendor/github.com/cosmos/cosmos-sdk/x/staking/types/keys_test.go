package types_test

import (
	"bytes"
	"encoding/hex"
	"math/big"
	"testing"
	"time"

	"cosmossdk.io/math"
	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/crypto/keys/ed25519"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/staking/types"
)

var (
	keysPK1   = ed25519.GenPrivKeyFromSecret([]byte{1}).PubKey()
	keysPK2   = ed25519.GenPrivKeyFromSecret([]byte{2}).PubKey()
	keysPK3   = ed25519.GenPrivKeyFromSecret([]byte{3}).PubKey()
	keysAddr1 = keysPK1.Address()
	keysAddr2 = keysPK2.Address()
	keysAddr3 = keysPK3.Address()
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
		got := hex.EncodeToString(types.GetValidatorsByPowerIndexKey(tt.validator, sdk.DefaultPowerReduction))

		require.Equal(t, tt.wantHex, got, "Keys did not match on test case %d", i)
	}
}

func TestGetREDByValDstIndexKey(t *testing.T) {
	tests := []struct {
		delAddr    sdk.AccAddress
		valSrcAddr sdk.ValAddress
		valDstAddr sdk.ValAddress
		wantHex    string
	}{
		{
			sdk.AccAddress(keysAddr1), sdk.ValAddress(keysAddr1), sdk.ValAddress(keysAddr1),
			"361463d771218209d8bd03c482f69dfba57310f086091463d771218209d8bd03c482f69dfba57310f086091463d771218209d8bd03c482f69dfba57310f08609",
		},
		{
			sdk.AccAddress(keysAddr1), sdk.ValAddress(keysAddr2), sdk.ValAddress(keysAddr3),
			"36143ab62f0d93849be495e21e3e9013a517038f45bd1463d771218209d8bd03c482f69dfba57310f08609145ef3b5f25c54946d4a89fc0d09d2f126614540f2",
		},
		{
			sdk.AccAddress(keysAddr2), sdk.ValAddress(keysAddr1), sdk.ValAddress(keysAddr3),
			"36143ab62f0d93849be495e21e3e9013a517038f45bd145ef3b5f25c54946d4a89fc0d09d2f126614540f21463d771218209d8bd03c482f69dfba57310f08609",
		},
	}
	for i, tt := range tests {
		got := hex.EncodeToString(types.GetREDByValDstIndexKey(tt.delAddr, tt.valSrcAddr, tt.valDstAddr))

		require.Equal(t, tt.wantHex, got, "Keys did not match on test case %d", i)
	}
}

func TestGetREDByValSrcIndexKey(t *testing.T) {
	tests := []struct {
		delAddr    sdk.AccAddress
		valSrcAddr sdk.ValAddress
		valDstAddr sdk.ValAddress
		wantHex    string
	}{
		{
			sdk.AccAddress(keysAddr1), sdk.ValAddress(keysAddr1), sdk.ValAddress(keysAddr1),
			"351463d771218209d8bd03c482f69dfba57310f086091463d771218209d8bd03c482f69dfba57310f086091463d771218209d8bd03c482f69dfba57310f08609",
		},
		{
			sdk.AccAddress(keysAddr1), sdk.ValAddress(keysAddr2), sdk.ValAddress(keysAddr3),
			"35145ef3b5f25c54946d4a89fc0d09d2f126614540f21463d771218209d8bd03c482f69dfba57310f08609143ab62f0d93849be495e21e3e9013a517038f45bd",
		},
		{
			sdk.AccAddress(keysAddr2), sdk.ValAddress(keysAddr1), sdk.ValAddress(keysAddr3),
			"351463d771218209d8bd03c482f69dfba57310f08609145ef3b5f25c54946d4a89fc0d09d2f126614540f2143ab62f0d93849be495e21e3e9013a517038f45bd",
		},
	}
	for i, tt := range tests {
		got := hex.EncodeToString(types.GetREDByValSrcIndexKey(tt.delAddr, tt.valSrcAddr, tt.valDstAddr))

		require.Equal(t, tt.wantHex, got, "Keys did not match on test case %d", i)
	}
}

func TestGetValidatorQueueKey(t *testing.T) {
	ts := time.Now()
	height := int64(1024)

	bz := types.GetValidatorQueueKey(ts, height)
	rTs, rHeight, err := types.ParseValidatorQueueKey(bz)
	require.NoError(t, err)
	require.Equal(t, ts.UTC(), rTs.UTC())
	require.Equal(t, rHeight, height)
}

func TestTestGetValidatorQueueKeyOrder(t *testing.T) {
	ts := time.Now().UTC()
	height := int64(1000)

	endKey := types.GetValidatorQueueKey(ts, height)

	keyA := types.GetValidatorQueueKey(ts.Add(-10*time.Minute), height-10)
	keyB := types.GetValidatorQueueKey(ts.Add(-5*time.Minute), height+50)
	keyC := types.GetValidatorQueueKey(ts.Add(10*time.Minute), height+100)

	require.Equal(t, -1, bytes.Compare(keyA, endKey)) // keyA <= endKey
	require.Equal(t, -1, bytes.Compare(keyB, endKey)) // keyB <= endKey
	require.Equal(t, 1, bytes.Compare(keyC, endKey))  // keyB >= endKey
}
