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

package storage

import (
	"testing"

	"github.com/google/go-cmp/cmp"
	"google.golang.org/api/option"
)

func TestApplyStorageOpt(t *testing.T) {
	for _, test := range []struct {
		desc string
		opts []option.ClientOption
		want storageConfig
	}{
		{
			desc: "set JSON option",
			opts: []option.ClientOption{WithJSONReads()},
			want: storageConfig{
				useJSONforReads: true,
				readAPIWasSet:   true,
			},
		},
		{
			desc: "set XML option",
			opts: []option.ClientOption{WithXMLReads()},
			want: storageConfig{
				useJSONforReads: false,
				readAPIWasSet:   true,
			},
		},
		{
			desc: "set conflicting options, last option set takes precedence",
			opts: []option.ClientOption{WithJSONReads(), WithXMLReads()},
			want: storageConfig{
				useJSONforReads: false,
				readAPIWasSet:   true,
			},
		},
		{
			desc: "empty options",
			opts: []option.ClientOption{},
			want: storageConfig{
				useJSONforReads: false,
				readAPIWasSet:   false,
			},
		},
		{
			desc: "set Google API option",
			opts: []option.ClientOption{option.WithEndpoint("")},
			want: storageConfig{
				useJSONforReads: false,
				readAPIWasSet:   false,
			},
		},
	} {
		t.Run(test.desc, func(t *testing.T) {
			var got storageConfig
			for _, opt := range test.opts {
				if storageOpt, ok := opt.(storageClientOption); ok {
					storageOpt.ApplyStorageOpt(&got)
				}
			}
			if !cmp.Equal(got, test.want, cmp.AllowUnexported(storageConfig{})) {
				t.Errorf(cmp.Diff(got, test.want, cmp.AllowUnexported(storageConfig{})))
			}
		})
	}
}
