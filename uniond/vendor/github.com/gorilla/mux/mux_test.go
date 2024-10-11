// Copyright 2012 The Gorilla Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package mux

import (
	"bufio"
	"bytes"
	"context"
	"errors"
	"fmt"
	"io"
	"log"
	"net/http"
	"net/http/httptest"
	"net/url"
	"reflect"
	"strings"
	"testing"
	"time"
)

func (r *Route) GoString() string {
	matchers := make([]string, len(r.matchers))
	for i, m := range r.matchers {
		matchers[i] = fmt.Sprintf("%#v", m)
	}
	return fmt.Sprintf("&Route{matchers:[]matcher{%s}}", strings.Join(matchers, ", "))
}

func (r *routeRegexp) GoString() string {
	return fmt.Sprintf("&routeRegexp{template: %q, regexpType: %v, options: %v, regexp: regexp.MustCompile(%q), reverse: %q, varsN: %v, varsR: %v", r.template, r.regexpType, r.options, r.regexp.String(), r.reverse, r.varsN, r.varsR)
}

type routeTest struct {
	title           string            // title of the test
	route           *Route            // the route being tested
	request         *http.Request     // a request to test the route
	vars            map[string]string // the expected vars of the match
	scheme          string            // the expected scheme of the built URL
	host            string            // the expected host of the built URL
	path            string            // the expected path of the built URL
	query           string            // the expected query string of the built URL
	pathTemplate    string            // the expected path template of the route
	hostTemplate    string            // the expected host template of the route
	queriesTemplate string            // the expected query template of the route
	methods         []string          // the expected route methods
	pathRegexp      string            // the expected path regexp
	queriesRegexp   string            // the expected query regexp
	shouldMatch     bool              // whether the request is expected to match the route at all
	shouldRedirect  bool              // whether the request should result in a redirect
}

func TestHost(t *testing.T) {

	tests := []routeTest{
		{
			title:       "Host route match",
			route:       new(Route).Host("aaa.bbb.ccc"),
			request:     newRequest("GET", "http://aaa.bbb.ccc/111/222/333"),
			vars:        map[string]string{},
			host:        "aaa.bbb.ccc",
			path:        "",
			shouldMatch: true,
		},
		{
			title:       "Host route, wrong host in request URL",
			route:       new(Route).Host("aaa.bbb.ccc"),
			request:     newRequest("GET", "http://aaa.222.ccc/111/222/333"),
			vars:        map[string]string{},
			host:        "aaa.bbb.ccc",
			path:        "",
			shouldMatch: false,
		},
		{
			title:       "Host route with port, match",
			route:       new(Route).Host("aaa.bbb.ccc:1234"),
			request:     newRequest("GET", "http://aaa.bbb.ccc:1234/111/222/333"),
			vars:        map[string]string{},
			host:        "aaa.bbb.ccc:1234",
			path:        "",
			shouldMatch: true,
		},
		{
			title:       "Host route with port, wrong port in request URL",
			route:       new(Route).Host("aaa.bbb.ccc:1234"),
			request:     newRequest("GET", "http://aaa.bbb.ccc:9999/111/222/333"),
			vars:        map[string]string{},
			host:        "aaa.bbb.ccc:1234",
			path:        "",
			shouldMatch: false,
		},
		{
			title:       "Host route, match with host in request header",
			route:       new(Route).Host("aaa.bbb.ccc"),
			request:     newRequestHost("GET", "/111/222/333", "aaa.bbb.ccc"),
			vars:        map[string]string{},
			host:        "aaa.bbb.ccc",
			path:        "",
			shouldMatch: true,
		},
		{
			title:       "Host route, wrong host in request header",
			route:       new(Route).Host("aaa.bbb.ccc"),
			request:     newRequestHost("GET", "/111/222/333", "aaa.222.ccc"),
			vars:        map[string]string{},
			host:        "aaa.bbb.ccc",
			path:        "",
			shouldMatch: false,
		},
		{
			title:       "Host route with port, match with request header",
			route:       new(Route).Host("aaa.bbb.ccc:1234"),
			request:     newRequestHost("GET", "/111/222/333", "aaa.bbb.ccc:1234"),
			vars:        map[string]string{},
			host:        "aaa.bbb.ccc:1234",
			path:        "",
			shouldMatch: true,
		},
		{
			title:       "Host route with port, wrong host in request header",
			route:       new(Route).Host("aaa.bbb.ccc:1234"),
			request:     newRequestHost("GET", "/111/222/333", "aaa.bbb.ccc:9999"),
			vars:        map[string]string{},
			host:        "aaa.bbb.ccc:1234",
			path:        "",
			shouldMatch: false,
		},
		{
			title:        "Host route with pattern, match with request header",
			route:        new(Route).Host("aaa.{v1:[a-z]{3}}.ccc:1{v2:(?:23|4)}"),
			request:      newRequestHost("GET", "/111/222/333", "aaa.bbb.ccc:123"),
			vars:         map[string]string{"v1": "bbb", "v2": "23"},
			host:         "aaa.bbb.ccc:123",
			path:         "",
			hostTemplate: `aaa.{v1:[a-z]{3}}.ccc:1{v2:(?:23|4)}`,
			shouldMatch:  true,
		},
		{
			title:        "Host route with pattern, match",
			route:        new(Route).Host("aaa.{v1:[a-z]{3}}.ccc"),
			request:      newRequest("GET", "http://aaa.bbb.ccc/111/222/333"),
			vars:         map[string]string{"v1": "bbb"},
			host:         "aaa.bbb.ccc",
			path:         "",
			hostTemplate: `aaa.{v1:[a-z]{3}}.ccc`,
			shouldMatch:  true,
		},
		{
			title:        "Host route with pattern, additional capturing group, match",
			route:        new(Route).Host("aaa.{v1:[a-z]{2}(?:b|c)}.ccc"),
			request:      newRequest("GET", "http://aaa.bbb.ccc/111/222/333"),
			vars:         map[string]string{"v1": "bbb"},
			host:         "aaa.bbb.ccc",
			path:         "",
			hostTemplate: `aaa.{v1:[a-z]{2}(?:b|c)}.ccc`,
			shouldMatch:  true,
		},
		{
			title:        "Host route with pattern, wrong host in request URL",
			route:        new(Route).Host("aaa.{v1:[a-z]{3}}.ccc"),
			request:      newRequest("GET", "http://aaa.222.ccc/111/222/333"),
			vars:         map[string]string{"v1": "bbb"},
			host:         "aaa.bbb.ccc",
			path:         "",
			hostTemplate: `aaa.{v1:[a-z]{3}}.ccc`,
			shouldMatch:  false,
		},
		{
			title:        "Host route with multiple patterns, match",
			route:        new(Route).Host("{v1:[a-z]{3}}.{v2:[a-z]{3}}.{v3:[a-z]{3}}"),
			request:      newRequest("GET", "http://aaa.bbb.ccc/111/222/333"),
			vars:         map[string]string{"v1": "aaa", "v2": "bbb", "v3": "ccc"},
			host:         "aaa.bbb.ccc",
			path:         "",
			hostTemplate: `{v1:[a-z]{3}}.{v2:[a-z]{3}}.{v3:[a-z]{3}}`,
			shouldMatch:  true,
		},
		{
			title:        "Host route with multiple patterns, wrong host in request URL",
			route:        new(Route).Host("{v1:[a-z]{3}}.{v2:[a-z]{3}}.{v3:[a-z]{3}}"),
			request:      newRequest("GET", "http://aaa.222.ccc/111/222/333"),
			vars:         map[string]string{"v1": "aaa", "v2": "bbb", "v3": "ccc"},
			host:         "aaa.bbb.ccc",
			path:         "",
			hostTemplate: `{v1:[a-z]{3}}.{v2:[a-z]{3}}.{v3:[a-z]{3}}`,
			shouldMatch:  false,
		},
		{
			title:        "Host route with hyphenated name and pattern, match",
			route:        new(Route).Host("aaa.{v-1:[a-z]{3}}.ccc"),
			request:      newRequest("GET", "http://aaa.bbb.ccc/111/222/333"),
			vars:         map[string]string{"v-1": "bbb"},
			host:         "aaa.bbb.ccc",
			path:         "",
			hostTemplate: `aaa.{v-1:[a-z]{3}}.ccc`,
			shouldMatch:  true,
		},
		{
			title:        "Host route with hyphenated name and pattern, additional capturing group, match",
			route:        new(Route).Host("aaa.{v-1:[a-z]{2}(?:b|c)}.ccc"),
			request:      newRequest("GET", "http://aaa.bbb.ccc/111/222/333"),
			vars:         map[string]string{"v-1": "bbb"},
			host:         "aaa.bbb.ccc",
			path:         "",
			hostTemplate: `aaa.{v-1:[a-z]{2}(?:b|c)}.ccc`,
			shouldMatch:  true,
		},
		{
			title:        "Host route with multiple hyphenated names and patterns, match",
			route:        new(Route).Host("{v-1:[a-z]{3}}.{v-2:[a-z]{3}}.{v-3:[a-z]{3}}"),
			request:      newRequest("GET", "http://aaa.bbb.ccc/111/222/333"),
			vars:         map[string]string{"v-1": "aaa", "v-2": "bbb", "v-3": "ccc"},
			host:         "aaa.bbb.ccc",
			path:         "",
			hostTemplate: `{v-1:[a-z]{3}}.{v-2:[a-z]{3}}.{v-3:[a-z]{3}}`,
			shouldMatch:  true,
		},
	}
	for _, test := range tests {
		t.Run(test.title, func(t *testing.T) {
			testRoute(t, test)
			testTemplate(t, test)
		})
	}
}

func TestPath(t *testing.T) {
	tests := []routeTest{
		{
			title:       "Path route, match",
			route:       new(Route).Path("/111/222/333"),
			request:     newRequest("GET", "http://localhost/111/222/333"),
			vars:        map[string]string{},
			host:        "",
			path:        "/111/222/333",
			shouldMatch: true,
		},
		{
			title:       "Path route, match with trailing slash in request and path",
			route:       new(Route).Path("/111/"),
			request:     newRequest("GET", "http://localhost/111/"),
			vars:        map[string]string{},
			host:        "",
			path:        "/111/",
			shouldMatch: true,
		},
		{
			title:        "Path route, do not match with trailing slash in path",
			route:        new(Route).Path("/111/"),
			request:      newRequest("GET", "http://localhost/111"),
			vars:         map[string]string{},
			host:         "",
			path:         "/111",
			pathTemplate: `/111/`,
			pathRegexp:   `^/111/$`,
			shouldMatch:  false,
		},
		{
			title:        "Path route, do not match with trailing slash in request",
			route:        new(Route).Path("/111"),
			request:      newRequest("GET", "http://localhost/111/"),
			vars:         map[string]string{},
			host:         "",
			path:         "/111/",
			pathTemplate: `/111`,
			shouldMatch:  false,
		},
		{
			title:        "Path route, match root with no host",
			route:        new(Route).Path("/"),
			request:      newRequest("GET", "/"),
			vars:         map[string]string{},
			host:         "",
			path:         "/",
			pathTemplate: `/`,
			pathRegexp:   `^/$`,
			shouldMatch:  true,
		},
		{
			title: "Path route, match root with no host, App Engine format",
			route: new(Route).Path("/"),
			request: func() *http.Request {
				r := newRequest("GET", "http://localhost/")
				r.RequestURI = "/"
				return r
			}(),
			vars:         map[string]string{},
			host:         "",
			path:         "/",
			pathTemplate: `/`,
			shouldMatch:  true,
		},
		{
			title:       "Path route, wrong path in request in request URL",
			route:       new(Route).Path("/111/222/333"),
			request:     newRequest("GET", "http://localhost/1/2/3"),
			vars:        map[string]string{},
			host:        "",
			path:        "/111/222/333",
			shouldMatch: false,
		},
		{
			title:        "Path route with pattern, match",
			route:        new(Route).Path("/111/{v1:[0-9]{3}}/333"),
			request:      newRequest("GET", "http://localhost/111/222/333"),
			vars:         map[string]string{"v1": "222"},
			host:         "",
			path:         "/111/222/333",
			pathTemplate: `/111/{v1:[0-9]{3}}/333`,
			shouldMatch:  true,
		},
		{
			title:        "Path route with pattern, URL in request does not match",
			route:        new(Route).Path("/111/{v1:[0-9]{3}}/333"),
			request:      newRequest("GET", "http://localhost/111/aaa/333"),
			vars:         map[string]string{"v1": "222"},
			host:         "",
			path:         "/111/222/333",
			pathTemplate: `/111/{v1:[0-9]{3}}/333`,
			pathRegexp:   `^/111/(?P<v0>[0-9]{3})/333$`,
			shouldMatch:  false,
		},
		{
			title:        "Path route with multiple patterns, match",
			route:        new(Route).Path("/{v1:[0-9]{3}}/{v2:[0-9]{3}}/{v3:[0-9]{3}}"),
			request:      newRequest("GET", "http://localhost/111/222/333"),
			vars:         map[string]string{"v1": "111", "v2": "222", "v3": "333"},
			host:         "",
			path:         "/111/222/333",
			pathTemplate: `/{v1:[0-9]{3}}/{v2:[0-9]{3}}/{v3:[0-9]{3}}`,
			pathRegexp:   `^/(?P<v0>[0-9]{3})/(?P<v1>[0-9]{3})/(?P<v2>[0-9]{3})$`,
			shouldMatch:  true,
		},
		{
			title:        "Path route with multiple patterns, URL in request does not match",
			route:        new(Route).Path("/{v1:[0-9]{3}}/{v2:[0-9]{3}}/{v3:[0-9]{3}}"),
			request:      newRequest("GET", "http://localhost/111/aaa/333"),
			vars:         map[string]string{"v1": "111", "v2": "222", "v3": "333"},
			host:         "",
			path:         "/111/222/333",
			pathTemplate: `/{v1:[0-9]{3}}/{v2:[0-9]{3}}/{v3:[0-9]{3}}`,
			pathRegexp:   `^/(?P<v0>[0-9]{3})/(?P<v1>[0-9]{3})/(?P<v2>[0-9]{3})$`,
			shouldMatch:  false,
		},
		{
			title:        "Path route with multiple patterns with pipe, match",
			route:        new(Route).Path("/{category:a|(?:b/c)}/{product}/{id:[0-9]+}"),
			request:      newRequest("GET", "http://localhost/a/product_name/1"),
			vars:         map[string]string{"category": "a", "product": "product_name", "id": "1"},
			host:         "",
			path:         "/a/product_name/1",
			pathTemplate: `/{category:a|(?:b/c)}/{product}/{id:[0-9]+}`,
			pathRegexp:   `^/(?P<v0>a|(?:b/c))/(?P<v1>[^/]+)/(?P<v2>[0-9]+)$`,
			shouldMatch:  true,
		},
		{
			title:        "Path route with hyphenated name and pattern, match",
			route:        new(Route).Path("/111/{v-1:[0-9]{3}}/333"),
			request:      newRequest("GET", "http://localhost/111/222/333"),
			vars:         map[string]string{"v-1": "222"},
			host:         "",
			path:         "/111/222/333",
			pathTemplate: `/111/{v-1:[0-9]{3}}/333`,
			pathRegexp:   `^/111/(?P<v0>[0-9]{3})/333$`,
			shouldMatch:  true,
		},
		{
			title:        "Path route with multiple hyphenated names and patterns, match",
			route:        new(Route).Path("/{v-1:[0-9]{3}}/{v-2:[0-9]{3}}/{v-3:[0-9]{3}}"),
			request:      newRequest("GET", "http://localhost/111/222/333"),
			vars:         map[string]string{"v-1": "111", "v-2": "222", "v-3": "333"},
			host:         "",
			path:         "/111/222/333",
			pathTemplate: `/{v-1:[0-9]{3}}/{v-2:[0-9]{3}}/{v-3:[0-9]{3}}`,
			pathRegexp:   `^/(?P<v0>[0-9]{3})/(?P<v1>[0-9]{3})/(?P<v2>[0-9]{3})$`,
			shouldMatch:  true,
		},
		{
			title:        "Path route with multiple hyphenated names and patterns with pipe, match",
			route:        new(Route).Path("/{product-category:a|(?:b/c)}/{product-name}/{product-id:[0-9]+}"),
			request:      newRequest("GET", "http://localhost/a/product_name/1"),
			vars:         map[string]string{"product-category": "a", "product-name": "product_name", "product-id": "1"},
			host:         "",
			path:         "/a/product_name/1",
			pathTemplate: `/{product-category:a|(?:b/c)}/{product-name}/{product-id:[0-9]+}`,
			pathRegexp:   `^/(?P<v0>a|(?:b/c))/(?P<v1>[^/]+)/(?P<v2>[0-9]+)$`,
			shouldMatch:  true,
		},
		{
			title:        "Path route with multiple hyphenated names and patterns with pipe and case insensitive, match",
			route:        new(Route).Path("/{type:(?i:daily|mini|variety)}-{date:\\d{4,4}-\\d{2,2}-\\d{2,2}}"),
			request:      newRequest("GET", "http://localhost/daily-2016-01-01"),
			vars:         map[string]string{"type": "daily", "date": "2016-01-01"},
			host:         "",
			path:         "/daily-2016-01-01",
			pathTemplate: `/{type:(?i:daily|mini|variety)}-{date:\d{4,4}-\d{2,2}-\d{2,2}}`,
			pathRegexp:   `^/(?P<v0>(?i:daily|mini|variety))-(?P<v1>\d{4,4}-\d{2,2}-\d{2,2})$`,
			shouldMatch:  true,
		},
		{
			title:        "Path route with empty match right after other match",
			route:        new(Route).Path(`/{v1:[0-9]*}{v2:[a-z]*}/{v3:[0-9]*}`),
			request:      newRequest("GET", "http://localhost/111/222"),
			vars:         map[string]string{"v1": "111", "v2": "", "v3": "222"},
			host:         "",
			path:         "/111/222",
			pathTemplate: `/{v1:[0-9]*}{v2:[a-z]*}/{v3:[0-9]*}`,
			pathRegexp:   `^/(?P<v0>[0-9]*)(?P<v1>[a-z]*)/(?P<v2>[0-9]*)$`,
			shouldMatch:  true,
		},
		{
			title:        "Path route with single pattern with pipe, match",
			route:        new(Route).Path("/{category:a|b/c}"),
			request:      newRequest("GET", "http://localhost/a"),
			vars:         map[string]string{"category": "a"},
			host:         "",
			path:         "/a",
			pathTemplate: `/{category:a|b/c}`,
			shouldMatch:  true,
		},
		{
			title:        "Path route with single pattern with pipe, match",
			route:        new(Route).Path("/{category:a|b/c}"),
			request:      newRequest("GET", "http://localhost/b/c"),
			vars:         map[string]string{"category": "b/c"},
			host:         "",
			path:         "/b/c",
			pathTemplate: `/{category:a|b/c}`,
			shouldMatch:  true,
		},
		{
			title:        "Path route with multiple patterns with pipe, match",
			route:        new(Route).Path("/{category:a|b/c}/{product}/{id:[0-9]+}"),
			request:      newRequest("GET", "http://localhost/a/product_name/1"),
			vars:         map[string]string{"category": "a", "product": "product_name", "id": "1"},
			host:         "",
			path:         "/a/product_name/1",
			pathTemplate: `/{category:a|b/c}/{product}/{id:[0-9]+}`,
			shouldMatch:  true,
		},
		{
			title:        "Path route with multiple patterns with pipe, match",
			route:        new(Route).Path("/{category:a|b/c}/{product}/{id:[0-9]+}"),
			request:      newRequest("GET", "http://localhost/b/c/product_name/1"),
			vars:         map[string]string{"category": "b/c", "product": "product_name", "id": "1"},
			host:         "",
			path:         "/b/c/product_name/1",
			pathTemplate: `/{category:a|b/c}/{product}/{id:[0-9]+}`,
			shouldMatch:  true,
		},
	}

	for _, test := range tests {
		t.Run(test.title, func(t *testing.T) {
			testRoute(t, test)
			testTemplate(t, test)
			testUseEscapedRoute(t, test)
			testRegexp(t, test)
		})
	}
}

func TestPathPrefix(t *testing.T) {
	tests := []routeTest{
		{
			title:       "PathPrefix route, match",
			route:       new(Route).PathPrefix("/111"),
			request:     newRequest("GET", "http://localhost/111/222/333"),
			vars:        map[string]string{},
			host:        "",
			path:        "/111",
			shouldMatch: true,
		},
		{
			title:       "PathPrefix route, match substring",
			route:       new(Route).PathPrefix("/1"),
			request:     newRequest("GET", "http://localhost/111/222/333"),
			vars:        map[string]string{},
			host:        "",
			path:        "/1",
			shouldMatch: true,
		},
		{
			title:       "PathPrefix route, URL prefix in request does not match",
			route:       new(Route).PathPrefix("/111"),
			request:     newRequest("GET", "http://localhost/1/2/3"),
			vars:        map[string]string{},
			host:        "",
			path:        "/111",
			shouldMatch: false,
		},
		{
			title:        "PathPrefix route with pattern, match",
			route:        new(Route).PathPrefix("/111/{v1:[0-9]{3}}"),
			request:      newRequest("GET", "http://localhost/111/222/333"),
			vars:         map[string]string{"v1": "222"},
			host:         "",
			path:         "/111/222",
			pathTemplate: `/111/{v1:[0-9]{3}}`,
			shouldMatch:  true,
		},
		{
			title:        "PathPrefix route with pattern, URL prefix in request does not match",
			route:        new(Route).PathPrefix("/111/{v1:[0-9]{3}}"),
			request:      newRequest("GET", "http://localhost/111/aaa/333"),
			vars:         map[string]string{"v1": "222"},
			host:         "",
			path:         "/111/222",
			pathTemplate: `/111/{v1:[0-9]{3}}`,
			shouldMatch:  false,
		},
		{
			title:        "PathPrefix route with multiple patterns, match",
			route:        new(Route).PathPrefix("/{v1:[0-9]{3}}/{v2:[0-9]{3}}"),
			request:      newRequest("GET", "http://localhost/111/222/333"),
			vars:         map[string]string{"v1": "111", "v2": "222"},
			host:         "",
			path:         "/111/222",
			pathTemplate: `/{v1:[0-9]{3}}/{v2:[0-9]{3}}`,
			shouldMatch:  true,
		},
		{
			title:        "PathPrefix route with multiple patterns, URL prefix in request does not match",
			route:        new(Route).PathPrefix("/{v1:[0-9]{3}}/{v2:[0-9]{3}}"),
			request:      newRequest("GET", "http://localhost/111/aaa/333"),
			vars:         map[string]string{"v1": "111", "v2": "222"},
			host:         "",
			path:         "/111/222",
			pathTemplate: `/{v1:[0-9]{3}}/{v2:[0-9]{3}}`,
			shouldMatch:  false,
		},
	}

	for _, test := range tests {
		t.Run(test.title, func(t *testing.T) {
			testRoute(t, test)
			testTemplate(t, test)
			testUseEscapedRoute(t, test)
		})
	}
}

func TestSchemeHostPath(t *testing.T) {
	tests := []routeTest{
		{
			title:        "Host and Path route, match",
			route:        new(Route).Host("aaa.bbb.ccc").Path("/111/222/333"),
			request:      newRequest("GET", "http://aaa.bbb.ccc/111/222/333"),
			vars:         map[string]string{},
			scheme:       "http",
			host:         "aaa.bbb.ccc",
			path:         "/111/222/333",
			pathTemplate: `/111/222/333`,
			hostTemplate: `aaa.bbb.ccc`,
			shouldMatch:  true,
		},
		{
			title:        "Scheme, Host, and Path route, match",
			route:        new(Route).Schemes("https").Host("aaa.bbb.ccc").Path("/111/222/333"),
			request:      newRequest("GET", "https://aaa.bbb.ccc/111/222/333"),
			vars:         map[string]string{},
			scheme:       "https",
			host:         "aaa.bbb.ccc",
			path:         "/111/222/333",
			pathTemplate: `/111/222/333`,
			hostTemplate: `aaa.bbb.ccc`,
			shouldMatch:  true,
		},
		{
			title:        "Host and Path route, wrong host in request URL",
			route:        new(Route).Host("aaa.bbb.ccc").Path("/111/222/333"),
			request:      newRequest("GET", "http://aaa.222.ccc/111/222/333"),
			vars:         map[string]string{},
			scheme:       "http",
			host:         "aaa.bbb.ccc",
			path:         "/111/222/333",
			pathTemplate: `/111/222/333`,
			hostTemplate: `aaa.bbb.ccc`,
			shouldMatch:  false,
		},
		{
			title:        "Host and Path route with pattern, match",
			route:        new(Route).Host("aaa.{v1:[a-z]{3}}.ccc").Path("/111/{v2:[0-9]{3}}/333"),
			request:      newRequest("GET", "http://aaa.bbb.ccc/111/222/333"),
			vars:         map[string]string{"v1": "bbb", "v2": "222"},
			scheme:       "http",
			host:         "aaa.bbb.ccc",
			path:         "/111/222/333",
			pathTemplate: `/111/{v2:[0-9]{3}}/333`,
			hostTemplate: `aaa.{v1:[a-z]{3}}.ccc`,
			shouldMatch:  true,
		},
		{
			title:        "Scheme, Host, and Path route with host and path patterns, match",
			route:        new(Route).Schemes("ftp", "ssss").Host("aaa.{v1:[a-z]{3}}.ccc").Path("/111/{v2:[0-9]{3}}/333"),
			request:      newRequest("GET", "ssss://aaa.bbb.ccc/111/222/333"),
			vars:         map[string]string{"v1": "bbb", "v2": "222"},
			scheme:       "ftp",
			host:         "aaa.bbb.ccc",
			path:         "/111/222/333",
			pathTemplate: `/111/{v2:[0-9]{3}}/333`,
			hostTemplate: `aaa.{v1:[a-z]{3}}.ccc`,
			shouldMatch:  true,
		},
		{
			title:        "Host and Path route with pattern, URL in request does not match",
			route:        new(Route).Host("aaa.{v1:[a-z]{3}}.ccc").Path("/111/{v2:[0-9]{3}}/333"),
			request:      newRequest("GET", "http://aaa.222.ccc/111/222/333"),
			vars:         map[string]string{"v1": "bbb", "v2": "222"},
			scheme:       "http",
			host:         "aaa.bbb.ccc",
			path:         "/111/222/333",
			pathTemplate: `/111/{v2:[0-9]{3}}/333`,
			hostTemplate: `aaa.{v1:[a-z]{3}}.ccc`,
			shouldMatch:  false,
		},
		{
			title:        "Host and Path route with multiple patterns, match",
			route:        new(Route).Host("{v1:[a-z]{3}}.{v2:[a-z]{3}}.{v3:[a-z]{3}}").Path("/{v4:[0-9]{3}}/{v5:[0-9]{3}}/{v6:[0-9]{3}}"),
			request:      newRequest("GET", "http://aaa.bbb.ccc/111/222/333"),
			vars:         map[string]string{"v1": "aaa", "v2": "bbb", "v3": "ccc", "v4": "111", "v5": "222", "v6": "333"},
			scheme:       "http",
			host:         "aaa.bbb.ccc",
			path:         "/111/222/333",
			pathTemplate: `/{v4:[0-9]{3}}/{v5:[0-9]{3}}/{v6:[0-9]{3}}`,
			hostTemplate: `{v1:[a-z]{3}}.{v2:[a-z]{3}}.{v3:[a-z]{3}}`,
			shouldMatch:  true,
		},
		{
			title:        "Host and Path route with multiple patterns, URL in request does not match",
			route:        new(Route).Host("{v1:[a-z]{3}}.{v2:[a-z]{3}}.{v3:[a-z]{3}}").Path("/{v4:[0-9]{3}}/{v5:[0-9]{3}}/{v6:[0-9]{3}}"),
			request:      newRequest("GET", "http://aaa.222.ccc/111/222/333"),
			vars:         map[string]string{"v1": "aaa", "v2": "bbb", "v3": "ccc", "v4": "111", "v5": "222", "v6": "333"},
			scheme:       "http",
			host:         "aaa.bbb.ccc",
			path:         "/111/222/333",
			pathTemplate: `/{v4:[0-9]{3}}/{v5:[0-9]{3}}/{v6:[0-9]{3}}`,
			hostTemplate: `{v1:[a-z]{3}}.{v2:[a-z]{3}}.{v3:[a-z]{3}}`,
			shouldMatch:  false,
		},
	}

	for _, test := range tests {
		t.Run(test.title, func(t *testing.T) {
			testRoute(t, test)
			testTemplate(t, test)
			testUseEscapedRoute(t, test)
		})
	}
}

func TestHeaders(t *testing.T) {
	// newRequestHeaders creates a new request with a method, url, and headers
	newRequestHeaders := func(method, url string, headers map[string]string) *http.Request {
		req, err := http.NewRequest(method, url, nil)
		if err != nil {
			panic(err)
		}
		for k, v := range headers {
			req.Header.Add(k, v)
		}
		return req
	}

	tests := []routeTest{
		{
			title:       "Headers route, match",
			route:       new(Route).Headers("foo", "bar", "baz", "ding"),
			request:     newRequestHeaders("GET", "http://localhost", map[string]string{"foo": "bar", "baz": "ding"}),
			vars:        map[string]string{},
			host:        "",
			path:        "",
			shouldMatch: true,
		},
		{
			title:       "Headers route, bad header values",
			route:       new(Route).Headers("foo", "bar", "baz", "ding"),
			request:     newRequestHeaders("GET", "http://localhost", map[string]string{"foo": "bar", "baz": "dong"}),
			vars:        map[string]string{},
			host:        "",
			path:        "",
			shouldMatch: false,
		},
		{
			title:       "Headers route, regex header values to match",
			route:       new(Route).HeadersRegexp("foo", "ba[zr]"),
			request:     newRequestHeaders("GET", "http://localhost", map[string]string{"foo": "baw"}),
			vars:        map[string]string{},
			host:        "",
			path:        "",
			shouldMatch: false,
		},
		{
			title:       "Headers route, regex header values to match",
			route:       new(Route).HeadersRegexp("foo", "ba[zr]"),
			request:     newRequestHeaders("GET", "http://localhost", map[string]string{"foo": "baz"}),
			vars:        map[string]string{},
			host:        "",
			path:        "",
			shouldMatch: true,
		},
	}

	for _, test := range tests {
		t.Run(test.title, func(t *testing.T) {
			testRoute(t, test)
			testTemplate(t, test)
		})
	}
}

func TestMethods(t *testing.T) {
	tests := []routeTest{
		{
			title:       "Methods route, match GET",
			route:       new(Route).Methods("GET", "POST"),
			request:     newRequest("GET", "http://localhost"),
			vars:        map[string]string{},
			host:        "",
			path:        "",
			methods:     []string{"GET", "POST"},
			shouldMatch: true,
		},
		{
			title:       "Methods route, match POST",
			route:       new(Route).Methods("GET", "POST"),
			request:     newRequest("POST", "http://localhost"),
			vars:        map[string]string{},
			host:        "",
			path:        "",
			methods:     []string{"GET", "POST"},
			shouldMatch: true,
		},
		{
			title:       "Methods route, bad method",
			route:       new(Route).Methods("GET", "POST"),
			request:     newRequest("PUT", "http://localhost"),
			vars:        map[string]string{},
			host:        "",
			path:        "",
			methods:     []string{"GET", "POST"},
			shouldMatch: false,
		},
		{
			title:       "Route without methods",
			route:       new(Route),
			request:     newRequest("PUT", "http://localhost"),
			vars:        map[string]string{},
			host:        "",
			path:        "",
			methods:     []string{},
			shouldMatch: true,
		},
	}

	for _, test := range tests {
		t.Run(test.title, func(t *testing.T) {
			testRoute(t, test)
			testTemplate(t, test)
			testMethods(t, test)
		})
	}
}

func TestQueries(t *testing.T) {
	tests := []routeTest{
		{
			title:           "Queries route, match",
			route:           new(Route).Queries("foo", "bar", "baz", "ding"),
			request:         newRequest("GET", "http://localhost?foo=bar&baz=ding"),
			vars:            map[string]string{},
			host:            "",
			path:            "",
			query:           "foo=bar&baz=ding",
			queriesTemplate: "foo=bar,baz=ding",
			queriesRegexp:   "^foo=bar$,^baz=ding$",
			shouldMatch:     true,
		},
		{
			title:           "Queries route, match with a query string",
			route:           new(Route).Host("www.example.com").Path("/api").Queries("foo", "bar", "baz", "ding"),
			request:         newRequest("GET", "http://www.example.com/api?foo=bar&baz=ding"),
			vars:            map[string]string{},
			host:            "",
			path:            "",
			query:           "foo=bar&baz=ding",
			pathTemplate:    `/api`,
			hostTemplate:    `www.example.com`,
			queriesTemplate: "foo=bar,baz=ding",
			queriesRegexp:   "^foo=bar$,^baz=ding$",
			shouldMatch:     true,
		},
		{
			title:           "Queries route, match with a query string out of order",
			route:           new(Route).Host("www.example.com").Path("/api").Queries("foo", "bar", "baz", "ding"),
			request:         newRequest("GET", "http://www.example.com/api?baz=ding&foo=bar"),
			vars:            map[string]string{},
			host:            "",
			path:            "",
			query:           "foo=bar&baz=ding",
			pathTemplate:    `/api`,
			hostTemplate:    `www.example.com`,
			queriesTemplate: "foo=bar,baz=ding",
			queriesRegexp:   "^foo=bar$,^baz=ding$",
			shouldMatch:     true,
		},
		{
			title:           "Queries route, bad query",
			route:           new(Route).Queries("foo", "bar", "baz", "ding"),
			request:         newRequest("GET", "http://localhost?foo=bar&baz=dong"),
			vars:            map[string]string{},
			host:            "",
			path:            "",
			queriesTemplate: "foo=bar,baz=ding",
			queriesRegexp:   "^foo=bar$,^baz=ding$",
			shouldMatch:     false,
		},
		{
			title:           "Queries route with pattern, match",
			route:           new(Route).Queries("foo", "{v1}"),
			request:         newRequest("GET", "http://localhost?foo=bar"),
			vars:            map[string]string{"v1": "bar"},
			host:            "",
			path:            "",
			query:           "foo=bar",
			queriesTemplate: "foo={v1}",
			queriesRegexp:   "^foo=(?P<v0>.*)$",
			shouldMatch:     true,
		},
		{
			title:           "Queries route with multiple patterns, match",
			route:           new(Route).Queries("foo", "{v1}", "baz", "{v2}"),
			request:         newRequest("GET", "http://localhost?foo=bar&baz=ding"),
			vars:            map[string]string{"v1": "bar", "v2": "ding"},
			host:            "",
			path:            "",
			query:           "foo=bar&baz=ding",
			queriesTemplate: "foo={v1},baz={v2}",
			queriesRegexp:   "^foo=(?P<v0>.*)$,^baz=(?P<v0>.*)$",
			shouldMatch:     true,
		},
		{
			title:           "Queries route with regexp pattern, match",
			route:           new(Route).Queries("foo", "{v1:[0-9]+}"),
			request:         newRequest("GET", "http://localhost?foo=10"),
			vars:            map[string]string{"v1": "10"},
			host:            "",
			path:            "",
			query:           "foo=10",
			queriesTemplate: "foo={v1:[0-9]+}",
			queriesRegexp:   "^foo=(?P<v0>[0-9]+)$",
			shouldMatch:     true,
		},
		{
			title:           "Queries route with regexp pattern, regexp does not match",
			route:           new(Route).Queries("foo", "{v1:[0-9]+}"),
			request:         newRequest("GET", "http://localhost?foo=a"),
			vars:            map[string]string{},
			host:            "",
			path:            "",
			queriesTemplate: "foo={v1:[0-9]+}",
			queriesRegexp:   "^foo=(?P<v0>[0-9]+)$",
			shouldMatch:     false,
		},
		{
			title:           "Queries route with regexp pattern with quantifier, match",
			route:           new(Route).Queries("foo", "{v1:[0-9]{1}}"),
			request:         newRequest("GET", "http://localhost?foo=1"),
			vars:            map[string]string{"v1": "1"},
			host:            "",
			path:            "",
			query:           "foo=1",
			queriesTemplate: "foo={v1:[0-9]{1}}",
			queriesRegexp:   "^foo=(?P<v0>[0-9]{1})$",
			shouldMatch:     true,
		},
		{
			title:           "Queries route with regexp pattern with quantifier, additional variable in query string, match",
			route:           new(Route).Queries("foo", "{v1:[0-9]{1}}"),
			request:         newRequest("GET", "http://localhost?bar=2&foo=1"),
			vars:            map[string]string{"v1": "1"},
			host:            "",
			path:            "",
			query:           "foo=1",
			queriesTemplate: "foo={v1:[0-9]{1}}",
			queriesRegexp:   "^foo=(?P<v0>[0-9]{1})$",
			shouldMatch:     true,
		},
		{
			title:           "Queries route with regexp pattern with quantifier, regexp does not match",
			route:           new(Route).Queries("foo", "{v1:[0-9]{1}}"),
			request:         newRequest("GET", "http://localhost?foo=12"),
			vars:            map[string]string{},
			host:            "",
			path:            "",
			queriesTemplate: "foo={v1:[0-9]{1}}",
			queriesRegexp:   "^foo=(?P<v0>[0-9]{1})$",
			shouldMatch:     false,
		},
		{
			title:           "Queries route with regexp pattern with quantifier, additional capturing group",
			route:           new(Route).Queries("foo", "{v1:[0-9]{1}(?:a|b)}"),
			request:         newRequest("GET", "http://localhost?foo=1a"),
			vars:            map[string]string{"v1": "1a"},
			host:            "",
			path:            "",
			query:           "foo=1a",
			queriesTemplate: "foo={v1:[0-9]{1}(?:a|b)}",
			queriesRegexp:   "^foo=(?P<v0>[0-9]{1}(?:a|b))$",
			shouldMatch:     true,
		},
		{
			title:           "Queries route with regexp pattern with quantifier, additional variable in query string, regexp does not match",
			route:           new(Route).Queries("foo", "{v1:[0-9]{1}}"),
			request:         newRequest("GET", "http://localhost?foo=12"),
			vars:            map[string]string{},
			host:            "",
			path:            "",
			queriesTemplate: "foo={v1:[0-9]{1}}",
			queriesRegexp:   "^foo=(?P<v0>[0-9]{1})$",
			shouldMatch:     false,
		},
		{
			title:           "Queries route with hyphenated name, match",
			route:           new(Route).Queries("foo", "{v-1}"),
			request:         newRequest("GET", "http://localhost?foo=bar"),
			vars:            map[string]string{"v-1": "bar"},
			host:            "",
			path:            "",
			query:           "foo=bar",
			queriesTemplate: "foo={v-1}",
			queriesRegexp:   "^foo=(?P<v0>.*)$",
			shouldMatch:     true,
		},
		{
			title:           "Queries route with multiple hyphenated names, match",
			route:           new(Route).Queries("foo", "{v-1}", "baz", "{v-2}"),
			request:         newRequest("GET", "http://localhost?foo=bar&baz=ding"),
			vars:            map[string]string{"v-1": "bar", "v-2": "ding"},
			host:            "",
			path:            "",
			query:           "foo=bar&baz=ding",
			queriesTemplate: "foo={v-1},baz={v-2}",
			queriesRegexp:   "^foo=(?P<v0>.*)$,^baz=(?P<v0>.*)$",
			shouldMatch:     true,
		},
		{
			title:           "Queries route with hyphenate name and pattern, match",
			route:           new(Route).Queries("foo", "{v-1:[0-9]+}"),
			request:         newRequest("GET", "http://localhost?foo=10"),
			vars:            map[string]string{"v-1": "10"},
			host:            "",
			path:            "",
			query:           "foo=10",
			queriesTemplate: "foo={v-1:[0-9]+}",
			queriesRegexp:   "^foo=(?P<v0>[0-9]+)$",
			shouldMatch:     true,
		},
		{
			title:           "Queries route with hyphenated name and pattern with quantifier, additional capturing group",
			route:           new(Route).Queries("foo", "{v-1:[0-9]{1}(?:a|b)}"),
			request:         newRequest("GET", "http://localhost?foo=1a"),
			vars:            map[string]string{"v-1": "1a"},
			host:            "",
			path:            "",
			query:           "foo=1a",
			queriesTemplate: "foo={v-1:[0-9]{1}(?:a|b)}",
			queriesRegexp:   "^foo=(?P<v0>[0-9]{1}(?:a|b))$",
			shouldMatch:     true,
		},
		{
			title:           "Queries route with empty value, should match",
			route:           new(Route).Queries("foo", ""),
			request:         newRequest("GET", "http://localhost?foo=bar"),
			vars:            map[string]string{},
			host:            "",
			path:            "",
			query:           "foo=",
			queriesTemplate: "foo=",
			queriesRegexp:   "^foo=.*$",
			shouldMatch:     true,
		},
		{
			title:           "Queries route with empty value and no parameter in request, should not match",
			route:           new(Route).Queries("foo", ""),
			request:         newRequest("GET", "http://localhost"),
			vars:            map[string]string{},
			host:            "",
			path:            "",
			queriesTemplate: "foo=",
			queriesRegexp:   "^foo=.*$",
			shouldMatch:     false,
		},
		{
			title:           "Queries route with empty value and empty parameter in request, should match",
			route:           new(Route).Queries("foo", ""),
			request:         newRequest("GET", "http://localhost?foo="),
			vars:            map[string]string{},
			host:            "",
			path:            "",
			query:           "foo=",
			queriesTemplate: "foo=",
			queriesRegexp:   "^foo=.*$",
			shouldMatch:     true,
		},
		{
			title:           "Queries route with overlapping value, should not match",
			route:           new(Route).Queries("foo", "bar"),
			request:         newRequest("GET", "http://localhost?foo=barfoo"),
			vars:            map[string]string{},
			host:            "",
			path:            "",
			queriesTemplate: "foo=bar",
			queriesRegexp:   "^foo=bar$",
			shouldMatch:     false,
		},
		{
			title:           "Queries route with no parameter in request, should not match",
			route:           new(Route).Queries("foo", "{bar}"),
			request:         newRequest("GET", "http://localhost"),
			vars:            map[string]string{},
			host:            "",
			path:            "",
			queriesTemplate: "foo={bar}",
			queriesRegexp:   "^foo=(?P<v0>.*)$",
			shouldMatch:     false,
		},
		{
			title:           "Queries route with empty parameter in request, should match",
			route:           new(Route).Queries("foo", "{bar}"),
			request:         newRequest("GET", "http://localhost?foo="),
			vars:            map[string]string{"foo": ""},
			host:            "",
			path:            "",
			query:           "foo=",
			queriesTemplate: "foo={bar}",
			queriesRegexp:   "^foo=(?P<v0>.*)$",
			shouldMatch:     true,
		},
		{
			title:           "Queries route, bad submatch",
			route:           new(Route).Queries("foo", "bar", "baz", "ding"),
			request:         newRequest("GET", "http://localhost?fffoo=bar&baz=dingggg"),
			vars:            map[string]string{},
			host:            "",
			path:            "",
			queriesTemplate: "foo=bar,baz=ding",
			queriesRegexp:   "^foo=bar$,^baz=ding$",
			shouldMatch:     false,
		},
		{
			title:           "Queries route with pattern, match, escaped value",
			route:           new(Route).Queries("foo", "{v1}"),
			request:         newRequest("GET", "http://localhost?foo=%25bar%26%20%2F%3D%3F"),
			vars:            map[string]string{"v1": "%bar& /=?"},
			host:            "",
			path:            "",
			query:           "foo=%25bar%26+%2F%3D%3F",
			queriesTemplate: "foo={v1}",
			queriesRegexp:   "^foo=(?P<v0>.*)$",
			shouldMatch:     true,
		},
	}

	for _, test := range tests {
		t.Run(test.title, func(t *testing.T) {
			testTemplate(t, test)
			testQueriesTemplates(t, test)
			testUseEscapedRoute(t, test)
			testQueriesRegexp(t, test)
		})
	}
}

func TestSchemes(t *testing.T) {
	tests := []routeTest{
		// Schemes
		{
			title:       "Schemes route, default scheme, match http, build http",
			route:       new(Route).Host("localhost"),
			request:     newRequest("GET", "http://localhost"),
			scheme:      "http",
			host:        "localhost",
			shouldMatch: true,
		},
		{
			title:       "Schemes route, match https, build https",
			route:       new(Route).Schemes("https", "ftp").Host("localhost"),
			request:     newRequest("GET", "https://localhost"),
			scheme:      "https",
			host:        "localhost",
			shouldMatch: true,
		},
		{
			title:       "Schemes route, match ftp, build https",
			route:       new(Route).Schemes("https", "ftp").Host("localhost"),
			request:     newRequest("GET", "ftp://localhost"),
			scheme:      "https",
			host:        "localhost",
			shouldMatch: true,
		},
		{
			title:       "Schemes route, match ftp, build ftp",
			route:       new(Route).Schemes("ftp", "https").Host("localhost"),
			request:     newRequest("GET", "ftp://localhost"),
			scheme:      "ftp",
			host:        "localhost",
			shouldMatch: true,
		},
		{
			title:       "Schemes route, bad scheme",
			route:       new(Route).Schemes("https", "ftp").Host("localhost"),
			request:     newRequest("GET", "http://localhost"),
			scheme:      "https",
			host:        "localhost",
			shouldMatch: false,
		},
	}
	for _, test := range tests {
		t.Run(test.title, func(t *testing.T) {
			testRoute(t, test)
			testTemplate(t, test)
		})
	}
}

func TestMatcherFunc(t *testing.T) {
	m := func(r *http.Request, m *RouteMatch) bool {
		return r.URL.Host == "aaa.bbb.ccc"
	}

	tests := []routeTest{
		{
			title:       "MatchFunc route, match",
			route:       new(Route).MatcherFunc(m),
			request:     newRequest("GET", "http://aaa.bbb.ccc"),
			vars:        map[string]string{},
			host:        "",
			path:        "",
			shouldMatch: true,
		},
		{
			title:       "MatchFunc route, non-match",
			route:       new(Route).MatcherFunc(m),
			request:     newRequest("GET", "http://aaa.222.ccc"),
			vars:        map[string]string{},
			host:        "",
			path:        "",
			shouldMatch: false,
		},
	}

	for _, test := range tests {
		t.Run(test.title, func(t *testing.T) {
			testRoute(t, test)
			testTemplate(t, test)
		})
	}
}

func TestBuildVarsFunc(t *testing.T) {
	tests := []routeTest{
		{
			title: "BuildVarsFunc set on route",
			route: new(Route).Path(`/111/{v1:\d}{v2:.*}`).BuildVarsFunc(func(vars map[string]string) map[string]string {
				vars["v1"] = "3"
				vars["v2"] = "a"
				return vars
			}),
			request:      newRequest("GET", "http://localhost/111/2"),
			path:         "/111/3a",
			pathTemplate: `/111/{v1:\d}{v2:.*}`,
			shouldMatch:  true,
		},
		{
			title: "BuildVarsFunc set on route and parent route",
			route: new(Route).PathPrefix(`/{v1:\d}`).BuildVarsFunc(func(vars map[string]string) map[string]string {
				vars["v1"] = "2"
				return vars
			}).Subrouter().Path(`/{v2:\w}`).BuildVarsFunc(func(vars map[string]string) map[string]string {
				vars["v2"] = "b"
				return vars
			}),
			request:      newRequest("GET", "http://localhost/1/a"),
			path:         "/2/b",
			pathTemplate: `/{v1:\d}/{v2:\w}`,
			shouldMatch:  true,
		},
	}

	for _, test := range tests {
		t.Run(test.title, func(t *testing.T) {
			testRoute(t, test)
			testTemplate(t, test)
		})
	}
}

func TestSubRouter(t *testing.T) {
	subrouter1 := new(Route).Host("{v1:[a-z]+}.google.com").Subrouter()
	subrouter2 := new(Route).PathPrefix("/foo/{v1}").Subrouter()
	subrouter3 := new(Route).PathPrefix("/foo").Subrouter()
	subrouter4 := new(Route).PathPrefix("/foo/bar").Subrouter()
	subrouter5 := new(Route).PathPrefix("/{category}").Subrouter()
	tests := []routeTest{
		{
			route:        subrouter1.Path("/{v2:[a-z]+}"),
			request:      newRequest("GET", "http://aaa.google.com/bbb"),
			vars:         map[string]string{"v1": "aaa", "v2": "bbb"},
			host:         "aaa.google.com",
			path:         "/bbb",
			pathTemplate: `/{v2:[a-z]+}`,
			hostTemplate: `{v1:[a-z]+}.google.com`,
			shouldMatch:  true,
		},
		{
			route:        subrouter1.Path("/{v2:[a-z]+}"),
			request:      newRequest("GET", "http://111.google.com/111"),
			vars:         map[string]string{"v1": "aaa", "v2": "bbb"},
			host:         "aaa.google.com",
			path:         "/bbb",
			pathTemplate: `/{v2:[a-z]+}`,
			hostTemplate: `{v1:[a-z]+}.google.com`,
			shouldMatch:  false,
		},
		{
			route:        subrouter2.Path("/baz/{v2}"),
			request:      newRequest("GET", "http://localhost/foo/bar/baz/ding"),
			vars:         map[string]string{"v1": "bar", "v2": "ding"},
			host:         "",
			path:         "/foo/bar/baz/ding",
			pathTemplate: `/foo/{v1}/baz/{v2}`,
			shouldMatch:  true,
		},
		{
			route:        subrouter2.Path("/baz/{v2}"),
			request:      newRequest("GET", "http://localhost/foo/bar"),
			vars:         map[string]string{"v1": "bar", "v2": "ding"},
			host:         "",
			path:         "/foo/bar/baz/ding",
			pathTemplate: `/foo/{v1}/baz/{v2}`,
			shouldMatch:  false,
		},
		{
			route:        subrouter3.Path("/"),
			request:      newRequest("GET", "http://localhost/foo/"),
			vars:         map[string]string{},
			host:         "",
			path:         "/foo/",
			pathTemplate: `/foo/`,
			shouldMatch:  true,
		},
		{
			route:        subrouter3.Path(""),
			request:      newRequest("GET", "http://localhost/foo"),
			vars:         map[string]string{},
			host:         "",
			path:         "/foo",
			pathTemplate: `/foo`,
			shouldMatch:  true,
		},

		{
			route:        subrouter4.Path("/"),
			request:      newRequest("GET", "http://localhost/foo/bar/"),
			vars:         map[string]string{},
			host:         "",
			path:         "/foo/bar/",
			pathTemplate: `/foo/bar/`,
			shouldMatch:  true,
		},
		{
			route:        subrouter4.Path(""),
			request:      newRequest("GET", "http://localhost/foo/bar"),
			vars:         map[string]string{},
			host:         "",
			path:         "/foo/bar",
			pathTemplate: `/foo/bar`,
			shouldMatch:  true,
		},
		{
			route:        subrouter5.Path("/"),
			request:      newRequest("GET", "http://localhost/baz/"),
			vars:         map[string]string{"category": "baz"},
			host:         "",
			path:         "/baz/",
			pathTemplate: `/{category}/`,
			shouldMatch:  true,
		},
		{
			route:        subrouter5.Path(""),
			request:      newRequest("GET", "http://localhost/baz"),
			vars:         map[string]string{"category": "baz"},
			host:         "",
			path:         "/baz",
			pathTemplate: `/{category}`,
			shouldMatch:  true,
		},
		{
			title:        "Mismatch method specified on parent route",
			route:        new(Route).Methods("POST").PathPrefix("/foo").Subrouter().Path("/"),
			request:      newRequest("GET", "http://localhost/foo/"),
			vars:         map[string]string{},
			host:         "",
			path:         "/foo/",
			pathTemplate: `/foo/`,
			shouldMatch:  false,
		},
		{
			title:        "Match method specified on parent route",
			route:        new(Route).Methods("POST").PathPrefix("/foo").Subrouter().Path("/"),
			request:      newRequest("POST", "http://localhost/foo/"),
			vars:         map[string]string{},
			host:         "",
			path:         "/foo/",
			pathTemplate: `/foo/`,
			shouldMatch:  true,
		},
		{
			title:        "Mismatch scheme specified on parent route",
			route:        new(Route).Schemes("https").Subrouter().PathPrefix("/"),
			request:      newRequest("GET", "http://localhost/"),
			vars:         map[string]string{},
			host:         "",
			path:         "/",
			pathTemplate: `/`,
			shouldMatch:  false,
		},
		{
			title:        "Match scheme specified on parent route",
			route:        new(Route).Schemes("http").Subrouter().PathPrefix("/"),
			request:      newRequest("GET", "http://localhost/"),
			vars:         map[string]string{},
			host:         "",
			path:         "/",
			pathTemplate: `/`,
			shouldMatch:  true,
		},
		{
			title:        "No match header specified on parent route",
			route:        new(Route).Headers("X-Forwarded-Proto", "https").Subrouter().PathPrefix("/"),
			request:      newRequest("GET", "http://localhost/"),
			vars:         map[string]string{},
			host:         "",
			path:         "/",
			pathTemplate: `/`,
			shouldMatch:  false,
		},
		{
			title:        "Header mismatch value specified on parent route",
			route:        new(Route).Headers("X-Forwarded-Proto", "https").Subrouter().PathPrefix("/"),
			request:      newRequestWithHeaders("GET", "http://localhost/", "X-Forwarded-Proto", "http"),
			vars:         map[string]string{},
			host:         "",
			path:         "/",
			pathTemplate: `/`,
			shouldMatch:  false,
		},
		{
			title:        "Header match value specified on parent route",
			route:        new(Route).Headers("X-Forwarded-Proto", "https").Subrouter().PathPrefix("/"),
			request:      newRequestWithHeaders("GET", "http://localhost/", "X-Forwarded-Proto", "https"),
			vars:         map[string]string{},
			host:         "",
			path:         "/",
			pathTemplate: `/`,
			shouldMatch:  true,
		},
		{
			title:        "Query specified on parent route not present",
			route:        new(Route).Headers("key", "foobar").Subrouter().PathPrefix("/"),
			request:      newRequest("GET", "http://localhost/"),
			vars:         map[string]string{},
			host:         "",
			path:         "/",
			pathTemplate: `/`,
			shouldMatch:  false,
		},
		{
			title:        "Query mismatch value specified on parent route",
			route:        new(Route).Queries("key", "foobar").Subrouter().PathPrefix("/"),
			request:      newRequest("GET", "http://localhost/?key=notfoobar"),
			vars:         map[string]string{},
			host:         "",
			path:         "/",
			pathTemplate: `/`,
			shouldMatch:  false,
		},
		{
			title:        "Query match value specified on subroute",
			route:        new(Route).Queries("key", "foobar").Subrouter().PathPrefix("/"),
			request:      newRequest("GET", "http://localhost/?key=foobar"),
			vars:         map[string]string{},
			host:         "",
			path:         "/",
			pathTemplate: `/`,
			shouldMatch:  true,
		},
		{
			title:        "Build with scheme on parent router",
			route:        new(Route).Schemes("ftp").Host("google.com").Subrouter().Path("/"),
			request:      newRequest("GET", "ftp://google.com/"),
			scheme:       "ftp",
			host:         "google.com",
			path:         "/",
			pathTemplate: `/`,
			hostTemplate: `google.com`,
			shouldMatch:  true,
		},
		{
			title:        "Prefer scheme on child route when building URLs",
			route:        new(Route).Schemes("https", "ftp").Host("google.com").Subrouter().Schemes("ftp").Path("/"),
			request:      newRequest("GET", "ftp://google.com/"),
			scheme:       "ftp",
			host:         "google.com",
			path:         "/",
			pathTemplate: `/`,
			hostTemplate: `google.com`,
			shouldMatch:  true,
		},
	}

	for _, test := range tests {
		t.Run(test.title, func(t *testing.T) {
			testRoute(t, test)
			testTemplate(t, test)
			testUseEscapedRoute(t, test)
		})
	}
}

func TestNamedRoutes(t *testing.T) {
	r1 := NewRouter()
	r1.NewRoute().Name("a")
	r1.NewRoute().Name("b")
	r1.NewRoute().Name("c")

	r2 := r1.NewRoute().Subrouter()
	r2.NewRoute().Name("d")
	r2.NewRoute().Name("e")
	r2.NewRoute().Name("f")

	r3 := r2.NewRoute().Subrouter()
	r3.NewRoute().Name("g")
	r3.NewRoute().Name("h")
	r3.NewRoute().Name("i")
	r3.Name("j")

	if r1.namedRoutes == nil || len(r1.namedRoutes) != 10 {
		t.Errorf("Expected 10 named routes, got %v", r1.namedRoutes)
	} else if r1.Get("j") == nil {
		t.Errorf("Subroute name not registered")
	}
}

func TestNameMultipleCalls(t *testing.T) {
	r1 := NewRouter()
	rt := r1.NewRoute().Name("foo").Name("bar")
	err := rt.GetError()
	if err == nil {
		t.Errorf("Expected an error")
	}
}

func TestStrictSlash(t *testing.T) {
	r := NewRouter()
	r.StrictSlash(true)

	tests := []routeTest{
		{
			title:          "Redirect path without slash",
			route:          r.NewRoute().Path("/111/"),
			request:        newRequest("GET", "http://localhost/111"),
			vars:           map[string]string{},
			host:           "",
			path:           "/111/",
			shouldMatch:    true,
			shouldRedirect: true,
		},
		{
			title:          "Do not redirect path with slash",
			route:          r.NewRoute().Path("/111/"),
			request:        newRequest("GET", "http://localhost/111/"),
			vars:           map[string]string{},
			host:           "",
			path:           "/111/",
			shouldMatch:    true,
			shouldRedirect: false,
		},
		{
			title:          "Redirect path with slash",
			route:          r.NewRoute().Path("/111"),
			request:        newRequest("GET", "http://localhost/111/"),
			vars:           map[string]string{},
			host:           "",
			path:           "/111",
			shouldMatch:    true,
			shouldRedirect: true,
		},
		{
			title:          "Do not redirect path without slash",
			route:          r.NewRoute().Path("/111"),
			request:        newRequest("GET", "http://localhost/111"),
			vars:           map[string]string{},
			host:           "",
			path:           "/111",
			shouldMatch:    true,
			shouldRedirect: false,
		},
		{
			title:          "Propagate StrictSlash to subrouters",
			route:          r.NewRoute().PathPrefix("/static/").Subrouter().Path("/images/"),
			request:        newRequest("GET", "http://localhost/static/images"),
			vars:           map[string]string{},
			host:           "",
			path:           "/static/images/",
			shouldMatch:    true,
			shouldRedirect: true,
		},
		{
			title:          "Ignore StrictSlash for path prefix",
			route:          r.NewRoute().PathPrefix("/static/"),
			request:        newRequest("GET", "http://localhost/static/logo.png"),
			vars:           map[string]string{},
			host:           "",
			path:           "/static/",
			shouldMatch:    true,
			shouldRedirect: false,
		},
	}

	for _, test := range tests {
		t.Run(test.title, func(t *testing.T) {
			testRoute(t, test)
			testTemplate(t, test)
			testUseEscapedRoute(t, test)
		})
	}
}

func TestUseEncodedPath(t *testing.T) {
	r := NewRouter()
	r.UseEncodedPath()

	tests := []routeTest{
		{
			title:        "Router with useEncodedPath, URL with encoded slash does match",
			route:        r.NewRoute().Path("/v1/{v1}/v2"),
			request:      newRequest("GET", "http://localhost/v1/1%2F2/v2"),
			vars:         map[string]string{"v1": "1%2F2"},
			host:         "",
			path:         "/v1/1%2F2/v2",
			pathTemplate: `/v1/{v1}/v2`,
			shouldMatch:  true,
		},
		{
			title:        "Router with useEncodedPath, URL with encoded slash doesn't match",
			route:        r.NewRoute().Path("/v1/1/2/v2"),
			request:      newRequest("GET", "http://localhost/v1/1%2F2/v2"),
			vars:         map[string]string{"v1": "1%2F2"},
			host:         "",
			path:         "/v1/1%2F2/v2",
			pathTemplate: `/v1/1/2/v2`,
			shouldMatch:  false,
		},
	}

	for _, test := range tests {
		t.Run(test.title, func(t *testing.T) {
			testRoute(t, test)
			testTemplate(t, test)
		})
	}
}

func TestWalkSingleDepth(t *testing.T) {
	r0 := NewRouter()
	r1 := NewRouter()
	r2 := NewRouter()

	r0.Path("/g")
	r0.Path("/o")
	r0.Path("/d").Handler(r1)
	r0.Path("/r").Handler(r2)
	r0.Path("/a")

	r1.Path("/z")
	r1.Path("/i")
	r1.Path("/l")
	r1.Path("/l")

	r2.Path("/i")
	r2.Path("/l")
	r2.Path("/l")

	paths := []string{"g", "o", "r", "i", "l", "l", "a"}
	depths := []int{0, 0, 0, 1, 1, 1, 0}
	i := 0
	err := r0.Walk(func(route *Route, router *Router, ancestors []*Route) error {
		matcher := route.matchers[0].(*routeRegexp)
		if matcher.template == "/d" {
			return SkipRouter
		}
		if len(ancestors) != depths[i] {
			t.Errorf(`Expected depth of %d at i = %d; got "%d"`, depths[i], i, len(ancestors))
		}
		if matcher.template != "/"+paths[i] {
			t.Errorf(`Expected "/%s" at i = %d; got "%s"`, paths[i], i, matcher.template)
		}
		i++
		return nil
	})
	if err != nil {
		panic(err)
	}
	if i != len(paths) {
		t.Errorf("Expected %d routes, found %d", len(paths), i)
	}
}

func TestWalkNested(t *testing.T) {
	router := NewRouter()

	routeSubrouter := func(r *Route) (*Route, *Router) {
		return r, r.Subrouter()
	}

	gRoute, g := routeSubrouter(router.Path("/g"))
	oRoute, o := routeSubrouter(g.PathPrefix("/o"))
	rRoute, r := routeSubrouter(o.PathPrefix("/r"))
	iRoute, i := routeSubrouter(r.PathPrefix("/i"))
	l1Route, l1 := routeSubrouter(i.PathPrefix("/l"))
	l2Route, l2 := routeSubrouter(l1.PathPrefix("/l"))
	l2.Path("/a")

	testCases := []struct {
		path      string
		ancestors []*Route
	}{
		{"/g", []*Route{}},
		{"/g/o", []*Route{gRoute}},
		{"/g/o/r", []*Route{gRoute, oRoute}},
		{"/g/o/r/i", []*Route{gRoute, oRoute, rRoute}},
		{"/g/o/r/i/l", []*Route{gRoute, oRoute, rRoute, iRoute}},
		{"/g/o/r/i/l/l", []*Route{gRoute, oRoute, rRoute, iRoute, l1Route}},
		{"/g/o/r/i/l/l/a", []*Route{gRoute, oRoute, rRoute, iRoute, l1Route, l2Route}},
	}

	idx := 0
	err := router.Walk(func(route *Route, router *Router, ancestors []*Route) error {
		path := testCases[idx].path
		tpl := route.regexp.path.template
		if tpl != path {
			t.Errorf(`Expected %s got %s`, path, tpl)
		}
		currWantAncestors := testCases[idx].ancestors
		if !reflect.DeepEqual(currWantAncestors, ancestors) {
			t.Errorf(`Expected %+v got %+v`, currWantAncestors, ancestors)
		}
		idx++
		return nil
	})
	if err != nil {
		panic(err)
	}
	if idx != len(testCases) {
		t.Errorf("Expected %d routes, found %d", len(testCases), idx)
	}
}

func TestWalkSubrouters(t *testing.T) {
	router := NewRouter()

	g := router.Path("/g").Subrouter()
	o := g.PathPrefix("/o").Subrouter()
	o.Methods("GET")
	o.Methods("PUT")

	// all 4 routes should be matched
	paths := []string{"/g", "/g/o", "/g/o", "/g/o"}
	idx := 0
	err := router.Walk(func(route *Route, router *Router, ancestors []*Route) error {
		path := paths[idx]
		tpl, _ := route.GetPathTemplate()
		if tpl != path {
			t.Errorf(`Expected %s got %s`, path, tpl)
		}
		idx++
		return nil
	})
	if err != nil {
		panic(err)
	}
	if idx != len(paths) {
		t.Errorf("Expected %d routes, found %d", len(paths), idx)
	}
}

func TestWalkErrorRoute(t *testing.T) {
	router := NewRouter()
	router.Path("/g")
	expectedError := errors.New("error")
	err := router.Walk(func(route *Route, router *Router, ancestors []*Route) error {
		return expectedError
	})
	if err != expectedError {
		t.Errorf("Expected %v routes, found %v", expectedError, err)
	}
}

func TestWalkErrorMatcher(t *testing.T) {
	router := NewRouter()
	expectedError := router.Path("/g").Subrouter().Path("").GetError()
	err := router.Walk(func(route *Route, router *Router, ancestors []*Route) error {
		return route.GetError()
	})
	if err != expectedError {
		t.Errorf("Expected %v routes, found %v", expectedError, err)
	}
}

func TestWalkErrorHandler(t *testing.T) {
	handler := NewRouter()
	expectedError := handler.Path("/path").Subrouter().Path("").GetError()
	router := NewRouter()
	router.Path("/g").Handler(handler)
	err := router.Walk(func(route *Route, router *Router, ancestors []*Route) error {
		return route.GetError()
	})
	if err != expectedError {
		t.Errorf("Expected %v routes, found %v", expectedError, err)
	}
}

func TestSubrouterErrorHandling(t *testing.T) {
	superRouterCalled := false
	subRouterCalled := false

	router := NewRouter()
	router.NotFoundHandler = http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		superRouterCalled = true
	})
	subRouter := router.PathPrefix("/bign8").Subrouter()
	subRouter.NotFoundHandler = http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		subRouterCalled = true
	})

	req, _ := http.NewRequest("GET", "http://localhost/bign8/was/here", nil)
	router.ServeHTTP(NewRecorder(), req)

	if superRouterCalled {
		t.Error("Super router 404 handler called when sub-router 404 handler is available.")
	}
	if !subRouterCalled {
		t.Error("Sub-router 404 handler was not called.")
	}
}

// See: https://github.com/gorilla/mux/issues/200
func TestPanicOnCapturingGroups(t *testing.T) {
	defer func() {
		if recover() == nil {
			t.Errorf("(Test that capturing groups now fail fast) Expected panic, however test completed successfully.\n")
		}
	}()
	NewRouter().NewRoute().Path("/{type:(promo|special)}/{promoId}.json")
}

// ----------------------------------------------------------------------------
// Helpers
// ----------------------------------------------------------------------------

func getRouteTemplate(route *Route) string {
	host, err := route.GetHostTemplate()
	if err != nil {
		host = "none"
	}
	path, err := route.GetPathTemplate()
	if err != nil {
		path = "none"
	}
	return fmt.Sprintf("Host: %v, Path: %v", host, path)
}

func testRoute(t *testing.T, test routeTest) {
	request := test.request
	route := test.route
	vars := test.vars
	shouldMatch := test.shouldMatch
	query := test.query
	shouldRedirect := test.shouldRedirect
	uri := url.URL{
		Scheme: test.scheme,
		Host:   test.host,
		Path:   test.path,
	}
	if uri.Scheme == "" {
		uri.Scheme = "http"
	}

	var match RouteMatch
	ok := route.Match(request, &match)
	if ok != shouldMatch {
		msg := "Should match"
		if !shouldMatch {
			msg = "Should not match"
		}
		t.Errorf("(%v) %v:\nRoute: %#v\nRequest: %#v\nVars: %v\n", test.title, msg, route, request, vars)
		return
	}
	if shouldMatch {
		if vars != nil && !stringMapEqual(vars, match.Vars) {
			t.Errorf("(%v) Vars not equal: expected %v, got %v", test.title, vars, match.Vars)
			return
		}
		if test.scheme != "" {
			u, err := route.URL(mapToPairs(match.Vars)...)
			if err != nil {
				t.Fatalf("(%v) URL error: %v -- %v", test.title, err, getRouteTemplate(route))
			}
			if uri.Scheme != u.Scheme {
				t.Errorf("(%v) URLScheme not equal: expected %v, got %v", test.title, uri.Scheme, u.Scheme)
				return
			}
		}
		if test.host != "" {
			u, err := test.route.URLHost(mapToPairs(match.Vars)...)
			if err != nil {
				t.Fatalf("(%v) URLHost error: %v -- %v", test.title, err, getRouteTemplate(route))
			}
			if uri.Scheme != u.Scheme {
				t.Errorf("(%v) URLHost scheme not equal: expected %v, got %v -- %v", test.title, uri.Scheme, u.Scheme, getRouteTemplate(route))
				return
			}
			if uri.Host != u.Host {
				t.Errorf("(%v) URLHost host not equal: expected %v, got %v -- %v", test.title, uri.Host, u.Host, getRouteTemplate(route))
				return
			}
		}
		if test.path != "" {
			u, err := route.URLPath(mapToPairs(match.Vars)...)
			if err != nil {
				t.Fatalf("(%v) URLPath error: %v -- %v", test.title, err, getRouteTemplate(route))
			}
			if uri.Path != u.Path {
				t.Errorf("(%v) URLPath not equal: expected %v, got %v -- %v", test.title, uri.Path, u.Path, getRouteTemplate(route))
				return
			}
		}
		if test.host != "" && test.path != "" {
			u, err := route.URL(mapToPairs(match.Vars)...)
			if err != nil {
				t.Fatalf("(%v) URL error: %v -- %v", test.title, err, getRouteTemplate(route))
			}
			if expected, got := uri.String(), u.String(); expected != got {
				t.Errorf("(%v) URL not equal: expected %v, got %v -- %v", test.title, expected, got, getRouteTemplate(route))
				return
			}
		}
		if query != "" {
			u, err := route.URL(mapToPairs(match.Vars)...)
			if err != nil {
				t.Errorf("(%v) erred while creating url: %v", test.title, err)
				return
			}
			if query != u.RawQuery {
				t.Errorf("(%v) URL query not equal: expected %v, got %v", test.title, query, u.RawQuery)
				return
			}
		}
		if shouldRedirect && match.Handler == nil {
			t.Errorf("(%v) Did not redirect", test.title)
			return
		}
		if !shouldRedirect && match.Handler != nil {
			t.Errorf("(%v) Unexpected redirect", test.title)
			return
		}
	}
}

func testUseEscapedRoute(t *testing.T, test routeTest) {
	test.route.useEncodedPath = true
	testRoute(t, test)
}

func testTemplate(t *testing.T, test routeTest) {
	route := test.route
	pathTemplate := test.pathTemplate
	if len(pathTemplate) == 0 {
		pathTemplate = test.path
	}
	hostTemplate := test.hostTemplate
	if len(hostTemplate) == 0 {
		hostTemplate = test.host
	}

	routePathTemplate, pathErr := route.GetPathTemplate()
	if pathErr == nil && routePathTemplate != pathTemplate {
		t.Errorf("(%v) GetPathTemplate not equal: expected %v, got %v", test.title, pathTemplate, routePathTemplate)
	}

	routeHostTemplate, hostErr := route.GetHostTemplate()
	if hostErr == nil && routeHostTemplate != hostTemplate {
		t.Errorf("(%v) GetHostTemplate not equal: expected %v, got %v", test.title, hostTemplate, routeHostTemplate)
	}
}

func testMethods(t *testing.T, test routeTest) {
	route := test.route
	methods, _ := route.GetMethods()
	if strings.Join(methods, ",") != strings.Join(test.methods, ",") {
		t.Errorf("(%v) GetMethods not equal: expected %v, got %v", test.title, test.methods, methods)
	}
}

func testRegexp(t *testing.T, test routeTest) {
	route := test.route
	routePathRegexp, regexpErr := route.GetPathRegexp()
	if test.pathRegexp != "" && regexpErr == nil && routePathRegexp != test.pathRegexp {
		t.Errorf("(%v) GetPathRegexp not equal: expected %v, got %v", test.title, test.pathRegexp, routePathRegexp)
	}
}

func testQueriesRegexp(t *testing.T, test routeTest) {
	route := test.route
	queries, queriesErr := route.GetQueriesRegexp()
	gotQueries := strings.Join(queries, ",")
	if test.queriesRegexp != "" && queriesErr == nil && gotQueries != test.queriesRegexp {
		t.Errorf("(%v) GetQueriesRegexp not equal: expected %v, got %v", test.title, test.queriesRegexp, gotQueries)
	}
}

func testQueriesTemplates(t *testing.T, test routeTest) {
	route := test.route
	queries, queriesErr := route.GetQueriesTemplates()
	gotQueries := strings.Join(queries, ",")
	if test.queriesTemplate != "" && queriesErr == nil && gotQueries != test.queriesTemplate {
		t.Errorf("(%v) GetQueriesTemplates not equal: expected %v, got %v", test.title, test.queriesTemplate, gotQueries)
	}
}

type TestA301ResponseWriter struct {
	hh     http.Header
	status int
}

func (ho *TestA301ResponseWriter) Header() http.Header {
	return ho.hh
}

func (ho *TestA301ResponseWriter) Write(b []byte) (int, error) {
	return 0, nil
}

func (ho *TestA301ResponseWriter) WriteHeader(code int) {
	ho.status = code
}

func Test301Redirect(t *testing.T) {
	m := make(http.Header)

	func1 := func(w http.ResponseWriter, r *http.Request) {}
	func2 := func(w http.ResponseWriter, r *http.Request) {}

	r := NewRouter()
	r.HandleFunc("/api/", func2).Name("func2")
	r.HandleFunc("/", func1).Name("func1")

	req, _ := http.NewRequest("GET", "http://localhost//api/?abc=def", nil)

	res := TestA301ResponseWriter{
		hh:     m,
		status: 0,
	}
	r.ServeHTTP(&res, req)

	if "http://localhost/api/?abc=def" != res.hh["Location"][0] {
		t.Errorf("Should have complete URL with query string")
	}
}

func TestSkipClean(t *testing.T) {
	func1 := func(w http.ResponseWriter, r *http.Request) {}
	func2 := func(w http.ResponseWriter, r *http.Request) {}

	r := NewRouter()
	r.SkipClean(true)
	r.HandleFunc("/api/", func2).Name("func2")
	r.HandleFunc("/", func1).Name("func1")

	req, _ := http.NewRequest("GET", "http://localhost//api/?abc=def", nil)
	res := NewRecorder()
	r.ServeHTTP(res, req)

	if len(res.HeaderMap["Location"]) != 0 {
		t.Errorf("Shouldn't redirect since skip clean is disabled")
	}
}

// https://plus.google.com/101022900381697718949/posts/eWy6DjFJ6uW
func TestSubrouterHeader(t *testing.T) {
	expected := "func1 response"
	func1 := func(w http.ResponseWriter, r *http.Request) {
		fmt.Fprint(w, expected)
	}
	func2 := func(http.ResponseWriter, *http.Request) {}

	r := NewRouter()
	s := r.Headers("SomeSpecialHeader", "").Subrouter()
	s.HandleFunc("/", func1).Name("func1")
	r.HandleFunc("/", func2).Name("func2")

	req, _ := http.NewRequest("GET", "http://localhost/", nil)
	req.Header.Add("SomeSpecialHeader", "foo")
	match := new(RouteMatch)
	matched := r.Match(req, match)
	if !matched {
		t.Errorf("Should match request")
	}
	if match.Route.GetName() != "func1" {
		t.Errorf("Expecting func1 handler, got %s", match.Route.GetName())
	}
	resp := NewRecorder()
	match.Handler.ServeHTTP(resp, req)
	if resp.Body.String() != expected {
		t.Errorf("Expecting %q", expected)
	}
}

func TestNoMatchMethodErrorHandler(t *testing.T) {
	func1 := func(w http.ResponseWriter, r *http.Request) {}

	r := NewRouter()
	r.HandleFunc("/", func1).Methods("GET", "POST")

	req, _ := http.NewRequest("PUT", "http://localhost/", nil)
	match := new(RouteMatch)
	matched := r.Match(req, match)

	if matched {
		t.Error("Should not have matched route for methods")
	}

	if match.MatchErr != ErrMethodMismatch {
		t.Error("Should get ErrMethodMismatch error")
	}

	resp := NewRecorder()
	r.ServeHTTP(resp, req)
	if resp.Code != http.StatusMethodNotAllowed {
		t.Errorf("Expecting code %v", 405)
	}

	// Add matching route
	r.HandleFunc("/", func1).Methods("PUT")

	match = new(RouteMatch)
	matched = r.Match(req, match)

	if !matched {
		t.Error("Should have matched route for methods")
	}

	if match.MatchErr != nil {
		t.Error("Should not have any matching error. Found:", match.MatchErr)
	}
}

func TestMultipleDefinitionOfSamePathWithDifferentMethods(t *testing.T) {
	emptyHandler := func(w http.ResponseWriter, r *http.Request) {}

	r := NewRouter()
	r.HandleFunc("/api", emptyHandler).Methods("POST")
	r.HandleFunc("/api", emptyHandler).Queries("time", "{time:[0-9]+}").Methods("GET")

	t.Run("Post Method should be matched properly", func(t *testing.T) {
		req, _ := http.NewRequest("POST", "http://localhost/api", nil)
		match := new(RouteMatch)
		matched := r.Match(req, match)
		if !matched {
			t.Error("Should have matched route for methods")
		}
		if match.MatchErr != nil {
			t.Error("Should not have any matching error. Found:", match.MatchErr)
		}
	})

	t.Run("Get Method with invalid query value should not match", func(t *testing.T) {
		req, _ := http.NewRequest("GET", "http://localhost/api?time=-4", nil)
		match := new(RouteMatch)
		matched := r.Match(req, match)
		if matched {
			t.Error("Should not have matched route for methods")
		}
		if match.MatchErr != ErrNotFound {
			t.Error("Should have ErrNotFound error. Found:", match.MatchErr)
		}
	})

	t.Run("A mismach method of a valid path should return ErrMethodMismatch", func(t *testing.T) {
		r := NewRouter()
		r.HandleFunc("/api2", emptyHandler).Methods("POST")
		req, _ := http.NewRequest("GET", "http://localhost/api2", nil)
		match := new(RouteMatch)
		matched := r.Match(req, match)
		if matched {
			t.Error("Should not have matched route for methods")
		}
		if match.MatchErr != ErrMethodMismatch {
			t.Error("Should have ErrMethodMismatch error. Found:", match.MatchErr)
		}
	})

}

func TestErrMatchNotFound(t *testing.T) {
	emptyHandler := func(w http.ResponseWriter, r *http.Request) {}

	r := NewRouter()
	r.HandleFunc("/", emptyHandler)
	s := r.PathPrefix("/sub/").Subrouter()
	s.HandleFunc("/", emptyHandler)

	// Regular 404 not found
	req, _ := http.NewRequest("GET", "/sub/whatever", nil)
	match := new(RouteMatch)
	matched := r.Match(req, match)

	if matched {
		t.Errorf("Subrouter should not have matched that, got %v", match.Route)
	}
	// Even without a custom handler, MatchErr is set to ErrNotFound
	if match.MatchErr != ErrNotFound {
		t.Errorf("Expected ErrNotFound MatchErr, but was %v", match.MatchErr)
	}

	// Now lets add a 404 handler to subrouter
	s.NotFoundHandler = http.NotFoundHandler()
	req, _ = http.NewRequest("GET", "/sub/whatever", nil)

	// Test the subrouter first
	match = new(RouteMatch)
	matched = s.Match(req, match)
	// Now we should get a match
	if !matched {
		t.Errorf("Subrouter should have matched %s", req.RequestURI)
	}
	// But MatchErr should be set to ErrNotFound anyway
	if match.MatchErr != ErrNotFound {
		t.Errorf("Expected ErrNotFound MatchErr, but was %v", match.MatchErr)
	}

	// Now test the parent (MatchErr should propagate)
	match = new(RouteMatch)
	matched = r.Match(req, match)

	// Now we should get a match
	if !matched {
		t.Errorf("Router should have matched %s via subrouter", req.RequestURI)
	}
	// But MatchErr should be set to ErrNotFound anyway
	if match.MatchErr != ErrNotFound {
		t.Errorf("Expected ErrNotFound MatchErr, but was %v", match.MatchErr)
	}
}

// methodsSubrouterTest models the data necessary for testing handler
// matching for subrouters created after HTTP methods matcher registration.
type methodsSubrouterTest struct {
	title    string
	wantCode int
	router   *Router
	// method is the input into the request and expected response
	method string
	// input request path
	path string
	// redirectTo is the expected location path for strict-slash matches
	redirectTo string
}

// methodHandler writes the method string in response.
func methodHandler(method string) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		_, err := w.Write([]byte(method))
		if err != nil {
			log.Printf("Failed writing HTTP response: %v", err)
		}
	}
}

// TestMethodsSubrouterCatchall matches handlers for subrouters where a
// catchall handler is set for a mis-matching method.
func TestMethodsSubrouterCatchall(t *testing.T) {
	t.Parallel()

	router := NewRouter()
	router.Methods("PATCH").Subrouter().PathPrefix("/").HandlerFunc(methodHandler("PUT"))
	router.Methods("GET").Subrouter().HandleFunc("/foo", methodHandler("GET"))
	router.Methods("POST").Subrouter().HandleFunc("/foo", methodHandler("POST"))
	router.Methods("DELETE").Subrouter().HandleFunc("/foo", methodHandler("DELETE"))

	tests := []methodsSubrouterTest{
		{
			title:    "match GET handler",
			router:   router,
			path:     "http://localhost/foo",
			method:   "GET",
			wantCode: http.StatusOK,
		},
		{
			title:    "match POST handler",
			router:   router,
			method:   "POST",
			path:     "http://localhost/foo",
			wantCode: http.StatusOK,
		},
		{
			title:    "match DELETE handler",
			router:   router,
			method:   "DELETE",
			path:     "http://localhost/foo",
			wantCode: http.StatusOK,
		},
		{
			title:    "disallow PUT method",
			router:   router,
			method:   "PUT",
			path:     "http://localhost/foo",
			wantCode: http.StatusMethodNotAllowed,
		},
	}

	for _, test := range tests {
		t.Run(test.title, func(t *testing.T) {
			testMethodsSubrouter(t, test)
		})
	}
}

// TestMethodsSubrouterStrictSlash matches handlers on subrouters with
// strict-slash matchers.
func TestMethodsSubrouterStrictSlash(t *testing.T) {
	t.Parallel()

	router := NewRouter()
	sub := router.PathPrefix("/").Subrouter()
	sub.StrictSlash(true).Path("/foo").Methods("GET").Subrouter().HandleFunc("", methodHandler("GET"))
	sub.StrictSlash(true).Path("/foo/").Methods("PUT").Subrouter().HandleFunc("/", methodHandler("PUT"))
	sub.StrictSlash(true).Path("/foo/").Methods("POST").Subrouter().HandleFunc("/", methodHandler("POST"))

	tests := []methodsSubrouterTest{
		{
			title:    "match POST handler",
			router:   router,
			method:   "POST",
			path:     "http://localhost/foo/",
			wantCode: http.StatusOK,
		},
		{
			title:    "match GET handler",
			router:   router,
			method:   "GET",
			path:     "http://localhost/foo",
			wantCode: http.StatusOK,
		},
		{
			title:      "match POST handler, redirect strict-slash",
			router:     router,
			method:     "POST",
			path:       "http://localhost/foo",
			redirectTo: "http://localhost/foo/",
			wantCode:   http.StatusMovedPermanently,
		},
		{
			title:      "match GET handler, redirect strict-slash",
			router:     router,
			method:     "GET",
			path:       "http://localhost/foo/",
			redirectTo: "http://localhost/foo",
			wantCode:   http.StatusMovedPermanently,
		},
		{
			title:    "disallow DELETE method",
			router:   router,
			method:   "DELETE",
			path:     "http://localhost/foo",
			wantCode: http.StatusMethodNotAllowed,
		},
	}

	for _, test := range tests {
		t.Run(test.title, func(t *testing.T) {
			testMethodsSubrouter(t, test)
		})
	}
}

// TestMethodsSubrouterPathPrefix matches handlers on subrouters created
// on a router with a path prefix matcher and method matcher.
func TestMethodsSubrouterPathPrefix(t *testing.T) {
	t.Parallel()

	router := NewRouter()
	router.PathPrefix("/1").Methods("POST").Subrouter().HandleFunc("/2", methodHandler("POST"))
	router.PathPrefix("/1").Methods("DELETE").Subrouter().HandleFunc("/2", methodHandler("DELETE"))
	router.PathPrefix("/1").Methods("PUT").Subrouter().HandleFunc("/2", methodHandler("PUT"))
	router.PathPrefix("/1").Methods("POST").Subrouter().HandleFunc("/2", methodHandler("POST2"))

	tests := []methodsSubrouterTest{
		{
			title:    "match first POST handler",
			router:   router,
			method:   "POST",
			path:     "http://localhost/1/2",
			wantCode: http.StatusOK,
		},
		{
			title:    "match DELETE handler",
			router:   router,
			method:   "DELETE",
			path:     "http://localhost/1/2",
			wantCode: http.StatusOK,
		},
		{
			title:    "match PUT handler",
			router:   router,
			method:   "PUT",
			path:     "http://localhost/1/2",
			wantCode: http.StatusOK,
		},
		{
			title:    "disallow PATCH method",
			router:   router,
			method:   "PATCH",
			path:     "http://localhost/1/2",
			wantCode: http.StatusMethodNotAllowed,
		},
	}

	for _, test := range tests {
		t.Run(test.title, func(t *testing.T) {
			testMethodsSubrouter(t, test)
		})
	}
}

// TestMethodsSubrouterSubrouter matches handlers on subrouters produced
// from method matchers registered on a root subrouter.
func TestMethodsSubrouterSubrouter(t *testing.T) {
	t.Parallel()

	router := NewRouter()
	sub := router.PathPrefix("/1").Subrouter()
	sub.Methods("POST").Subrouter().HandleFunc("/2", methodHandler("POST"))
	sub.Methods("GET").Subrouter().HandleFunc("/2", methodHandler("GET"))
	sub.Methods("PATCH").Subrouter().HandleFunc("/2", methodHandler("PATCH"))
	sub.HandleFunc("/2", methodHandler("PUT")).Subrouter().Methods("PUT")
	sub.HandleFunc("/2", methodHandler("POST2")).Subrouter().Methods("POST")

	tests := []methodsSubrouterTest{
		{
			title:    "match first POST handler",
			router:   router,
			method:   "POST",
			path:     "http://localhost/1/2",
			wantCode: http.StatusOK,
		},
		{
			title:    "match GET handler",
			router:   router,
			method:   "GET",
			path:     "http://localhost/1/2",
			wantCode: http.StatusOK,
		},
		{
			title:    "match PATCH handler",
			router:   router,
			method:   "PATCH",
			path:     "http://localhost/1/2",
			wantCode: http.StatusOK,
		},
		{
			title:    "match PUT handler",
			router:   router,
			method:   "PUT",
			path:     "http://localhost/1/2",
			wantCode: http.StatusOK,
		},
		{
			title:    "disallow DELETE method",
			router:   router,
			method:   "DELETE",
			path:     "http://localhost/1/2",
			wantCode: http.StatusMethodNotAllowed,
		},
	}

	for _, test := range tests {
		t.Run(test.title, func(t *testing.T) {
			testMethodsSubrouter(t, test)
		})
	}
}

// TestMethodsSubrouterPathVariable matches handlers on matching paths
// with path variables in them.
func TestMethodsSubrouterPathVariable(t *testing.T) {
	t.Parallel()

	router := NewRouter()
	router.Methods("GET").Subrouter().HandleFunc("/foo", methodHandler("GET"))
	router.Methods("POST").Subrouter().HandleFunc("/{any}", methodHandler("POST"))
	router.Methods("DELETE").Subrouter().HandleFunc("/1/{any}", methodHandler("DELETE"))
	router.Methods("PUT").Subrouter().HandleFunc("/1/{any}", methodHandler("PUT"))

	tests := []methodsSubrouterTest{
		{
			title:    "match GET handler",
			router:   router,
			method:   "GET",
			path:     "http://localhost/foo",
			wantCode: http.StatusOK,
		},
		{
			title:    "match POST handler",
			router:   router,
			method:   "POST",
			path:     "http://localhost/foo",
			wantCode: http.StatusOK,
		},
		{
			title:    "match DELETE handler",
			router:   router,
			method:   "DELETE",
			path:     "http://localhost/1/foo",
			wantCode: http.StatusOK,
		},
		{
			title:    "match PUT handler",
			router:   router,
			method:   "PUT",
			path:     "http://localhost/1/foo",
			wantCode: http.StatusOK,
		},
		{
			title:    "disallow PATCH method",
			router:   router,
			method:   "PATCH",
			path:     "http://localhost/1/foo",
			wantCode: http.StatusMethodNotAllowed,
		},
	}

	for _, test := range tests {
		t.Run(test.title, func(t *testing.T) {
			testMethodsSubrouter(t, test)
		})
	}
}

func ExampleSetURLVars() {
	req, _ := http.NewRequest("GET", "/foo", nil)
	req = SetURLVars(req, map[string]string{"foo": "bar"})

	fmt.Println(Vars(req)["foo"])

	// Output: bar
}

// testMethodsSubrouter runs an individual methodsSubrouterTest.
func testMethodsSubrouter(t *testing.T, test methodsSubrouterTest) {
	// Execute request
	req, _ := http.NewRequest(test.method, test.path, nil)
	resp := NewRecorder()
	test.router.ServeHTTP(resp, req)

	switch test.wantCode {
	case http.StatusMethodNotAllowed:
		if resp.Code != http.StatusMethodNotAllowed {
			t.Errorf(`(%s) Expected "405 Method Not Allowed", but got %d code`, test.title, resp.Code)
		} else if matchedMethod := resp.Body.String(); matchedMethod != "" {
			t.Errorf(`(%s) Expected "405 Method Not Allowed", but %q handler was called`, test.title, matchedMethod)
		}

	case http.StatusMovedPermanently:
		if gotLocation := resp.HeaderMap.Get("Location"); gotLocation != test.redirectTo {
			t.Errorf("(%s) Expected %q route-match to redirect to %q, but got %q", test.title, test.method, test.redirectTo, gotLocation)
		}

	case http.StatusOK:
		if matchedMethod := resp.Body.String(); matchedMethod != test.method {
			t.Errorf("(%s) Expected %q handler to be called, but %q handler was called", test.title, test.method, matchedMethod)
		}

	default:
		expectedCodes := []int{http.StatusMethodNotAllowed, http.StatusMovedPermanently, http.StatusOK}
		t.Errorf("(%s) Expected wantCode to be one of: %v, but got %d", test.title, expectedCodes, test.wantCode)
	}
}

func TestSubrouterMatching(t *testing.T) {
	const (
		none, stdOnly, subOnly uint8 = 0, 1 << 0, 1 << 1
		both                         = subOnly | stdOnly
	)

	type request struct {
		Name    string
		Request *http.Request
		Flags   uint8
	}

	cases := []struct {
		Name                string
		Standard, Subrouter func(*Router)
		Requests            []request
	}{
		{
			"pathPrefix",
			func(r *Router) {
				r.PathPrefix("/before").PathPrefix("/after")
			},
			func(r *Router) {
				r.PathPrefix("/before").Subrouter().PathPrefix("/after")
			},
			[]request{
				{"no match final path prefix", newRequest("GET", "/after"), none},
				{"no match parent path prefix", newRequest("GET", "/before"), none},
				{"matches append", newRequest("GET", "/before/after"), both},
				{"matches as prefix", newRequest("GET", "/before/after/1234"), both},
			},
		},
		{
			"path",
			func(r *Router) {
				r.Path("/before").Path("/after")
			},
			func(r *Router) {
				r.Path("/before").Subrouter().Path("/after")
			},
			[]request{
				{"no match subroute path", newRequest("GET", "/after"), none},
				{"no match parent path", newRequest("GET", "/before"), none},
				{"no match as prefix", newRequest("GET", "/before/after/1234"), none},
				{"no match append", newRequest("GET", "/before/after"), none},
			},
		},
		{
			"host",
			func(r *Router) {
				r.Host("before.com").Host("after.com")
			},
			func(r *Router) {
				r.Host("before.com").Subrouter().Host("after.com")
			},
			[]request{
				{"no match before", newRequestHost("GET", "/", "before.com"), none},
				{"no match other", newRequestHost("GET", "/", "other.com"), none},
				{"matches after", newRequestHost("GET", "/", "after.com"), none},
			},
		},
		{
			"queries variant keys",
			func(r *Router) {
				r.Queries("foo", "bar").Queries("cricket", "baseball")
			},
			func(r *Router) {
				r.Queries("foo", "bar").Subrouter().Queries("cricket", "baseball")
			},
			[]request{
				{"matches with all", newRequest("GET", "/?foo=bar&cricket=baseball"), both},
				{"matches with more", newRequest("GET", "/?foo=bar&cricket=baseball&something=else"), both},
				{"no match with none", newRequest("GET", "/"), none},
				{"no match with some", newRequest("GET", "/?cricket=baseball"), none},
			},
		},
		{
			"queries overlapping keys",
			func(r *Router) {
				r.Queries("foo", "bar").Queries("foo", "baz")
			},
			func(r *Router) {
				r.Queries("foo", "bar").Subrouter().Queries("foo", "baz")
			},
			[]request{
				{"no match old value", newRequest("GET", "/?foo=bar"), none},
				{"no match diff value", newRequest("GET", "/?foo=bak"), none},
				{"no match with none", newRequest("GET", "/"), none},
				{"matches override", newRequest("GET", "/?foo=baz"), none},
			},
		},
		{
			"header variant keys",
			func(r *Router) {
				r.Headers("foo", "bar").Headers("cricket", "baseball")
			},
			func(r *Router) {
				r.Headers("foo", "bar").Subrouter().Headers("cricket", "baseball")
			},
			[]request{
				{
					"matches with all",
					newRequestWithHeaders("GET", "/", "foo", "bar", "cricket", "baseball"),
					both,
				},
				{
					"matches with more",
					newRequestWithHeaders("GET", "/", "foo", "bar", "cricket", "baseball", "something", "else"),
					both,
				},
				{"no match with none", newRequest("GET", "/"), none},
				{"no match with some", newRequestWithHeaders("GET", "/", "cricket", "baseball"), none},
			},
		},
		{
			"header overlapping keys",
			func(r *Router) {
				r.Headers("foo", "bar").Headers("foo", "baz")
			},
			func(r *Router) {
				r.Headers("foo", "bar").Subrouter().Headers("foo", "baz")
			},
			[]request{
				{"no match old value", newRequestWithHeaders("GET", "/", "foo", "bar"), none},
				{"no match diff value", newRequestWithHeaders("GET", "/", "foo", "bak"), none},
				{"no match with none", newRequest("GET", "/"), none},
				{"matches override", newRequestWithHeaders("GET", "/", "foo", "baz"), none},
			},
		},
		{
			"method",
			func(r *Router) {
				r.Methods("POST").Methods("GET")
			},
			func(r *Router) {
				r.Methods("POST").Subrouter().Methods("GET")
			},
			[]request{
				{"matches before", newRequest("POST", "/"), none},
				{"no match other", newRequest("HEAD", "/"), none},
				{"matches override", newRequest("GET", "/"), none},
			},
		},
		{
			"schemes",
			func(r *Router) {
				r.Schemes("http").Schemes("https")
			},
			func(r *Router) {
				r.Schemes("http").Subrouter().Schemes("https")
			},
			[]request{
				{"matches overrides", newRequest("GET", "https://www.example.com/"), none},
				{"matches original", newRequest("GET", "http://www.example.com/"), none},
				{"no match other", newRequest("GET", "ftp://www.example.com/"), none},
			},
		},
	}

	// case -> request -> router
	for _, c := range cases {
		t.Run(c.Name, func(t *testing.T) {
			for _, req := range c.Requests {
				t.Run(req.Name, func(t *testing.T) {
					for _, v := range []struct {
						Name     string
						Config   func(*Router)
						Expected bool
					}{
						{"subrouter", c.Subrouter, (req.Flags & subOnly) != 0},
						{"standard", c.Standard, (req.Flags & stdOnly) != 0},
					} {
						r := NewRouter()
						v.Config(r)
						if r.Match(req.Request, &RouteMatch{}) != v.Expected {
							if v.Expected {
								t.Errorf("expected %v match", v.Name)
							} else {
								t.Errorf("expected %v no match", v.Name)
							}
						}
					}
				})
			}
		})
	}
}

// verify that copyRouteConf copies fields as expected.
func Test_copyRouteConf(t *testing.T) {
	var (
		m MatcherFunc = func(*http.Request, *RouteMatch) bool {
			return true
		}
		b BuildVarsFunc = func(i map[string]string) map[string]string {
			return i
		}
		r, _ = newRouteRegexp("hi", regexpTypeHost, routeRegexpOptions{})
	)

	tests := []struct {
		name string
		args routeConf
		want routeConf
	}{
		{
			"empty",
			routeConf{},
			routeConf{},
		},
		{
			"full",
			routeConf{
				useEncodedPath: true,
				strictSlash:    true,
				skipClean:      true,
				regexp:         routeRegexpGroup{host: r, path: r, queries: []*routeRegexp{r}},
				matchers:       []matcher{m},
				buildScheme:    "https",
				buildVarsFunc:  b,
			},
			routeConf{
				useEncodedPath: true,
				strictSlash:    true,
				skipClean:      true,
				regexp:         routeRegexpGroup{host: r, path: r, queries: []*routeRegexp{r}},
				matchers:       []matcher{m},
				buildScheme:    "https",
				buildVarsFunc:  b,
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// special case some incomparable fields of routeConf before delegating to reflect.DeepEqual
			got := copyRouteConf(tt.args)

			// funcs not comparable, just compare length of slices
			if len(got.matchers) != len(tt.want.matchers) {
				t.Errorf("matchers different lengths: %v %v", len(got.matchers), len(tt.want.matchers))
			}
			got.matchers, tt.want.matchers = nil, nil

			// deep equal treats nil slice differently to empty slice so check for zero len first
			{
				bothZero := len(got.regexp.queries) == 0 && len(tt.want.regexp.queries) == 0
				if !bothZero && !reflect.DeepEqual(got.regexp.queries, tt.want.regexp.queries) {
					t.Errorf("queries unequal: %v %v", got.regexp.queries, tt.want.regexp.queries)
				}
				got.regexp.queries, tt.want.regexp.queries = nil, nil
			}

			// funcs not comparable, just compare nullity
			if (got.buildVarsFunc == nil) != (tt.want.buildVarsFunc == nil) {
				t.Errorf("build vars funcs unequal: %v %v", got.buildVarsFunc == nil, tt.want.buildVarsFunc == nil)
			}
			got.buildVarsFunc, tt.want.buildVarsFunc = nil, nil

			// finish the deal
			if !reflect.DeepEqual(got, tt.want) {
				t.Errorf("route confs unequal: %v %v", got, tt.want)
			}
		})
	}
}

func TestMethodNotAllowed(t *testing.T) {
	handler := func(w http.ResponseWriter, r *http.Request) { w.WriteHeader(http.StatusOK) }
	router := NewRouter()
	router.HandleFunc("/thing", handler).Methods(http.MethodGet)
	router.HandleFunc("/something", handler).Methods(http.MethodGet)

	w := NewRecorder()
	req := newRequest(http.MethodPut, "/thing")

	router.ServeHTTP(w, req)

	if w.Code != http.StatusMethodNotAllowed {
		t.Fatalf("Expected status code 405 (got %d)", w.Code)
	}
}

type customMethodNotAllowedHandler struct {
	msg string
}

func (h customMethodNotAllowedHandler) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	w.WriteHeader(http.StatusMethodNotAllowed)
	fmt.Fprint(w, h.msg)
}

func TestSubrouterCustomMethodNotAllowed(t *testing.T) {
	handler := func(w http.ResponseWriter, r *http.Request) { w.WriteHeader(http.StatusOK) }

	router := NewRouter()
	router.HandleFunc("/test", handler).Methods(http.MethodGet)
	router.MethodNotAllowedHandler = customMethodNotAllowedHandler{msg: "custom router handler"}

	subrouter := router.PathPrefix("/sub").Subrouter()
	subrouter.HandleFunc("/test", handler).Methods(http.MethodGet)
	subrouter.MethodNotAllowedHandler = customMethodNotAllowedHandler{msg: "custom sub router handler"}

	testCases := map[string]struct {
		path   string
		expMsg string
	}{
		"router method not allowed": {
			path:   "/test",
			expMsg: "custom router handler",
		},
		"subrouter method not allowed": {
			path:   "/sub/test",
			expMsg: "custom sub router handler",
		},
	}

	for name, tc := range testCases {
		t.Run(name, func(tt *testing.T) {
			w := NewRecorder()
			req := newRequest(http.MethodPut, tc.path)

			router.ServeHTTP(w, req)

			if w.Code != http.StatusMethodNotAllowed {
				tt.Errorf("Expected status code 405 (got %d)", w.Code)
			}

			b, err := io.ReadAll(w.Body)
			if err != nil {
				tt.Errorf("failed to read body: %v", err)
			}

			if string(b) != tc.expMsg {
				tt.Errorf("expected msg %q, got %q", tc.expMsg, string(b))
			}
		})
	}
}

func TestSubrouterNotFound(t *testing.T) {
	handler := func(w http.ResponseWriter, r *http.Request) { w.WriteHeader(http.StatusOK) }
	router := NewRouter()
	router.Path("/a").Subrouter().HandleFunc("/thing", handler).Methods(http.MethodGet)
	router.Path("/b").Subrouter().HandleFunc("/something", handler).Methods(http.MethodGet)

	w := NewRecorder()
	req := newRequest(http.MethodPut, "/not-present")

	router.ServeHTTP(w, req)

	if w.Code != http.StatusNotFound {
		t.Fatalf("Expected status code 404 (got %d)", w.Code)
	}
}

func TestContextMiddleware(t *testing.T) {
	withTimeout := func(h http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			ctx, cancel := context.WithTimeout(r.Context(), time.Minute)
			defer cancel()
			h.ServeHTTP(w, r.WithContext(ctx))
		})
	}

	r := NewRouter()
	r.Handle("/path/{foo}", withTimeout(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		vars := Vars(r)
		if vars["foo"] != "bar" {
			t.Fatal("Expected foo var to be set")
		}
	})))

	rec := NewRecorder()
	req := newRequest("GET", "/path/bar")
	r.ServeHTTP(rec, req)
}

func TestGetVarNames(t *testing.T) {
	r := NewRouter()

	route := r.Host("{domain}").
		Path("/{group}/{item_id}").
		Queries("some_data1", "{some_data1}").
		Queries("some_data2_and_3", "{some_data2}.{some_data3}")

	// Order of vars in the slice is not guaranteed, so just check for existence
	expected := map[string]bool{
		"domain":     true,
		"group":      true,
		"item_id":    true,
		"some_data1": true,
		"some_data2": true,
		"some_data3": true,
	}

	varNames, err := route.GetVarNames()
	if err != nil {
		t.Fatal(err)
	}

	if len(varNames) != len(expected) {
		t.Fatalf("expected %d names, got %d", len(expected), len(varNames))
	}

	for _, varName := range varNames {
		if !expected[varName] {
			t.Fatalf("got unexpected %s", varName)
		}
	}
}

// mapToPairs converts a string map to a slice of string pairs
func mapToPairs(m map[string]string) []string {
	var i int
	p := make([]string, len(m)*2)
	for k, v := range m {
		p[i] = k
		p[i+1] = v
		i += 2
	}
	return p
}

// stringMapEqual checks the equality of two string maps
func stringMapEqual(m1, m2 map[string]string) bool {
	nil1 := m1 == nil
	nil2 := m2 == nil
	if nil1 != nil2 || len(m1) != len(m2) {
		return false
	}
	for k, v := range m1 {
		if v != m2[k] {
			return false
		}
	}
	return true
}

// stringHandler returns a handler func that writes a message 's' to the
// http.ResponseWriter.
func stringHandler(s string) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		_, err := w.Write([]byte(s))
		if err != nil {
			log.Printf("Failed writing HTTP response: %v", err)
		}
	}
}

// newRequest is a helper function to create a new request with a method and url.
// The request returned is a 'server' request as opposed to a 'client' one through
// simulated write onto the wire and read off of the wire.
// The differences between requests are detailed in the net/http package.
func newRequest(method, url string) *http.Request {
	req, err := http.NewRequest(method, url, nil)
	if err != nil {
		panic(err)
	}
	// extract the escaped original host+path from url
	// http://localhost/path/here?v=1#frag -> //localhost/path/here
	opaque := ""
	if i := len(req.URL.Scheme); i > 0 {
		opaque = url[i+1:]
	}

	if i := strings.LastIndex(opaque, "?"); i > -1 {
		opaque = opaque[:i]
	}
	if i := strings.LastIndex(opaque, "#"); i > -1 {
		opaque = opaque[:i]
	}

	// Escaped host+path workaround as detailed in https://golang.org/pkg/net/url/#URL
	// for < 1.5 client side workaround
	req.URL.Opaque = opaque

	// Simulate writing to wire
	var buff bytes.Buffer
	err = req.Write(&buff)
	if err != nil {
		log.Printf("Failed writing HTTP request: %v", err)
	}
	ioreader := bufio.NewReader(&buff)

	// Parse request off of 'wire'
	req, err = http.ReadRequest(ioreader)
	if err != nil {
		panic(err)
	}
	return req
}

// create a new request with the provided headers
func newRequestWithHeaders(method, url string, headers ...string) *http.Request {
	req := newRequest(method, url)

	if len(headers)%2 != 0 {
		panic(fmt.Sprintf("Expected headers length divisible by 2 but got %v", len(headers)))
	}

	for i := 0; i < len(headers); i += 2 {
		req.Header.Set(headers[i], headers[i+1])
	}

	return req
}

// newRequestHost a new request with a method, url, and host header
func newRequestHost(method, url, host string) *http.Request {
	req := httptest.NewRequest(method, url, nil)
	req.Host = host
	return req
}
