// Copyright 2020 Google LLC.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package idtoken

import (
	"fmt"
	"net/url"
	"time"

	"cloud.google.com/go/compute/metadata"
	"golang.org/x/oauth2"

	"google.golang.org/api/internal"
)

// computeTokenSource checks if this code is being run on GCE. If it is, it will
// use the metadata service to build a TokenSource that fetches ID tokens.
func computeTokenSource(audience string, ds *internal.DialSettings) (oauth2.TokenSource, error) {
	if ds.CustomClaims != nil {
		return nil, fmt.Errorf("idtoken: WithCustomClaims can't be used with the metadata service, please provide a service account if you would like to use this feature")
	}
	ts := computeIDTokenSource{
		audience: audience,
	}
	tok, err := ts.Token()
	if err != nil {
		return nil, err
	}
	return oauth2.ReuseTokenSource(tok, ts), nil
}

type computeIDTokenSource struct {
	audience string
}

func (c computeIDTokenSource) Token() (*oauth2.Token, error) {
	v := url.Values{}
	v.Set("audience", c.audience)
	v.Set("format", "full")
	urlSuffix := "instance/service-accounts/default/identity?" + v.Encode()
	res, err := metadata.Get(urlSuffix)
	if err != nil {
		return nil, err
	}
	if res == "" {
		return nil, fmt.Errorf("idtoken: invalid response from metadata service")
	}
	return &oauth2.Token{
		AccessToken: res,
		TokenType:   "bearer",
		// Compute tokens are valid for one hour, leave a little buffer
		Expiry: time.Now().Add(55 * time.Minute),
	}, nil
}
