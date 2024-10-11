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

package idtoken_test

import (
	"context"
	"net/http"

	"cloud.google.com/go/auth/credentials/idtoken"
	"cloud.google.com/go/auth/httptransport"
)

func ExampleNewCredentials_setAuthorizationHeader() {
	ctx := context.Background()
	audience := "http://example.com"
	creds, err := idtoken.NewCredentials(&idtoken.Options{
		Audience: audience,
	})
	if err != nil {
		// Handle error.
	}
	token, err := creds.Token(ctx)
	if err != nil {
		// Handle error.
	}
	req, err := http.NewRequest(http.MethodGet, audience, nil)
	if err != nil {
		// Handle error.
	}
	httptransport.SetAuthHeader(token, req)
}
