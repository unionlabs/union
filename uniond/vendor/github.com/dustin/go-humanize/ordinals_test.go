package humanize

import (
	"testing"
)

func TestOrdinals(t *testing.T) {
	testList{
		{"0", Ordinal(0), "0th"},
		{"1", Ordinal(1), "1st"},
		{"2", Ordinal(2), "2nd"},
		{"3", Ordinal(3), "3rd"},
		{"4", Ordinal(4), "4th"},
		{"10", Ordinal(10), "10th"},
		{"11", Ordinal(11), "11th"},
		{"12", Ordinal(12), "12th"},
		{"13", Ordinal(13), "13th"},
		{"21", Ordinal(21), "21st"},
		{"32", Ordinal(32), "32nd"},
		{"43", Ordinal(43), "43rd"},
		{"101", Ordinal(101), "101st"},
		{"102", Ordinal(102), "102nd"},
		{"103", Ordinal(103), "103rd"},
		{"211", Ordinal(211), "211th"},
		{"212", Ordinal(212), "212th"},
		{"213", Ordinal(213), "213th"},
	}.validate(t)
}
