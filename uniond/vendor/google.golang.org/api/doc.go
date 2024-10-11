// Copyright 2019 Google LLC. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

// Package api is the root of the packages used to access Google Cloud
// Services. See https://godoc.org/google.golang.org/api for a full list of
// sub-packages.
//
// Within api there exist numerous clients which connect to Google APIs,
// and various utility packages.
//
// # Client Options
//
// All clients in sub-packages are configurable via client options. These
// options are described here: https://godoc.org/google.golang.org/api/option.
//
// # Authentication and Authorization
//
// All the clients in sub-packages support authentication via Google
// Application Default Credentials (see
// https://cloud.google.com/docs/authentication/production), or by providing a
// JSON key file for a Service Account. See the authentication examples in
// https://godoc.org/google.golang.org/api/transport for more details.
//
// # Versioning and Stability
//
// Due to the auto-generated nature of this collection of libraries, complete
// APIs or specific versions can appear or go away without notice. As a result,
// you should always locally vendor any API(s) that your code relies upon.
//
// Google APIs follow semver as specified by
// https://cloud.google.com/apis/design/versioning. The code generator and
// the code it produces - the libraries in the google.golang.org/api/...
// subpackages - are beta.
//
// Note that versioning and stability is strictly not communicated through Go
// modules. Go modules are used only for dependency management.
//
// # Integers
//
// Many parameters are specified using ints. However, underlying APIs might
// operate on a finer granularity, expecting int64, int32, uint64, or uint32,
// all of whom have different maximum values. Subsequently, specifying an int
// parameter in one of these clients may result in an error from the API
// because the value is too large.
//
// To see the exact type of int that the API expects, you can inspect the API's
// discovery doc. A global catalogue pointing to the discovery doc of APIs can
// be found at https://www.googleapis.com/discovery/v1/apis.
//
// # ForceSendFields
//
// This field can be found on all Request/Response structs in the generated
// clients. All of these types have the JSON `omitempty` field tag present on
// their fields. This means if a type is set to its default value it will not be
// marshalled. Sometimes you may actually want to send a default value, for
// instance sending an int of `0`. In this case you can override the `omitempty`
// feature by adding the field name to the `ForceSendFields` slice. See docs on
// any struct for more details. This may be used to include empty fields in
// Patch requests.
//
// # NullFields
//
// This field can be found on all Request/Response structs in the generated
// clients. It can be be used to send JSON null values for the listed fields.
// By default, fields with empty values are omitted from API requests because of
// the presence of the `omitempty` field tag on all fields. However, any field
// with an empty value appearing in NullFields will be sent to the server as
// null. It is an error if a field in this list has a non-empty value. This may
// be used to include null fields in Patch requests.
//
// # Inspecting errors
//
// An error returned by a client's Do method may be cast to a *googleapi.Error
// or unwrapped to an *apierror.APIError.
//
// The https://pkg.go.dev/google.golang.org/api/googleapi#Error type is useful
// for getting the HTTP status code:
//
//	if _, err := svc.FooCall().Do(); err != nil {
//		if gErr, ok := err.(*googleapi.Error); ok {
//			fmt.Println("Status code: %v", gErr.Code)
//		}
//	}
//
// The https://pkg.go.dev/github.com/googleapis/gax-go/v2/apierror#APIError type
// is useful for inspecting structured details of the underlying API response,
// such as the reason for the error and the error domain, which is typically the
// registered service name of the tool or product that generated the error:
//
//	if _, err := svc.FooCall().Do(); err != nil {
//		var aErr *apierror.APIError
//		if ok := errors.As(err, &aErr); ok {
//			fmt.Println("Reason: %s", aErr.Reason())
//			fmt.Println("Domain: %s", aErr.Domain())
//		}
//	}
//
// # Polling Operations
//
// If an API call returns an Operation, that means it could take some time to
// complete the work initiated by the API call. Applications that are interested
// in the end result of the operation they initiated should wait until the
// Operation.Done field indicates it is finished. To do this, use the service's
// Operation client, and a loop, like so:
//
//	  import (
//			gax "github.com/googleapis/gax-go/v2"
//	  )
//
//	  // existing application code...
//
//	  // API call that returns an Operation.
//		 op, err := myApiClient.CalculateFoo().Do()
//		 if err != nil {
//			// handle err
//		 }
//
//		 operationsService = myapi.NewOperationsService(myApiClient)
//	  pollingBackoff := gax.Backoff{
//		    Initial:    time.Second,
//		    Max:        time.Minute, // Max time between polling attempts.
//		    Multiplier: 2,
//	  }
//		 for {
//			if op.Done {
//				break
//			}
//			// not done, sleep with backoff, then poll again
//			if err := gax.Sleep(ctx, pollingBackoff.Pause()); err != nil {
//				// handle error
//			}
//			op, err := operationsService.Get(op.Name).Do()
//			if err != nil {
//				// handle error
//			}
//		 }
//
//		 if op.Error != nil {
//			// handle operation err
//		 }
//
//		 // Do something with the response
//		 fmt.Println(op.Response)
package api

import (
	// Force dependency on main module to ensure it is unambiguous during
	// module resolution.
	// See: https://github.com/googleapis/google-api-go-client/issues/2559.
	_ "cloud.google.com/go/civil"
	_ "google.golang.org/genproto/googleapis/type/datetime"
)
