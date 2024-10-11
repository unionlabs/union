/*
 *
 * Copyright 2021 Google LLC
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

package tokenmanager

import (
	"os"
	"testing"

	commonpb "github.com/google/s2a-go/internal/proto/common_go_proto"
)

const testAccessToken = "test_access_token"

func TestNewSingleTokenAccessTokenManagerFailure(t *testing.T) {
	os.Unsetenv(s2aAccessTokenEnvironmentVariable)
	if _, err := NewSingleTokenAccessTokenManager(); err == nil {
		t.Errorf("expected NewSingleTokenAccessTokenManager() to return non-nil error")
	}
}

func TestNewSingleTokenAccessTokenManagerDefaultTokenSuccess(t *testing.T) {
	os.Setenv(s2aAccessTokenEnvironmentVariable, testAccessToken)
	tokenManager, err := NewSingleTokenAccessTokenManager()
	if err != nil {
		t.Errorf("NewSingleTokenAccessTokenManager() returned unexpected error: %v", err)
	}

	token, err := tokenManager.DefaultToken()
	if err != nil {
		t.Errorf("tokenManager.DefaultToken() returned unexpected error: %v", err)
	}
	if got, want := token, testAccessToken; got != want {
		t.Errorf("tokenManager.DefaultToken()= %v, want %s", got, want)
	}
}

func TestNewSingleTokenAccessTokenManagerTokenSuccess(t *testing.T) {
	os.Setenv(s2aAccessTokenEnvironmentVariable, testAccessToken)
	tokenManager, err := NewSingleTokenAccessTokenManager()
	if err != nil {
		t.Errorf("NewSingleTokenAccessTokenManager() returned unexpected error: %v", err)
	}

	token, err := tokenManager.Token(&commonpb.Identity{})
	if err != nil {
		t.Errorf("tokenManager.Token() returned unexpected error: %v", err)
	}
	if got, want := token, testAccessToken; got != want {
		t.Errorf("tokenManager.Token(%v)= %v, want %s", &commonpb.Identity{}, got, want)
	}
}
