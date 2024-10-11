// Copyright 2013 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package main

import (
	"bytes"
	"crypto/x509"
	"encoding/pem"
	"fmt"
	"os"
	"os/exec"
	"os/user"
	"path/filepath"
	"sync"
)

var debugDarwinRoots = true

// This code is only used when compiling without cgo.
// It is here, instead of root_nocgo_darwin.go, so that tests can check it
// even if the tests are run with cgo enabled.
// The linker will not include these unused functions in binaries built with cgo enabled.

// execSecurityRoots finds the macOS list of trusted root certificates
// using only command-line tools. This is our fallback path when cgo isn't available.
//
// The strategy is as follows:
//
//  1. Run "security find-certificate" to dump the list of system root
//     CAs in PEM format.
//
//  2. For each dumped cert, conditionally verify it with "security
//     verify-cert" if that cert was not in the SystemRootCertificates
//     keychain, which can't have custom trust policies.
//
// We need to run "verify-cert" for all certificates not in SystemRootCertificates
// because there might be certificates in the keychains without a corresponding
// trust entry, in which case the logic is complicated (see root_cgo_darwin.go).
//
// TODO: actually parse the "trust-settings-export" output and apply the full
// logic. See Issue 26830.
func execSecurityRoots() (*x509.CertPool, error) {
	keychains := []string{"/Library/Keychains/System.keychain"}

	// Note that this results in trusting roots from $HOME/... (the environment
	// variable), which might not be expected.
	u, err := user.Current()
	if err != nil {
		if debugDarwinRoots {
			fmt.Printf("crypto/x509: get current user: %v\n", err)
		}
	} else {
		keychains = append(keychains,
			filepath.Join(u.HomeDir, "/Library/Keychains/login.keychain"),

			// Fresh installs of Sierra use a slightly different path for the login keychain
			filepath.Join(u.HomeDir, "/Library/Keychains/login.keychain-db"),
		)
	}

	var (
		mu          sync.Mutex
		roots       = x509.NewCertPool()
		numVerified int // number of execs of 'security verify-cert', for debug stats
		wg          sync.WaitGroup
		verifyCh    = make(chan *x509.Certificate)
	)

	// Using 4 goroutines to pipe into verify-cert seems to be
	// about the best we can do. The verify-cert binary seems to
	// just RPC to another server with coarse locking anyway, so
	// running 16 at a time for instance doesn't help at all.
	for i := 0; i < 4; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			for cert := range verifyCh {
				valid := verifyCertWithSystem(cert)

				mu.Lock()
				numVerified++
				if valid {
					roots.AddCert(cert)
				}
				mu.Unlock()
			}
		}()
	}
	err = forEachCertInKeychains(keychains, func(cert *x509.Certificate) {
		verifyCh <- cert
	})
	if err != nil {
		return nil, err
	}
	close(verifyCh)
	wg.Wait()

	if debugDarwinRoots {
		fmt.Printf("crypto/x509: ran security verify-cert %d times\n", numVerified)
	}

	err = forEachCertInKeychains([]string{
		"/System/Library/Keychains/SystemRootCertificates.keychain",
	}, roots.AddCert)
	if err != nil {
		return nil, err
	}

	return roots, nil
}

func forEachCertInKeychains(paths []string, f func(*x509.Certificate)) error {
	args := append([]string{"find-certificate", "-a", "-p"}, paths...)
	cmd := exec.Command("/usr/bin/security", args...)
	data, err := cmd.Output()
	if err != nil {
		return err
	}
	for len(data) > 0 {
		var block *pem.Block
		block, data = pem.Decode(data)
		if block == nil {
			break
		}
		if block.Type != "CERTIFICATE" || len(block.Headers) != 0 {
			continue
		}
		cert, err := x509.ParseCertificate(block.Bytes)
		if err != nil {
			continue
		}
		f(cert)
	}
	return nil
}

func verifyCertWithSystem(cert *x509.Certificate) bool {
	data := pem.EncodeToMemory(&pem.Block{
		Type: "CERTIFICATE", Bytes: cert.Raw,
	})

	f, err := os.CreateTemp("", "cert")
	if err != nil {
		fmt.Fprintf(os.Stderr, "can't create temporary file for cert: %v", err)
		return false
	}
	defer os.Remove(f.Name())
	if _, err := f.Write(data); err != nil {
		fmt.Fprintf(os.Stderr, "can't write temporary file for cert: %v", err)
		return false
	}
	if err := f.Close(); err != nil {
		fmt.Fprintf(os.Stderr, "can't write temporary file for cert: %v", err)
		return false
	}
	cmd := exec.Command("/usr/bin/security", "verify-cert", "-p", "ssl", "-c", f.Name(), "-l", "-L")
	var stderr bytes.Buffer
	if debugDarwinRoots {
		cmd.Stderr = &stderr
	}
	if err := cmd.Run(); err != nil {
		if debugDarwinRoots {
			fmt.Printf("crypto/x509: verify-cert rejected %s: %q\n", cert.Subject, bytes.TrimSpace(stderr.Bytes()))
		}
		return false
	}
	if debugDarwinRoots {
		fmt.Printf("crypto/x509: verify-cert approved %s\n", cert.Subject)
	}
	return true
}
