package integration_test

import (
	"context"
	"fmt"
	"io/ioutil"
	"net/http"
	"strings"
	"testing"
	"time"

	"github.com/golang/protobuf/jsonpb"
	"github.com/grpc-ecosystem/grpc-gateway/runtime"
	spb "google.golang.org/genproto/googleapis/rpc/status"
	"google.golang.org/grpc/codes"
)

func runServer(ctx context.Context, t *testing.T, port uint16) {
	opt := runtime.WithProtoErrorHandler(runtime.DefaultHTTPProtoErrorHandler)
	if err := runGateway(ctx, fmt.Sprintf(":%d", port), opt); err != nil {
		t.Errorf("runGateway() failed with %v; want success", err)
	}
}

func TestWithProtoErrorHandler(t *testing.T) {
	if testing.Short() {
		t.Skip()
		return
	}

	ctx := context.Background()
	ctx, cancel := context.WithCancel(ctx)
	defer cancel()

	const port = 8082
	go runServer(ctx, t, port)
	if err := waitForGateway(ctx, 8082); err != nil {
		t.Errorf("waitForGateway(ctx, 8082) failed with %v; want success", err)
	}
	testEcho(t, port, "application/json")
	testEchoBody(t, port)
}

func TestABEWithProtoErrorHandler(t *testing.T) {
	if testing.Short() {
		t.Skip()
		return
	}

	ctx := context.Background()
	ctx, cancel := context.WithCancel(ctx)
	defer cancel()

	const port = 8083
	go runServer(ctx, t, port)
	if err := waitForGateway(ctx, 8083); err != nil {
		t.Errorf("waitForGateway(ctx, 8083) failed with %v; want success", err)
	}

	testABECreate(t, port)
	testABECreateBody(t, port)
	testABEBulkCreate(t, port)
	testABELookup(t, port)
	testABELookupNotFoundWithProtoError(t, port)
	testABEList(t, port)
	testABEBulkEcho(t, port)
	testABEBulkEchoZeroLength(t, port)
	testAdditionalBindings(t, port)
}

func testABELookupNotFoundWithProtoError(t *testing.T, port uint16) {
	url := fmt.Sprintf("http://localhost:%d/v1/example/a_bit_of_everything", port)
	uuid := "not_exist"
	url = fmt.Sprintf("%s/%s", url, uuid)
	resp, err := http.Get(url)
	if err != nil {
		t.Errorf("http.Get(%q) failed with %v; want success", url, err)
		return
	}
	defer resp.Body.Close()

	buf, err := ioutil.ReadAll(resp.Body)
	if err != nil {
		t.Errorf("ioutil.ReadAll(resp.Body) failed with %v; want success", err)
		return
	}

	if got, want := resp.StatusCode, http.StatusNotFound; got != want {
		t.Errorf("resp.StatusCode = %d; want %d", got, want)
		t.Logf("%s", buf)
		return
	}

	var msg spb.Status
	if err := jsonpb.UnmarshalString(string(buf), &msg); err != nil {
		t.Errorf("jsonpb.UnmarshalString(%s, &msg) failed with %v; want success", buf, err)
		return
	}

	if got, want := msg.Code, int32(codes.NotFound); got != want {
		t.Errorf("msg.Code = %d; want %d", got, want)
		return
	}

	if got, want := msg.Message, "not found"; got != want {
		t.Errorf("msg.Message = %s; want %s", got, want)
		return
	}

	if got, want := resp.Header.Get("Grpc-Metadata-Uuid"), uuid; got != want {
		t.Errorf("Grpc-Metadata-Uuid was %s, wanted %s", got, want)
	}
	if got, want := resp.Trailer.Get("Grpc-Trailer-Foo"), "foo2"; got != want {
		t.Errorf("Grpc-Trailer-Foo was %q, wanted %q", got, want)
	}
	if got, want := resp.Trailer.Get("Grpc-Trailer-Bar"), "bar2"; got != want {
		t.Errorf("Grpc-Trailer-Bar was %q, wanted %q", got, want)
	}
}

func TestUnknownPathWithProtoError(t *testing.T) {
	if testing.Short() {
		t.Skip()
		return
	}

	ctx := context.Background()
	ctx, cancel := context.WithCancel(ctx)
	defer cancel()

	const port = 8084
	go runServer(ctx, t, port)
	if err := waitForGateway(ctx, 8084); err != nil {
		t.Errorf("waitForGateway(ctx, 8084) failed with %v; want success", err)
	}

	url := fmt.Sprintf("http://localhost:%d", port)
	resp, err := http.Post(url, "application/json", strings.NewReader("{}"))
	if err != nil {
		t.Errorf("http.Post(%q) failed with %v; want success", url, err)
		return
	}
	defer resp.Body.Close()
	buf, err := ioutil.ReadAll(resp.Body)
	if err != nil {
		t.Errorf("ioutil.ReadAll(resp.Body) failed with %v; want success", err)
		return
	}

	if got, want := resp.StatusCode, http.StatusNotImplemented; got != want {
		t.Errorf("resp.StatusCode = %d; want %d", got, want)
		t.Logf("%s", buf)
	}

	var msg spb.Status
	if err := jsonpb.UnmarshalString(string(buf), &msg); err != nil {
		t.Errorf("jsonpb.UnmarshalString(%s, &msg) failed with %v; want success", buf, err)
		return
	}

	if got, want := msg.Code, int32(codes.Unimplemented); got != want {
		t.Errorf("msg.Code = %d; want %d", got, want)
		return
	}

	if msg.Message == "" {
		t.Errorf("msg.Message should not be empty")
		return
	}
}

func TestMethodNotAllowedWithProtoError(t *testing.T) {
	if testing.Short() {
		t.Skip()
		return
	}

	ctx := context.Background()
	ctx, cancel := context.WithCancel(ctx)
	defer cancel()

	const port = 8085
	go runServer(ctx, t, port)

	// Waiting for the server's getting available.
	// TODO(yugui) find a better way to wait
	time.Sleep(100 * time.Millisecond)

	url := fmt.Sprintf("http://localhost:%d/v1/example/echo/myid", port)
	resp, err := http.Get(url)
	if err != nil {
		t.Errorf("http.Post(%q) failed with %v; want success", url, err)
		return
	}
	defer resp.Body.Close()
	buf, err := ioutil.ReadAll(resp.Body)
	if err != nil {
		t.Errorf("ioutil.ReadAll(resp.Body) failed with %v; want success", err)
		return
	}

	if got, want := resp.StatusCode, http.StatusNotImplemented; got != want {
		t.Errorf("resp.StatusCode = %d; want %d", got, want)
		t.Logf("%s", buf)
	}

	var msg spb.Status
	if err := jsonpb.UnmarshalString(string(buf), &msg); err != nil {
		t.Errorf("jsonpb.UnmarshalString(%s, &msg) failed with %v; want success", buf, err)
		return
	}

	if got, want := msg.Code, int32(codes.Unimplemented); got != want {
		t.Errorf("msg.Code = %d; want %d", got, want)
		return
	}

	if msg.Message == "" {
		t.Errorf("msg.Message should not be empty")
		return
	}
}
