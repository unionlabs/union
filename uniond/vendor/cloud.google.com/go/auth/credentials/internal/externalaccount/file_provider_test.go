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

package externalaccount

import (
	"context"
	"testing"

	"cloud.google.com/go/auth/internal/credsfile"
)

func TestRetrieveFileSubjectToken(t *testing.T) {
	var tests = []struct {
		name string
		cs   *credsfile.CredentialSource
		want string
	}{
		{
			name: "untyped file format",
			cs: &credsfile.CredentialSource{
				File: textBaseCredPath,
			},
			want: "street123",
		},
		{
			name: "text file format",
			cs: &credsfile.CredentialSource{
				File:   textBaseCredPath,
				Format: &credsfile.Format{Type: fileTypeText},
			},
			want: "street123",
		},
		{
			name: "JSON file format",
			cs: &credsfile.CredentialSource{
				File:   jsonBaseCredPath,
				Format: &credsfile.Format{Type: fileTypeJSON, SubjectTokenFieldName: "SubjToken"},
			},
			want: "321road",
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			opts := cloneTestOpts()
			opts.CredentialSource = test.cs
			base, err := newSubjectTokenProvider(opts)
			if err != nil {
				t.Fatalf("parse() failed %v", err)
			}

			out, err := base.subjectToken(context.Background())
			if err != nil {
				t.Errorf("Method subjectToken() errored.")
			} else if test.want != out {
				t.Errorf("got %v, want %v", out, test.want)
			}
			if got, want := base.providerType(), fileProviderType; got != want {
				t.Fatalf("got %q, want %q", got, want)
			}
		})
	}
}
