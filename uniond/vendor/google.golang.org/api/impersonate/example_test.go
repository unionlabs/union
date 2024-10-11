// Copyright 2021 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package impersonate_test

import (
	"context"
	"log"

	admin "google.golang.org/api/admin/directory/v1"
	"google.golang.org/api/impersonate"
	"google.golang.org/api/option"
	"google.golang.org/api/secretmanager/v1"
	"google.golang.org/api/transport"
)

func ExampleCredentialsTokenSource_serviceAccount() {
	ctx := context.Background()

	// Base credentials sourced from ADC or provided client options.
	ts, err := impersonate.CredentialsTokenSource(ctx, impersonate.CredentialsConfig{
		TargetPrincipal: "foo@project-id.iam.gserviceaccount.com",
		Scopes:          []string{"https://www.googleapis.com/auth/cloud-platform"},
		// Optionally supply delegates.
		Delegates: []string{"bar@project-id.iam.gserviceaccount.com"},
	})
	if err != nil {
		log.Fatal(err)
	}

	// Pass an impersonated credential to any function that takes client
	// options.
	client, err := secretmanager.NewService(ctx, option.WithTokenSource(ts))
	if err != nil {
		log.Fatal(err)
	}

	// Use your client that is authenticated with impersonated credentials to
	// make requests.
	client.Projects.Secrets.Get("...")
}

func ExampleCredentialsTokenSource_adminUser() {
	ctx := context.Background()

	// Base credentials sourced from ADC or provided client options.
	ts, err := impersonate.CredentialsTokenSource(ctx, impersonate.CredentialsConfig{
		TargetPrincipal: "foo@project-id.iam.gserviceaccount.com",
		Scopes:          []string{"https://www.googleapis.com/auth/cloud-platform"},
		// Optionally supply delegates.
		Delegates: []string{"bar@project-id.iam.gserviceaccount.com"},
		// Specify user to impersonate
		Subject: "admin@example.com",
	})
	if err != nil {
		log.Fatal(err)
	}

	// Pass an impersonated credential to any function that takes client
	// options.
	client, err := admin.NewService(ctx, option.WithTokenSource(ts))
	if err != nil {
		log.Fatal(err)
	}

	// Use your client that is authenticated with impersonated credentials to
	// make requests.
	client.Groups.Delete("...")
}

func ExampleIDTokenSource() {
	ctx := context.Background()

	// Base credentials sourced from ADC or provided client options.
	ts, err := impersonate.IDTokenSource(ctx, impersonate.IDTokenConfig{
		Audience:        "http://example.com/",
		TargetPrincipal: "foo@project-id.iam.gserviceaccount.com",
		IncludeEmail:    true,
		// Optionally supply delegates.
		Delegates: []string{"bar@project-id.iam.gserviceaccount.com"},
	})
	if err != nil {
		log.Fatal(err)
	}

	// Pass an impersonated credential to any function that takes client
	// options.
	client, _, err := transport.NewHTTPClient(ctx, option.WithTokenSource(ts))
	if err != nil {
		log.Fatal(err)
	}

	// Use your client that is authenticated with impersonated credentials to
	// make requests.
	client.Get("http://example.com/")
}
