// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21

package quic

import (
	"crypto/tls"
	"strings"
)

func newTestTLSConfig(side connSide) *tls.Config {
	config := &tls.Config{
		InsecureSkipVerify: true,
		CipherSuites: []uint16{
			tls.TLS_AES_128_GCM_SHA256,
			tls.TLS_AES_256_GCM_SHA384,
			tls.TLS_CHACHA20_POLY1305_SHA256,
		},
		MinVersion: tls.VersionTLS13,
		// Default key exchange mechanisms as of Go 1.23 minus X25519Kyber768Draft00,
		// which bloats the client hello enough to spill into a second datagram.
		// Tests were written with the assuption each flight in the handshake
		// fits in one datagram, and it's simpler to keep that property.
		CurvePreferences: []tls.CurveID{
			tls.X25519, tls.CurveP256, tls.CurveP384, tls.CurveP521,
		},
	}
	if side == serverSide {
		config.Certificates = []tls.Certificate{testCert}
	}
	return config
}

// newTestTLSConfigWithMoreDefaults returns a *tls.Config for testing
// which behaves more like a default, empty config.
//
// In particular, it uses the default curve preferences, which can increase
// the size of the handshake.
func newTestTLSConfigWithMoreDefaults(side connSide) *tls.Config {
	config := newTestTLSConfig(side)
	config.CipherSuites = nil
	config.CurvePreferences = nil
	return config
}

var testCert = func() tls.Certificate {
	cert, err := tls.X509KeyPair(localhostCert, localhostKey)
	if err != nil {
		panic(err)
	}
	return cert
}()

// localhostCert is a PEM-encoded TLS cert with SAN IPs
// "127.0.0.1" and "[::1]", expiring at Jan 29 16:00:00 2084 GMT.
// generated from src/crypto/tls:
// go run generate_cert.go  --ecdsa-curve P256 --host 127.0.0.1,::1,example.com --ca --start-date "Jan 1 00:00:00 1970" --duration=1000000h
var localhostCert = []byte(`-----BEGIN CERTIFICATE-----
MIIBrDCCAVKgAwIBAgIPCvPhO+Hfv+NW76kWxULUMAoGCCqGSM49BAMCMBIxEDAO
BgNVBAoTB0FjbWUgQ28wIBcNNzAwMTAxMDAwMDAwWhgPMjA4NDAxMjkxNjAwMDBa
MBIxEDAOBgNVBAoTB0FjbWUgQ28wWTATBgcqhkjOPQIBBggqhkjOPQMBBwNCAARh
WRF8p8X9scgW7JjqAwI9nYV8jtkdhqAXG9gyEgnaFNN5Ze9l3Tp1R9yCDBMNsGms
PyfMPe5Jrha/LmjgR1G9o4GIMIGFMA4GA1UdDwEB/wQEAwIChDATBgNVHSUEDDAK
BggrBgEFBQcDATAPBgNVHRMBAf8EBTADAQH/MB0GA1UdDgQWBBSOJri/wLQxq6oC
Y6ZImms/STbTljAuBgNVHREEJzAlggtleGFtcGxlLmNvbYcEfwAAAYcQAAAAAAAA
AAAAAAAAAAAAATAKBggqhkjOPQQDAgNIADBFAiBUguxsW6TGhixBAdORmVNnkx40
HjkKwncMSDbUaeL9jQIhAJwQ8zV9JpQvYpsiDuMmqCuW35XXil3cQ6Drz82c+fvE
-----END CERTIFICATE-----`)

// localhostKey is the private key for localhostCert.
var localhostKey = []byte(testingKey(`-----BEGIN TESTING KEY-----
MIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQgY1B1eL/Bbwf/MDcs
rnvvWhFNr1aGmJJR59PdCN9lVVqhRANCAARhWRF8p8X9scgW7JjqAwI9nYV8jtkd
hqAXG9gyEgnaFNN5Ze9l3Tp1R9yCDBMNsGmsPyfMPe5Jrha/LmjgR1G9
-----END TESTING KEY-----`))

// testingKey helps keep security scanners from getting excited about a private key in this file.
func testingKey(s string) string { return strings.ReplaceAll(s, "TESTING KEY", "PRIVATE KEY") }
