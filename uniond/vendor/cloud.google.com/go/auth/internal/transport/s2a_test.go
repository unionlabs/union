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
	"testing"
	"time"
)

const (
	testS2AAddr = "testS2AAddress:port"
)

func TestGetS2AAddress(t *testing.T) {
	testCases := []struct {
		name   string
		respFn func() (string, error)
		want   string
	}{
		{
			name:   "test valid config",
			respFn: validConfigResp,
			want:   testS2AAddr,
		},
		{
			name:   "test error when getting config",
			respFn: errorConfigResp,
			want:   "",
		},
		{
			name:   "test invalid config",
			respFn: invalidConfigResp,
			want:   "",
		},
		{
			name:   "test invalid JSON response",
			respFn: invalidJSONResp,
			want:   "",
		},
	}

	oldHTTPGet := httpGetMetadataMTLSConfig
	oldExpiry := configExpiry
	configExpiry = time.Millisecond
	defer func() {
		httpGetMetadataMTLSConfig = oldHTTPGet
		configExpiry = oldExpiry
	}()
	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			httpGetMetadataMTLSConfig = tc.respFn
			if want, got := tc.want, GetS2AAddress(); got != want {
				t.Errorf("want address [%s], got address [%s]", want, got)
			}
			// Let the MTLS config expire at the end of each test case.
			time.Sleep(2 * time.Millisecond)
		})
	}
}

func TestMTLSConfigExpiry(t *testing.T) {
	oldHTTPGet := httpGetMetadataMTLSConfig
	oldExpiry := configExpiry
	configExpiry = 1 * time.Second
	defer func() {
		httpGetMetadataMTLSConfig = oldHTTPGet
		configExpiry = oldExpiry
	}()
	httpGetMetadataMTLSConfig = validConfigResp
	if got, want := GetS2AAddress(), testS2AAddr; got != want {
		t.Errorf("expected address: [%s], got [%s]", want, got)
	}
	httpGetMetadataMTLSConfig = invalidConfigResp
	if got, want := GetS2AAddress(), testS2AAddr; got != want {
		t.Errorf("cached config should still be valid, expected address: [%s], got [%s]", want, got)
	}
	time.Sleep(1 * time.Second)
	if got, want := GetS2AAddress(), ""; got != want {
		t.Errorf("config should be refreshed, expected address: [%s], got [%s]", want, got)
	}
	// Let the MTLS config expire before running other tests.
	time.Sleep(1 * time.Second)
}

func TestIsGoogleS2AEnabled(t *testing.T) {
	testCases := []struct {
		name      string
		useS2AEnv string
		want      bool
	}{
		{
			name:      "true",
			useS2AEnv: "true",
			want:      true,
		},
		{
			name:      "false",
			useS2AEnv: "false",
			want:      false,
		},
		{
			name:      "empty",
			useS2AEnv: "",
			want:      false,
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			if tc.useS2AEnv != "" {
				t.Setenv(googleAPIUseS2AEnv, tc.useS2AEnv)
			}

			if got := isGoogleS2AEnabled(); got != tc.want {
				t.Errorf("got %t, want %t", got, tc.want)
			}
		})
	}
}
