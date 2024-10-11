// Copyright 2018, Google Inc.
// All rights reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are
// met:
//
//     * Redistributions of source code must retain the above copyright
// notice, this list of conditions and the following disclaimer.
//     * Redistributions in binary form must reproduce the above
// copyright notice, this list of conditions and the following disclaimer
// in the documentation and/or other materials provided with the
// distribution.
//     * Neither the name of Google Inc. nor the names of its
// contributors may be used to endorse or promote products derived from
// this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

package gax

import (
	"context"
	"net/http"
	"testing"

	"github.com/google/go-cmp/cmp"
	"github.com/googleapis/gax-go/v2/callctx"
	"google.golang.org/grpc/metadata"
)

func TestXGoogHeader(t *testing.T) {
	for _, tst := range []struct {
		kv   []string
		want string
	}{
		{nil, ""},
		{[]string{"abc", "def"}, "abc/def"},
		{[]string{"abc", "def", "xyz", "123", "foo", ""}, "abc/def xyz/123 foo/"},
	} {
		got := XGoogHeader(tst.kv...)
		if got != tst.want {
			t.Errorf("Header(%q) = %q, want %q", tst.kv, got, tst.want)
		}
	}
}

func TestGoVersion(t *testing.T) {
	testVersion := func(v string) func() string {
		return func() string {
			return v
		}
	}
	for _, tst := range []struct {
		v    func() string
		want string
	}{
		{
			testVersion("go1.19"),
			"1.19.0",
		},
		{
			testVersion("go1.21-20230317-RC01"),
			"1.21.0-20230317-RC01",
		},
		{
			testVersion("devel +abc1234"),
			"abc1234",
		},
		{
			testVersion("this should be unknown"),
			versionUnknown,
		},
		{
			testVersion("go1.21-20230101-RC01 cl/1234567 +abc1234"),
			"1.21.0-20230101-RC01",
		},
	} {
		version = tst.v
		got := goVersion()
		if diff := cmp.Diff(got, tst.want); diff != "" {
			t.Errorf("got(-),want(+):\n%s", diff)
		}
	}
}

func TestInsertMetadataIntoOutgoingContext(t *testing.T) {
	for _, tst := range []struct {
		// User-provided metadata set in context
		userMd metadata.MD
		// User-provided headers set in context
		userHeaders []string
		// Client-provided headers passed to func
		clientHeaders []string
		want          metadata.MD
	}{
		{
			userMd: metadata.Pairs("key_1", "val_1", "key_2", "val_21"),
			want:   metadata.Pairs("key_1", "val_1", "key_2", "val_21"),
		},
		{
			userHeaders: []string{"key_2", "val_22"},
			want:        metadata.Pairs("key_2", "val_22"),
		},
		{
			clientHeaders: []string{"key_2", "val_23", "key_2", "val_24"},
			want:          metadata.Pairs("key_2", "val_23", "key_2", "val_24"),
		},
		{
			userMd:        metadata.Pairs("key_1", "val_1", "key_2", "val_21"),
			userHeaders:   []string{"key_2", "val_22"},
			clientHeaders: []string{"key_2", "val_23", "key_2", "val_24"},
			want:          metadata.Pairs("key_1", "val_1", "key_2", "val_21", "key_2", "val_22", "key_2", "val_23", "key_2", "val_24"),
		},
		{
			clientHeaders: []string{"x-goog-api-client", "val_23 val_22", "key_2", "val_24", "x-goog-api-client", "val_1"},
			want:          metadata.Pairs("key_2", "val_24", "x-goog-api-client", "val_23 val_22 val_1"),
		},
		{
			userHeaders:   []string{"key_2", "val_22", "x-goog-api-client", "val_1 val_2"},
			clientHeaders: []string{"x-goog-api-client", "val_3 val_4", "key_2", "val_24", "x-goog-api-client", "val_11 val_22"},
			want:          metadata.Pairs("key_2", "val_22", "key_2", "val_24", "x-goog-api-client", "val_1 val_2 val_3 val_4 val_11 val_22"),
		},
	} {
		ctx := context.Background()
		if tst.userMd != nil {
			ctx = metadata.NewOutgoingContext(ctx, tst.userMd)
		}
		ctx = callctx.SetHeaders(ctx, tst.userHeaders...)

		ctx = InsertMetadataIntoOutgoingContext(ctx, tst.clientHeaders...)

		got, _ := metadata.FromOutgoingContext(ctx)
		if diff := cmp.Diff(tst.want, got); diff != "" {
			t.Errorf("InsertMetadata(ctx, %q) mismatch (-want +got):\n%s", tst.clientHeaders, diff)
		}
	}
}

func TestBuildHeaders(t *testing.T) {
	// User-provided metadata set in context
	existingMd := metadata.Pairs("key_1", "val_1", "key_2", "val_21")
	ctx := metadata.NewOutgoingContext(context.Background(), existingMd)
	// User-provided headers set in context
	ctx = callctx.SetHeaders(ctx, "key_2", "val_22")
	// Client-provided headers
	keyvals := []string{"key_2", "val_23", "key_2", "val_24"}

	got := BuildHeaders(ctx, keyvals...)

	want := http.Header{"key_1": []string{"val_1"}, "key_2": []string{"val_21", "val_22", "val_23", "val_24"}}
	if diff := cmp.Diff(want, got); diff != "" {
		t.Errorf("InsertMetadata(ctx, %q) mismatch (-want +got):\n%s", keyvals, diff)
	}
}
