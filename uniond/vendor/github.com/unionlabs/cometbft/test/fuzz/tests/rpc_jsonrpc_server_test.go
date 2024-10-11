//go:build gofuzz || go1.20

package tests

import (
	"bytes"
	"encoding/json"
	"io"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/cometbft/cometbft/libs/log"
	rpcserver "github.com/cometbft/cometbft/rpc/jsonrpc/server"
	rpctypes "github.com/cometbft/cometbft/rpc/jsonrpc/types"
)

func FuzzRPCJSONRPCServer(f *testing.F) {
	type args struct {
		S string `json:"s"`
		I int    `json:"i"`
	}
	rpcFuncMap := map[string]*rpcserver.RPCFunc{
		"c": rpcserver.NewRPCFunc(func(_ *rpctypes.Context, _ *args, _ ...rpcserver.Option) (string, error) {
			return "foo", nil
		}, "args"),
	}

	mux := http.NewServeMux()
	rpcserver.RegisterRPCFuncs(mux, rpcFuncMap, log.NewNopLogger())
	f.Fuzz(func(_ *testing.T, data []byte) {
		if len(data) == 0 {
			return
		}

		req, err := http.NewRequest(http.MethodPost, "http://localhost/", bytes.NewReader(data))
		if err != nil {
			panic(err)
		}
		rec := httptest.NewRecorder()
		mux.ServeHTTP(rec, req)
		res := rec.Result()
		blob, err := io.ReadAll(res.Body)
		if err != nil {
			panic(err)
		}
		if err := res.Body.Close(); err != nil {
			panic(err)
		}
		if len(blob) == 0 {
			return
		}

		if outputJSONIsSlice(blob) {
			var recv []rpctypes.RPCResponse
			if err := json.Unmarshal(blob, &recv); err != nil {
				panic(err)
			}
			return
		}
		var recv rpctypes.RPCResponse
		if err := json.Unmarshal(blob, &recv); err != nil {
			panic(err)
		}
	})
}

func outputJSONIsSlice(input []byte) bool {
	var slice []json.RawMessage
	return json.Unmarshal(input, &slice) == nil
}
