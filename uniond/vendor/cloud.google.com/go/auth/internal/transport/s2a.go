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
	"encoding/json"
	"log"
	"os"
	"strconv"
	"sync"
	"time"

	"cloud.google.com/go/auth/internal/transport/cert"
	"cloud.google.com/go/compute/metadata"
)

const (
	configEndpointSuffix = "instance/platform-security/auto-mtls-configuration"
)

var (
	// The period an MTLS config can be reused before needing refresh.
	configExpiry = time.Hour

	// mdsMTLSAutoConfigSource is an instance of reuseMTLSConfigSource, with metadataMTLSAutoConfig as its config source.
	mtlsOnce sync.Once
)

// GetS2AAddress returns the S2A address to be reached via plaintext connection.
// Returns empty string if not set or invalid.
func GetS2AAddress() string {
	c, err := getMetadataMTLSAutoConfig().Config()
	if err != nil {
		return ""
	}
	if !c.Valid() {
		return ""
	}
	return c.S2A.PlaintextAddress
}

type mtlsConfigSource interface {
	Config() (*mtlsConfig, error)
}

// mtlsConfig contains the configuration for establishing MTLS connections with Google APIs.
type mtlsConfig struct {
	S2A    *s2aAddresses `json:"s2a"`
	Expiry time.Time
}

func (c *mtlsConfig) Valid() bool {
	return c != nil && c.S2A != nil && !c.expired()
}
func (c *mtlsConfig) expired() bool {
	return c.Expiry.Before(time.Now())
}

// s2aAddresses contains the plaintext and/or MTLS S2A addresses.
type s2aAddresses struct {
	// PlaintextAddress is the plaintext address to reach S2A
	PlaintextAddress string `json:"plaintext_address"`
	// MTLSAddress is the MTLS address to reach S2A
	MTLSAddress string `json:"mtls_address"`
}

// getMetadataMTLSAutoConfig returns mdsMTLSAutoConfigSource, which is backed by config from MDS with auto-refresh.
func getMetadataMTLSAutoConfig() mtlsConfigSource {
	mtlsOnce.Do(func() {
		mdsMTLSAutoConfigSource = &reuseMTLSConfigSource{
			src: &metadataMTLSAutoConfig{},
		}
	})
	return mdsMTLSAutoConfigSource
}

// reuseMTLSConfigSource caches a valid version of mtlsConfig, and uses `src` to refresh upon config expiry.
// It implements the mtlsConfigSource interface, so calling Config() on it returns an mtlsConfig.
type reuseMTLSConfigSource struct {
	src    mtlsConfigSource // src.Config() is called when config is expired
	mu     sync.Mutex       // mutex guards config
	config *mtlsConfig      // cached config
}

func (cs *reuseMTLSConfigSource) Config() (*mtlsConfig, error) {
	cs.mu.Lock()
	defer cs.mu.Unlock()

	if cs.config.Valid() {
		return cs.config, nil
	}
	c, err := cs.src.Config()
	if err != nil {
		return nil, err
	}
	cs.config = c
	return c, nil
}

// metadataMTLSAutoConfig is an implementation of the interface mtlsConfigSource
// It has the logic to query MDS and return an mtlsConfig
type metadataMTLSAutoConfig struct{}

var httpGetMetadataMTLSConfig = func() (string, error) {
	return metadata.Get(configEndpointSuffix)
}

func (cs *metadataMTLSAutoConfig) Config() (*mtlsConfig, error) {
	resp, err := httpGetMetadataMTLSConfig()
	if err != nil {
		log.Printf("querying MTLS config from MDS endpoint failed: %v", err)
		return defaultMTLSConfig(), nil
	}
	var config mtlsConfig
	err = json.Unmarshal([]byte(resp), &config)
	if err != nil {
		log.Printf("unmarshalling MTLS config from MDS endpoint failed: %v", err)
		return defaultMTLSConfig(), nil
	}

	if config.S2A == nil {
		log.Printf("returned MTLS config from MDS endpoint is invalid: %v", config)
		return defaultMTLSConfig(), nil
	}

	// set new expiry
	config.Expiry = time.Now().Add(configExpiry)
	return &config, nil
}

func defaultMTLSConfig() *mtlsConfig {
	return &mtlsConfig{
		S2A: &s2aAddresses{
			PlaintextAddress: "",
			MTLSAddress:      "",
		},
		Expiry: time.Now().Add(configExpiry),
	}
}

func shouldUseS2A(clientCertSource cert.Provider, opts *Options) bool {
	// If client cert is found, use that over S2A.
	if clientCertSource != nil {
		return false
	}
	// If EXPERIMENTAL_GOOGLE_API_USE_S2A is not set to true, skip S2A.
	if !isGoogleS2AEnabled() {
		return false
	}
	// If DefaultMTLSEndpoint is not set or has endpoint override, skip S2A.
	if opts.DefaultMTLSEndpoint == "" || opts.Endpoint != "" {
		return false
	}
	// If custom HTTP client is provided, skip S2A.
	if opts.Client != nil {
		return false
	}
	// If directPath is enabled, skip S2A.
	return !opts.EnableDirectPath && !opts.EnableDirectPathXds
}

func isGoogleS2AEnabled() bool {
	b, err := strconv.ParseBool(os.Getenv(googleAPIUseS2AEnv))
	if err != nil {
		return false
	}
	return b
}
