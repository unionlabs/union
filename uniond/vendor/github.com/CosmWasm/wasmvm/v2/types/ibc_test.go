package types

import (
	"encoding/json"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestIbcTimeoutSerialization(t *testing.T) {
	// All set
	timeout := IBCTimeout{
		Block: &IBCTimeoutBlock{
			Revision: 17,
			Height:   42,
		},
		Timestamp: 1578939743_987654321,
	}
	bz, err := json.Marshal(timeout)
	require.NoError(t, err)
	assert.Equal(t, `{"block":{"revision":17,"height":42},"timestamp":"1578939743987654321"}`, string(bz))

	// Null block
	timeout = IBCTimeout{
		Block:     nil,
		Timestamp: 1578939743_987654321,
	}
	bz, err = json.Marshal(timeout)
	require.NoError(t, err)
	assert.Equal(t, `{"block":null,"timestamp":"1578939743987654321"}`, string(bz))

	// Null timestamp
	// This should be `"timestamp":null`, but we are lacking this feature: https://github.com/golang/go/issues/37711
	// However, this is good enough right now because in Rust a missing field is deserialized as `None` into `Option<Timestamp>`
	timeout = IBCTimeout{
		Block: &IBCTimeoutBlock{
			Revision: 17,
			Height:   42,
		},
		Timestamp: 0,
	}
	bz, err = json.Marshal(timeout)
	require.NoError(t, err)
	assert.Equal(t, `{"block":{"revision":17,"height":42}}`, string(bz))
}

func TestIbcTimeoutDeserialization(t *testing.T) {
	var err error

	// All set
	var timeout1 IBCTimeout
	err = json.Unmarshal([]byte(`{"block":{"revision":17,"height":42},"timestamp":"1578939743987654321"}`), &timeout1)
	require.NoError(t, err)
	assert.Equal(t, IBCTimeout{
		Block: &IBCTimeoutBlock{
			Revision: 17,
			Height:   42,
		},
		Timestamp: 1578939743_987654321,
	}, timeout1)

	// Null block
	var timeout2 IBCTimeout
	err = json.Unmarshal([]byte(`{"block":null,"timestamp":"1578939743987654321"}`), &timeout2)
	require.NoError(t, err)
	assert.Equal(t, IBCTimeout{
		Block:     nil,
		Timestamp: 1578939743_987654321,
	}, timeout2)

	// Null timestamp
	var timeout3 IBCTimeout
	err = json.Unmarshal([]byte(`{"block":{"revision":17,"height":42},"timestamp":null}`), &timeout3)
	require.NoError(t, err)
	assert.Equal(t, IBCTimeout{
		Block: &IBCTimeoutBlock{
			Revision: 17,
			Height:   42,
		},
		Timestamp: 0,
	}, timeout3)

	// Zero timestamp
	// This is not very useful but something a contract developer can do in the current type
	// system by setting timestamp to IbcTimeout::with_timestamp(Timestamp::from_nanos(0))
	var timeout4 IBCTimeout
	err = json.Unmarshal([]byte(`{"block":null,"timestamp":"0"}`), &timeout4)
	require.NoError(t, err)
	assert.Equal(t, IBCTimeout{
		Block:     nil,
		Timestamp: 0,
	}, timeout4)
}

func TestIbcReceiveResponseDeserialization(t *testing.T) {
	var err error

	// without acknowledgement
	var resp IBCReceiveResponse
	err = json.Unmarshal([]byte(`{"acknowledgement":null,"messages":[],"attributes":[],"events":[]}`), &resp)
	require.NoError(t, err)
	assert.Equal(t, IBCReceiveResponse{
		Acknowledgement: nil,
		Messages:        []SubMsg{},
		Attributes:      []EventAttribute{},
		Events:          []Event{},
	}, resp)

	// with acknowledgement
	err = json.Unmarshal([]byte(`{"acknowledgement":"YWNr","messages":[],"attributes":[],"events":[]}`), &resp)
	require.NoError(t, err)
	assert.Equal(t, IBCReceiveResponse{
		Acknowledgement: []byte("ack"),
		Messages:        []SubMsg{},
		Attributes:      []EventAttribute{},
		Events:          []Event{},
	}, resp)
}
