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

package idtoken

import (
	"context"
	"fmt"
	"net/url"
	"time"

	"cloud.google.com/go/auth"
	"cloud.google.com/go/auth/internal"
	"cloud.google.com/go/compute/metadata"
)

const identitySuffix = "instance/service-accounts/default/identity"

// computeCredentials checks if this code is being run on GCE. If it is, it
// will use the metadata service to build a Credentials that fetches ID
// tokens.
func computeCredentials(opts *Options) (*auth.Credentials, error) {
	if opts.CustomClaims != nil {
		return nil, fmt.Errorf("idtoken: Options.CustomClaims can't be used with the metadata service, please provide a service account if you would like to use this feature")
	}
	tp := computeIDTokenProvider{
		audience: opts.Audience,
		format:   opts.ComputeTokenFormat,
		client:   *metadata.NewClient(opts.client()),
	}
	return auth.NewCredentials(&auth.CredentialsOptions{
		TokenProvider: auth.NewCachedTokenProvider(tp, &auth.CachedTokenProviderOptions{
			ExpireEarly: 5 * time.Minute,
		}),
		ProjectIDProvider: auth.CredentialsPropertyFunc(func(context.Context) (string, error) {
			return metadata.ProjectID()
		}),
		UniverseDomainProvider: &internal.ComputeUniverseDomainProvider{},
	}), nil
}

type computeIDTokenProvider struct {
	audience string
	format   ComputeTokenFormat
	client   metadata.Client
}

func (c computeIDTokenProvider) Token(ctx context.Context) (*auth.Token, error) {
	v := url.Values{}
	v.Set("audience", c.audience)
	if c.format != ComputeTokenFormatStandard {
		v.Set("format", "full")
	}
	if c.format == ComputeTokenFormatFullWithLicense {
		v.Set("licenses", "TRUE")
	}
	urlSuffix := identitySuffix + "?" + v.Encode()
	res, err := c.client.Get(urlSuffix)
	if err != nil {
		return nil, err
	}
	if res == "" {
		return nil, fmt.Errorf("idtoken: invalid empty response from metadata service")
	}
	return &auth.Token{
		Value: res,
		Type:  internal.TokenTypeBearer,
		// Compute tokens are valid for one hour:
		// https://cloud.google.com/iam/docs/create-short-lived-credentials-direct#create-id
		Expiry: time.Now().Add(1 * time.Hour),
	}, nil
}
