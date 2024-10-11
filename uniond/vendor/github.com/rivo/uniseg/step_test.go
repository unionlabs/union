package uniseg

import (
	"testing"
)

// Test official Grapheme Cluster Unicode test cases for grapheme clusters using
// the [Step] function.
func TestStepBytesGrapheme(t *testing.T) {
	for testNum, testCase := range graphemeBreakTestCases {
		/*t.Logf(`Test case %d %q: Expecting %x, getting %x, code points %x"`,
		testNum,
		strings.TrimSpace(testCase.original),
		testCase.expected,
		decomposed(testCase.original),
		[]rune(testCase.original))*/
		b := []byte(testCase.original)
		state := -1
		var (
			index int
			c     []byte
		)
	GraphemeLoop:
		for len(b) > 0 {
			c, b, _, state = Step(b, state)

			if index >= len(testCase.expected) {
				t.Errorf(`Test case %d %q failed: More grapheme clusters returned than expected %d`,
					testNum,
					testCase.original,
					len(testCase.expected))
				break
			}

			cluster := []rune(string(c))
			if len(cluster) != len(testCase.expected[index]) {
				t.Errorf(`Test case %d %q failed: Grapheme cluster at index %d has %d codepoints %x, %d expected %x`,
					testNum,
					testCase.original,
					index,
					len(cluster),
					cluster,
					len(testCase.expected[index]),
					testCase.expected[index])
				break
			}
			for i, r := range cluster {
				if r != testCase.expected[index][i] {
					t.Errorf(`Test case %d %q failed: Grapheme cluster at index %d is %x, expected %x`,
						testNum,
						testCase.original,
						index,
						cluster,
						testCase.expected[index])
					break GraphemeLoop
				}
			}

			index++
		}
		if index < len(testCase.expected) {
			t.Errorf(`Test case %d %q failed: Fewer grapheme clusters returned (%d) than expected (%d)`,
				testNum,
				testCase.original,
				index,
				len(testCase.expected))
		}
	}
}

// Test official word boundaries Unicode test cases for grapheme clusters using
// the [Step] function.
func TestStepBytesWord(t *testing.T) {
	for testNum, testCase := range wordBreakTestCases {
		if testNum == 1700 {
			// This test case reveals an inconsistency in the Unicode rule set,
			// namely the handling of ZWJ within two RI graphemes. (Grapheme
			// rules will restart the RI count, word rules will ignore the ZWJ.)
			// An error has been reported.
			continue
		}
		/*t.Logf(`Test case %d %q: Expecting %x, getting %x, code points %x"`,
		testNum,
		strings.TrimSpace(testCase.original),
		testCase.expected,
		decomposed(testCase.original),
		[]rune(testCase.original))*/
		b := []byte(testCase.original)
		state := -1
		var (
			index, boundaries int
			c                 []byte
			growingCluster    []rune
		)
	GraphemeLoop:
		for len(b) > 0 {
			c, b, boundaries, state = Step(b, state)

			if index >= len(testCase.expected) {
				t.Errorf(`Test case %d %q failed: More words returned than expected %d`,
					testNum,
					testCase.original,
					len(testCase.expected))
				break
			}

			growingCluster = append(growingCluster, []rune(string(c))...)
			if boundaries&MaskWord == 0 {
				continue
			}
			cluster := growingCluster
			growingCluster = nil
			if len(cluster) != len(testCase.expected[index]) {
				t.Errorf(`Test case %d %q failed: Word at index %d has %d codepoints %x, %d expected %x`,
					testNum,
					testCase.original,
					index,
					len(cluster),
					cluster,
					len(testCase.expected[index]),
					testCase.expected[index])
				break
			}
			for i, r := range cluster {
				if r != testCase.expected[index][i] {
					t.Errorf(`Test case %d %q failed: Word at index %d is %x, expected %x`,
						testNum,
						testCase.original,
						index,
						cluster,
						testCase.expected[index])
					break GraphemeLoop
				}
			}

			index++
		}
		if index < len(testCase.expected) {
			t.Errorf(`Test case %d %q failed: Fewer words returned (%d) than expected (%d)`,
				testNum,
				testCase.original,
				index,
				len(testCase.expected))
		}
	}
}

// Test official sentence boundaries Unicode test cases for grapheme clusters
// using the [Step] function.
func TestStepBytesSentence(t *testing.T) {
	for testNum, testCase := range sentenceBreakTestCases {
		/*t.Logf(`Test case %d %q: Expecting %x, getting %x, code points %x"`,
		testNum,
		strings.TrimSpace(testCase.original),
		testCase.expected,
		decomposed(testCase.original),
		[]rune(testCase.original))*/
		b := []byte(testCase.original)
		state := -1
		var (
			index, boundaries int
			c                 []byte
			growingCluster    []rune
		)
	GraphemeLoop:
		for len(b) > 0 {
			c, b, boundaries, state = Step(b, state)

			if index >= len(testCase.expected) {
				t.Errorf(`Test case %d %q failed: More sentences returned than expected %d`,
					testNum,
					testCase.original,
					len(testCase.expected))
				break
			}

			growingCluster = append(growingCluster, []rune(string(c))...)
			if boundaries&MaskSentence == 0 {
				continue
			}
			cluster := growingCluster
			growingCluster = nil
			if len(cluster) != len(testCase.expected[index]) {
				t.Errorf(`Test case %d %q failed: Sentence at index %d has %d codepoints %x, %d expected %x`,
					testNum,
					testCase.original,
					index,
					len(cluster),
					cluster,
					len(testCase.expected[index]),
					testCase.expected[index])
				break
			}
			for i, r := range cluster {
				if r != testCase.expected[index][i] {
					t.Errorf(`Test case %d %q failed: Sentence at index %d is %x, expected %x`,
						testNum,
						testCase.original,
						index,
						cluster,
						testCase.expected[index])
					break GraphemeLoop
				}
			}

			index++
		}
		if index < len(testCase.expected) {
			t.Errorf(`Test case %d %q failed: Fewer sentences returned (%d) than expected (%d)`,
				testNum,
				testCase.original,
				index,
				len(testCase.expected))
		}
	}
}

// We don't test the [Step] function for UAX #14 line breaking because the rules
// aren't really compatible. Specifically emoji modifiers and zero-width joiners
// are kept together by the grapheme cluster rules while line breaking rules
// will allow them to be broken apart. The handling of this limitation is
// outlined in Section 8.2 Example 6 of UAX #14.

// Test official Grapheme Cluster Unicode test cases for grapheme clusters using
// the StepString() function.
func TestStepStringGrapheme(t *testing.T) {
	for testNum, testCase := range graphemeBreakTestCases {
		/*t.Logf(`Test case %d %q: Expecting %x, getting %x, code points %x"`,
		testNum,
		strings.TrimSpace(testCase.original),
		testCase.expected,
		decomposed(testCase.original),
		[]rune(testCase.original))*/
		str := testCase.original
		state := -1
		var (
			index int
			c     string
		)
	GraphemeLoop:
		for len(str) > 0 {
			c, str, _, state = StepString(str, state)

			if index >= len(testCase.expected) {
				t.Errorf(`Test case %d %q failed: More grapheme clusters returned than expected %d`,
					testNum,
					testCase.original,
					len(testCase.expected))
				break
			}

			cluster := []rune(c)
			if len(cluster) != len(testCase.expected[index]) {
				t.Errorf(`Test case %d %q failed: Grapheme cluster at index %d has %d codepoints %x, %d expected %x`,
					testNum,
					testCase.original,
					index,
					len(cluster),
					cluster,
					len(testCase.expected[index]),
					testCase.expected[index])
				break
			}
			for i, r := range cluster {
				if r != testCase.expected[index][i] {
					t.Errorf(`Test case %d %q failed: Grapheme cluster at index %d is %x, expected %x`,
						testNum,
						testCase.original,
						index,
						cluster,
						testCase.expected[index])
					break GraphemeLoop
				}
			}

			index++
		}
		if index < len(testCase.expected) {
			t.Errorf(`Test case %d %q failed: Fewer grapheme clusters returned (%d) than expected (%d)`,
				testNum,
				testCase.original,
				index,
				len(testCase.expected))
		}
	}
}

// Test official word boundaries Unicode test cases for grapheme clusters using
// the StepString() function.
func TestStepStringWord(t *testing.T) {
	for testNum, testCase := range wordBreakTestCases {
		if testNum == 1700 {
			// This test case reveals an inconsistency in the Unicode rule set,
			// namely the handling of ZWJ within two RI graphemes. (Grapheme
			// rules will restart the RI count, word rules will ignore the ZWJ.)
			// An error has been reported.
			continue
		}
		/*t.Logf(`Test case %d %q: Expecting %x, getting %x, code points %x"`,
		testNum,
		strings.TrimSpace(testCase.original),
		testCase.expected,
		decomposed(testCase.original),
		[]rune(testCase.original))*/
		str := testCase.original
		state := -1
		var (
			index, boundaries int
			c                 string
			growingCluster    []rune
		)
	GraphemeLoop:
		for len(str) > 0 {
			c, str, boundaries, state = StepString(str, state)

			if index >= len(testCase.expected) {
				t.Errorf(`Test case %d %q failed: More words returned than expected %d`,
					testNum,
					testCase.original,
					len(testCase.expected))
				break
			}

			growingCluster = append(growingCluster, []rune(c)...)
			if boundaries&MaskWord == 0 {
				continue
			}
			cluster := growingCluster
			growingCluster = nil
			if len(cluster) != len(testCase.expected[index]) {
				t.Errorf(`Test case %d %q failed: Word at index %d has %d codepoints %x, %d expected %x`,
					testNum,
					testCase.original,
					index,
					len(cluster),
					cluster,
					len(testCase.expected[index]),
					testCase.expected[index])
				break
			}
			for i, r := range cluster {
				if r != testCase.expected[index][i] {
					t.Errorf(`Test case %d %q failed: Word at index %d is %x, expected %x`,
						testNum,
						testCase.original,
						index,
						cluster,
						testCase.expected[index])
					break GraphemeLoop
				}
			}

			index++
		}
		if index < len(testCase.expected) {
			t.Errorf(`Test case %d %q failed: Fewer words returned (%d) than expected (%d)`,
				testNum,
				testCase.original,
				index,
				len(testCase.expected))
		}
	}
}

// Test official sentence boundaries Unicode test cases for grapheme clusters
// using the StepString() function.
func TestStepStringSentence(t *testing.T) {
	for testNum, testCase := range sentenceBreakTestCases {
		/*t.Logf(`Test case %d %q: Expecting %x, getting %x, code points %x"`,
		testNum,
		strings.TrimSpace(testCase.original),
		testCase.expected,
		decomposed(testCase.original),
		[]rune(testCase.original))*/
		str := testCase.original
		state := -1
		var (
			index, boundaries int
			c                 string
			growingCluster    []rune
		)
	GraphemeLoop:
		for len(str) > 0 {
			c, str, boundaries, state = StepString(str, state)

			if index >= len(testCase.expected) {
				t.Errorf(`Test case %d %q failed: More sentences returned than expected %d`,
					testNum,
					testCase.original,
					len(testCase.expected))
				break
			}

			growingCluster = append(growingCluster, []rune(c)...)
			if boundaries&MaskSentence == 0 {
				continue
			}
			cluster := growingCluster
			growingCluster = nil
			if len(cluster) != len(testCase.expected[index]) {
				t.Errorf(`Test case %d %q failed: Sentence at index %d has %d codepoints %x, %d expected %x`,
					testNum,
					testCase.original,
					index,
					len(cluster),
					cluster,
					len(testCase.expected[index]),
					testCase.expected[index])
				break
			}
			for i, r := range cluster {
				if r != testCase.expected[index][i] {
					t.Errorf(`Test case %d %q failed: Sentence at index %d is %x, expected %x`,
						testNum,
						testCase.original,
						index,
						cluster,
						testCase.expected[index])
					break GraphemeLoop
				}
			}

			index++
		}
		if index < len(testCase.expected) {
			t.Errorf(`Test case %d %q failed: Fewer sentences returned (%d) than expected (%d)`,
				testNum,
				testCase.original,
				index,
				len(testCase.expected))
		}
	}
}

// Benchmark the use of the [Step] function.
func BenchmarkStepBytes(b *testing.B) {
	str := []byte(benchmarkStr)
	for i := 0; i < b.N; i++ {
		var c []byte
		state := -1
		for len(str) > 0 {
			c, str, _, state = Step(str, state)
			resultRunes = []rune(string(c))
		}
	}
}

// Benchmark the use of the StepString() function.
func BenchmarkStepString(b *testing.B) {
	str := benchmarkStr
	for i := 0; i < b.N; i++ {
		var c string
		state := -1
		for len(str) > 0 {
			c, str, _, state = StepString(str, state)
			resultRunes = []rune(c)
		}
	}
}

// Fuzz the StepString function.
func FuzzStepString(f *testing.F) {
	for _, tc := range graphemeBreakTestCases {
		f.Add(tc.original)
	}
	f.Fuzz(func(t *testing.T, orig string) {
		var (
			c          string
			b          []byte
			boundaries int
		)
		str := orig
		state := -1
		for len(str) > 0 {
			c, str, boundaries, state = StepString(str, state)
			b = append(b, []byte(c)...)
		}

		// Check if the constructed string is the same as the original.
		if string(b) != orig {
			t.Errorf("Fuzzing failed: %q != %q", string(b), orig)
		}

		// For all other checks, we need to have a non-empty string.
		if orig == "" {
			return
		}

		// Check end boundaries.
		if boundaries&MaskWord == 0 {
			t.Errorf("String %q does not end on a word boundary (final boundary = %x)", orig, state)
		}
		if boundaries&MaskSentence == 0 {
			t.Errorf("String %q does not end on a sentence boundary (final boundary = %x)", orig, state)
		}
		if boundaries&MaskLine != LineMustBreak {
			t.Errorf("String %q does not end with a mandatory line break (final boundary = %x)", orig, state)
		}

		// Note: If you have ideas for more useful checks we could add here,
		// please submit them here:
		// https://github.com/rivo/uniseg/issues
	})
}
