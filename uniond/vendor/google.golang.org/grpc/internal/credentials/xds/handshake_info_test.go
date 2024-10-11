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

package xds

import (
	"crypto/x509"
	"net"
	"net/url"
	"regexp"
	"testing"

	"google.golang.org/grpc/internal/xds/matcher"
)

func TestDNSMatch(t *testing.T) {
	tests := []struct {
		desc      string
		host      string
		pattern   string
		wantMatch bool
	}{
		{
			desc:      "invalid wildcard 1",
			host:      "aa.example.com",
			pattern:   "*a.example.com",
			wantMatch: false,
		},
		{
			desc:      "invalid wildcard 2",
			host:      "aa.example.com",
			pattern:   "a*.example.com",
			wantMatch: false,
		},
		{
			desc:      "invalid wildcard 3",
			host:      "abc.example.com",
			pattern:   "a*c.example.com",
			wantMatch: false,
		},
		{
			desc:      "wildcard in one of the middle components",
			host:      "abc.test.example.com",
			pattern:   "abc.*.example.com",
			wantMatch: false,
		},
		{
			desc:      "single component wildcard",
			host:      "a.example.com",
			pattern:   "*",
			wantMatch: false,
		},
		{
			desc:      "short host name",
			host:      "a.com",
			pattern:   "*.example.com",
			wantMatch: false,
		},
		{
			desc:      "suffix mismatch",
			host:      "a.notexample.com",
			pattern:   "*.example.com",
			wantMatch: false,
		},
		{
			desc:      "wildcard match across components",
			host:      "sub.test.example.com",
			pattern:   "*.example.com.",
			wantMatch: false,
		},
		{
			desc:      "host doesn't end in period",
			host:      "test.example.com",
			pattern:   "test.example.com.",
			wantMatch: true,
		},
		{
			desc:      "pattern doesn't end in period",
			host:      "test.example.com.",
			pattern:   "test.example.com",
			wantMatch: true,
		},
		{
			desc:      "case insensitive",
			host:      "TEST.EXAMPLE.COM.",
			pattern:   "test.example.com.",
			wantMatch: true,
		},
		{
			desc:      "simple match",
			host:      "test.example.com",
			pattern:   "test.example.com",
			wantMatch: true,
		},
		{
			desc:      "good wildcard",
			host:      "a.example.com",
			pattern:   "*.example.com",
			wantMatch: true,
		},
	}

	for _, test := range tests {
		t.Run(test.desc, func(t *testing.T) {
			gotMatch := dnsMatch(test.host, test.pattern)
			if gotMatch != test.wantMatch {
				t.Fatalf("dnsMatch(%s, %s) = %v, want %v", test.host, test.pattern, gotMatch, test.wantMatch)
			}
		})
	}
}

func TestMatchingSANExists_FailureCases(t *testing.T) {
	url1, err := url.Parse("http://golang.org")
	if err != nil {
		t.Fatalf("url.Parse() failed: %v", err)
	}
	url2, err := url.Parse("https://github.com/grpc/grpc-go")
	if err != nil {
		t.Fatalf("url.Parse() failed: %v", err)
	}
	inputCert := &x509.Certificate{
		DNSNames:       []string{"foo.bar.example.com", "bar.baz.test.com", "*.example.com"},
		EmailAddresses: []string{"foobar@example.com", "barbaz@test.com"},
		IPAddresses:    []net.IP{net.ParseIP("192.0.0.1"), net.ParseIP("2001:db8::68")},
		URIs:           []*url.URL{url1, url2},
	}

	tests := []struct {
		desc        string
		sanMatchers []matcher.StringMatcher
	}{
		{
			desc: "exact match",
			sanMatchers: []matcher.StringMatcher{
				matcher.StringMatcherForTesting(newStringP("abcd.test.com"), nil, nil, nil, nil, false),
				matcher.StringMatcherForTesting(newStringP("http://golang"), nil, nil, nil, nil, false),
				matcher.StringMatcherForTesting(newStringP("HTTP://GOLANG.ORG"), nil, nil, nil, nil, false),
			},
		},
		{
			desc: "prefix match",
			sanMatchers: []matcher.StringMatcher{
				matcher.StringMatcherForTesting(nil, newStringP("i-aint-the-one"), nil, nil, nil, false),
				matcher.StringMatcherForTesting(nil, newStringP("192.168.1.1"), nil, nil, nil, false),
				matcher.StringMatcherForTesting(nil, newStringP("FOO.BAR"), nil, nil, nil, false),
			},
		},
		{
			desc: "suffix match",
			sanMatchers: []matcher.StringMatcher{
				matcher.StringMatcherForTesting(nil, nil, newStringP("i-aint-the-one"), nil, nil, false),
				matcher.StringMatcherForTesting(nil, nil, newStringP("1::68"), nil, nil, false),
				matcher.StringMatcherForTesting(nil, nil, newStringP(".COM"), nil, nil, false),
			},
		},
		{
			desc: "regex match",
			sanMatchers: []matcher.StringMatcher{
				matcher.StringMatcherForTesting(nil, nil, nil, nil, regexp.MustCompile(`.*\.examples\.com`), false),
				matcher.StringMatcherForTesting(nil, nil, nil, nil, regexp.MustCompile(`192\.[0-9]{1,3}\.1\.1`), false),
			},
		},
		{
			desc: "contains match",
			sanMatchers: []matcher.StringMatcher{
				matcher.StringMatcherForTesting(nil, nil, nil, newStringP("i-aint-the-one"), nil, false),
				matcher.StringMatcherForTesting(nil, nil, nil, newStringP("2001:db8:1:1::68"), nil, false),
				matcher.StringMatcherForTesting(nil, nil, nil, newStringP("GRPC"), nil, false),
			},
		},
	}

	for _, test := range tests {
		t.Run(test.desc, func(t *testing.T) {
			hi := NewHandshakeInfo(nil, nil, test.sanMatchers, false)

			if hi.MatchingSANExists(inputCert) {
				t.Fatalf("hi.MatchingSANExists(%+v) with SAN matchers +%v succeeded when expected to fail", inputCert, test.sanMatchers)
			}
		})
	}
}

func TestMatchingSANExists_Success(t *testing.T) {
	url1, err := url.Parse("http://golang.org")
	if err != nil {
		t.Fatalf("url.Parse() failed: %v", err)
	}
	url2, err := url.Parse("https://github.com/grpc/grpc-go")
	if err != nil {
		t.Fatalf("url.Parse() failed: %v", err)
	}
	inputCert := &x509.Certificate{
		DNSNames:       []string{"baz.test.com", "*.example.com"},
		EmailAddresses: []string{"foobar@example.com", "barbaz@test.com"},
		IPAddresses:    []net.IP{net.ParseIP("192.0.0.1"), net.ParseIP("2001:db8::68")},
		URIs:           []*url.URL{url1, url2},
	}

	tests := []struct {
		desc        string
		sanMatchers []matcher.StringMatcher
	}{
		{
			desc: "no san matchers",
		},
		{
			desc: "exact match dns wildcard",
			sanMatchers: []matcher.StringMatcher{
				matcher.StringMatcherForTesting(nil, newStringP("192.168.1.1"), nil, nil, nil, false),
				matcher.StringMatcherForTesting(newStringP("https://github.com/grpc/grpc-java"), nil, nil, nil, nil, false),
				matcher.StringMatcherForTesting(newStringP("abc.example.com"), nil, nil, nil, nil, false),
			},
		},
		{
			desc: "exact match ignore case",
			sanMatchers: []matcher.StringMatcher{
				matcher.StringMatcherForTesting(newStringP("FOOBAR@EXAMPLE.COM"), nil, nil, nil, nil, true),
			},
		},
		{
			desc: "prefix match",
			sanMatchers: []matcher.StringMatcher{
				matcher.StringMatcherForTesting(nil, nil, newStringP(".co.in"), nil, nil, false),
				matcher.StringMatcherForTesting(nil, newStringP("192.168.1.1"), nil, nil, nil, false),
				matcher.StringMatcherForTesting(nil, newStringP("baz.test"), nil, nil, nil, false),
			},
		},
		{
			desc: "prefix match ignore case",
			sanMatchers: []matcher.StringMatcher{
				matcher.StringMatcherForTesting(nil, newStringP("BAZ.test"), nil, nil, nil, true),
			},
		},
		{
			desc: "suffix  match",
			sanMatchers: []matcher.StringMatcher{
				matcher.StringMatcherForTesting(nil, nil, nil, nil, regexp.MustCompile(`192\.[0-9]{1,3}\.1\.1`), false),
				matcher.StringMatcherForTesting(nil, nil, newStringP("192.168.1.1"), nil, nil, false),
				matcher.StringMatcherForTesting(nil, nil, newStringP("@test.com"), nil, nil, false),
			},
		},
		{
			desc: "suffix  match ignore case",
			sanMatchers: []matcher.StringMatcher{
				matcher.StringMatcherForTesting(nil, nil, newStringP("@test.COM"), nil, nil, true),
			},
		},
		{
			desc: "regex match",
			sanMatchers: []matcher.StringMatcher{
				matcher.StringMatcherForTesting(nil, nil, nil, newStringP("https://github.com/grpc/grpc-java"), nil, false),
				matcher.StringMatcherForTesting(nil, nil, nil, nil, regexp.MustCompile(`192\.[0-9]{1,3}\.1\.1`), false),
				matcher.StringMatcherForTesting(nil, nil, nil, nil, regexp.MustCompile(`.*\.test\.com`), false),
			},
		},
		{
			desc: "contains match",
			sanMatchers: []matcher.StringMatcher{
				matcher.StringMatcherForTesting(newStringP("https://github.com/grpc/grpc-java"), nil, nil, nil, nil, false),
				matcher.StringMatcherForTesting(nil, nil, nil, newStringP("2001:68::db8"), nil, false),
				matcher.StringMatcherForTesting(nil, nil, nil, newStringP("192.0.0"), nil, false),
			},
		},
		{
			desc: "contains match ignore case",
			sanMatchers: []matcher.StringMatcher{
				matcher.StringMatcherForTesting(nil, nil, nil, newStringP("GRPC"), nil, true),
			},
		},
	}

	for _, test := range tests {
		t.Run(test.desc, func(t *testing.T) {
			hi := NewHandshakeInfo(nil, nil, test.sanMatchers, false)

			if !hi.MatchingSANExists(inputCert) {
				t.Fatalf("hi.MatchingSANExists(%+v) with SAN matchers +%v failed when expected to succeed", inputCert, test.sanMatchers)
			}
		})
	}
}

func newStringP(s string) *string {
	return &s
}
