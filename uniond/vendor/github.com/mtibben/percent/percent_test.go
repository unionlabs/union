package percent

import (
	"testing"
)

var escapeTests = []struct {
	input          string
	charsToEscape  string
	expectedOutput string
}{
	{"a b c", "", "a b c"},
	{"a b c", "/!@#$", "a b c"},
	{"a c", " ", "a%20c"},
	{"a/c", "/", "a%2Fc"},
	{"é", "é", "%C3%A9"},       // mulitbyte char
	{"😀", "😀", "%F0%9F%98%80"}, // emoji
}

func TestRoundTrip(t *testing.T) {
	for _, tt := range escapeTests {
		t.Run(tt.input, func(t *testing.T) {
			result1 := Encode(tt.input, tt.charsToEscape)
			if result1 != tt.expectedOutput {
				t.Errorf("result1: got %q, want %q", result1, tt.expectedOutput)
			}
			result2 := Decode(result1)
			if result2 != tt.input {
				t.Errorf("result2: got %q, want %q", result2, tt.input)
			}
		})
	}
}
