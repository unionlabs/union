/*
 *
 * Copyright 2023 gRPC authors.
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

package encoding_test

import (
	"context"
	"errors"
	"fmt"
	"strings"
	"sync/atomic"
	"testing"
	"time"

	"github.com/google/go-cmp/cmp"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/encoding"
	"google.golang.org/grpc/encoding/proto"
	"google.golang.org/grpc/internal/grpctest"
	"google.golang.org/grpc/internal/grpcutil"
	"google.golang.org/grpc/internal/stubserver"
	"google.golang.org/grpc/mem"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/status"

	testgrpc "google.golang.org/grpc/interop/grpc_testing"
	testpb "google.golang.org/grpc/interop/grpc_testing"
)

const defaultTestTimeout = 10 * time.Second

type s struct {
	grpctest.Tester
}

func Test(t *testing.T) {
	grpctest.RunSubTests(t, s{})
}

type mockNamedCompressor struct {
	encoding.Compressor
}

func (mockNamedCompressor) Name() string {
	return "mock-compressor"
}

// Tests the case where a compressor with the same name is registered multiple
// times. Test verifies the following:
//   - the most recent registration is the one which is active
//   - grpcutil.RegisteredCompressorNames contains a single instance of the
//     previously registered compressor's name
func (s) TestDuplicateCompressorRegister(t *testing.T) {
	encoding.RegisterCompressor(&mockNamedCompressor{})

	// Register another instance of the same compressor.
	mc := &mockNamedCompressor{}
	encoding.RegisterCompressor(mc)
	if got := encoding.GetCompressor("mock-compressor"); got != mc {
		t.Fatalf("Unexpected compressor, got: %+v, want:%+v", got, mc)
	}

	wantNames := []string{"mock-compressor"}
	if !cmp.Equal(wantNames, grpcutil.RegisteredCompressorNames) {
		t.Fatalf("Unexpected compressor names, got: %+v, want:%+v", grpcutil.RegisteredCompressorNames, wantNames)
	}
}

// errProtoCodec wraps the proto codec and delegates to it if it is configured
// to return a nil error. Else, it returns the configured error.
type errProtoCodec struct {
	name        string
	encodingErr error
	decodingErr error
}

func (c *errProtoCodec) Marshal(v any) (mem.BufferSlice, error) {
	if c.encodingErr != nil {
		return nil, c.encodingErr
	}
	return encoding.GetCodecV2(proto.Name).Marshal(v)
}

func (c *errProtoCodec) Unmarshal(data mem.BufferSlice, v any) error {
	if c.decodingErr != nil {
		return c.decodingErr
	}
	return encoding.GetCodecV2(proto.Name).Unmarshal(data, v)
}

func (c *errProtoCodec) Name() string {
	return c.name
}

// Tests the case where encoding fails on the server. Verifies that there is
// no panic and that the encoding error is propagated to the client.
func (s) TestEncodeDoesntPanicOnServer(t *testing.T) {
	grpctest.TLogger.ExpectError("grpc: server failed to encode response")

	// Create an codec that errors when encoding messages.
	encodingErr := errors.New("encoding failed")
	ec := &errProtoCodec{name: t.Name(), encodingErr: encodingErr}

	// Start a server with the above codec.
	backend := stubserver.StartTestService(t, nil, grpc.ForceServerCodecV2(ec))
	defer backend.Stop()

	// Create a channel to the above server.
	cc, err := grpc.NewClient(backend.Address, grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("Failed to dial test backend at %q: %v", backend.Address, err)
	}
	defer cc.Close()

	// Make an RPC and expect it to fail. Since we do not specify any codec
	// here, the proto codec will get automatically used.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	client := testgrpc.NewTestServiceClient(cc)
	_, err = client.EmptyCall(ctx, &testpb.Empty{})
	if err == nil || !strings.Contains(err.Error(), encodingErr.Error()) {
		t.Fatalf("RPC failed with error: %v, want: %v", err, encodingErr)
	}

	// Configure the codec on the server to not return errors anymore and expect
	// the RPC to succeed.
	ec.encodingErr = nil
	if _, err := client.EmptyCall(ctx, &testpb.Empty{}); err != nil {
		t.Fatalf("RPC failed with error: %v", err)
	}
}

// Tests the case where decoding fails on the server. Verifies that there is
// no panic and that the decoding error is propagated to the client.
func (s) TestDecodeDoesntPanicOnServer(t *testing.T) {
	// Create an codec that errors when decoding messages.
	decodingErr := errors.New("decoding failed")
	ec := &errProtoCodec{name: t.Name(), decodingErr: decodingErr}

	// Start a server with the above codec.
	backend := stubserver.StartTestService(t, nil, grpc.ForceServerCodecV2(ec))
	defer backend.Stop()

	// Create a channel to the above server. Since we do not specify any codec
	// here, the proto codec will get automatically used.
	cc, err := grpc.NewClient(backend.Address, grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("Failed to dial test backend at %q: %v", backend.Address, err)
	}
	defer cc.Close()

	// Make an RPC and expect it to fail. Since we do not specify any codec
	// here, the proto codec will get automatically used.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	client := testgrpc.NewTestServiceClient(cc)
	_, err = client.EmptyCall(ctx, &testpb.Empty{})
	if err == nil || !strings.Contains(err.Error(), decodingErr.Error()) || !strings.Contains(err.Error(), "grpc: error unmarshalling request") {
		t.Fatalf("RPC failed with error: %v, want: %v", err, decodingErr)
	}

	// Configure the codec on the server to not return errors anymore and expect
	// the RPC to succeed.
	ec.decodingErr = nil
	if _, err := client.EmptyCall(ctx, &testpb.Empty{}); err != nil {
		t.Fatalf("RPC failed with error: %v", err)
	}
}

// Tests the case where encoding fails on the client . Verifies that there is
// no panic and that the encoding error is propagated to the RPC caller.
func (s) TestEncodeDoesntPanicOnClient(t *testing.T) {
	// Start a server and since we do not specify any codec here, the proto
	// codec will get automatically used.
	backend := stubserver.StartTestService(t, nil)
	defer backend.Stop()

	// Create an codec that errors when encoding messages.
	encodingErr := errors.New("encoding failed")
	ec := &errProtoCodec{name: t.Name(), encodingErr: encodingErr}

	// Create a channel to the above server.
	cc, err := grpc.NewClient(backend.Address, grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("Failed to dial test backend at %q: %v", backend.Address, err)
	}
	defer cc.Close()

	// Make an RPC with the erroring codec and expect it to fail.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	client := testgrpc.NewTestServiceClient(cc)
	_, err = client.EmptyCall(ctx, &testpb.Empty{}, grpc.ForceCodecV2(ec))
	if err == nil || !strings.Contains(err.Error(), encodingErr.Error()) {
		t.Fatalf("RPC failed with error: %v, want: %v", err, encodingErr)
	}

	// Configure the codec on the client to not return errors anymore and expect
	// the RPC to succeed.
	ec.encodingErr = nil
	if _, err := client.EmptyCall(ctx, &testpb.Empty{}, grpc.ForceCodecV2(ec)); err != nil {
		t.Fatalf("RPC failed with error: %v", err)
	}
}

// Tests the case where decoding fails on the server. Verifies that there is
// no panic and that the decoding error is propagated to the RPC caller.
func (s) TestDecodeDoesntPanicOnClient(t *testing.T) {
	// Start a server and since we do not specify any codec here, the proto
	// codec will get automatically used.
	backend := stubserver.StartTestService(t, nil)
	defer backend.Stop()

	// Create an codec that errors when decoding messages.
	decodingErr := errors.New("decoding failed")
	ec := &errProtoCodec{name: t.Name(), decodingErr: decodingErr}

	// Create a channel to the above server.
	cc, err := grpc.NewClient(backend.Address, grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("Failed to dial test backend at %q: %v", backend.Address, err)
	}
	defer cc.Close()

	// Make an RPC with the erroring codec and expect it to fail.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	client := testgrpc.NewTestServiceClient(cc)
	_, err = client.EmptyCall(ctx, &testpb.Empty{}, grpc.ForceCodecV2(ec))
	if err == nil || !strings.Contains(err.Error(), decodingErr.Error()) {
		t.Fatalf("RPC failed with error: %v, want: %v", err, decodingErr)
	}

	// Configure the codec on the client to not return errors anymore and expect
	// the RPC to succeed.
	ec.decodingErr = nil
	if _, err := client.EmptyCall(ctx, &testpb.Empty{}, grpc.ForceCodecV2(ec)); err != nil {
		t.Fatalf("RPC failed with error: %v", err)
	}
}

// countingProtoCodec wraps the proto codec and counts the number of times
// Marshal and Unmarshal are called.
type countingProtoCodec struct {
	name string

	// The following fields are accessed atomically.
	marshalCount   int32
	unmarshalCount int32
}

func (p *countingProtoCodec) Marshal(v any) (mem.BufferSlice, error) {
	atomic.AddInt32(&p.marshalCount, 1)
	return encoding.GetCodecV2(proto.Name).Marshal(v)
}

func (p *countingProtoCodec) Unmarshal(data mem.BufferSlice, v any) error {
	atomic.AddInt32(&p.unmarshalCount, 1)
	return encoding.GetCodecV2(proto.Name).Unmarshal(data, v)
}

func (p *countingProtoCodec) Name() string {
	return p.name
}

// Tests the case where ForceServerCodec option is used on the server. Verifies
// that encoding and decoding happen once per RPC.
func (s) TestForceServerCodec(t *testing.T) {
	// Create an server with the counting proto codec.
	codec := &countingProtoCodec{name: t.Name()}
	backend := stubserver.StartTestService(t, nil, grpc.ForceServerCodecV2(codec))
	defer backend.Stop()

	// Create a channel to the above server.
	cc, err := grpc.NewClient(backend.Address, grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("Failed to dial test backend at %q: %v", backend.Address, err)
	}
	defer cc.Close()

	// Make an RPC and expect it to fail. Since we do not specify any codec
	// here, the proto codec will get automatically used.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	client := testgrpc.NewTestServiceClient(cc)
	if _, err := client.EmptyCall(ctx, &testpb.Empty{}); err != nil {
		t.Fatalf("ss.Client.EmptyCall(_, _) = _, %v; want _, nil", err)
	}

	unmarshalCount := atomic.LoadInt32(&codec.unmarshalCount)
	const wantUnmarshalCount = 1
	if unmarshalCount != wantUnmarshalCount {
		t.Fatalf("Unmarshal Count = %d; want %d", unmarshalCount, wantUnmarshalCount)
	}
	marshalCount := atomic.LoadInt32(&codec.marshalCount)
	const wantMarshalCount = 1
	if marshalCount != wantMarshalCount {
		t.Fatalf("MarshalCount = %d; want %d", marshalCount, wantMarshalCount)
	}
}

// renameProtoCodec wraps the proto codec and allows customizing the Name().
type renameProtoCodec struct {
	encoding.CodecV2
	name string
}

func (r *renameProtoCodec) Name() string { return r.name }

// TestForceCodecName confirms that the ForceCodec call option sets the subtype
// in the content-type header according to the Name() of the codec provided.
// Verifies that the name is converted to lowercase before transmitting.
func (s) TestForceCodecName(t *testing.T) {
	wantContentTypeCh := make(chan []string, 1)
	defer close(wantContentTypeCh)

	// Create a test service backend that pushes the received content-type on a
	// channel for the test to inspect.
	ss := &stubserver.StubServer{
		EmptyCallF: func(ctx context.Context, in *testpb.Empty) (*testpb.Empty, error) {
			md, ok := metadata.FromIncomingContext(ctx)
			if !ok {
				return nil, status.Errorf(codes.Internal, "no metadata in context")
			}
			if got, want := md["content-type"], <-wantContentTypeCh; !cmp.Equal(got, want) {
				return nil, status.Errorf(codes.Internal, "got content-type=%q; want [%q]", got, want)
			}
			return &testpb.Empty{}, nil
		},
	}
	// Since we don't specify a codec as a server option, it will end up
	// automatically using the proto codec.
	if err := ss.Start(nil); err != nil {
		t.Fatalf("Error starting endpoint server: %v", err)
	}
	defer ss.Stop()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	// Force the use of the custom codec on the client with the ForceCodec call
	// option. Confirm the name is converted to lowercase before transmitting.
	codec := &renameProtoCodec{CodecV2: encoding.GetCodecV2(proto.Name), name: t.Name()}
	wantContentTypeCh <- []string{fmt.Sprintf("application/grpc+%s", strings.ToLower(t.Name()))}
	if _, err := ss.Client.EmptyCall(ctx, &testpb.Empty{}, grpc.ForceCodecV2(codec)); err != nil {
		t.Fatalf("ss.Client.EmptyCall(_, _) = _, %v; want _, nil", err)
	}
}
