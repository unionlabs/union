package proxy

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/rpc"
	"github.com/prysmaticlabs/prysm/v4/crypto/rand"
	pb "github.com/prysmaticlabs/prysm/v4/proto/engine/v1"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/sirupsen/logrus"
	logTest "github.com/sirupsen/logrus/hooks/test"
)

func TestProxy(t *testing.T) {
	t.Run("fails to proxy if destination is down", func(t *testing.T) {
		logger := logrus.New()
		hook := logTest.NewLocal(logger)
		ctx := context.Background()
		r := rand.NewGenerator()
		proxy, err := New(
			WithPort(r.Intn(50000)),
			WithDestinationAddress("http://localhost:43239"), // Nothing running at destination server.
			WithLogger(logger),
		)
		require.NoError(t, err)
		go func() {
			if err := proxy.Start(ctx); err != nil {
				t.Log(err)
			}
		}()
		time.Sleep(time.Millisecond * 100)

		rpcClient, err := rpc.DialHTTP("http://" + proxy.Address())
		require.NoError(t, err)

		err = rpcClient.CallContext(ctx, nil, "someEngineMethod")
		require.ErrorContains(t, "EOF", err)

		// Expect issues when reaching destination server.
		require.LogsContain(t, hook, "Could not forward request to destination server")
	})
	t.Run("properly proxies request/response", func(t *testing.T) {
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()

		wantDestinationResponse := &pb.ForkchoiceState{
			HeadBlockHash:      []byte("foo"),
			SafeBlockHash:      []byte("bar"),
			FinalizedBlockHash: []byte("baz"),
		}
		srv := destinationServerSetup(t, wantDestinationResponse)
		defer srv.Close()

		// Destination address server responds to JSON-RPC requests.
		r := rand.NewGenerator()
		proxy, err := New(
			WithPort(r.Intn(50000)),
			WithDestinationAddress(srv.URL),
		)
		require.NoError(t, err)
		go func() {
			if err := proxy.Start(ctx); err != nil {
				t.Log(err)
			}
		}()
		time.Sleep(time.Millisecond * 100)

		// Dials the proxy.
		rpcClient, err := rpc.DialHTTP("http://" + proxy.Address())
		require.NoError(t, err)

		// Expect the result from the proxy is the same as that one returned
		// by the destination address.
		proxyResult := &pb.ForkchoiceState{}
		err = rpcClient.CallContext(ctx, proxyResult, "someEngineMethod")
		require.NoError(t, err)
		require.DeepEqual(t, wantDestinationResponse, proxyResult)
	})
}

func TestProxy_CustomInterceptors(t *testing.T) {
	t.Run("only intercepts engine API methods", func(t *testing.T) {
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()

		type syncingResponse struct {
			Syncing bool `json:"syncing"`
		}

		wantDestinationResponse := &syncingResponse{Syncing: true}
		srv := destinationServerSetup(t, wantDestinationResponse)
		defer srv.Close()

		// Destination address server responds to JSON-RPC requests.
		r := rand.NewGenerator()
		proxy, err := New(
			WithPort(r.Intn(50000)),
			WithDestinationAddress(srv.URL),
		)
		require.NoError(t, err)
		go func() {
			if err := proxy.Start(ctx); err != nil {
				t.Log(err)
			}
		}()
		time.Sleep(time.Millisecond * 100)

		method := "eth_syncing"

		// RPC method to intercept.
		proxy.AddRequestInterceptor(
			method,
			func() interface{} {
				return &syncingResponse{Syncing: false}
			}, // Custom response.
			func() bool {
				return true // Always intercept with a custom response.
			},
		)

		// Dials the proxy.
		rpcClient, err := rpc.DialHTTP("http://" + proxy.Address())
		require.NoError(t, err)

		// Expect the result from the proxy is the same as that one returned
		// by the destination address.
		proxyResult := &syncingResponse{}
		err = rpcClient.CallContext(ctx, proxyResult, method)
		require.NoError(t, err)

		// The interception SHOULD NOT work because we should only intercept `engine` namespace methods.
		require.DeepEqual(t, wantDestinationResponse, proxyResult)
	})
	t.Run("only intercepts if trigger function returns true", func(t *testing.T) {
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()

		type engineResponse struct {
			BlockHash common.Hash `json:"blockHash"`
		}

		destinationResponse := &engineResponse{BlockHash: common.BytesToHash([]byte("foo"))}
		srv := destinationServerSetup(t, destinationResponse)
		defer srv.Close()

		// Destination address server responds to JSON-RPC requests.
		r := rand.NewGenerator()
		proxy, err := New(
			WithPort(r.Intn(50000)),
			WithDestinationAddress(srv.URL),
		)
		require.NoError(t, err)
		go func() {
			if err := proxy.Start(ctx); err != nil {
				t.Log(err)
			}
		}()
		time.Sleep(time.Millisecond * 100)

		method := "engine_newPayloadV1"

		// RPC method to intercept.
		wantInterceptedResponse := func() interface{} {
			return &engineResponse{BlockHash: common.BytesToHash([]byte("bar"))}
		}
		conditional := false
		proxy.AddRequestInterceptor(
			method,
			wantInterceptedResponse,
			func() bool {
				return conditional // Conditional trigger.
			},
		)

		// Dials the proxy.
		rpcClient, err := rpc.DialHTTP("http://" + proxy.Address())
		require.NoError(t, err)

		proxyResult := &engineResponse{}
		err = rpcClient.CallContext(ctx, proxyResult, method)
		require.NoError(t, err)

		// The interception SHOULD NOT work because we should only intercept if trigger conditional is true.
		require.DeepEqual(t, destinationResponse, proxyResult)

		conditional = true
		proxy.AddRequestInterceptor(
			method,
			wantInterceptedResponse,
			func() bool {
				return conditional // Conditional trigger.
			},
		)

		// The interception should work and we should not be getting the destination
		// response but rather a custom response from the interceptor config now that the trigger is true.
		proxyResult = &engineResponse{}
		err = rpcClient.CallContext(ctx, proxyResult, method)
		require.NoError(t, err)
		require.DeepEqual(t, wantInterceptedResponse(), proxyResult)
	})
	t.Run("triggers interceptor response correctly", func(t *testing.T) {
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()

		type engineResponse struct {
			BlockHash common.Hash `json:"blockHash"`
		}

		destinationResponse := &engineResponse{BlockHash: common.BytesToHash([]byte("foo"))}
		srv := destinationServerSetup(t, destinationResponse)
		defer srv.Close()

		// Destination address server responds to JSON-RPC requests.
		r := rand.NewGenerator()
		proxy, err := New(
			WithPort(r.Intn(50000)),
			WithDestinationAddress(srv.URL),
		)
		require.NoError(t, err)
		go func() {
			if err := proxy.Start(ctx); err != nil {
				t.Log(err)
			}
		}()
		time.Sleep(time.Millisecond * 100)

		method := "engine_newPayloadV1"

		// RPC method to intercept.
		wantInterceptedResponse := func() interface{} {
			return &engineResponse{BlockHash: common.BytesToHash([]byte("bar"))}
		}
		proxy.AddRequestInterceptor(
			method,
			wantInterceptedResponse,
			func() bool {
				return true // Always intercept with a custom response.
			},
		)

		// Dials the proxy.
		rpcClient, err := rpc.DialHTTP("http://" + proxy.Address())
		require.NoError(t, err)

		proxyResult := &engineResponse{}
		err = rpcClient.CallContext(ctx, proxyResult, method)
		require.NoError(t, err)

		// The interception should work and we should not be getting the destination
		// response but rather a custom response from the interceptor config.
		require.DeepEqual(t, wantInterceptedResponse(), proxyResult)
	})
}

func Test_isEngineAPICall(t *testing.T) {
	tests := []struct {
		name string
		args *jsonRPCObject
		want bool
	}{
		{
			name: "nil data",
			args: nil,
			want: false,
		},
		{
			name: "engine method",
			args: &jsonRPCObject{
				Method: "engine_newPayloadV1",
				ID:     1,
				Result: 5,
			},
			want: true,
		},
		{
			name: "non-engine method",
			args: &jsonRPCObject{
				Method: "eth_syncing",
				ID:     1,
				Result: false,
			},
			want: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			enc, err := json.Marshal(tt.args)
			require.NoError(t, err)
			if got := isEngineAPICall(enc); got != tt.want {
				t.Errorf("isEngineAPICall() = %v, want %v", got, tt.want)
			}
		})
	}
}

func destinationServerSetup(t *testing.T, response interface{}) *httptest.Server {
	return httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		defer func() {
			require.NoError(t, r.Body.Close())
		}()
		resp := map[string]interface{}{
			"jsonrpc": "2.0",
			"id":      1,
			"result":  response,
		}
		err := json.NewEncoder(w).Encode(resp)
		require.NoError(t, err)
	}))
}
