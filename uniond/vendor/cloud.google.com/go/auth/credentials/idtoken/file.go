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
	"encoding/json"
	"fmt"
	"path/filepath"
	"strings"

	"cloud.google.com/go/auth"
	"cloud.google.com/go/auth/credentials"
	"cloud.google.com/go/auth/credentials/impersonate"
	"cloud.google.com/go/auth/internal"
	"cloud.google.com/go/auth/internal/credsfile"
)

const (
	jwtTokenURL = "https://oauth2.googleapis.com/token"
	iamCredAud  = "https://iamcredentials.googleapis.com/"
)

var (
	defaultScopes = []string{
		"https://iamcredentials.googleapis.com/",
		"https://www.googleapis.com/auth/cloud-platform",
	}
)

func credsFromBytes(b []byte, opts *Options) (*auth.Credentials, error) {
	t, err := credsfile.ParseFileType(b)
	if err != nil {
		return nil, err
	}
	switch t {
	case credsfile.ServiceAccountKey:
		f, err := credsfile.ParseServiceAccount(b)
		if err != nil {
			return nil, err
		}
		opts2LO := &auth.Options2LO{
			Email:        f.ClientEmail,
			PrivateKey:   []byte(f.PrivateKey),
			PrivateKeyID: f.PrivateKeyID,
			TokenURL:     f.TokenURL,
			UseIDToken:   true,
		}
		if opts2LO.TokenURL == "" {
			opts2LO.TokenURL = jwtTokenURL
		}

		var customClaims map[string]interface{}
		if opts != nil {
			customClaims = opts.CustomClaims
		}
		if customClaims == nil {
			customClaims = make(map[string]interface{})
		}
		customClaims["target_audience"] = opts.Audience

		opts2LO.PrivateClaims = customClaims
		tp, err := auth.New2LOTokenProvider(opts2LO)
		if err != nil {
			return nil, err
		}
		tp = auth.NewCachedTokenProvider(tp, nil)
		return auth.NewCredentials(&auth.CredentialsOptions{
			TokenProvider:          tp,
			JSON:                   b,
			ProjectIDProvider:      internal.StaticCredentialsProperty(f.ProjectID),
			UniverseDomainProvider: internal.StaticCredentialsProperty(f.UniverseDomain),
		}), nil
	case credsfile.ImpersonatedServiceAccountKey, credsfile.ExternalAccountKey:
		type url struct {
			ServiceAccountImpersonationURL string `json:"service_account_impersonation_url"`
		}
		var accountURL url
		if err := json.Unmarshal(b, &accountURL); err != nil {
			return nil, err
		}
		account := filepath.Base(accountURL.ServiceAccountImpersonationURL)
		account = strings.Split(account, ":")[0]

		baseCreds, err := credentials.DetectDefault(&credentials.DetectOptions{
			Scopes:           defaultScopes,
			CredentialsJSON:  b,
			Client:           opts.client(),
			UseSelfSignedJWT: true,
		})
		if err != nil {
			return nil, err
		}

		config := impersonate.IDTokenOptions{
			Audience:        opts.Audience,
			TargetPrincipal: account,
			IncludeEmail:    true,
			Client:          opts.client(),
			Credentials:     baseCreds,
		}
		creds, err := impersonate.NewIDTokenCredentials(&config)
		if err != nil {
			return nil, err
		}
		return auth.NewCredentials(&auth.CredentialsOptions{
			TokenProvider:          creds,
			JSON:                   b,
			ProjectIDProvider:      auth.CredentialsPropertyFunc(baseCreds.ProjectID),
			UniverseDomainProvider: auth.CredentialsPropertyFunc(baseCreds.UniverseDomain),
			QuotaProjectIDProvider: auth.CredentialsPropertyFunc(baseCreds.QuotaProjectID),
		}), nil
	default:
		return nil, fmt.Errorf("idtoken: unsupported credentials type: %v", t)
	}
}
