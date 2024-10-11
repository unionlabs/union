// Copyright 2016 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package autocert

import (
	"context"
	"crypto"
	"crypto/ecdsa"
	"testing"
	"time"

	"golang.org/x/crypto/acme"
	"golang.org/x/crypto/acme/autocert/internal/acmetest"
)

func TestRenewalNext(t *testing.T) {
	now := time.Now()
	man := &Manager{
		RenewBefore: 7 * 24 * time.Hour,
		nowFunc:     func() time.Time { return now },
	}
	defer man.stopRenew()
	tt := []struct {
		expiry   time.Time
		min, max time.Duration
	}{
		{now.Add(90 * 24 * time.Hour), 83*24*time.Hour - renewJitter, 83 * 24 * time.Hour},
		{now.Add(time.Hour), 0, 1},
		{now, 0, 1},
		{now.Add(-time.Hour), 0, 1},
	}

	dr := &domainRenewal{m: man}
	for i, test := range tt {
		next := dr.next(test.expiry)
		if next < test.min || test.max < next {
			t.Errorf("%d: next = %v; want between %v and %v", i, next, test.min, test.max)
		}
	}
}

func TestRenewFromCache(t *testing.T) {
	man := testManager(t)
	man.RenewBefore = 24 * time.Hour

	ca := acmetest.NewCAServer(t).Start()
	ca.ResolveGetCertificate(exampleDomain, man.GetCertificate)

	man.Client = &acme.Client{
		DirectoryURL: ca.URL(),
	}

	// cache an almost expired cert
	now := time.Now()
	c := ca.LeafCert(exampleDomain, "ECDSA", now.Add(-2*time.Hour), now.Add(time.Minute))
	if err := man.cachePut(context.Background(), exampleCertKey, c); err != nil {
		t.Fatal(err)
	}

	// verify the renewal happened
	defer func() {
		// Stop the timers that read and execute testDidRenewLoop before restoring it.
		// Otherwise the timer callback may race with the deferred write.
		man.stopRenew()
		testDidRenewLoop = func(next time.Duration, err error) {}
	}()
	renewed := make(chan bool, 1)
	testDidRenewLoop = func(next time.Duration, err error) {
		defer func() {
			select {
			case renewed <- true:
			default:
				// The renewal timer uses a random backoff. If the first renewal fails for
				// some reason, we could end up with multiple calls here before the test
				// stops the timer.
			}
		}()

		if err != nil {
			t.Errorf("testDidRenewLoop: %v", err)
		}
		// Next should be about 90 days:
		// CaServer creates 90days expiry + account for man.RenewBefore.
		// Previous expiration was within 1 min.
		future := 88 * 24 * time.Hour
		if next < future {
			t.Errorf("testDidRenewLoop: next = %v; want >= %v", next, future)
		}

		// ensure the new cert is cached
		after := time.Now().Add(future)
		tlscert, err := man.cacheGet(context.Background(), exampleCertKey)
		if err != nil {
			t.Errorf("man.cacheGet: %v", err)
			return
		}
		if !tlscert.Leaf.NotAfter.After(after) {
			t.Errorf("cache leaf.NotAfter = %v; want > %v", tlscert.Leaf.NotAfter, after)
		}

		// verify the old cert is also replaced in memory
		man.stateMu.Lock()
		defer man.stateMu.Unlock()
		s := man.state[exampleCertKey]
		if s == nil {
			t.Errorf("m.state[%q] is nil", exampleCertKey)
			return
		}
		tlscert, err = s.tlscert()
		if err != nil {
			t.Errorf("s.tlscert: %v", err)
			return
		}
		if !tlscert.Leaf.NotAfter.After(after) {
			t.Errorf("state leaf.NotAfter = %v; want > %v", tlscert.Leaf.NotAfter, after)
		}
	}

	// trigger renew
	hello := clientHelloInfo(exampleDomain, algECDSA)
	if _, err := man.GetCertificate(hello); err != nil {
		t.Fatal(err)
	}
	<-renewed
}

func TestRenewFromCacheAlreadyRenewed(t *testing.T) {
	ca := acmetest.NewCAServer(t).Start()
	man := testManager(t)
	man.RenewBefore = 24 * time.Hour
	man.Client = &acme.Client{
		DirectoryURL: "invalid",
	}

	// cache a recently renewed cert with a different private key
	now := time.Now()
	newCert := ca.LeafCert(exampleDomain, "ECDSA", now.Add(-2*time.Hour), now.Add(time.Hour*24*90))
	if err := man.cachePut(context.Background(), exampleCertKey, newCert); err != nil {
		t.Fatal(err)
	}
	newLeaf, err := validCert(exampleCertKey, newCert.Certificate, newCert.PrivateKey.(crypto.Signer), now)
	if err != nil {
		t.Fatal(err)
	}

	// set internal state to an almost expired cert
	oldCert := ca.LeafCert(exampleDomain, "ECDSA", now.Add(-2*time.Hour), now.Add(time.Minute))
	if err != nil {
		t.Fatal(err)
	}
	oldLeaf, err := validCert(exampleCertKey, oldCert.Certificate, oldCert.PrivateKey.(crypto.Signer), now)
	if err != nil {
		t.Fatal(err)
	}
	man.stateMu.Lock()
	if man.state == nil {
		man.state = make(map[certKey]*certState)
	}
	s := &certState{
		key:  oldCert.PrivateKey.(crypto.Signer),
		cert: oldCert.Certificate,
		leaf: oldLeaf,
	}
	man.state[exampleCertKey] = s
	man.stateMu.Unlock()

	// verify the renewal accepted the newer cached cert
	defer func() {
		// Stop the timers that read and execute testDidRenewLoop before restoring it.
		// Otherwise the timer callback may race with the deferred write.
		man.stopRenew()
		testDidRenewLoop = func(next time.Duration, err error) {}
	}()
	renewed := make(chan bool, 1)
	testDidRenewLoop = func(next time.Duration, err error) {
		defer func() {
			select {
			case renewed <- true:
			default:
				// The renewal timer uses a random backoff. If the first renewal fails for
				// some reason, we could end up with multiple calls here before the test
				// stops the timer.
			}
		}()

		if err != nil {
			t.Errorf("testDidRenewLoop: %v", err)
		}
		// Next should be about 90 days
		// Previous expiration was within 1 min.
		future := 88 * 24 * time.Hour
		if next < future {
			t.Errorf("testDidRenewLoop: next = %v; want >= %v", next, future)
		}

		// ensure the cached cert was not modified
		tlscert, err := man.cacheGet(context.Background(), exampleCertKey)
		if err != nil {
			t.Errorf("man.cacheGet: %v", err)
			return
		}
		if !tlscert.Leaf.NotAfter.Equal(newLeaf.NotAfter) {
			t.Errorf("cache leaf.NotAfter = %v; want == %v", tlscert.Leaf.NotAfter, newLeaf.NotAfter)
		}

		// verify the old cert is also replaced in memory
		man.stateMu.Lock()
		defer man.stateMu.Unlock()
		s := man.state[exampleCertKey]
		if s == nil {
			t.Errorf("m.state[%q] is nil", exampleCertKey)
			return
		}
		stateKey := s.key.Public().(*ecdsa.PublicKey)
		if !stateKey.Equal(newLeaf.PublicKey) {
			t.Error("state key was not updated from cache")
			return
		}
		tlscert, err = s.tlscert()
		if err != nil {
			t.Errorf("s.tlscert: %v", err)
			return
		}
		if !tlscert.Leaf.NotAfter.Equal(newLeaf.NotAfter) {
			t.Errorf("state leaf.NotAfter = %v; want == %v", tlscert.Leaf.NotAfter, newLeaf.NotAfter)
		}
	}

	// assert the expiring cert is returned from state
	hello := clientHelloInfo(exampleDomain, algECDSA)
	tlscert, err := man.GetCertificate(hello)
	if err != nil {
		t.Fatal(err)
	}
	if !oldLeaf.NotAfter.Equal(tlscert.Leaf.NotAfter) {
		t.Errorf("state leaf.NotAfter = %v; want == %v", tlscert.Leaf.NotAfter, oldLeaf.NotAfter)
	}

	// trigger renew
	man.startRenew(exampleCertKey, s.key, s.leaf.NotAfter)
	<-renewed
	func() {
		man.renewalMu.Lock()
		defer man.renewalMu.Unlock()

		// verify the private key is replaced in the renewal state
		r := man.renewal[exampleCertKey]
		if r == nil {
			t.Errorf("m.renewal[%q] is nil", exampleCertKey)
			return
		}
		renewalKey := r.key.Public().(*ecdsa.PublicKey)
		if !renewalKey.Equal(newLeaf.PublicKey) {
			t.Error("renewal private key was not updated from cache")
		}
	}()

	// assert the new cert is returned from state after renew
	hello = clientHelloInfo(exampleDomain, algECDSA)
	tlscert, err = man.GetCertificate(hello)
	if err != nil {
		t.Fatal(err)
	}
	if !newLeaf.NotAfter.Equal(tlscert.Leaf.NotAfter) {
		t.Errorf("state leaf.NotAfter = %v; want == %v", tlscert.Leaf.NotAfter, newLeaf.NotAfter)
	}
}
