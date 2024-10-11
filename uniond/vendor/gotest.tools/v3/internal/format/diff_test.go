package format_test

import (
	"testing"

	"gotest.tools/v3/assert"
	"gotest.tools/v3/golden"
	"gotest.tools/v3/internal/format"
)

func TestUnifiedDiff(t *testing.T) {
	var testcases = []struct {
		name     string
		a        string
		b        string
		expected string
		from     string
		to       string
	}{
		{
			name: "empty diff",
			a:    "a\nb\nc",
			b:    "a\nb\nc",
			from: "from",
			to:   "to",
		},
		{
			name:     "one diff with header",
			a:        "a\nxyz\nc",
			b:        "a\nb\nc",
			from:     "from",
			to:       "to",
			expected: "one-diff-with-header.golden",
		},
		{
			name:     "many diffs",
			a:        "a123\nxyz\nc\nbaba\nz\nt\nj2j2\nok\nok\ndone\n",
			b:        "a123\nxyz\nc\nabab\nz\nt\nj2j2\nok\nok\n",
			expected: "many-diff.golden",
		},
		{
			name:     "no trailing newline",
			a:        "a123\nxyz\nc\nbaba\nz\nt\nj2j2\nok\nok\ndone\n",
			b:        "a123\nxyz\nc\nabab\nz\nt\nj2j2\nok\nok",
			expected: "many-diff-no-trailing-newline.golden",
		},
		{
			name:     "whitespace diff",
			a:        "  something\n      something\n    \v\r\n",
			b:        "  something\n\tsomething\n  \n",
			expected: "whitespace-diff.golden",
		},
	}

	for _, testcase := range testcases {
		t.Run(testcase.name, func(t *testing.T) {
			diff := format.UnifiedDiff(format.DiffConfig{
				A:    testcase.a,
				B:    testcase.b,
				From: testcase.from,
				To:   testcase.to,
			})

			if testcase.expected != "" {
				assert.Assert(t, golden.String(diff, testcase.expected))
				return
			}
			assert.Equal(t, diff, "")
		})
	}
}
