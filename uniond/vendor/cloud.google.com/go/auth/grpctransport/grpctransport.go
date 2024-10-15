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

package grpctransport

import (
	"context"
	"errors"
	"fmt"
	"net/http"

	"cloud.google.com/go/auth"
	"cloud.google.com/go/auth/credentials"
	"cloud.google.com/go/auth/internal"
	"cloud.google.com/go/auth/internal/transport"
	"go.opencensus.io/plugin/ocgrpc"
	"google.golang.org/grpc"
	grpccreds "google.golang.org/grpc/credentials"
	grpcinsecure "google.golang.org/grpc/credentials/insecure"
)

const (
	// Check env to disable DirectPath traffic.
	disableDirectPathEnvVar = "GOOGLE_CLOUD_DISABLE_DIRECT_PATH"

	// Check env to decide if using google-c2p resolver for DirectPath traffic.
	enableDirectPathXdsEnvVar = "GOOGLE_CLOUD_ENABLE_DIRECT_PATH_XDS"

	quotaProjectHeaderKey = "X-Goog-User-Project"
)

var (
	// Set at init time by dial_socketopt.go. If nil, socketopt is not supported.
	timeoutDialerOption grpc.DialOption
)

// Options used to configure a [GRPCClientConnPool] from [Dial].
type Options struct {
	// DisableTelemetry disables default telemetry (OpenTelemetry). An example
	// reason to do so would be to bind custom telemetry that overrides the
	// defaults.
	DisableTelemetry bool
	// DisableAuthentication specifies that no authentication should be used. It
	// is suitable only for testing and for accessing public resources, like
	// public Google Cloud Storage buckets.
	DisableAuthentication bool
	// Endpoint overrides the default endpoint to be used for a service.
	Endpoint string
	// Metadata is extra gRPC metadata that will be appended to every outgoing
	// request.
	Metadata map[string]string
	// GRPCDialOpts are dial options that will be passed to `grpc.Dial` when
	// establishing a`grpc.Conn``
	GRPCDialOpts []grpc.DialOption
	// PoolSize is specifies how many connections to balance between when making
	// requests. If unset or less than 1, the value defaults to 1.
	PoolSize int
	// Credentials used to add Authorization metadata to all requests. If set
	// DetectOpts are ignored.
	Credentials *auth.Credentials
	// DetectOpts configures settings for detect Application Default
	// Credentials.
	DetectOpts *credentials.DetectOptions
	// UniverseDomain is the default service domain for a given Cloud universe.
	// The default value is "googleapis.com". This is the universe domain
	// configured for the client, which will be compared to the universe domain
	// that is separately configured for the credentials.
	UniverseDomain string

	// InternalOptions are NOT meant to be set directly by consumers of this
	// package, they should only be set by generated client code.
	InternalOptions *InternalOptions
}

// client returns the client a user set for the detect options or nil if one was
// not set.
func (o *Options) client() *http.Client {
	if o.DetectOpts != nil && o.DetectOpts.Client != nil {
		return o.DetectOpts.Client
	}
	return nil
}

func (o *Options) validate() error {
	if o == nil {
		return errors.New("grpctransport: opts required to be non-nil")
	}
	if o.InternalOptions != nil && o.InternalOptions.SkipValidation {
		return nil
	}
	hasCreds := o.Credentials != nil ||
		(o.DetectOpts != nil && len(o.DetectOpts.CredentialsJSON) > 0) ||
		(o.DetectOpts != nil && o.DetectOpts.CredentialsFile != "")
	if o.DisableAuthentication && hasCreds {
		return errors.New("grpctransport: DisableAuthentication is incompatible with options that set or detect credentials")
	}
	return nil
}

func (o *Options) resolveDetectOptions() *credentials.DetectOptions {
	io := o.InternalOptions
	// soft-clone these so we are not updating a ref the user holds and may reuse
	do := transport.CloneDetectOptions(o.DetectOpts)

	// If scoped JWTs are enabled user provided an aud, allow self-signed JWT.
	if (io != nil && io.EnableJWTWithScope) || do.Audience != "" {
		do.UseSelfSignedJWT = true
	}
	// Only default scopes if user did not also set an audience.
	if len(do.Scopes) == 0 && do.Audience == "" && io != nil && len(io.DefaultScopes) > 0 {
		do.Scopes = make([]string, len(io.DefaultScopes))
		copy(do.Scopes, io.DefaultScopes)
	}
	if len(do.Scopes) == 0 && do.Audience == "" && io != nil {
		do.Audience = o.InternalOptions.DefaultAudience
	}
	return do
}

// InternalOptions are only meant to be set by generated client code. These are
// not meant to be set directly by consumers of this package. Configuration in
// this type is considered EXPERIMENTAL and may be removed at any time in the
// future without warning.
type InternalOptions struct {
	// EnableNonDefaultSAForDirectPath overrides the default requirement for
	// using the default service account for DirectPath.
	EnableNonDefaultSAForDirectPath bool
	// EnableDirectPath overrides the default attempt to use DirectPath.
	EnableDirectPath bool
	// EnableDirectPathXds overrides the default DirectPath type. It is only
	// valid when DirectPath is enabled.
	EnableDirectPathXds bool
	// EnableJWTWithScope specifies if scope can be used with self-signed JWT.
	EnableJWTWithScope bool
	// DefaultAudience specifies a default audience to be used as the audience
	// field ("aud") for the JWT token authentication.
	DefaultAudience string
	// DefaultEndpointTemplate combined with UniverseDomain specifies
	// the default endpoint.
	DefaultEndpointTemplate string
	// DefaultMTLSEndpoint specifies the default mTLS endpoint.
	DefaultMTLSEndpoint string
	// DefaultScopes specifies the default OAuth2 scopes to be used for a
	// service.
	DefaultScopes []string
	// SkipValidation bypasses validation on Options. It should only be used
	// internally for clients that needs more control over their transport.
	SkipValidation bool
}

// Dial returns a GRPCClientConnPool that can be used to communicate with a
// Google cloud service, configured with the provided [Options]. It
// automatically appends Authorization metadata to all outgoing requests.
func Dial(ctx context.Context, secure bool, opts *Options) (GRPCClientConnPool, error) {
	if err := opts.validate(); err != nil {
		return nil, err
	}
	if opts.PoolSize <= 1 {
		conn, err := dial(ctx, secure, opts)
		if err != nil {
			return nil, err
		}
		return &singleConnPool{conn}, nil
	}
	pool := &roundRobinConnPool{}
	for i := 0; i < opts.PoolSize; i++ {
		conn, err := dial(ctx, secure, opts)
		if err != nil {
			// ignore close error, if any
			defer pool.Close()
			return nil, err
		}
		pool.conns = append(pool.conns, conn)
	}
	return pool, nil
}

// return a GRPCClientConnPool if pool == 1 or else a pool of of them if >1
func dial(ctx context.Context, secure bool, opts *Options) (*grpc.ClientConn, error) {
	tOpts := &transport.Options{
		Endpoint:       opts.Endpoint,
		Client:         opts.client(),
		UniverseDomain: opts.UniverseDomain,
	}
	if io := opts.InternalOptions; io != nil {
		tOpts.DefaultEndpointTemplate = io.DefaultEndpointTemplate
		tOpts.DefaultMTLSEndpoint = io.DefaultMTLSEndpoint
		tOpts.EnableDirectPath = io.EnableDirectPath
		tOpts.EnableDirectPathXds = io.EnableDirectPathXds
	}
	transportCreds, endpoint, err := transport.GetGRPCTransportCredsAndEndpoint(tOpts)
	if err != nil {
		return nil, err
	}

	if !secure {
		transportCreds = grpcinsecure.NewCredentials()
	}

	// Initialize gRPC dial options with transport-level security options.
	grpcOpts := []grpc.DialOption{
		grpc.WithTransportCredentials(transportCreds),
	}

	// Authentication can only be sent when communicating over a secure connection.
	if !opts.DisableAuthentication {
		metadata := opts.Metadata

		var creds *auth.Credentials
		if opts.Credentials != nil {
			creds = opts.Credentials
		} else {
			var err error
			creds, err = credentials.DetectDefault(opts.resolveDetectOptions())
			if err != nil {
				return nil, err
			}
		}

		qp, err := creds.QuotaProjectID(ctx)
		if err != nil {
			return nil, err
		}
		if qp != "" {
			if metadata == nil {
				metadata = make(map[string]string, 1)
			}
			metadata[quotaProjectHeaderKey] = qp
		}
		grpcOpts = append(grpcOpts,
			grpc.WithPerRPCCredentials(&grpcCredentialsProvider{
				creds:                creds,
				metadata:             metadata,
				clientUniverseDomain: opts.UniverseDomain,
			}),
		)

		// Attempt Direct Path
		grpcOpts, endpoint = configureDirectPath(grpcOpts, opts, endpoint, creds)
	}

	// Add tracing, but before the other options, so that clients can override the
	// gRPC stats handler.
	// This assumes that gRPC options are processed in order, left to right.
	grpcOpts = addOCStatsHandler(grpcOpts, opts)
	grpcOpts = append(grpcOpts, opts.GRPCDialOpts...)

	return grpc.DialContext(ctx, endpoint, grpcOpts...)
}

// grpcCredentialsProvider satisfies https://pkg.go.dev/google.golang.org/grpc/credentials#PerRPCCredentials.
type grpcCredentialsProvider struct {
	creds *auth.Credentials

	secure bool

	// Additional metadata attached as headers.
	metadata             map[string]string
	clientUniverseDomain string
}

// getClientUniverseDomain returns the default service domain for a given Cloud universe.
// The default value is "googleapis.com". This is the universe domain
// configured for the client, which will be compared to the universe domain
// that is separately configured for the credentials.
func (c *grpcCredentialsProvider) getClientUniverseDomain() string {
	if c.clientUniverseDomain == "" {
		return internal.DefaultUniverseDomain
	}
	return c.clientUniverseDomain
}

func (c *grpcCredentialsProvider) GetRequestMetadata(ctx context.Context, uri ...string) (map[string]string, error) {
	credentialsUniverseDomain, err := c.creds.UniverseDomain(ctx)
	if err != nil {
		return nil, err
	}
	if err := transport.ValidateUniverseDomain(c.getClientUniverseDomain(), credentialsUniverseDomain); err != nil {
		return nil, err
	}
	token, err := c.creds.Token(ctx)
	if err != nil {
		return nil, err
	}
	if c.secure {
		ri, _ := grpccreds.RequestInfoFromContext(ctx)
		if err = grpccreds.CheckSecurityLevel(ri.AuthInfo, grpccreds.PrivacyAndIntegrity); err != nil {
			return nil, fmt.Errorf("unable to transfer credentials PerRPCCredentials: %v", err)
		}
	}
	metadata := make(map[string]string, len(c.metadata)+1)
	setAuthMetadata(token, metadata)
	for k, v := range c.metadata {
		metadata[k] = v
	}
	return metadata, nil
}

// setAuthMetadata uses the provided token to set the Authorization metadata.
// If the token.Type is empty, the type is assumed to be Bearer.
func setAuthMetadata(token *auth.Token, m map[string]string) {
	typ := token.Type
	if typ == "" {
		typ = internal.TokenTypeBearer
	}
	m["authorization"] = typ + " " + token.Value
}

func (c *grpcCredentialsProvider) RequireTransportSecurity() bool {
	return c.secure
}

func addOCStatsHandler(dialOpts []grpc.DialOption, opts *Options) []grpc.DialOption {
	if opts.DisableTelemetry {
		return dialOpts
	}
	return append(dialOpts, grpc.WithStatsHandler(&ocgrpc.ClientHandler{}))
}
