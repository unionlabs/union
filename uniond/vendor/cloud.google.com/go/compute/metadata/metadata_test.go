// Copyright 2016 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package metadata

import (
	"bytes"
	"context"
	"io"
	"log"
	"net/http"
	"os"
	"strings"
	"sync"
	"testing"
	"time"
)

func TestOnGCE_Stress(t *testing.T) {
	if testing.Short() {
		t.Skip("skipping in -short mode")
	}
	var last bool
	for i := 0; i < 100; i++ {
		onGCEOnce = sync.Once{}

		now := OnGCE()
		if i > 0 && now != last {
			t.Errorf("%d. changed from %v to %v", i, last, now)
		}
		last = now
	}
	t.Logf("OnGCE() = %v", last)
}

func TestOnGCE_Force(t *testing.T) {
	onGCEOnce = sync.Once{}
	old := os.Getenv(metadataHostEnv)
	defer os.Setenv(metadataHostEnv, old)
	os.Setenv(metadataHostEnv, "127.0.0.1")
	if !OnGCE() {
		t.Error("OnGCE() = false; want true")
	}
}

func TestOverrideUserAgent(t *testing.T) {
	ctx := context.Background()
	const userAgent = "my-user-agent"
	rt := &rrt{}
	c := NewClient(&http.Client{Transport: userAgentTransport{userAgent, rt}})
	c.GetWithContext(ctx, "foo")
	if got, want := rt.gotUserAgent, userAgent; got != want {
		t.Errorf("got %q, want %q", got, want)
	}
}

func TestGetFailsOnBadURL(t *testing.T) {
	ctx := context.Background()
	c := NewClient(http.DefaultClient)
	t.Setenv(metadataHostEnv, "host:-1")
	_, err := c.GetWithContext(ctx, "suffix")
	log.Printf("%v", err)
	if err == nil {
		t.Errorf("got %v, want non-nil error", err)
	}
}

func TestGet_LeadingSlash(t *testing.T) {
	want := "http://169.254.169.254/computeMetadata/v1/instance/service-accounts/default/identity?audience=http://example.com"
	tests := []struct {
		name   string
		suffix string
	}{
		{
			name:   "without leading slash",
			suffix: "instance/service-accounts/default/identity?audience=http://example.com",
		},
		{
			name:   "with leading slash",
			suffix: "/instance/service-accounts/default/identity?audience=http://example.com",
		},
	}
	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			ctx := context.Background()
			ct := &captureTransport{}
			c := NewClient(&http.Client{Transport: ct})
			_, _ = c.GetWithContext(ctx, tc.suffix)
			if ct.url != want {
				t.Fatalf("got %v, want %v", ct.url, want)
			}
		})
	}
}

type captureTransport struct {
	url string
}

func (ct *captureTransport) RoundTrip(req *http.Request) (*http.Response, error) {
	ct.url = req.URL.String()
	return &http.Response{Body: io.NopCloser(io.Reader(bytes.NewReader(nil)))}, nil
}

type userAgentTransport struct {
	userAgent string
	base      http.RoundTripper
}

func (t userAgentTransport) RoundTrip(req *http.Request) (*http.Response, error) {
	req.Header.Set("User-Agent", t.userAgent)
	return t.base.RoundTrip(req)
}

type rrt struct {
	gotUserAgent string
}

func (r *rrt) RoundTrip(req *http.Request) (*http.Response, error) {
	r.gotUserAgent = req.Header.Get("User-Agent")
	return &http.Response{Body: io.NopCloser(bytes.NewReader(nil))}, nil
}

func TestRetry(t *testing.T) {
	tests := []struct {
		name        string
		timesToFail int
		failCode    int
		failErr     error
		response    string
		expectError bool
	}{
		{
			name:     "no retries",
			response: "test",
		},
		{
			name:        "retry 500 once",
			response:    "test",
			failCode:    500,
			timesToFail: 1,
		},
		{
			name:        "retry io.ErrUnexpectedEOF once",
			response:    "test",
			failErr:     io.ErrUnexpectedEOF,
			timesToFail: 1,
		},
		{
			name:        "retry io.ErrUnexpectedEOF permanent",
			failErr:     io.ErrUnexpectedEOF,
			timesToFail: maxRetryAttempts + 1,
			expectError: true,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			ctx := context.Background()
			ft := &failingTransport{
				timesToFail: tt.timesToFail,
				failCode:    tt.failCode,
				failErr:     tt.failErr,
				response:    tt.response,
			}
			c := NewClient(&http.Client{Transport: ft})
			s, err := c.GetWithContext(ctx, "")
			if tt.expectError && err == nil {
				t.Fatalf("did not receive expected error")
			} else if !tt.expectError && err != nil {
				t.Fatalf("unexpected error: %v", err)
			}

			expectedCount := ft.failedAttempts + 1
			if tt.expectError {
				expectedCount = ft.failedAttempts
			} else if s != tt.response {
				// Responses are only meaningful if err == nil
				t.Fatalf("c.Get() = %q, want %q", s, tt.response)
			}

			if ft.called != expectedCount {
				t.Fatalf("failed %d times, want %d", ft.called, expectedCount)
			}
		})
	}
}

func TestClientGetWithContext(t *testing.T) {
	tests := []struct {
		name       string
		ctxTimeout time.Duration
		wantErr    bool
	}{
		{
			name:       "ok",
			ctxTimeout: 1 * time.Second,
		},
		{
			name:       "times out",
			ctxTimeout: 200 * time.Millisecond,
			wantErr:    true,
		},
	}
	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			ctx, cancel := context.WithTimeout(context.Background(), tc.ctxTimeout)
			defer cancel()
			c := NewClient(&http.Client{Transport: sleepyTransport{}})
			_, err := c.GetWithContext(ctx, "foo")
			if tc.wantErr && err == nil {
				t.Fatal("c.GetWithContext() == nil, want an error")
			}
			if !tc.wantErr && err != nil {
				t.Fatalf("c.GetWithContext() = %v, want nil", err)
			}
		})
	}
}

type sleepyTransport struct {
}

func (s sleepyTransport) RoundTrip(req *http.Request) (*http.Response, error) {
	req.Context().Done()
	select {
	case <-req.Context().Done():
		return nil, req.Context().Err()
	case <-time.After(500 * time.Millisecond):
	}
	return &http.Response{StatusCode: http.StatusOK, Body: io.NopCloser(strings.NewReader("I woke up"))}, nil
}

type failingTransport struct {
	timesToFail int
	failCode    int
	failErr     error
	response    string

	failedAttempts int
	called         int
}

func (r *failingTransport) RoundTrip(req *http.Request) (*http.Response, error) {
	r.called++
	if r.failedAttempts < r.timesToFail {
		r.failedAttempts++
		if r.failErr != nil {
			return nil, r.failErr
		}
		return &http.Response{StatusCode: r.failCode}, nil
	}
	return &http.Response{StatusCode: http.StatusOK, Body: io.NopCloser(strings.NewReader(r.response))}, nil
}
