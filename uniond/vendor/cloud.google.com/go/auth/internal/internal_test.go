// Copyright 2024 Google LLC
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

package internal

import (
	"context"
	"errors"
	"testing"

	"cloud.google.com/go/compute/metadata"
)

func TestComputeUniverseDomainProvider(t *testing.T) {
	fatalErr := errors.New("fatal error")
	notDefinedError := metadata.NotDefinedError("universe/universe_domain")
	testCases := []struct {
		name    string
		getFunc func(ctx context.Context) (string, error)
		want    string
		wantErr error
	}{
		{
			name: "test error",
			getFunc: func(ctx context.Context) (string, error) {
				return "", fatalErr
			},
			want:    "",
			wantErr: fatalErr,
		},
		{
			name: "test error 404",
			getFunc: func(ctx context.Context) (string, error) {
				return "", notDefinedError
			},
			want:    DefaultUniverseDomain,
			wantErr: nil,
		},
		{
			name: "test valid",
			getFunc: func(ctx context.Context) (string, error) {
				return "example.com", nil
			},
			want:    "example.com",
			wantErr: nil,
		},
	}

	oldGet := httpGetMetadataUniverseDomain
	defer func() {
		httpGetMetadataUniverseDomain = oldGet
	}()
	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			httpGetMetadataUniverseDomain = tc.getFunc
			c := ComputeUniverseDomainProvider{}
			got, err := c.GetProperty(context.Background())
			if err != tc.wantErr {
				t.Errorf("got error %v; want error %v", err, tc.wantErr)
			}
			if got != tc.want {
				t.Errorf("got %v; want %v", got, tc.want)
			}
		})
	}
}
