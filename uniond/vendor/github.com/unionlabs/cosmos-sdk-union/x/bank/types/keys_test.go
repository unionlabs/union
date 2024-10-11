package types

import (
	"testing"

	"github.com/stretchr/testify/require"

	"cosmossdk.io/collections/colltest"
	"cosmossdk.io/math"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

func TestBalanceValueCodec(t *testing.T) {
	t.Run("value codec implementation", func(t *testing.T) {
		colltest.TestValueCodec(t, BalanceValueCodec, math.NewInt(100))
	})

	t.Run("legacy coin", func(t *testing.T) {
		coin := sdk.NewInt64Coin("coin", 1000)
		b, err := coin.Marshal()
		require.NoError(t, err)
		amt, err := BalanceValueCodec.Decode(b)
		require.NoError(t, err)
		require.Equal(t, coin.Amount, amt)
	})
}
