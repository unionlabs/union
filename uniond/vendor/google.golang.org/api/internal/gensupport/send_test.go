// Copyright 2017 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package gensupport

import (
	"context"
	"errors"
	"fmt"
	"net/http"
	"regexp"
	"testing"

	"github.com/google/go-cmp/cmp"
	"github.com/googleapis/gax-go/v2/callctx"
)

func TestSendRequest(t *testing.T) {
	// Setting Accept-Encoding should give an error immediately.
	req, _ := http.NewRequest("GET", "url", nil)
	req.Header.Set("Accept-Encoding", "")
	_, err := SendRequest(context.Background(), nil, req)
	if err == nil {
		t.Error("got nil, want error")
	}
}

func TestSendRequestWithRetry(t *testing.T) {
	// Setting Accept-Encoding should give an error immediately.
	req, _ := http.NewRequest("GET", "url", nil)
	req.Header.Set("Accept-Encoding", "")
	_, err := SendRequestWithRetry(context.Background(), nil, req, nil)
	if err == nil {
		t.Error("got nil, want error")
	}
}

type headerRoundTripper struct {
	wantHeader        http.Header
	wantXgoogAPIRegex string // test x-goog-api-client separately
}

func (rt *headerRoundTripper) RoundTrip(r *http.Request) (*http.Response, error) {
	// Test x-goog-api-client with a regex, since invocation ids are randomly generated
	match, err := regexp.MatchString(rt.wantXgoogAPIRegex, r.Header.Get("X-Goog-Api-Client"))
	if err != nil {
		return nil, fmt.Errorf("compiling regexp: %v", err)
	}
	if !match {
		return nil, fmt.Errorf("X-Goog-Api-Client header has wrong format\ngot %v\nwant regex matching %v", r.Header.Get("X-Goog-Api-Client"), rt.wantXgoogAPIRegex)
	}

	// Ignore x-goog headers sent by SendRequestWithRetry
	r.Header.Del("X-Goog-Gcs-Idempotency-Token")
	r.Header.Del("X-Goog-Api-Client") // this was tested above already

	if diff := cmp.Diff(r.Header, rt.wantHeader); diff != "" {
		return nil, fmt.Errorf("headers don't match: %v", diff)
	}
	return &http.Response{StatusCode: 200}, nil
}

// Ensure that headers set via the context are passed through to the request as expected.
func TestSendRequestHeader(t *testing.T) {
	ctx := context.Background()
	ctx = callctx.SetHeaders(ctx, "foo", "100", "bar", "200")
	transport := &headerRoundTripper{
		wantHeader: map[string][]string{"Foo": {"100"}, "Bar": {"200"}},
	}
	client := http.Client{Transport: transport}

	req, _ := http.NewRequest("GET", "url", nil)
	if _, err := SendRequest(ctx, &client, req); err != nil {
		t.Errorf("SendRequest: %v", err)
	}

	// SendRequestWithRetry adds retry and idempotency headers
	transport.wantXgoogAPIRegex = "^gccl-invocation-id/.{36} gccl-attempt-count/[0-9]+ $"
	req2, _ := http.NewRequest("GET", "url", nil)
	if _, err := SendRequestWithRetry(ctx, &client, req2, nil); err != nil {
		t.Errorf("SendRequestWithRetry: %v", err)
	}
}

// Ensure that x-goog-api-client headers set via the context are merged properly
// and passed through to the request as expected.
func TestSendRequestXgoogHeaderxxx(t *testing.T) {
	ctx := context.Background()
	ctx = callctx.SetHeaders(ctx, "x-goog-api-client", "val/1", "bar", "200", "x-goog-api-client", "val/2")
	ctx = callctx.SetHeaders(ctx, "x-goog-api-client", "val/11 val/22")

	transport := &headerRoundTripper{
		wantHeader:        map[string][]string{"Bar": {"200"}},
		wantXgoogAPIRegex: "^val/1 val/2 val/11 val/22$",
	}
	client := http.Client{Transport: transport}

	req, _ := http.NewRequest("GET", "url", nil)
	if _, err := SendRequest(ctx, &client, req); err != nil {
		t.Errorf("SendRequest: %v", err)
	}

	// SendRequestWithRetry adds retry and idempotency headers
	transport.wantXgoogAPIRegex = fmt.Sprintf("^gccl-invocation-id/.{36} gccl-attempt-count/[0-9]+ %s$", transport.wantXgoogAPIRegex[1:len(transport.wantXgoogAPIRegex)-1])
	req2, _ := http.NewRequest("GET", "url", nil)
	if _, err := SendRequestWithRetry(ctx, &client, req2, nil); err != nil {
		t.Errorf("SendRequestWithRetry: %v", err)
	}
}

type brokenRoundTripper struct{}

func (t *brokenRoundTripper) RoundTrip(r *http.Request) (*http.Response, error) {
	return nil, errors.New("this should not happen")
}

func TestCanceledContextDoesNotPerformRequest(t *testing.T) {
	client := http.Client{
		Transport: &brokenRoundTripper{},
	}
	for i := 0; i < 1000; i++ {
		req, _ := http.NewRequest("GET", "url", nil)
		ctx, cancel := context.WithCancel(context.Background())
		cancel()
		_, err := SendRequestWithRetry(ctx, &client, req, nil)
		if !errors.Is(err, context.Canceled) {
			t.Fatalf("got %v, want %v", err, context.Canceled)
		}
	}
}
