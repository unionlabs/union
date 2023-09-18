package utils

import (
	"encoding/json"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestNewParamChangeJSON(t *testing.T) {
	pcj := NewParamChangeJSON("subspace", "key", json.RawMessage(`{}`))
	require.Equal(t, "subspace", pcj.Subspace)
	require.Equal(t, "key", pcj.Key)
	require.Equal(t, json.RawMessage(`{}`), pcj.Value)
}

func TestToParamChanges(t *testing.T) {
	pcj1 := NewParamChangeJSON("subspace", "key1", json.RawMessage(`{}`))
	pcj2 := NewParamChangeJSON("subspace", "key2", json.RawMessage(`{}`))
	pcjs := ParamChangesJSON{pcj1, pcj2}

	paramChanges := pcjs.ToParamChanges()
	require.Len(t, paramChanges, 2)

	require.Equal(t, paramChanges[0].Subspace, pcj1.Subspace)
	require.Equal(t, paramChanges[0].Key, pcj1.Key)
	require.Equal(t, paramChanges[0].Value, string(pcj1.Value))

	require.Equal(t, paramChanges[1].Subspace, pcj2.Subspace)
	require.Equal(t, paramChanges[1].Key, pcj2.Key)
	require.Equal(t, paramChanges[1].Value, string(pcj2.Value))
}
