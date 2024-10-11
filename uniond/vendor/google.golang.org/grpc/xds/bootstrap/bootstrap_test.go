/*
 *
 * Copyright 2022 gRPC authors.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */
package bootstrap

import (
	"encoding/json"
	"testing"

	"google.golang.org/grpc/credentials"
)

const testCredsBuilderName = "test_creds"

var builder = &testCredsBuilder{}

func init() {
	RegisterCredentials(builder)
}

type testCredsBuilder struct {
	config json.RawMessage
}

func (t *testCredsBuilder) Build(config json.RawMessage) (credentials.Bundle, func(), error) {
	t.config = config
	return nil, nil, nil
}

func (t *testCredsBuilder) Name() string {
	return testCredsBuilderName
}

func TestRegisterNew(t *testing.T) {
	c := GetCredentials(testCredsBuilderName)
	if c == nil {
		t.Fatalf("GetCredentials(%q) credential = nil", testCredsBuilderName)
	}

	const sampleConfig = "sample_config"
	rawMessage := json.RawMessage(sampleConfig)
	if _, _, err := c.Build(rawMessage); err != nil {
		t.Errorf("Build(%v) error = %v, want nil", rawMessage, err)
	}

	if got, want := string(builder.config), sampleConfig; got != want {
		t.Errorf("Build config = %v, want %v", got, want)
	}
}

func TestCredsBuilders(t *testing.T) {
	tests := []struct {
		typename string
		builder  Credentials
	}{
		{"google_default", &googleDefaultCredsBuilder{}},
		{"insecure", &insecureCredsBuilder{}},
		{"tls", &tlsCredsBuilder{}},
	}

	for _, test := range tests {
		t.Run(test.typename, func(t *testing.T) {
			if got, want := test.builder.Name(), test.typename; got != want {
				t.Errorf("%T.Name = %v, want %v", test.builder, got, want)
			}

			_, stop, err := test.builder.Build(nil)
			if err != nil {
				t.Fatalf("%T.Build failed: %v", test.builder, err)
			}
			stop()
		})
	}
}

func TestTlsCredsBuilder(t *testing.T) {
	tls := &tlsCredsBuilder{}
	_, stop, err := tls.Build(json.RawMessage(`{}`))
	if err != nil {
		t.Fatalf("tls.Build() failed with error %s when expected to succeed", err)
	}
	stop()

	if _, stop, err := tls.Build(json.RawMessage(`{"ca_certificate_file":"/ca_certificates.pem","refresh_interval": "asdf"}`)); err == nil {
		t.Errorf("tls.Build() succeeded with an invalid refresh interval, when expected to fail")
		stop()
	}
}
