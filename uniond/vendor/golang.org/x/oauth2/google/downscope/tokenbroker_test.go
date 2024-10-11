// Copyright 2021 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package downscope_test

import (
	"context"
	"fmt"

	"golang.org/x/oauth2/google"

	"golang.org/x/oauth2"
	"golang.org/x/oauth2/google/downscope"
)

func ExampleNewTokenSource() {
	// This shows how to generate a downscoped token. This code would be run on the
	// token broker, which holds the root token used to generate the downscoped token.
	ctx := context.Background()
	// Initializes an accessBoundary with one Rule which restricts the downscoped
	// token to only be able to access the bucket "foo" and only grants it the
	// permission "storage.objectViewer".
	accessBoundary := []downscope.AccessBoundaryRule{
		{
			AvailableResource:    "//storage.googleapis.com/projects/_/buckets/foo",
			AvailablePermissions: []string{"inRole:roles/storage.objectViewer"},
		},
	}

	var rootSource oauth2.TokenSource
	// This Source can be initialized in multiple ways; the following example uses
	// Application Default Credentials.

	rootSource, err := google.DefaultTokenSource(ctx, "https://www.googleapis.com/auth/cloud-platform")

	dts, err := downscope.NewTokenSource(ctx, downscope.DownscopingConfig{RootSource: rootSource, Rules: accessBoundary})
	if err != nil {
		fmt.Printf("failed to generate downscoped token source: %v", err)
		return
	}

	tok, err := dts.Token()
	if err != nil {
		fmt.Printf("failed to generate token: %v", err)
		return
	}
	_ = tok
	// You can now pass tok to a token consumer however you wish, such as exposing
	// a REST API and sending it over HTTP.

	// You can instead use the token held in dts to make
	// Google Cloud Storage calls, as follows:

	// storageClient, err := storage.NewClient(ctx, option.WithTokenSource(dts))

}
