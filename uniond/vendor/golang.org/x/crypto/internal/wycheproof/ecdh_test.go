// Copyright 2019 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package wycheproof

import (
	"bytes"
	"crypto/ecdsa"
	"crypto/elliptic"
	"crypto/x509"
	"encoding/asn1"
	"errors"
	"fmt"
	"testing"

	"golang.org/x/crypto/cryptobyte"
	casn1 "golang.org/x/crypto/cryptobyte/asn1"
)

func TestECDH(t *testing.T) {
	type ECDHTestVector struct {
		// A brief description of the test case
		Comment string `json:"comment,omitempty"`
		// A list of flags
		Flags []string `json:"flags,omitempty"`
		// the private key
		Private string `json:"private,omitempty"`
		// Encoded public key
		Public string `json:"public,omitempty"`
		// Test result
		Result string `json:"result,omitempty"`
		// The shared secret key
		Shared string `json:"shared,omitempty"`
		// Identifier of the test case
		TcID int `json:"tcId,omitempty"`
	}

	type ECDHTestGroup struct {
		Curve string            `json:"curve,omitempty"`
		Tests []*ECDHTestVector `json:"tests,omitempty"`
	}

	type Root struct {
		TestGroups []*ECDHTestGroup `json:"testGroups,omitempty"`
	}

	flagsShouldPass := map[string]bool{
		// ParsePKIXPublicKey doesn't support compressed points, but we test
		// them against UnmarshalCompressed anyway.
		"CompressedPoint": true,
		// We don't support decoding custom curves.
		"UnnamedCurve": false,
		// WrongOrder and UnusedParam are only found with UnnamedCurve.
		"WrongOrder":  false,
		"UnusedParam": false,
	}

	// supportedCurves is a map of all elliptic curves supported
	// by crypto/elliptic, which can subsequently be parsed and tested.
	supportedCurves := map[string]bool{
		"secp224r1": true,
		"secp256r1": true,
		"secp384r1": true,
		"secp521r1": true,
	}

	var root Root
	readTestVector(t, "ecdh_test.json", &root)
	for _, tg := range root.TestGroups {
		if !supportedCurves[tg.Curve] {
			continue
		}
		for _, tt := range tg.Tests {
			tg, tt := tg, tt
			t.Run(fmt.Sprintf("%s/%d", tg.Curve, tt.TcID), func(t *testing.T) {
				t.Logf("Type: %v", tt.Result)
				t.Logf("Flags: %q", tt.Flags)
				t.Log(tt.Comment)

				shouldPass := shouldPass(tt.Result, tt.Flags, flagsShouldPass)

				p := decodeHex(tt.Public)
				pp, err := x509.ParsePKIXPublicKey(p)
				if err != nil {
					pp, err = decodeCompressedPKIX(p)
				}
				if err != nil {
					if shouldPass {
						t.Errorf("unexpected parsing error: %s", err)
					}
					return
				}
				pub := pp.(*ecdsa.PublicKey)

				priv := decodeHex(tt.Private)
				shared := decodeHex(tt.Shared)

				x, _ := pub.Curve.ScalarMult(pub.X, pub.Y, priv)
				xBytes := make([]byte, (pub.Curve.Params().BitSize+7)/8)
				got := bytes.Equal(shared, x.FillBytes(xBytes))

				if want := shouldPass; got != want {
					t.Errorf("wanted success %v, got %v", want, got)
				}
			})
		}
	}
}

func decodeCompressedPKIX(der []byte) (interface{}, error) {
	s := cryptobyte.String(der)
	var s1, s2 cryptobyte.String
	var algoOID, namedCurveOID asn1.ObjectIdentifier
	var pointDER []byte
	if !s.ReadASN1(&s1, casn1.SEQUENCE) || !s.Empty() ||
		!s1.ReadASN1(&s2, casn1.SEQUENCE) ||
		!s2.ReadASN1ObjectIdentifier(&algoOID) ||
		!s2.ReadASN1ObjectIdentifier(&namedCurveOID) || !s2.Empty() ||
		!s1.ReadASN1BitStringAsBytes(&pointDER) || !s1.Empty() {
		return nil, errors.New("failed to parse PKIX structure")
	}

	if !algoOID.Equal(oidPublicKeyECDSA) {
		return nil, errors.New("wrong algorithm OID")
	}
	namedCurve := namedCurveFromOID(namedCurveOID)
	if namedCurve == nil {
		return nil, errors.New("unsupported elliptic curve")
	}
	x, y := elliptic.UnmarshalCompressed(namedCurve, pointDER)
	if x == nil {
		return nil, errors.New("failed to unmarshal elliptic curve point")
	}
	pub := &ecdsa.PublicKey{
		Curve: namedCurve,
		X:     x,
		Y:     y,
	}
	return pub, nil
}

var (
	oidPublicKeyECDSA = asn1.ObjectIdentifier{1, 2, 840, 10045, 2, 1}
	oidNamedCurveP224 = asn1.ObjectIdentifier{1, 3, 132, 0, 33}
	oidNamedCurveP256 = asn1.ObjectIdentifier{1, 2, 840, 10045, 3, 1, 7}
	oidNamedCurveP384 = asn1.ObjectIdentifier{1, 3, 132, 0, 34}
	oidNamedCurveP521 = asn1.ObjectIdentifier{1, 3, 132, 0, 35}
)

func namedCurveFromOID(oid asn1.ObjectIdentifier) elliptic.Curve {
	switch {
	case oid.Equal(oidNamedCurveP224):
		return elliptic.P224()
	case oid.Equal(oidNamedCurveP256):
		return elliptic.P256()
	case oid.Equal(oidNamedCurveP384):
		return elliptic.P384()
	case oid.Equal(oidNamedCurveP521):
		return elliptic.P521()
	}
	return nil
}
