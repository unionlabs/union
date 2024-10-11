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

package impersonate

import (
	"bytes"
	"context"
	"encoding/json"
	"io"
	"net/http"
	"testing"
)

func TestNewIDTokenCredentials(t *testing.T) {
	ctx := context.Background()
	tests := []struct {
		name            string
		aud             string
		targetPrincipal string
		wantErr         bool
	}{
		{
			name:            "missing aud",
			targetPrincipal: "foo@project-id.iam.gserviceaccount.com",
			wantErr:         true,
		},
		{
			name:    "missing targetPrincipal",
			aud:     "http://example.com/",
			wantErr: true,
		},
		{
			name:            "works",
			aud:             "http://example.com/",
			targetPrincipal: "foo@project-id.iam.gserviceaccount.com",
			wantErr:         false,
		},
	}

	for _, tt := range tests {
		name := tt.name
		t.Run(name, func(t *testing.T) {
			idTok := "id-token"
			client := &http.Client{
				Transport: RoundTripFn(func(req *http.Request) *http.Response {
					defer req.Body.Close()
					b, err := io.ReadAll(req.Body)
					if err != nil {
						t.Error(err)
					}
					var r generateIDTokenRequest
					if err := json.Unmarshal(b, &r); err != nil {
						t.Error(err)
					}
					if r.Audience != tt.aud {
						t.Errorf("got %q, want %q", r.Audience, tt.aud)
					}

					resp := generateIDTokenResponse{
						Token: idTok,
					}
					b, err = json.Marshal(&resp)
					if err != nil {
						t.Fatalf("unable to marshal response: %v", err)
					}
					return &http.Response{
						StatusCode: 200,
						Body:       io.NopCloser(bytes.NewReader(b)),
						Header:     make(http.Header),
					}
				}),
			}
			creds, err := NewIDTokenCredentials(&IDTokenOptions{
				Audience:        tt.aud,
				TargetPrincipal: tt.targetPrincipal,
				Client:          client,
			},
			)
			if tt.wantErr && err != nil {
				return
			}
			if err != nil {
				t.Fatal(err)
			}
			tok, err := creds.Token(ctx)
			if err != nil {
				t.Fatal(err)
			}
			if tok.Value != idTok {
				t.Fatalf("got %q, want %q", tok.Value, idTok)
			}
		})
	}
}
