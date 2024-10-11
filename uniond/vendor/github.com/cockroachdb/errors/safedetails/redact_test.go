// Copyright 2020 The Cockroach Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
// implied. See the License for the specific language governing
// permissions and limitations under the License.

package safedetails_test

import (
	"context"
	"errors"
	"net"
	"os"
	"regexp"
	"runtime"
	"testing"

	"github.com/cockroachdb/errors/safedetails"
	"github.com/cockroachdb/errors/testutils"
	"github.com/cockroachdb/redact"
)

func TestRedact(t *testing.T) {
	errSentinel := (error)(struct{ error }{})

	// rm is what's left over after redaction.
	rm := string(redact.RedactableBytes(redact.RedactedMarker()).StripMarkers())

	testData := []struct {
		obj      interface{}
		expected string
	}{
		// Redacting non-error values.

		{123, rm},
		{"secret", rm},

		// Redacting SafeMessagers.

		{mySafer{}, `hello`},

		{safedetails.Safe(123), `123`},

		{mySafeError{}, `hello`},

		{&werrFmt{mySafeError{}, "unseen"}, rm + `: hello`},

		// Redacting errors.

		// Unspecial cases, get redacted.
		{errors.New("secret"), rm},

		// Special cases, unredacted.
		{os.ErrInvalid, `invalid argument`},
		{os.ErrPermission, `permission denied`},
		{os.ErrExist, `file already exists`},
		{os.ErrNotExist, `file does not exist`},
		{os.ErrClosed, `file already closed`},
		{os.ErrNoDeadline, `file type does not support deadline`},

		{context.Canceled, `context canceled`},
		{context.DeadlineExceeded, `context deadline exceeded`},

		{makeTypeAssertionErr(), `interface conversion: interface {} is nil, not int`},

		{errSentinel, // explodes if Error() called
			`%!v(PANIC=SafeFormatter method: runtime error: invalid memory address or nil pointer dereference)`},

		{&werrFmt{&werrFmt{os.ErrClosed, "unseen"}, "unsung"},
			rm + `: ` + rm + `: file already closed`},

		// Special cases, get partly redacted.

		{os.NewSyscallError("rename", os.ErrNotExist),
			`rename: file does not exist`},

		{&os.PathError{Op: "rename", Path: "secret", Err: os.ErrNotExist},
			`rename ` + rm + `: file does not exist`},

		{&os.LinkError{
			Op:  "moo",
			Old: "sec",
			New: "cret",
			Err: os.ErrNotExist,
		},
			`moo ` + rm + ` ` + rm + `: file does not exist`},

		{&net.OpError{
			Op:     "write",
			Net:    "tcp",
			Source: &net.IPAddr{IP: net.IP("sensitive-source")},
			Addr:   &net.IPAddr{IP: net.IP("sensitive-addr")},
			Err:    errors.New("not safe"),
		}, `write tcp ` + rm + ` -> ` + rm + `: ` + rm},
	}

	tt := testutils.T{T: t}

	for _, tc := range testData {
		s := safedetails.Redact(tc.obj)
		s = fileref.ReplaceAllString(s, "...path...")

		tt.CheckStringEqual(s, tc.expected)
	}
}

var fileref = regexp.MustCompile(`([a-zA-Z0-9\._/@-]*\.(?:go|s):\d+)`)

// makeTypeAssertionErr returns a runtime.Error with the message:
//     interface conversion: interface {} is nil, not int
func makeTypeAssertionErr() (result runtime.Error) {
	defer func() {
		e := recover()
		result = e.(runtime.Error)
	}()
	var x interface{}
	_ = x.(int)
	return nil
}

type mySafer struct{}

func (mySafer) SafeMessage() string { return "hello" }

type mySafeError struct{}

func (mySafeError) SafeMessage() string { return "hello" }
func (mySafeError) Error() string       { return "helloerr" }
