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

package internal

import (
	"context"
	"crypto/rsa"
	"crypto/x509"
	"encoding/json"
	"encoding/pem"
	"errors"
	"fmt"
	"io"
	"net/http"
	"os"
	"sync"
	"time"

	"cloud.google.com/go/compute/metadata"
)

const (
	// TokenTypeBearer is the auth header prefix for bearer tokens.
	TokenTypeBearer = "Bearer"

	// QuotaProjectEnvVar is the environment variable for setting the quota
	// project.
	QuotaProjectEnvVar = "GOOGLE_CLOUD_QUOTA_PROJECT"
	projectEnvVar      = "GOOGLE_CLOUD_PROJECT"
	maxBodySize        = 1 << 20

	// DefaultUniverseDomain is the default value for universe domain.
	// Universe domain is the default service domain for a given Cloud universe.
	DefaultUniverseDomain = "googleapis.com"
)

// CloneDefaultClient returns a [http.Client] with some good defaults.
func CloneDefaultClient() *http.Client {
	return &http.Client{
		Transport: http.DefaultTransport.(*http.Transport).Clone(),
		Timeout:   30 * time.Second,
	}
}

// ParseKey converts the binary contents of a private key file
// to an *rsa.PrivateKey. It detects whether the private key is in a
// PEM container or not. If so, it extracts the the private key
// from PEM container before conversion. It only supports PEM
// containers with no passphrase.
func ParseKey(key []byte) (*rsa.PrivateKey, error) {
	block, _ := pem.Decode(key)
	if block != nil {
		key = block.Bytes
	}
	parsedKey, err := x509.ParsePKCS8PrivateKey(key)
	if err != nil {
		parsedKey, err = x509.ParsePKCS1PrivateKey(key)
		if err != nil {
			return nil, fmt.Errorf("private key should be a PEM or plain PKCS1 or PKCS8: %w", err)
		}
	}
	parsed, ok := parsedKey.(*rsa.PrivateKey)
	if !ok {
		return nil, errors.New("private key is invalid")
	}
	return parsed, nil
}

// GetQuotaProject retrieves quota project with precedence being: override,
// environment variable, creds json file.
func GetQuotaProject(b []byte, override string) string {
	if override != "" {
		return override
	}
	if env := os.Getenv(QuotaProjectEnvVar); env != "" {
		return env
	}
	if b == nil {
		return ""
	}
	var v struct {
		QuotaProject string `json:"quota_project_id"`
	}
	if err := json.Unmarshal(b, &v); err != nil {
		return ""
	}
	return v.QuotaProject
}

// GetProjectID retrieves project with precedence being: override,
// environment variable, creds json file.
func GetProjectID(b []byte, override string) string {
	if override != "" {
		return override
	}
	if env := os.Getenv(projectEnvVar); env != "" {
		return env
	}
	if b == nil {
		return ""
	}
	var v struct {
		ProjectID string `json:"project_id"` // standard service account key
		Project   string `json:"project"`    // gdch key
	}
	if err := json.Unmarshal(b, &v); err != nil {
		return ""
	}
	if v.ProjectID != "" {
		return v.ProjectID
	}
	return v.Project
}

// ReadAll consumes the whole reader and safely reads the content of its body
// with some overflow protection.
func ReadAll(r io.Reader) ([]byte, error) {
	return io.ReadAll(io.LimitReader(r, maxBodySize))
}

// StaticCredentialsProperty is a helper for creating static credentials
// properties.
func StaticCredentialsProperty(s string) StaticProperty {
	return StaticProperty(s)
}

// StaticProperty always returns that value of the underlying string.
type StaticProperty string

// GetProperty loads the properly value provided the given context.
func (p StaticProperty) GetProperty(context.Context) (string, error) {
	return string(p), nil
}

// ComputeUniverseDomainProvider fetches the credentials universe domain from
// the google cloud metadata service.
type ComputeUniverseDomainProvider struct {
	universeDomainOnce sync.Once
	universeDomain     string
	universeDomainErr  error
}

// GetProperty fetches the credentials universe domain from the google cloud
// metadata service.
func (c *ComputeUniverseDomainProvider) GetProperty(ctx context.Context) (string, error) {
	c.universeDomainOnce.Do(func() {
		c.universeDomain, c.universeDomainErr = getMetadataUniverseDomain(ctx)
	})
	if c.universeDomainErr != nil {
		return "", c.universeDomainErr
	}
	return c.universeDomain, nil
}

// httpGetMetadataUniverseDomain is a package var for unit test substitution.
var httpGetMetadataUniverseDomain = func(ctx context.Context) (string, error) {
	client := metadata.NewClient(&http.Client{Timeout: time.Second})
	// TODO(quartzmo): set ctx on request
	return client.Get("universe/universe_domain")
}

func getMetadataUniverseDomain(ctx context.Context) (string, error) {
	universeDomain, err := httpGetMetadataUniverseDomain(ctx)
	if err == nil {
		return universeDomain, nil
	}
	if _, ok := err.(metadata.NotDefinedError); ok {
		// http.StatusNotFound (404)
		return DefaultUniverseDomain, nil
	}
	return "", err
}
