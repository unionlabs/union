// Copyright 2020 The go-ethereum Authors
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

//go:build gofuzz
// +build gofuzz

package signify

import (
	"bufio"
	"fmt"
	"log"
	"os"
	"os/exec"

	fuzz "github.com/google/gofuzz"
	"github.com/jedisct1/go-minisign"
)

func Fuzz(data []byte) int {
	if len(data) < 32 {
		return -1
	}
	tmpFile, err := os.CreateTemp("", "")
	if err != nil {
		panic(err)
	}
	defer os.Remove(tmpFile.Name())
	defer tmpFile.Close()

	testSecKey, testPubKey := createKeyPair()
	// Create message
	tmpFile.Write(data)
	if err = tmpFile.Close(); err != nil {
		panic(err)
	}
	// Fuzz comments
	var untrustedComment string
	var trustedComment string
	f := fuzz.NewFromGoFuzz(data)
	f.Fuzz(&untrustedComment)
	f.Fuzz(&trustedComment)
	fmt.Printf("untrusted: %v\n", untrustedComment)
	fmt.Printf("trusted: %v\n", trustedComment)

	err = SignifySignFile(tmpFile.Name(), tmpFile.Name()+".sig", testSecKey, untrustedComment, trustedComment)
	if err != nil {
		panic(err)
	}
	defer os.Remove(tmpFile.Name() + ".sig")

	signify := "signify"
	path := os.Getenv("SIGNIFY")
	if path != "" {
		signify = path
	}

	_, err := exec.LookPath(signify)
	if err != nil {
		panic(err)
	}

	// Write the public key into the file to pass it as
	// an argument to signify-openbsd
	pubKeyFile, err := os.CreateTemp("", "")
	if err != nil {
		panic(err)
	}
	defer os.Remove(pubKeyFile.Name())
	defer pubKeyFile.Close()
	pubKeyFile.WriteString("untrusted comment: signify public key\n")
	pubKeyFile.WriteString(testPubKey)
	pubKeyFile.WriteString("\n")

	cmd := exec.Command(signify, "-V", "-p", pubKeyFile.Name(), "-x", tmpFile.Name()+".sig", "-m", tmpFile.Name())
	if output, err := cmd.CombinedOutput(); err != nil {
		panic(fmt.Sprintf("could not verify the file: %v, output: \n%s", err, output))
	}

	// Verify the signature using a golang library
	sig, err := minisign.NewSignatureFromFile(tmpFile.Name() + ".sig")
	if err != nil {
		panic(err)
	}

	pKey, err := minisign.NewPublicKey(testPubKey)
	if err != nil {
		panic(err)
	}

	valid, err := pKey.VerifyFromFile(tmpFile.Name(), sig)
	if err != nil {
		panic(err)
	}
	if !valid {
		panic("invalid signature")
	}
	return 1
}

func getKey(fileS string) (string, error) {
	file, err := os.Open(fileS)
	if err != nil {
		log.Fatal(err)
	}
	defer file.Close()

	scanner := bufio.NewScanner(file)
	// Discard the first line
	scanner.Scan()
	scanner.Scan()
	return scanner.Text(), scanner.Err()
}

func createKeyPair() (string, string) {
	// Create key and put it in correct format
	tmpKey, err := os.CreateTemp("", "")
	if err != nil {
		panic(err)
	}
	defer os.Remove(tmpKey.Name())
	defer os.Remove(tmpKey.Name() + ".pub")
	defer os.Remove(tmpKey.Name() + ".sec")
	cmd := exec.Command("signify", "-G", "-n", "-p", tmpKey.Name()+".pub", "-s", tmpKey.Name()+".sec")
	if output, err := cmd.CombinedOutput(); err != nil {
		panic(fmt.Sprintf("could not verify the file: %v, output: \n%s", err, output))
	}
	secKey, err := getKey(tmpKey.Name() + ".sec")
	if err != nil {
		panic(err)
	}
	pubKey, err := getKey(tmpKey.Name() + ".pub")
	if err != nil {
		panic(err)
	}
	return secKey, pubKey
}
