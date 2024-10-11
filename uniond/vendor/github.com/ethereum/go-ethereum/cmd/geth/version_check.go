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
	"errors"
	"fmt"
	"io"
	"net/http"
	"os"
	"regexp"
	"strings"

	"github.com/ethereum/go-ethereum/log"
	"github.com/jedisct1/go-minisign"
	"github.com/urfave/cli/v2"
)

var gethPubKeys []string = []string{
	//@holiman, minisign public key FB1D084D39BAEC24
	"RWQk7Lo5TQgd+wxBNZM+Zoy+7UhhMHaWKzqoes9tvSbFLJYZhNTbrIjx",
	//minisign public key 138B1CA303E51687
	"RWSHFuUDoxyLEzjszuWZI1xStS66QTyXFFZG18uDfO26CuCsbckX1e9J",
	//minisign public key FD9813B2D2098484
	"RWSEhAnSshOY/b+GmaiDkObbCWefsAoavjoLcPjBo1xn71yuOH5I+Lts",
}

type vulnJson struct {
	Name        string
	Uid         string
	Summary     string
	Description string
	Links       []string
	Introduced  string
	Fixed       string
	Published   string
	Severity    string
	Check       string
	CVE         string
}

func versionCheck(ctx *cli.Context) error {
	url := ctx.String(VersionCheckUrlFlag.Name)
	version := ctx.String(VersionCheckVersionFlag.Name)
	log.Info("Checking vulnerabilities", "version", version, "url", url)
	return checkCurrent(url, version)
}

func checkCurrent(url, current string) error {
	var (
		data []byte
		sig  []byte
		err  error
	)
	if data, err = fetch(url); err != nil {
		return fmt.Errorf("could not retrieve data: %w", err)
	}
	if sig, err = fetch(fmt.Sprintf("%v.minisig", url)); err != nil {
		return fmt.Errorf("could not retrieve signature: %w", err)
	}
	if err = verifySignature(gethPubKeys, data, sig); err != nil {
		return err
	}
	var vulns []vulnJson
	if err = json.Unmarshal(data, &vulns); err != nil {
		return err
	}
	allOk := true
	for _, vuln := range vulns {
		r, err := regexp.Compile(vuln.Check)
		if err != nil {
			return err
		}
		if r.MatchString(current) {
			allOk = false
			fmt.Printf("## Vulnerable to %v (%v)\n\n", vuln.Uid, vuln.Name)
			fmt.Printf("Severity: %v\n", vuln.Severity)
			fmt.Printf("Summary : %v\n", vuln.Summary)
			fmt.Printf("Fixed in: %v\n", vuln.Fixed)
			if len(vuln.CVE) > 0 {
				fmt.Printf("CVE: %v\n", vuln.CVE)
			}
			if len(vuln.Links) > 0 {
				fmt.Printf("References:\n")
				for _, ref := range vuln.Links {
					fmt.Printf("\t- %v\n", ref)
				}
			}
			fmt.Println()
		}
	}
	if allOk {
		fmt.Println("No vulnerabilities found")
	}
	return nil
}

// fetch makes an HTTP request to the given url and returns the response body
func fetch(url string) ([]byte, error) {
	if filep := strings.TrimPrefix(url, "file://"); filep != url {
		return os.ReadFile(filep)
	}
	res, err := http.Get(url)
	if err != nil {
		return nil, err
	}
	defer res.Body.Close()
	body, err := io.ReadAll(res.Body)
	if err != nil {
		return nil, err
	}
	return body, nil
}

// verifySignature checks that the sigData is a valid signature of the given
// data, for pubkey GethPubkey
func verifySignature(pubkeys []string, data, sigdata []byte) error {
	sig, err := minisign.DecodeSignature(string(sigdata))
	if err != nil {
		return err
	}
	// find the used key
	var key *minisign.PublicKey
	for _, pubkey := range pubkeys {
		pub, err := minisign.NewPublicKey(pubkey)
		if err != nil {
			// our pubkeys should be parseable
			return err
		}
		if pub.KeyId != sig.KeyId {
			continue
		}
		key = &pub
		break
	}
	if key == nil {
		log.Info("Signing key not trusted", "keyid", keyID(sig.KeyId), "error", err)
		return errors.New("signature could not be verified")
	}
	if ok, err := key.Verify(data, sig); !ok || err != nil {
		log.Info("Verification failed error", "keyid", keyID(key.KeyId), "error", err)
		return errors.New("signature could not be verified")
	}
	return nil
}

// keyID turns a binary minisign key ID into a hex string.
// Note: key IDs are printed in reverse byte order.
func keyID(id [8]byte) string {
	var rev [8]byte
	for i := range id {
		rev[len(rev)-1-i] = id[i]
	}
	return fmt.Sprintf("%X", rev)
}
