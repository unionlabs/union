// Copyright 2021 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

/*
Package downscope implements the ability to downscope, or restrict, the
Identity and Access Management permissions that a short-lived Token
can use. Please note that only Google Cloud Storage supports this feature.
For complete documentation, see https://cloud.google.com/iam/docs/downscoping-short-lived-credentials

To downscope permissions of a source credential, you need to define
a Credential Access Boundary. Said Boundary specifies which resources
the newly created credential can access, an upper bound on the permissions
it has over those resources, and optionally attribute-based conditional
access to the aforementioned resources. For more information on IAM
Conditions, see https://cloud.google.com/iam/docs/conditions-overview.

This functionality can be used to provide a third party with
limited access to and permissions on resources held by the owner of the root
credential or internally in conjunction with the principle of least privilege
to ensure that internal services only hold the minimum necessary privileges
for their function.

For example, a token broker can be set up on a server in a private network.
Various workloads (token consumers) in the same network will send authenticated
requests to that broker for downscoped tokens to access or modify specific google
cloud storage buckets. See the NewTokenSource example for an example of how a
token broker would use this package.

The broker will use the functionality in this package to generate a downscoped
token with the requested configuration, and then pass it back to the token
consumer. These downscoped access tokens can then be used to access Google
Storage resources. For instance, you can create a NewClient from the
"cloud.google.com/go/storage" package and pass in option.WithTokenSource(yourTokenSource))
*/
package downscope

import (
	"context"
	"encoding/json"
	"fmt"
	"io/ioutil"
	"net/http"
	"net/url"
	"strings"
	"time"

	"golang.org/x/oauth2"
)

const (
	universeDomainPlaceholder       = "UNIVERSE_DOMAIN"
	identityBindingEndpointTemplate = "https://sts.UNIVERSE_DOMAIN/v1/token"
	defaultUniverseDomain           = "googleapis.com"
)

type accessBoundary struct {
	AccessBoundaryRules []AccessBoundaryRule `json:"accessBoundaryRules"`
}

// An AvailabilityCondition restricts access to a given Resource.
type AvailabilityCondition struct {
	// An Expression specifies the Cloud Storage objects where
	// permissions are available. For further documentation, see
	// https://cloud.google.com/iam/docs/conditions-overview
	Expression string `json:"expression"`
	// Title is short string that identifies the purpose of the condition. Optional.
	Title string `json:"title,omitempty"`
	// Description details about the purpose of the condition. Optional.
	Description string `json:"description,omitempty"`
}

// An AccessBoundaryRule Sets the permissions (and optionally conditions)
// that the new token has on given resource.
type AccessBoundaryRule struct {
	// AvailableResource is the full resource name of the Cloud Storage bucket that the rule applies to.
	// Use the format //storage.googleapis.com/projects/_/buckets/bucket-name.
	AvailableResource string `json:"availableResource"`
	// AvailablePermissions is a list that defines the upper bound on the available permissions
	// for the resource. Each value is the identifier for an IAM predefined role or custom role,
	// with the prefix inRole:. For example: inRole:roles/storage.objectViewer.
	// Only the permissions in these roles will be available.
	AvailablePermissions []string `json:"availablePermissions"`
	// An Condition restricts the availability of permissions
	// to specific Cloud Storage objects. Optional.
	//
	// A Condition can be used to make permissions available for specific objects,
	// rather than all objects in a Cloud Storage bucket.
	Condition *AvailabilityCondition `json:"availabilityCondition,omitempty"`
}

type downscopedTokenResponse struct {
	AccessToken     string `json:"access_token"`
	IssuedTokenType string `json:"issued_token_type"`
	TokenType       string `json:"token_type"`
	ExpiresIn       int    `json:"expires_in"`
}

// DownscopingConfig specifies the information necessary to request a downscoped token.
type DownscopingConfig struct {
	// RootSource is the TokenSource used to create the downscoped token.
	// The downscoped token therefore has some subset of the accesses of
	// the original RootSource.
	RootSource oauth2.TokenSource
	// Rules defines the accesses held by the new
	// downscoped Token. One or more AccessBoundaryRules are required to
	// define permissions for the new downscoped token. Each one defines an
	// access (or set of accesses) that the new token has to a given resource.
	// There can be a maximum of 10 AccessBoundaryRules.
	Rules []AccessBoundaryRule
	// UniverseDomain is the default service domain for a given Cloud universe.
	// The default value is "googleapis.com". Optional.
	UniverseDomain string
}

// identityBindingEndpoint returns the identity binding endpoint with the
// configured universe domain.
func (dc *DownscopingConfig) identityBindingEndpoint() string {
	if dc.UniverseDomain == "" {
		return strings.Replace(identityBindingEndpointTemplate, universeDomainPlaceholder, defaultUniverseDomain, 1)
	}
	return strings.Replace(identityBindingEndpointTemplate, universeDomainPlaceholder, dc.UniverseDomain, 1)
}

// A downscopingTokenSource is used to retrieve a downscoped token with restricted
// permissions compared to the root Token that is used to generate it.
type downscopingTokenSource struct {
	// ctx is the context used to query the API to retrieve a downscoped Token.
	ctx context.Context
	// config holds the information necessary to generate a downscoped Token.
	config DownscopingConfig
	// identityBindingEndpoint is the identity binding endpoint with the
	// configured universe domain.
	identityBindingEndpoint string
}

// NewTokenSource returns a configured downscopingTokenSource.
func NewTokenSource(ctx context.Context, conf DownscopingConfig) (oauth2.TokenSource, error) {
	if conf.RootSource == nil {
		return nil, fmt.Errorf("downscope: rootSource cannot be nil")
	}
	if len(conf.Rules) == 0 {
		return nil, fmt.Errorf("downscope: length of AccessBoundaryRules must be at least 1")
	}
	if len(conf.Rules) > 10 {
		return nil, fmt.Errorf("downscope: length of AccessBoundaryRules may not be greater than 10")
	}
	for _, val := range conf.Rules {
		if val.AvailableResource == "" {
			return nil, fmt.Errorf("downscope: all rules must have a nonempty AvailableResource: %+v", val)
		}
		if len(val.AvailablePermissions) == 0 {
			return nil, fmt.Errorf("downscope: all rules must provide at least one permission: %+v", val)
		}
	}
	return downscopingTokenSource{
		ctx:                     ctx,
		config:                  conf,
		identityBindingEndpoint: conf.identityBindingEndpoint(),
	}, nil
}

// Token() uses a downscopingTokenSource to generate an oauth2 Token.
// Do note that the returned TokenSource is an oauth2.StaticTokenSource. If you wish
// to refresh this token automatically, then initialize a locally defined
// TokenSource struct with the Token held by the StaticTokenSource and wrap
// that TokenSource in an oauth2.ReuseTokenSource.
func (dts downscopingTokenSource) Token() (*oauth2.Token, error) {

	downscopedOptions := struct {
		Boundary accessBoundary `json:"accessBoundary"`
	}{
		Boundary: accessBoundary{
			AccessBoundaryRules: dts.config.Rules,
		},
	}

	tok, err := dts.config.RootSource.Token()
	if err != nil {
		return nil, fmt.Errorf("downscope: unable to obtain root token: %v", err)
	}

	b, err := json.Marshal(downscopedOptions)
	if err != nil {
		return nil, fmt.Errorf("downscope: unable to marshal AccessBoundary payload %v", err)
	}

	form := url.Values{}
	form.Add("grant_type", "urn:ietf:params:oauth:grant-type:token-exchange")
	form.Add("subject_token_type", "urn:ietf:params:oauth:token-type:access_token")
	form.Add("requested_token_type", "urn:ietf:params:oauth:token-type:access_token")
	form.Add("subject_token", tok.AccessToken)
	form.Add("options", string(b))

	myClient := oauth2.NewClient(dts.ctx, nil)
	resp, err := myClient.PostForm(dts.identityBindingEndpoint, form)
	if err != nil {
		return nil, fmt.Errorf("unable to generate POST Request %v", err)
	}
	defer resp.Body.Close()
	respBody, err := ioutil.ReadAll(resp.Body)
	if err != nil {
		return nil, fmt.Errorf("downscope: unable to read response body: %v", err)
	}
	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("downscope: unable to exchange token; %v. Server responded: %s", resp.StatusCode, respBody)
	}

	var tresp downscopedTokenResponse

	err = json.Unmarshal(respBody, &tresp)
	if err != nil {
		return nil, fmt.Errorf("downscope: unable to unmarshal response body: %v", err)
	}

	// an exchanged token that is derived from a service account (2LO) has an expired_in value
	// a token derived from a users token (3LO) does not.
	// The following code uses the time remaining on rootToken for a user as the value for the
	// derived token's lifetime
	var expiryTime time.Time
	if tresp.ExpiresIn > 0 {
		expiryTime = time.Now().Add(time.Duration(tresp.ExpiresIn) * time.Second)
	} else {
		expiryTime = tok.Expiry
	}

	newToken := &oauth2.Token{
		AccessToken: tresp.AccessToken,
		TokenType:   tresp.TokenType,
		Expiry:      expiryTime,
	}
	return newToken, nil
}
