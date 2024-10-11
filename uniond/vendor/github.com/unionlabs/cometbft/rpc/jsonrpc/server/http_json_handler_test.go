package server

import (
	"bytes"
	"encoding/json"
	"io"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/cometbft/cometbft/libs/log"
	"github.com/cometbft/cometbft/rpc/jsonrpc/types"
)

func testMux() *http.ServeMux {
	funcMap := map[string]*RPCFunc{
		"c":     NewRPCFunc(func(_ *types.Context, _ string, _ int) (string, error) { return "foo", nil }, "s,i"),
		"block": NewRPCFunc(func(_ *types.Context, _ int) (string, error) { return "block", nil }, "height", Cacheable("height")),
	}
	mux := http.NewServeMux()
	buf := new(bytes.Buffer)
	logger := log.NewTMLogger(buf)
	RegisterRPCFuncs(mux, funcMap, logger)

	return mux
}

func statusOK(code int) bool { return code >= 200 && code <= 299 }

// Ensure that nefarious/unintended inputs to `params`
// do not crash our RPC handlers.
// See Issue https://github.com/tendermint/tendermint/issues/708.
func TestRPCParams(t *testing.T) {
	mux := testMux()
	tests := []struct {
		payload    string
		wantErr    string
		expectedID any
	}{
		// bad
		{`{"jsonrpc": "2.0", "id": "0"}`, "Method not found", types.JSONRPCStringID("0")},
		{`{"jsonrpc": "2.0", "method": "y", "id": "0"}`, "Method not found", types.JSONRPCStringID("0")},
		// id not captured in JSON parsing failures
		{`{"method": "c", "id": "0", "params": a}`, "invalid character", nil},
		{`{"method": "c", "id": "0", "params": ["a"]}`, "got 1", types.JSONRPCStringID("0")},
		{`{"method": "c", "id": "0", "params": ["a", "b"]}`, "invalid character", types.JSONRPCStringID("0")},
		{`{"method": "c", "id": "0", "params": [1, 1]}`, "of type string", types.JSONRPCStringID("0")},

		// no ID - notification
		// {`{"jsonrpc": "2.0", "method": "c", "params": ["a", "10"]}`, false, nil},

		// good
		{`{"jsonrpc": "2.0", "method": "c", "id": "0", "params": null}`, "", types.JSONRPCStringID("0")},
		{`{"method": "c", "id": "0", "params": {}}`, "", types.JSONRPCStringID("0")},
		{`{"method": "c", "id": "0", "params": ["a", "10"]}`, "", types.JSONRPCStringID("0")},
	}

	for i, tt := range tests {
		req, _ := http.NewRequest(http.MethodPost, "http://localhost/", strings.NewReader(tt.payload))
		rec := httptest.NewRecorder()
		mux.ServeHTTP(rec, req)
		res := rec.Result()
		defer res.Body.Close()
		// Always expecting back a JSONRPCResponse
		assert.NotZero(t, res.StatusCode, "#%d: should always return code", i)
		blob, err := io.ReadAll(res.Body)
		if err != nil {
			t.Errorf("#%d: err reading body: %v", i, err)
			continue
		}

		recv := new(types.RPCResponse)
		require.NoError(t, json.Unmarshal(blob, recv), "#%d: expecting successful parsing of an RPCResponse:\nblob: %s", i, blob)
		assert.NotEqual(t, recv, new(types.RPCResponse), "#%d: not expecting a blank RPCResponse", i)
		assert.Equal(t, tt.expectedID, recv.ID, "#%d: expected ID not matched in RPCResponse", i)
		if tt.wantErr == "" {
			assert.Nil(t, recv.Error, "#%d: not expecting an error", i)
		} else {
			assert.Less(t, recv.Error.Code, 0, "#%d: not expecting a positive JSONRPC code", i)
			// The wanted error is either in the message or the data
			assert.Contains(t, recv.Error.Message+recv.Error.Data, tt.wantErr, "#%d: expected substring", i)
		}
	}
}

func TestJSONRPCID(t *testing.T) {
	mux := testMux()
	tests := []struct {
		payload    string
		wantErr    bool
		expectedID any
	}{
		// good id
		{`{"jsonrpc": "2.0", "method": "c", "id": "0", "params": ["a", "10"]}`, false, types.JSONRPCStringID("0")},
		{`{"jsonrpc": "2.0", "method": "c", "id": "abc", "params": ["a", "10"]}`, false, types.JSONRPCStringID("abc")},
		{`{"jsonrpc": "2.0", "method": "c", "id": 0, "params": ["a", "10"]}`, false, types.JSONRPCIntID(0)},
		{`{"jsonrpc": "2.0", "method": "c", "id": 1, "params": ["a", "10"]}`, false, types.JSONRPCIntID(1)},
		{`{"jsonrpc": "2.0", "method": "c", "id": 1.3, "params": ["a", "10"]}`, false, types.JSONRPCIntID(1)},
		{`{"jsonrpc": "2.0", "method": "c", "id": -1, "params": ["a", "10"]}`, false, types.JSONRPCIntID(-1)},

		// bad id
		{`{"jsonrpc": "2.0", "method": "c", "id": {}, "params": ["a", "10"]}`, true, nil},
		{`{"jsonrpc": "2.0", "method": "c", "id": [], "params": ["a", "10"]}`, true, nil},
	}

	for i, tt := range tests {
		req, _ := http.NewRequest(http.MethodPost, "http://localhost/", strings.NewReader(tt.payload))
		rec := httptest.NewRecorder()
		mux.ServeHTTP(rec, req)
		res := rec.Result()
		// Always expecting back a JSONRPCResponse
		assert.NotZero(t, res.StatusCode, "#%d: should always return code", i)
		blob, err := io.ReadAll(res.Body)
		if err != nil {
			t.Errorf("#%d: err reading body: %v", i, err)
			continue
		}
		res.Body.Close()

		recv := new(types.RPCResponse)
		err = json.Unmarshal(blob, recv)
		require.NoError(t, err, "#%d: expecting successful parsing of an RPCResponse:\nblob: %s", i, blob)
		if !tt.wantErr {
			assert.NotEqual(t, recv, new(types.RPCResponse), "#%d: not expecting a blank RPCResponse", i)
			assert.Equal(t, tt.expectedID, recv.ID, "#%d: expected ID not matched in RPCResponse", i)
			assert.Nil(t, recv.Error, "#%d: not expecting an error", i)
		} else {
			assert.Less(t, recv.Error.Code, 0, "#%d: not expecting a positive JSONRPC code", i)
		}
	}
}

func TestRPCNotification(t *testing.T) {
	mux := testMux()
	body := strings.NewReader(`{"jsonrpc": "2.0"}`)
	req, _ := http.NewRequest(http.MethodPost, "http://localhost/", body)
	rec := httptest.NewRecorder()
	mux.ServeHTTP(rec, req)
	res := rec.Result()

	// Always expecting back a JSONRPCResponse
	require.True(t, statusOK(res.StatusCode), "should always return 2XX")
	blob, err := io.ReadAll(res.Body)
	res.Body.Close()
	require.NoError(t, err, "reading from the body should not give back an error")
	require.Empty(t, blob, "a notification SHOULD NOT be responded to by the server")
}

func TestRPCNotificationInBatch(t *testing.T) {
	mux := testMux()
	tests := []struct {
		payload     string
		expectCount int
	}{
		{
			`[
				{"jsonrpc": "2.0"},
				{"jsonrpc": "2.0","method":"c","id":"abc","params":["a","10"]}
			 ]`,
			1,
		},
		{
			`[
				{"jsonrpc": "2.0"},
				{"jsonrpc": "2.0","method":"c","id":"abc","params":["a","10"]},
				{"jsonrpc": "2.0"},
				{"jsonrpc": "2.0","method":"c","id":"abc","params":["a","10"]}
			 ]`,
			2,
		},
	}
	for i, tt := range tests {
		req, _ := http.NewRequest(http.MethodPost, "http://localhost/", strings.NewReader(tt.payload))
		rec := httptest.NewRecorder()
		mux.ServeHTTP(rec, req)
		res := rec.Result()
		// Always expecting back a JSONRPCResponse
		assert.True(t, statusOK(res.StatusCode), "#%d: should always return 2XX", i)
		blob, err := io.ReadAll(res.Body)
		if err != nil {
			t.Errorf("#%d: err reading body: %v", i, err)
			continue
		}
		res.Body.Close()

		var responses []types.RPCResponse
		// try to unmarshal an array first
		err = json.Unmarshal(blob, &responses)
		if err != nil {
			// if we were actually expecting an array, but got an error
			if tt.expectCount > 1 {
				t.Errorf("#%d: expected an array, couldn't unmarshal it\nblob: %s", i, blob)
				continue
			}
			// we were expecting an error here, so let's unmarshal a single response
			var response types.RPCResponse
			err = json.Unmarshal(blob, &response)
			if err != nil {
				t.Errorf("#%d: expected successful parsing of an RPCResponse\nblob: %s", i, blob)
				continue
			}
			// have a single-element result
			responses = []types.RPCResponse{response}
		}
		if tt.expectCount != len(responses) {
			t.Errorf("#%d: expected %d response(s), but got %d\nblob: %s", i, tt.expectCount, len(responses), blob)
			continue
		}
		for _, response := range responses {
			assert.NotEqual(t, response, new(types.RPCResponse), "#%d: not expecting a blank RPCResponse", i)
		}
	}
}

func TestUnknownRPCPath(t *testing.T) {
	mux := testMux()
	req, _ := http.NewRequest(http.MethodGet, "http://localhost/unknownrpcpath", nil)
	rec := httptest.NewRecorder()
	mux.ServeHTTP(rec, req)
	res := rec.Result()

	// Always expecting back a 404 error
	require.Equal(t, http.StatusNotFound, res.StatusCode, "should always return 404")
	res.Body.Close()
}

func TestRPCResponseCache(t *testing.T) {
	mux := testMux()
	body := strings.NewReader(`{"jsonrpc": "2.0","method":"block","id": 0, "params": ["1"]}`)
	req, _ := http.NewRequest(http.MethodGet, "http://localhost/", body)
	rec := httptest.NewRecorder()
	mux.ServeHTTP(rec, req)
	res := rec.Result()

	// Always expecting back a JSONRPCResponse
	require.True(t, statusOK(res.StatusCode), "should always return 2XX")
	require.Equal(t, "public, max-age=86400", res.Header.Get("Cache-Control"))

	_, err := io.ReadAll(res.Body)
	res.Body.Close()
	require.NoError(t, err, "reading from the body should not give back an error")

	// send a request with default height.
	body = strings.NewReader(`{"jsonrpc": "2.0","method":"block","id": 0, "params": ["0"]}`)
	req, _ = http.NewRequest(http.MethodGet, "http://localhost/", body)
	rec = httptest.NewRecorder()
	mux.ServeHTTP(rec, req)
	res = rec.Result()

	// Always expecting back a JSONRPCResponse
	require.True(t, statusOK(res.StatusCode), "should always return 2XX")
	require.Equal(t, "", res.Header.Get("Cache-Control"))

	_, err = io.ReadAll(res.Body)

	res.Body.Close()
	require.NoError(t, err, "reading from the body should not give back an error")

	// send a request with default height, but as empty set of parameters.
	body = strings.NewReader(`{"jsonrpc": "2.0","method":"block","id": 0, "params": []}`)
	req, _ = http.NewRequest(http.MethodGet, "http://localhost/", body)
	rec = httptest.NewRecorder()
	mux.ServeHTTP(rec, req)
	res = rec.Result()

	// Always expecting back a JSONRPCResponse
	require.True(t, statusOK(res.StatusCode), "should always return 2XX")
	require.Equal(t, "", res.Header.Get("Cache-Control"))

	_, err = io.ReadAll(res.Body)

	res.Body.Close()
	require.NoError(t, err, "reading from the body should not give back an error")
}
