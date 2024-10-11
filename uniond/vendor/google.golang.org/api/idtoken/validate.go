// Copyright 2020 Google LLC.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package idtoken

import (
	"context"
	"crypto"
	"crypto/ecdsa"
	"crypto/elliptic"
	"crypto/rsa"
	"crypto/sha256"
	"encoding/base64"
	"encoding/json"
	"fmt"
	"math/big"
	"net/http"
	"strings"
	"time"

	"google.golang.org/api/option"
	"google.golang.org/api/option/internaloption"
	htransport "google.golang.org/api/transport/http"
)

const (
	es256KeySize      int    = 32
	googleIAPCertsURL string = "https://www.gstatic.com/iap/verify/public_key-jwk"
	googleSACertsURL  string = "https://www.googleapis.com/oauth2/v3/certs"
)

var (
	defaultValidator = &Validator{client: newCachingClient(http.DefaultClient)}
	// now aliases time.Now for testing.
	now = time.Now
)

func defaultValidatorOpts() []ClientOption {
	return []ClientOption{
		internaloption.WithDefaultScopes("https://www.googleapis.com/auth/cloud-platform"),
		option.WithoutAuthentication(),
	}
}

// Payload represents a decoded payload of an ID Token.
type Payload struct {
	Issuer   string                 `json:"iss"`
	Audience string                 `json:"aud"`
	Expires  int64                  `json:"exp"`
	IssuedAt int64                  `json:"iat"`
	Subject  string                 `json:"sub,omitempty"`
	Claims   map[string]interface{} `json:"-"`
}

// jwt represents the segments of a jwt and exposes convenience methods for
// working with the different segments.
type jwt struct {
	header    string
	payload   string
	signature string
}

// jwtHeader represents a parted jwt's header segment.
type jwtHeader struct {
	Algorithm string `json:"alg"`
	Type      string `json:"typ"`
	KeyID     string `json:"kid"`
}

// certResponse represents a list jwks. It is the format returned from known
// Google cert endpoints.
type certResponse struct {
	Keys []jwk `json:"keys"`
}

// jwk is a simplified representation of a standard jwk. It only includes the
// fields used by Google's cert endpoints.
type jwk struct {
	Alg string `json:"alg"`
	Crv string `json:"crv"`
	Kid string `json:"kid"`
	Kty string `json:"kty"`
	Use string `json:"use"`
	E   string `json:"e"`
	N   string `json:"n"`
	X   string `json:"x"`
	Y   string `json:"y"`
}

// Validator provides a way to validate Google ID Tokens with a user provided
// http.Client.
type Validator struct {
	client *cachingClient
}

// NewValidator creates a Validator that uses the options provided to configure
// a the internal http.Client that will be used to make requests to fetch JWKs.
func NewValidator(ctx context.Context, opts ...ClientOption) (*Validator, error) {
	opts = append(defaultValidatorOpts(), opts...)
	client, _, err := htransport.NewClient(ctx, opts...)
	if err != nil {
		return nil, err
	}
	return &Validator{client: newCachingClient(client)}, nil
}

// Validate is used to validate the provided idToken with a known Google cert
// URL. If audience is not empty the audience claim of the Token is validated.
// Upon successful validation a parsed token Payload is returned allowing the
// caller to validate any additional claims.
func (v *Validator) Validate(ctx context.Context, idToken string, audience string) (*Payload, error) {
	return v.validate(ctx, idToken, audience)
}

// Validate is used to validate the provided idToken with a known Google cert
// URL. If audience is not empty the audience claim of the Token is validated.
// Upon successful validation a parsed token Payload is returned allowing the
// caller to validate any additional claims.
func Validate(ctx context.Context, idToken string, audience string) (*Payload, error) {
	// TODO(codyoss): consider adding a check revoked version of the api. See: https://pkg.go.dev/firebase.google.com/go/auth?tab=doc#Client.VerifyIDTokenAndCheckRevoked
	return defaultValidator.validate(ctx, idToken, audience)
}

// ParsePayload parses the given token and returns its payload.
//
// Warning: This function does not validate the token prior to parsing it.
//
// ParsePayload is primarily meant to be used to inspect a token's payload. This is
// useful when validation fails and the payload needs to be inspected.
//
// Note: A successful Validate() invocation with the same token will return an
// identical payload.
func ParsePayload(idToken string) (*Payload, error) {
	jwt, err := parseJWT(idToken)
	if err != nil {
		return nil, err
	}
	return jwt.parsedPayload()
}

func (v *Validator) validate(ctx context.Context, idToken string, audience string) (*Payload, error) {
	jwt, err := parseJWT(idToken)
	if err != nil {
		return nil, err
	}
	header, err := jwt.parsedHeader()
	if err != nil {
		return nil, err
	}
	payload, err := jwt.parsedPayload()
	if err != nil {
		return nil, err
	}
	sig, err := jwt.decodedSignature()
	if err != nil {
		return nil, err
	}

	if audience != "" && payload.Audience != audience {
		return nil, fmt.Errorf("idtoken: audience provided does not match aud claim in the JWT")
	}

	if now().Unix() > payload.Expires {
		return nil, fmt.Errorf("idtoken: token expired: now=%v, expires=%v", now().Unix(), payload.Expires)
	}

	switch header.Algorithm {
	case "RS256":
		if err := v.validateRS256(ctx, header.KeyID, jwt.hashedContent(), sig); err != nil {
			return nil, err
		}
	case "ES256":
		if err := v.validateES256(ctx, header.KeyID, jwt.hashedContent(), sig); err != nil {
			return nil, err
		}
	default:
		return nil, fmt.Errorf("idtoken: expected JWT signed with RS256 or ES256 but found %q", header.Algorithm)
	}

	return payload, nil
}

func (v *Validator) validateRS256(ctx context.Context, keyID string, hashedContent []byte, sig []byte) error {
	certResp, err := v.client.getCert(ctx, googleSACertsURL)
	if err != nil {
		return err
	}
	j, err := findMatchingKey(certResp, keyID)
	if err != nil {
		return err
	}
	dn, err := decode(j.N)
	if err != nil {
		return err
	}
	de, err := decode(j.E)
	if err != nil {
		return err
	}

	pk := &rsa.PublicKey{
		N: new(big.Int).SetBytes(dn),
		E: int(new(big.Int).SetBytes(de).Int64()),
	}
	return rsa.VerifyPKCS1v15(pk, crypto.SHA256, hashedContent, sig)
}

func (v *Validator) validateES256(ctx context.Context, keyID string, hashedContent []byte, sig []byte) error {
	certResp, err := v.client.getCert(ctx, googleIAPCertsURL)
	if err != nil {
		return err
	}
	j, err := findMatchingKey(certResp, keyID)
	if err != nil {
		return err
	}
	dx, err := decode(j.X)
	if err != nil {
		return err
	}
	dy, err := decode(j.Y)
	if err != nil {
		return err
	}

	pk := &ecdsa.PublicKey{
		Curve: elliptic.P256(),
		X:     new(big.Int).SetBytes(dx),
		Y:     new(big.Int).SetBytes(dy),
	}
	r := big.NewInt(0).SetBytes(sig[:es256KeySize])
	s := big.NewInt(0).SetBytes(sig[es256KeySize:])
	if valid := ecdsa.Verify(pk, hashedContent, r, s); !valid {
		return fmt.Errorf("idtoken: ES256 signature not valid")
	}
	return nil
}

func findMatchingKey(response *certResponse, keyID string) (*jwk, error) {
	if response == nil {
		return nil, fmt.Errorf("idtoken: cert response is nil")
	}
	for _, v := range response.Keys {
		if v.Kid == keyID {
			return &v, nil
		}
	}
	return nil, fmt.Errorf("idtoken: could not find matching cert keyId for the token provided")
}

func parseJWT(idToken string) (*jwt, error) {
	segments := strings.Split(idToken, ".")
	if len(segments) != 3 {
		return nil, fmt.Errorf("idtoken: invalid token, token must have three segments; found %d", len(segments))
	}
	return &jwt{
		header:    segments[0],
		payload:   segments[1],
		signature: segments[2],
	}, nil
}

// decodedHeader base64 decodes the header segment.
func (j *jwt) decodedHeader() ([]byte, error) {
	dh, err := decode(j.header)
	if err != nil {
		return nil, fmt.Errorf("idtoken: unable to decode JWT header: %v", err)
	}
	return dh, nil
}

// decodedPayload base64 payload the header segment.
func (j *jwt) decodedPayload() ([]byte, error) {
	p, err := decode(j.payload)
	if err != nil {
		return nil, fmt.Errorf("idtoken: unable to decode JWT payload: %v", err)
	}
	return p, nil
}

// decodedPayload base64 payload the header segment.
func (j *jwt) decodedSignature() ([]byte, error) {
	p, err := decode(j.signature)
	if err != nil {
		return nil, fmt.Errorf("idtoken: unable to decode JWT signature: %v", err)
	}
	return p, nil
}

// parsedHeader returns a struct representing a JWT header.
func (j *jwt) parsedHeader() (jwtHeader, error) {
	var h jwtHeader
	dh, err := j.decodedHeader()
	if err != nil {
		return h, err
	}
	err = json.Unmarshal(dh, &h)
	if err != nil {
		return h, fmt.Errorf("idtoken: unable to unmarshal JWT header: %v", err)
	}
	return h, nil
}

// parsedPayload returns a struct representing a JWT payload.
func (j *jwt) parsedPayload() (*Payload, error) {
	var p Payload
	dp, err := j.decodedPayload()
	if err != nil {
		return nil, err
	}
	if err := json.Unmarshal(dp, &p); err != nil {
		return nil, fmt.Errorf("idtoken: unable to unmarshal JWT payload: %v", err)
	}
	if err := json.Unmarshal(dp, &p.Claims); err != nil {
		return nil, fmt.Errorf("idtoken: unable to unmarshal JWT payload claims: %v", err)
	}
	return &p, nil
}

// hashedContent gets the SHA256 checksum for verification of the JWT.
func (j *jwt) hashedContent() []byte {
	signedContent := j.header + "." + j.payload
	hashed := sha256.Sum256([]byte(signedContent))
	return hashed[:]
}

func (j *jwt) String() string {
	return fmt.Sprintf("%s.%s.%s", j.header, j.payload, j.signature)
}

func decode(s string) ([]byte, error) {
	return base64.RawURLEncoding.DecodeString(s)
}
