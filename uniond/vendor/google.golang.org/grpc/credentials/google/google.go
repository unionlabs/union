/*
 *
 * Copyright 2018 gRPC authors.
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

// Package google defines credentials for google cloud services.
package google

import (
	"context"
	"fmt"
	"time"

	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/credentials/alts"
	"google.golang.org/grpc/credentials/oauth"
	"google.golang.org/grpc/grpclog"
	"google.golang.org/grpc/internal"
)

const tokenRequestTimeout = 30 * time.Second

var logger = grpclog.Component("credentials")

// DefaultCredentialsOptions constructs options to build DefaultCredentials.
type DefaultCredentialsOptions struct {
	// PerRPCCreds is a per RPC credentials that is passed to a bundle.
	PerRPCCreds credentials.PerRPCCredentials
}

// NewDefaultCredentialsWithOptions returns a credentials bundle that is
// configured to work with google services.
//
// This API is experimental.
func NewDefaultCredentialsWithOptions(opts DefaultCredentialsOptions) credentials.Bundle {
	if opts.PerRPCCreds == nil {
		ctx, cancel := context.WithTimeout(context.Background(), tokenRequestTimeout)
		defer cancel()
		var err error
		opts.PerRPCCreds, err = newADC(ctx)
		if err != nil {
			logger.Warningf("NewDefaultCredentialsWithOptions: failed to create application oauth: %v", err)
		}
	}
	c := &creds{opts: opts}
	bundle, err := c.NewWithMode(internal.CredsBundleModeFallback)
	if err != nil {
		logger.Warningf("NewDefaultCredentialsWithOptions: failed to create new creds: %v", err)
	}
	return bundle
}

// NewDefaultCredentials returns a credentials bundle that is configured to work
// with google services.
//
// This API is experimental.
func NewDefaultCredentials() credentials.Bundle {
	return NewDefaultCredentialsWithOptions(DefaultCredentialsOptions{})
}

// NewComputeEngineCredentials returns a credentials bundle that is configured to work
// with google services. This API must only be used when running on GCE. Authentication configured
// by this API represents the GCE VM's default service account.
//
// This API is experimental.
func NewComputeEngineCredentials() credentials.Bundle {
	return NewDefaultCredentialsWithOptions(DefaultCredentialsOptions{
		PerRPCCreds: oauth.NewComputeEngine(),
	})
}

// creds implements credentials.Bundle.
type creds struct {
	opts DefaultCredentialsOptions

	// Supported modes are defined in internal/internal.go.
	mode string
	// The active transport credentials associated with this bundle.
	transportCreds credentials.TransportCredentials
	// The active per RPC credentials associated with this bundle.
	perRPCCreds credentials.PerRPCCredentials
}

func (c *creds) TransportCredentials() credentials.TransportCredentials {
	return c.transportCreds
}

func (c *creds) PerRPCCredentials() credentials.PerRPCCredentials {
	if c == nil {
		return nil
	}
	return c.perRPCCreds
}

var (
	newTLS = func() credentials.TransportCredentials {
		return credentials.NewTLS(nil)
	}
	newALTS = func() credentials.TransportCredentials {
		return alts.NewClientCreds(alts.DefaultClientOptions())
	}
	newADC = func(ctx context.Context) (credentials.PerRPCCredentials, error) {
		return oauth.NewApplicationDefault(ctx)
	}
)

// NewWithMode should make a copy of Bundle, and switch mode. Modifying the
// existing Bundle may cause races.
func (c *creds) NewWithMode(mode string) (credentials.Bundle, error) {
	newCreds := &creds{
		opts: c.opts,
		mode: mode,
	}

	// Create transport credentials.
	switch mode {
	case internal.CredsBundleModeFallback:
		newCreds.transportCreds = newClusterTransportCreds(newTLS(), newALTS())
	case internal.CredsBundleModeBackendFromBalancer, internal.CredsBundleModeBalancer:
		// Only the clients can use google default credentials, so we only need
		// to create new ALTS client creds here.
		newCreds.transportCreds = newALTS()
	default:
		return nil, fmt.Errorf("unsupported mode: %v", mode)
	}

	if mode == internal.CredsBundleModeFallback || mode == internal.CredsBundleModeBackendFromBalancer {
		newCreds.perRPCCreds = newCreds.opts.PerRPCCreds
	}

	return newCreds, nil
}
