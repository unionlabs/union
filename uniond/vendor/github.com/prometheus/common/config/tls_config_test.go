// Copyright 2016 The Prometheus Authors
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package config

import (
	"bytes"
	"crypto/tls"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"reflect"
	"strings"
	"testing"

	"gopkg.in/yaml.v2"
)

// LoadTLSConfig parses the given file into a tls.Config.
func LoadTLSConfig(filename string) (*tls.Config, error) {
	content, err := os.ReadFile(filename)
	if err != nil {
		return nil, err
	}
	cfg := TLSConfig{}
	switch filepath.Ext(filename) {
	case ".yml":
		if err = yaml.UnmarshalStrict(content, &cfg); err != nil {
			return nil, err
		}
	case ".json":
		decoder := json.NewDecoder(bytes.NewReader(content))
		decoder.DisallowUnknownFields()
		if err = decoder.Decode(&cfg); err != nil {
			return nil, err
		}
	default:
		return nil, fmt.Errorf("Unknown extension: %s", filepath.Ext(filename))
	}
	return NewTLSConfig(&cfg)
}

var expectedTLSConfigs = []struct {
	filename string
	config   *tls.Config
}{
	{
		filename: "tls_config.empty.good.json",
		config:   &tls.Config{},
	},
	{
		filename: "tls_config.insecure.good.json",
		config:   &tls.Config{InsecureSkipVerify: true},
	},
	{
		filename: "tls_config.tlsversion.good.json",
		config:   &tls.Config{MinVersion: tls.VersionTLS11},
	},
	{
		filename: "tls_config.max_version.good.json",
		config:   &tls.Config{MaxVersion: tls.VersionTLS12},
	},
	{
		filename: "tls_config.empty.good.yml",
		config:   &tls.Config{},
	},
	{
		filename: "tls_config.insecure.good.yml",
		config:   &tls.Config{InsecureSkipVerify: true},
	},
	{
		filename: "tls_config.tlsversion.good.yml",
		config:   &tls.Config{MinVersion: tls.VersionTLS11},
	},
	{
		filename: "tls_config.max_version.good.yml",
		config:   &tls.Config{MaxVersion: tls.VersionTLS12},
	},
	{
		filename: "tls_config.max_and_min_version.good.yml",
		config:   &tls.Config{MaxVersion: tls.VersionTLS12, MinVersion: tls.VersionTLS11},
	},
	{
		filename: "tls_config.max_and_min_version_same.good.yml",
		config:   &tls.Config{MaxVersion: tls.VersionTLS12, MinVersion: tls.VersionTLS12},
	},
}

func TestValidTLSConfig(t *testing.T) {
	for _, cfg := range expectedTLSConfigs {
		got, err := LoadTLSConfig("testdata/" + cfg.filename)
		if err != nil {
			t.Fatalf("Error parsing %s: %s", cfg.filename, err)
		}
		// non-nil functions are never equal.
		got.GetClientCertificate = nil
		if !reflect.DeepEqual(got, cfg.config) {
			t.Fatalf("%v: unexpected config result: \n\n%v\n expected\n\n%v", cfg.filename, got, cfg.config)
		}
	}
}

var invalidTLSConfigs = []struct {
	filename string
	errMsg   string
}{
	{
		filename: "tls_config.max_and_min_version.bad.yml",
		errMsg:   "tls_config.max_version must be greater than or equal to tls_config.min_version if both are specified",
	},
}

func TestInvalidTLSConfig(t *testing.T) {
	for _, ee := range invalidTLSConfigs {
		_, err := LoadTLSConfig("testdata/" + ee.filename)
		if err == nil {
			t.Error("Expected error with config but got none")
			continue
		}
		if !strings.Contains(err.Error(), ee.errMsg) {
			t.Errorf("Expected error for invalid HTTP client configuration to contain %q but got: %s", ee.errMsg, err)
		}
	}
}

func TestTLSVersionStringer(t *testing.T) {
	if s := (TLSVersion)(tls.VersionTLS13); s.String() != "TLS13" {
		t.Fatalf("tls.VersionTLS13 string should be TLS13, got %s", s.String())
	}
}

func TestTLSVersionMarshalYAML(t *testing.T) {
	tests := []struct {
		input    TLSVersion
		expected string
		err      error
	}{
		{
			input:    TLSVersions["TLS13"],
			expected: "TLS13\n",
			err:      nil,
		},
		{
			input:    TLSVersions["TLS10"],
			expected: "TLS10\n",
			err:      nil,
		},
		{
			input:    TLSVersion(999),
			expected: "",
			err:      fmt.Errorf("unknown TLS version: 999"),
		},
	}

	for _, test := range tests {
		t.Run(fmt.Sprintf("MarshalYAML(%d)", test.input), func(t *testing.T) {
			actualBytes, err := yaml.Marshal(&test.input)
			if err != nil {
				if test.err == nil || err.Error() != test.err.Error() {
					t.Fatalf("error %v, expected %v", err, test.err)
				}
				return
			}
			actual := string(actualBytes)
			if actual != test.expected {
				t.Fatalf("returned %s, expected %s", actual, test.expected)
			}
		})
	}
}

func TestTLSVersionMarshalJSON(t *testing.T) {
	tests := []struct {
		input    TLSVersion
		expected string
		err      error
	}{
		{
			input:    TLSVersions["TLS13"],
			expected: `"TLS13"`,
			err:      nil,
		},
		{
			input:    TLSVersions["TLS10"],
			expected: `"TLS10"`,
			err:      nil,
		},
		{
			input:    TLSVersion(999),
			expected: "",
			err:      fmt.Errorf("unknown TLS version: 999"),
		},
	}

	for _, test := range tests {
		t.Run(fmt.Sprintf("MarshalJSON(%d)", test.input), func(t *testing.T) {
			actualBytes, err := json.Marshal(&test.input)
			if err != nil {
				if test.err == nil || !strings.HasSuffix(err.Error(), test.err.Error()) {
					t.Fatalf("error %v, expected %v", err, test.err)
				}
				return
			}
			actual := string(actualBytes)
			if actual != test.expected {
				t.Fatalf("returned %s, expected %s", actual, test.expected)
			}
		})
	}
}
