/*
Copyright 2023 The Kubernetes Authors.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

package yaml

import (
	"strings"
	"testing"
)

func TestErrors(t *testing.T) {
	type Into struct {
		Map map[string]interface{} `json:"map"`
		Int int32                  `json:"int"`
	}

	testcases := []struct {
		Name                   string
		Data                   string
		UnmarshalPrefix        string
		UnmarshalStrictPrefix  string
		YAMLToJSONPrefix       string
		YAMLToJSONStrictPrefix string
	}{
		{
			Name:                   "unmarshal syntax",
			Data:                   `map: {`,
			UnmarshalPrefix:        `error converting YAML to JSON: yaml: line 1: `,
			UnmarshalStrictPrefix:  `error converting YAML to JSON: yaml: line 1: `,
			YAMLToJSONPrefix:       `yaml: line 1: `,
			YAMLToJSONStrictPrefix: `yaml: line 1: `,
		},
		{
			Name:                   "unmarshal type",
			Data:                   `map: ""`,
			UnmarshalPrefix:        `error unmarshaling JSON: while decoding JSON: json: `,
			UnmarshalStrictPrefix:  `error unmarshaling JSON: while decoding JSON: json: `,
			YAMLToJSONPrefix:       ``,
			YAMLToJSONStrictPrefix: ``,
		},
		{
			Name:                   "unmarshal unknown",
			Data:                   `unknown: {}`,
			UnmarshalPrefix:        ``,
			UnmarshalStrictPrefix:  `error unmarshaling JSON: while decoding JSON: json: `,
			YAMLToJSONPrefix:       ``,
			YAMLToJSONStrictPrefix: ``,
		},
		{
			Name: "unmarshal duplicate",
			Data: `
int: 0
int: 0`,
			UnmarshalPrefix:        ``,
			UnmarshalStrictPrefix:  `error converting YAML to JSON: yaml: `,
			YAMLToJSONPrefix:       ``,
			YAMLToJSONStrictPrefix: `yaml: unmarshal errors:`,
		},
	}

	for _, tc := range testcases {
		t.Run(tc.Name, func(t *testing.T) {
			v := Into{}
			if err := Unmarshal([]byte(tc.Data), &v); err == nil {
				if len(tc.UnmarshalPrefix) > 0 {
					t.Fatal("expected err")
				}
			} else {
				if len(tc.UnmarshalPrefix) == 0 {
					t.Fatalf("unexpected err %v", err)
				}
				if !strings.HasPrefix(err.Error(), tc.UnmarshalPrefix) {
					t.Fatalf("expected '%s' to start with '%s'", err.Error(), tc.UnmarshalPrefix)
				}
			}

			if err := UnmarshalStrict([]byte(tc.Data), &v); err == nil {
				if len(tc.UnmarshalStrictPrefix) > 0 {
					t.Fatal("expected err")
				}
			} else {
				if len(tc.UnmarshalStrictPrefix) == 0 {
					t.Fatalf("unexpected err %v", err)
				}
				if !strings.HasPrefix(err.Error(), tc.UnmarshalStrictPrefix) {
					t.Fatalf("expected '%s' to start with '%s'", err.Error(), tc.UnmarshalStrictPrefix)
				}
			}

			if _, err := YAMLToJSON([]byte(tc.Data)); err == nil {
				if len(tc.YAMLToJSONPrefix) > 0 {
					t.Fatal("expected err")
				}
			} else {
				if len(tc.YAMLToJSONPrefix) == 0 {
					t.Fatalf("unexpected err %v", err)
				}
				if !strings.HasPrefix(err.Error(), tc.YAMLToJSONPrefix) {
					t.Fatalf("expected '%s' to start with '%s'", err.Error(), tc.YAMLToJSONPrefix)
				}
			}

			if _, err := YAMLToJSONStrict([]byte(tc.Data)); err == nil {
				if len(tc.YAMLToJSONStrictPrefix) > 0 {
					t.Fatal("expected err")
				}
			} else {
				if len(tc.YAMLToJSONStrictPrefix) == 0 {
					t.Fatalf("unexpected err %v", err)
				}
				if !strings.HasPrefix(err.Error(), tc.YAMLToJSONStrictPrefix) {
					t.Fatalf("expected '%s' to start with '%s'", err.Error(), tc.YAMLToJSONStrictPrefix)
				}
			}
		})
	}
}
