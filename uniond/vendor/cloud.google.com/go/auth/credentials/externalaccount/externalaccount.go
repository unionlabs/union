// Copyright 2024 Google LLC
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

package externalaccount

import (
	"context"
	"fmt"
	"net/http"

	"cloud.google.com/go/auth"
	iexacc "cloud.google.com/go/auth/credentials/internal/externalaccount"
	"cloud.google.com/go/auth/internal"
	"cloud.google.com/go/auth/internal/credsfile"
)

// Options for creating a [cloud.google.com/go/auth.Credentials].
type Options struct {
	// Audience is the Secure Token Service (STS) audience which contains the
	// resource name for the workload identity pool or the workforce pool and
	// the provider identifier in that pool. Required.
	Audience string
	// SubjectTokenType is the STS token type based on the Oauth2.0 token
	// exchange spec. Expected values include:
	// - “urn:ietf:params:oauth:token-type:jwt”
	// - “urn:ietf:params:oauth:token-type:id-token”
	// - “urn:ietf:params:oauth:token-type:saml2”
	// - “urn:ietf:params:aws:token-type:aws4_request”
	// Required.
	SubjectTokenType string
	// TokenURL is the STS token exchange endpoint. If not provided, will
	// default to https://sts.UNIVERSE_DOMAIN/v1/token, with UNIVERSE_DOMAIN set
	// to the default service domain googleapis.com unless UniverseDomain is
	// set. Optional.
	TokenURL string
	// TokenInfoURL is the token_info endpoint used to retrieve the account
	// related information (user attributes like account identifier, eg. email,
	// username, uid, etc). This is needed for gCloud session account
	// identification. Optional.
	TokenInfoURL string
	// ServiceAccountImpersonationURL is the URL for the service account
	// impersonation request. This is only required for workload identity pools
	// when APIs to be accessed have not integrated with UberMint.
	ServiceAccountImpersonationURL string
	// ServiceAccountImpersonationLifetimeSeconds is the number of seconds the
	// service account impersonation token will be valid for.
	ServiceAccountImpersonationLifetimeSeconds int
	// ClientSecret is currently only required if token_info endpoint also
	// needs to be called with the generated GCP access token. When provided,
	// STS will be called with additional basic authentication using client_id
	// as username and client_secret as password. Optional.
	ClientSecret string
	// ClientID is only required in conjunction with ClientSecret, as described
	// above. Optional.
	ClientID string
	// CredentialSource contains the necessary information to retrieve the token
	// itself, as well as some environmental information. Optional.
	CredentialSource *CredentialSource
	// QuotaProjectID is injected by gCloud. If the value is non-empty, the Auth
	// libraries will set the x-goog-user-project which overrides the project
	// associated with the credentials. Optional.
	QuotaProjectID string
	// Scopes contains the desired scopes for the returned access token.
	// Optional.
	Scopes []string
	// WorkforcePoolUserProject should be set when it is a workforce pool and
	// not a workload identity pool. The underlying principal must still have
	// serviceusage.services.use IAM permission to use the project for
	// billing/quota. Optional.
	WorkforcePoolUserProject string
	// UniverseDomain is the default service domain for a given Cloud universe.
	// This value will be used in the default STS token URL. The default value
	// is "googleapis.com". It will not be used if TokenURL is set. Optional.
	UniverseDomain string
	// SubjectTokenProvider is an optional token provider for OIDC/SAML
	// credentials. One of SubjectTokenProvider, AWSSecurityCredentialProvider
	// or CredentialSource must be provided. Optional.
	SubjectTokenProvider SubjectTokenProvider
	// AwsSecurityCredentialsProvider is an AWS Security Credential provider
	// for AWS credentials. One of SubjectTokenProvider,
	// AWSSecurityCredentialProvider or CredentialSource must be provided. Optional.
	AwsSecurityCredentialsProvider AwsSecurityCredentialsProvider

	// Client configures the underlying client used to make network requests
	// when fetching tokens. Optional.
	Client *http.Client
}

// CredentialSource stores the information necessary to retrieve the credentials for the STS exchange.
type CredentialSource struct {
	// File is the location for file sourced credentials.
	// One field amongst File, URL, Executable, or EnvironmentID should be
	// provided, depending on the kind of credential in question.
	File string
	// Url is the URL to call for URL sourced credentials.
	// One field amongst File, URL, Executable, or EnvironmentID should be
	// provided, depending on the kind of credential in question.
	URL string
	// Executable is the configuration object for executable sourced credentials.
	// One field amongst File, URL, Executable, or EnvironmentID should be
	// provided, depending on the kind of credential in question.
	Executable *ExecutableConfig
	// EnvironmentID is the EnvironmentID used for AWS sourced credentials.
	// This should start with "AWS".
	// One field amongst File, URL, Executable, or EnvironmentID should be provided, depending on the kind of credential in question.
	EnvironmentID string

	// Headers are the headers to attach to the request for URL sourced
	// credentials.
	Headers map[string]string
	// RegionURL is the metadata URL to retrieve the region from for EC2 AWS
	// credentials.
	RegionURL string
	// RegionalCredVerificationURL is the AWS regional credential verification
	// URL, will default to `https://sts.{region}.amazonaws.com?Action=GetCallerIdentity&Version=2011-06-15`
	// if not provided.
	RegionalCredVerificationURL string
	// IMDSv2SessionTokenURL is the URL to retrieve the session token when using
	// IMDSv2 in AWS.
	IMDSv2SessionTokenURL string
	// Format is the format type for the subject token. Used for File and URL
	// sourced credentials.
	Format *Format
}

// Format contains information needed to retrieve a subject token for URL or
// File sourced credentials.
type Format struct {
	// Type should be either "text" or "json". This determines whether the file
	// or URL sourced credentials expect a simple text subject token or if the
	// subject token will be contained in a JSON object. When not provided
	// "text" type is assumed.
	Type string
	// SubjectTokenFieldName is only required for JSON format. This is the field
	// name that the credentials will check for the subject token in the file or
	// URL response. This would be "access_token" for azure.
	SubjectTokenFieldName string
}

// ExecutableConfig contains information needed for executable sourced credentials.
type ExecutableConfig struct {
	// Command is the the full command to run to retrieve the subject token.
	// This can include arguments. Must be an absolute path for the program. Required.
	Command string
	// TimeoutMillis is the timeout duration, in milliseconds. Defaults to 30000 milliseconds when not provided. Optional.
	TimeoutMillis int
	// OutputFile is the absolute path to the output file where the executable will cache the response.
	// If specified the auth libraries will first check this location before running the executable. Optional.
	OutputFile string
}

// SubjectTokenProvider can be used to supply a subject token to exchange for a
// GCP access token.
type SubjectTokenProvider interface {
	// SubjectToken should return a valid subject token or an error.
	// The external account token provider does not cache the returned subject
	// token, so caching logic should be implemented in the provider to prevent
	// multiple requests for the same subject token.
	SubjectToken(ctx context.Context, opts *RequestOptions) (string, error)
}

// RequestOptions contains information about the requested subject token or AWS
// security credentials from the Google external account credential.
type RequestOptions struct {
	// Audience is the requested audience for the external account credential.
	Audience string
	// Subject token type is the requested subject token type for the external
	// account credential. Expected values include:
	// “urn:ietf:params:oauth:token-type:jwt”
	// “urn:ietf:params:oauth:token-type:id-token”
	// “urn:ietf:params:oauth:token-type:saml2”
	// “urn:ietf:params:aws:token-type:aws4_request”
	SubjectTokenType string
}

// AwsSecurityCredentialsProvider can be used to supply AwsSecurityCredentials
// and an AWS Region to exchange for a GCP access token.
type AwsSecurityCredentialsProvider interface {
	// AwsRegion should return the AWS region or an error.
	AwsRegion(ctx context.Context, opts *RequestOptions) (string, error)
	// GetAwsSecurityCredentials should return a valid set of
	// AwsSecurityCredentials or an error. The external account token provider
	// does not cache the returned security credentials, so caching logic should
	// be implemented in the provider to prevent multiple requests for the
	// same security credentials.
	AwsSecurityCredentials(ctx context.Context, opts *RequestOptions) (*AwsSecurityCredentials, error)
}

// AwsSecurityCredentials models AWS security credentials.
type AwsSecurityCredentials struct {
	// AccessKeyId is the AWS Access Key ID - Required.
	AccessKeyID string `json:"AccessKeyID"`
	// SecretAccessKey is the AWS Secret Access Key - Required.
	SecretAccessKey string `json:"SecretAccessKey"`
	// SessionToken is the AWS Session token. This should be provided for
	// temporary AWS security credentials - Optional.
	SessionToken string `json:"Token"`
}

func (o *Options) validate() error {
	if o == nil {
		return fmt.Errorf("externalaccount: options must be provided")
	}
	return nil
}

func (o *Options) client() *http.Client {
	if o.Client != nil {
		return o.Client
	}
	return internal.CloneDefaultClient()
}

func (o *Options) toInternalOpts() *iexacc.Options {
	if o == nil {
		return nil
	}
	iOpts := &iexacc.Options{
		Audience:                       o.Audience,
		SubjectTokenType:               o.SubjectTokenType,
		TokenURL:                       o.TokenURL,
		TokenInfoURL:                   o.TokenInfoURL,
		ServiceAccountImpersonationURL: o.ServiceAccountImpersonationURL,
		ServiceAccountImpersonationLifetimeSeconds: o.ServiceAccountImpersonationLifetimeSeconds,
		ClientSecret:                   o.ClientSecret,
		ClientID:                       o.ClientID,
		QuotaProjectID:                 o.QuotaProjectID,
		Scopes:                         o.Scopes,
		WorkforcePoolUserProject:       o.WorkforcePoolUserProject,
		UniverseDomain:                 o.UniverseDomain,
		SubjectTokenProvider:           toInternalSubjectTokenProvider(o.SubjectTokenProvider),
		AwsSecurityCredentialsProvider: toInternalAwsSecurityCredentialsProvider(o.AwsSecurityCredentialsProvider),
		Client:                         o.client(),
	}
	if o.CredentialSource != nil {
		cs := o.CredentialSource
		iOpts.CredentialSource = &credsfile.CredentialSource{
			File:                        cs.File,
			URL:                         cs.URL,
			Headers:                     cs.Headers,
			EnvironmentID:               cs.EnvironmentID,
			RegionURL:                   cs.RegionURL,
			RegionalCredVerificationURL: cs.RegionalCredVerificationURL,
			CredVerificationURL:         cs.URL,
			IMDSv2SessionTokenURL:       cs.IMDSv2SessionTokenURL,
		}
		if cs.Executable != nil {
			cse := cs.Executable
			iOpts.CredentialSource.Executable = &credsfile.ExecutableConfig{
				Command:       cse.Command,
				TimeoutMillis: cse.TimeoutMillis,
				OutputFile:    cse.OutputFile,
			}
		}
		if cs.Format != nil {
			csf := cs.Format
			iOpts.CredentialSource.Format = &credsfile.Format{
				Type:                  csf.Type,
				SubjectTokenFieldName: csf.SubjectTokenFieldName,
			}
		}
	}
	return iOpts
}

// NewCredentials returns a [cloud.google.com/go/auth.Credentials] configured
// with the provided options.
func NewCredentials(opts *Options) (*auth.Credentials, error) {
	if err := opts.validate(); err != nil {
		return nil, err
	}

	tp, err := iexacc.NewTokenProvider(opts.toInternalOpts())
	if err != nil {
		return nil, err
	}

	var udp, qpp auth.CredentialsPropertyProvider
	if opts.UniverseDomain != "" {
		udp = internal.StaticCredentialsProperty(opts.UniverseDomain)
	}
	if opts.QuotaProjectID != "" {
		qpp = internal.StaticCredentialsProperty(opts.QuotaProjectID)
	}
	return auth.NewCredentials(&auth.CredentialsOptions{
		TokenProvider:          auth.NewCachedTokenProvider(tp, nil),
		UniverseDomainProvider: udp,
		QuotaProjectIDProvider: qpp,
	}), nil
}

func toInternalSubjectTokenProvider(stp SubjectTokenProvider) iexacc.SubjectTokenProvider {
	if stp == nil {
		return nil
	}
	return &subjectTokenProviderAdapter{stp: stp}
}

func toInternalAwsSecurityCredentialsProvider(scp AwsSecurityCredentialsProvider) iexacc.AwsSecurityCredentialsProvider {
	if scp == nil {
		return nil
	}
	return &awsSecurityCredentialsAdapter{scp: scp}
}

func toInternalAwsSecurityCredentials(sc *AwsSecurityCredentials) *iexacc.AwsSecurityCredentials {
	if sc == nil {
		return nil
	}
	return &iexacc.AwsSecurityCredentials{
		AccessKeyID:     sc.AccessKeyID,
		SecretAccessKey: sc.SecretAccessKey,
		SessionToken:    sc.SessionToken,
	}
}

func toRequestOptions(opts *iexacc.RequestOptions) *RequestOptions {
	if opts == nil {
		return nil
	}
	return &RequestOptions{
		Audience:         opts.Audience,
		SubjectTokenType: opts.SubjectTokenType,
	}
}

// subjectTokenProviderAdapter is an adapter to convert the user supplied
// interface to its internal counterpart.
type subjectTokenProviderAdapter struct {
	stp SubjectTokenProvider
}

func (tp *subjectTokenProviderAdapter) SubjectToken(ctx context.Context, opts *iexacc.RequestOptions) (string, error) {
	return tp.stp.SubjectToken(ctx, toRequestOptions(opts))
}

// awsSecurityCredentialsAdapter is an adapter to convert the user supplied
// interface to its internal counterpart.
type awsSecurityCredentialsAdapter struct {
	scp AwsSecurityCredentialsProvider
}

func (sc *awsSecurityCredentialsAdapter) AwsRegion(ctx context.Context, opts *iexacc.RequestOptions) (string, error) {
	return sc.scp.AwsRegion(ctx, toRequestOptions(opts))
}

func (sc *awsSecurityCredentialsAdapter) AwsSecurityCredentials(ctx context.Context, opts *iexacc.RequestOptions) (*iexacc.AwsSecurityCredentials, error) {
	resp, err := sc.scp.AwsSecurityCredentials(ctx, toRequestOptions(opts))
	if err != nil {
		return nil, err
	}
	return toInternalAwsSecurityCredentials(resp), nil
}
