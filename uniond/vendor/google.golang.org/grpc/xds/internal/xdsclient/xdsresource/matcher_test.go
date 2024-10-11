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
 */

package xdsresource

import (
	"context"
	"math/rand"
	"testing"

	"github.com/google/go-cmp/cmp"
	"google.golang.org/grpc/internal/grpcutil"
	iresolver "google.golang.org/grpc/internal/resolver"
	"google.golang.org/grpc/internal/xds/matcher"
	"google.golang.org/grpc/metadata"
	"google.golang.org/protobuf/proto"
)

func (s) TestAndMatcherMatch(t *testing.T) {
	tests := []struct {
		name string
		pm   pathMatcher
		hm   matcher.HeaderMatcher
		info iresolver.RPCInfo
		want bool
	}{
		{
			name: "both match",
			pm:   newPathExactMatcher("/a/b", false),
			hm:   matcher.NewHeaderExactMatcher("th", "tv", false),
			info: iresolver.RPCInfo{
				Method:  "/a/b",
				Context: metadata.NewOutgoingContext(context.Background(), metadata.Pairs("th", "tv")),
			},
			want: true,
		},
		{
			name: "both match with path case insensitive",
			pm:   newPathExactMatcher("/A/B", true),
			hm:   matcher.NewHeaderExactMatcher("th", "tv", false),
			info: iresolver.RPCInfo{
				Method:  "/a/b",
				Context: metadata.NewOutgoingContext(context.Background(), metadata.Pairs("th", "tv")),
			},
			want: true,
		},
		{
			name: "only one match",
			pm:   newPathExactMatcher("/a/b", false),
			hm:   matcher.NewHeaderExactMatcher("th", "tv", false),
			info: iresolver.RPCInfo{
				Method:  "/z/y",
				Context: metadata.NewOutgoingContext(context.Background(), metadata.Pairs("th", "tv")),
			},
			want: false,
		},
		{
			name: "both not match",
			pm:   newPathExactMatcher("/z/y", false),
			hm:   matcher.NewHeaderExactMatcher("th", "abc", false),
			info: iresolver.RPCInfo{
				Method:  "/a/b",
				Context: metadata.NewOutgoingContext(context.Background(), metadata.Pairs("th", "tv")),
			},
			want: false,
		},
		{
			name: "fake header",
			pm:   newPathPrefixMatcher("/", false),
			hm:   matcher.NewHeaderExactMatcher("content-type", "fake", false),
			info: iresolver.RPCInfo{
				Method: "/a/b",
				Context: grpcutil.WithExtraMetadata(context.Background(), metadata.Pairs(
					"content-type", "fake",
				)),
			},
			want: true,
		},
		{
			name: "binary header",
			pm:   newPathPrefixMatcher("/", false),
			hm:   matcher.NewHeaderPresentMatcher("t-bin", true, false),
			info: iresolver.RPCInfo{
				Method: "/a/b",
				Context: grpcutil.WithExtraMetadata(
					metadata.NewOutgoingContext(context.Background(), metadata.Pairs("t-bin", "123")), metadata.Pairs(
						"content-type", "fake",
					)),
			},
			// Shouldn't match binary header, even though it's in metadata.
			want: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			a := newCompositeMatcher(tt.pm, []matcher.HeaderMatcher{tt.hm}, nil)
			if got := a.Match(tt.info); got != tt.want {
				t.Errorf("match() = %v, want %v", got, tt.want)
			}
		})
	}
}

func (s) TestFractionMatcherMatch(t *testing.T) {
	const fraction = 500000
	fm := newFractionMatcher(fraction)
	defer func() {
		RandInt63n = rand.Int63n
	}()

	// rand > fraction, should return false.
	RandInt63n = func(int64) int64 {
		return fraction + 1
	}
	if matched := fm.match(); matched {
		t.Errorf("match() = %v, want not match", matched)
	}

	// rand == fraction, should return true.
	RandInt63n = func(int64) int64 {
		return fraction
	}
	if matched := fm.match(); !matched {
		t.Errorf("match() = %v, want match", matched)
	}

	// rand < fraction, should return true.
	RandInt63n = func(int64) int64 {
		return fraction - 1
	}
	if matched := fm.match(); !matched {
		t.Errorf("match() = %v, want match", matched)
	}
}

func (s) TestMatchTypeForDomain(t *testing.T) {
	tests := []struct {
		d    string
		want domainMatchType
	}{
		{d: "", want: domainMatchTypeInvalid},
		{d: "*", want: domainMatchTypeUniversal},
		{d: "bar.*", want: domainMatchTypePrefix},
		{d: "*.abc.com", want: domainMatchTypeSuffix},
		{d: "foo.bar.com", want: domainMatchTypeExact},
		{d: "foo.*.com", want: domainMatchTypeInvalid},
	}
	for _, tt := range tests {
		if got := matchTypeForDomain(tt.d); got != tt.want {
			t.Errorf("matchTypeForDomain(%q) = %v, want %v", tt.d, got, tt.want)
		}
	}
}

func (s) TestMatch(t *testing.T) {
	tests := []struct {
		name        string
		domain      string
		host        string
		wantTyp     domainMatchType
		wantMatched bool
	}{
		{name: "invalid-empty", domain: "", host: "", wantTyp: domainMatchTypeInvalid, wantMatched: false},
		{name: "invalid", domain: "a.*.b", host: "", wantTyp: domainMatchTypeInvalid, wantMatched: false},
		{name: "universal", domain: "*", host: "abc.com", wantTyp: domainMatchTypeUniversal, wantMatched: true},
		{name: "prefix-match", domain: "abc.*", host: "abc.123", wantTyp: domainMatchTypePrefix, wantMatched: true},
		{name: "prefix-no-match", domain: "abc.*", host: "abcd.123", wantTyp: domainMatchTypePrefix, wantMatched: false},
		{name: "suffix-match", domain: "*.123", host: "abc.123", wantTyp: domainMatchTypeSuffix, wantMatched: true},
		{name: "suffix-no-match", domain: "*.123", host: "abc.1234", wantTyp: domainMatchTypeSuffix, wantMatched: false},
		{name: "exact-match", domain: "foo.bar", host: "foo.bar", wantTyp: domainMatchTypeExact, wantMatched: true},
		{name: "exact-no-match", domain: "foo.bar.com", host: "foo.bar", wantTyp: domainMatchTypeExact, wantMatched: false},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if gotTyp, gotMatched := match(tt.domain, tt.host); gotTyp != tt.wantTyp || gotMatched != tt.wantMatched {
				t.Errorf("match() = %v, %v, want %v, %v", gotTyp, gotMatched, tt.wantTyp, tt.wantMatched)
			}
		})
	}
}

func (s) TestFindBestMatchingVirtualHost(t *testing.T) {
	var (
		oneExactMatch     = &VirtualHost{Domains: []string{"foo.bar.com"}}
		oneSuffixMatch    = &VirtualHost{Domains: []string{"*.bar.com"}}
		onePrefixMatch    = &VirtualHost{Domains: []string{"foo.bar.*"}}
		oneUniversalMatch = &VirtualHost{Domains: []string{"*"}}
		longExactMatch    = &VirtualHost{Domains: []string{"v2.foo.bar.com"}}
		multipleMatch     = &VirtualHost{Domains: []string{"pi.foo.bar.com", "314.*", "*.159"}}
		vhs               = []*VirtualHost{oneExactMatch, oneSuffixMatch, onePrefixMatch, oneUniversalMatch, longExactMatch, multipleMatch}
	)

	tests := []struct {
		name   string
		host   string
		vHosts []*VirtualHost
		want   *VirtualHost
	}{
		{name: "exact-match", host: "foo.bar.com", vHosts: vhs, want: oneExactMatch},
		{name: "suffix-match", host: "123.bar.com", vHosts: vhs, want: oneSuffixMatch},
		{name: "prefix-match", host: "foo.bar.org", vHosts: vhs, want: onePrefixMatch},
		{name: "universal-match", host: "abc.123", vHosts: vhs, want: oneUniversalMatch},
		{name: "long-exact-match", host: "v2.foo.bar.com", vHosts: vhs, want: longExactMatch},
		// Matches suffix "*.bar.com" and exact "pi.foo.bar.com". Takes exact.
		{name: "multiple-match-exact", host: "pi.foo.bar.com", vHosts: vhs, want: multipleMatch},
		// Matches suffix "*.159" and prefix "foo.bar.*". Takes suffix.
		{name: "multiple-match-suffix", host: "foo.bar.159", vHosts: vhs, want: multipleMatch},
		// Matches suffix "*.bar.com" and prefix "314.*". Takes suffix.
		{name: "multiple-match-prefix", host: "314.bar.com", vHosts: vhs, want: oneSuffixMatch},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := FindBestMatchingVirtualHost(tt.host, tt.vHosts); !cmp.Equal(got, tt.want, cmp.Comparer(proto.Equal)) {
				t.Errorf("FindBestMatchingxdsclient.VirtualHost() = %v, want %v", got, tt.want)
			}
		})
	}
}
