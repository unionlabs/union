// Copyright 2016 Google LLC.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package grpc

import (
	"bytes"
	"context"
	"log"
	"strings"
	"testing"

	"cloud.google.com/go/compute/metadata"
	"github.com/google/go-cmp/cmp"
	"golang.org/x/oauth2/google"
	"google.golang.org/api/internal"
	"google.golang.org/grpc"
)

func TestDial(t *testing.T) {
	oldDialContext := dialContext
	// Replace package var in order to assert DialContext args.
	dialContext = func(ctxGot context.Context, target string, opts ...grpc.DialOption) (conn *grpc.ClientConn, err error) {
		if len(opts) != 4 {
			t.Fatalf("got: %d, want: 4", len(opts))
		}
		return nil, nil
	}
	defer func() {
		dialContext = oldDialContext
	}()

	var o internal.DialSettings
	dial(context.Background(), false, &o)
}

func TestCheckDirectPathEndPoint(t *testing.T) {
	for _, testcase := range []struct {
		name     string
		endpoint string
		want     bool
	}{
		{
			name:     "empty endpoint are disallowed",
			endpoint: "",
			want:     false,
		},
		{
			name:     "dns schemes are allowed",
			endpoint: "dns:///foo",
			want:     true,
		},
		{
			name:     "host without no prefix are allowed",
			endpoint: "foo",
			want:     true,
		},
		{
			name:     "host with port are allowed",
			endpoint: "foo:1234",
			want:     true,
		},
		{
			name:     "non-dns schemes are disallowed",
			endpoint: "https://foo",
			want:     false,
		},
	} {
		t.Run(testcase.name, func(t *testing.T) {
			if got := checkDirectPathEndPoint(testcase.endpoint); got != testcase.want {
				t.Fatalf("got %v, want %v", got, testcase.want)
			}
		})
	}
}

func TestLogDirectPathMisconfigAttrempDirectPathNotSet(t *testing.T) {
	o := &internal.DialSettings{}
	o.EnableDirectPathXds = true

	endpoint := "abc.googleapis.com"

	creds, err := internal.Creds(context.Context(context.Background()), o)
	if err != nil {
		t.Fatalf("failed to create creds")
	}

	buf := bytes.Buffer{}
	log.SetOutput(&buf)

	logDirectPathMisconfig(endpoint, creds.TokenSource, o)

	wantedLog := "WARNING: DirectPath is misconfigured. Please set the EnableDirectPath option along with the EnableDirectPathXds option."
	if !strings.Contains(buf.String(), wantedLog) {
		t.Fatalf("got: %v, want: %v", buf.String(), wantedLog)
	}

}

func TestLogDirectPathMisconfigWrongCredential(t *testing.T) {
	o := &internal.DialSettings{}
	o.EnableDirectPath = true
	o.EnableDirectPathXds = true

	endpoint := "abc.googleapis.com"

	creds := &google.Credentials{}

	buf := bytes.Buffer{}
	log.SetOutput(&buf)

	logDirectPathMisconfig(endpoint, creds.TokenSource, o)

	wantedLog := "WARNING: DirectPath is misconfigured. Please make sure the token source is fetched from GCE metadata server and the default service account is used."
	if !strings.Contains(buf.String(), wantedLog) {
		t.Fatalf("got: %v, want: %v", buf.String(), wantedLog)
	}

}

func TestLogDirectPathMisconfigNotOnGCE(t *testing.T) {
	o := &internal.DialSettings{}
	o.EnableDirectPath = true
	o.EnableDirectPathXds = true

	endpoint := "abc.googleapis.com"

	creds, err := internal.Creds(context.Context(context.Background()), o)
	if err != nil {
		t.Fatalf("failed to create creds")
	}

	buf := bytes.Buffer{}
	log.SetOutput(&buf)

	logDirectPathMisconfig(endpoint, creds.TokenSource, o)

	if !metadata.OnGCE() {
		wantedLog := "WARNING: DirectPath is misconfigured. DirectPath is only available in a GCE environment."
		if !strings.Contains(buf.String(), wantedLog) {
			t.Fatalf("got: %v, want: %v", buf.String(), wantedLog)
		}
	}

}

func TestGRPCAPIKey_GetRequestMetadata(t *testing.T) {
	for _, test := range []struct {
		apiKey string
		reason string
	}{
		{
			apiKey: "MY_API_KEY",
			reason: "MY_REQUEST_REASON",
		},
	} {
		ts := grpcAPIKey{
			apiKey:        test.apiKey,
			requestReason: test.reason,
		}
		got, err := ts.GetRequestMetadata(context.Background())
		if err != nil {
			t.Fatal(err)
		}
		want := map[string]string{
			"X-goog-api-key":        ts.apiKey,
			"X-goog-request-reason": ts.requestReason,
		}
		if diff := cmp.Diff(want, got); diff != "" {
			t.Errorf("mismatch (-want +got):\n%s", diff)
		}
	}
}
