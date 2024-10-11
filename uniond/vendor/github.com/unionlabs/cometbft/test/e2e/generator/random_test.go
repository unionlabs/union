package main

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestCombinations(t *testing.T) {
	input := map[string][]any{
		"bool":   {false, true},
		"int":    {1, 2, 3},
		"string": {"foo", "bar"},
	}

	c := combinations(input)
	assert.Equal(t, []map[string]any{
		{"bool": false, "int": 1, "string": "foo"},
		{"bool": false, "int": 1, "string": "bar"},
		{"bool": false, "int": 2, "string": "foo"},
		{"bool": false, "int": 2, "string": "bar"},
		{"bool": false, "int": 3, "string": "foo"},
		{"bool": false, "int": 3, "string": "bar"},
		{"bool": true, "int": 1, "string": "foo"},
		{"bool": true, "int": 1, "string": "bar"},
		{"bool": true, "int": 2, "string": "foo"},
		{"bool": true, "int": 2, "string": "bar"},
		{"bool": true, "int": 3, "string": "foo"},
		{"bool": true, "int": 3, "string": "bar"},
	}, c)
}
