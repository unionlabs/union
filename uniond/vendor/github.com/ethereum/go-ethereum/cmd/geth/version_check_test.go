// Copyright 2020 The go-ethereum Authors
// This file is part of go-ethereum.
//
// go-ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// go-ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with go-ethereum. If not, see <http://www.gnu.org/licenses/>.

package main

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"regexp"
	"strconv"
	"strings"
	"testing"

	"github.com/jedisct1/go-minisign"
)

func TestVerification(t *testing.T) {
	// Signatures generated with `minisign`. Legacy format, not pre-hashed file.
	t.Run("minisig-legacy", func(t *testing.T) {
		// For this test, the pubkey is in testdata/vcheck/minisign.pub
		// (the privkey is `minisign.sec`, if we want to expand this test. Password 'test' )
		pub := "RWQkliYstQBOKOdtClfgC3IypIPX6TAmoEi7beZ4gyR3wsaezvqOMWsp"
		testVerification(t, pub, "./testdata/vcheck/minisig-sigs/")
	})
	t.Run("minisig-new", func(t *testing.T) {
		// For this test, the pubkey is in testdata/vcheck/minisign.pub
		// (the privkey is `minisign.sec`, if we want to expand this test. Password 'test' )
		// `minisign -S -s ./minisign.sec  -m data.json  -x ./minisig-sigs-new/data.json.minisig`
		pub := "RWQkliYstQBOKOdtClfgC3IypIPX6TAmoEi7beZ4gyR3wsaezvqOMWsp"
		testVerification(t, pub, "./testdata/vcheck/minisig-sigs-new/")
	})
	// Signatures generated with `signify-openbsd`
	t.Run("signify-openbsd", func(t *testing.T) {
		t.Skip("This currently fails, minisign expects 4 lines of data, signify provides only 2")
		// For this test, the pubkey is in testdata/vcheck/signifykey.pub
		// (the privkey is `signifykey.sec`, if we want to expand this test. Password 'test' )
		pub := "RWSKLNhZb0KdATtRT7mZC/bybI3t3+Hv/O2i3ye04Dq9fnT9slpZ1a2/"
		testVerification(t, pub, "./testdata/vcheck/signify-sigs/")
	})
}

func testVerification(t *testing.T, pubkey, sigdir string) {
	// Data to verify
	data, err := os.ReadFile("./testdata/vcheck/data.json")
	if err != nil {
		t.Fatal(err)
	}
	// Signatures, with and without comments, both trusted and untrusted
	files, err := os.ReadDir(sigdir)
	if err != nil {
		t.Fatal(err)
	}
	if len(files) == 0 {
		t.Fatal("Missing tests")
	}
	for _, f := range files {
		sig, err := os.ReadFile(filepath.Join(sigdir, f.Name()))
		if err != nil {
			t.Fatal(err)
		}
		err = verifySignature([]string{pubkey}, data, sig)
		if err != nil {
			t.Fatal(err)
		}
	}
}

func versionUint(v string) int {
	mustInt := func(s string) int {
		a, err := strconv.Atoi(s)
		if err != nil {
			panic(v)
		}
		return a
	}
	components := strings.Split(strings.TrimPrefix(v, "v"), ".")
	a := mustInt(components[0])
	b := mustInt(components[1])
	c := mustInt(components[2])
	return a*100*100 + b*100 + c
}

// TestMatching can be used to check that the regexps are correct
func TestMatching(t *testing.T) {
	data, _ := os.ReadFile("./testdata/vcheck/vulnerabilities.json")
	var vulns []vulnJson
	if err := json.Unmarshal(data, &vulns); err != nil {
		t.Fatal(err)
	}
	check := func(version string) {
		vFull := fmt.Sprintf("Geth/%v-unstable-15339cf1-20201204/linux-amd64/go1.15.4", version)
		for _, vuln := range vulns {
			r, err := regexp.Compile(vuln.Check)
			vulnIntro := versionUint(vuln.Introduced)
			vulnFixed := versionUint(vuln.Fixed)
			current := versionUint(version)
			if err != nil {
				t.Fatal(err)
			}
			if vuln.Name == "Denial of service due to Go CVE-2020-28362" {
				// this one is not tied to geth-versions
				continue
			}
			if vulnIntro <= current && vulnFixed > current {
				// Should be vulnerable
				if !r.MatchString(vFull) {
					t.Errorf("Should be vulnerable, version %v, intro: %v, fixed: %v %v %v",
						version, vuln.Introduced, vuln.Fixed, vuln.Name, vuln.Check)
				}
			} else {
				if r.MatchString(vFull) {
					t.Errorf("Should not be flagged vulnerable, version %v, intro: %v, fixed: %v %v %d %d %d",
						version, vuln.Introduced, vuln.Fixed, vuln.Name, vulnIntro, current, vulnFixed)
				}
			}
		}
	}
	for major := 1; major < 2; major++ {
		for minor := 0; minor < 30; minor++ {
			for patch := 0; patch < 30; patch++ {
				vShort := fmt.Sprintf("v%d.%d.%d", major, minor, patch)
				check(vShort)
			}
		}
	}
}

func TestGethPubKeysParseable(t *testing.T) {
	for _, pubkey := range gethPubKeys {
		_, err := minisign.NewPublicKey(pubkey)
		if err != nil {
			t.Errorf("Should be parseable")
		}
	}
}

func TestKeyID(t *testing.T) {
	type args struct {
		id [8]byte
	}
	tests := []struct {
		name string
		args args
		want string
	}{
		{"@holiman key", args{id: extractKeyId(gethPubKeys[0])}, "FB1D084D39BAEC24"},
		{"second key", args{id: extractKeyId(gethPubKeys[1])}, "138B1CA303E51687"},
		{"third key", args{id: extractKeyId(gethPubKeys[2])}, "FD9813B2D2098484"},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := keyID(tt.args.id); got != tt.want {
				t.Errorf("keyID() = %v, want %v", got, tt.want)
			}
		})
	}
}

func extractKeyId(pubkey string) [8]byte {
	p, _ := minisign.NewPublicKey(pubkey)
	return p.KeyId
}
