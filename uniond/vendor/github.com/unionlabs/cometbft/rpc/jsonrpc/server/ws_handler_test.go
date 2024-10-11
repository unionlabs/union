package server

import (
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/gorilla/websocket"
	"github.com/stretchr/testify/require"

	"github.com/cometbft/cometbft/libs/log"
	"github.com/cometbft/cometbft/rpc/jsonrpc/types"
)

func TestWebsocketManagerHandler(t *testing.T) {
	s := newWSServer()
	defer s.Close()

	for _, ep := range []string{"/websocket", "/v1/websocket"} {
		// check upgrader works
		d := websocket.Dialer{}
		c, dialResp, err := d.Dial("ws://"+s.Listener.Addr().String()+ep, nil)
		require.NoError(t, err)

		if got, want := dialResp.StatusCode, http.StatusSwitchingProtocols; got != want {
			t.Errorf("dialResp.StatusCode = %q, want %q", got, want)
		}

		// check basic functionality works
		req, err := types.MapToRequest(
			types.JSONRPCStringID("TestWebsocketManager"),
			"c",
			map[string]any{"s": "a", "i": 10},
		)
		require.NoError(t, err)
		err = c.WriteJSON(req)
		require.NoError(t, err)

		var resp types.RPCResponse
		err = c.ReadJSON(&resp)
		require.NoError(t, err)
		require.Nil(t, resp.Error)
		dialResp.Body.Close()
	}
}

func newWSServer() *httptest.Server {
	funcMap := map[string]*RPCFunc{
		"c": NewWSRPCFunc(func(_ *types.Context, _ string, _ int) (string, error) { return "foo", nil }, "s,i"),
	}
	wm := NewWebsocketManager(funcMap)
	wm.SetLogger(log.TestingLogger())

	mux := http.NewServeMux()
	mux.HandleFunc("/websocket", wm.WebsocketHandler)
	mux.HandleFunc("/v1/websocket", wm.WebsocketHandler)

	return httptest.NewServer(mux)
}
