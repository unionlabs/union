package uniseg

import (
	"testing"
)

// widthTestCases is a list of test cases for the calculation of string widths.
var widthTestCases = []struct {
	original string
	expected int
}{
	{"", 0}, // Control
	{"\b", 0},
	{"\x00", 0},
	{"\x05", 0},
	{"\a", 0},
	{"\u000a", 0}, // LF
	{"\u000d", 0}, // CR
	{"\n", 0},
	{"\v", 0},
	{"\f", 0},
	{"\r", 0},
	{"\x0e", 0},
	{"\x0f", 0},
	{"\u0300", 0}, // Extend
	{"\u200d", 0}, // ZERO WIDTH JOINER
	{"a", 1},
	{"\u1b05", 1},     // N
	{"\u2985", 1},     // Na
	{"\U0001F100", 1}, // A
	{"\uff61", 1},     // H
	{"\ufe6a", 2},     // W
	{"\uff01", 2},     // F
	{"\u2e3a", 3},     // TWO-EM DASH
	{"\u2e3b", 4},     // THREE-EM DASH
	{"\u00a9", 1},     // Extended Pictographic (Emoji Presentation = No)
	{"\U0001F60A", 2}, // Extended Pictographic (Emoji Presentation = Yes)
	{"\U0001F1E6", 2}, // Regional Indicator
	{"\u061c\u061c", 0},
	{"\u061c\u000a", 0},
	{"\u061c\u000d", 0},
	{"\u061c\u0300", 0},
	{"\u061c\u200d", 0},
	{"\u061ca", 1},
	{"\u061c\u1b05", 1},
	{"\u061c\u2985", 1},
	{"\u061c\U0001F100", 1},
	{"\u061c\uff61", 1},
	{"\u061c\ufe6a", 2},
	{"\u061c\uff01", 2},
	{"\u061c\u2e3a", 3},
	{"\u061c\u2e3b", 4},
	{"\u061c\u00a9", 1},
	{"\u061c\U0001F60A", 2},
	{"\u061c\U0001F1E6", 2},
	{"\u000a\u061c", 0},
	{"\u000a\u000a", 0},
	{"\u000a\u000d", 0},
	{"\u000a\u0300", 0},
	{"\u000a\u200d", 0},
	{"\u000aa", 1},
	{"\u000a\u1b05", 1},
	{"\u000a\u2985", 1},
	{"\u000a\U0001F100", 1},
	{"\u000a\uff61", 1},
	{"\u000a\ufe6a", 2},
	{"\u000a\uff01", 2},
	{"\u000a\u2e3a", 3},
	{"\u000a\u2e3b", 4},
	{"\u000a\u00a9", 1},
	{"\u000a\U0001F60A", 2},
	{"\u000a\U0001F1E6", 2},
	{"\u000d\u061c", 0},
	{"\u000d\u000a", 0},
	{"\u000d\u000d", 0},
	{"\u000d\u0300", 0},
	{"\u000d\u200d", 0},
	{"\u000da", 1},
	{"\u000d\u1b05", 1},
	{"\u000d\u2985", 1},
	{"\u000d\U0001F100", 1},
	{"\u000d\uff61", 1},
	{"\u000d\ufe6a", 2},
	{"\u000d\uff01", 2},
	{"\u000d\u2e3a", 3},
	{"\u000d\u2e3b", 4},
	{"\u000d\u00a9", 1},
	{"\u000d\U0001F60A", 2},
	{"\u000d\U0001F1E6", 2},
	{"\u0300\u061c", 0},
	{"\u0300\u000a", 0},
	{"\u0300\u000d", 0},
	{"\u0300\u0300", 0},
	{"\u0300\u200d", 0},
	{"\u0300a", 1},
	{"\u0300\u1b05", 1},
	{"\u0300\u2985", 1},
	{"\u0300\U0001F100", 1},
	{"\u0300\uff61", 1},
	{"\u0300\ufe6a", 2},
	{"\u0300\uff01", 2},
	{"\u0300\u2e3a", 3},
	{"\u0300\u2e3b", 4},
	{"\u0300\u00a9", 1},
	{"\u0300\U0001F60A", 2},
	{"\u0300\U0001F1E6", 2},
	{"\u200d\u061c", 0},
	{"\u200d\u000a", 0},
	{"\u200d\u000d", 0},
	{"\u200d\u0300", 0},
	{"\u200d\u200d", 0},
	{"\u200da", 1},
	{"\u200d\u1b05", 1},
	{"\u200d\u2985", 1},
	{"\u200d\U0001F100", 1},
	{"\u200d\uff61", 1},
	{"\u200d\ufe6a", 2},
	{"\u200d\uff01", 2},
	{"\u200d\u2e3a", 3},
	{"\u200d\u2e3b", 4},
	{"\u200d\u00a9", 1},
	{"\u200d\U0001F60A", 2},
	{"\u200d\U0001F1E6", 2},
	{"a\u061c", 1},
	{"a\u000a", 1},
	{"a\u000d", 1},
	{"a\u0300", 1},
	{"a\u200d", 1},
	{"aa", 2},
	{"a\u1b05", 2},
	{"a\u2985", 2},
	{"a\U0001F100", 2},
	{"a\uff61", 2},
	{"a\ufe6a", 3},
	{"a\uff01", 3},
	{"a\u2e3a", 4},
	{"a\u2e3b", 5},
	{"a\u00a9", 2},
	{"a\U0001F60A", 3},
	{"a\U0001F1E6", 3},
	{"\u1b05\u061c", 1},
	{"\u1b05\u000a", 1},
	{"\u1b05\u000d", 1},
	{"\u1b05\u0300", 1},
	{"\u1b05\u200d", 1},
	{"\u1b05a", 2},
	{"\u1b05\u1b05", 2},
	{"\u1b05\u2985", 2},
	{"\u1b05\U0001F100", 2},
	{"\u1b05\uff61", 2},
	{"\u1b05\ufe6a", 3},
	{"\u1b05\uff01", 3},
	{"\u1b05\u2e3a", 4},
	{"\u1b05\u2e3b", 5},
	{"\u1b05\u00a9", 2},
	{"\u1b05\U0001F60A", 3},
	{"\u1b05\U0001F1E6", 3},
	{"\u2985\u061c", 1},
	{"\u2985\u000a", 1},
	{"\u2985\u000d", 1},
	{"\u2985\u0300", 1},
	{"\u2985\u200d", 1},
	{"\u2985a", 2},
	{"\u2985\u1b05", 2},
	{"\u2985\u2985", 2},
	{"\u2985\U0001F100", 2},
	{"\u2985\uff61", 2},
	{"\u2985\ufe6a", 3},
	{"\u2985\uff01", 3},
	{"\u2985\u2e3a", 4},
	{"\u2985\u2e3b", 5},
	{"\u2985\u00a9", 2},
	{"\u2985\U0001F60A", 3},
	{"\u2985\U0001F1E6", 3},
	{"\U0001F100\u061c", 1},
	{"\U0001F100\u000a", 1},
	{"\U0001F100\u000d", 1},
	{"\U0001F100\u0300", 1},
	{"\U0001F100\u200d", 1},
	{"\U0001F100a", 2},
	{"\U0001F100\u1b05", 2},
	{"\U0001F100\u2985", 2},
	{"\U0001F100\U0001F100", 2},
	{"\U0001F100\uff61", 2},
	{"\U0001F100\ufe6a", 3},
	{"\U0001F100\uff01", 3},
	{"\U0001F100\u2e3a", 4},
	{"\U0001F100\u2e3b", 5},
	{"\U0001F100\u00a9", 2},
	{"\U0001F100\U0001F60A", 3},
	{"\U0001F100\U0001F1E6", 3},
	{"\uff61\u061c", 1},
	{"\uff61\u000a", 1},
	{"\uff61\u000d", 1},
	{"\uff61\u0300", 1},
	{"\uff61\u200d", 1},
	{"\uff61a", 2},
	{"\uff61\u1b05", 2},
	{"\uff61\u2985", 2},
	{"\uff61\U0001F100", 2},
	{"\uff61\uff61", 2},
	{"\uff61\ufe6a", 3},
	{"\uff61\uff01", 3},
	{"\uff61\u2e3a", 4},
	{"\uff61\u2e3b", 5},
	{"\uff61\u00a9", 2},
	{"\uff61\U0001F60A", 3},
	{"\uff61\U0001F1E6", 3},
	{"\ufe6a\u061c", 2},
	{"\ufe6a\u000a", 2},
	{"\ufe6a\u000d", 2},
	{"\ufe6a\u0300", 2},
	{"\ufe6a\u200d", 2},
	{"\ufe6aa", 3},
	{"\ufe6a\u1b05", 3},
	{"\ufe6a\u2985", 3},
	{"\ufe6a\U0001F100", 3},
	{"\ufe6a\uff61", 3},
	{"\ufe6a\ufe6a", 4},
	{"\ufe6a\uff01", 4},
	{"\ufe6a\u2e3a", 5},
	{"\ufe6a\u2e3b", 6},
	{"\ufe6a\u00a9", 3},
	{"\ufe6a\U0001F60A", 4},
	{"\ufe6a\U0001F1E6", 4},
	{"\uff01\u061c", 2},
	{"\uff01\u000a", 2},
	{"\uff01\u000d", 2},
	{"\uff01\u0300", 2},
	{"\uff01\u200d", 2},
	{"\uff01a", 3},
	{"\uff01\u1b05", 3},
	{"\uff01\u2985", 3},
	{"\uff01\U0001F100", 3},
	{"\uff01\uff61", 3},
	{"\uff01\ufe6a", 4},
	{"\uff01\uff01", 4},
	{"\uff01\u2e3a", 5},
	{"\uff01\u2e3b", 6},
	{"\uff01\u00a9", 3},
	{"\uff01\U0001F60A", 4},
	{"\uff01\U0001F1E6", 4},
	{"\u2e3a\u061c", 3},
	{"\u2e3a\u000a", 3},
	{"\u2e3a\u000d", 3},
	{"\u2e3a\u0300", 3},
	{"\u2e3a\u200d", 3},
	{"\u2e3aa", 4},
	{"\u2e3a\u1b05", 4},
	{"\u2e3a\u2985", 4},
	{"\u2e3a\U0001F100", 4},
	{"\u2e3a\uff61", 4},
	{"\u2e3a\ufe6a", 5},
	{"\u2e3a\uff01", 5},
	{"\u2e3a\u2e3a", 6},
	{"\u2e3a\u2e3b", 7},
	{"\u2e3a\u00a9", 4},
	{"\u2e3a\U0001F60A", 5},
	{"\u2e3a\U0001F1E6", 5},
	{"\u2e3b\u061c", 4},
	{"\u2e3b\u000a", 4},
	{"\u2e3b\u000d", 4},
	{"\u2e3b\u0300", 4},
	{"\u2e3b\u200d", 4},
	{"\u2e3ba", 5},
	{"\u2e3b\u1b05", 5},
	{"\u2e3b\u2985", 5},
	{"\u2e3b\U0001F100", 5},
	{"\u2e3b\uff61", 5},
	{"\u2e3b\ufe6a", 6},
	{"\u2e3b\uff01", 6},
	{"\u2e3b\u2e3a", 7},
	{"\u2e3b\u2e3b", 8},
	{"\u2e3b\u00a9", 5},
	{"\u2e3b\U0001F60A", 6},
	{"\u2e3b\U0001F1E6", 6},
	{"\u00a9\u061c", 1},
	{"\u00a9\u000a", 1},
	{"\u00a9\u000d", 1},
	{"\u00a9\u0300", 2}, // This is really 1 but we can't handle it.
	{"\u00a9\u200d", 2},
	{"\u00a9a", 2},
	{"\u00a9\u1b05", 2},
	{"\u00a9\u2985", 2},
	{"\u00a9\U0001F100", 2},
	{"\u00a9\uff61", 2},
	{"\u00a9\ufe6a", 3},
	{"\u00a9\uff01", 3},
	{"\u00a9\u2e3a", 4},
	{"\u00a9\u2e3b", 5},
	{"\u00a9\u00a9", 2},
	{"\u00a9\U0001F60A", 3},
	{"\u00a9\U0001F1E6", 3},
	{"\U0001F60A\u061c", 2},
	{"\U0001F60A\u000a", 2},
	{"\U0001F60A\u000d", 2},
	{"\U0001F60A\u0300", 2},
	{"\U0001F60A\u200d", 2},
	{"\U0001F60Aa", 3},
	{"\U0001F60A\u1b05", 3},
	{"\U0001F60A\u2985", 3},
	{"\U0001F60A\U0001F100", 3},
	{"\U0001F60A\uff61", 3},
	{"\U0001F60A\ufe6a", 4},
	{"\U0001F60A\uff01", 4},
	{"\U0001F60A\u2e3a", 5},
	{"\U0001F60A\u2e3b", 6},
	{"\U0001F60A\u00a9", 3},
	{"\U0001F60A\U0001F60A", 4},
	{"\U0001F60A\U0001F1E6", 4},
	{"\U0001F1E6\u061c", 2},
	{"\U0001F1E6\u000a", 2},
	{"\U0001F1E6\u000d", 2},
	{"\U0001F1E6\u0300", 2},
	{"\U0001F1E6\u200d", 2},
	{"\U0001F1E6a", 3},
	{"\U0001F1E6\u1b05", 3},
	{"\U0001F1E6\u2985", 3},
	{"\U0001F1E6\U0001F100", 3},
	{"\U0001F1E6\uff61", 3},
	{"\U0001F1E6\ufe6a", 4},
	{"\U0001F1E6\uff01", 4},
	{"\U0001F1E6\u2e3a", 5},
	{"\U0001F1E6\u2e3b", 6},
	{"\U0001F1E6\u00a9", 3},
	{"\U0001F1E6\U0001F60A", 4},
	{"\U0001F1E6\U0001F1E6", 2},
	{"Ka\u0308se", 4},                       // Käse (German, "cheese")
	{"\U0001f3f3\ufe0f\u200d\U0001f308", 2}, // Rainbow flag
	{"\U0001f1e9\U0001f1ea", 2},             // German flag
	{"\u0916\u093e", 2},                     // खा (Hindi, "eat")
	{"\u0915\u0948\u0938\u0947", 2},         // कैसे (Hindi, "how")
	{"\U0001f468\u200d\U0001f469\u200d\U0001f467\u200d\U0001f466", 2}, // Family: Man, Woman, Girl, Boy
	{"\u1112\u116f\u11b6", 2},                   // 훯 (Hangul, conjoining Jamo, "h+weo+lh")
	{"\ud6ef", 2},                               // 훯 (Hangul, precomposed, "h+weo+lh")
	{"\u79f0\u8c13", 4},                         // 称谓 (Chinese, "title")
	{"\u0e1c\u0e39\u0e49", 1},                   // ผู้ (Thai, "person")
	{"\u0623\u0643\u062a\u0648\u0628\u0631", 6}, // أكتوبر (Arabic, "October")
	{"\ua992\ua997\ua983", 3},                   // ꦒꦗꦃ (Javanese, "elephant")
	{"\u263a", 1},                               // White smiling face
	{"\u263a\ufe0f", 2},                         // White smiling face (with variation selector 16 = emoji presentation)
	{"\u231b", 2},                               // Hourglass
	{"\u231b\ufe0e", 1},                         // Hourglass (with variation selector 15 = text presentation)
	{"1\ufe0f", 2},                              // Emoji presentation of digit one.
}

// String width tests using the StringWidth function.
func TestWidthStringWidth(t *testing.T) {
	for index, testCase := range widthTestCases {
		actual := StringWidth(testCase.original)
		if actual != testCase.expected {
			t.Errorf("StringWidth(%q) is %d, expected %d (test case %d)", testCase.original, actual, testCase.expected, index)
		}
	}
}

// String width tests using the Graphemes class.
func TestWidthGraphemes(t *testing.T) {
	for index, testCase := range widthTestCases {
		var actual int
		graphemes := NewGraphemes(testCase.original)
		for graphemes.Next() {
			actual += graphemes.Width()
		}
		if actual != testCase.expected {
			t.Errorf("Width of %q is %d, expected %d (test case %d)", testCase.original, actual, testCase.expected, index)
		}
	}
}

// String width tests using the FirstGraphemeCluster function.
func TestWidthGraphemesFunctionBytes(t *testing.T) {
	for index, testCase := range widthTestCases {
		var actual, width int
		state := -1
		text := []byte(testCase.original)
		for len(text) > 0 {
			_, text, width, state = FirstGraphemeCluster(text, state)
			actual += width
		}
		if actual != testCase.expected {
			t.Errorf("Width of %q is %d, expected %d (test case %d)", testCase.original, actual, testCase.expected, index)
		}
	}
}

// String width tests using the FirstGraphemeClusterString function.
func TestWidthGraphemesFunctionString(t *testing.T) {
	for index, testCase := range widthTestCases {
		var actual, width int
		state := -1
		text := testCase.original
		for len(text) > 0 {
			_, text, width, state = FirstGraphemeClusterInString(text, state)
			actual += width
		}
		if actual != testCase.expected {
			t.Errorf("Width of %q is %d, expected %d (test case %d)", testCase.original, actual, testCase.expected, index)
		}
	}
}

// String width tests using the Step function.
func TestWidthStepBytes(t *testing.T) {
	for index, testCase := range widthTestCases {
		var actual, boundaries int
		state := -1
		text := []byte(testCase.original)
		for len(text) > 0 {
			_, text, boundaries, state = Step(text, state)
			actual += boundaries >> ShiftWidth
		}
		if actual != testCase.expected {
			t.Errorf("Width of %q is %d, expected %d (test case %d)", testCase.original, actual, testCase.expected, index)
		}
	}
}

// String width tests using the StepString function.
func TestWidthStepString(t *testing.T) {
	for index, testCase := range widthTestCases {
		var actual, boundaries int
		state := -1
		text := testCase.original
		for len(text) > 0 {
			_, text, boundaries, state = StepString(text, state)
			actual += boundaries >> ShiftWidth
		}
		if actual != testCase.expected {
			t.Errorf("Width of %q is %d, expected %d (test case %d)", testCase.original, actual, testCase.expected, index)
		}
	}
}

func TestRunesWidth(t *testing.T) {
	tc := []struct {
		name  string
		raw   string
		width int
	}{
		{"latin    ", "long", 4},
		{"chinese  ", "中国", 4},
		{"combining", "shangha\u0308\u0308i", 8},
		{
			"emoji 1", "🏝",
			1,
		},
		{
			"emoji 2", "🗻",
			2,
		},
		{
			"emoji 3", "🏖",
			1,
		},
		{
			"flags", "🇳🇱🇧🇷i",
			5,
		},
		{
			"flag 2", "🇨🇳",
			2,
		},
	}

	for _, v := range tc {
		graphemes := NewGraphemes(v.raw)
		width := 0
		var rs []rune
		for graphemes.Next() {
			rs = graphemes.Runes()
			width += StringWidth(string(rs))
		}

		if v.width != width {
			t.Logf("%s :\t %q %U\n", v.name, v.raw, rs)
			t.Errorf("%s:\t %q  expect width %d, got %d\n", v.name, v.raw, v.width, width)
		}
	}
}
