// Copyright 2019 The Cockroach Authors.
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

package errbase_test

import (
	"context"
	"errors"
	"fmt"
	"reflect"
	"strings"
	"testing"

	"github.com/cockroachdb/errors/errbase"
	"github.com/cockroachdb/errors/errbase/internal"
	"github.com/cockroachdb/errors/testutils"
	"github.com/gogo/protobuf/proto"
	"github.com/kr/pretty"
)

// These tests demonstrate how the library makes a best effort at
// preserving types it does not know about.

// myError is a type that the rest of the library has no idea about.
type myError struct{ val int }

func (m *myError) Error() string { return fmt.Sprintf("hello %d", m.val) }

func TestEncodeUnknownError(t *testing.T) {
	origErr := &myError{val: 123}
	t.Logf("start err: %# v", pretty.Formatter(origErr))

	newErr := network(t, origErr)

	tt := testutils.T{T: t}

	// In any case, the library preserves the error message!
	tt.CheckEqual(newErr.Error(), origErr.Error())

	// However, since the type is unknown and is not protobuf-encodable
	// natively, the library is unable to preserve the Go type.
	tt.Check(!reflect.DeepEqual(origErr, newErr))

	// That being said, it will remember the original type of the error,
	// which can aid in troubleshooting.
	details := fmt.Sprintf("%+v", newErr)
	tt.Check(strings.Contains(details, "myError"))
}

func TestUnknownErrorTraversal(t *testing.T) {
	// This tests hows that if a decoder for an error type becomes
	// available after the error has traversed the network
	// as "unknown", it can be recovered fully.
	origErr := &myError{val: 123}
	t.Logf("start err: %# v", pretty.Formatter(origErr))

	// Register a temporary encoder.
	myEncode := func(_ context.Context, err error) (string, []string, proto.Message) {
		m := err.(*myError)
		return "", nil, &internal.MyPayload{Val: int32(m.val)}
	}
	tn := errbase.GetTypeKey((*myError)(nil))
	errbase.RegisterLeafEncoder(tn, myEncode)

	// Encode the error, this will use the encoder.
	enc := errbase.EncodeError(context.Background(), origErr)
	t.Logf("encoded: %# v", pretty.Formatter(enc))

	// Forget about the encoder.
	errbase.RegisterLeafEncoder(tn, nil)

	// Simulate the error traversing a node that knows nothing about the
	// error (it doesn't know about the type)

	newErr := errbase.DecodeError(context.Background(), enc)
	t.Logf("decoded: %# v", pretty.Formatter(newErr))

	if _, ok := newErr.(*myError); ok {
		t.Errorf("unexpected: type was preserved")
	}

	// Encode it again, to simulate the error passed on to another system.
	enc2 := errbase.EncodeError(context.Background(), newErr)
	t.Logf("encoded2: %# v", pretty.Formatter(enc))

	// Now register a temporary decoder.
	myDecode := func(_ context.Context, _ string, _ []string, payload proto.Message) error {
		return &myError{val: int(payload.(*internal.MyPayload).Val)}
	}
	errbase.RegisterLeafDecoder(tn, myDecode)

	// Then decode again.
	newErr2 := errbase.DecodeError(context.Background(), enc2)
	t.Logf("decoded: %# v", pretty.Formatter(newErr2))

	// Forget about the decoder so as to not pollute other tests.
	errbase.RegisterLeafDecoder(tn, nil)

	// The original object has been restored!
	tt := testutils.T{T: t}
	tt.CheckDeepEqual(newErr2, origErr)
}

// myWrap is a wrapper type that the rest of the library has no idea about.
type myWrap struct {
	cause error
	val   int
}

func (m *myWrap) Cause() error { return m.cause }
func (m *myWrap) Error() string {
	return fmt.Sprintf("hi %d: %s", m.val, m.cause)
}

func TestEncodeUnknownWrapper(t *testing.T) {
	origErr := &myWrap{cause: errors.New("hello"), val: 123}
	t.Logf("start err: %# v", pretty.Formatter(origErr))

	newErr := network(t, origErr)

	tt := testutils.T{T: t}

	// In any case, the library preserves the error message!
	tt.CheckEqual(newErr.Error(), origErr.Error())

	// Also, if the cause inside the wrapper was perfectly encodable,
	// the cause will be preserved even if the wrapper was not.
	c, ok := newErr.(interface{ Cause() error })
	if !ok {
		t.Errorf("unexpected: decoded error is not causer")
	} else {
		cause := c.Cause()
		tt.CheckDeepEqual(cause, origErr.cause)
	}

	// However, since the type is unknown and is not protobuf-encodable
	// natively, the library is unable to preserve the Go type.
	tt.Check(!reflect.DeepEqual(origErr, newErr))

	// That being said, it will remember the original type of the error,
	// which can aid in troubleshooting.
	details := fmt.Sprintf("%+v", newErr)
	tt.Check(strings.Contains(details, "myWrap"))
}

func TestUnknownWrapperTraversal(t *testing.T) {
	// This tests hows that if a decoder for an wrapper type becomes
	// available after the wrapper has traversed the network
	// as "unknown", it can be recovered fully.
	origErr := &myWrap{cause: errors.New("hello"), val: 123}
	t.Logf("start err: %# v", pretty.Formatter(origErr))

	// Register a temporary encoder.
	myEncode := func(_ context.Context, err error) (string, []string, proto.Message) {
		m := err.(*myWrap)
		return "", nil, &internal.MyPayload{Val: int32(m.val)}
	}
	tn := errbase.GetTypeKey((*myWrap)(nil))
	errbase.RegisterWrapperEncoder(tn, myEncode)

	// Encode the error, this will use the encoder.
	enc := errbase.EncodeError(context.Background(), origErr)
	t.Logf("encoded: %# v", pretty.Formatter(enc))

	// Forget about the encoder.
	errbase.RegisterWrapperEncoder(tn, nil)

	// Simulate the error traversing a node that knows nothing about the
	// error (it doesn't know about the type)

	newErr := errbase.DecodeError(context.Background(), enc)
	t.Logf("decoded: %# v", pretty.Formatter(newErr))

	if _, ok := newErr.(*myWrap); ok {
		t.Errorf("unexpected: type was preserved")
	}

	// Encode it again, to simulate the error passed on to another system.
	enc2 := errbase.EncodeError(context.Background(), newErr)
	t.Logf("encoded2: %# v", pretty.Formatter(enc))

	// Now register a temporary decoder.
	myDecode := func(_ context.Context,
		cause error, _ string, _ []string, payload proto.Message) error {
		return &myWrap{cause: cause, val: int(payload.(*internal.MyPayload).Val)}
	}
	errbase.RegisterWrapperDecoder(tn, myDecode)

	// Then decode again.
	newErr2 := errbase.DecodeError(context.Background(), enc2)
	t.Logf("decoded: %# v", pretty.Formatter(newErr2))

	// Forget about the decoder so as to not pollute other tests.
	errbase.RegisterWrapperDecoder(tn, nil)

	tt := testutils.T{T: t}

	// The original object has been restored!
	tt.CheckDeepEqual(newErr2, origErr)
}
