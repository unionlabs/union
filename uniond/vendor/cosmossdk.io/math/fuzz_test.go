package math

import (
	"testing"
)

func FuzzLegacyNewDecFromStr(f *testing.F) {
	if testing.Short() {
		f.Skip("running in -short mode")
	}

	f.Add("-123.456")
	f.Add("123.456789")
	f.Add("123456789")
	f.Add("0.12123456789")
	f.Add("-12123456789")

	f.Fuzz(func(t *testing.T, input string) {
		dec, err := LegacyNewDecFromStr(input)
		if err != nil && !dec.IsNil() {
			t.Fatalf("Inconsistency: dec.notNil=%v yet err=%v", dec, err)
		}
	})
}
