package format_test

import (
	"testing"

	"gotest.tools/v3/assert"
	"gotest.tools/v3/internal/format"
)

func TestMessage(t *testing.T) {
	var testcases = []struct {
		doc      string
		args     []interface{}
		expected string
	}{
		{
			doc: "none",
		},
		{
			doc:      "single string",
			args:     args("foo"),
			expected: "foo",
		},
		{
			doc:      "single non-string",
			args:     args(123),
			expected: "123",
		},
		{
			doc:      "format string and args",
			args:     args("%s %v", "a", 3),
			expected: "a 3",
		},
	}

	for _, tc := range testcases {
		t.Run(tc.doc, func(t *testing.T) {
			assert.Equal(t, format.Message(tc.args...), tc.expected)
		})
	}
}

func args(a ...interface{}) []interface{} {
	return a
}

func TestWithCustomMessage(t *testing.T) {
	t.Run("only custom", func(t *testing.T) {
		msg := format.WithCustomMessage("", "extra")
		assert.Equal(t, msg, "extra")
	})

	t.Run("only source", func(t *testing.T) {
		msg := format.WithCustomMessage("source")
		assert.Equal(t, msg, "source")
	})

	t.Run("source and custom", func(t *testing.T) {
		msg := format.WithCustomMessage("source", "extra")
		assert.Equal(t, msg, "source: extra")
	})
}
