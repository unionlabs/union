// Copyright 2020 Google LLC.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package internal

import (
	"crypto/tls"
	"net/http"
	"os"
	"testing"
	"time"
)

const (
	testMTLSEndpoint           = "https://test.mtls.googleapis.com/"
	testRegularEndpoint        = "https://test.googleapis.com/"
	testEndpointTemplate       = "https://test.UNIVERSE_DOMAIN/"
	testOverrideEndpoint       = "https://test.override.example.com/"
	testUniverseDomain         = "example.com"
	testUniverseDomainEndpoint = "https://test.example.com/"
)

var dummyClientCertSource = func(info *tls.CertificateRequestInfo) (*tls.Certificate, error) { return nil, nil }

func TestGetEndpoint(t *testing.T) {
	testCases := []struct {
		UserEndpoint            string
		DefaultEndpoint         string
		DefaultEndpointTemplate string
		Want                    string
		WantErr                 bool
	}{
		{
			DefaultEndpointTemplate: "https://foo.UNIVERSE_DOMAIN/bar/baz",
			Want:                    "https://foo.googleapis.com/bar/baz",
		},
		{
			UserEndpoint:            "myhost:3999",
			DefaultEndpointTemplate: "https://foo.UNIVERSE_DOMAIN/bar/baz",
			Want:                    "https://myhost:3999/bar/baz",
		},
		{
			UserEndpoint:            "https://host/path/to/bar",
			DefaultEndpointTemplate: "https://foo.UNIVERSE_DOMAIN/bar/baz",
			Want:                    "https://host/path/to/bar",
		},
		{
			UserEndpoint:    "host:123",
			DefaultEndpoint: "",
			Want:            "host:123",
		},
		{
			UserEndpoint:    "host:123",
			DefaultEndpoint: "default:443",
			Want:            "host:123",
		},
		{
			UserEndpoint:    "host:123",
			DefaultEndpoint: "default:443/bar/baz",
			Want:            "host:123/bar/baz",
		},
	}

	for _, tc := range testCases {
		got, err := getEndpoint(&DialSettings{
			Endpoint:                tc.UserEndpoint,
			DefaultEndpoint:         tc.DefaultEndpoint,
			DefaultEndpointTemplate: tc.DefaultEndpointTemplate,
		}, nil)
		if tc.WantErr && err == nil {
			t.Errorf("want err, got nil err")
			continue
		}
		if !tc.WantErr && err != nil {
			t.Errorf("want nil err, got %v", err)
			continue
		}
		if tc.Want != got {
			t.Errorf("getEndpoint(%q, %q): got %v; want %v", tc.UserEndpoint, tc.DefaultEndpointTemplate, got, tc.Want)
		}
	}
}

func TestGetEndpointWithClientCertSource(t *testing.T) {

	testCases := []struct {
		UserEndpoint        string
		DefaultEndpoint     string
		DefaultMTLSEndpoint string
		Want                string
		WantErr             bool
	}{
		{
			DefaultEndpoint:     "https://foo.googleapis.com/bar/baz",
			DefaultMTLSEndpoint: "https://foo.mtls.googleapis.com/bar/baz",
			Want:                "https://foo.mtls.googleapis.com/bar/baz",
		},
		{
			DefaultEndpoint:     "https://staging-foo.sandbox.googleapis.com/bar/baz",
			DefaultMTLSEndpoint: "https://staging-foo.mtls.sandbox.googleapis.com/bar/baz",
			Want:                "https://staging-foo.mtls.sandbox.googleapis.com/bar/baz",
		},
		{
			UserEndpoint:    "myhost:3999",
			DefaultEndpoint: "https://foo.googleapis.com/bar/baz",
			Want:            "https://myhost:3999/bar/baz",
		},
		{
			UserEndpoint:    "https://host/path/to/bar",
			DefaultEndpoint: "https://foo.googleapis.com/bar/baz",
			Want:            "https://host/path/to/bar",
		},
		{
			UserEndpoint:    "host:port",
			DefaultEndpoint: "",
			Want:            "host:port",
		},
	}

	for _, tc := range testCases {
		got, err := getEndpoint(&DialSettings{
			Endpoint:            tc.UserEndpoint,
			DefaultEndpoint:     tc.DefaultEndpoint,
			DefaultMTLSEndpoint: tc.DefaultMTLSEndpoint,
		}, dummyClientCertSource)
		if tc.WantErr && err == nil {
			t.Errorf("want err, got nil err")
			continue
		}
		if !tc.WantErr && err != nil {
			t.Errorf("want nil err, got %v", err)
			continue
		}
		if tc.Want != got {
			t.Errorf("getEndpoint(%q, %q): got %v; want %v", tc.UserEndpoint, tc.DefaultEndpoint, got, tc.Want)
		}
	}
}

func TestGetGRPCTransportConfigAndEndpoint(t *testing.T) {
	testCases := []struct {
		Desc          string
		InputSettings *DialSettings
		S2ARespFunc   func() (string, error)
		WantEndpoint  string
	}{
		{
			"has client cert",
			&DialSettings{
				DefaultMTLSEndpoint: testMTLSEndpoint,
				DefaultEndpoint:     testRegularEndpoint,
				ClientCertSource:    dummyClientCertSource,
			},
			validConfigResp,
			testMTLSEndpoint,
		},
		{
			"no client cert, S2A address not empty",
			&DialSettings{
				DefaultMTLSEndpoint: testMTLSEndpoint,
				DefaultEndpoint:     testRegularEndpoint,
			},
			validConfigResp,
			testMTLSEndpoint,
		},
		{
			"no client cert, S2A address not empty, EnableDirectPath == true",
			&DialSettings{
				DefaultMTLSEndpoint: testMTLSEndpoint,
				DefaultEndpoint:     testRegularEndpoint,
				EnableDirectPath:    true,
			},
			validConfigResp,
			testRegularEndpoint,
		},
		{
			"no client cert, S2A address not empty, EnableDirectPathXds == true",
			&DialSettings{
				DefaultMTLSEndpoint: testMTLSEndpoint,
				DefaultEndpoint:     testRegularEndpoint,
				EnableDirectPathXds: true,
			},
			validConfigResp,
			testRegularEndpoint,
		},
		{
			"no client cert, S2A address empty",
			&DialSettings{
				DefaultMTLSEndpoint: testMTLSEndpoint,
				DefaultEndpoint:     testRegularEndpoint,
			},
			invalidConfigResp,
			testRegularEndpoint,
		},
		{
			"no client cert, S2A address not empty, override endpoint",
			&DialSettings{
				DefaultMTLSEndpoint:     testMTLSEndpoint,
				DefaultEndpointTemplate: testEndpointTemplate,
				Endpoint:                testOverrideEndpoint,
			},
			validConfigResp,
			testOverrideEndpoint,
		},
		{
			"no client cert, S2A address not empty, DefaultMTLSEndpoint not set",
			&DialSettings{
				DefaultMTLSEndpoint:     "",
				DefaultEndpointTemplate: testEndpointTemplate,
			},
			validConfigResp,
			testRegularEndpoint,
		},
	}
	defer setupTest()()

	for _, tc := range testCases {
		httpGetMetadataMTLSConfig = tc.S2ARespFunc
		if tc.InputSettings.ClientCertSource != nil {
			os.Setenv("GOOGLE_API_USE_CLIENT_CERTIFICATE", "true")
		} else {
			os.Setenv("GOOGLE_API_USE_CLIENT_CERTIFICATE", "false")
		}
		_, endpoint, _ := GetGRPCTransportConfigAndEndpoint(tc.InputSettings)
		if tc.WantEndpoint != endpoint {
			t.Errorf("%s: want endpoint: [%s], got [%s]", tc.Desc, tc.WantEndpoint, endpoint)
		}
		// Let the cached MTLS config expire at the end of each test case.
		time.Sleep(2 * time.Millisecond)
	}
}

func TestGetHTTPTransportConfigAndEndpoint_s2a(t *testing.T) {
	testCases := []struct {
		Desc          string
		InputSettings *DialSettings
		S2ARespFunc   func() (string, error)
		WantEndpoint  string
		DialFuncNil   bool
	}{
		{
			"has client cert",
			&DialSettings{
				DefaultMTLSEndpoint: testMTLSEndpoint,
				DefaultEndpoint:     testRegularEndpoint,
				ClientCertSource:    dummyClientCertSource,
			},
			validConfigResp,
			testMTLSEndpoint,
			true,
		},
		{
			"no client cert, S2A address not empty",
			&DialSettings{
				DefaultMTLSEndpoint: testMTLSEndpoint,
				DefaultEndpoint:     testRegularEndpoint,
			},
			validConfigResp,
			testMTLSEndpoint,
			false,
		},
		{
			"no client cert, S2A address not empty, EnableDirectPath == true",
			&DialSettings{
				DefaultMTLSEndpoint: testMTLSEndpoint,
				DefaultEndpoint:     testRegularEndpoint,
				EnableDirectPath:    true,
			},
			validConfigResp,
			testRegularEndpoint,
			true,
		},
		{
			"no client cert, S2A address not empty, EnableDirectPathXds == true",
			&DialSettings{
				DefaultMTLSEndpoint: testMTLSEndpoint,
				DefaultEndpoint:     testRegularEndpoint,
				EnableDirectPathXds: true,
			},
			validConfigResp,
			testRegularEndpoint,
			true,
		},
		{
			"no client cert, S2A address empty",
			&DialSettings{
				DefaultMTLSEndpoint: testMTLSEndpoint,
				DefaultEndpoint:     testRegularEndpoint,
			},
			invalidConfigResp,
			testRegularEndpoint,
			true,
		},
		{
			"no client cert, S2A address not empty, override endpoint",
			&DialSettings{
				DefaultMTLSEndpoint: testMTLSEndpoint,
				DefaultEndpoint:     testRegularEndpoint,
				Endpoint:            testOverrideEndpoint,
			},
			validConfigResp,
			testOverrideEndpoint,
			true,
		},
		{
			"no client cert, S2A address not empty, but DefaultMTLSEndpoint is not set",
			&DialSettings{
				DefaultMTLSEndpoint: "",
				DefaultEndpoint:     testRegularEndpoint,
			},
			validConfigResp,
			testRegularEndpoint,
			true,
		},
		{
			"no client cert, S2A address not empty, custom HTTP client",
			&DialSettings{
				DefaultMTLSEndpoint: testMTLSEndpoint,
				DefaultEndpoint:     testRegularEndpoint,
				HTTPClient:          http.DefaultClient,
			},
			validConfigResp,
			testRegularEndpoint,
			true,
		},
	}

	defer setupTest()()

	for _, tc := range testCases {
		httpGetMetadataMTLSConfig = tc.S2ARespFunc
		if tc.InputSettings.ClientCertSource != nil {
			os.Setenv("GOOGLE_API_USE_CLIENT_CERTIFICATE", "true")
		} else {
			os.Setenv("GOOGLE_API_USE_CLIENT_CERTIFICATE", "false")
		}
		_, dialFunc, endpoint, err := GetHTTPTransportConfigAndEndpoint(tc.InputSettings)
		if err != nil {
			t.Fatalf("%s: err: %v", tc.Desc, err)
		}
		if tc.WantEndpoint != endpoint {
			t.Errorf("%s: want endpoint: [%s], got [%s]", tc.Desc, tc.WantEndpoint, endpoint)
		}
		if want, got := tc.DialFuncNil, dialFunc == nil; want != got {
			t.Errorf("%s: expecting returned dialFunc is nil: [%v], got [%v]", tc.Desc, tc.DialFuncNil, got)
		}
		// Let MTLS config expire at end of each test case.
		time.Sleep(2 * time.Millisecond)
	}
}

func setupTest() func() {
	oldHTTPGet := httpGetMetadataMTLSConfig
	oldExpiry := configExpiry
	oldUseS2A := os.Getenv(googleAPIUseS2AEnv)
	oldUseClientCert := os.Getenv("GOOGLE_API_USE_CLIENT_CERTIFICATE")

	configExpiry = time.Millisecond
	os.Setenv(googleAPIUseS2AEnv, "true")

	return func() {
		httpGetMetadataMTLSConfig = oldHTTPGet
		configExpiry = oldExpiry
		os.Setenv(googleAPIUseS2AEnv, oldUseS2A)
		os.Setenv("GOOGLE_API_USE_CLIENT_CERTIFICATE", oldUseClientCert)
	}
}

func TestGetHTTPTransportConfigAndEndpoint_UniverseDomain(t *testing.T) {
	testCases := []struct {
		name         string
		ds           *DialSettings
		wantEndpoint string
		wantErr      error
	}{
		{
			name: "google default universe (GDU), no client cert",
			ds: &DialSettings{
				DefaultEndpoint:         testRegularEndpoint,
				DefaultEndpointTemplate: testEndpointTemplate,
				DefaultMTLSEndpoint:     testMTLSEndpoint,
			},
			wantEndpoint: testRegularEndpoint,
		},
		{
			name: "google default universe (GDU), client cert",
			ds: &DialSettings{
				DefaultEndpoint:         testRegularEndpoint,
				DefaultEndpointTemplate: testEndpointTemplate,
				DefaultMTLSEndpoint:     testMTLSEndpoint,
				ClientCertSource:        dummyClientCertSource,
			},
			wantEndpoint: testMTLSEndpoint,
		},
		{
			name: "UniverseDomain, no client cert",
			ds: &DialSettings{
				DefaultEndpoint:         testRegularEndpoint,
				DefaultEndpointTemplate: testEndpointTemplate,
				DefaultMTLSEndpoint:     testMTLSEndpoint,
				UniverseDomain:          testUniverseDomain,
			},
			wantEndpoint: testUniverseDomainEndpoint,
		},
		{
			name: "UniverseDomain, client cert",
			ds: &DialSettings{
				DefaultEndpoint:         testRegularEndpoint,
				DefaultEndpointTemplate: testEndpointTemplate,
				DefaultMTLSEndpoint:     testMTLSEndpoint,
				UniverseDomain:          testUniverseDomain,
				ClientCertSource:        dummyClientCertSource,
			},
			wantEndpoint: testUniverseDomainEndpoint,
			wantErr:      errUniverseNotSupportedMTLS,
		},
	}

	for _, tc := range testCases {
		if tc.ds.ClientCertSource != nil {
			os.Setenv("GOOGLE_API_USE_CLIENT_CERTIFICATE", "true")
		} else {
			os.Setenv("GOOGLE_API_USE_CLIENT_CERTIFICATE", "false")
		}
		_, _, endpoint, err := GetHTTPTransportConfigAndEndpoint(tc.ds)
		if err != nil {
			if err != tc.wantErr {
				t.Fatalf("%s: err: %v", tc.name, err)
			}
		} else {
			if tc.wantEndpoint != endpoint {
				t.Errorf("%s: want endpoint: [%s], got [%s]", tc.name, tc.wantEndpoint, endpoint)
			}
		}
	}
}

func TestGetGRPCTransportConfigAndEndpoint_UniverseDomain(t *testing.T) {
	testCases := []struct {
		name         string
		ds           *DialSettings
		wantEndpoint string
		wantErr      error
	}{
		{
			name: "google default universe (GDU), no client cert",
			ds: &DialSettings{
				DefaultEndpoint:         testRegularEndpoint,
				DefaultEndpointTemplate: testEndpointTemplate,
				DefaultMTLSEndpoint:     testMTLSEndpoint,
			},
			wantEndpoint: testRegularEndpoint,
		},
		{
			name: "google default universe (GDU), no client cert, endpoint",
			ds: &DialSettings{
				DefaultEndpoint:         testRegularEndpoint,
				DefaultEndpointTemplate: testEndpointTemplate,
				DefaultMTLSEndpoint:     testMTLSEndpoint,
				Endpoint:                testOverrideEndpoint,
			},
			wantEndpoint: testOverrideEndpoint,
		},
		{
			name: "google default universe (GDU), client cert",
			ds: &DialSettings{
				DefaultEndpoint:         testRegularEndpoint,
				DefaultEndpointTemplate: testEndpointTemplate,
				DefaultMTLSEndpoint:     testMTLSEndpoint,
				ClientCertSource:        dummyClientCertSource,
			},
			wantEndpoint: testMTLSEndpoint,
		},
		{
			name: "google default universe (GDU), client cert, endpoint",
			ds: &DialSettings{
				DefaultEndpoint:         testRegularEndpoint,
				DefaultEndpointTemplate: testEndpointTemplate,
				DefaultMTLSEndpoint:     testMTLSEndpoint,
				ClientCertSource:        dummyClientCertSource,
				Endpoint:                testOverrideEndpoint,
			},
			wantEndpoint: testOverrideEndpoint,
		},
		{
			name: "UniverseDomain, no client cert",
			ds: &DialSettings{
				DefaultEndpoint:         testRegularEndpoint,
				DefaultEndpointTemplate: testEndpointTemplate,
				DefaultMTLSEndpoint:     testMTLSEndpoint,
				UniverseDomain:          testUniverseDomain,
			},
			wantEndpoint: testUniverseDomainEndpoint,
		},
		{
			name: "UniverseDomain, no client cert, endpoint",
			ds: &DialSettings{
				DefaultEndpoint:         testRegularEndpoint,
				DefaultEndpointTemplate: testEndpointTemplate,
				DefaultMTLSEndpoint:     testMTLSEndpoint,
				UniverseDomain:          testUniverseDomain,
				Endpoint:                testOverrideEndpoint,
			},
			wantEndpoint: testOverrideEndpoint,
		},
		{
			name: "UniverseDomain, client cert",
			ds: &DialSettings{
				DefaultEndpoint:         testRegularEndpoint,
				DefaultEndpointTemplate: testEndpointTemplate,
				DefaultMTLSEndpoint:     testMTLSEndpoint,
				UniverseDomain:          testUniverseDomain,
				ClientCertSource:        dummyClientCertSource,
			},
			wantErr: errUniverseNotSupportedMTLS,
		},
		{
			name: "UniverseDomain, client cert, endpoint",
			ds: &DialSettings{
				DefaultEndpoint:         testRegularEndpoint,
				DefaultEndpointTemplate: testEndpointTemplate,
				DefaultMTLSEndpoint:     testMTLSEndpoint,
				UniverseDomain:          testUniverseDomain,
				ClientCertSource:        dummyClientCertSource,
				Endpoint:                testOverrideEndpoint,
			},
			wantEndpoint: testOverrideEndpoint,
		},
	}

	for _, tc := range testCases {
		if tc.ds.ClientCertSource != nil {
			os.Setenv("GOOGLE_API_USE_CLIENT_CERTIFICATE", "true")
		} else {
			os.Setenv("GOOGLE_API_USE_CLIENT_CERTIFICATE", "false")
		}
		_, endpoint, err := GetGRPCTransportConfigAndEndpoint(tc.ds)
		if err != nil {
			if err != tc.wantErr {
				t.Fatalf("%s: err: %v", tc.name, err)
			}
		} else {
			if tc.wantEndpoint != endpoint {
				t.Errorf("%s: want endpoint: [%s], got [%s]", tc.name, tc.wantEndpoint, endpoint)
			}
		}
	}
}
