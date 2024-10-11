// Copyright 2018 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

package storage

import (
	"bytes"
	"encoding/json"
	"fmt"
	"os"
	"testing"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/log"
	"github.com/mattn/go-colorable"
)

func TestEncryption(t *testing.T) {
	//	key := []byte("AES256Key-32Characters1234567890")
	//	plaintext := []byte(value)
	key := []byte("AES256Key-32Characters1234567890")
	plaintext := []byte("exampleplaintext")

	c, iv, err := encrypt(key, plaintext, nil)
	if err != nil {
		t.Fatal(err)
	}
	t.Logf("Ciphertext %x, nonce %x\n", c, iv)

	p, err := decrypt(key, iv, c, nil)
	if err != nil {
		t.Fatal(err)
	}
	t.Logf("Plaintext %v\n", string(p))
	if !bytes.Equal(plaintext, p) {
		t.Errorf("Failed: expected plaintext recovery, got %v expected %v", string(plaintext), string(p))
	}
}

func TestFileStorage(t *testing.T) {
	a := map[string]storedCredential{
		"secret": {
			Iv:         common.Hex2Bytes("cdb30036279601aeee60f16b"),
			CipherText: common.Hex2Bytes("f311ac49859d7260c2c464c28ffac122daf6be801d3cfd3edcbde7e00c9ff74f"),
		},
		"secret2": {
			Iv:         common.Hex2Bytes("afb8a7579bf971db9f8ceeed"),
			CipherText: common.Hex2Bytes("2df87baf86b5073ef1f03e3cc738de75b511400f5465bb0ddeacf47ae4dc267d"),
		},
	}
	d := t.TempDir()
	stored := &AESEncryptedStorage{
		filename: fmt.Sprintf("%v/vault.json", d),
		key:      []byte("AES256Key-32Characters1234567890"),
	}
	stored.writeEncryptedStorage(a)
	read := &AESEncryptedStorage{
		filename: fmt.Sprintf("%v/vault.json", d),
		key:      []byte("AES256Key-32Characters1234567890"),
	}
	creds, err := read.readEncryptedStorage()
	if err != nil {
		t.Fatal(err)
	}
	for k, v := range a {
		if v2, exist := creds[k]; !exist {
			t.Errorf("Missing entry %v", k)
		} else {
			if !bytes.Equal(v.CipherText, v2.CipherText) {
				t.Errorf("Wrong ciphertext, expected %x got %x", v.CipherText, v2.CipherText)
			}
			if !bytes.Equal(v.Iv, v2.Iv) {
				t.Errorf("Wrong iv")
			}
		}
	}
}
func TestEnd2End(t *testing.T) {
	log.Root().SetHandler(log.LvlFilterHandler(log.Lvl(3), log.StreamHandler(colorable.NewColorableStderr(), log.TerminalFormat(true))))

	d := t.TempDir()

	s1 := &AESEncryptedStorage{
		filename: fmt.Sprintf("%v/vault.json", d),
		key:      []byte("AES256Key-32Characters1234567890"),
	}
	s2 := &AESEncryptedStorage{
		filename: fmt.Sprintf("%v/vault.json", d),
		key:      []byte("AES256Key-32Characters1234567890"),
	}

	s1.Put("bazonk", "foobar")
	if v, err := s2.Get("bazonk"); v != "foobar" || err != nil {
		t.Errorf("Expected bazonk->foobar (nil error), got '%v' (%v error)", v, err)
	}
}

func TestSwappedKeys(t *testing.T) {
	// It should not be possible to swap the keys/values, so that
	// K1:V1, K2:V2 can be swapped into K1:V2, K2:V1
	log.Root().SetHandler(log.LvlFilterHandler(log.Lvl(3), log.StreamHandler(colorable.NewColorableStderr(), log.TerminalFormat(true))))

	d := t.TempDir()

	s1 := &AESEncryptedStorage{
		filename: fmt.Sprintf("%v/vault.json", d),
		key:      []byte("AES256Key-32Characters1234567890"),
	}
	s1.Put("k1", "v1")
	s1.Put("k2", "v2")
	// Now make a modified copy

	creds := make(map[string]storedCredential)
	raw, err := os.ReadFile(s1.filename)
	if err != nil {
		t.Fatal(err)
	}
	if err = json.Unmarshal(raw, &creds); err != nil {
		t.Fatal(err)
	}
	swap := func() {
		// Turn it into K1:V2, K2:V2
		v1, v2 := creds["k1"], creds["k2"]
		creds["k2"], creds["k1"] = v1, v2
		raw, err = json.Marshal(creds)
		if err != nil {
			t.Fatal(err)
		}
		if err = os.WriteFile(s1.filename, raw, 0600); err != nil {
			t.Fatal(err)
		}
	}
	swap()
	if v, _ := s1.Get("k1"); v != "" {
		t.Errorf("swapped value should return empty")
	}
	swap()
	if v, _ := s1.Get("k1"); v != "v1" {
		t.Errorf("double-swapped value should work fine")
	}
}
