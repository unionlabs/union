package types

import (
	"testing"
	"time"

	"github.com/stretchr/testify/require"
	"pgregory.net/rapid"

	"cosmossdk.io/collections/colltest"
)

func TestCollectionsCorrectness(t *testing.T) {
	t.Run("AccAddress", func(t *testing.T) {
		colltest.TestKeyCodec(t, AccAddressKey, AccAddress{0x0, 0x2, 0x3, 0x5})
	})

	t.Run("ValAddress", func(t *testing.T) {
		colltest.TestKeyCodec(t, ValAddressKey, ValAddress{0x1, 0x3, 0x4})
	})

	t.Run("ConsAddress", func(t *testing.T) {
		colltest.TestKeyCodec(t, ConsAddressKey, ConsAddress{0x32, 0x0, 0x0, 0x3})
	})

	t.Run("AddressIndexingKey", func(t *testing.T) {
		colltest.TestKeyCodec(t, LengthPrefixedAddressKey(AccAddressKey), AccAddress{0x2, 0x5, 0x8})
	})

	t.Run("Time", func(t *testing.T) {
		colltest.TestKeyCodec(t, TimeKey, time.Time{})
	})

	t.Run("BytesIndexingKey", func(t *testing.T) {
		colltest.TestKeyCodec(t, LengthPrefixedBytesKey, []byte{})
	})
}

func TestLEUint64Key(t *testing.T) {
	t.Run("conformance", rapid.MakeCheck(func(r *rapid.T) {
		colltest.TestKeyCodec(t, LEUint64Key, rapid.Uint64().Draw(r, "uint64"))
	}))

	t.Run("buffer too small", func(t *testing.T) {
		_, _, err := LEUint64Key.Decode([]byte{0})
		require.ErrorContains(t, err, "invalid buffer size")
	})
}
