package addressutil

import (
	"bytes"
	"testing"
)

func TestHexAddressCodec(t *testing.T) {
	tt := []struct {
		text string
		bz   []byte
		err  bool
	}{
		{
			text: "0x1234",
			bz:   []byte{0x12, 0x34},
		},
		{
			text: "0x",
			bz:   []byte{},
		},
		{
			text: "0x123",
			err:  true,
		},
		{
			text: "1234",
			err:  true,
		},
	}

	h := HexAddressCodec{}
	for _, tc := range tt {
		bz, err := h.StringToBytes(tc.text)
		if tc.err && err == nil {
			t.Fatalf("expected error, got none")
		}
		if !tc.err && err != nil {
			t.Fatalf("unexpected error: %v", err)
		}
		if !tc.err && !bytes.Equal(bz, tc.bz) {
			t.Fatalf("expected %v, got %v", tc.bz, bz)
		}

		// check address rendering if no error
		if !tc.err {
			if str, err := h.BytesToString(tc.bz); err != nil {
				t.Fatalf("unexpected error: %v", err)
			} else if str != tc.text {
				t.Fatalf("expected %s, got %s", tc.text, str)
			}
		}
	}
}
