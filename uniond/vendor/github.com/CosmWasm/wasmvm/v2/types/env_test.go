package types

import (
	"encoding/json"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestMessageInfoHandlesMultipleCoins(t *testing.T) {
	info := MessageInfo{
		Sender: "foobar",
		Funds: []Coin{
			{Denom: "peth", Amount: "12345"},
			{Denom: "uatom", Amount: "789876"},
		},
	}
	bz, err := json.Marshal(info)
	require.NoError(t, err)

	// we can unmarshal it properly into struct
	var recover MessageInfo
	err = json.Unmarshal(bz, &recover)
	require.NoError(t, err)
	assert.Equal(t, info, recover)
}

func TestMessageInfoHandlesMissingCoins(t *testing.T) {
	info := MessageInfo{
		Sender: "baz",
	}
	bz, err := json.Marshal(info)
	require.NoError(t, err)

	// we can unmarshal it properly into struct
	expected := MessageInfo{
		Sender: "baz",
		Funds:  []Coin{},
	}
	var recover MessageInfo
	err = json.Unmarshal(bz, &recover)
	require.NoError(t, err)
	assert.Equal(t, expected, recover)

	// make sure "funds":[] is in JSON
	var raw map[string]json.RawMessage
	err = json.Unmarshal(bz, &raw)
	require.NoError(t, err)
	funds, ok := raw["funds"]
	require.True(t, ok)
	assert.Equal(t, "[]", string(funds))
}

func TestBlockInfoSerialization(t *testing.T) {
	block := BlockInfo{
		Height:  123,
		Time:    1578939743_987654321,
		ChainID: "foobar",
	}
	bz, err := json.Marshal(block)
	require.NoError(t, err)
	assert.Equal(t, `{"height":123,"time":"1578939743987654321","chain_id":"foobar"}`, string(bz))

	block = BlockInfo{
		Height:  0,
		Time:    0,
		ChainID: "",
	}
	bz, err = json.Marshal(block)
	require.NoError(t, err)
	assert.Equal(t, `{"height":0,"time":"0","chain_id":""}`, string(bz))
}

func TestBlockInfoDeserialization(t *testing.T) {
	var block BlockInfo
	var err error

	// All set
	err = json.Unmarshal([]byte(`{"height":123,"time":"1578939743987654321","chain_id":"foobar"}`), &block)
	require.NoError(t, err)
	assert.Equal(t, BlockInfo{
		Height:  123,
		Time:    1578939743_987654321,
		ChainID: "foobar",
	}, block)

	// All zero
	err = json.Unmarshal([]byte(`{"height":0,"time":"0","chain_id":""}`), &block)
	require.NoError(t, err)
	assert.Equal(t, BlockInfo{
		Height:  0,
		Time:    0,
		ChainID: "",
	}, block)

	// Empty string is not a valid uint64 string
	err = json.Unmarshal([]byte(`{"height":0,"time":"","chain_id":""}`), &block)
	require.ErrorContains(t, err, "cannot unmarshal \"\" into Uint64, failed to parse integer")
}
