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
	"errors"
	"fmt"
	"net/http"
	"os"

	"cloud.google.com/go/auth"
	"cloud.google.com/go/auth/internal"
	"cloud.google.com/go/auth/internal/credsfile"
	"cloud.google.com/go/compute/metadata"
)

// ComputeTokenFormat dictates the the token format when requesting an ID token
// from the compute metadata service.
type ComputeTokenFormat int

const (
	// ComputeTokenFormatDefault means the same as [ComputeTokenFormatFull].
	ComputeTokenFormatDefault ComputeTokenFormat = iota
	// ComputeTokenFormatStandard mean only standard JWT fields will be included
	// in the token.
	ComputeTokenFormatStandard
	// ComputeTokenFormatFull means the token will include claims about the
	// virtual machine instance and its project.
	ComputeTokenFormatFull
	// ComputeTokenFormatFullWithLicense means the same as
	// [ComputeTokenFormatFull] with the addition of claims about licenses
	// associated with the instance.
	ComputeTokenFormatFullWithLicense
)

// Options for the configuration of creation of an ID token with
// [NewCredentials].
type Options struct {
	// Audience is the `aud` field for the token, such as an API endpoint the
	// token will grant access to. Required.
	Audience string
	// ComputeTokenFormat dictates the the token format when requesting an ID
	// token from the compute metadata service. Optional.
	ComputeTokenFormat ComputeTokenFormat
	// CustomClaims specifies private non-standard claims for an ID token.
	// Optional.
	CustomClaims map[string]interface{}

	// CredentialsFile overrides detection logic and sources a credential file
	// from the provided filepath. Optional.
	CredentialsFile string
	// CredentialsJSON overrides detection logic and uses the JSON bytes as the
	// source for the credential. Optional.
	CredentialsJSON []byte
	// Client configures the underlying client used to make network requests
	// when fetching tokens. If provided this should be a fully authenticated
	// client. Optional.
	Client *http.Client
}

func (o *Options) client() *http.Client {
	if o == nil || o.Client == nil {
		return internal.CloneDefaultClient()
	}
	return o.Client
}

func (o *Options) validate() error {
	if o == nil {
		return errors.New("idtoken: opts must be provided")
	}
	if o.Audience == "" {
		return errors.New("idtoken: audience must be specified")
	}
	return nil
}

// NewCredentials creates a [cloud.google.com/go/auth.Credentials] that
// returns ID tokens configured by the opts provided. The parameter
// opts.Audience may not be empty.
func NewCredentials(opts *Options) (*auth.Credentials, error) {
	if err := opts.validate(); err != nil {
		return nil, err
	}
	if b := opts.jsonBytes(); b != nil {
		return credsFromBytes(b, opts)
	}
	if metadata.OnGCE() {
		return computeCredentials(opts)
	}
	return nil, fmt.Errorf("idtoken: couldn't find any credentials")
}

func (o *Options) jsonBytes() []byte {
	if o.CredentialsJSON != nil {
		return o.CredentialsJSON
	}
	var fnOverride string
	if o != nil {
		fnOverride = o.CredentialsFile
	}
	filename := credsfile.GetFileNameFromEnv(fnOverride)
	if filename != "" {
		b, _ := os.ReadFile(filename)
		return b
	}
	return nil
}

// Payload represents a decoded payload of an ID token.
type Payload struct {
	Issuer   string                 `json:"iss"`
	Audience string                 `json:"aud"`
	Expires  int64                  `json:"exp"`
	IssuedAt int64                  `json:"iat"`
	Subject  string                 `json:"sub,omitempty"`
	Claims   map[string]interface{} `json:"-"`
}
