package jsonrpc_test

import (
	"context"
	"encoding/json"
	"io"
	"io/ioutil"
	"net/http"
	"net/http/httptest"
	"net/url"
	"testing"

	"github.com/go-kit/kit/transport/http/jsonrpc"
)

type TestResponse struct {
	Body   io.ReadCloser
	String string
}

type testServerResponseOptions struct {
	Body   string
	Status int
}

func httptestServer(t *testing.T) *httptest.Server {
	return httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		defer r.Body.Close()

		var testReq jsonrpc.Request
		if err := json.NewDecoder(r.Body).Decode(&testReq); err != nil {
			t.Fatal(err)
		}

		var options testServerResponseOptions
		if err := json.Unmarshal(testReq.Params, &options); err != nil {
			t.Fatal(err)
		}

		if options.Status == 0 {
			options.Status = http.StatusOK
		}

		w.WriteHeader(options.Status)
		w.Write([]byte(options.Body))
	}))
}

func TestBeforeAfterFuncs(t *testing.T) {
	t.Parallel()

	var tests = []struct {
		name   string
		status int
		body   string
	}{
		{
			name: "empty body",
			body: "",
		},
		{
			name:   "empty body 500",
			body:   "",
			status: 500,
		},

		{
			name: "empty json body",
			body: "{}",
		},
		{
			name: "error",
			body: `{"jsonrpc":"2.0","error":{"code":32603,"message":"Bad thing happened."}}`,
		},
	}

	server := httptestServer(t)
	defer server.Close()

	testUrl, err := url.Parse(server.URL)
	if err != nil {
		t.Fatal(err)
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			beforeCalled := false
			afterCalled := false
			finalizerCalled := false

			sut := jsonrpc.NewClient(
				testUrl,
				"dummy",
				jsonrpc.ClientBefore(func(ctx context.Context, req *http.Request) context.Context {
					beforeCalled = true
					return ctx
				}),
				jsonrpc.ClientAfter(func(ctx context.Context, resp *http.Response) context.Context {
					afterCalled = true
					return ctx
				}),
				jsonrpc.ClientFinalizer(func(ctx context.Context, err error) {
					finalizerCalled = true
				}),
			)

			sut.Endpoint()(context.TODO(), testServerResponseOptions{Body: tt.body, Status: tt.status})
			if !beforeCalled {
				t.Fatal("Expected client before func to be called. Wasn't.")
			}
			if !afterCalled {
				t.Fatal("Expected client after func to be called. Wasn't.")
			}
			if !finalizerCalled {
				t.Fatal("Expected client finalizer func to be called. Wasn't.")
			}

		})

	}

}

type staticIDGenerator int

func (g staticIDGenerator) Generate() interface{} { return g }

func TestClientHappyPath(t *testing.T) {
	t.Parallel()

	var (
		afterCalledKey    = "AC"
		beforeHeaderKey   = "BF"
		beforeHeaderValue = "beforeFuncWozEre"
		testbody          = `{"jsonrpc":"2.0", "result":5}`
		requestBody       []byte
		beforeFunc        = func(ctx context.Context, r *http.Request) context.Context {
			r.Header.Add(beforeHeaderKey, beforeHeaderValue)
			return ctx
		}
		encode = func(ctx context.Context, req interface{}) (json.RawMessage, error) {
			return json.Marshal(req)
		}
		afterFunc = func(ctx context.Context, r *http.Response) context.Context {
			return context.WithValue(ctx, afterCalledKey, true)
		}
		finalizerCalled = false
		fin             = func(ctx context.Context, err error) {
			finalizerCalled = true
		}
		decode = func(ctx context.Context, res jsonrpc.Response) (interface{}, error) {
			if ac := ctx.Value(afterCalledKey); ac == nil {
				t.Fatal("after not called")
			}
			var result int
			err := json.Unmarshal(res.Result, &result)
			if err != nil {
				return nil, err
			}
			return result, nil
		}

		wantID = 666
		gen    = staticIDGenerator(wantID)
	)

	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Header.Get(beforeHeaderKey) != beforeHeaderValue {
			t.Fatal("Header not set by before func.")
		}

		b, err := ioutil.ReadAll(r.Body)
		if err != nil && err != io.EOF {
			t.Fatal(err)
		}
		requestBody = b

		w.WriteHeader(http.StatusOK)
		w.Write([]byte(testbody))
	}))
	defer server.Close()

	sut := jsonrpc.NewClient(
		mustParse(server.URL),
		"add",
		jsonrpc.ClientRequestEncoder(encode),
		jsonrpc.ClientResponseDecoder(decode),
		jsonrpc.ClientBefore(beforeFunc),
		jsonrpc.ClientAfter(afterFunc),
		jsonrpc.ClientRequestIDGenerator(gen),
		jsonrpc.ClientFinalizer(fin),
		jsonrpc.SetClient(http.DefaultClient),
		jsonrpc.BufferedStream(false),
	)

	type addRequest struct {
		A int
		B int
	}

	in := addRequest{2, 2}

	result, err := sut.Endpoint()(context.Background(), in)
	if err != nil {
		t.Fatal(err)
	}
	ri, ok := result.(int)
	if !ok {
		t.Fatalf("result is not int: (%T)%+v", result, result)
	}
	if ri != 5 {
		t.Fatalf("want=5, got=%d", ri)
	}

	var requestAtServer jsonrpc.Request
	err = json.Unmarshal(requestBody, &requestAtServer)
	if err != nil {
		t.Fatal(err)
	}
	if id, _ := requestAtServer.ID.Int(); id != wantID {
		t.Fatalf("Request ID at server: want=%d, got=%d", wantID, id)
	}
	if requestAtServer.JSONRPC != jsonrpc.Version {
		t.Fatalf("JSON-RPC version at server: want=%s, got=%s", jsonrpc.Version, requestAtServer.JSONRPC)
	}

	var paramsAtServer addRequest
	err = json.Unmarshal(requestAtServer.Params, &paramsAtServer)
	if err != nil {
		t.Fatal(err)
	}

	if paramsAtServer != in {
		t.Fatalf("want=%+v, got=%+v", in, paramsAtServer)
	}

	if !finalizerCalled {
		t.Fatal("Expected finalizer to be called. Wasn't.")
	}
}

func TestCanUseDefaults(t *testing.T) {
	t.Parallel()

	var (
		testbody    = `{"jsonrpc":"2.0", "result":"boogaloo"}`
		requestBody []byte
	)

	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		b, err := ioutil.ReadAll(r.Body)
		if err != nil && err != io.EOF {
			t.Fatal(err)
		}
		requestBody = b

		w.WriteHeader(http.StatusOK)
		w.Write([]byte(testbody))
	}))
	defer server.Close()

	sut := jsonrpc.NewClient(
		mustParse(server.URL),
		"add",
	)

	type addRequest struct {
		A int
		B int
	}

	in := addRequest{2, 2}

	result, err := sut.Endpoint()(context.Background(), in)
	if err != nil {
		t.Fatal(err)
	}
	rs, ok := result.(string)
	if !ok {
		t.Fatalf("result is not string: (%T)%+v", result, result)
	}
	if rs != "boogaloo" {
		t.Fatalf("want=boogaloo, got=%s", rs)
	}

	var requestAtServer jsonrpc.Request
	err = json.Unmarshal(requestBody, &requestAtServer)
	if err != nil {
		t.Fatal(err)
	}
	var paramsAtServer addRequest
	err = json.Unmarshal(requestAtServer.Params, &paramsAtServer)
	if err != nil {
		t.Fatal(err)
	}

	if paramsAtServer != in {
		t.Fatalf("want=%+v, got=%+v", in, paramsAtServer)
	}
}

func TestClientCanHandleJSONRPCError(t *testing.T) {
	t.Parallel()

	var testbody = `{
		"jsonrpc": "2.0",
		"error": {
			"code": -32603,
			"message": "Bad thing happened."
		}
	}`
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
		w.Write([]byte(testbody))
	}))
	defer server.Close()

	sut := jsonrpc.NewClient(mustParse(server.URL), "add")

	_, err := sut.Endpoint()(context.Background(), 5)
	if err == nil {
		t.Fatal("Expected error, got none.")
	}

	{
		want := "Bad thing happened."
		got := err.Error()
		if got != want {
			t.Fatalf("error message: want=%s, got=%s", want, got)
		}
	}

	type errorCoder interface {
		ErrorCode() int
	}
	ec, ok := err.(errorCoder)
	if !ok {
		t.Fatal("Error is not errorCoder")
	}

	{
		want := -32603
		got := ec.ErrorCode()
		if got != want {
			t.Fatalf("error code: want=%d, got=%d", want, got)
		}
	}
}

func TestDefaultAutoIncrementer(t *testing.T) {
	t.Parallel()

	sut := jsonrpc.NewAutoIncrementID(0)
	var want uint64
	for ; want < 100; want++ {
		got := sut.Generate()
		if got != want {
			t.Fatalf("want=%d, got=%d", want, got)
		}
	}
}

func mustParse(s string) *url.URL {
	u, err := url.Parse(s)
	if err != nil {
		panic(err)
	}
	return u
}
