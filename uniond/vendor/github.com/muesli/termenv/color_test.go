package termenv

import "testing"

func TestXTermColor(t *testing.T) {
	var tests = []struct {
		input string
		color RGBColor
		valid bool
	}{
		{
			"\033]11;rgb:fafa/fafa/fafa\033",
			RGBColor("#fafafa"),
			true,
		},
		{
			"\033]11;rgb:fafa/fafa/fafa\033\\",
			RGBColor("#fafafa"),
			true,
		},
		{
			"\033]11;rgb:1212/3434/5656\a",
			RGBColor("#123456"),
			true,
		},
		{
			"\033]11;foo:fafa/fafa/fafaZZ",
			"",
			false,
		},
		{
			"\033]11;rgb:fafa/fafa",
			"",
			false,
		},
		{
			"\033]11;rgb:fafa/fafa/fafaY",
			"",
			false,
		},
		{
			"\033]11;rgb:fafa/fafa/fafaZZ",
			"",
			false,
		},
	}

	for _, test := range tests {
		t.Run("", func(t *testing.T) {
			color, err := xTermColor(test.input)
			if err != nil && test.valid {
				t.Fatalf("unexpected error for input %q: %v", test.input, err)
			}

			if err == nil && !test.valid {
				t.Fatalf("expected error for input %v not found", test.input)
			}

			if color != test.color {
				t.Fatalf("wrong color returned, want %v, got %v", test.color, color)
			}
		})
	}
}
