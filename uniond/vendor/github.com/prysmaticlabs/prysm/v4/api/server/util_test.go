package server

import (
	"testing"

	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestNormalizeQueryValues(t *testing.T) {
	input := make(map[string][]string)
	input["key"] = []string{"value1", "value2,value3,value4", "value5"}

	NormalizeQueryValues(input)
	require.Equal(t, 5, len(input["key"]))
	assert.Equal(t, "value1", input["key"][0])
	assert.Equal(t, "value2", input["key"][1])
	assert.Equal(t, "value3", input["key"][2])
	assert.Equal(t, "value4", input["key"][3])
	assert.Equal(t, "value5", input["key"][4])
}
