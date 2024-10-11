/*
 *
 * Copyright 2021 gRPC authors.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

package grpc

import (
	"context"
	"errors"
	"fmt"
	"net"
	"testing"
	"time"

	"github.com/google/go-cmp/cmp"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/internal"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/resolver"
)

func generateTarget(target string) resolver.Target {
	return resolver.Target{URL: *testutils.MustParseURL(target)}
}

// Resets the default scheme as though it was never set by the user.
func resetInitialResolverState() {
	resolver.SetDefaultScheme("passthrough")
	internal.UserSetDefaultScheme = false
}

type testResolverForParser struct {
	resolver.Resolver
}

func (testResolverForParser) Build(resolver.Target, resolver.ClientConn, resolver.BuildOptions) (resolver.Resolver, error) {
	return testResolverForParser{}, nil
}

func (testResolverForParser) Close() {}

func (testResolverForParser) Scheme() string {
	return "testresolverforparser"
}

func init() { resolver.Register(testResolverForParser{}) }

func (s) TestParsedTarget_Success_WithoutCustomDialer(t *testing.T) {
	tests := []struct {
		target             string
		wantDialParse      resolver.Target
		wantNewClientParse resolver.Target
		wantCustomParse    resolver.Target
	}{
		// No scheme is specified.
		{
			target:             "://a/b",
			wantDialParse:      generateTarget("passthrough:///://a/b"),
			wantNewClientParse: generateTarget("dns:///://a/b"),
			wantCustomParse:    generateTarget("testresolverforparser:///://a/b"),
		},
		{
			target:             "a//b",
			wantDialParse:      generateTarget("passthrough:///a//b"),
			wantNewClientParse: generateTarget("dns:///a//b"),
			wantCustomParse:    generateTarget("testresolverforparser:///a//b"),
		},

		// An unregistered scheme is specified.
		{
			target:             "a:///",
			wantDialParse:      generateTarget("passthrough:///a:///"),
			wantNewClientParse: generateTarget("dns:///a:///"),
			wantCustomParse:    generateTarget("testresolverforparser:///a:///"),
		},
		{
			target:             "a:b",
			wantDialParse:      generateTarget("passthrough:///a:b"),
			wantNewClientParse: generateTarget("dns:///a:b"),
			wantCustomParse:    generateTarget("testresolverforparser:///a:b"),
		},

		// A registered scheme is specified.
		{
			target:             "dns://a.server.com/google.com",
			wantDialParse:      generateTarget("dns://a.server.com/google.com"),
			wantNewClientParse: generateTarget("dns://a.server.com/google.com"),
			wantCustomParse:    generateTarget("dns://a.server.com/google.com"),
		},
		{
			target:             "unix-abstract:/ a///://::!@#$%25^&*()b",
			wantDialParse:      generateTarget("unix-abstract:/ a///://::!@#$%25^&*()b"),
			wantNewClientParse: generateTarget("unix-abstract:/ a///://::!@#$%25^&*()b"),
			wantCustomParse:    generateTarget("unix-abstract:/ a///://::!@#$%25^&*()b"),
		},
		{
			target:             "unix-abstract:passthrough:abc",
			wantDialParse:      generateTarget("unix-abstract:passthrough:abc"),
			wantNewClientParse: generateTarget("unix-abstract:passthrough:abc"),
			wantCustomParse:    generateTarget("unix-abstract:passthrough:abc"),
		},
		{
			target:             "passthrough:///unix:///a/b/c",
			wantDialParse:      generateTarget("passthrough:///unix:///a/b/c"),
			wantNewClientParse: generateTarget("passthrough:///unix:///a/b/c"),
			wantCustomParse:    generateTarget("passthrough:///unix:///a/b/c"),
		},

		// Cases for `scheme:absolute-path`.
		{
			target:             "dns:/a/b/c",
			wantDialParse:      generateTarget("dns:/a/b/c"),
			wantNewClientParse: generateTarget("dns:/a/b/c"),
			wantCustomParse:    generateTarget("dns:/a/b/c"),
		},
		{
			target:             "unregistered:/a/b/c",
			wantDialParse:      generateTarget("passthrough:///unregistered:/a/b/c"),
			wantNewClientParse: generateTarget("dns:///unregistered:/a/b/c"),
			wantCustomParse:    generateTarget("testresolverforparser:///unregistered:/a/b/c"),
		},
	}

	for _, test := range tests {
		t.Run(test.target, func(t *testing.T) {
			resetInitialResolverState()
			cc, err := Dial(test.target, WithTransportCredentials(insecure.NewCredentials()))
			if err != nil {
				t.Fatalf("Dial(%q) failed: %v", test.target, err)
			}
			cc.Close()

			if !cmp.Equal(cc.parsedTarget, test.wantDialParse) {
				t.Errorf("cc.parsedTarget for dial target %q = %+v, want %+v", test.target, cc.parsedTarget, test.wantDialParse)
			}

			cc, err = NewClient(test.target, WithTransportCredentials(insecure.NewCredentials()))
			if err != nil {
				t.Fatalf("NewClient(%q) failed: %v", test.target, err)
			}
			cc.Close()

			if !cmp.Equal(cc.parsedTarget, test.wantNewClientParse) {
				t.Errorf("cc.parsedTarget for newClient target %q = %+v, want %+v", test.target, cc.parsedTarget, test.wantNewClientParse)
			}

			resolver.SetDefaultScheme("testresolverforparser")
			cc, err = Dial(test.target, WithTransportCredentials(insecure.NewCredentials()))
			if err != nil {
				t.Fatalf("Dial(%q) failed: %v", test.target, err)
			}
			cc.Close()

			if !cmp.Equal(cc.parsedTarget, test.wantCustomParse) {
				t.Errorf("cc.parsedTarget for dial target %q = %+v, want %+v", test.target, cc.parsedTarget, test.wantDialParse)
			}

			cc, err = NewClient(test.target, WithTransportCredentials(insecure.NewCredentials()))
			if err != nil {
				t.Fatalf("NewClient(%q) failed: %v", test.target, err)
			}
			cc.Close()

			if !cmp.Equal(cc.parsedTarget, test.wantCustomParse) {
				t.Errorf("cc.parsedTarget for newClient target %q = %+v, want %+v", test.target, cc.parsedTarget, test.wantNewClientParse)
			}

		})
	}
	resetInitialResolverState()
}

func (s) TestParsedTarget_Failure_WithoutCustomDialer(t *testing.T) {
	targets := []string{
		"",
		"unix://a/b/c",
		"unix://authority",
		"unix-abstract://authority/a/b/c",
		"unix-abstract://authority",
	}

	for _, target := range targets {
		t.Run(target, func(t *testing.T) {
			if cc, err := Dial(target, WithTransportCredentials(insecure.NewCredentials())); err == nil {
				defer cc.Close()
				t.Fatalf("Dial(%q) succeeded cc.parsedTarget = %+v, expected to fail", target, cc.parsedTarget)
			}
		})
	}
}

func (s) TestParsedTarget_WithCustomDialer(t *testing.T) {
	resetInitialResolverState()
	defScheme := resolver.GetDefaultScheme()
	tests := []struct {
		target            string
		wantParsed        resolver.Target
		wantDialerAddress string
	}{
		// unix:[local_path], unix:[/absolute], and unix://[/absolute] have
		// different behaviors with a custom dialer.
		{
			target:            "unix:a/b/c",
			wantParsed:        resolver.Target{URL: *testutils.MustParseURL("unix:a/b/c")},
			wantDialerAddress: "unix:a/b/c",
		},
		{
			target:            "unix:/a/b/c",
			wantParsed:        resolver.Target{URL: *testutils.MustParseURL("unix:/a/b/c")},
			wantDialerAddress: "unix:///a/b/c",
		},
		{
			target:            "unix:///a/b/c",
			wantParsed:        resolver.Target{URL: *testutils.MustParseURL("unix:///a/b/c")},
			wantDialerAddress: "unix:///a/b/c",
		},
		{
			target:            "dns:///127.0.0.1:50051",
			wantParsed:        resolver.Target{URL: *testutils.MustParseURL("dns:///127.0.0.1:50051")},
			wantDialerAddress: "127.0.0.1:50051",
		},
		{
			target:            ":///127.0.0.1:50051",
			wantParsed:        resolver.Target{URL: *testutils.MustParseURL(fmt.Sprintf("%s:///%s", defScheme, ":///127.0.0.1:50051"))},
			wantDialerAddress: ":///127.0.0.1:50051",
		},
		{
			target:            "dns://authority/127.0.0.1:50051",
			wantParsed:        resolver.Target{URL: *testutils.MustParseURL("dns://authority/127.0.0.1:50051")},
			wantDialerAddress: "127.0.0.1:50051",
		},
		{
			target:            "://authority/127.0.0.1:50051",
			wantParsed:        resolver.Target{URL: *testutils.MustParseURL(fmt.Sprintf("%s:///%s", defScheme, "://authority/127.0.0.1:50051"))},
			wantDialerAddress: "://authority/127.0.0.1:50051",
		},
		{
			target:            "/unix/socket/address",
			wantParsed:        resolver.Target{URL: *testutils.MustParseURL(fmt.Sprintf("%s:///%s", defScheme, "/unix/socket/address"))},
			wantDialerAddress: "/unix/socket/address",
		},
		{
			target:            "",
			wantParsed:        resolver.Target{URL: *testutils.MustParseURL(fmt.Sprintf("%s:///%s", defScheme, ""))},
			wantDialerAddress: "",
		},
		{
			target:            "passthrough://a.server.com/google.com",
			wantParsed:        resolver.Target{URL: *testutils.MustParseURL("passthrough://a.server.com/google.com")},
			wantDialerAddress: "google.com",
		},
	}

	for _, test := range tests {
		t.Run(test.target, func(t *testing.T) {
			addrCh := make(chan string, 1)
			dialer := func(_ context.Context, address string) (net.Conn, error) {
				addrCh <- address
				return nil, errors.New("dialer error")
			}

			cc, err := Dial(test.target, WithTransportCredentials(insecure.NewCredentials()), WithContextDialer(dialer))
			if err != nil {
				t.Fatalf("Dial(%q) failed: %v", test.target, err)
			}
			defer cc.Close()

			select {
			case addr := <-addrCh:
				if addr != test.wantDialerAddress {
					t.Fatalf("address in custom dialer is %q, want %q", addr, test.wantDialerAddress)
				}
			case <-time.After(time.Second):
				t.Fatal("timeout when waiting for custom dialer to be invoked")
			}
			if !cmp.Equal(cc.parsedTarget, test.wantParsed) {
				t.Errorf("cc.parsedTarget for dial target %q = %+v, want %+v", test.target, cc.parsedTarget, test.wantParsed)
			}
		})
	}
}
