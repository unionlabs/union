/*
 *
 * Copyright 2020 gRPC authors.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

package certprovider

import (
	"context"
	"crypto/tls"
	"crypto/x509"
	"errors"
	"fmt"
	"os"
	"testing"
	"time"

	"google.golang.org/grpc/internal/grpctest"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/testdata"
)

const (
	fakeProvider1Name       = "fake-certificate-provider-1"
	fakeProvider2Name       = "fake-certificate-provider-2"
	fakeConfig              = "my fake config"
	defaultTestTimeout      = 5 * time.Second
	defaultTestShortTimeout = 10 * time.Millisecond
)

var fpb1, fpb2 *fakeProviderBuilder

func init() {
	fpb1 = &fakeProviderBuilder{
		name:         fakeProvider1Name,
		providerChan: testutils.NewChannel(),
	}
	fpb2 = &fakeProviderBuilder{
		name:         fakeProvider2Name,
		providerChan: testutils.NewChannel(),
	}
	Register(fpb1)
	Register(fpb2)
}

type s struct {
	grpctest.Tester
}

func Test(t *testing.T) {
	grpctest.RunSubTests(t, s{})
}

// fakeProviderBuilder builds new instances of fakeProvider and interprets the
// config provided to it as a string.
type fakeProviderBuilder struct {
	name         string
	providerChan *testutils.Channel
}

func (b *fakeProviderBuilder) ParseConfig(config any) (*BuildableConfig, error) {
	s, ok := config.(string)
	if !ok {
		return nil, fmt.Errorf("providerBuilder %s received config of type %T, want string", b.name, config)
	}
	return NewBuildableConfig(b.name, []byte(s), func(BuildOptions) Provider {
		fp := &fakeProvider{
			Distributor: NewDistributor(),
			config:      s,
		}
		b.providerChan.Send(fp)
		return fp
	}), nil
}

func (b *fakeProviderBuilder) Name() string {
	return b.name
}

// fakeProvider is an implementation of the Provider interface which provides a
// method for tests to invoke to push new key materials.
type fakeProvider struct {
	*Distributor
	config string
}

func (p *fakeProvider) Start(BuildOptions) Provider {
	// This is practically a no-op since this provider doesn't do any work which
	// needs to be started at this point.
	return p
}

// newKeyMaterial allows tests to push new key material to the fake provider
// which will be made available to users of this provider.
func (p *fakeProvider) newKeyMaterial(km *KeyMaterial, err error) {
	p.Distributor.Set(km, err)
}

// Close helps implement the Provider interface.
func (p *fakeProvider) Close() {
	p.Distributor.Stop()
}

// loadKeyMaterials is a helper to read cert/key files from testdata and convert
// them into a KeyMaterialReader struct.
func loadKeyMaterials(t *testing.T, cert, key, ca string) *KeyMaterial {
	t.Helper()

	certs, err := tls.LoadX509KeyPair(testdata.Path(cert), testdata.Path(key))
	if err != nil {
		t.Fatalf("Failed to load keyPair: %v", err)
	}

	pemData, err := os.ReadFile(testdata.Path(ca))
	if err != nil {
		t.Fatal(err)
	}
	roots := x509.NewCertPool()
	roots.AppendCertsFromPEM(pemData)
	return &KeyMaterial{Certs: []tls.Certificate{certs}, Roots: roots}
}

// kmReader wraps the KeyMaterial method exposed by Provider and Distributor
// implementations. Defining the interface here makes it possible to use the
// same helper from both provider and distributor tests.
type kmReader interface {
	KeyMaterial(context.Context) (*KeyMaterial, error)
}

// readAndVerifyKeyMaterial attempts to read key material from the given
// provider and compares it against the expected key material.
func readAndVerifyKeyMaterial(ctx context.Context, kmr kmReader, wantKM *KeyMaterial) error {
	gotKM, err := kmr.KeyMaterial(ctx)
	if err != nil {
		return fmt.Errorf("KeyMaterial(ctx) failed: %w", err)
	}
	return compareKeyMaterial(gotKM, wantKM)
}

func compareKeyMaterial(got, want *KeyMaterial) error {
	if len(got.Certs) != len(want.Certs) {
		return fmt.Errorf("keyMaterial certs = %+v, want %+v", got, want)
	}
	for i := 0; i < len(got.Certs); i++ {
		if !got.Certs[i].Leaf.Equal(want.Certs[i].Leaf) {
			return fmt.Errorf("keyMaterial certs = %+v, want %+v", got, want)
		}
	}

	if gotR, wantR := got.Roots, want.Roots; !gotR.Equal(wantR) {
		return fmt.Errorf("keyMaterial roots = %v, want %v", gotR, wantR)
	}

	return nil
}

func createProvider(t *testing.T, name, config string, opts BuildOptions) Provider {
	t.Helper()
	prov, err := GetProvider(name, config, opts)
	if err != nil {
		t.Fatalf("GetProvider(%s, %s, %v) failed: %v", name, config, opts, err)
	}
	return prov
}

// TestStoreSingleProvider creates a single provider through the store and calls
// methods on them.
func (s) TestStoreSingleProvider(t *testing.T) {
	prov := createProvider(t, fakeProvider1Name, fakeConfig, BuildOptions{CertName: "default"})
	defer prov.Close()

	// Our fakeProviderBuilder pushes newly created providers on a channel. Grab
	// the fake provider from that channel.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	p, err := fpb1.providerChan.Receive(ctx)
	if err != nil {
		t.Fatalf("Timeout when expecting certProvider %q to be created", fakeProvider1Name)
	}
	fakeProv := p.(*fakeProvider)

	// Attempt to read from key material from the Provider returned by the
	// store. This will fail because we have not pushed any key material into
	// our fake provider.
	sCtx, sCancel := context.WithTimeout(context.Background(), defaultTestShortTimeout)
	defer sCancel()
	if err := readAndVerifyKeyMaterial(sCtx, prov, nil); !errors.Is(err, context.DeadlineExceeded) {
		t.Fatal(err)
	}

	// Load key material from testdata directory, push it into out fakeProvider
	// and attempt to read from the Provider returned by the store.
	testKM1 := loadKeyMaterials(t, "x509/server1_cert.pem", "x509/server1_key.pem", "x509/client_ca_cert.pem")
	fakeProv.newKeyMaterial(testKM1, nil)
	if err := readAndVerifyKeyMaterial(ctx, prov, testKM1); err != nil {
		t.Fatal(err)
	}

	// Push new key material and read from the Provider. This should returned
	// updated key material.
	testKM2 := loadKeyMaterials(t, "x509/server2_cert.pem", "x509/server2_key.pem", "x509/client_ca_cert.pem")
	fakeProv.newKeyMaterial(testKM2, nil)
	if err := readAndVerifyKeyMaterial(ctx, prov, testKM2); err != nil {
		t.Fatal(err)
	}
}

// TestStoreSingleProviderSameConfigDifferentOpts creates multiple providers of
// same type, for same configs but different keyMaterial options through the
// store (and expects the store's sharing mechanism to kick in) and calls
// methods on them.
func (s) TestStoreSingleProviderSameConfigDifferentOpts(t *testing.T) {
	// Create three readers on the same fake provider. Two of these readers use
	// certName `foo`, while the third one uses certName `bar`.
	optsFoo := BuildOptions{CertName: "foo"}
	provFoo1 := createProvider(t, fakeProvider1Name, fakeConfig, optsFoo)
	provFoo2 := createProvider(t, fakeProvider1Name, fakeConfig, optsFoo)
	defer func() {
		provFoo1.Close()
		provFoo2.Close()
	}()

	// Our fakeProviderBuilder pushes newly created providers on a channel.
	// Grab the fake provider for optsFoo.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	p, err := fpb1.providerChan.Receive(ctx)
	if err != nil {
		t.Fatalf("Timeout when expecting certProvider %q to be created", fakeProvider1Name)
	}
	fakeProvFoo := p.(*fakeProvider)

	// Make sure only provider was created by the builder so far. The store
	// should be able to share the providers.
	sCtx, sCancel := context.WithTimeout(context.Background(), defaultTestShortTimeout)
	defer sCancel()
	if _, err := fpb1.providerChan.Receive(sCtx); !errors.Is(err, context.DeadlineExceeded) {
		t.Fatalf("A second provider created when expected to be shared by the store")
	}

	optsBar := BuildOptions{CertName: "bar"}
	provBar1 := createProvider(t, fakeProvider1Name, fakeConfig, optsBar)
	defer provBar1.Close()

	// Grab the fake provider for optsBar.
	p, err = fpb1.providerChan.Receive(ctx)
	if err != nil {
		t.Fatalf("Timeout when expecting certProvider %q to be created", fakeProvider1Name)
	}
	fakeProvBar := p.(*fakeProvider)

	// Push key material for optsFoo, and make sure the foo providers return
	// appropriate key material and the bar provider times out.
	fooKM := loadKeyMaterials(t, "x509/server1_cert.pem", "x509/server1_key.pem", "x509/client_ca_cert.pem")
	fakeProvFoo.newKeyMaterial(fooKM, nil)
	if err := readAndVerifyKeyMaterial(ctx, provFoo1, fooKM); err != nil {
		t.Fatal(err)
	}
	if err := readAndVerifyKeyMaterial(ctx, provFoo2, fooKM); err != nil {
		t.Fatal(err)
	}
	sCtx, sCancel = context.WithTimeout(context.Background(), defaultTestShortTimeout)
	defer sCancel()
	if err := readAndVerifyKeyMaterial(sCtx, provBar1, nil); !errors.Is(err, context.DeadlineExceeded) {
		t.Fatal(err)
	}

	// Push key material for optsBar, and make sure the bar provider returns
	// appropriate key material.
	barKM := loadKeyMaterials(t, "x509/server2_cert.pem", "x509/server2_key.pem", "x509/client_ca_cert.pem")
	fakeProvBar.newKeyMaterial(barKM, nil)
	if err := readAndVerifyKeyMaterial(ctx, provBar1, barKM); err != nil {
		t.Fatal(err)
	}

	// Make sure the above push of new key material does not affect foo readers.
	if err := readAndVerifyKeyMaterial(ctx, provFoo1, fooKM); err != nil {
		t.Fatal(err)
	}
}

// TestStoreSingleProviderDifferentConfigs creates multiple instances of the
// same type of provider through the store with different configs. The store
// would end up creating different provider instances for these and no sharing
// would take place.
func (s) TestStoreSingleProviderDifferentConfigs(t *testing.T) {
	// Create two providers of the same type, but with different configs.
	opts := BuildOptions{CertName: "foo"}
	cfg1 := fakeConfig + "1111"
	cfg2 := fakeConfig + "2222"

	prov1 := createProvider(t, fakeProvider1Name, cfg1, opts)
	defer prov1.Close()
	// Our fakeProviderBuilder pushes newly created providers on a channel. Grab
	// the fake provider from that channel.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	p1, err := fpb1.providerChan.Receive(ctx)
	if err != nil {
		t.Fatalf("Timeout when expecting certProvider %q to be created", fakeProvider1Name)
	}
	fakeProv1 := p1.(*fakeProvider)

	prov2 := createProvider(t, fakeProvider1Name, cfg2, opts)
	defer prov2.Close()
	// Grab the second provider from the channel.
	p2, err := fpb1.providerChan.Receive(ctx)
	if err != nil {
		t.Fatalf("Timeout when expecting certProvider %q to be created", fakeProvider1Name)
	}
	fakeProv2 := p2.(*fakeProvider)

	// Push the same key material into both fake providers and verify that the
	// providers returned by the store return the appropriate key material.
	km1 := loadKeyMaterials(t, "x509/server1_cert.pem", "x509/server1_key.pem", "x509/client_ca_cert.pem")
	fakeProv1.newKeyMaterial(km1, nil)
	fakeProv2.newKeyMaterial(km1, nil)
	if err := readAndVerifyKeyMaterial(ctx, prov1, km1); err != nil {
		t.Fatal(err)
	}
	if err := readAndVerifyKeyMaterial(ctx, prov2, km1); err != nil {
		t.Fatal(err)
	}

	// Push new key material into only one of the fake providers and verify
	// that the providers returned by the store return the appropriate key
	// material.
	km2 := loadKeyMaterials(t, "x509/server2_cert.pem", "x509/server2_key.pem", "x509/client_ca_cert.pem")
	fakeProv2.newKeyMaterial(km2, nil)
	if err := readAndVerifyKeyMaterial(ctx, prov1, km1); err != nil {
		t.Fatal(err)
	}
	if err := readAndVerifyKeyMaterial(ctx, prov2, km2); err != nil {
		t.Fatal(err)
	}

	// Close one of the providers and verify that the other one is not affected.
	prov1.Close()
	if err := readAndVerifyKeyMaterial(ctx, prov2, km2); err != nil {
		t.Fatal(err)
	}
}

// TestStoreMultipleProviders creates providers of different types and makes
// sure closing of one does not affect the other.
func (s) TestStoreMultipleProviders(t *testing.T) {
	opts := BuildOptions{CertName: "foo"}
	prov1 := createProvider(t, fakeProvider1Name, fakeConfig, opts)
	defer prov1.Close()
	// Our fakeProviderBuilder pushes newly created providers on a channel. Grab
	// the fake provider from that channel.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	p1, err := fpb1.providerChan.Receive(ctx)
	if err != nil {
		t.Fatalf("Timeout when expecting certProvider %q to be created", fakeProvider1Name)
	}
	fakeProv1 := p1.(*fakeProvider)

	prov2 := createProvider(t, fakeProvider2Name, fakeConfig, opts)
	defer prov2.Close()
	// Grab the second provider from the channel.
	p2, err := fpb2.providerChan.Receive(ctx)
	if err != nil {
		t.Fatalf("Timeout when expecting certProvider %q to be created", fakeProvider2Name)
	}
	fakeProv2 := p2.(*fakeProvider)

	// Push the key material into both providers and verify that the
	// readers return the appropriate key material.
	km1 := loadKeyMaterials(t, "x509/server1_cert.pem", "x509/server1_key.pem", "x509/client_ca_cert.pem")
	fakeProv1.newKeyMaterial(km1, nil)
	km2 := loadKeyMaterials(t, "x509/server2_cert.pem", "x509/server2_key.pem", "x509/client_ca_cert.pem")
	fakeProv2.newKeyMaterial(km2, nil)
	if err := readAndVerifyKeyMaterial(ctx, prov1, km1); err != nil {
		t.Fatal(err)
	}
	if err := readAndVerifyKeyMaterial(ctx, prov2, km2); err != nil {
		t.Fatal(err)
	}

	// Close one of the providers and verify that the other one is not affected.
	prov1.Close()
	if err := readAndVerifyKeyMaterial(ctx, prov2, km2); err != nil {
		t.Fatal(err)
	}
}
