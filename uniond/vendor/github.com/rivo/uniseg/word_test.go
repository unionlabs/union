package uniseg

import "testing"

// Test all official Unicode test cases for word boundaries using the byte slice
// function.
func TestWordCasesBytes(t *testing.T) {
	for testNum, testCase := range wordBreakTestCases {
		/*t.Logf(`Test case %d %q: Expecting %x, getting %x, code points %x"`,
		testNum,
		strings.TrimSpace(testCase.original),
		testCase.expected,
		decomposed(testCase.original),
		[]rune(testCase.original))*/
		var (
			word  []byte
			index int
		)
		state := -1
		b := []byte(testCase.original)
	WordLoop:
		for index = 0; len(b) > 0; index++ {
			if index >= len(testCase.expected) {
				t.Errorf(`Test case %d %q failed: More words %d returned than expected %d`,
					testNum,
					testCase.original,
					index,
					len(testCase.expected))
				break
			}
			word, b, state = FirstWord(b, state)
			cluster := []rune(string(word))
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
					break WordLoop
				}
			}
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

// Test all official Unicode test cases for word boundaries using the string
// function.
func TestWordCasesString(t *testing.T) {
	for testNum, testCase := range wordBreakTestCases {
		/*t.Logf(`Test case %d %q: Expecting %x, getting %x, code points %x"`,
		testNum,
		strings.TrimSpace(testCase.original),
		testCase.expected,
		decomposed(testCase.original),
		[]rune(testCase.original))*/
		var (
			word  string
			index int
		)
		state := -1
		str := testCase.original
	WordLoop:
		for index = 0; len(str) > 0; index++ {
			if index >= len(testCase.expected) {
				t.Errorf(`Test case %d %q failed: More words %d returned than expected %d`,
					testNum,
					testCase.original,
					index,
					len(testCase.expected))
				break
			}
			word, str, state = FirstWordInString(str, state)
			cluster := []rune(string(word))
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
					break WordLoop
				}
			}
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

// Benchmark the use of the word break function for byte slices.
func BenchmarkWordFunctionBytes(b *testing.B) {
	str := []byte(benchmarkStr)
	for i := 0; i < b.N; i++ {
		var c []byte
		state := -1
		for len(str) > 0 {
			c, str, state = FirstWord(str, state)
			resultRunes = []rune(string(c))
		}
	}
}

// Benchmark the use of the word break function for strings.
func BenchmarkWordFunctionString(b *testing.B) {
	str := benchmarkStr
	for i := 0; i < b.N; i++ {
		var c string
		state := -1
		for len(str) > 0 {
			c, str, state = FirstWordInString(str, state)
			resultRunes = []rune(c)
		}
	}
}
