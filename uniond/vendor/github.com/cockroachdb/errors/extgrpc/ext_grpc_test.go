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

package extgrpc_test

import (
	"context"
	"fmt"
	"reflect"
	"strings"
	"testing"
	"unicode"

	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/errors/errbase"
	"github.com/cockroachdb/errors/errorspb"
	"github.com/cockroachdb/errors/extgrpc"
	"github.com/cockroachdb/errors/testutils"
	"github.com/gogo/protobuf/proto"
	gogostatus "github.com/gogo/status"
	"github.com/stretchr/testify/require"
	"google.golang.org/grpc/codes"
	grpcstatus "google.golang.org/grpc/status"
	"google.golang.org/protobuf/runtime/protoiface"
)

func TestGrpc(t *testing.T) {
	err := fmt.Errorf("hello")
	err = extgrpc.WrapWithGrpcCode(err, codes.Unavailable)

	// Simulate a network transfer.
	enc := errors.EncodeError(context.Background(), err)
	otherErr := errors.DecodeError(context.Background(), enc)

	tt := testutils.T{T: t}

	// Error is preserved through the network.
	tt.CheckDeepEqual(otherErr, err)

	// It's possible to extract the Grpc code.
	tt.CheckEqual(extgrpc.GetGrpcCode(otherErr), codes.Unavailable)

	// If there are multiple codes, the most recent one wins.
	otherErr = extgrpc.WrapWithGrpcCode(otherErr, codes.NotFound)
	tt.CheckEqual(extgrpc.GetGrpcCode(otherErr), codes.NotFound)

	// The code is hidden when the error is printed with %v.
	tt.CheckStringEqual(fmt.Sprintf("%v", err), `hello`)
	// The code appears when the error is printed verbosely.
	tt.CheckStringEqual(fmt.Sprintf("%+v", err), `hello
(1) gRPC code: Unavailable
Wraps: (2) hello
Error types: (1) *extgrpc.withGrpcCode (2) *errors.errorString`)

	// Checking the code of a nil error should be codes.OK
	var noErr error
	tt.Assert(extgrpc.GetGrpcCode(noErr) == codes.OK)
}

// dummyProto is a dummy Protobuf message which satisfies the proto.Message
// interface but is not registered with either the standard Protobuf or GoGo
// Protobuf type registries.
type dummyProto struct {
	value string
}

func (p *dummyProto) Reset()         {}
func (p *dummyProto) String() string { return "" }
func (p *dummyProto) ProtoMessage()  {}

// statusIface is a thin interface for common gRPC and gogo Status functionality.
type statusIface interface {
	Code() codes.Code
	Message() string
	Details() []interface{}
	Err() error
}

func TestEncodeDecodeStatus(t *testing.T) {
	testcases := []struct {
		desc          string
		makeStatus    func(*testing.T, codes.Code, string, []proto.Message) statusIface
		fromError     func(err error) statusIface
		expectDetails []interface{} // nil elements signify errors
	}{
		{
			desc: "gogo status",
			makeStatus: func(t *testing.T, code codes.Code, msg string, details []proto.Message) statusIface {
				s, err := gogostatus.New(code, msg).WithDetails(details...)
				require.NoError(t, err)
				return s
			},
			fromError: func(err error) statusIface {
				return gogostatus.Convert(err)
			},
			expectDetails: []interface{}{
				nil, // Protobuf decode fails
				&errorspb.StringsPayload{Details: []string{"foo", "bar"}}, // gogoproto succeeds
				nil, // dummy decode fails
			},
		},
		{
			desc: "grpc status",
			makeStatus: func(t *testing.T, code codes.Code, msg string, details []proto.Message) statusIface {
				s := grpcstatus.New(code, msg)
				for _, detail := range details {
					var err error
					s, err = s.WithDetails(protoiface.MessageV1(detail))
					require.NoError(t, err)
				}
				return s
			},
			fromError: func(err error) statusIface {
				return grpcstatus.Convert(err)
			},
			expectDetails: []interface{}{
				// Protobuf succeeds
				func() interface{} {
					var st interface{} = grpcstatus.New(codes.Internal, "status").Proto()
					res := reflect.New(reflect.TypeOf(st).Elem()).Interface()
					copyPublicFields(res, st)
					return res
				}(),
				nil, // gogoproto decode fails
				nil, // dummy decode fails
			},
		},
	}
	for _, tc := range testcases {
		tc := tc
		t.Run(tc.desc, func(t *testing.T) {
			ctx := context.Background()

			// Create a Status, using statusIface to support gRPC and gogo variants.
			status := tc.makeStatus(t, codes.NotFound, "message", []proto.Message{
				grpcstatus.New(codes.Internal, "status").Proto(),          // standard Protobuf
				&errorspb.StringsPayload{Details: []string{"foo", "bar"}}, // GoGo Protobuf
				&dummyProto{value: "dummy"},                               // unregistered
			})
			require.Equal(t, codes.NotFound, status.Code())
			require.Equal(t, "message", status.Message())

			// Check the details. This varies by implementation, since different
			// Protobuf decoders are used -- gRPC Status can only decode
			// standard Protobufs, while gogo Status can only decode gogoproto
			// Protobufs.
			statusDetails := status.Details()
			require.Equal(t, len(tc.expectDetails), len(statusDetails), "detail mismatch")
			for i, expectDetail := range tc.expectDetails {
				if expectDetail == nil {
					require.Implements(t, (*error)(nil), statusDetails[i], "detail %v", i)
				} else {
					// gRPC populates a non-public field in the decoded struct.
					// This causes a direct deep equality comparison to fail.
					// To avert this, we compare just the public fields.
					actual := reflect.New(reflect.TypeOf(statusDetails[i]).Elem()).Interface()
					copyPublicFields(actual, statusDetails[i])
					require.Equal(t, expectDetail, actual, "detail %v", i)
				}
			}

			// Encode the error and check some fields.
			encodedError := errbase.EncodeError(ctx, status.Err())
			leaf := encodedError.GetLeaf()
			require.NotNil(t, leaf, "expected leaf")
			require.Equal(t, status.Message(), leaf.Message)
			require.Equal(t, []string{}, leaf.Details.ReportablePayload) // test this?
			require.NotNil(t, leaf.Details.FullDetails, "expected full details")
			require.Nil(t, encodedError.GetWrapper(), "unexpected wrapper")

			// Marshal and unmarshal the error, checking that
			// it equals the encoded error.
			marshaledError, err := encodedError.Marshal()
			require.NoError(t, err)
			require.NotEmpty(t, marshaledError)

			unmarshaledError := errorspb.EncodedError{}
			err = proto.Unmarshal(marshaledError, &unmarshaledError)
			require.NoError(t, err)
			require.True(t, proto.Equal(&encodedError, &unmarshaledError),
				"unmarshaled Protobuf differs")

			// Decode the error.
			decodedError := errbase.DecodeError(ctx, unmarshaledError)
			require.Equal(t, status.Err().Error(), decodedError.Error())

			// Convert the error into a status, and check its properties.
			decodedStatus := tc.fromError(decodedError)
			require.Equal(t, status.Code(), decodedStatus.Code())
			require.Equal(t, status.Message(), decodedStatus.Message())

			decodedDetails := decodedStatus.Details()
			require.Equal(t, len(tc.expectDetails), len(decodedDetails), "detail mismatch")
			for i, expectDetail := range tc.expectDetails {
				if expectDetail == nil {
					require.Implements(t, (*error)(nil), decodedDetails[i], "detail %v", i)
				} else {
					// gRPC populates a non-public field in the decoded struct.
					// This causes a direct deep equality comparison to fail.
					// To avert this, we compare just the public fields.
					actual := reflect.New(reflect.TypeOf(decodedDetails[i]).Elem()).Interface()
					copyPublicFields(actual, decodedDetails[i])
					require.Equal(t, expectDetail, actual, "detail %v", i)
				}
			}
		})
	}
}

func copyPublicFields(dst, src interface{}) {
	srcval := reflect.Indirect(reflect.ValueOf(src))
	dstval := reflect.Indirect(reflect.ValueOf(dst))
	typ := srcval.Type()
	for i := 0; i < srcval.NumField(); i++ {
		fname := typ.Field(i).Name
		if unicode.IsUpper(rune(fname[0])) && !strings.HasPrefix(fname, "XXX_") {
			dstval.Field(i).Set(srcval.Field(i))
		}
	}
}
