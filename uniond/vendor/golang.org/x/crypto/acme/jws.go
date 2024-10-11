// Copyright 2015 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package acme

import (
	"crypto"
	"crypto/ecdsa"
	"crypto/hmac"
	"crypto/rand"
	"crypto/rsa"
	"crypto/sha256"
	_ "crypto/sha512" // need for EC keys
	"encoding/asn1"
	"encoding/base64"
	"encoding/json"
	"errors"
	"fmt"
	"math/big"
)

// KeyID is the account key identity provided by a CA during registration.
type KeyID string

// noKeyID indicates that jwsEncodeJSON should compute and use JWK instead of a KID.
// See jwsEncodeJSON for details.
const noKeyID = KeyID("")

// noPayload indicates jwsEncodeJSON will encode zero-length octet string
// in a JWS request. This is called POST-as-GET in RFC 8555 and is used to make
// authenticated GET requests via POSTing with an empty payload.
// See https://tools.ietf.org/html/rfc8555#section-6.3 for more details.
const noPayload = ""

// noNonce indicates that the nonce should be omitted from the protected header.
// See jwsEncodeJSON for details.
const noNonce = ""

// jsonWebSignature can be easily serialized into a JWS following
// https://tools.ietf.org/html/rfc7515#section-3.2.
type jsonWebSignature struct {
	Protected string `json:"protected"`
	Payload   string `json:"payload"`
	Sig       string `json:"signature"`
}

// jwsEncodeJSON signs claimset using provided key and a nonce.
// The result is serialized in JSON format containing either kid or jwk
// fields based on the provided KeyID value.
//
// The claimset is marshalled using json.Marshal unless it is a string.
// In which case it is inserted directly into the message.
//
// If kid is non-empty, its quoted value is inserted in the protected header
// as "kid" field value. Otherwise, JWK is computed using jwkEncode and inserted
// as "jwk" field value. The "jwk" and "kid" fields are mutually exclusive.
//
// If nonce is non-empty, its quoted value is inserted in the protected header.
//
// See https://tools.ietf.org/html/rfc7515#section-7.
func jwsEncodeJSON(claimset interface{}, key crypto.Signer, kid KeyID, nonce, url string) ([]byte, error) {
	if key == nil {
		return nil, errors.New("nil key")
	}
	alg, sha := jwsHasher(key.Public())
	if alg == "" || !sha.Available() {
		return nil, ErrUnsupportedKey
	}
	headers := struct {
		Alg   string          `json:"alg"`
		KID   string          `json:"kid,omitempty"`
		JWK   json.RawMessage `json:"jwk,omitempty"`
		Nonce string          `json:"nonce,omitempty"`
		URL   string          `json:"url"`
	}{
		Alg:   alg,
		Nonce: nonce,
		URL:   url,
	}
	switch kid {
	case noKeyID:
		jwk, err := jwkEncode(key.Public())
		if err != nil {
			return nil, err
		}
		headers.JWK = json.RawMessage(jwk)
	default:
		headers.KID = string(kid)
	}
	phJSON, err := json.Marshal(headers)
	if err != nil {
		return nil, err
	}
	phead := base64.RawURLEncoding.EncodeToString([]byte(phJSON))
	var payload string
	if val, ok := claimset.(string); ok {
		payload = val
	} else {
		cs, err := json.Marshal(claimset)
		if err != nil {
			return nil, err
		}
		payload = base64.RawURLEncoding.EncodeToString(cs)
	}
	hash := sha.New()
	hash.Write([]byte(phead + "." + payload))
	sig, err := jwsSign(key, sha, hash.Sum(nil))
	if err != nil {
		return nil, err
	}
	enc := jsonWebSignature{
		Protected: phead,
		Payload:   payload,
		Sig:       base64.RawURLEncoding.EncodeToString(sig),
	}
	return json.Marshal(&enc)
}

// jwsWithMAC creates and signs a JWS using the given key and the HS256
// algorithm. kid and url are included in the protected header. rawPayload
// should not be base64-URL-encoded.
func jwsWithMAC(key []byte, kid, url string, rawPayload []byte) (*jsonWebSignature, error) {
	if len(key) == 0 {
		return nil, errors.New("acme: cannot sign JWS with an empty MAC key")
	}
	header := struct {
		Algorithm string `json:"alg"`
		KID       string `json:"kid"`
		URL       string `json:"url,omitempty"`
	}{
		// Only HMAC-SHA256 is supported.
		Algorithm: "HS256",
		KID:       kid,
		URL:       url,
	}
	rawProtected, err := json.Marshal(header)
	if err != nil {
		return nil, err
	}
	protected := base64.RawURLEncoding.EncodeToString(rawProtected)
	payload := base64.RawURLEncoding.EncodeToString(rawPayload)

	h := hmac.New(sha256.New, key)
	if _, err := h.Write([]byte(protected + "." + payload)); err != nil {
		return nil, err
	}
	mac := h.Sum(nil)

	return &jsonWebSignature{
		Protected: protected,
		Payload:   payload,
		Sig:       base64.RawURLEncoding.EncodeToString(mac),
	}, nil
}

// jwkEncode encodes public part of an RSA or ECDSA key into a JWK.
// The result is also suitable for creating a JWK thumbprint.
// https://tools.ietf.org/html/rfc7517
func jwkEncode(pub crypto.PublicKey) (string, error) {
	switch pub := pub.(type) {
	case *rsa.PublicKey:
		// https://tools.ietf.org/html/rfc7518#section-6.3.1
		n := pub.N
		e := big.NewInt(int64(pub.E))
		// Field order is important.
		// See https://tools.ietf.org/html/rfc7638#section-3.3 for details.
		return fmt.Sprintf(`{"e":"%s","kty":"RSA","n":"%s"}`,
			base64.RawURLEncoding.EncodeToString(e.Bytes()),
			base64.RawURLEncoding.EncodeToString(n.Bytes()),
		), nil
	case *ecdsa.PublicKey:
		// https://tools.ietf.org/html/rfc7518#section-6.2.1
		p := pub.Curve.Params()
		n := p.BitSize / 8
		if p.BitSize%8 != 0 {
			n++
		}
		x := pub.X.Bytes()
		if n > len(x) {
			x = append(make([]byte, n-len(x)), x...)
		}
		y := pub.Y.Bytes()
		if n > len(y) {
			y = append(make([]byte, n-len(y)), y...)
		}
		// Field order is important.
		// See https://tools.ietf.org/html/rfc7638#section-3.3 for details.
		return fmt.Sprintf(`{"crv":"%s","kty":"EC","x":"%s","y":"%s"}`,
			p.Name,
			base64.RawURLEncoding.EncodeToString(x),
			base64.RawURLEncoding.EncodeToString(y),
		), nil
	}
	return "", ErrUnsupportedKey
}

// jwsSign signs the digest using the given key.
// The hash is unused for ECDSA keys.
func jwsSign(key crypto.Signer, hash crypto.Hash, digest []byte) ([]byte, error) {
	switch pub := key.Public().(type) {
	case *rsa.PublicKey:
		return key.Sign(rand.Reader, digest, hash)
	case *ecdsa.PublicKey:
		sigASN1, err := key.Sign(rand.Reader, digest, hash)
		if err != nil {
			return nil, err
		}

		var rs struct{ R, S *big.Int }
		if _, err := asn1.Unmarshal(sigASN1, &rs); err != nil {
			return nil, err
		}

		rb, sb := rs.R.Bytes(), rs.S.Bytes()
		size := pub.Params().BitSize / 8
		if size%8 > 0 {
			size++
		}
		sig := make([]byte, size*2)
		copy(sig[size-len(rb):], rb)
		copy(sig[size*2-len(sb):], sb)
		return sig, nil
	}
	return nil, ErrUnsupportedKey
}

// jwsHasher indicates suitable JWS algorithm name and a hash function
// to use for signing a digest with the provided key.
// It returns ("", 0) if the key is not supported.
func jwsHasher(pub crypto.PublicKey) (string, crypto.Hash) {
	switch pub := pub.(type) {
	case *rsa.PublicKey:
		return "RS256", crypto.SHA256
	case *ecdsa.PublicKey:
		switch pub.Params().Name {
		case "P-256":
			return "ES256", crypto.SHA256
		case "P-384":
			return "ES384", crypto.SHA384
		case "P-521":
			return "ES512", crypto.SHA512
		}
	}
	return "", 0
}

// JWKThumbprint creates a JWK thumbprint out of pub
// as specified in https://tools.ietf.org/html/rfc7638.
func JWKThumbprint(pub crypto.PublicKey) (string, error) {
	jwk, err := jwkEncode(pub)
	if err != nil {
		return "", err
	}
	b := sha256.Sum256([]byte(jwk))
	return base64.RawURLEncoding.EncodeToString(b[:]), nil
}
