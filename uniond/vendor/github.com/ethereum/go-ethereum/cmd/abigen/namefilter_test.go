package main

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestNameFilter(t *testing.T) {
	_, err := newNameFilter("Foo")
	require.Error(t, err)
	_, err = newNameFilter("too/many:colons:Foo")
	require.Error(t, err)

	f, err := newNameFilter("a/path:A", "*:B", "c/path:*")
	require.NoError(t, err)

	for _, tt := range []struct {
		name  string
		match bool
	}{
		{"a/path:A", true},
		{"unknown/path:A", false},
		{"a/path:X", false},
		{"unknown/path:X", false},
		{"any/path:B", true},
		{"c/path:X", true},
		{"c/path:foo:B", false},
	} {
		match := f.Matches(tt.name)
		if tt.match {
			assert.True(t, match, "expected match")
		} else {
			assert.False(t, match, "expected no match")
		}
	}
}
