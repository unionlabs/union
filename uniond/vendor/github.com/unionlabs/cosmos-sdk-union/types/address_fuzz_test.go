package types_test

import (
	"encoding/hex"
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/types"
)

func FuzzBech32AccAddrConsistencyYAML(f *testing.F) {
	if testing.Short() {
		f.Skip("running in -short mode")
	}

	f.Add([]byte{0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19})
	f.Add([]byte{16, 1, 2, 3, 4, 5, 16, 27, 58, 9, 51, 11, 12, 13, 14, 15, 16, 17, 20, 21})
	f.Add([]byte{19, 18, 17, 16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0})

	f.Fuzz(func(t *testing.T, input []byte) {
		acc := types.AccAddress(input)
		res := &types.AccAddress{}

		testMarshalYAML(t, &acc, res, acc.MarshalYAML, res.UnmarshalYAML)

		str := acc.String()
		var err error
		*res, err = types.AccAddressFromBech32(str)
		require.NoError(t, err)
		require.Equal(t, acc, *res)

		str = hex.EncodeToString(acc)
		*res, err = types.AccAddressFromHexUnsafe(str)
		require.NoError(t, err)
		require.Equal(t, acc, *res)
	})
}
