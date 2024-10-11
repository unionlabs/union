// +build !386

package cbor

import (
	"encoding/hex"
	"testing"
)

var enc2 = Encoder{}

var integerTestCases_64bit = []struct {
	val    int
	binary string
}{
	// Value in 8 bytes.
	{0xabcd100000000, "\x1b\x00\x0a\xbc\xd1\x00\x00\x00\x00"},
	{1000000000000, "\x1b\x00\x00\x00\xe8\xd4\xa5\x10\x00"},
	// Value in 8 bytes.
	{-0xabcd100000001, "\x3b\x00\x0a\xbc\xd1\x00\x00\x00\x00"},
	{-1000000000001, "\x3b\x00\x00\x00\xe8\xd4\xa5\x10\x00"},
}

func TestAppendInt_64bit(t *testing.T) {
	for _, tc := range integerTestCases_64bit {
		s := enc2.AppendInt([]byte{}, tc.val)
		got := string(s)
		if got != tc.binary {
			t.Errorf("AppendInt(0x%x)=0x%s, want: 0x%s",
				tc.val, hex.EncodeToString(s),
				hex.EncodeToString([]byte(tc.binary)))
		}
	}
}
