package types

import (
	"encoding/json"
	"errors"
	"fmt"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

type SampleResult struct {
	Value string
}

type responseTest struct {
	id       jsonrpcid
	expected string
}

var responseTests = []responseTest{
	{JSONRPCStringID("1"), `"1"`},
	{JSONRPCStringID("alphabet"), `"alphabet"`},
	{JSONRPCStringID(""), `""`},
	{JSONRPCStringID("àáâ"), `"àáâ"`},
	{JSONRPCIntID(-1), "-1"},
	{JSONRPCIntID(0), "0"},
	{JSONRPCIntID(1), "1"},
	{JSONRPCIntID(100), "100"},
}

func TestResponses(t *testing.T) {
	assert := assert.New(t)
	for _, tt := range responseTests {
		jsonid := tt.id
		a := NewRPCSuccessResponse(jsonid, &SampleResult{"hello"})
		b, err := json.Marshal(a)
		require.NoError(t, err)
		s := fmt.Sprintf(`{"jsonrpc":"2.0","id":%v,"result":{"Value":"hello"}}`, tt.expected)
		assert.Equal(s, string(b))

		d := RPCParseError(errors.New("hello world"))
		e, err := json.Marshal(d)
		require.NoError(t, err)
		f := `{"jsonrpc":"2.0","error":{"code":-32700,"message":"Parse error. Invalid JSON","data":"hello world"}}`
		assert.Equal(f, string(e))

		g := RPCMethodNotFoundError(jsonid)
		h, err := json.Marshal(g)
		require.NoError(t, err)
		i := fmt.Sprintf(`{"jsonrpc":"2.0","id":%v,"error":{"code":-32601,"message":"Method not found"}}`, tt.expected)
		assert.Equal(string(h), i)
	}
}

func TestUnmarshallResponses(t *testing.T) {
	assert := assert.New(t)
	for _, tt := range responseTests {
		response := &RPCResponse{}
		err := json.Unmarshal(
			[]byte(fmt.Sprintf(`{"jsonrpc":"2.0","id":%v,"result":{"Value":"hello"}}`, tt.expected)),
			response,
		)
		require.NoError(t, err)
		a := NewRPCSuccessResponse(tt.id, &SampleResult{"hello"})
		assert.Equal(*response, a)
	}
	response := &RPCResponse{}
	err := json.Unmarshal([]byte(`{"jsonrpc":"2.0","id":true,"result":{"Value":"hello"}}`), response)
	require.Error(t, err)
}

func TestRPCError(t *testing.T) {
	testCases := []struct {
		name     string
		err      *RPCError
		expected string
	}{
		{
			name: "With data",
			err: &RPCError{
				Code:    12,
				Message: "Badness",
				Data:    "One worse than a code 11",
			},
			expected: "RPC error 12 - Badness: One worse than a code 11",
		},
		{
			name: "Without data",
			err: &RPCError{
				Code:    12,
				Message: "Badness",
			},
			expected: "RPC error 12 - Badness",
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			assert.Equal(t, tc.expected, tc.err.Error())
		})
	}
}
