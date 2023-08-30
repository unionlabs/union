// Copyright 2015 Google LLC.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

// Package grpc supports network connections to GRPC servers.
// This package is not intended for use by end developers. Use the
// google.golang.org/api/option package to configure API clients.
package grpc

import (
	"context"
	"errors"
	"log"
	"net"
	"os"
	"strings"

	"cloud.google.com/go/compute/metadata"
	"go.opencensus.io/plugin/ocgrpc"
	"golang.org/x/oauth2"
	"google.golang.org/api/internal"
	"google.golang.org/api/option"
	"google.golang.org/grpc"
	grpcgoogle "google.golang.org/grpc/credentials/google"
	grpcinsecure "google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/credentials/oauth"

	// Install grpclb, which is required for direct path.
	_ "google.golang.org/grpc/balancer/grpclb"
)

// Check env to disable DirectPath traffic.
const disableDirectPath = "GOOGLE_CLOUD_DISABLE_DIRECT_PATH"

// Check env to decide if using google-c2p resolver for DirectPath traffic.
const enableDirectPathXds = "GOOGLE_CLOUD_ENABLE_DIRECT_PATH_XDS"

// Set at init time by dial_appengine.go. If nil, we're not on App Engine.
var appengineDialerHook func(context.Context) grpc.DialOption

// Set at init time by dial_socketopt.go. If nil, socketopt is not supported.
var timeoutDialerOption grpc.DialOption

// Dial returns a GRPC connection for use communicating with a Google cloud
// service, configured with the given ClientOptions.
func Dial(ctx context.Context, opts ...option.ClientOption) (*grpc.ClientConn, error) {
	o, err := processAndValidateOpts(opts)
	if err != nil {
		return nil, err
	}
	if o.GRPCConnPool != nil {
		return o.GRPCConnPool.Conn(), nil
	}
	// NOTE(cbro): We removed support for option.WithGRPCConnPool (GRPCConnPoolSize)
	// on 2020-02-12 because RoundRobin and WithBalancer are deprecated and we need to remove usages of it.
	//
	// Connection pooling is only done via DialPool.
	return dial(ctx, false, o)
}

// DialInsecure returns an insecure GRPC connection for use communicating
// with fake or mock Google cloud service implementations, such as emulators.
// The connection is configured with the given ClientOptions.
func DialInsecure(ctx context.Context, opts ...option.ClientOption) (*grpc.ClientConn, error) {
	o, err := processAndValidateOpts(opts)
	if err != nil {
		return nil, err
	}
	return dial(ctx, true, o)
}

// DialPool returns a pool of GRPC connections for the given service.
// This differs from the connection pooling implementation used by Dial, which uses a custom GRPC load balancer.
// DialPool should be used instead of Dial when a pool is used by default or a different custom GRPC load balancer is needed.
// The context and options are shared between each Conn in the pool.
// The pool size is configured using the WithGRPCConnectionPool option.
//
// This API is subject to change as we further refine requirements. It will go away if gRPC stubs accept an interface instead of the concrete ClientConn type. See https://github.com/grpc/grpc-go/issues/1287.
func DialPool(ctx context.Context, opts ...option.ClientOption) (ConnPool, error) {
	o, err := processAndValidateOpts(opts)
	if err != nil {
		return nil, err
	}
	if o.GRPCConnPool != nil {
		return o.GRPCConnPool, nil
	}
	poolSize := o.GRPCConnPoolSize
	if o.GRPCConn != nil {
		// WithGRPCConn is technically incompatible with WithGRPCConnectionPool.
		// Always assume pool size is 1 when a grpc.ClientConn is explicitly used.
		poolSize = 1
	}
	o.GRPCConnPoolSize = 0 // we don't *need* to set this to zero, but it's safe to.

	if poolSize == 0 || poolSize == 1 {
		// Fast path for common case for a connection pool with a single connection.
		conn, err := dial(ctx, false, o)
		if err != nil {
			return nil, err
		}
		return &singleConnPool{conn}, nil
	}

	pool := &roundRobinConnPool{}
	for i := 0; i < poolSize; i++ {
		conn, err := dial(ctx, false, o)
		if err != nil {
			defer pool.Close() // NOTE: error from Close is ignored.
			return nil, err
		}
		pool.conns = append(pool.conns, conn)
	}
	return pool, nil
}

func dial(ctx context.Context, insecure bool, o *internal.DialSettings) (*grpc.ClientConn, error) {
	if o.HTTPClient != nil {
		return nil, errors.New("unsupported HTTP client specified")
	}
	if o.GRPCConn != nil {
		return o.GRPCConn, nil
	}
	transportCreds, endpoint, err := internal.GetGRPCTransportConfigAndEndpoint(o)
	if err != nil {
		return nil, err
	}

	if insecure {
		transportCreds = grpcinsecure.NewCredentials()
	}

	// Initialize gRPC dial options with transport-level security options.
	grpcOpts := []grpc.DialOption{
		grpc.WithTransportCredentials(transportCreds),
	}

	// Authentication can only be sent when communicating over a secure connection.
	//
	// TODO: Should we be more lenient in the future and allow sending credentials
	// when dialing an insecure connection?
	if !o.NoAuth && !insecure {
		if o.APIKey != "" {
			log.Print("API keys are not supported for gRPC APIs. Remove the WithAPIKey option from your client-creating call.")
		}
		creds, err := internal.Creds(ctx, o)
		if err != nil {
			return nil, err
		}

		grpcOpts = append(grpcOpts,
			grpc.WithPerRPCCredentials(grpcTokenSource{
				TokenSource:   oauth.TokenSource{creds.TokenSource},
				quotaProject:  internal.GetQuotaProject(creds, o.QuotaProject),
				requestReason: o.RequestReason,
			}),
		)

		// Attempt Direct Path:
		if isDirectPathEnabled(endpoint, o) && isTokenSourceDirectPathCompatible(creds.TokenSource, o) && metadata.OnGCE() {
			// Overwrite all of the previously specific DialOptions, DirectPath uses its own set of credentials and certificates.
			grpcOpts = []grpc.DialOption{
				grpc.WithCredentialsBundle(grpcgoogle.NewDefaultCredentialsWithOptions(grpcgoogle.DefaultCredentialsOptions{oauth.TokenSource{creds.TokenSource}}))}
			if timeoutDialerOption != nil {
				grpcOpts = append(grpcOpts, timeoutDialerOption)
			}
			// Check if google-c2p resolver is enabled for DirectPath
			if isDirectPathXdsUsed(o) {
				// google-c2p resolver target must not have a port number
				if addr, _, err := net.SplitHostPort(endpoint); err == nil {
					endpoint = "google-c2p:///" + addr
				} else {
					endpoint = "google-c2p:///" + endpoint
				}
			} else {
				if !strings.HasPrefix(endpoint, "dns:///") {
					endpoint = "dns:///" + endpoint
				}
				grpcOpts = append(grpcOpts,
					// For now all DirectPath go clients will be using the following lb config, but in future
					// when different services need different configs, then we should change this to a
					// per-service config.
					grpc.WithDisableServiceConfig(),
					grpc.WithDefaultServiceConfig(`{"loadBalancingConfig":[{"grpclb":{"childPolicy":[{"pick_first":{}}]}}]}`))
			}
			// TODO(cbro): add support for system parameters (quota project, request reason) via chained interceptor.
		}
	}

	if appengineDialerHook != nil {
		// Use the Socket API on App Engine.
		// appengine dialer will override socketopt dialer
		grpcOpts = append(grpcOpts, appengineDialerHook(ctx))
	}

	// Add tracing, but before the other options, so that clients can override the
	// gRPC stats handler.
	// This assumes that gRPC options are processed in order, left to right.
	grpcOpts = addOCStatsHandler(grpcOpts, o)
	grpcOpts = append(grpcOpts, o.GRPCDialOpts...)
	if o.UserAgent != "" {
		grpcOpts = append(grpcOpts, grpc.WithUserAgent(o.UserAgent))
	}

	return grpc.DialContext(ctx, endpoint, grpcOpts...)
}

func addOCStatsHandler(opts []grpc.DialOption, settings *internal.DialSettings) []grpc.DialOption {
	if settings.TelemetryDisabled {
		return opts
	}
	return append(opts, grpc.WithStatsHandler(&ocgrpc.ClientHandler{}))
}

// grpcTokenSource supplies PerRPCCredentials from an oauth.TokenSource.
type grpcTokenSource struct {
	oauth.TokenSource

	// Additional metadata attached as headers.
	quotaProject  string
	requestReason string
}

// GetRequestMetadata gets the request metadata as a map from a grpcTokenSource.
func (ts grpcTokenSource) GetRequestMetadata(ctx context.Context, uri ...string) (
	map[string]string, error) {
	metadata, err := ts.TokenSource.GetRequestMetadata(ctx, uri...)
	if err != nil {
		return nil, err
	}

	// Attach system parameter
	if ts.quotaProject != "" {
		metadata["X-goog-user-project"] = ts.quotaProject
	}
	if ts.requestReason != "" {
		metadata["X-goog-request-reason"] = ts.requestReason
	}
	return metadata, nil
}

func isDirectPathEnabled(endpoint string, o *internal.DialSettings) bool {
	if !o.EnableDirectPath {
		return false
	}
	if !checkDirectPathEndPoint(endpoint) {
		return false
	}
	if strings.EqualFold(os.Getenv(disableDirectPath), "true") {
		return false
	}
	return true
}

func isDirectPathXdsUsed(o *internal.DialSettings) bool {
	// Method 1: Enable DirectPath xDS by env;
	if strings.EqualFold(os.Getenv(enableDirectPathXds), "true") {
		return true
	}
	// Method 2: Enable DirectPath xDS by option;
	if o.EnableDirectPathXds {
		return true
	}
	return false

}

func isTokenSourceDirectPathCompatible(ts oauth2.TokenSource, o *internal.DialSettings) bool {
	if ts == nil {
		return false
	}
	tok, err := ts.Token()
	if err != nil {
		return false
	}
	if tok == nil {
		return false
	}
	if o.AllowNonDefaultServiceAccount {
		return true
	}
	if source, _ := tok.Extra("oauth2.google.tokenSource").(string); source != "compute-metadata" {
		return false
	}
	if acct, _ := tok.Extra("oauth2.google.serviceAccount").(string); acct != "default" {
		return false
	}
	return true
}

func checkDirectPathEndPoint(endpoint string) bool {
	// Only [dns:///]host[:port] is supported, not other schemes (e.g., "tcp://" or "unix://").
	// Also don't try direct path if the user has chosen an alternate name resolver
	// (i.e., via ":///" prefix).
	//
	// TODO(cbro): once gRPC has introspectible options, check the user hasn't
	// provided a custom dialer in gRPC options.
	if strings.Contains(endpoint, "://") && !strings.HasPrefix(endpoint, "dns:///") {
		return false
	}

	if endpoint == "" {
		return false
	}

	return true
}

func processAndValidateOpts(opts []option.ClientOption) (*internal.DialSettings, error) {
	var o internal.DialSettings
	for _, opt := range opts {
		opt.Apply(&o)
	}
	if err := o.Validate(); err != nil {
		return nil, err
	}

	return &o, nil
}

type connPoolOption struct{ ConnPool }

// WithConnPool returns a ClientOption that specifies the ConnPool
// connection to use as the basis of communications.
//
// This is only to be used by Google client libraries internally, for example
// when creating a longrunning API client that shares the same connection pool
// as a service client.
func WithConnPool(p ConnPool) option.ClientOption {
	return connPoolOption{p}
}

func (o connPoolOption) Apply(s *internal.DialSettings) {
	s.GRPCConnPool = o.ConnPool
}
