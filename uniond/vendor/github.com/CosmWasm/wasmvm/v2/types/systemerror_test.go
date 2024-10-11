package types

import (
	"encoding/json"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestSystemErrorNoSuchContractSerialization(t *testing.T) {
	// Deserializaton
	document := []byte(`{"no_such_contract":{"addr":"nada"}}`)
	var se SystemError
	err := json.Unmarshal(document, &se)
	require.NoError(t, err)
	require.Equal(t, SystemError{
		NoSuchContract: &NoSuchContract{
			Addr: "nada",
		},
	}, se)

	// Serialization
	mySE := SystemError{
		NoSuchContract: &NoSuchContract{
			Addr: "404",
		},
	}
	serialized, err := json.Marshal(&mySE)
	require.NoError(t, err)
	require.Equal(t, `{"no_such_contract":{"addr":"404"}}`, string(serialized))
}

func TestSystemErrorNoSuchCodeSerialization(t *testing.T) {
	// Deserializaton
	document := []byte(`{"no_such_code":{"code_id":987}}`)
	var se SystemError
	err := json.Unmarshal(document, &se)
	require.NoError(t, err)
	require.Equal(t, SystemError{
		NoSuchCode: &NoSuchCode{
			CodeID: uint64(987),
		},
	}, se)

	// Serialization
	mySE := SystemError{
		NoSuchCode: &NoSuchCode{
			CodeID: uint64(321),
		},
	}
	serialized, err := json.Marshal(&mySE)
	require.NoError(t, err)
	require.Equal(t, `{"no_such_code":{"code_id":321}}`, string(serialized))
}
