// Copyright 2016 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package autocert

import (
	"bytes"
	"context"
	"crypto"
	"crypto/ecdsa"
	"crypto/elliptic"
	"crypto/rand"
	"crypto/rsa"
	"crypto/tls"
	"crypto/x509"
	"crypto/x509/pkix"
	"encoding/asn1"
	"fmt"
	"io"
	"math/big"
	"net/http"
	"net/http/httptest"
	"reflect"
	"strings"
	"sync"
	"testing"
	"time"

	"golang.org/x/crypto/acme"
	"golang.org/x/crypto/acme/autocert/internal/acmetest"
)

var (
	exampleDomain     = "example.org"
	exampleCertKey    = certKey{domain: exampleDomain}
	exampleCertKeyRSA = certKey{domain: exampleDomain, isRSA: true}
)

type memCache struct {
	t       *testing.T
	mu      sync.Mutex
	keyData map[string][]byte
}

func (m *memCache) Get(ctx context.Context, key string) ([]byte, error) {
	m.mu.Lock()
	defer m.mu.Unlock()

	v, ok := m.keyData[key]
	if !ok {
		return nil, ErrCacheMiss
	}
	return v, nil
}

// filenameSafe returns whether all characters in s are printable ASCII
// and safe to use in a filename on most filesystems.
func filenameSafe(s string) bool {
	for _, c := range s {
		if c < 0x20 || c > 0x7E {
			return false
		}
		switch c {
		case '\\', '/', ':', '*', '?', '"', '<', '>', '|':
			return false
		}
	}
	return true
}

func (m *memCache) Put(ctx context.Context, key string, data []byte) error {
	if !filenameSafe(key) {
		m.t.Errorf("invalid characters in cache key %q", key)
	}

	m.mu.Lock()
	defer m.mu.Unlock()

	m.keyData[key] = data
	return nil
}

func (m *memCache) Delete(ctx context.Context, key string) error {
	m.mu.Lock()
	defer m.mu.Unlock()

	delete(m.keyData, key)
	return nil
}

func newMemCache(t *testing.T) *memCache {
	return &memCache{
		t:       t,
		keyData: make(map[string][]byte),
	}
}

func (m *memCache) numCerts() int {
	m.mu.Lock()
	defer m.mu.Unlock()

	res := 0
	for key := range m.keyData {
		if strings.HasSuffix(key, "+token") ||
			strings.HasSuffix(key, "+key") ||
			strings.HasSuffix(key, "+http-01") {
			continue
		}
		res++
	}
	return res
}

func dummyCert(pub interface{}, san ...string) ([]byte, error) {
	return dateDummyCert(pub, time.Now(), time.Now().Add(90*24*time.Hour), san...)
}

func dateDummyCert(pub interface{}, start, end time.Time, san ...string) ([]byte, error) {
	// use EC key to run faster on 386
	key, err := ecdsa.GenerateKey(elliptic.P256(), rand.Reader)
	if err != nil {
		return nil, err
	}
	t := &x509.Certificate{
		SerialNumber:          randomSerial(),
		NotBefore:             start,
		NotAfter:              end,
		BasicConstraintsValid: true,
		KeyUsage:              x509.KeyUsageKeyEncipherment,
		DNSNames:              san,
	}
	if pub == nil {
		pub = &key.PublicKey
	}
	return x509.CreateCertificate(rand.Reader, t, t, pub, key)
}

func randomSerial() *big.Int {
	serial, err := rand.Int(rand.Reader, new(big.Int).Lsh(big.NewInt(1), 32))
	if err != nil {
		panic(err)
	}
	return serial
}

type algorithmSupport int

const (
	algRSA algorithmSupport = iota
	algECDSA
)

func clientHelloInfo(sni string, alg algorithmSupport) *tls.ClientHelloInfo {
	hello := &tls.ClientHelloInfo{
		ServerName:   sni,
		CipherSuites: []uint16{tls.TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305},
	}
	if alg == algECDSA {
		hello.CipherSuites = append(hello.CipherSuites, tls.TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305)
	}
	return hello
}

func testManager(t *testing.T) *Manager {
	man := &Manager{
		Prompt: AcceptTOS,
		Cache:  newMemCache(t),
	}
	t.Cleanup(man.stopRenew)
	return man
}

func TestGetCertificate(t *testing.T) {
	tests := []struct {
		name        string
		hello       *tls.ClientHelloInfo
		domain      string
		expectError string
		prepare     func(t *testing.T, man *Manager, s *acmetest.CAServer)
		verify      func(t *testing.T, man *Manager, leaf *x509.Certificate)
		disableALPN bool
		disableHTTP bool
	}{
		{
			name:        "ALPN",
			hello:       clientHelloInfo("example.org", algECDSA),
			domain:      "example.org",
			disableHTTP: true,
		},
		{
			name:        "HTTP",
			hello:       clientHelloInfo("example.org", algECDSA),
			domain:      "example.org",
			disableALPN: true,
		},
		{
			name:   "nilPrompt",
			hello:  clientHelloInfo("example.org", algECDSA),
			domain: "example.org",
			prepare: func(t *testing.T, man *Manager, s *acmetest.CAServer) {
				man.Prompt = nil
			},
			expectError: "Manager.Prompt not set",
		},
		{
			name:   "trailingDot",
			hello:  clientHelloInfo("example.org.", algECDSA),
			domain: "example.org",
		},
		{
			name:   "unicodeIDN",
			hello:  clientHelloInfo("éé.com", algECDSA),
			domain: "xn--9caa.com",
		},
		{
			name:   "unicodeIDN/mixedCase",
			hello:  clientHelloInfo("éÉ.com", algECDSA),
			domain: "xn--9caa.com",
		},
		{
			name:   "upperCase",
			hello:  clientHelloInfo("EXAMPLE.ORG", algECDSA),
			domain: "example.org",
		},
		{
			name:   "goodCache",
			hello:  clientHelloInfo("example.org", algECDSA),
			domain: "example.org",
			prepare: func(t *testing.T, man *Manager, s *acmetest.CAServer) {
				// Make a valid cert and cache it.
				c := s.Start().LeafCert(exampleDomain, "ECDSA",
					// Use a time before the Let's Encrypt revocation cutoff to also test
					// that non-Let's Encrypt certificates are not renewed.
					time.Date(2022, time.January, 1, 0, 0, 0, 0, time.UTC),
					time.Date(2122, time.January, 1, 0, 0, 0, 0, time.UTC),
				)
				if err := man.cachePut(context.Background(), exampleCertKey, c); err != nil {
					t.Fatalf("man.cachePut: %v", err)
				}
			},
			// Break the server to check that the cache is used.
			disableALPN: true, disableHTTP: true,
		},
		{
			name:   "expiredCache",
			hello:  clientHelloInfo("example.org", algECDSA),
			domain: "example.org",
			prepare: func(t *testing.T, man *Manager, s *acmetest.CAServer) {
				// Make an expired cert and cache it.
				c := s.Start().LeafCert(exampleDomain, "ECDSA", time.Now().Add(-10*time.Minute), time.Now().Add(-5*time.Minute))
				if err := man.cachePut(context.Background(), exampleCertKey, c); err != nil {
					t.Fatalf("man.cachePut: %v", err)
				}
			},
		},
		{
			name:   "forceRSA",
			hello:  clientHelloInfo("example.org", algECDSA),
			domain: "example.org",
			prepare: func(t *testing.T, man *Manager, s *acmetest.CAServer) {
				man.ForceRSA = true
			},
			verify: func(t *testing.T, man *Manager, leaf *x509.Certificate) {
				if _, ok := leaf.PublicKey.(*ecdsa.PublicKey); !ok {
					t.Errorf("leaf.PublicKey is %T; want *ecdsa.PublicKey", leaf.PublicKey)
				}
			},
		},
		{
			name:   "goodLetsEncrypt",
			hello:  clientHelloInfo("example.org", algECDSA),
			domain: "example.org",
			prepare: func(t *testing.T, man *Manager, s *acmetest.CAServer) {
				// Make a valid certificate issued after the TLS-ALPN-01
				// revocation window and cache it.
				s.IssuerName(pkix.Name{Country: []string{"US"},
					Organization: []string{"Let's Encrypt"}, CommonName: "R3"})
				c := s.Start().LeafCert(exampleDomain, "ECDSA",
					time.Date(2022, time.January, 26, 12, 0, 0, 0, time.UTC),
					time.Date(2122, time.January, 1, 0, 0, 0, 0, time.UTC),
				)
				if err := man.cachePut(context.Background(), exampleCertKey, c); err != nil {
					t.Fatalf("man.cachePut: %v", err)
				}
			},
			// Break the server to check that the cache is used.
			disableALPN: true, disableHTTP: true,
		},
		{
			name:   "revokedLetsEncrypt",
			hello:  clientHelloInfo("example.org", algECDSA),
			domain: "example.org",
			prepare: func(t *testing.T, man *Manager, s *acmetest.CAServer) {
				// Make a certificate issued during the TLS-ALPN-01
				// revocation window and cache it.
				s.IssuerName(pkix.Name{Country: []string{"US"},
					Organization: []string{"Let's Encrypt"}, CommonName: "R3"})
				c := s.Start().LeafCert(exampleDomain, "ECDSA",
					time.Date(2022, time.January, 1, 0, 0, 0, 0, time.UTC),
					time.Date(2122, time.January, 1, 0, 0, 0, 0, time.UTC),
				)
				if err := man.cachePut(context.Background(), exampleCertKey, c); err != nil {
					t.Fatalf("man.cachePut: %v", err)
				}
			},
			verify: func(t *testing.T, man *Manager, leaf *x509.Certificate) {
				if leaf.NotBefore.Before(time.Now().Add(-10 * time.Minute)) {
					t.Error("certificate was not reissued")
				}
			},
		},
		{
			// TestGetCertificate/tokenCache tests the fallback of token
			// certificate fetches to cache when Manager.certTokens misses.
			name:   "tokenCacheALPN",
			hello:  clientHelloInfo("example.org", algECDSA),
			domain: "example.org",
			prepare: func(t *testing.T, man *Manager, s *acmetest.CAServer) {
				// Make a separate manager with a shared cache, simulating
				// separate nodes that serve requests for the same domain.
				man2 := testManager(t)
				man2.Cache = man.Cache
				// Redirect the verification request to man2, although the
				// client request will hit man, testing that they can complete a
				// verification by communicating through the cache.
				s.ResolveGetCertificate("example.org", man2.GetCertificate)
			},
			// Drop the default verification paths.
			disableALPN: true,
		},
		{
			name:   "tokenCacheHTTP",
			hello:  clientHelloInfo("example.org", algECDSA),
			domain: "example.org",
			prepare: func(t *testing.T, man *Manager, s *acmetest.CAServer) {
				man2 := testManager(t)
				man2.Cache = man.Cache
				s.ResolveHandler("example.org", man2.HTTPHandler(nil))
			},
			disableHTTP: true,
		},
		{
			name:   "ecdsa",
			hello:  clientHelloInfo("example.org", algECDSA),
			domain: "example.org",
			verify: func(t *testing.T, man *Manager, leaf *x509.Certificate) {
				if _, ok := leaf.PublicKey.(*ecdsa.PublicKey); !ok {
					t.Error("an ECDSA client was served a non-ECDSA certificate")
				}
			},
		},
		{
			name:   "rsa",
			hello:  clientHelloInfo("example.org", algRSA),
			domain: "example.org",
			verify: func(t *testing.T, man *Manager, leaf *x509.Certificate) {
				if _, ok := leaf.PublicKey.(*rsa.PublicKey); !ok {
					t.Error("an RSA client was served a non-RSA certificate")
				}
			},
		},
		{
			name:   "wrongCacheKeyType",
			hello:  clientHelloInfo("example.org", algECDSA),
			domain: "example.org",
			prepare: func(t *testing.T, man *Manager, s *acmetest.CAServer) {
				// Make an RSA cert and cache it without suffix.
				c := s.Start().LeafCert(exampleDomain, "RSA", time.Now(), time.Now().Add(90*24*time.Hour))
				if err := man.cachePut(context.Background(), exampleCertKey, c); err != nil {
					t.Fatalf("man.cachePut: %v", err)
				}
			},
			verify: func(t *testing.T, man *Manager, leaf *x509.Certificate) {
				// The RSA cached cert should be silently ignored and replaced.
				if _, ok := leaf.PublicKey.(*ecdsa.PublicKey); !ok {
					t.Error("an ECDSA client was served a non-ECDSA certificate")
				}
				if numCerts := man.Cache.(*memCache).numCerts(); numCerts != 1 {
					t.Errorf("found %d certificates in cache; want %d", numCerts, 1)
				}
			},
		},
		{
			name:   "almostExpiredCache",
			hello:  clientHelloInfo("example.org", algECDSA),
			domain: "example.org",
			prepare: func(t *testing.T, man *Manager, s *acmetest.CAServer) {
				man.RenewBefore = 24 * time.Hour
				// Cache an almost expired cert.
				c := s.Start().LeafCert(exampleDomain, "ECDSA", time.Now(), time.Now().Add(10*time.Minute))
				if err := man.cachePut(context.Background(), exampleCertKey, c); err != nil {
					t.Fatalf("man.cachePut: %v", err)
				}
			},
		},
		{
			name:   "provideExternalAuth",
			hello:  clientHelloInfo("example.org", algECDSA),
			domain: "example.org",
			prepare: func(t *testing.T, man *Manager, s *acmetest.CAServer) {
				s.ExternalAccountRequired()

				man.ExternalAccountBinding = &acme.ExternalAccountBinding{
					KID: "test-key",
					Key: make([]byte, 32),
				}
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			man := testManager(t)
			s := acmetest.NewCAServer(t)
			if !tt.disableALPN {
				s.ResolveGetCertificate(tt.domain, man.GetCertificate)
			}
			if !tt.disableHTTP {
				s.ResolveHandler(tt.domain, man.HTTPHandler(nil))
			}

			if tt.prepare != nil {
				tt.prepare(t, man, s)
			}

			s.Start()

			man.Client = &acme.Client{DirectoryURL: s.URL()}

			tlscert, err := man.GetCertificate(tt.hello)
			if tt.expectError != "" {
				if err == nil {
					t.Fatal("expected error, got certificate")
				}
				if !strings.Contains(err.Error(), tt.expectError) {
					t.Errorf("got %q, expected %q", err, tt.expectError)
				}
				return
			}
			if err != nil {
				t.Fatalf("man.GetCertificate: %v", err)
			}

			leaf, err := x509.ParseCertificate(tlscert.Certificate[0])
			if err != nil {
				t.Fatal(err)
			}
			opts := x509.VerifyOptions{
				DNSName:       tt.domain,
				Intermediates: x509.NewCertPool(),
				Roots:         s.Roots(),
			}
			for _, cert := range tlscert.Certificate[1:] {
				c, err := x509.ParseCertificate(cert)
				if err != nil {
					t.Fatal(err)
				}
				opts.Intermediates.AddCert(c)
			}
			if _, err := leaf.Verify(opts); err != nil {
				t.Error(err)
			}

			if san := leaf.DNSNames[0]; san != tt.domain {
				t.Errorf("got SAN %q, expected %q", san, tt.domain)
			}

			if tt.verify != nil {
				tt.verify(t, man, leaf)
			}
		})
	}
}

func TestGetCertificate_failedAttempt(t *testing.T) {
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusBadRequest)
	}))
	defer ts.Close()

	d := createCertRetryAfter
	f := testDidRemoveState
	defer func() {
		createCertRetryAfter = d
		testDidRemoveState = f
	}()
	createCertRetryAfter = 0
	done := make(chan struct{})
	testDidRemoveState = func(ck certKey) {
		if ck != exampleCertKey {
			t.Errorf("testDidRemoveState: domain = %v; want %v", ck, exampleCertKey)
		}
		close(done)
	}

	man := &Manager{
		Prompt: AcceptTOS,
		Client: &acme.Client{
			DirectoryURL: ts.URL,
		},
	}
	defer man.stopRenew()
	hello := clientHelloInfo(exampleDomain, algECDSA)
	if _, err := man.GetCertificate(hello); err == nil {
		t.Error("GetCertificate: err is nil")
	}

	<-done
	man.stateMu.Lock()
	defer man.stateMu.Unlock()
	if v, exist := man.state[exampleCertKey]; exist {
		t.Errorf("state exists for %v: %+v", exampleCertKey, v)
	}
}

func TestRevokeFailedAuthz(t *testing.T) {
	ca := acmetest.NewCAServer(t)
	// Make the authz unfulfillable on the client side, so it will be left
	// pending at the end of the verification attempt.
	ca.ChallengeTypes("fake-01", "fake-02")
	ca.Start()

	m := testManager(t)
	m.Client = &acme.Client{DirectoryURL: ca.URL()}

	_, err := m.GetCertificate(clientHelloInfo("example.org", algECDSA))
	if err == nil {
		t.Fatal("expected GetCertificate to fail")
	}

	logTicker := time.NewTicker(3 * time.Second)
	defer logTicker.Stop()
	for {
		authz, err := m.Client.GetAuthorization(context.Background(), ca.URL()+"/authz/0")
		if err != nil {
			t.Fatal(err)
		}
		if authz.Status == acme.StatusDeactivated {
			return
		}

		select {
		case <-logTicker.C:
			t.Logf("still waiting on revocations")
		default:
		}
		time.Sleep(50 * time.Millisecond)
	}
}

func TestHTTPHandlerDefaultFallback(t *testing.T) {
	tt := []struct {
		method, url  string
		wantCode     int
		wantLocation string
	}{
		{"GET", "http://example.org", 302, "https://example.org/"},
		{"GET", "http://example.org/foo", 302, "https://example.org/foo"},
		{"GET", "http://example.org/foo/bar/", 302, "https://example.org/foo/bar/"},
		{"GET", "http://example.org/?a=b", 302, "https://example.org/?a=b"},
		{"GET", "http://example.org/foo?a=b", 302, "https://example.org/foo?a=b"},
		{"GET", "http://example.org:80/foo?a=b", 302, "https://example.org:443/foo?a=b"},
		{"GET", "http://example.org:80/foo%20bar", 302, "https://example.org:443/foo%20bar"},
		{"GET", "http://[2602:d1:xxxx::c60a]:1234", 302, "https://[2602:d1:xxxx::c60a]:443/"},
		{"GET", "http://[2602:d1:xxxx::c60a]", 302, "https://[2602:d1:xxxx::c60a]/"},
		{"GET", "http://[2602:d1:xxxx::c60a]/foo?a=b", 302, "https://[2602:d1:xxxx::c60a]/foo?a=b"},
		{"HEAD", "http://example.org", 302, "https://example.org/"},
		{"HEAD", "http://example.org/foo", 302, "https://example.org/foo"},
		{"HEAD", "http://example.org/foo/bar/", 302, "https://example.org/foo/bar/"},
		{"HEAD", "http://example.org/?a=b", 302, "https://example.org/?a=b"},
		{"HEAD", "http://example.org/foo?a=b", 302, "https://example.org/foo?a=b"},
		{"POST", "http://example.org", 400, ""},
		{"PUT", "http://example.org", 400, ""},
		{"GET", "http://example.org/.well-known/acme-challenge/x", 404, ""},
	}
	var m Manager
	h := m.HTTPHandler(nil)
	for i, test := range tt {
		r := httptest.NewRequest(test.method, test.url, nil)
		w := httptest.NewRecorder()
		h.ServeHTTP(w, r)
		if w.Code != test.wantCode {
			t.Errorf("%d: w.Code = %d; want %d", i, w.Code, test.wantCode)
			t.Errorf("%d: body: %s", i, w.Body.Bytes())
		}
		if v := w.Header().Get("Location"); v != test.wantLocation {
			t.Errorf("%d: Location = %q; want %q", i, v, test.wantLocation)
		}
	}
}

func TestAccountKeyCache(t *testing.T) {
	m := Manager{Cache: newMemCache(t)}
	ctx := context.Background()
	k1, err := m.accountKey(ctx)
	if err != nil {
		t.Fatal(err)
	}
	k2, err := m.accountKey(ctx)
	if err != nil {
		t.Fatal(err)
	}
	if !reflect.DeepEqual(k1, k2) {
		t.Errorf("account keys don't match: k1 = %#v; k2 = %#v", k1, k2)
	}
}

func TestCache(t *testing.T) {
	ecdsaKey, err := ecdsa.GenerateKey(elliptic.P256(), rand.Reader)
	if err != nil {
		t.Fatal(err)
	}
	cert, err := dummyCert(ecdsaKey.Public(), exampleDomain)
	if err != nil {
		t.Fatal(err)
	}
	ecdsaCert := &tls.Certificate{
		Certificate: [][]byte{cert},
		PrivateKey:  ecdsaKey,
	}

	rsaKey, err := rsa.GenerateKey(rand.Reader, 512)
	if err != nil {
		t.Fatal(err)
	}
	cert, err = dummyCert(rsaKey.Public(), exampleDomain)
	if err != nil {
		t.Fatal(err)
	}
	rsaCert := &tls.Certificate{
		Certificate: [][]byte{cert},
		PrivateKey:  rsaKey,
	}

	man := &Manager{Cache: newMemCache(t)}
	defer man.stopRenew()
	ctx := context.Background()

	if err := man.cachePut(ctx, exampleCertKey, ecdsaCert); err != nil {
		t.Fatalf("man.cachePut: %v", err)
	}
	if err := man.cachePut(ctx, exampleCertKeyRSA, rsaCert); err != nil {
		t.Fatalf("man.cachePut: %v", err)
	}

	res, err := man.cacheGet(ctx, exampleCertKey)
	if err != nil {
		t.Fatalf("man.cacheGet: %v", err)
	}
	if res == nil || !bytes.Equal(res.Certificate[0], ecdsaCert.Certificate[0]) {
		t.Errorf("man.cacheGet = %+v; want %+v", res, ecdsaCert)
	}

	res, err = man.cacheGet(ctx, exampleCertKeyRSA)
	if err != nil {
		t.Fatalf("man.cacheGet: %v", err)
	}
	if res == nil || !bytes.Equal(res.Certificate[0], rsaCert.Certificate[0]) {
		t.Errorf("man.cacheGet = %+v; want %+v", res, rsaCert)
	}
}

func TestHostWhitelist(t *testing.T) {
	policy := HostWhitelist("example.com", "EXAMPLE.ORG", "*.example.net", "éÉ.com")
	tt := []struct {
		host  string
		allow bool
	}{
		{"example.com", true},
		{"example.org", true},
		{"xn--9caa.com", true}, // éé.com
		{"one.example.com", false},
		{"two.example.org", false},
		{"three.example.net", false},
		{"dummy", false},
	}
	for i, test := range tt {
		err := policy(nil, test.host)
		if err != nil && test.allow {
			t.Errorf("%d: policy(%q): %v; want nil", i, test.host, err)
		}
		if err == nil && !test.allow {
			t.Errorf("%d: policy(%q): nil; want an error", i, test.host)
		}
	}
}

func TestValidCert(t *testing.T) {
	key1, err := ecdsa.GenerateKey(elliptic.P256(), rand.Reader)
	if err != nil {
		t.Fatal(err)
	}
	key2, err := ecdsa.GenerateKey(elliptic.P256(), rand.Reader)
	if err != nil {
		t.Fatal(err)
	}
	key3, err := rsa.GenerateKey(rand.Reader, 512)
	if err != nil {
		t.Fatal(err)
	}
	cert1, err := dummyCert(key1.Public(), "example.org")
	if err != nil {
		t.Fatal(err)
	}
	cert2, err := dummyCert(key2.Public(), "example.org")
	if err != nil {
		t.Fatal(err)
	}
	cert3, err := dummyCert(key3.Public(), "example.org")
	if err != nil {
		t.Fatal(err)
	}
	now := time.Now()
	early, err := dateDummyCert(key1.Public(), now.Add(time.Hour), now.Add(2*time.Hour), "example.org")
	if err != nil {
		t.Fatal(err)
	}
	expired, err := dateDummyCert(key1.Public(), now.Add(-2*time.Hour), now.Add(-time.Hour), "example.org")
	if err != nil {
		t.Fatal(err)
	}

	tt := []struct {
		ck   certKey
		key  crypto.Signer
		cert [][]byte
		ok   bool
	}{
		{certKey{domain: "example.org"}, key1, [][]byte{cert1}, true},
		{certKey{domain: "example.org", isRSA: true}, key3, [][]byte{cert3}, true},
		{certKey{domain: "example.org"}, key1, [][]byte{cert1, cert2, cert3}, true},
		{certKey{domain: "example.org"}, key1, [][]byte{cert1, {1}}, false},
		{certKey{domain: "example.org"}, key1, [][]byte{{1}}, false},
		{certKey{domain: "example.org"}, key1, [][]byte{cert2}, false},
		{certKey{domain: "example.org"}, key2, [][]byte{cert1}, false},
		{certKey{domain: "example.org"}, key1, [][]byte{cert3}, false},
		{certKey{domain: "example.org"}, key3, [][]byte{cert1}, false},
		{certKey{domain: "example.net"}, key1, [][]byte{cert1}, false},
		{certKey{domain: "example.org"}, key1, [][]byte{early}, false},
		{certKey{domain: "example.org"}, key1, [][]byte{expired}, false},
		{certKey{domain: "example.org", isRSA: true}, key1, [][]byte{cert1}, false},
		{certKey{domain: "example.org"}, key3, [][]byte{cert3}, false},
	}
	for i, test := range tt {
		leaf, err := validCert(test.ck, test.cert, test.key, now)
		if err != nil && test.ok {
			t.Errorf("%d: err = %v", i, err)
		}
		if err == nil && !test.ok {
			t.Errorf("%d: err is nil", i)
		}
		if err == nil && test.ok && leaf == nil {
			t.Errorf("%d: leaf is nil", i)
		}
	}
}

type cacheGetFunc func(ctx context.Context, key string) ([]byte, error)

func (f cacheGetFunc) Get(ctx context.Context, key string) ([]byte, error) {
	return f(ctx, key)
}

func (f cacheGetFunc) Put(ctx context.Context, key string, data []byte) error {
	return fmt.Errorf("unsupported Put of %q = %q", key, data)
}

func (f cacheGetFunc) Delete(ctx context.Context, key string) error {
	return fmt.Errorf("unsupported Delete of %q", key)
}

func TestManagerGetCertificateBogusSNI(t *testing.T) {
	m := Manager{
		Prompt: AcceptTOS,
		Cache: cacheGetFunc(func(ctx context.Context, key string) ([]byte, error) {
			return nil, fmt.Errorf("cache.Get of %s", key)
		}),
	}
	tests := []struct {
		name    string
		wantErr string
	}{
		{"foo.com", "cache.Get of foo.com"},
		{"foo.com.", "cache.Get of foo.com"},
		{`a\b.com`, "acme/autocert: server name contains invalid character"},
		{`a/b.com`, "acme/autocert: server name contains invalid character"},
		{"", "acme/autocert: missing server name"},
		{"foo", "acme/autocert: server name component count invalid"},
		{".foo", "acme/autocert: server name component count invalid"},
		{"foo.", "acme/autocert: server name component count invalid"},
		{"fo.o", "cache.Get of fo.o"},
	}
	for _, tt := range tests {
		_, err := m.GetCertificate(clientHelloInfo(tt.name, algECDSA))
		got := fmt.Sprint(err)
		if got != tt.wantErr {
			t.Errorf("GetCertificate(SNI = %q) = %q; want %q", tt.name, got, tt.wantErr)
		}
	}
}

func TestCertRequest(t *testing.T) {
	key, err := ecdsa.GenerateKey(elliptic.P256(), rand.Reader)
	if err != nil {
		t.Fatal(err)
	}
	// An extension from RFC7633. Any will do.
	ext := pkix.Extension{
		Id:    asn1.ObjectIdentifier{1, 3, 6, 1, 5, 5, 7, 1},
		Value: []byte("dummy"),
	}
	b, err := certRequest(key, "example.org", []pkix.Extension{ext})
	if err != nil {
		t.Fatalf("certRequest: %v", err)
	}
	r, err := x509.ParseCertificateRequest(b)
	if err != nil {
		t.Fatalf("ParseCertificateRequest: %v", err)
	}
	var found bool
	for _, v := range r.Extensions {
		if v.Id.Equal(ext.Id) {
			found = true
			break
		}
	}
	if !found {
		t.Errorf("want %v in Extensions: %v", ext, r.Extensions)
	}
}

func TestSupportsECDSA(t *testing.T) {
	tests := []struct {
		CipherSuites     []uint16
		SignatureSchemes []tls.SignatureScheme
		SupportedCurves  []tls.CurveID
		ecdsaOk          bool
	}{
		{[]uint16{
			tls.TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256,
		}, nil, nil, false},
		{[]uint16{
			tls.TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256, tls.TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256,
		}, nil, nil, true},

		// SignatureSchemes limits, not extends, CipherSuites
		{[]uint16{
			tls.TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256,
		}, []tls.SignatureScheme{
			tls.PKCS1WithSHA256, tls.ECDSAWithP256AndSHA256,
		}, nil, false},
		{[]uint16{
			tls.TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256, tls.TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256,
		}, []tls.SignatureScheme{
			tls.PKCS1WithSHA256,
		}, nil, false},
		{[]uint16{
			tls.TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256, tls.TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256,
		}, []tls.SignatureScheme{
			tls.PKCS1WithSHA256, tls.ECDSAWithP256AndSHA256,
		}, nil, true},

		{[]uint16{
			tls.TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256, tls.TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256,
		}, []tls.SignatureScheme{
			tls.PKCS1WithSHA256, tls.ECDSAWithP256AndSHA256,
		}, []tls.CurveID{
			tls.CurveP521,
		}, false},
		{[]uint16{
			tls.TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256, tls.TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256,
		}, []tls.SignatureScheme{
			tls.PKCS1WithSHA256, tls.ECDSAWithP256AndSHA256,
		}, []tls.CurveID{
			tls.CurveP256,
			tls.CurveP521,
		}, true},
	}
	for i, tt := range tests {
		result := supportsECDSA(&tls.ClientHelloInfo{
			CipherSuites:     tt.CipherSuites,
			SignatureSchemes: tt.SignatureSchemes,
			SupportedCurves:  tt.SupportedCurves,
		})
		if result != tt.ecdsaOk {
			t.Errorf("%d: supportsECDSA = %v; want %v", i, result, tt.ecdsaOk)
		}
	}
}

func TestEndToEndALPN(t *testing.T) {
	const domain = "example.org"

	// ACME CA server
	ca := acmetest.NewCAServer(t).Start()

	// User HTTPS server.
	m := &Manager{
		Prompt: AcceptTOS,
		Client: &acme.Client{DirectoryURL: ca.URL()},
	}
	us := httptest.NewUnstartedServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Write([]byte("OK"))
	}))
	us.TLS = &tls.Config{
		NextProtos: []string{"http/1.1", acme.ALPNProto},
		GetCertificate: func(hello *tls.ClientHelloInfo) (*tls.Certificate, error) {
			cert, err := m.GetCertificate(hello)
			if err != nil {
				t.Errorf("m.GetCertificate: %v", err)
			}
			return cert, err
		},
	}
	us.StartTLS()
	defer us.Close()
	// In TLS-ALPN challenge verification, CA connects to the domain:443 in question.
	// Because the domain won't resolve in tests, we need to tell the CA
	// where to dial to instead.
	ca.Resolve(domain, strings.TrimPrefix(us.URL, "https://"))

	// A client visiting user's HTTPS server.
	tr := &http.Transport{
		TLSClientConfig: &tls.Config{
			RootCAs:    ca.Roots(),
			ServerName: domain,
		},
	}
	client := &http.Client{Transport: tr}
	res, err := client.Get(us.URL)
	if err != nil {
		t.Fatal(err)
	}
	defer res.Body.Close()
	b, err := io.ReadAll(res.Body)
	if err != nil {
		t.Fatal(err)
	}
	if v := string(b); v != "OK" {
		t.Errorf("user server response: %q; want 'OK'", v)
	}
}

func TestEndToEndHTTP(t *testing.T) {
	const domain = "example.org"

	// ACME CA server.
	ca := acmetest.NewCAServer(t).ChallengeTypes("http-01").Start()

	// User HTTP server for the ACME challenge.
	m := testManager(t)
	m.Client = &acme.Client{DirectoryURL: ca.URL()}
	s := httptest.NewServer(m.HTTPHandler(nil))
	defer s.Close()

	// User HTTPS server.
	ss := httptest.NewUnstartedServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Write([]byte("OK"))
	}))
	ss.TLS = &tls.Config{
		NextProtos: []string{"http/1.1", acme.ALPNProto},
		GetCertificate: func(hello *tls.ClientHelloInfo) (*tls.Certificate, error) {
			cert, err := m.GetCertificate(hello)
			if err != nil {
				t.Errorf("m.GetCertificate: %v", err)
			}
			return cert, err
		},
	}
	ss.StartTLS()
	defer ss.Close()

	// Redirect the CA requests to the HTTP server.
	ca.Resolve(domain, strings.TrimPrefix(s.URL, "http://"))

	// A client visiting user's HTTPS server.
	tr := &http.Transport{
		TLSClientConfig: &tls.Config{
			RootCAs:    ca.Roots(),
			ServerName: domain,
		},
	}
	client := &http.Client{Transport: tr}
	res, err := client.Get(ss.URL)
	if err != nil {
		t.Fatal(err)
	}
	defer res.Body.Close()
	b, err := io.ReadAll(res.Body)
	if err != nil {
		t.Fatal(err)
	}
	if v := string(b); v != "OK" {
		t.Errorf("user server response: %q; want 'OK'", v)
	}
}
