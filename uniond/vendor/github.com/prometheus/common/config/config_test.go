// Copyright 2021 The Prometheus Authors
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
	"encoding/json"
	"net/http"
	"reflect"
	"testing"

	"gopkg.in/yaml.v2"
)

func TestJSONMarshalSecret(t *testing.T) {
	type tmp struct {
		S Secret
	}
	for _, tc := range []struct {
		desc          string
		data          tmp
		expected      string
		marshalSecret bool
		testYAML      bool
	}{
		{
			desc: "inhabited",
			// u003c -> "<"
			// u003e -> ">"
			data:     tmp{"test"},
			expected: "{\"S\":\"\\u003csecret\\u003e\"}",
		},
		{
			desc:          "true value in JSON",
			data:          tmp{"test"},
			expected:      `{"S":"test"}`,
			marshalSecret: true,
		},
		{
			desc: "true value in YAML",
			data: tmp{"test"},
			expected: `s: test
`,
			marshalSecret: true,
			testYAML:      true,
		},
		{
			desc:     "empty",
			data:     tmp{},
			expected: "{\"S\":\"\"}",
		},
	} {
		t.Run(tc.desc, func(t *testing.T) {
			MarshalSecretValue = tc.marshalSecret

			var marshalFN func(any) ([]byte, error)
			if tc.testYAML {
				marshalFN = yaml.Marshal
			} else {
				marshalFN = json.Marshal
			}
			c, err := marshalFN(tc.data)
			if err != nil {
				t.Fatal(err)
			}
			if tc.expected != string(c) {
				t.Fatalf("Secret not marshaled correctly, got '%s'", string(c))
			}
		})
	}
}

func TestHeaderHTTPHeader(t *testing.T) {
	testcases := map[string]struct {
		header   ProxyHeader
		expected http.Header
	}{
		"basic": {
			header: ProxyHeader{
				"single": []Secret{"v1"},
				"multi":  []Secret{"v1", "v2"},
				"empty":  []Secret{},
				"nil":    nil,
			},
			expected: http.Header{
				"single": []string{"v1"},
				"multi":  []string{"v1", "v2"},
				"empty":  []string{},
				"nil":    nil,
			},
		},
		"nil": {
			header:   nil,
			expected: nil,
		},
	}

	for name, tc := range testcases {
		t.Run(name, func(t *testing.T) {
			actual := tc.header.HTTPHeader()
			if !reflect.DeepEqual(actual, tc.expected) {
				t.Fatalf("expecting: %#v, actual: %#v", tc.expected, actual)
			}
		})
	}
}

func TestHeaderYamlUnmarshal(t *testing.T) {
	testcases := map[string]struct {
		input    string
		expected ProxyHeader
	}{
		"void": {
			input: ``,
		},
		"simple": {
			input:    "single:\n- a\n",
			expected: ProxyHeader{"single": []Secret{"a"}},
		},
		"multi": {
			input:    "multi:\n- a\n- b\n",
			expected: ProxyHeader{"multi": []Secret{"a", "b"}},
		},
		"empty": {
			input:    "{}",
			expected: ProxyHeader{},
		},
		"empty value": {
			input:    "empty:\n",
			expected: ProxyHeader{"empty": nil},
		},
	}

	for name, tc := range testcases {
		t.Run(name, func(t *testing.T) {
			var actual ProxyHeader
			err := yaml.Unmarshal([]byte(tc.input), &actual)
			if err != nil {
				t.Fatalf("error unmarshaling %s: %s", tc.input, err)
			}
			if !reflect.DeepEqual(actual, tc.expected) {
				t.Fatalf("expecting: %#v, actual: %#v", tc.expected, actual)
			}
		})
	}
}

func TestHeaderYamlMarshal(t *testing.T) {
	testcases := map[string]struct {
		input    ProxyHeader
		expected []byte
	}{
		"void": {
			input:    nil,
			expected: []byte("{}\n"),
		},
		"simple": {
			input:    ProxyHeader{"single": []Secret{"a"}},
			expected: []byte("single:\n- <secret>\n"),
		},
		"multi": {
			input:    ProxyHeader{"multi": []Secret{"a", "b"}},
			expected: []byte("multi:\n- <secret>\n- <secret>\n"),
		},
		"empty": {
			input:    ProxyHeader{"empty": nil},
			expected: []byte("empty: []\n"),
		},
	}

	for name, tc := range testcases {
		t.Run(name, func(t *testing.T) {
			actual, err := yaml.Marshal(tc.input)
			if err != nil {
				t.Fatalf("error unmarshaling %#v: %s", tc.input, err)
			}
			if !bytes.Equal(actual, tc.expected) {
				t.Fatalf("expecting: %q, actual: %q", tc.expected, actual)
			}
		})
	}
}

func TestHeaderJsonUnmarshal(t *testing.T) {
	testcases := map[string]struct {
		input    string
		expected ProxyHeader
	}{
		"void": {
			input: `null`,
		},
		"simple": {
			input:    `{"single": ["a"]}`,
			expected: ProxyHeader{"single": []Secret{"a"}},
		},
		"multi": {
			input:    `{"multi": ["a", "b"]}`,
			expected: ProxyHeader{"multi": []Secret{"a", "b"}},
		},
		"empty": {
			input:    `{}`,
			expected: ProxyHeader{},
		},
		"empty value": {
			input:    `{"empty":null}`,
			expected: ProxyHeader{"empty": nil},
		},
	}

	for name, tc := range testcases {
		t.Run(name, func(t *testing.T) {
			var actual ProxyHeader
			err := json.Unmarshal([]byte(tc.input), &actual)
			if err != nil {
				t.Fatalf("error unmarshaling %s: %s", tc.input, err)
			}
			if !reflect.DeepEqual(actual, tc.expected) {
				t.Fatalf("expecting: %#v, actual: %#v", tc.expected, actual)
			}
		})
	}
}

func TestHeaderJsonMarshal(t *testing.T) {
	testcases := map[string]struct {
		input    ProxyHeader
		expected []byte
	}{
		"void": {
			input:    nil,
			expected: []byte("null"),
		},
		"simple": {
			input:    ProxyHeader{"single": []Secret{"a"}},
			expected: []byte("{\"single\":[\"\\u003csecret\\u003e\"]}"),
		},
		"multi": {
			input:    ProxyHeader{"multi": []Secret{"a", "b"}},
			expected: []byte("{\"multi\":[\"\\u003csecret\\u003e\",\"\\u003csecret\\u003e\"]}"),
		},
		"empty": {
			input:    ProxyHeader{"empty": nil},
			expected: []byte(`{"empty":null}`),
		},
	}

	for name, tc := range testcases {
		t.Run(name, func(t *testing.T) {
			actual, err := json.Marshal(tc.input)
			if err != nil {
				t.Fatalf("error marshaling %#v: %s", tc.input, err)
			}
			if !bytes.Equal(actual, tc.expected) {
				t.Fatalf("expecting: %q, actual: %q", tc.expected, actual)
			}
		})
	}
}
