// Copyright 2018 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

// Package acmetest provides types for testing acme and autocert packages.
//
// TODO: Consider moving this to x/crypto/acme/internal/acmetest for acme tests as well.
package acmetest

import (
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
	"encoding/base64"
	"encoding/json"
	"encoding/pem"
	"fmt"
	"io"
	"math/big"
	"net"
	"net/http"
	"net/http/httptest"
	"path"
	"strconv"
	"strings"
	"sync"
	"testing"
	"time"

	"golang.org/x/crypto/acme"
)

// CAServer is a simple test server which implements ACME spec bits needed for testing.
type CAServer struct {
	rootKey      crypto.Signer
	rootCert     []byte // DER encoding
	rootTemplate *x509.Certificate

	t              *testing.T
	server         *httptest.Server
	issuer         pkix.Name
	challengeTypes []string
	url            string
	roots          *x509.CertPool
	eabRequired    bool

	mu             sync.Mutex
	certCount      int                           // number of issued certs
	acctRegistered bool                          // set once an account has been registered
	domainAddr     map[string]string             // domain name to addr:port resolution
	domainGetCert  map[string]getCertificateFunc // domain name to GetCertificate function
	domainHandler  map[string]http.Handler       // domain name to Handle function
	validAuthz     map[string]*authorization     // valid authz, keyed by domain name
	authorizations []*authorization              // all authz, index is used as ID
	orders         []*order                      // index is used as order ID
	errors         []error                       // encountered client errors
}

type getCertificateFunc func(hello *tls.ClientHelloInfo) (*tls.Certificate, error)

// NewCAServer creates a new ACME test server. The returned CAServer issues
// certs signed with the CA roots available in the Roots field.
func NewCAServer(t *testing.T) *CAServer {
	ca := &CAServer{t: t,
		challengeTypes: []string{"fake-01", "tls-alpn-01", "http-01"},
		domainAddr:     make(map[string]string),
		domainGetCert:  make(map[string]getCertificateFunc),
		domainHandler:  make(map[string]http.Handler),
		validAuthz:     make(map[string]*authorization),
	}

	ca.server = httptest.NewUnstartedServer(http.HandlerFunc(ca.handle))

	r, err := rand.Int(rand.Reader, big.NewInt(1000000))
	if err != nil {
		panic(fmt.Sprintf("rand.Int: %v", err))
	}
	ca.issuer = pkix.Name{
		Organization: []string{"Test Acme Co"},
		CommonName:   "Root CA " + r.String(),
	}

	return ca
}

func (ca *CAServer) generateRoot() {
	key, err := ecdsa.GenerateKey(elliptic.P256(), rand.Reader)
	if err != nil {
		panic(fmt.Sprintf("ecdsa.GenerateKey: %v", err))
	}
	tmpl := &x509.Certificate{
		SerialNumber:          big.NewInt(1),
		Subject:               ca.issuer,
		NotBefore:             time.Now(),
		NotAfter:              time.Now().Add(365 * 24 * time.Hour),
		KeyUsage:              x509.KeyUsageCertSign,
		BasicConstraintsValid: true,
		IsCA:                  true,
	}
	der, err := x509.CreateCertificate(rand.Reader, tmpl, tmpl, &key.PublicKey, key)
	if err != nil {
		panic(fmt.Sprintf("x509.CreateCertificate: %v", err))
	}
	cert, err := x509.ParseCertificate(der)
	if err != nil {
		panic(fmt.Sprintf("x509.ParseCertificate: %v", err))
	}
	ca.roots = x509.NewCertPool()
	ca.roots.AddCert(cert)
	ca.rootKey = key
	ca.rootCert = der
	ca.rootTemplate = tmpl
}

// IssuerName sets the name of the issuing CA.
func (ca *CAServer) IssuerName(name pkix.Name) *CAServer {
	if ca.url != "" {
		panic("IssuerName must be called before Start")
	}
	ca.issuer = name
	return ca
}

// ChallengeTypes sets the supported challenge types.
func (ca *CAServer) ChallengeTypes(types ...string) *CAServer {
	if ca.url != "" {
		panic("ChallengeTypes must be called before Start")
	}
	ca.challengeTypes = types
	return ca
}

// URL returns the server address, after Start has been called.
func (ca *CAServer) URL() string {
	if ca.url == "" {
		panic("URL called before Start")
	}
	return ca.url
}

// Roots returns a pool cointaining the CA root.
func (ca *CAServer) Roots() *x509.CertPool {
	if ca.url == "" {
		panic("Roots called before Start")
	}
	return ca.roots
}

// ExternalAccountRequired makes an EAB JWS required for account registration.
func (ca *CAServer) ExternalAccountRequired() *CAServer {
	if ca.url != "" {
		panic("ExternalAccountRequired must be called before Start")
	}
	ca.eabRequired = true
	return ca
}

// Start starts serving requests. The server address becomes available in the
// URL field.
func (ca *CAServer) Start() *CAServer {
	if ca.url == "" {
		ca.generateRoot()
		ca.server.Start()
		ca.t.Cleanup(ca.server.Close)
		ca.url = ca.server.URL
	}
	return ca
}

func (ca *CAServer) serverURL(format string, arg ...interface{}) string {
	return ca.server.URL + fmt.Sprintf(format, arg...)
}

func (ca *CAServer) addr(domain string) (string, bool) {
	ca.mu.Lock()
	defer ca.mu.Unlock()
	addr, ok := ca.domainAddr[domain]
	return addr, ok
}

func (ca *CAServer) getCert(domain string) (getCertificateFunc, bool) {
	ca.mu.Lock()
	defer ca.mu.Unlock()
	f, ok := ca.domainGetCert[domain]
	return f, ok
}

func (ca *CAServer) getHandler(domain string) (http.Handler, bool) {
	ca.mu.Lock()
	defer ca.mu.Unlock()
	h, ok := ca.domainHandler[domain]
	return h, ok
}

func (ca *CAServer) httpErrorf(w http.ResponseWriter, code int, format string, a ...interface{}) {
	s := fmt.Sprintf(format, a...)
	ca.t.Errorf(format, a...)
	http.Error(w, s, code)
}

// Resolve adds a domain to address resolution for the ca to dial to
// when validating challenges for the domain authorization.
func (ca *CAServer) Resolve(domain, addr string) {
	ca.mu.Lock()
	defer ca.mu.Unlock()
	ca.domainAddr[domain] = addr
}

// ResolveGetCertificate redirects TLS connections for domain to f when
// validating challenges for the domain authorization.
func (ca *CAServer) ResolveGetCertificate(domain string, f getCertificateFunc) {
	ca.mu.Lock()
	defer ca.mu.Unlock()
	ca.domainGetCert[domain] = f
}

// ResolveHandler redirects HTTP requests for domain to f when
// validating challenges for the domain authorization.
func (ca *CAServer) ResolveHandler(domain string, h http.Handler) {
	ca.mu.Lock()
	defer ca.mu.Unlock()
	ca.domainHandler[domain] = h
}

type discovery struct {
	NewNonce   string `json:"newNonce"`
	NewAccount string `json:"newAccount"`
	NewOrder   string `json:"newOrder"`
	NewAuthz   string `json:"newAuthz"`

	Meta discoveryMeta `json:"meta,omitempty"`
}

type discoveryMeta struct {
	ExternalAccountRequired bool `json:"externalAccountRequired,omitempty"`
}

type challenge struct {
	URI   string `json:"uri"`
	Type  string `json:"type"`
	Token string `json:"token"`
}

type authorization struct {
	Status     string      `json:"status"`
	Challenges []challenge `json:"challenges"`

	domain string
	id     int
}

type order struct {
	Status      string   `json:"status"`
	AuthzURLs   []string `json:"authorizations"`
	FinalizeURL string   `json:"finalize"`    // CSR submit URL
	CertURL     string   `json:"certificate"` // already issued cert

	leaf []byte // issued cert in DER format
}

func (ca *CAServer) handle(w http.ResponseWriter, r *http.Request) {
	ca.t.Logf("%s %s", r.Method, r.URL)
	w.Header().Set("Replay-Nonce", "nonce")
	// TODO: Verify nonce header for all POST requests.

	switch {
	default:
		ca.httpErrorf(w, http.StatusBadRequest, "unrecognized r.URL.Path: %s", r.URL.Path)

	// Discovery request.
	case r.URL.Path == "/":
		resp := &discovery{
			NewNonce:   ca.serverURL("/new-nonce"),
			NewAccount: ca.serverURL("/new-account"),
			NewOrder:   ca.serverURL("/new-order"),
			Meta: discoveryMeta{
				ExternalAccountRequired: ca.eabRequired,
			},
		}
		if err := json.NewEncoder(w).Encode(resp); err != nil {
			panic(fmt.Sprintf("discovery response: %v", err))
		}

	// Nonce requests.
	case r.URL.Path == "/new-nonce":
		// Nonce values are always set. Nothing else to do.
		return

	// Client key registration request.
	case r.URL.Path == "/new-account":
		ca.mu.Lock()
		defer ca.mu.Unlock()
		if ca.acctRegistered {
			ca.httpErrorf(w, http.StatusServiceUnavailable, "multiple accounts are not implemented")
			return
		}
		ca.acctRegistered = true

		var req struct {
			ExternalAccountBinding json.RawMessage
		}

		if err := decodePayload(&req, r.Body); err != nil {
			ca.httpErrorf(w, http.StatusBadRequest, "%v", err)
			return
		}

		if ca.eabRequired && len(req.ExternalAccountBinding) == 0 {
			ca.httpErrorf(w, http.StatusBadRequest, "registration failed: no JWS for EAB")
			return
		}

		// TODO: Check the user account key against a ca.accountKeys?
		w.Header().Set("Location", ca.serverURL("/accounts/1"))
		w.WriteHeader(http.StatusCreated)
		w.Write([]byte("{}"))

	// New order request.
	case r.URL.Path == "/new-order":
		var req struct {
			Identifiers []struct{ Value string }
		}
		if err := decodePayload(&req, r.Body); err != nil {
			ca.httpErrorf(w, http.StatusBadRequest, "%v", err)
			return
		}
		ca.mu.Lock()
		defer ca.mu.Unlock()
		o := &order{Status: acme.StatusPending}
		for _, id := range req.Identifiers {
			z := ca.authz(id.Value)
			o.AuthzURLs = append(o.AuthzURLs, ca.serverURL("/authz/%d", z.id))
		}
		orderID := len(ca.orders)
		ca.orders = append(ca.orders, o)
		w.Header().Set("Location", ca.serverURL("/orders/%d", orderID))
		w.WriteHeader(http.StatusCreated)
		if err := json.NewEncoder(w).Encode(o); err != nil {
			panic(err)
		}

	// Existing order status requests.
	case strings.HasPrefix(r.URL.Path, "/orders/"):
		ca.mu.Lock()
		defer ca.mu.Unlock()
		o, err := ca.storedOrder(strings.TrimPrefix(r.URL.Path, "/orders/"))
		if err != nil {
			ca.httpErrorf(w, http.StatusBadRequest, "%v", err)
			return
		}
		if err := json.NewEncoder(w).Encode(o); err != nil {
			panic(err)
		}

	// Accept challenge requests.
	case strings.HasPrefix(r.URL.Path, "/challenge/"):
		parts := strings.Split(r.URL.Path, "/")
		typ, id := parts[len(parts)-2], parts[len(parts)-1]
		ca.mu.Lock()
		supported := false
		for _, suppTyp := range ca.challengeTypes {
			if suppTyp == typ {
				supported = true
			}
		}
		a, err := ca.storedAuthz(id)
		ca.mu.Unlock()
		if !supported {
			ca.httpErrorf(w, http.StatusBadRequest, "unsupported challenge: %v", typ)
			return
		}
		if err != nil {
			ca.httpErrorf(w, http.StatusBadRequest, "challenge accept: %v", err)
			return
		}
		ca.validateChallenge(a, typ)
		w.Write([]byte("{}"))

	// Get authorization status requests.
	case strings.HasPrefix(r.URL.Path, "/authz/"):
		var req struct{ Status string }
		decodePayload(&req, r.Body)
		deactivate := req.Status == "deactivated"
		ca.mu.Lock()
		defer ca.mu.Unlock()
		authz, err := ca.storedAuthz(strings.TrimPrefix(r.URL.Path, "/authz/"))
		if err != nil {
			ca.httpErrorf(w, http.StatusNotFound, "%v", err)
			return
		}
		if deactivate {
			// Note we don't invalidate authorized orders as we should.
			authz.Status = "deactivated"
			ca.t.Logf("authz %d is now %s", authz.id, authz.Status)
			ca.updatePendingOrders()
		}
		if err := json.NewEncoder(w).Encode(authz); err != nil {
			panic(fmt.Sprintf("encoding authz %d: %v", authz.id, err))
		}

	// Certificate issuance request.
	case strings.HasPrefix(r.URL.Path, "/new-cert/"):
		ca.mu.Lock()
		defer ca.mu.Unlock()
		orderID := strings.TrimPrefix(r.URL.Path, "/new-cert/")
		o, err := ca.storedOrder(orderID)
		if err != nil {
			ca.httpErrorf(w, http.StatusBadRequest, "%v", err)
			return
		}
		if o.Status != acme.StatusReady {
			ca.httpErrorf(w, http.StatusForbidden, "order status: %s", o.Status)
			return
		}
		// Validate CSR request.
		var req struct {
			CSR string `json:"csr"`
		}
		decodePayload(&req, r.Body)
		b, _ := base64.RawURLEncoding.DecodeString(req.CSR)
		csr, err := x509.ParseCertificateRequest(b)
		if err != nil {
			ca.httpErrorf(w, http.StatusBadRequest, "%v", err)
			return
		}
		// Issue the certificate.
		der, err := ca.leafCert(csr)
		if err != nil {
			ca.httpErrorf(w, http.StatusBadRequest, "new-cert response: ca.leafCert: %v", err)
			return
		}
		o.leaf = der
		o.CertURL = ca.serverURL("/issued-cert/%s", orderID)
		o.Status = acme.StatusValid
		if err := json.NewEncoder(w).Encode(o); err != nil {
			panic(err)
		}

	// Already issued cert download requests.
	case strings.HasPrefix(r.URL.Path, "/issued-cert/"):
		ca.mu.Lock()
		defer ca.mu.Unlock()
		o, err := ca.storedOrder(strings.TrimPrefix(r.URL.Path, "/issued-cert/"))
		if err != nil {
			ca.httpErrorf(w, http.StatusBadRequest, "%v", err)
			return
		}
		if o.Status != acme.StatusValid {
			ca.httpErrorf(w, http.StatusForbidden, "order status: %s", o.Status)
			return
		}
		w.Header().Set("Content-Type", "application/pem-certificate-chain")
		pem.Encode(w, &pem.Block{Type: "CERTIFICATE", Bytes: o.leaf})
		pem.Encode(w, &pem.Block{Type: "CERTIFICATE", Bytes: ca.rootCert})
	}
}

// storedOrder retrieves a previously created order at index i.
// It requires ca.mu to be locked.
func (ca *CAServer) storedOrder(i string) (*order, error) {
	idx, err := strconv.Atoi(i)
	if err != nil {
		return nil, fmt.Errorf("storedOrder: %v", err)
	}
	if idx < 0 {
		return nil, fmt.Errorf("storedOrder: invalid order index %d", idx)
	}
	if idx > len(ca.orders)-1 {
		return nil, fmt.Errorf("storedOrder: no such order %d", idx)
	}

	ca.updatePendingOrders()
	return ca.orders[idx], nil
}

// storedAuthz retrieves a previously created authz at index i.
// It requires ca.mu to be locked.
func (ca *CAServer) storedAuthz(i string) (*authorization, error) {
	idx, err := strconv.Atoi(i)
	if err != nil {
		return nil, fmt.Errorf("storedAuthz: %v", err)
	}
	if idx < 0 {
		return nil, fmt.Errorf("storedAuthz: invalid authz index %d", idx)
	}
	if idx > len(ca.authorizations)-1 {
		return nil, fmt.Errorf("storedAuthz: no such authz %d", idx)
	}
	return ca.authorizations[idx], nil
}

// authz returns an existing valid authorization for the identifier or creates a
// new one. It requires ca.mu to be locked.
func (ca *CAServer) authz(identifier string) *authorization {
	authz, ok := ca.validAuthz[identifier]
	if !ok {
		authzId := len(ca.authorizations)
		authz = &authorization{
			id:     authzId,
			domain: identifier,
			Status: acme.StatusPending,
		}
		for _, typ := range ca.challengeTypes {
			authz.Challenges = append(authz.Challenges, challenge{
				Type:  typ,
				URI:   ca.serverURL("/challenge/%s/%d", typ, authzId),
				Token: challengeToken(authz.domain, typ, authzId),
			})
		}
		ca.authorizations = append(ca.authorizations, authz)
	}
	return authz
}

// leafCert issues a new certificate.
// It requires ca.mu to be locked.
func (ca *CAServer) leafCert(csr *x509.CertificateRequest) (der []byte, err error) {
	ca.certCount++ // next leaf cert serial number
	leaf := &x509.Certificate{
		SerialNumber:          big.NewInt(int64(ca.certCount)),
		Subject:               pkix.Name{Organization: []string{"Test Acme Co"}},
		NotBefore:             time.Now(),
		NotAfter:              time.Now().Add(90 * 24 * time.Hour),
		KeyUsage:              x509.KeyUsageDigitalSignature | x509.KeyUsageKeyEncipherment,
		ExtKeyUsage:           []x509.ExtKeyUsage{x509.ExtKeyUsageServerAuth},
		DNSNames:              csr.DNSNames,
		BasicConstraintsValid: true,
	}
	if len(csr.DNSNames) == 0 {
		leaf.DNSNames = []string{csr.Subject.CommonName}
	}
	return x509.CreateCertificate(rand.Reader, leaf, ca.rootTemplate, csr.PublicKey, ca.rootKey)
}

// LeafCert issues a leaf certificate.
func (ca *CAServer) LeafCert(name, keyType string, notBefore, notAfter time.Time) *tls.Certificate {
	if ca.url == "" {
		panic("LeafCert called before Start")
	}

	ca.mu.Lock()
	defer ca.mu.Unlock()
	var pk crypto.Signer
	switch keyType {
	case "RSA":
		var err error
		pk, err = rsa.GenerateKey(rand.Reader, 1024)
		if err != nil {
			ca.t.Fatal(err)
		}
	case "ECDSA":
		var err error
		pk, err = ecdsa.GenerateKey(elliptic.P256(), rand.Reader)
		if err != nil {
			ca.t.Fatal(err)
		}
	default:
		panic("LeafCert: unknown key type")
	}
	ca.certCount++ // next leaf cert serial number
	leaf := &x509.Certificate{
		SerialNumber:          big.NewInt(int64(ca.certCount)),
		Subject:               pkix.Name{Organization: []string{"Test Acme Co"}},
		NotBefore:             notBefore,
		NotAfter:              notAfter,
		KeyUsage:              x509.KeyUsageDigitalSignature | x509.KeyUsageKeyEncipherment,
		ExtKeyUsage:           []x509.ExtKeyUsage{x509.ExtKeyUsageServerAuth},
		DNSNames:              []string{name},
		BasicConstraintsValid: true,
	}
	der, err := x509.CreateCertificate(rand.Reader, leaf, ca.rootTemplate, pk.Public(), ca.rootKey)
	if err != nil {
		ca.t.Fatal(err)
	}
	return &tls.Certificate{
		Certificate: [][]byte{der},
		PrivateKey:  pk,
	}
}

func (ca *CAServer) validateChallenge(authz *authorization, typ string) {
	var err error
	switch typ {
	case "tls-alpn-01":
		err = ca.verifyALPNChallenge(authz)
	case "http-01":
		err = ca.verifyHTTPChallenge(authz)
	default:
		panic(fmt.Sprintf("validation of %q is not implemented", typ))
	}
	ca.mu.Lock()
	defer ca.mu.Unlock()
	if err != nil {
		authz.Status = "invalid"
	} else {
		authz.Status = "valid"
		ca.validAuthz[authz.domain] = authz
	}
	ca.t.Logf("validated %q for %q, err: %v", typ, authz.domain, err)
	ca.t.Logf("authz %d is now %s", authz.id, authz.Status)

	ca.updatePendingOrders()
}

func (ca *CAServer) updatePendingOrders() {
	// Update all pending orders.
	// An order becomes "ready" if all authorizations are "valid".
	// An order becomes "invalid" if any authorization is "invalid".
	// Status changes: https://tools.ietf.org/html/rfc8555#section-7.1.6
	for i, o := range ca.orders {
		if o.Status != acme.StatusPending {
			continue
		}

		countValid, countInvalid := ca.validateAuthzURLs(o.AuthzURLs, i)
		if countInvalid > 0 {
			o.Status = acme.StatusInvalid
			ca.t.Logf("order %d is now invalid", i)
			continue
		}
		if countValid == len(o.AuthzURLs) {
			o.Status = acme.StatusReady
			o.FinalizeURL = ca.serverURL("/new-cert/%d", i)
			ca.t.Logf("order %d is now ready", i)
		}
	}
}

func (ca *CAServer) validateAuthzURLs(urls []string, orderNum int) (countValid, countInvalid int) {
	for _, zurl := range urls {
		z, err := ca.storedAuthz(path.Base(zurl))
		if err != nil {
			ca.t.Logf("no authz %q for order %d", zurl, orderNum)
			continue
		}
		if z.Status == acme.StatusInvalid {
			countInvalid++
		}
		if z.Status == acme.StatusValid {
			countValid++
		}
	}
	return countValid, countInvalid
}

func (ca *CAServer) verifyALPNChallenge(a *authorization) error {
	const acmeALPNProto = "acme-tls/1"

	addr, haveAddr := ca.addr(a.domain)
	getCert, haveGetCert := ca.getCert(a.domain)
	if !haveAddr && !haveGetCert {
		return fmt.Errorf("no resolution information for %q", a.domain)
	}
	if haveAddr && haveGetCert {
		return fmt.Errorf("overlapping resolution information for %q", a.domain)
	}

	var crt *x509.Certificate
	switch {
	case haveAddr:
		conn, err := tls.Dial("tcp", addr, &tls.Config{
			ServerName:         a.domain,
			InsecureSkipVerify: true,
			NextProtos:         []string{acmeALPNProto},
			MinVersion:         tls.VersionTLS12,
		})
		if err != nil {
			return err
		}
		if v := conn.ConnectionState().NegotiatedProtocol; v != acmeALPNProto {
			return fmt.Errorf("CAServer: verifyALPNChallenge: negotiated proto is %q; want %q", v, acmeALPNProto)
		}
		if n := len(conn.ConnectionState().PeerCertificates); n != 1 {
			return fmt.Errorf("len(PeerCertificates) = %d; want 1", n)
		}
		crt = conn.ConnectionState().PeerCertificates[0]
	case haveGetCert:
		hello := &tls.ClientHelloInfo{
			ServerName: a.domain,
			// TODO: support selecting ECDSA.
			CipherSuites:      []uint16{tls.TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305},
			SupportedProtos:   []string{acme.ALPNProto},
			SupportedVersions: []uint16{tls.VersionTLS12},
		}
		c, err := getCert(hello)
		if err != nil {
			return err
		}
		crt, err = x509.ParseCertificate(c.Certificate[0])
		if err != nil {
			return err
		}
	}

	if err := crt.VerifyHostname(a.domain); err != nil {
		return fmt.Errorf("verifyALPNChallenge: VerifyHostname: %v", err)
	}
	// See RFC 8737, Section 6.1.
	oid := asn1.ObjectIdentifier{1, 3, 6, 1, 5, 5, 7, 1, 31}
	for _, x := range crt.Extensions {
		if x.Id.Equal(oid) {
			// TODO: check the token.
			return nil
		}
	}
	return fmt.Errorf("verifyTokenCert: no id-pe-acmeIdentifier extension found")
}

func (ca *CAServer) verifyHTTPChallenge(a *authorization) error {
	addr, haveAddr := ca.addr(a.domain)
	handler, haveHandler := ca.getHandler(a.domain)
	if !haveAddr && !haveHandler {
		return fmt.Errorf("no resolution information for %q", a.domain)
	}
	if haveAddr && haveHandler {
		return fmt.Errorf("overlapping resolution information for %q", a.domain)
	}

	token := challengeToken(a.domain, "http-01", a.id)
	path := "/.well-known/acme-challenge/" + token

	var body string
	switch {
	case haveAddr:
		t := &http.Transport{
			DialContext: func(ctx context.Context, network, _ string) (net.Conn, error) {
				return (&net.Dialer{}).DialContext(ctx, network, addr)
			},
		}
		req, err := http.NewRequest("GET", "http://"+a.domain+path, nil)
		if err != nil {
			return err
		}
		res, err := t.RoundTrip(req)
		if err != nil {
			return err
		}
		if res.StatusCode != http.StatusOK {
			return fmt.Errorf("http token: w.Code = %d; want %d", res.StatusCode, http.StatusOK)
		}
		b, err := io.ReadAll(res.Body)
		if err != nil {
			return err
		}
		body = string(b)
	case haveHandler:
		r := httptest.NewRequest("GET", path, nil)
		r.Host = a.domain
		w := httptest.NewRecorder()
		handler.ServeHTTP(w, r)
		if w.Code != http.StatusOK {
			return fmt.Errorf("http token: w.Code = %d; want %d", w.Code, http.StatusOK)
		}
		body = w.Body.String()
	}

	if !strings.HasPrefix(body, token) {
		return fmt.Errorf("http token value = %q; want 'token-http-01.' prefix", body)
	}
	return nil
}

func decodePayload(v interface{}, r io.Reader) error {
	var req struct{ Payload string }
	if err := json.NewDecoder(r).Decode(&req); err != nil {
		return err
	}
	payload, err := base64.RawURLEncoding.DecodeString(req.Payload)
	if err != nil {
		return err
	}
	return json.Unmarshal(payload, v)
}

func challengeToken(domain, challType string, authzID int) string {
	return fmt.Sprintf("token-%s-%s-%d", domain, challType, authzID)
}

func unique(a []string) []string {
	seen := make(map[string]bool)
	var res []string
	for _, s := range a {
		if s != "" && !seen[s] {
			seen[s] = true
			res = append(res, s)
		}
	}
	return res
}
