package humanize

import (
	"math"
	"testing"
)

func TestSI(t *testing.T) {
	tests := []struct {
		name      string
		num       float64
		formatted string
	}{
		{"e-30", 1e-30, "1 qF"},
		{"e-27", 1e-27, "1 rF"},
		{"e-24", 1e-24, "1 yF"},
		{"e-21", 1e-21, "1 zF"},
		{"e-18", 1e-18, "1 aF"},
		{"e-15", 1e-15, "1 fF"},
		{"e-12", 1e-12, "1 pF"},
		{"e-12", 2.2345e-12, "2.2345 pF"},
		{"e-12", 2.23e-12, "2.23 pF"},
		{"e-11", 2.23e-11, "22.3 pF"},
		{"e-10", 2.2e-10, "220 pF"},
		{"e-9", 2.2e-9, "2.2 nF"},
		{"e-8", 2.2e-8, "22 nF"},
		{"e-7", 2.2e-7, "220 nF"},
		{"e-6", 2.2e-6, "2.2 µF"},
		{"e-6", 1e-6, "1 µF"},
		{"e-5", 2.2e-5, "22 µF"},
		{"e-4", 2.2e-4, "220 µF"},
		{"e-3", 2.2e-3, "2.2 mF"},
		{"e-2", 2.2e-2, "22 mF"},
		{"e-1", 2.2e-1, "220 mF"},
		{"e+0", 2.2e-0, "2.2 F"},
		{"e+0", 2.2, "2.2 F"},
		{"e+1", 2.2e+1, "22 F"},
		{"0", 0, "0 F"},
		{"e+1", 22, "22 F"},
		{"e+2", 2.2e+2, "220 F"},
		{"e+2", 220, "220 F"},
		{"e+3", 2.2e+3, "2.2 kF"},
		{"e+3", 2200, "2.2 kF"},
		{"e+4", 2.2e+4, "22 kF"},
		{"e+4", 22000, "22 kF"},
		{"e+5", 2.2e+5, "220 kF"},
		{"e+6", 2.2e+6, "2.2 MF"},
		{"e+6", 1e+6, "1 MF"},
		{"e+7", 2.2e+7, "22 MF"},
		{"e+8", 2.2e+8, "220 MF"},
		{"e+9", 2.2e+9, "2.2 GF"},
		{"e+10", 2.2e+10, "22 GF"},
		{"e+11", 2.2e+11, "220 GF"},
		{"e+12", 2.2e+12, "2.2 TF"},
		{"e+15", 2.2e+15, "2.2 PF"},
		{"e+18", 2.2e+18, "2.2 EF"},
		{"e+21", 2.2e+21, "2.2 ZF"},
		{"e+24", 2.2e+24, "2.2 YF"},
		{"e+27", 2.2e+27, "2.2 RF"},
		{"e+30", 2.2e+30, "2.2 QF"},

		// special case
		{"1F", 1000 * 1000, "1 MF"},
		{"1F", 1e6, "1 MF"},

		// negative number
		{"-100 F", -100, "-100 F"},
	}

	for _, test := range tests {
		got := SI(test.num, "F")
		if got != test.formatted {
			t.Errorf("On %v (%v), got %v, wanted %v",
				test.name, test.num, got, test.formatted)
		}

		gotf, gotu, err := ParseSI(test.formatted)
		if err != nil {
			t.Errorf("Error parsing %v (%v): %v", test.name, test.formatted, err)
			continue
		}

		if math.Abs(1-(gotf/test.num)) > 0.01 {
			t.Errorf("On %v (%v), got %v, wanted %v (±%v)",
				test.name, test.formatted, gotf, test.num,
				math.Abs(1-(gotf/test.num)))
		}
		if gotu != "F" {
			t.Errorf("On %v (%v), expected unit F, got %v",
				test.name, test.formatted, gotu)
		}
	}

	// Parse error
	gotf, gotu, err := ParseSI("x1.21JW") // 1.21 jigga whats
	if err == nil {
		t.Errorf("Expected error on x1.21JW, got %v %v", gotf, gotu)
	}
}

func TestSIWithDigits(t *testing.T) {
	tests := []struct {
		name      string
		num       float64
		digits    int
		formatted string
	}{
		{"e-12", 2.234e-12, 0, "2 pF"},
		{"e-12", 2.234e-12, 1, "2.2 pF"},
		{"e-12", 2.234e-12, 2, "2.23 pF"},
		{"e-12", 2.234e-12, 3, "2.234 pF"},
		{"e-12", 2.234e-12, 4, "2.234 pF"},
	}

	for _, test := range tests {
		got := SIWithDigits(test.num, test.digits, "F")
		if got != test.formatted {
			t.Errorf("On %v (%v), got %v, wanted %v",
				test.name, test.num, got, test.formatted)
		}
	}
}

func BenchmarkParseSI(b *testing.B) {
	for i := 0; i < b.N; i++ {
		ParseSI("2.2346ZB")
	}
}

// There was a report that zeroes were being truncated incorrectly
func TestBug106(t *testing.T) {
	tests := []struct{
		in float64
		want string
	}{
		{20.0, "20 U"},
		{200.0, "200 U"},
	}

	for _, test := range tests {
		if got :=SIWithDigits(test.in, 0, "U") ;  got != test.want {
			t.Errorf("on %f got %v, want %v", test.in, got, test.want);
		}
	}
}
