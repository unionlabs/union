// Copyright 2020 Google LLC.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package idtoken_test

import (
	"context"
	"net/http"

	"google.golang.org/api/idtoken"
)

func ExampleNewTokenSource_setAuthorizationHeader() {
	ctx := context.Background()
	audience := "http://example.com"
	ts, err := idtoken.NewTokenSource(ctx, audience)
	if err != nil {
		// TODO: Handle error.
	}
	token, err := ts.Token()
	if err != nil {
		// TODO: Handle error.
	}
	req, err := http.NewRequest(http.MethodGet, audience, nil)
	if err != nil {
		// TODO: Handle error.
	}
	token.SetAuthHeader(req)
}
