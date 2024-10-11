package cbor_test

import (
	"bytes"
	"encoding/hex"
	"fmt"
	"testing"

	"github.com/stretchr/testify/require"

	"cosmossdk.io/x/tx/signing/textual/internal/cbor"
)

var (
	ui  = cbor.NewUint
	txt = cbor.NewText
	arr = cbor.NewArray
	mp  = cbor.NewMap
	ent = cbor.NewEntry
)

func TestCborRFC(t *testing.T) {
	for i, tc := range []struct {
		cb          cbor.Cbor
		encoding    string
		expectError bool
	}{
		// Examples come from RFC8949, Appendix A
		{cb: ui(0), encoding: "00"},
		{cb: ui(1), encoding: "01"},
		{cb: ui(10), encoding: "0a"},
		{cb: ui(23), encoding: "17"},
		{cb: ui(24), encoding: "1818"},
		{cb: ui(25), encoding: "1819"},
		{cb: ui(100), encoding: "1864"},
		{cb: ui(1000), encoding: "1903e8"},
		{cb: ui(1000000), encoding: "1a000f4240"},
		{cb: ui(1000000000000), encoding: "1b000000e8d4a51000"},
		{cb: ui(18446744073709551615), encoding: "1bffffffffffffffff"},
		{cb: cbor.NewBool(false), encoding: "f4"},
		{cb: cbor.NewBool(true), encoding: "f5"},
		{cb: txt(""), encoding: "60"},
		{cb: txt("a"), encoding: "6161"},
		{cb: txt("IETF"), encoding: "6449455446"},
		{cb: txt("\"\\"), encoding: "62225c"},
		{cb: txt("\u00fc"), encoding: "62c3bc"},
		{cb: txt("\u6c34"), encoding: "63e6b0b4"},
		// Go doesn't like string literals with surrogate pairs, create manually
		{cb: txt(string([]byte{0xf0, 0x90, 0x85, 0x91})), encoding: "64f0908591"},
		{cb: arr(), encoding: "80"},
		{cb: arr(ui(1), ui(2)).Append(ui(3)), encoding: "83010203"},
		{
			cb: arr(ui(1)).
				Append(arr(ui(2), ui(3))).
				Append(arr().Append(ui(4)).Append(ui(5))),
			encoding: "8301820203820405",
		},
		{
			cb: arr(
				ui(1), ui(2), ui(3), ui(4), ui(5),
				ui(6), ui(7), ui(8), ui(9), ui(10),
				ui(11), ui(12), ui(13), ui(14), ui(15),
				ui(16), ui(17), ui(18), ui(19), ui(20),
				ui(21), ui(22), ui(23), ui(24), ui(25)),
			encoding: "98190102030405060708090a0b0c0d0e0f101112131415161718181819",
		},
		{cb: mp(), encoding: "a0"},
		{cb: mp(ent(ui(1), ui(2))).Add(ui(3), ui(4)), encoding: "a201020304"},
		{cb: mp(ent(txt("a"), ui(1)), ent(txt("b"), arr(ui(2), ui(3)))), encoding: "a26161016162820203"},
		{cb: arr(txt("a"), mp(ent(txt("b"), txt("c")))), encoding: "826161a161626163"},
		{
			cb: mp(
				ent(txt("a"), txt("A")),
				ent(txt("b"), txt("B")),
				ent(txt("c"), txt("C")),
				ent(txt("d"), txt("D")),
				ent(txt("e"), txt("E"))),
			encoding: "a56161614161626142616361436164614461656145",
		},
		// Departing from the RFC
		{cb: mp(ent(ui(1), ui(2)), ent(ui(1), ui(2))), expectError: true},
		// Map has deterministic order based on key encoding
		{
			cb: mp(
				ent(txt("aa"), ui(0)),
				ent(txt("a"), ui(2)),
				ent(ui(1), txt("b"))),
			encoding: "a301616261610262616100",
		},
	} {
		t.Run(fmt.Sprintf("%d", i), func(t *testing.T) {
			var buf bytes.Buffer
			err := tc.cb.Encode(&buf)
			if tc.expectError {
				require.Error(t, err)
				return
			}
			require.NoError(t, err)
			want, err := hex.DecodeString(tc.encoding)
			require.NoError(t, err)
			require.Equal(t, want, buf.Bytes())
		})
	}
}
