// Copyright 2022 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

package main

import (
	"fmt"
	"net"
	"net/http"
	"sync/atomic"
	"testing"
)

type testHandler struct {
	body func(http.ResponseWriter, *http.Request)
}

func (t *testHandler) ServeHTTP(out http.ResponseWriter, in *http.Request) {
	t.body(out, in)
}

// TestAttachWithHeaders tests that 'geth attach' with custom headers works, i.e
// that custom headers are forwarded to the target.
func TestAttachWithHeaders(t *testing.T) {
	t.Parallel()
	ln, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatal(err)
	}
	port := ln.Addr().(*net.TCPAddr).Port
	testReceiveHeaders(t, ln, "attach", "-H", "first: one", "-H", "second: two", fmt.Sprintf("http://localhost:%d", port))
	// This way to do it fails due to flag ordering:
	//
	// testReceiveHeaders(t, ln, "-H", "first: one", "-H", "second: two", "attach", fmt.Sprintf("http://localhost:%d", port))
	// This is fixed in a follow-up PR.
}

// TestAttachWithHeaders tests that 'geth db --remotedb' with custom headers works, i.e
// that custom headers are forwarded to the target.
func TestRemoteDbWithHeaders(t *testing.T) {
	t.Parallel()
	ln, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatal(err)
	}
	port := ln.Addr().(*net.TCPAddr).Port
	testReceiveHeaders(t, ln, "db", "metadata", "--remotedb", fmt.Sprintf("http://localhost:%d", port), "-H", "first: one", "-H", "second: two")
}

func testReceiveHeaders(t *testing.T, ln net.Listener, gethArgs ...string) {
	var ok atomic.Uint32
	server := &http.Server{
		Addr: "localhost:0",
		Handler: &testHandler{func(w http.ResponseWriter, r *http.Request) {
			// We expect two headers
			if have, want := r.Header.Get("first"), "one"; have != want {
				t.Fatalf("missing header, have %v want %v", have, want)
			}
			if have, want := r.Header.Get("second"), "two"; have != want {
				t.Fatalf("missing header, have %v want %v", have, want)
			}
			ok.Store(1)
		}}}
	go server.Serve(ln)
	defer server.Close()
	runGeth(t, gethArgs...).WaitExit()
	if ok.Load() != 1 {
		t.Fatal("Test fail, expected invocation to succeed")
	}
}
