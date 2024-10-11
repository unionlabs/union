// Copyright 2023 Google LLC.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package internal

import (
	"encoding/json"
	"fmt"
	"testing"
	"time"
)

const testS2AAddr = "testS2AAddress:port"

var validConfigResp = func() (string, error) {
	validConfig := mtlsConfig{
		S2A: &s2aAddresses{
			PlaintextAddress: testS2AAddr,
			MTLSAddress:      "",
		},
	}
	configStr, err := json.Marshal(validConfig)
	if err != nil {
		return "", err
	}
	return string(configStr), nil
}

var errorConfigResp = func() (string, error) {
	return "", fmt.Errorf("error getting config")
}

var invalidConfigResp = func() (string, error) {
	return "{}", nil
}

var invalidJSONResp = func() (string, error) {
	return "test", nil
}

func TestGetS2AAddress(t *testing.T) {
	testCases := []struct {
		Desc     string
		RespFunc func() (string, error)
		Want     string
	}{
		{
			Desc:     "test valid config",
			RespFunc: validConfigResp,
			Want:     testS2AAddr,
		},
		{
			Desc:     "test error when getting config",
			RespFunc: errorConfigResp,
			Want:     "",
		},
		{
			Desc:     "test invalid config",
			RespFunc: invalidConfigResp,
			Want:     "",
		},
		{
			Desc:     "test invalid JSON response",
			RespFunc: invalidJSONResp,
			Want:     "",
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
		httpGetMetadataMTLSConfig = tc.RespFunc
		if want, got := tc.Want, GetS2AAddress(); got != want {
			t.Errorf("%s: want address [%s], got address [%s]", tc.Desc, want, got)
		}
		// Let the MTLS config expire at the end of each test case.
		time.Sleep(2 * time.Millisecond)
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
