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

package impersonate

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"net/http"
	"time"

	"cloud.google.com/go/auth"
	"cloud.google.com/go/auth/credentials"
	"cloud.google.com/go/auth/httptransport"
	"cloud.google.com/go/auth/internal"
)

// IDTokenOptions for generating an impersonated ID token.
type IDTokenOptions struct {
	// Audience is the `aud` field for the token, such as an API endpoint the
	// token will grant access to. Required.
	Audience string
	// TargetPrincipal is the email address of the service account to
	// impersonate. Required.
	TargetPrincipal string
	// IncludeEmail includes the target service account's email in the token.
	// The resulting token will include both an `email` and `email_verified`
	// claim. Optional.
	IncludeEmail bool
	// Delegates are the ordered service account email addresses in a delegation
	// chain. Each service account must be granted
	// roles/iam.serviceAccountTokenCreator on the next service account in the
	// chain. Optional.
	Delegates []string

	// Credentials used to fetch the ID token. If not provided, and a Client is
	// also not provided, base credentials will try to be detected from the
	// environment. Optional.
	Credentials *auth.Credentials
	// Client configures the underlying client used to make network requests
	// when fetching tokens. If provided the client should provide it's own
	// base credentials at call time. Optional.
	Client *http.Client
}

func (o *IDTokenOptions) validate() error {
	if o == nil {
		return errors.New("impersonate: options must be provided")
	}
	if o.Audience == "" {
		return errors.New("impersonate: audience must be provided")
	}
	if o.TargetPrincipal == "" {
		return errors.New("impersonate: target service account must be provided")
	}
	return nil
}

var (
	defaultScope = "https://www.googleapis.com/auth/cloud-platform"
)

// NewIDTokenCredentials creates an impersonated
// [cloud.google.com/go/auth/Credentials] that returns ID tokens configured
// with the provided config and using credentials loaded from Application
// Default Credentials as the base credentials if not provided with the opts.
// The tokens produced are valid for one hour and are automatically refreshed.
func NewIDTokenCredentials(opts *IDTokenOptions) (*auth.Credentials, error) {
	if err := opts.validate(); err != nil {
		return nil, err
	}
	var client *http.Client
	var creds *auth.Credentials
	if opts.Client == nil && opts.Credentials == nil {
		var err error
		// TODO: test not signed jwt more
		creds, err = credentials.DetectDefault(&credentials.DetectOptions{
			Scopes:           []string{defaultScope},
			UseSelfSignedJWT: true,
		})
		if err != nil {
			return nil, err
		}
		client, err = httptransport.NewClient(&httptransport.Options{
			Credentials: creds,
		})
		if err != nil {
			return nil, err
		}
	} else if opts.Client == nil {
		creds = opts.Credentials
		client = internal.CloneDefaultClient()
		if err := httptransport.AddAuthorizationMiddleware(client, opts.Credentials); err != nil {
			return nil, err
		}
	} else {
		client = opts.Client
	}

	itp := impersonatedIDTokenProvider{
		client:          client,
		targetPrincipal: opts.TargetPrincipal,
		audience:        opts.Audience,
		includeEmail:    opts.IncludeEmail,
	}
	for _, v := range opts.Delegates {
		itp.delegates = append(itp.delegates, formatIAMServiceAccountName(v))
	}

	var udp auth.CredentialsPropertyProvider
	if creds != nil {
		udp = auth.CredentialsPropertyFunc(creds.UniverseDomain)
	}
	return auth.NewCredentials(&auth.CredentialsOptions{
		TokenProvider:          auth.NewCachedTokenProvider(itp, nil),
		UniverseDomainProvider: udp,
	}), nil
}

type generateIDTokenRequest struct {
	Audience     string   `json:"audience"`
	IncludeEmail bool     `json:"includeEmail"`
	Delegates    []string `json:"delegates,omitempty"`
}

type generateIDTokenResponse struct {
	Token string `json:"token"`
}

type impersonatedIDTokenProvider struct {
	client *http.Client

	targetPrincipal string
	audience        string
	includeEmail    bool
	delegates       []string
}

func (i impersonatedIDTokenProvider) Token(ctx context.Context) (*auth.Token, error) {
	genIDTokenReq := generateIDTokenRequest{
		Audience:     i.audience,
		IncludeEmail: i.includeEmail,
		Delegates:    i.delegates,
	}
	bodyBytes, err := json.Marshal(genIDTokenReq)
	if err != nil {
		return nil, fmt.Errorf("impersonate: unable to marshal request: %w", err)
	}

	url := fmt.Sprintf("%s/v1/%s:generateIdToken", iamCredentialsEndpoint, formatIAMServiceAccountName(i.targetPrincipal))
	req, err := http.NewRequest("POST", url, bytes.NewReader(bodyBytes))
	if err != nil {
		return nil, fmt.Errorf("impersonate: unable to create request: %w", err)
	}
	req.Header.Set("Content-Type", "application/json")
	resp, err := i.client.Do(req)
	if err != nil {
		return nil, fmt.Errorf("impersonate: unable to generate ID token: %w", err)
	}
	defer resp.Body.Close()
	body, err := internal.ReadAll(resp.Body)
	if err != nil {
		return nil, fmt.Errorf("impersonate: unable to read body: %w", err)
	}
	if c := resp.StatusCode; c < 200 || c > 299 {
		return nil, fmt.Errorf("impersonate: status code %d: %s", c, body)
	}

	var generateIDTokenResp generateIDTokenResponse
	if err := json.Unmarshal(body, &generateIDTokenResp); err != nil {
		return nil, fmt.Errorf("impersonate: unable to parse response: %w", err)
	}
	return &auth.Token{
		Value: generateIDTokenResp.Token,
		// Generated ID tokens are good for one hour.
		Expiry: time.Now().Add(1 * time.Hour),
	}, nil
}
