// Copyright 2022 Google LLC.
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// Signer.go is a net/rpc server that listens on stdin/stdout, exposing
// methods that perform device certificate signing for Linux using PKCS11
// shared library.
// This server is intended to be launched as a subprocess by the signer client,
// and should not be launched manually as a stand-alone process.
package main

import (
	"crypto"
	"crypto/rsa"
	"crypto/x509"
	"encoding/gob"
	"io"
	"log"
	"net/rpc"
	"os"
	"time"

	"github.com/googleapis/enterprise-certificate-proxy/internal/signer/linux/pkcs11"
	"github.com/googleapis/enterprise-certificate-proxy/internal/signer/util"
)

// If ECP Logging is enabled return true
// Otherwise return false
func enableECPLogging() bool {
	if os.Getenv("ENABLE_ENTERPRISE_CERTIFICATE_LOGS") != "" {
		return true
	}

	log.SetOutput(io.Discard)
	return false
}

func init() {
	gob.Register(crypto.SHA256)
	gob.Register(crypto.SHA384)
	gob.Register(crypto.SHA512)
	gob.Register(&rsa.PSSOptions{})
	gob.Register(&rsa.OAEPOptions{})
}

// SignArgs contains arguments for a Sign API call.
type SignArgs struct {
	Digest []byte            // The content to sign.
	Opts   crypto.SignerOpts // Options for signing. Must implement HashFunc().
}

// EncryptArgs contains arguments for an Encrypt API call.
type EncryptArgs struct {
	Plaintext []byte // The plaintext to encrypt.
	Opts      any    // Options for encryption. Ex: an instance of crypto.Hash.
}

// DecryptArgs contains arguments to for a Decrypt API call.
type DecryptArgs struct {
	Ciphertext []byte               // The ciphertext to decrypt.
	Opts       crypto.DecrypterOpts // Options for decryption. Ex: an instance of *rsa.OAEPOptions.
}

// A EnterpriseCertSigner exports RPC methods for signing.
type EnterpriseCertSigner struct {
	key *pkcs11.Key
}

// A Connection wraps a pair of unidirectional streams as an io.ReadWriteCloser.
type Connection struct {
	io.ReadCloser
	io.WriteCloser
}

// Close closes c's underlying ReadCloser and WriteCloser.
func (c *Connection) Close() error {
	rerr := c.ReadCloser.Close()
	werr := c.WriteCloser.Close()
	if rerr != nil {
		return rerr
	}
	return werr
}

// CertificateChain returns the credential as a raw X509 cert chain. This
// contains the public key.
func (k *EnterpriseCertSigner) CertificateChain(ignored struct{}, certificateChain *[][]byte) (err error) {
	*certificateChain = k.key.CertificateChain()
	return nil
}

// Public returns the corresponding public key for this Key, in ASN.1 DER form.
func (k *EnterpriseCertSigner) Public(ignored struct{}, publicKey *[]byte) (err error) {
	*publicKey, err = x509.MarshalPKIXPublicKey(k.key.Public())
	return
}

// Sign signs a message digest. Stores result in "resp".
func (k *EnterpriseCertSigner) Sign(args SignArgs, resp *[]byte) (err error) {
	*resp, err = k.key.Sign(nil, args.Digest, args.Opts)
	return
}

// Encrypt encrypts a plaintext msg. Stores result in "resp".
func (k *EnterpriseCertSigner) Encrypt(args EncryptArgs, resp *[]byte) (err error) {
	*resp, err = k.key.Encrypt(args.Plaintext, args.Opts)
	return
}

// Decrypt decrypts a ciphertext msg. Stores result in "resp".
func (k *EnterpriseCertSigner) Decrypt(args DecryptArgs, resp *[]byte) (err error) {
	*resp, err = k.key.Decrypt(args.Ciphertext, args.Opts)
	return
}

func main() {
	enableECPLogging()
	if len(os.Args) != 2 {
		log.Fatalln("Signer is not meant to be invoked manually, exiting...")
	}
	configFilePath := os.Args[1]
	config, err := util.LoadConfig(configFilePath)
	if err != nil {
		log.Fatalf("Failed to load enterprise cert config: %v", err)
	}

	enterpriseCertSigner := new(EnterpriseCertSigner)
	enterpriseCertSigner.key, err = pkcs11.Cred(config.CertConfigs.PKCS11.PKCS11Module, config.CertConfigs.PKCS11.Slot, config.CertConfigs.PKCS11.Label, config.CertConfigs.PKCS11.UserPin)
	if err != nil {
		log.Fatalf("Failed to initialize enterprise cert signer using pkcs11: %v", err)
	}

	if err := rpc.Register(enterpriseCertSigner); err != nil {
		log.Fatalf("Failed to register enterprise cert signer with net/rpc: %v", err)
	}

	// If the parent process dies, we should exit.
	// We can detect this by periodically checking if the PID of the parent
	// process is 1 (https://stackoverflow.com/a/2035683).
	go func() {
		for {
			if os.Getppid() == 1 {
				log.Fatalln("Enterprise cert signer's parent process died, exiting...")
			}
			time.Sleep(time.Second)
		}
	}()

	rpc.ServeConn(&Connection{os.Stdin, os.Stdout})
}
