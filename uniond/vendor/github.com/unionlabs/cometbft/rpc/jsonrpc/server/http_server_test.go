package server

import (
	"crypto/tls"
	"errors"
	"fmt"
	"io"
	"net"
	"net/http"
	"net/http/httptest"
	"sync"
	"sync/atomic"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/cometbft/cometbft/libs/log"
	"github.com/cometbft/cometbft/rpc/jsonrpc/types"
)

type sampleResult struct {
	Value string `json:"value"`
}

func TestMaxOpenConnections(t *testing.T) {
	const max = 5 // max simultaneous connections

	// Start the server.
	var open int32
	mux := http.NewServeMux()
	mux.HandleFunc("/", func(w http.ResponseWriter, _ *http.Request) {
		if n := atomic.AddInt32(&open, 1); n > int32(max) {
			t.Errorf("%d open connections, want <= %d", n, max)
		}
		defer atomic.AddInt32(&open, -1)
		time.Sleep(10 * time.Millisecond)
		fmt.Fprint(w, "some body")
	})
	config := DefaultConfig()
	l, err := Listen("tcp://127.0.0.1:0", max)
	require.NoError(t, err)
	defer l.Close()
	go Serve(l, mux, log.TestingLogger(), config) //nolint:errcheck // ignore for tests

	// Make N GET calls to the server.
	attempts := max * 2
	var wg sync.WaitGroup
	var failed int32
	for i := 0; i < attempts; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			c := http.Client{Timeout: 3 * time.Second}
			r, err := c.Get("http://" + l.Addr().String())
			if err != nil {
				atomic.AddInt32(&failed, 1)
				return
			}
			defer r.Body.Close()
		}()
	}
	wg.Wait()

	// We expect some Gets to fail as the server's accept queue is filled,
	// but most should succeed.
	if int(failed) >= attempts/2 {
		t.Errorf("%d requests failed within %d attempts", failed, attempts)
	}
}

func TestServeTLS(t *testing.T) {
	ln, err := net.Listen("tcp", "localhost:0")
	require.NoError(t, err)
	defer ln.Close()

	mux := http.NewServeMux()
	mux.HandleFunc("/", func(w http.ResponseWriter, _ *http.Request) {
		fmt.Fprint(w, "some body")
	})

	chErr := make(chan error, 1)
	go func() {
		// FIXME This goroutine leaks
		chErr <- ServeTLS(ln, mux, "test.crt", "test.key", log.TestingLogger(), DefaultConfig())
	}()

	select {
	case err := <-chErr:
		require.NoError(t, err)
	case <-time.After(100 * time.Millisecond):
	}

	tr := &http.Transport{
		TLSClientConfig: &tls.Config{InsecureSkipVerify: true},
	}
	c := &http.Client{Transport: tr}
	res, err := c.Get("https://" + ln.Addr().String())
	require.NoError(t, err)
	defer res.Body.Close()
	assert.Equal(t, http.StatusOK, res.StatusCode)

	body, err := io.ReadAll(res.Body)
	require.NoError(t, err)
	assert.Equal(t, []byte("some body"), body)
}

func TestWriteRPCResponseHTTP(t *testing.T) {
	id := types.JSONRPCIntID(-1)

	// one argument
	w := httptest.NewRecorder()
	err := WriteCacheableRPCResponseHTTP(w, types.NewRPCSuccessResponse(id, &sampleResult{"hello"}))
	require.NoError(t, err)
	resp := w.Result()
	body, err := io.ReadAll(resp.Body)
	_ = resp.Body.Close()
	require.NoError(t, err)
	assert.Equal(t, 200, resp.StatusCode)
	assert.Equal(t, "application/json", resp.Header.Get("Content-Type"))
	assert.Equal(t, "public, max-age=86400", resp.Header.Get("Cache-Control"))
	assert.Equal(t, `{"jsonrpc":"2.0","id":-1,"result":{"value":"hello"}}`, string(body))

	// multiple arguments
	w = httptest.NewRecorder()
	err = WriteRPCResponseHTTP(w,
		types.NewRPCSuccessResponse(id, &sampleResult{"hello"}),
		types.NewRPCSuccessResponse(id, &sampleResult{"world"}))
	require.NoError(t, err)
	resp = w.Result()
	body, err = io.ReadAll(resp.Body)
	_ = resp.Body.Close()
	require.NoError(t, err)

	assert.Equal(t, 200, resp.StatusCode)
	assert.Equal(t, "application/json", resp.Header.Get("Content-Type"))
	assert.Equal(t, `[{"jsonrpc":"2.0","id":-1,"result":{"value":"hello"}},{"jsonrpc":"2.0","id":-1,"result":{"value":"world"}}]`, string(body))
}

// TestWriteRPCResponseHTTPError tests WriteRPCResponseHTTPError.
func TestWriteRPCResponseHTTPError(t *testing.T) {
	w := httptest.NewRecorder()
	err := WriteRPCResponseHTTPError(
		w,
		http.StatusInternalServerError,
		types.RPCInternalError(types.JSONRPCIntID(-1), errors.New("foo")))
	require.NoError(t, err)
	resp := w.Result()
	body, err := io.ReadAll(resp.Body)
	_ = resp.Body.Close()
	require.NoError(t, err)

	assert.Equal(t, http.StatusInternalServerError, resp.StatusCode)
	assert.Equal(t, "application/json", resp.Header.Get("Content-Type"))
	assert.Equal(t, `{"jsonrpc":"2.0","id":-1,"error":{"code":-32603,"message":"Internal error","data":"foo"}}`, string(body))
}
