package gotenv

import (
	"bufio"
	"strings"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestScanner(t *testing.T) {
	type testCase struct {
		name string
		in   string
		exp  []string
	}

	testCases := []testCase{
		{
			"regular LF split with trailing LF",
			"aa\nbb\ncc\n",
			[]string{"aa", "bb", "cc", ""},
		},
		{
			"regular LF split with no trailing LF",
			"aa\nbb\ncc",
			[]string{"aa", "bb", "cc"},
		},

		{
			"regular CR split with trailing CR",
			"aa\rbb\rcc\r",
			[]string{"aa", "bb", "cc", ""},
		},
		{
			"regular CR split with no trailing CR",
			"aa\rbb\rcc",
			[]string{"aa", "bb", "cc"},
		},

		{
			"regular CRLF split with trailing CRLF",
			"aa\r\nbb\r\ncc\r\n",
			[]string{"aa", "bb", "cc", ""},
		},
		{
			"regular CRLF split with no trailing CRLF",
			"aa\r\nbb\r\ncc",
			[]string{"aa", "bb", "cc"},
		},

		{
			"mix of possible line endings",
			"aa\r\nbb\ncc\rdd",
			[]string{"aa", "bb", "cc", "dd"},
		},
	}

	for _, tc := range testCases {
		s := bufio.NewScanner(strings.NewReader(tc.in))
		s.Split(splitLines)

		i := 0
		for s.Scan() {
			if i >= len(tc.exp) {
				assert.Fail(t, "unexpected line", "testCase: %s - got extra line: %q", tc.name, s.Text())
			} else {
				got := s.Text()
				assert.Equal(t, tc.exp[i], got, "testCase: %s - line %d", tc.name, i)
			}
			i++
		}

		assert.NoError(t, s.Err(), "testCase: %s", tc.name)
		assert.Equal(t, len(tc.exp), i, "testCase: %s - expected to have the correct line count", tc.name)
	}
}
