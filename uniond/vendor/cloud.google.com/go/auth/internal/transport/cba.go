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

package transport

import (
	"context"
	"crypto/tls"
	"errors"
	"net"
	"net/http"
	"net/url"
	"os"
	"strconv"
	"strings"

	"cloud.google.com/go/auth/internal"
	"cloud.google.com/go/auth/internal/transport/cert"
	"github.com/google/s2a-go"
	"github.com/google/s2a-go/fallback"
	"google.golang.org/grpc/credentials"
)

const (
	mTLSModeAlways = "always"
	mTLSModeNever  = "never"
	mTLSModeAuto   = "auto"

	// Experimental: if true, the code will try MTLS with S2A as the default for transport security. Default value is false.
	googleAPIUseS2AEnv     = "EXPERIMENTAL_GOOGLE_API_USE_S2A"
	googleAPIUseCertSource = "GOOGLE_API_USE_CLIENT_CERTIFICATE"
	googleAPIUseMTLS       = "GOOGLE_API_USE_MTLS_ENDPOINT"
	googleAPIUseMTLSOld    = "GOOGLE_API_USE_MTLS"

	universeDomainPlaceholder = "UNIVERSE_DOMAIN"
)

var (
	mdsMTLSAutoConfigSource     mtlsConfigSource
	errUniverseNotSupportedMTLS = errors.New("mTLS is not supported in any universe other than googleapis.com")
)

// Options is a struct that is duplicated information from the individual
// transport packages in order to avoid cyclic deps. It correlates 1:1 with
// fields on httptransport.Options and grpctransport.Options.
type Options struct {
	Endpoint                string
	DefaultMTLSEndpoint     string
	DefaultEndpointTemplate string
	ClientCertProvider      cert.Provider
	Client                  *http.Client
	UniverseDomain          string
	EnableDirectPath        bool
	EnableDirectPathXds     bool
}

// getUniverseDomain returns the default service domain for a given Cloud
// universe.
func (o *Options) getUniverseDomain() string {
	if o.UniverseDomain == "" {
		return internal.DefaultUniverseDomain
	}
	return o.UniverseDomain
}

// isUniverseDomainGDU returns true if the universe domain is the default Google
// universe.
func (o *Options) isUniverseDomainGDU() bool {
	return o.getUniverseDomain() == internal.DefaultUniverseDomain
}

// defaultEndpoint returns the DefaultEndpointTemplate merged with the
// universe domain if the DefaultEndpointTemplate is set, otherwise returns an
// empty string.
func (o *Options) defaultEndpoint() string {
	if o.DefaultEndpointTemplate == "" {
		return ""
	}
	return strings.Replace(o.DefaultEndpointTemplate, universeDomainPlaceholder, o.getUniverseDomain(), 1)
}

// mergedEndpoint merges a user-provided Endpoint of format host[:port] with the
// default endpoint.
func (o *Options) mergedEndpoint() (string, error) {
	defaultEndpoint := o.defaultEndpoint()
	u, err := url.Parse(fixScheme(defaultEndpoint))
	if err != nil {
		return "", err
	}
	return strings.Replace(defaultEndpoint, u.Host, o.Endpoint, 1), nil
}

func fixScheme(baseURL string) string {
	if !strings.Contains(baseURL, "://") {
		baseURL = "https://" + baseURL
	}
	return baseURL
}

// GetGRPCTransportCredsAndEndpoint returns an instance of
// [google.golang.org/grpc/credentials.TransportCredentials], and the
// corresponding endpoint to use for GRPC client.
func GetGRPCTransportCredsAndEndpoint(opts *Options) (credentials.TransportCredentials, string, error) {
	config, err := getTransportConfig(opts)
	if err != nil {
		return nil, "", err
	}

	defaultTransportCreds := credentials.NewTLS(&tls.Config{
		GetClientCertificate: config.clientCertSource,
	})
	if config.s2aAddress == "" {
		return defaultTransportCreds, config.endpoint, nil
	}

	var fallbackOpts *s2a.FallbackOptions
	// In case of S2A failure, fall back to the endpoint that would've been used without S2A.
	if fallbackHandshake, err := fallback.DefaultFallbackClientHandshakeFunc(config.endpoint); err == nil {
		fallbackOpts = &s2a.FallbackOptions{
			FallbackClientHandshakeFunc: fallbackHandshake,
		}
	}

	s2aTransportCreds, err := s2a.NewClientCreds(&s2a.ClientOptions{
		S2AAddress:   config.s2aAddress,
		FallbackOpts: fallbackOpts,
	})
	if err != nil {
		// Use default if we cannot initialize S2A client transport credentials.
		return defaultTransportCreds, config.endpoint, nil
	}
	return s2aTransportCreds, config.s2aMTLSEndpoint, nil
}

// GetHTTPTransportConfig returns a client certificate source and a function for
// dialing MTLS with S2A.
func GetHTTPTransportConfig(opts *Options) (cert.Provider, func(context.Context, string, string) (net.Conn, error), error) {
	config, err := getTransportConfig(opts)
	if err != nil {
		return nil, nil, err
	}

	if config.s2aAddress == "" {
		return config.clientCertSource, nil, nil
	}

	var fallbackOpts *s2a.FallbackOptions
	// In case of S2A failure, fall back to the endpoint that would've been used without S2A.
	if fallbackURL, err := url.Parse(config.endpoint); err == nil {
		if fallbackDialer, fallbackServerAddr, err := fallback.DefaultFallbackDialerAndAddress(fallbackURL.Hostname()); err == nil {
			fallbackOpts = &s2a.FallbackOptions{
				FallbackDialer: &s2a.FallbackDialer{
					Dialer:     fallbackDialer,
					ServerAddr: fallbackServerAddr,
				},
			}
		}
	}

	dialTLSContextFunc := s2a.NewS2ADialTLSContextFunc(&s2a.ClientOptions{
		S2AAddress:   config.s2aAddress,
		FallbackOpts: fallbackOpts,
	})
	return nil, dialTLSContextFunc, nil
}

func getTransportConfig(opts *Options) (*transportConfig, error) {
	clientCertSource, err := getClientCertificateSource(opts)
	if err != nil {
		return nil, err
	}
	endpoint, err := getEndpoint(opts, clientCertSource)
	if err != nil {
		return nil, err
	}
	defaultTransportConfig := transportConfig{
		clientCertSource: clientCertSource,
		endpoint:         endpoint,
	}

	if !shouldUseS2A(clientCertSource, opts) {
		return &defaultTransportConfig, nil
	}
	if !opts.isUniverseDomainGDU() {
		return nil, errUniverseNotSupportedMTLS
	}

	s2aMTLSEndpoint := opts.DefaultMTLSEndpoint

	s2aAddress := GetS2AAddress()
	if s2aAddress == "" {
		return &defaultTransportConfig, nil
	}
	return &transportConfig{
		clientCertSource: clientCertSource,
		endpoint:         endpoint,
		s2aAddress:       s2aAddress,
		s2aMTLSEndpoint:  s2aMTLSEndpoint,
	}, nil
}

// getClientCertificateSource returns a default client certificate source, if
// not provided by the user.
//
// A nil default source can be returned if the source does not exist. Any exceptions
// encountered while initializing the default source will be reported as client
// error (ex. corrupt metadata file).
func getClientCertificateSource(opts *Options) (cert.Provider, error) {
	if !isClientCertificateEnabled(opts) {
		return nil, nil
	} else if opts.ClientCertProvider != nil {
		return opts.ClientCertProvider, nil
	}
	return cert.DefaultProvider()

}

// isClientCertificateEnabled returns true by default for all GDU universe domain, unless explicitly overridden by env var
func isClientCertificateEnabled(opts *Options) bool {
	if value, ok := os.LookupEnv(googleAPIUseCertSource); ok {
		// error as false is OK
		b, _ := strconv.ParseBool(value)
		return b
	}
	return opts.isUniverseDomainGDU()
}

type transportConfig struct {
	// The client certificate source.
	clientCertSource cert.Provider
	// The corresponding endpoint to use based on client certificate source.
	endpoint string
	// The S2A address if it can be used, otherwise an empty string.
	s2aAddress string
	// The MTLS endpoint to use with S2A.
	s2aMTLSEndpoint string
}

// getEndpoint returns the endpoint for the service, taking into account the
// user-provided endpoint override "settings.Endpoint".
//
// If no endpoint override is specified, we will either return the default endpoint or
// the default mTLS endpoint if a client certificate is available.
//
// You can override the default endpoint choice (mtls vs. regular) by setting the
// GOOGLE_API_USE_MTLS_ENDPOINT environment variable.
//
// If the endpoint override is an address (host:port) rather than full base
// URL (ex. https://...), then the user-provided address will be merged into
// the default endpoint. For example, WithEndpoint("myhost:8000") and
// DefaultEndpointTemplate("https://UNIVERSE_DOMAIN/bar/baz") will return "https://myhost:8080/bar/baz"
func getEndpoint(opts *Options, clientCertSource cert.Provider) (string, error) {
	if opts.Endpoint == "" {
		mtlsMode := getMTLSMode()
		if mtlsMode == mTLSModeAlways || (clientCertSource != nil && mtlsMode == mTLSModeAuto) {
			if !opts.isUniverseDomainGDU() {
				return "", errUniverseNotSupportedMTLS
			}
			return opts.DefaultMTLSEndpoint, nil
		}
		return opts.defaultEndpoint(), nil
	}
	if strings.Contains(opts.Endpoint, "://") {
		// User passed in a full URL path, use it verbatim.
		return opts.Endpoint, nil
	}
	if opts.defaultEndpoint() == "" {
		// If DefaultEndpointTemplate is not configured,
		// use the user provided endpoint verbatim. This allows a naked
		// "host[:port]" URL to be used with GRPC Direct Path.
		return opts.Endpoint, nil
	}

	// Assume user-provided endpoint is host[:port], merge it with the default endpoint.
	return opts.mergedEndpoint()
}

func getMTLSMode() string {
	mode := os.Getenv(googleAPIUseMTLS)
	if mode == "" {
		mode = os.Getenv(googleAPIUseMTLSOld) // Deprecated.
	}
	if mode == "" {
		return mTLSModeAuto
	}
	return strings.ToLower(mode)
}
