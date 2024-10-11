// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// Package debugger is an auto-generated package for the
// Stackdriver Debugger API.
//
// Examines the call stack and variables of a running application without
// stopping or slowing it down.
//
// # General documentation
//
// For information about setting deadlines, reusing contexts, and more
// please visit https://pkg.go.dev/cloud.google.com/go.
//
// # Example usage
//
// To get started with this package, create a client.
//
//	ctx := context.Background()
//	// This snippet has been automatically generated and should be regarded as a code template only.
//	// It will require modifications to work:
//	// - It may require correct/in-range values for request initialization.
//	// - It may require specifying regional endpoints when creating the service client as shown in:
//	//   https://pkg.go.dev/cloud.google.com/go#hdr-Client_Options
//	c, err := debugger.NewController2Client(ctx)
//	if err != nil {
//		// TODO: Handle error.
//	}
//	defer c.Close()
//
// The client will use your default application credentials. Clients should be reused instead of created as needed.
// The methods of Client are safe for concurrent use by multiple goroutines.
// The returned client must be Closed when it is done being used.
//
// # Using the Client
//
// The following is an example of making an API call with the newly created client.
//
//	ctx := context.Background()
//	// This snippet has been automatically generated and should be regarded as a code template only.
//	// It will require modifications to work:
//	// - It may require correct/in-range values for request initialization.
//	// - It may require specifying regional endpoints when creating the service client as shown in:
//	//   https://pkg.go.dev/cloud.google.com/go#hdr-Client_Options
//	c, err := debugger.NewController2Client(ctx)
//	if err != nil {
//		// TODO: Handle error.
//	}
//	defer c.Close()
//
//	req := &debuggerpb.RegisterDebuggeeRequest{
//		// TODO: Fill request struct fields.
//		// See https://pkg.go.dev/cloud.google.com/go/debugger/apiv2/debuggerpb#RegisterDebuggeeRequest.
//	}
//	resp, err := c.RegisterDebuggee(ctx, req)
//	if err != nil {
//		// TODO: Handle error.
//	}
//	// TODO: Use resp.
//	_ = resp
//
// # Use of Context
//
// The ctx passed to NewController2Client is used for authentication requests and
// for creating the underlying connection, but is not used for subsequent calls.
// Individual methods on the client use the ctx given to them.
//
// To close the open connection, use the Close() method.
package debugger // import "cloud.google.com/go/debugger/apiv2"

import (
	"context"
	"net/http"
	"os"
	"strconv"

	"google.golang.org/api/option"
	"google.golang.org/grpc/metadata"
)

// For more information on implementing a client constructor hook, see
// https://github.com/googleapis/google-cloud-go/wiki/Customizing-constructors.
type clientHookParams struct{}
type clientHook func(context.Context, clientHookParams) ([]option.ClientOption, error)

var versionClient string

func getVersionClient() string {
	if versionClient == "" {
		return "UNKNOWN"
	}
	return versionClient
}

func insertMetadata(ctx context.Context, mds ...metadata.MD) context.Context {
	out, _ := metadata.FromOutgoingContext(ctx)
	out = out.Copy()
	for _, md := range mds {
		for k, v := range md {
			out[k] = append(out[k], v...)
		}
	}
	return metadata.NewOutgoingContext(ctx, out)
}

func checkDisableDeadlines() (bool, error) {
	raw, ok := os.LookupEnv("GOOGLE_API_GO_EXPERIMENTAL_DISABLE_DEFAULT_DEADLINE")
	if !ok {
		return false, nil
	}

	b, err := strconv.ParseBool(raw)
	return b, err
}

// DefaultAuthScopes reports the default set of authentication scopes to use with this package.
func DefaultAuthScopes() []string {
	return []string{
		"https://www.googleapis.com/auth/cloud-platform",
		"https://www.googleapis.com/auth/cloud_debugger",
	}
}

// buildHeaders extracts metadata from the outgoing context, joins it with any other
// given metadata, and converts them into a http.Header.
func buildHeaders(ctx context.Context, mds ...metadata.MD) http.Header {
	if cmd, ok := metadata.FromOutgoingContext(ctx); ok {
		mds = append(mds, cmd)
	}
	md := metadata.Join(mds...)
	return http.Header(md)
}
