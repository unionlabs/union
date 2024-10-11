// Copyright 2022, Google Inc.
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
	"bytes"
	"errors"
	"io"
	"io/ioutil"
	"testing"
	"time"

	"github.com/google/go-cmp/cmp"
	serviceconfigpb "google.golang.org/genproto/googleapis/api/serviceconfig"
	"google.golang.org/genproto/googleapis/rpc/code"
	"google.golang.org/genproto/googleapis/rpc/status"
	"google.golang.org/protobuf/encoding/protojson"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/reflect/protoreflect"
	"google.golang.org/protobuf/types/known/anypb"
	"google.golang.org/protobuf/types/known/durationpb"
)

func TestRecv(t *testing.T) {
	locations := []proto.Message{
		&serviceconfigpb.Property{
			Name:        "property1",
			Type:        serviceconfigpb.Property_STRING,
			Description: "Property 1",
		},
		&serviceconfigpb.Property{
			Name:        "property2",
			Type:        serviceconfigpb.Property_STRING,
			Description: "Property 2",
		},
		&serviceconfigpb.Property{
			Name:        "property3",
			Type:        serviceconfigpb.Property_STRING,
			Description: "Property 3",
		},
	}

	durations := []proto.Message{
		durationpb.New(time.Second),
		durationpb.New(time.Minute),
		durationpb.New(time.Hour),
	}

	detail, err := anypb.New(locations[0])
	if err != nil {
		t.Fatal(err)
	}
	nested := []proto.Message{
		&status.Status{
			Code:    int32(code.Code_INTERNAL),
			Message: "oops",
			Details: []*anypb.Any{
				detail,
			},
		},
	}

	for _, tst := range []struct {
		name string
		want []proto.Message
		typ  protoreflect.MessageType
	}{
		{
			name: "empty",
		},
		{
			name: "simple_locations",
			want: locations,
			typ:  locations[0].ProtoReflect().Type(),
		},
		{
			// google.type.Duration is JSON encoded as a string, not an object.
			name: "message_as_primitive",
			want: durations,
			typ:  durations[0].ProtoReflect().Type(),
		},
		{
			name: "nested",
			want: nested,
			typ:  nested[0].ProtoReflect().Type(),
		},
	} {
		s, err := prepareStream(tst.want)
		if err != nil {
			t.Errorf("%s: %v", tst.name, err)
			continue
		}
		stream := NewProtoJSONStreamReader(s, tst.typ)
		defer stream.Close()

		got, err := stream.Recv()
		for ndx := 0; err == nil; ndx++ {
			if diff := cmp.Diff(got, tst.want[ndx], cmp.Comparer(proto.Equal)); diff != "" {
				t.Errorf("%s: got(-),want(+):\n%s", tst.name, diff)
			}
			got, err = stream.Recv()
		}
		if !errors.Is(err, io.EOF) {
			t.Errorf("%s: expected %v but got %v", tst.name, io.EOF, err)
		}
	}
}

func TestRecvAfterClose(t *testing.T) {
	empty := ioutil.NopCloser(bytes.NewReader([]byte("[]")))
	s := NewProtoJSONStreamReader(empty, nil)
	if _, err := s.Recv(); !errors.Is(err, io.EOF) {
		t.Errorf("Expected %v but got %v", io.EOF, err)
	}

	// Close to ensure reader is closed.
	s.Close()
	if _, err := s.Recv(); !errors.Is(err, io.EOF) {
		t.Errorf("Expected %v after close but got %v", io.EOF, err)
	}

}

func TestRecvError(t *testing.T) {
	noOpening := ioutil.NopCloser(bytes.NewReader([]byte{'{'}))
	s := NewProtoJSONStreamReader(noOpening, nil)
	if _, err := s.Recv(); !errors.Is(err, errBadOpening) {
		t.Errorf("Expected %v but got %v", errBadOpening, err)
	}
}

func prepareStream(messages []proto.Message) (io.ReadCloser, error) {
	if len(messages) == 0 {
		return ioutil.NopCloser(bytes.NewReader([]byte("[]"))), nil
	}

	data := []byte("[")
	mo := protojson.MarshalOptions{AllowPartial: true, UseEnumNumbers: true}
	for _, m := range messages {
		d, err := mo.Marshal(m)
		if err != nil {
			return nil, err
		}
		data = append(data, d...)
		data = append(data, ',')
	}
	// Set the trailing ',' to a closing ']'.
	data[len(data)-1] = ']'
	return ioutil.NopCloser(bytes.NewReader(data)), nil
}
