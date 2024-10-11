package cbor

import (
	"encoding/hex"
	"fmt"
	"math"
	"testing"
	"time"
)

func TestAppendTimeNow(t *testing.T) {
	tm := time.Now()
	s := enc.AppendTime([]byte{}, tm, "unused")
	got := string(s)

	tm1 := float64(tm.Unix()) + float64(tm.Nanosecond())*1E-9
	tm2 := math.Float64bits(tm1)
	var tm3 [8]byte
	for i := uint(0); i < 8; i++ {
		tm3[i] = byte(tm2 >> ((8 - i - 1) * 8))
	}
	want := append([]byte{0xc1, 0xfb}, tm3[:]...)
	if got != string(want) {
		t.Errorf("Appendtime(%s)=0x%s, want: 0x%s",
			"time.Now()", hex.EncodeToString(s),
			hex.EncodeToString(want))
	}
}

var timeIntegerTestcases = []struct {
	txt    string
	binary string
	rfcStr string
}{
	{"2013-02-03T19:54:00-08:00", "\xc1\x1a\x51\x0f\x30\xd8", "2013-02-04T03:54:00Z"},
	{"1950-02-03T19:54:00-08:00", "\xc1\x3a\x25\x71\x93\xa7", "1950-02-04T03:54:00Z"},
}

func TestAppendTimePastPresentInteger(t *testing.T) {
	for _, tt := range timeIntegerTestcases {
		tin, err := time.Parse(time.RFC3339, tt.txt)
		if err != nil {
			fmt.Println("Cannot parse input", tt.txt, ".. Skipping!", err)
			continue
		}
		b := enc.AppendTime([]byte{}, tin, "unused")
		if got, want := string(b), tt.binary; got != want {
			t.Errorf("appendString(%s) = 0x%s, want 0x%s", tt.txt,
				hex.EncodeToString(b),
				hex.EncodeToString([]byte(want)))
		}
	}
}

var timeFloatTestcases = []struct {
	rfcStr string
	out    string
}{
	{"2006-01-02T15:04:05.999999-08:00", "\xc1\xfb\x41\xd0\xee\x6c\x59\x7f\xff\xfc"},
	{"1956-01-02T15:04:05.999999-08:00", "\xc1\xfb\xc1\xba\x53\x81\x1a\x00\x00\x11"},
}

func TestAppendTimePastPresentFloat(t *testing.T) {
	const timeFloatFmt = "2006-01-02T15:04:05.999999-07:00"
	for _, tt := range timeFloatTestcases {
		tin, err := time.Parse(timeFloatFmt, tt.rfcStr)
		if err != nil {
			fmt.Println("Cannot parse input", tt.rfcStr, ".. Skipping!")
			continue
		}
		b := enc.AppendTime([]byte{}, tin, "unused")
		if got, want := string(b), tt.out; got != want {
			t.Errorf("appendString(%s) = 0x%s, want 0x%s", tt.rfcStr,
				hex.EncodeToString(b),
				hex.EncodeToString([]byte(want)))
		}
	}
}

func BenchmarkAppendTime(b *testing.B) {
	tests := map[string]string{
		"Integer": "Feb 3, 2013 at 7:54pm (PST)",
		"Float":   "2006-01-02T15:04:05.999999-08:00",
	}
	const timeFloatFmt = "2006-01-02T15:04:05.999999-07:00"

	for name, str := range tests {
		t, err := time.Parse(time.RFC3339, str)
		if err != nil {
			t, _ = time.Parse(timeFloatFmt, str)
		}
		b.Run(name, func(b *testing.B) {
			buf := make([]byte, 0, 100)
			for i := 0; i < b.N; i++ {
				_ = enc.AppendTime(buf, t, "unused")
			}
		})
	}
}
