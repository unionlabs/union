package main

import (
	"bytes"
	"fmt"
	"io"
	"strings"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestConvert(t *testing.T) {
	examples := []struct {
		name     string
		input    io.Reader
		expected string
		errors   bool
	}{
		{
			name: "valid toml",
			input: strings.NewReader(`
[mytoml]
a = 42`),
			expected: `{
  "mytoml": {
    "a": 42
  }
}
`,
		},
		{
			name:   "invalid toml",
			input:  strings.NewReader(`bad = []]`),
			errors: true,
		},
		{
			name:   "bad reader",
			input:  &badReader{},
			errors: true,
		},
	}

	for _, e := range examples {
		b := new(bytes.Buffer)
		err := convert(e.input, b)
		if e.errors {
			require.Error(t, err)
		} else {
			assert.NoError(t, err)
			assert.Equal(t, e.expected, b.String())
		}
	}
}

type badReader struct{}

func (r *badReader) Read([]byte) (int, error) {
	return 0, fmt.Errorf("reader failed on purpose")
}
