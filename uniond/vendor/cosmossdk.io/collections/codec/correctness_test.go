package codec_test

import (
	"testing"

	"cosmossdk.io/collections"
	"cosmossdk.io/collections/colltest"
)

func TestKeyCorrectness(t *testing.T) {
	t.Run("bytes", func(t *testing.T) {
		colltest.TestKeyCodec(t, collections.BytesKey, []byte("some_cool_bytes"))
	})

	t.Run("string", func(t *testing.T) {
		colltest.TestKeyCodec(t, collections.StringKey, "some string")
	})

	t.Run("uint64", func(t *testing.T) {
		colltest.TestKeyCodec(t, collections.Uint64Key, 5949485)
	})

	t.Run("uint32", func(t *testing.T) {
		colltest.TestKeyCodec(t, collections.Uint32Key, 5548458)
	})

	t.Run("uint16", func(t *testing.T) {
		colltest.TestKeyCodec(t, collections.Uint16Key, 1005)
	})

	t.Run("bool", func(t *testing.T) {
		colltest.TestKeyCodec(t, collections.BoolKey, true)
		colltest.TestKeyCodec(t, collections.BoolKey, false)
	})

	t.Run("int32", func(t *testing.T) {
		colltest.TestKeyCodec(t, collections.Int32Key, -500)
	})

	t.Run("int64", func(t *testing.T) {
		colltest.TestKeyCodec(t, collections.Int64Key, -100)
	})

	t.Run("Pair", func(t *testing.T) {
		colltest.TestKeyCodec(
			t,
			collections.PairKeyCodec(collections.StringKey, collections.StringKey),
			collections.Join("hello", "testing"),
		)
	})
}
