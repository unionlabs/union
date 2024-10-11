/*
*
* Copyright 2022 gRPC authors.
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
package test

import (
	"context"
	"fmt"
	"net"
	"testing"

	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/internal/transport"
	"google.golang.org/grpc/status"

	testgrpc "google.golang.org/grpc/interop/grpc_testing"
)

func (s) TestHTTPHeaderFrameErrorHandlingHTTPMode(t *testing.T) {
	type test struct {
		name    string
		header  []string
		errCode codes.Code
	}

	var tests []test

	// Non-gRPC content-type fallback path.
	for httpCode := range transport.HTTPStatusConvTab {
		tests = append(tests, test{
			name: fmt.Sprintf("Non-gRPC content-type fallback path with httpCode: %v", httpCode),
			header: []string{
				":status", fmt.Sprintf("%d", httpCode),
				"content-type", "text/html", // non-gRPC content type to switch to HTTP mode.
				"grpc-status", "1", // Make up a gRPC status error
				"grpc-status-details-bin", "???", // Make up a gRPC field parsing error
			},
			errCode: transport.HTTPStatusConvTab[int(httpCode)],
		})
	}

	// Missing content-type fallback path.
	for httpCode := range transport.HTTPStatusConvTab {
		tests = append(tests, test{
			name: fmt.Sprintf("Missing content-type fallback path with httpCode: %v", httpCode),
			header: []string{
				":status", fmt.Sprintf("%d", httpCode),
				// Omitting content type to switch to HTTP mode.
				"grpc-status", "1", // Make up a gRPC status error
				"grpc-status-details-bin", "???", // Make up a gRPC field parsing error
			},
			errCode: transport.HTTPStatusConvTab[int(httpCode)],
		})
	}

	// Malformed HTTP status when fallback.
	tests = append(tests, test{
		name: "Malformed HTTP status when fallback",
		header: []string{
			":status", "abc",
			// Omitting content type to switch to HTTP mode.
			"grpc-status", "1", // Make up a gRPC status error
			"grpc-status-details-bin", "???", // Make up a gRPC field parsing error
		},
		errCode: codes.Internal,
	})

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			serverAddr, cleanup, err := startServer(t, test.header)
			if err != nil {
				t.Fatal(err)
			}
			defer cleanup()
			if err := doHTTPHeaderTest(serverAddr, test.errCode); err != nil {
				t.Error(err)
			}
		})
	}
}

// Testing erroneous ResponseHeader or Trailers-only (delivered in the first HEADERS frame).
func (s) TestHTTPHeaderFrameErrorHandlingInitialHeader(t *testing.T) {
	for _, test := range []struct {
		name    string
		header  []string
		errCode codes.Code
	}{
		{
			name: "missing gRPC status",
			header: []string{
				":status", "403",
				"content-type", "application/grpc",
			},
			errCode: codes.PermissionDenied,
		},
		{
			name: "malformed grpc-status",
			header: []string{
				":status", "502",
				"content-type", "application/grpc",
				"grpc-status", "abc",
			},
			errCode: codes.Internal,
		},
		{
			name: "Malformed grpc-tags-bin field",
			header: []string{
				":status", "502",
				"content-type", "application/grpc",
				"grpc-status", "0",
				"grpc-tags-bin", "???",
			},
			errCode: codes.Unavailable,
		},
		{
			name: "gRPC status error",
			header: []string{
				":status", "502",
				"content-type", "application/grpc",
				"grpc-status", "3",
			},
			errCode: codes.Unavailable,
		},
	} {
		t.Run(test.name, func(t *testing.T) {
			serverAddr, cleanup, err := startServer(t, test.header)
			if err != nil {
				t.Fatal(err)
			}
			defer cleanup()
			if err := doHTTPHeaderTest(serverAddr, test.errCode); err != nil {
				t.Error(err)
			}
		})
	}
}

// Testing non-Trailers-only Trailers (delivered in second HEADERS frame)
func (s) TestHTTPHeaderFrameErrorHandlingNormalTrailer(t *testing.T) {
	tests := []struct {
		name           string
		responseHeader []string
		trailer        []string
		errCode        codes.Code
	}{
		{
			name: "trailer missing grpc-status",
			responseHeader: []string{
				":status", "200",
				"content-type", "application/grpc",
			},
			trailer: []string{
				// trailer missing grpc-status
				":status", "502",
			},
			errCode: codes.Unavailable,
		},
		{
			name: "malformed grpc-status-details-bin field with status 404",
			responseHeader: []string{
				":status", "404",
				"content-type", "application/grpc",
			},
			trailer: []string{
				// malformed grpc-status-details-bin field
				"grpc-status", "0",
				"grpc-status-details-bin", "????",
			},
			errCode: codes.Unimplemented,
		},
		{
			name: "malformed grpc-status-details-bin field with status 200",
			responseHeader: []string{
				":status", "200",
				"content-type", "application/grpc",
			},
			trailer: []string{
				// malformed grpc-status-details-bin field
				"grpc-status", "0",
				"grpc-status-details-bin", "????",
			},
			errCode: codes.Internal,
		},
	}
	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			serverAddr, cleanup, err := startServer(t, test.responseHeader, test.trailer)
			if err != nil {
				t.Fatal(err)
			}
			defer cleanup()
			if err := doHTTPHeaderTest(serverAddr, test.errCode); err != nil {
				t.Error(err)
			}
		})

	}
}

func (s) TestHTTPHeaderFrameErrorHandlingMoreThanTwoHeaders(t *testing.T) {
	header := []string{
		":status", "200",
		"content-type", "application/grpc",
	}
	serverAddr, cleanup, err := startServer(t, header, header, header)
	if err != nil {
		t.Fatal(err)
	}
	defer cleanup()
	if err := doHTTPHeaderTest(serverAddr, codes.Internal); err != nil {
		t.Fatal(err)
	}
}

func startServer(t *testing.T, headerFields ...[]string) (serverAddr string, cleanup func(), err error) {
	t.Helper()

	lis, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		return "", nil, fmt.Errorf("listening on %q: %v", "localhost:0", err)
	}
	server := &httpServer{responses: []httpServerResponse{{trailers: headerFields}}}
	server.start(t, lis)
	return lis.Addr().String(), func() { lis.Close() }, nil
}

func doHTTPHeaderTest(lisAddr string, errCode codes.Code) error {
	cc, err := grpc.NewClient(lisAddr, grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		return fmt.Errorf("NewClient(%q): %v", lisAddr, err)
	}
	defer cc.Close()
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	client := testgrpc.NewTestServiceClient(cc)
	stream, err := client.FullDuplexCall(ctx)
	if err != nil {
		return fmt.Errorf("creating FullDuplex stream: %v", err)
	}
	if _, err := stream.Recv(); err == nil || status.Code(err) != errCode {
		return fmt.Errorf("stream.Recv() = %v, want error code: %v", err, errCode)
	}
	return nil
}
