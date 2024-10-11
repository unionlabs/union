/*
 *
 * Copyright 2020 gRPC authors.
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

package resolver

import (
	"context"
	"regexp"
	"testing"

	xxhash "github.com/cespare/xxhash/v2"
	"github.com/google/go-cmp/cmp"
	"google.golang.org/grpc/internal/grpctest"
	"google.golang.org/grpc/internal/grpcutil"
	iresolver "google.golang.org/grpc/internal/resolver"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/metadata"
	_ "google.golang.org/grpc/xds/internal/balancer/cdsbalancer" // To parse LB config
	"google.golang.org/grpc/xds/internal/xdsclient/xdsresource"
)

type s struct {
	grpctest.Tester
}

func Test(t *testing.T) {
	grpctest.RunSubTests(t, s{})
}

func (s) TestPruneActiveClusters(t *testing.T) {
	r := &xdsResolver{activeClusters: map[string]*clusterInfo{
		"zero":        {refCount: 0},
		"one":         {refCount: 1},
		"two":         {refCount: 2},
		"anotherzero": {refCount: 0},
	}}
	want := map[string]*clusterInfo{
		"one": {refCount: 1},
		"two": {refCount: 2},
	}
	r.pruneActiveClusters()
	if d := cmp.Diff(r.activeClusters, want, cmp.AllowUnexported(clusterInfo{})); d != "" {
		t.Fatalf("r.activeClusters = %v; want %v\nDiffs: %v", r.activeClusters, want, d)
	}
}

func (s) TestGenerateRequestHash(t *testing.T) {
	const channelID = 12378921
	cs := &configSelector{
		r: &xdsResolver{
			cc:        &testutils.ResolverClientConn{Logger: t},
			channelID: channelID,
		},
	}
	tests := []struct {
		name            string
		hashPolicies    []*xdsresource.HashPolicy
		requestHashWant uint64
		rpcInfo         iresolver.RPCInfo
	}{
		// TestGenerateRequestHashHeaders tests generating request hashes for
		// hash policies that specify to hash headers.
		{
			name: "test-generate-request-hash-headers",
			hashPolicies: []*xdsresource.HashPolicy{{
				HashPolicyType:    xdsresource.HashPolicyTypeHeader,
				HeaderName:        ":path",
				Regex:             func() *regexp.Regexp { return regexp.MustCompile("/products") }(), // Will replace /products with /new-products, to test find and replace functionality.
				RegexSubstitution: "/new-products",
			}},
			requestHashWant: xxhash.Sum64String("/new-products"),
			rpcInfo: iresolver.RPCInfo{
				Context: metadata.NewOutgoingContext(context.Background(), metadata.Pairs(":path", "/products")),
				Method:  "/some-method",
			},
		},
		// TestGenerateHashChannelID tests generating request hashes for hash
		// policies that specify to hash something that uniquely identifies the
		// ClientConn (the pointer).
		{
			name: "test-generate-request-hash-channel-id",
			hashPolicies: []*xdsresource.HashPolicy{{
				HashPolicyType: xdsresource.HashPolicyTypeChannelID,
			}},
			requestHashWant: channelID,
			rpcInfo:         iresolver.RPCInfo{},
		},
		// TestGenerateRequestHashEmptyString tests generating request hashes
		// for hash policies that specify to hash headers and replace empty
		// strings in the headers.
		{
			name: "test-generate-request-hash-empty-string",
			hashPolicies: []*xdsresource.HashPolicy{{
				HashPolicyType:    xdsresource.HashPolicyTypeHeader,
				HeaderName:        ":path",
				Regex:             func() *regexp.Regexp { return regexp.MustCompile("") }(),
				RegexSubstitution: "e",
			}},
			requestHashWant: xxhash.Sum64String("eaebece"),
			rpcInfo: iresolver.RPCInfo{
				Context: metadata.NewOutgoingContext(context.Background(), metadata.Pairs(":path", "abc")),
				Method:  "/some-method",
			},
		},
		// Tests that bin headers are skipped.
		{
			name: "skip-bin",
			hashPolicies: []*xdsresource.HashPolicy{{
				HashPolicyType: xdsresource.HashPolicyTypeHeader,
				HeaderName:     "something-bin",
			}, {
				HashPolicyType: xdsresource.HashPolicyTypeChannelID,
			}},
			requestHashWant: channelID,
			rpcInfo: iresolver.RPCInfo{
				Context: metadata.NewOutgoingContext(context.Background(), metadata.Pairs("something-bin", "xyz")),
			},
		},
		// Tests that extra metadata takes precedence over the user's metadata.
		{
			name: "extra-metadata",
			hashPolicies: []*xdsresource.HashPolicy{{
				HashPolicyType: xdsresource.HashPolicyTypeHeader,
				HeaderName:     "content-type",
			}},
			requestHashWant: xxhash.Sum64String("grpc value"),
			rpcInfo: iresolver.RPCInfo{
				Context: grpcutil.WithExtraMetadata(
					metadata.NewOutgoingContext(context.Background(), metadata.Pairs("content-type", "user value")),
					metadata.Pairs("content-type", "grpc value"),
				),
			},
		},
	}
	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			requestHashGot := cs.generateHash(test.rpcInfo, test.hashPolicies)
			if requestHashGot != test.requestHashWant {
				t.Fatalf("requestHashGot = %v, requestHashWant = %v", requestHashGot, test.requestHashWant)
			}
		})
	}
}
