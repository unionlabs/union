package jsonrpc

import (
	"bytes"
	"context"
	crand "crypto/rand"
	"encoding/json"
	"fmt"
	"net/http"
	"net/url"
	"os"
	"os/exec"
	"strconv"
	"strings"
	"testing"
	"time"

	"github.com/go-kit/log/term"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/cometbft/cometbft/internal/net"
	cmtrand "github.com/cometbft/cometbft/internal/rand"
	cmtbytes "github.com/cometbft/cometbft/libs/bytes"
	"github.com/cometbft/cometbft/libs/log"
	"github.com/cometbft/cometbft/rpc/jsonrpc/client"
	"github.com/cometbft/cometbft/rpc/jsonrpc/server"
	"github.com/cometbft/cometbft/rpc/jsonrpc/types"
)

// Client and Server should work over tcp or unix sockets.
const (
	unixSocket = "/tmp/rpc_test.sock"
	unixAddr   = "unix://" + unixSocket

	websocketEndpoint = "/websocket/endpoint"

	testVal = "acbd"
)

var (
	ctx     = context.Background()
	baseIP  = "tcp://127.0.0.1"
	port, _ = net.GetFreePort()
	tcpAddr = baseIP + ":" + strconv.Itoa(port)
)

type ResultEcho struct {
	Value string `json:"value"`
}

type ResultEchoInt struct {
	Value int `json:"value"`
}

type ResultEchoBytes struct {
	Value []byte `json:"value"`
}

type ResultEchoDataBytes struct {
	Value cmtbytes.HexBytes `json:"value"`
}

type ResultEchoWithDefault struct {
	Value int `json:"value"`
}

// Define some routes.
var Routes = map[string]*server.RPCFunc{
	"echo":            server.NewRPCFunc(EchoResult, "arg"),
	"echo_ws":         server.NewWSRPCFunc(EchoWSResult, "arg"),
	"echo_bytes":      server.NewRPCFunc(EchoBytesResult, "arg"),
	"echo_data_bytes": server.NewRPCFunc(EchoDataBytesResult, "arg"),
	"echo_int":        server.NewRPCFunc(EchoIntResult, "arg"),
	"echo_default":    server.NewRPCFunc(EchoWithDefault, "arg", server.Cacheable("arg")),
}

func EchoResult(_ *types.Context, v string) (*ResultEcho, error) {
	return &ResultEcho{v}, nil
}

func EchoWSResult(_ *types.Context, v string) (*ResultEcho, error) {
	return &ResultEcho{v}, nil
}

func EchoIntResult(_ *types.Context, v int) (*ResultEchoInt, error) {
	return &ResultEchoInt{v}, nil
}

func EchoBytesResult(_ *types.Context, v []byte) (*ResultEchoBytes, error) {
	return &ResultEchoBytes{v}, nil
}

func EchoDataBytesResult(_ *types.Context, v cmtbytes.HexBytes) (*ResultEchoDataBytes, error) {
	return &ResultEchoDataBytes{v}, nil
}

func EchoWithDefault(_ *types.Context, v *int) (*ResultEchoWithDefault, error) {
	val := -1
	if v != nil {
		val = *v
	}
	return &ResultEchoWithDefault{val}, nil
}

func TestMain(m *testing.M) {
	setup()
	code := m.Run()
	os.Exit(code)
}

var colorFn = func(keyvals ...any) term.FgBgColor {
	for i := 0; i < len(keyvals)-1; i += 2 {
		if keyvals[i] == "socket" {
			if keyvals[i+1] == "tcp" {
				return term.FgBgColor{Fg: term.DarkBlue}
			} else if keyvals[i+1] == "unix" {
				return term.FgBgColor{Fg: term.DarkCyan}
			}
		}
	}
	return term.FgBgColor{}
}

// launch unix and tcp servers.
func setup() {
	logger := log.NewTMLoggerWithColorFn(log.NewSyncWriter(os.Stdout), colorFn)

	cmd := exec.Command("rm", "-f", unixSocket)
	err := cmd.Start()
	if err != nil {
		panic(err)
	}
	if err = cmd.Wait(); err != nil {
		panic(err)
	}

	tcpLogger := logger.With("socket", "tcp")
	mux := http.NewServeMux()
	server.RegisterRPCFuncs(mux, Routes, tcpLogger)
	wm := server.NewWebsocketManager(Routes, server.ReadWait(5*time.Second), server.PingPeriod(1*time.Second))
	wm.SetLogger(tcpLogger)
	mux.HandleFunc(websocketEndpoint, wm.WebsocketHandler)
	mux.HandleFunc("/v1"+websocketEndpoint, wm.WebsocketHandler)
	config := server.DefaultConfig()
	listener1, err := server.Listen(tcpAddr, config.MaxOpenConnections)
	if err != nil {
		panic(err)
	}
	go func() {
		if err := server.Serve(listener1, mux, tcpLogger, config); err != nil {
			panic(err)
		}
	}()

	unixLogger := logger.With("socket", "unix")
	mux2 := http.NewServeMux()
	server.RegisterRPCFuncs(mux2, Routes, unixLogger)
	wm = server.NewWebsocketManager(Routes)
	wm.SetLogger(unixLogger)
	mux2.HandleFunc(websocketEndpoint, wm.WebsocketHandler)
	mux2.HandleFunc("/v1"+websocketEndpoint, wm.WebsocketHandler)
	listener2, err := server.Listen(unixAddr, config.MaxOpenConnections)
	if err != nil {
		panic(err)
	}
	go func() {
		if err := server.Serve(listener2, mux2, unixLogger, config); err != nil {
			panic(err)
		}
	}()

	// wait for servers to start
	time.Sleep(time.Second * 2)
}

func echoViaHTTP(cl client.Caller, val string) (string, error) {
	params := map[string]any{
		"arg": val,
	}
	result := new(ResultEcho)
	if _, err := cl.Call(ctx, "echo", params, result); err != nil {
		return "", err
	}
	return result.Value, nil
}

func echoIntViaHTTP(cl client.Caller, val int) (int, error) {
	params := map[string]any{
		"arg": val,
	}
	result := new(ResultEchoInt)
	if _, err := cl.Call(ctx, "echo_int", params, result); err != nil {
		return 0, err
	}
	return result.Value, nil
}

func echoBytesViaHTTP(cl client.Caller, bytes []byte) ([]byte, error) {
	params := map[string]any{
		"arg": bytes,
	}
	result := new(ResultEchoBytes)
	if _, err := cl.Call(ctx, "echo_bytes", params, result); err != nil {
		return []byte{}, err
	}
	return result.Value, nil
}

func echoDataBytesViaHTTP(cl client.Caller, bytes cmtbytes.HexBytes) (cmtbytes.HexBytes, error) {
	params := map[string]any{
		"arg": bytes,
	}
	result := new(ResultEchoDataBytes)
	if _, err := cl.Call(ctx, "echo_data_bytes", params, result); err != nil {
		return []byte{}, err
	}
	return result.Value, nil
}

func echoWithDefaultViaHTTP(cl client.Caller, v *int) (int, error) {
	params := map[string]any{}
	if v != nil {
		params["arg"] = *v
	}
	result := new(ResultEchoWithDefault)
	if _, err := cl.Call(ctx, "echo_default", params, result); err != nil {
		return 0, err
	}
	return result.Value, nil
}

func testWithHTTPClient(t *testing.T, cl client.HTTPClient) {
	t.Helper()
	val := testVal
	got, err := echoViaHTTP(cl, val)
	require.NoError(t, err)
	assert.Equal(t, got, val)

	val2 := randBytes(t)
	got2, err := echoBytesViaHTTP(cl, val2)
	require.NoError(t, err)
	assert.Equal(t, got2, val2)

	val3 := cmtbytes.HexBytes(randBytes(t))
	got3, err := echoDataBytesViaHTTP(cl, val3)
	require.NoError(t, err)
	assert.Equal(t, got3, val3)

	val4 := cmtrand.Intn(10000)
	got4, err := echoIntViaHTTP(cl, val4)
	require.NoError(t, err)
	assert.Equal(t, got4, val4)

	got5, err := echoWithDefaultViaHTTP(cl, nil)
	require.NoError(t, err)
	assert.Equal(t, -1, got5)

	val6 := cmtrand.Intn(10000)
	got6, err := echoWithDefaultViaHTTP(cl, &val6)
	require.NoError(t, err)
	assert.Equal(t, got6, val6)
}

func echoViaWS(cl *client.WSClient, val string) (string, error) {
	params := map[string]any{
		"arg": val,
	}
	err := cl.Call(context.Background(), "echo", params)
	if err != nil {
		return "", err
	}

	msg := <-cl.ResponsesCh
	if msg.Error != nil {
		return "", err
	}
	result := new(ResultEcho)
	err = json.Unmarshal(msg.Result, result)
	if err != nil {
		return "", err
	}
	return result.Value, nil
}

func echoBytesViaWS(cl *client.WSClient, bytes []byte) ([]byte, error) {
	params := map[string]any{
		"arg": bytes,
	}
	err := cl.Call(context.Background(), "echo_bytes", params)
	if err != nil {
		return []byte{}, err
	}

	msg := <-cl.ResponsesCh
	if msg.Error != nil {
		return []byte{}, msg.Error
	}
	result := new(ResultEchoBytes)
	err = json.Unmarshal(msg.Result, result)
	if err != nil {
		return []byte{}, err
	}
	return result.Value, nil
}

func testWithWSClient(t *testing.T, cl *client.WSClient) {
	t.Helper()
	val := testVal
	got, err := echoViaWS(cl, val)
	require.NoError(t, err)
	assert.Equal(t, got, val)

	val2 := randBytes(t)
	got2, err := echoBytesViaWS(cl, val2)
	require.NoError(t, err)
	assert.Equal(t, got2, val2)
}

// -------------

func TestServersAndClientsBasic(t *testing.T) {
	serverAddrs := [...]string{tcpAddr, unixAddr}
	for _, addr := range serverAddrs {
		cl1, err := client.NewURI(addr)
		require.NoError(t, err)
		fmt.Printf("=== testing server on %s using URI client", addr)
		testWithHTTPClient(t, cl1)

		cl2, err := client.New(addr)
		require.NoError(t, err)
		fmt.Printf("=== testing server on %s using JSONRPC client", addr)
		testWithHTTPClient(t, cl2)

		cl3, err := client.NewWS(addr, websocketEndpoint)
		require.NoError(t, err)
		cl3.SetLogger(log.TestingLogger())
		err = cl3.Start()
		require.NoError(t, err)
		fmt.Printf("=== testing server on %s using WS client", addr)
		testWithWSClient(t, cl3)
		err = cl3.Stop()
		require.NoError(t, err)
	}
}

func TestServersAndClientsBasicV1(t *testing.T) {
	serverAddrs := [...]string{tcpAddr, unixAddr}
	for _, addr := range serverAddrs {
		cl1, err := client.NewURI(addr)
		require.NoError(t, err)
		fmt.Printf("=== testing server on %s using URI client", addr)
		testWithHTTPClient(t, cl1)

		cl2, err := client.New(addr)
		require.NoError(t, err)
		fmt.Printf("=== testing server on %s using JSONRPC client", addr)
		testWithHTTPClient(t, cl2)

		cl3, err := client.NewWS(addr, "/v1"+websocketEndpoint)
		require.NoError(t, err)
		cl3.SetLogger(log.TestingLogger())
		err = cl3.Start()
		require.NoError(t, err)
		fmt.Printf("=== testing server on %s using WS client", addr)
		testWithWSClient(t, cl3)
		err = cl3.Stop()
		require.NoError(t, err)
	}
}

func TestHexStringArg(t *testing.T) {
	cl, err := client.NewURI(tcpAddr)
	require.NoError(t, err)
	// should NOT be handled as hex
	val := "0xabc"
	got, err := echoViaHTTP(cl, val)
	require.NoError(t, err)
	assert.Equal(t, got, val)
}

func TestQuotedStringArg(t *testing.T) {
	cl, err := client.NewURI(tcpAddr)
	require.NoError(t, err)
	// should NOT be unquoted
	val := "\"abc\""
	got, err := echoViaHTTP(cl, val)
	require.NoError(t, err)
	assert.Equal(t, got, val)
}

func TestWSNewWSRPCFunc(t *testing.T) {
	cl, err := client.NewWS(tcpAddr, websocketEndpoint)
	require.NoError(t, err)
	cl.SetLogger(log.TestingLogger())
	err = cl.Start()
	require.NoError(t, err)
	t.Cleanup(func() {
		if err := cl.Stop(); err != nil {
			t.Error(err)
		}
	})

	val := testVal
	params := map[string]any{
		"arg": val,
	}
	err = cl.Call(context.Background(), "echo_ws", params)
	require.NoError(t, err)

	msg := <-cl.ResponsesCh
	if msg.Error != nil {
		t.Fatal(err)
	}
	result := new(ResultEcho)
	err = json.Unmarshal(msg.Result, result)
	require.NoError(t, err)
	got := result.Value
	assert.Equal(t, got, val)
}

func TestWSNewWSRPCFuncV1(t *testing.T) {
	cl, err := client.NewWS(tcpAddr, "/v1"+websocketEndpoint)
	require.NoError(t, err)
	cl.SetLogger(log.TestingLogger())
	err = cl.Start()
	require.NoError(t, err)
	t.Cleanup(func() {
		if err := cl.Stop(); err != nil {
			t.Error(err)
		}
	})

	val := testVal
	params := map[string]any{
		"arg": val,
	}
	err = cl.Call(context.Background(), "echo_ws", params)
	require.NoError(t, err)

	msg := <-cl.ResponsesCh
	if msg.Error != nil {
		t.Fatal(err)
	}
	result := new(ResultEcho)
	err = json.Unmarshal(msg.Result, result)
	require.NoError(t, err)
	got := result.Value
	assert.Equal(t, got, val)
}

func TestWSHandlesArrayParams(t *testing.T) {
	cl, err := client.NewWS(tcpAddr, websocketEndpoint)
	require.NoError(t, err)
	cl.SetLogger(log.TestingLogger())
	err = cl.Start()
	require.NoError(t, err)
	t.Cleanup(func() {
		if err := cl.Stop(); err != nil {
			t.Error(err)
		}
	})

	val := testVal
	params := []any{val}
	err = cl.CallWithArrayParams(context.Background(), "echo_ws", params)
	require.NoError(t, err)

	msg := <-cl.ResponsesCh
	if msg.Error != nil {
		t.Fatalf("%+v", err)
	}
	result := new(ResultEcho)
	err = json.Unmarshal(msg.Result, result)
	require.NoError(t, err)
	got := result.Value
	assert.Equal(t, got, val)
}

func TestWSHandlesArrayParamsV1(t *testing.T) {
	cl, err := client.NewWS(tcpAddr, "/v1"+websocketEndpoint)
	require.NoError(t, err)
	cl.SetLogger(log.TestingLogger())
	err = cl.Start()
	require.NoError(t, err)
	t.Cleanup(func() {
		if err := cl.Stop(); err != nil {
			t.Error(err)
		}
	})

	val := testVal
	params := []any{val}
	err = cl.CallWithArrayParams(context.Background(), "echo_ws", params)
	require.NoError(t, err)

	msg := <-cl.ResponsesCh
	if msg.Error != nil {
		t.Fatalf("%+v", err)
	}
	result := new(ResultEcho)
	err = json.Unmarshal(msg.Result, result)
	require.NoError(t, err)
	got := result.Value
	assert.Equal(t, got, val)
}

// TestWSClientPingPong checks that a client & server exchange pings
// & pongs so connection stays alive.
func TestWSClientPingPong(t *testing.T) {
	cl, err := client.NewWS(tcpAddr, websocketEndpoint)
	require.NoError(t, err)
	cl.SetLogger(log.TestingLogger())
	err = cl.Start()
	require.NoError(t, err)
	t.Cleanup(func() {
		if err := cl.Stop(); err != nil {
			t.Error(err)
		}
	})

	time.Sleep(6 * time.Second)
}

func TestWSClientPingPongV1(t *testing.T) {
	cl, err := client.NewWS(tcpAddr, "/v1"+websocketEndpoint)
	require.NoError(t, err)
	cl.SetLogger(log.TestingLogger())
	err = cl.Start()
	require.NoError(t, err)
	t.Cleanup(func() {
		if err := cl.Stop(); err != nil {
			t.Error(err)
		}
	})

	time.Sleep(6 * time.Second)
}

func TestJSONRPCCaching(t *testing.T) {
	httpAddr := strings.Replace(tcpAddr, "tcp://", "http://", 1)
	cl, err := client.DefaultHTTPClient(httpAddr)
	require.NoError(t, err)

	// Not supplying the arg should result in not caching
	params := make(map[string]any)
	req, err := types.MapToRequest(types.JSONRPCIntID(1000), "echo_default", params)
	require.NoError(t, err)

	res1, err := rawJSONRPCRequest(t, cl, httpAddr, req)
	defer func() { _ = res1.Body.Close() }()
	require.NoError(t, err)
	assert.Equal(t, "", res1.Header.Get("Cache-Control"))

	// Supplying the arg should result in caching
	params["arg"] = cmtrand.Intn(10000)
	req, err = types.MapToRequest(types.JSONRPCIntID(1001), "echo_default", params)
	require.NoError(t, err)

	res2, err := rawJSONRPCRequest(t, cl, httpAddr, req)
	defer func() { _ = res2.Body.Close() }()
	require.NoError(t, err)
	assert.Equal(t, "public, max-age=86400", res2.Header.Get("Cache-Control"))
}

func rawJSONRPCRequest(t *testing.T, cl *http.Client, url string, req any) (*http.Response, error) {
	t.Helper()
	reqBytes, err := json.Marshal(req)
	require.NoError(t, err)

	reqBuf := bytes.NewBuffer(reqBytes)
	httpReq, err := http.NewRequest(http.MethodPost, url, reqBuf)
	require.NoError(t, err)

	httpReq.Header.Set("Content-Type", "application/json")

	return cl.Do(httpReq)
}

func TestURICaching(t *testing.T) {
	t.Helper()
	httpAddr := strings.Replace(tcpAddr, "tcp://", "http://", 1)
	cl, err := client.DefaultHTTPClient(httpAddr)
	require.NoError(t, err)

	// Not supplying the arg should result in not caching
	args := url.Values{}
	res1, err := rawURIRequest(t, cl, httpAddr+"/echo_default", args)
	defer func() { _ = res1.Body.Close() }()
	require.NoError(t, err)
	assert.Equal(t, "", res1.Header.Get("Cache-Control"))

	// Supplying the arg should result in caching
	args.Set("arg", strconv.Itoa(cmtrand.Intn(10000)))
	res2, err := rawURIRequest(t, cl, httpAddr+"/echo_default", args)
	defer func() { _ = res2.Body.Close() }()
	require.NoError(t, err)
	assert.Equal(t, "public, max-age=86400", res2.Header.Get("Cache-Control"))
}

func rawURIRequest(t *testing.T, cl *http.Client, url string, args url.Values) (*http.Response, error) {
	t.Helper()
	req, err := http.NewRequest(http.MethodPost, url, strings.NewReader(args.Encode()))
	require.NoError(t, err)

	req.Header.Set("Content-Type", "application/x-www-form-urlencoded")

	return cl.Do(req)
}

func randBytes(t *testing.T) []byte {
	t.Helper()
	n := cmtrand.Intn(10) + 2
	buf := make([]byte, n)
	_, err := crand.Read(buf)
	require.NoError(t, err)
	return bytes.ReplaceAll(buf, []byte("="), []byte{100})
}
