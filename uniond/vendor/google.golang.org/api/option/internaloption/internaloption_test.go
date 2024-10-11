// Copyright 2021 Google LLC.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package internaloption

import (
	"testing"

	"github.com/google/go-cmp/cmp"
	"github.com/google/go-cmp/cmp/cmpopts"
	"golang.org/x/oauth2"
	"golang.org/x/oauth2/google"
	"google.golang.org/api/internal"
	"google.golang.org/api/option"
	"google.golang.org/grpc"
)

func TestWithCredentials(t *testing.T) {
	want := "fakeToken"
	fakeCreds := &google.Credentials{
		TokenSource: oauth2.StaticTokenSource(&oauth2.Token{AccessToken: want}),
	}
	opt := WithCredentials(fakeCreds)
	ds := &internal.DialSettings{}
	opt.Apply(ds)
	if ds.InternalCredentials == nil || ds.InternalCredentials.TokenSource == nil {
		t.Errorf("ds.InternalCredentials should be initialized")
	}
	if tok, err := ds.InternalCredentials.TokenSource.Token(); err != nil || tok.AccessToken != "fakeToken" {
		t.Errorf("ts.Token() = %q, want %q", tok.AccessToken, "")
	}
}

func TestDefaultApply(t *testing.T) {
	opts := []option.ClientOption{
		WithDefaultEndpoint("https://example.com:443"),
		WithDefaultEndpointTemplate("https://foo.UNIVERSE_DOMAIN/"),
		WithDefaultMTLSEndpoint("http://mtls.example.com:445"),
		WithDefaultScopes("a"),
		WithDefaultUniverseDomain("foo.com"),
		WithDefaultAudience("audience"),
	}
	var got internal.DialSettings
	for _, opt := range opts {
		opt.Apply(&got)
	}
	want := internal.DialSettings{
		DefaultScopes:           []string{"a"},
		DefaultEndpoint:         "https://example.com:443",
		DefaultEndpointTemplate: "https://foo.UNIVERSE_DOMAIN/",
		DefaultUniverseDomain:   "foo.com",
		DefaultAudience:         "audience",
		DefaultMTLSEndpoint:     "http://mtls.example.com:445",
	}
	ignore := []cmp.Option{
		cmpopts.IgnoreUnexported(grpc.ClientConn{}),
		cmpopts.IgnoreFields(google.Credentials{}, "udMu", "universeDomain"),
	}
	if !cmp.Equal(got, want, ignore...) {
		t.Errorf(cmp.Diff(got, want, ignore...))
	}
}
