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
	"fmt"
	"net/http"
	"net/url"
	"strings"
	"time"

	"cloud.google.com/go/auth"
	"cloud.google.com/go/auth/internal"
)

// user provides an auth flow for domain-wide delegation, setting
// CredentialsConfig.Subject to be the impersonated user.
func user(opts *CredentialsOptions, client *http.Client, lifetime time.Duration, isStaticToken bool) (auth.TokenProvider, error) {
	u := userTokenProvider{
		client:          client,
		targetPrincipal: opts.TargetPrincipal,
		subject:         opts.Subject,
		lifetime:        lifetime,
	}
	u.delegates = make([]string, len(opts.Delegates))
	for i, v := range opts.Delegates {
		u.delegates[i] = formatIAMServiceAccountName(v)
	}
	u.scopes = make([]string, len(opts.Scopes))
	copy(u.scopes, opts.Scopes)
	var tpo *auth.CachedTokenProviderOptions
	if isStaticToken {
		tpo = &auth.CachedTokenProviderOptions{
			DisableAutoRefresh: true,
		}
	}
	return auth.NewCachedTokenProvider(u, tpo), nil
}

type claimSet struct {
	Iss   string `json:"iss"`
	Scope string `json:"scope,omitempty"`
	Sub   string `json:"sub,omitempty"`
	Aud   string `json:"aud"`
	Iat   int64  `json:"iat"`
	Exp   int64  `json:"exp"`
}

type signJWTRequest struct {
	Payload   string   `json:"payload"`
	Delegates []string `json:"delegates,omitempty"`
}

type signJWTResponse struct {
	// KeyID is the key used to sign the JWT.
	KeyID string `json:"keyId"`
	// SignedJwt contains the automatically generated header; the
	// client-supplied payload; and the signature, which is generated using
	// the key referenced by the `kid` field in the header.
	SignedJWT string `json:"signedJwt"`
}

type exchangeTokenResponse struct {
	AccessToken string `json:"access_token"`
	TokenType   string `json:"token_type"`
	ExpiresIn   int64  `json:"expires_in"`
}

type userTokenProvider struct {
	client *http.Client

	targetPrincipal string
	subject         string
	scopes          []string
	lifetime        time.Duration
	delegates       []string
}

func (u userTokenProvider) Token(ctx context.Context) (*auth.Token, error) {
	signedJWT, err := u.signJWT()
	if err != nil {
		return nil, err
	}
	return u.exchangeToken(ctx, signedJWT)
}

func (u userTokenProvider) signJWT() (string, error) {
	now := time.Now()
	exp := now.Add(u.lifetime)
	claims := claimSet{
		Iss:   u.targetPrincipal,
		Scope: strings.Join(u.scopes, " "),
		Sub:   u.subject,
		Aud:   fmt.Sprintf("%s/token", oauth2Endpoint),
		Iat:   now.Unix(),
		Exp:   exp.Unix(),
	}
	payloadBytes, err := json.Marshal(claims)
	if err != nil {
		return "", fmt.Errorf("impersonate: unable to marshal claims: %w", err)
	}
	signJWTReq := signJWTRequest{
		Payload:   string(payloadBytes),
		Delegates: u.delegates,
	}

	bodyBytes, err := json.Marshal(signJWTReq)
	if err != nil {
		return "", fmt.Errorf("impersonate: unable to marshal request: %w", err)
	}
	reqURL := fmt.Sprintf("%s/v1/%s:signJwt", iamCredentialsEndpoint, formatIAMServiceAccountName(u.targetPrincipal))
	req, err := http.NewRequest("POST", reqURL, bytes.NewReader(bodyBytes))
	if err != nil {
		return "", fmt.Errorf("impersonate: unable to create request: %w", err)
	}
	req.Header.Set("Content-Type", "application/json")
	rawResp, err := u.client.Do(req)
	if err != nil {
		return "", fmt.Errorf("impersonate: unable to sign JWT: %w", err)
	}
	body, err := internal.ReadAll(rawResp.Body)
	if err != nil {
		return "", fmt.Errorf("impersonate: unable to read body: %w", err)
	}
	if c := rawResp.StatusCode; c < 200 || c > 299 {
		return "", fmt.Errorf("impersonate: status code %d: %s", c, body)
	}

	var signJWTResp signJWTResponse
	if err := json.Unmarshal(body, &signJWTResp); err != nil {
		return "", fmt.Errorf("impersonate: unable to parse response: %w", err)
	}
	return signJWTResp.SignedJWT, nil
}

func (u userTokenProvider) exchangeToken(ctx context.Context, signedJWT string) (*auth.Token, error) {
	v := url.Values{}
	v.Set("grant_type", "assertion")
	v.Set("assertion_type", "http://oauth.net/grant_type/jwt/1.0/bearer")
	v.Set("assertion", signedJWT)
	req, err := http.NewRequestWithContext(ctx, "POST", fmt.Sprintf("%s/token", oauth2Endpoint), strings.NewReader(v.Encode()))
	if err != nil {
		return nil, err
	}
	rawResp, err := u.client.Do(req)
	if err != nil {
		return nil, fmt.Errorf("impersonate: unable to exchange token: %w", err)
	}
	body, err := internal.ReadAll(rawResp.Body)
	if err != nil {
		return nil, fmt.Errorf("impersonate: unable to read body: %w", err)
	}
	if c := rawResp.StatusCode; c < 200 || c > 299 {
		return nil, fmt.Errorf("impersonate: status code %d: %s", c, body)
	}

	var tokenResp exchangeTokenResponse
	if err := json.Unmarshal(body, &tokenResp); err != nil {
		return nil, fmt.Errorf("impersonate: unable to parse response: %w", err)
	}

	return &auth.Token{
		Value:  tokenResp.AccessToken,
		Type:   tokenResp.TokenType,
		Expiry: time.Now().Add(time.Second * time.Duration(tokenResp.ExpiresIn)),
	}, nil
}
