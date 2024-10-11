// Copyright 2018 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build darwin

// Command macOS-roots-test runs crypto/x509.TestSystemRoots as a
// stand-alone binary for crowdsourced testing.
package main

import (
	"crypto/x509"
	"fmt"
	"log"
	"os"
	"os/exec"
	"time"
	"unsafe"
)

type CertPool struct {
	bySubjectKeyId map[string][]int
	byName         map[string][]int
	certs          []*x509.Certificate
}

func (s *CertPool) contains(cert *x509.Certificate) bool {
	if s == nil {
		return false
	}

	candidates := s.byName[string(cert.RawSubject)]
	for _, c := range candidates {
		if s.certs[c].Equal(cert) {
			return true
		}
	}

	return false
}

func main() {
	var failed bool

	t0 := time.Now()
	sysRootsExt, err := loadSystemRoots() // actual system roots
	sysRootsDuration := time.Since(t0)

	if err != nil {
		log.Fatalf("failed to read system roots (cgo): %v", err)
	}
	sysRoots := (*CertPool)(unsafe.Pointer(sysRootsExt))

	t1 := time.Now()
	execRootsExt, err := execSecurityRoots() // non-cgo roots
	execSysRootsDuration := time.Since(t1)

	if err != nil {
		log.Fatalf("failed to read system roots (nocgo): %v", err)
	}
	execRoots := (*CertPool)(unsafe.Pointer(execRootsExt))

	fmt.Printf("    cgo sys roots: %v\n", sysRootsDuration)
	fmt.Printf("non-cgo sys roots: %v\n", execSysRootsDuration)

	// On Mavericks, there are 212 bundled certs, at least there was at
	// one point in time on one machine. (Maybe it was a corp laptop
	// with extra certs?) Other OS X users report 135, 142, 145...
	// Let's try requiring at least 100, since this is just a sanity
	// check.
	if want, have := 100, len(sysRoots.certs); have < want {
		failed = true
		fmt.Printf("want at least %d system roots, have %d\n", want, have)
	}

	// Check that the two cert pools are the same.
	sysPool := make(map[string]*x509.Certificate, len(sysRoots.certs))
	for _, c := range sysRoots.certs {
		sysPool[string(c.Raw)] = c
	}
	for _, c := range execRoots.certs {
		if _, ok := sysPool[string(c.Raw)]; ok {
			delete(sysPool, string(c.Raw))
		} else {
			// verify-cert lets in certificates that are not trusted roots, but are
			// signed by trusted roots. This should not be a problem, so confirm that's
			// the case and skip them.
			if _, err := c.Verify(x509.VerifyOptions{
				Roots:         sysRootsExt,
				Intermediates: execRootsExt, // the intermediates for EAP certs are stored in the keychain
				KeyUsages:     []x509.ExtKeyUsage{x509.ExtKeyUsageAny},
			}); err != nil {
				failed = true
				fmt.Printf("certificate only present in non-cgo pool: %v (verify error: %v)\n", c.Subject, err)
			} else {
				fmt.Printf("signed certificate only present in non-cgo pool (acceptable): %v\n", c.Subject)
			}
		}
	}
	for _, c := range sysPool {
		failed = true
		fmt.Printf("certificate only present in cgo pool: %v\n", c.Subject)
	}

	if failed && debugDarwinRoots {
		cmd := exec.Command("security", "dump-trust-settings")
		cmd.Stdout = os.Stdout
		cmd.Stderr = os.Stderr
		cmd.Run()
		cmd = exec.Command("security", "dump-trust-settings", "-d")
		cmd.Stdout = os.Stdout
		cmd.Stderr = os.Stderr
		cmd.Run()
	}

	if failed {
		fmt.Printf("\n\n!!! The test failed!\n\nPlease report *the whole output* at https://github.com/golang/go/issues/24652 wrapping it in ``` a code block ```\nThank you!\n")
	} else {
		fmt.Printf("\n\nThe test passed, no need to report the output. Thank you.\n")
	}
}
