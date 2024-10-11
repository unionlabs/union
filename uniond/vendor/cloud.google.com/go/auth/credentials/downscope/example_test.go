// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package downscope_test

import (
	"context"
	"fmt"

	"cloud.google.com/go/auth/credentials"
	"cloud.google.com/go/auth/credentials/downscope"
)

func ExampleNewCredentials() {
	// This shows how to generate a downscoped token. This code would be run on
	// the token broker, which holds the root token used to generate the
	// downscoped token.
	ctx := context.Background()

	// Initializes an accessBoundary with one Rule which restricts the
	// downscoped token to only be able to access the bucket "foo" and only
	// grants it the permission "storage.objectViewer".
	accessBoundary := []downscope.AccessBoundaryRule{
		{
			AvailableResource:    "//storage.googleapis.com/projects/_/buckets/foo",
			AvailablePermissions: []string{"inRole:roles/storage.objectViewer"},
		},
	}

	// This Source can be initialized in multiple ways; the following example uses
	// Application Default Credentials.
	baseProvider, err := credentials.DetectDefault(&credentials.DetectOptions{
		Scopes: []string{"https://www.googleapis.com/auth/cloud-platform"},
	})
	creds, err := downscope.NewCredentials(&downscope.Options{Credentials: baseProvider, Rules: accessBoundary})
	if err != nil {
		fmt.Printf("failed to generate downscoped token provider: %v", err)
		return
	}

	tok, err := creds.Token(ctx)
	if err != nil {
		fmt.Printf("failed to generate token: %v", err)
		return
	}
	_ = tok
	// You can now pass tok to a token consumer however you wish, such as exposing
	// a REST API and sending it over HTTP.

	// You can instead use the token held in tp to make
	// Google Cloud Storage calls, as follows:
	// storageClient, err := storage.NewClient(ctx, option.WithTokenProvider(tp))
}
