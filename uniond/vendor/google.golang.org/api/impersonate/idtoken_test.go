// Copyright 2021 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package impersonate

import (
	"bytes"
	"context"
	"encoding/json"
	"io"
	"net/http"
	"testing"

	"google.golang.org/api/option"
)

func TestIDTokenSource(t *testing.T) {
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
					resp := generateIDTokenResponse{
						Token: idTok,
					}
					b, err := json.Marshal(&resp)
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
			ts, err := IDTokenSource(ctx, IDTokenConfig{
				Audience:        tt.aud,
				TargetPrincipal: tt.targetPrincipal,
			}, option.WithHTTPClient(client))
			if tt.wantErr && err != nil {
				return
			}
			if err != nil {
				t.Fatal(err)
			}
			tok, err := ts.Token()
			if err != nil {
				t.Fatal(err)
			}
			if tok.AccessToken != idTok {
				t.Fatalf("got %q, want %q", tok.AccessToken, idTok)
			}
		})
	}
}
