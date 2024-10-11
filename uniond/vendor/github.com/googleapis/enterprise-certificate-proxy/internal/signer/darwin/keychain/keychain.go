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

//go:build darwin && cgo
// +build darwin,cgo

// Package keychain contains functions for retrieving certificates from the Darwin Keychain.
package keychain

/*
#cgo CFLAGS: -mmacosx-version-min=10.12
#cgo LDFLAGS: -framework CoreFoundation -framework Security

#include <CoreFoundation/CoreFoundation.h>
#include <Security/Security.h>
*/
import "C"

import (
	"bytes"
	"crypto"
	"crypto/ecdsa"
	"crypto/rsa"
	"crypto/x509"
	"encoding/pem"
	"fmt"
	"io"
	"runtime"
	"sync"
	"time"
	"unsafe"
)

// Maps for translating from crypto.Hash to SecKeyAlgorithm.
// https://developer.apple.com/documentation/security/seckeyalgorithm
var (
	ecdsaAlgorithms = map[crypto.Hash]C.CFStringRef{
		crypto.SHA256: C.kSecKeyAlgorithmECDSASignatureDigestX962SHA256,
		crypto.SHA384: C.kSecKeyAlgorithmECDSASignatureDigestX962SHA384,
		crypto.SHA512: C.kSecKeyAlgorithmECDSASignatureDigestX962SHA512,
	}
	rsaRaw = map[crypto.Hash]C.CFStringRef{
		crypto.SHA256: C.kSecKeyAlgorithmRSAEncryptionRaw,
	}
	rsaPKCS1v15Algorithms = map[crypto.Hash]C.CFStringRef{
		crypto.SHA256: C.kSecKeyAlgorithmRSASignatureDigestPKCS1v15SHA256,
		crypto.SHA384: C.kSecKeyAlgorithmRSASignatureDigestPKCS1v15SHA384,
		crypto.SHA512: C.kSecKeyAlgorithmRSASignatureDigestPKCS1v15SHA512,
	}
	rsaPSSAlgorithms = map[crypto.Hash]C.CFStringRef{
		crypto.SHA256: C.kSecKeyAlgorithmRSASignatureDigestPSSSHA256,
		crypto.SHA384: C.kSecKeyAlgorithmRSASignatureDigestPSSSHA384,
		crypto.SHA512: C.kSecKeyAlgorithmRSASignatureDigestPSSSHA512,
	}
	rsaOAEPAlgorithms = map[crypto.Hash]C.CFStringRef{
		crypto.SHA256: C.kSecKeyAlgorithmRSAEncryptionOAEPSHA256,
		crypto.SHA384: C.kSecKeyAlgorithmRSAEncryptionOAEPSHA384,
		crypto.SHA512: C.kSecKeyAlgorithmRSAEncryptionOAEPSHA512,
	}
)

const unknownSecKeyAlgorithm = C.CFStringRef(0)
const invalidKey = C.SecKeyRef(0)

// cfStringToString returns a Go string given a CFString.
func cfStringToString(cfStr C.CFStringRef) string {
	s := C.CFStringGetCStringPtr(cfStr, C.kCFStringEncodingUTF8)
	if s != nil {
		return C.GoString(s)
	}
	glyphLength := C.CFStringGetLength(cfStr) + 1
	utf8Length := C.CFStringGetMaximumSizeForEncoding(glyphLength, C.kCFStringEncodingUTF8)
	if s = (*C.char)(C.malloc(C.size_t(utf8Length))); s == nil {
		panic("unable to allocate memory")
	}
	defer C.free(unsafe.Pointer(s))
	if C.CFStringGetCString(cfStr, s, utf8Length, C.kCFStringEncodingUTF8) == 0 {
		panic("unable to convert cfStringref to string")
	}
	return C.GoString(s)
}

func cfRelease(x unsafe.Pointer) {
	C.CFRelease(C.CFTypeRef(x))
}

// cfError is an error type that owns a CFErrorRef, and obtains the error string
// by using CFErrorCopyDescription.
type cfError struct {
	e C.CFErrorRef
}

// cfErrorFromRef converts a C.CFErrorRef to a cfError, taking ownership of the
// reference and releasing when the value is finalized.
func cfErrorFromRef(cfErr C.CFErrorRef) error {
	if cfErr == 0 {
		return nil
	}
	c := &cfError{e: cfErr}
	runtime.SetFinalizer(c, func(x interface{}) {
		C.CFRelease(C.CFTypeRef(x.(*cfError).e))
	})
	return c
}

func (e *cfError) Error() string {
	s := C.CFErrorCopyDescription(C.CFErrorRef(e.e))
	defer C.CFRelease(C.CFTypeRef(s))
	return cfStringToString(s)
}

// keychainError is an error type that is based on an OSStatus return code, and
// obtains the error string with SecCopyErrorMessageString.
type keychainError C.OSStatus

func (e keychainError) Error() string {
	s := C.SecCopyErrorMessageString(C.OSStatus(e), nil)
	defer C.CFRelease(C.CFTypeRef(s))
	return cfStringToString(s)
}

// cfDataToBytes turns a CFDataRef into a byte slice.
func cfDataToBytes(cfData C.CFDataRef) []byte {
	return C.GoBytes(unsafe.Pointer(C.CFDataGetBytePtr(cfData)), C.int(C.CFDataGetLength(cfData)))
}

// bytesToCFData turns a byte slice into a CFDataRef. Caller then "owns" the
// CFDataRef and must CFRelease the CFDataRef when done.
func bytesToCFData(buf []byte) C.CFDataRef {
	return C.CFDataCreate(C.kCFAllocatorDefault, (*C.UInt8)(unsafe.Pointer(&buf[0])), C.CFIndex(len(buf)))
}

// int32ToCFNumber turns an int32 into a CFNumberRef. Caller then "owns"
// the CFNumberRef and must CFRelease the CFNumberRef when done.
func int32ToCFNumber(n int32) C.CFNumberRef {
	return C.CFNumberCreate(C.kCFAllocatorDefault, C.kCFNumberSInt32Type, unsafe.Pointer(&n))
}

// Key is a wrapper around the Keychain reference that uses it to
// implement signing-related methods with Keychain functionality.
type Key struct {
	privateKeyRef C.SecKeyRef
	certs         []*x509.Certificate
	once          sync.Once
	publicKeyRef  C.SecKeyRef
	hash          crypto.Hash
}

// newKey makes a new Key wrapper around the key reference,
// takes ownership of the reference, and sets up a finalizer to handle releasing
// the reference.
func newKey(privateKeyRef C.SecKeyRef, certs []*x509.Certificate, publicKeyRef C.SecKeyRef) (*Key, error) {
	k := &Key{
		privateKeyRef: privateKeyRef,
		certs:         certs,
		publicKeyRef:  publicKeyRef,
		hash:          crypto.SHA256,
	}

	// This struct now owns the key reference. Retain now and release on
	// finalise in case the credential gets forgotten about.
	C.CFRetain(C.CFTypeRef(privateKeyRef))
	C.CFRetain(C.CFTypeRef(publicKeyRef))
	runtime.SetFinalizer(k, func(x interface{}) {
		x.(*Key).Close()
	})
	return k, nil
}

// CertificateChain returns the credential as a raw X509 cert chain. This
// contains the public key.
func (k *Key) CertificateChain() [][]byte {
	rv := make([][]byte, len(k.certs))
	for i, c := range k.certs {
		rv[i] = c.Raw
	}
	return rv
}

// Close releases resources held by the credential.
func (k *Key) Close() error {
	// Don't double-release references.
	k.once.Do(func() {
		C.CFRelease(C.CFTypeRef(k.privateKeyRef))
		C.CFRelease(C.CFTypeRef(k.publicKeyRef))
	})
	return nil
}

// Public returns the corresponding public key for this Key. Good
// thing we extracted it when we created it.
func (k *Key) Public() crypto.PublicKey {
	return k.certs[0].PublicKey
}

// Sign signs a message digest. Here, we pass off the signing to Keychain library.
func (k *Key) Sign(rand io.Reader, digest []byte, opts crypto.SignerOpts) (signature []byte, err error) {
	// Map the signing algorithm and hash function to a SecKeyAlgorithm constant.
	var algorithms map[crypto.Hash]C.CFStringRef
	switch pub := k.Public().(type) {
	case *ecdsa.PublicKey:
		algorithms = ecdsaAlgorithms
	case *rsa.PublicKey:
		if _, ok := opts.(*rsa.PSSOptions); ok {
			algorithms = rsaPSSAlgorithms
			break
		}
		algorithms = rsaPKCS1v15Algorithms
	default:
		return nil, fmt.Errorf("unsupported algorithm %T", pub)
	}
	algorithm, ok := algorithms[opts.HashFunc()]
	if !ok {
		return nil, fmt.Errorf("unsupported hash function %T", opts.HashFunc())
	}

	// Copy input over into CF-land.
	cfDigest := bytesToCFData(digest)
	defer C.CFRelease(C.CFTypeRef(cfDigest))

	var cfErr C.CFErrorRef
	sig := C.SecKeyCreateSignature(C.SecKeyRef(k.privateKeyRef), algorithm, C.CFDataRef(cfDigest), &cfErr)
	if cfErr != 0 {
		return nil, cfErrorFromRef(cfErr)
	}
	defer C.CFRelease(C.CFTypeRef(sig))

	return cfDataToBytes(C.CFDataRef(sig)), nil
}

// Cred gets the first Credential (filtering on issuer) corresponding to
// available certificate and private key pairs (i.e. identities) available in
// the Keychain. This includes both the current login keychain for the user,
// and the system keychain.
func Cred(issuerCN string) (*Key, error) {
	leafSearch := C.CFDictionaryCreateMutable(C.kCFAllocatorDefault, 5, &C.kCFTypeDictionaryKeyCallBacks, &C.kCFTypeDictionaryValueCallBacks)
	defer C.CFRelease(C.CFTypeRef(unsafe.Pointer(leafSearch)))
	// Get identities (certificate + private key pairs).
	C.CFDictionaryAddValue(leafSearch, unsafe.Pointer(C.kSecClass), unsafe.Pointer(C.kSecClassIdentity))
	// Get identities that are signing capable.
	C.CFDictionaryAddValue(leafSearch, unsafe.Pointer(C.kSecAttrCanSign), unsafe.Pointer(C.kCFBooleanTrue))
	// For each identity, give us the reference to it.
	C.CFDictionaryAddValue(leafSearch, unsafe.Pointer(C.kSecReturnRef), unsafe.Pointer(C.kCFBooleanTrue))
	// Be sure to list out all the matches.
	C.CFDictionaryAddValue(leafSearch, unsafe.Pointer(C.kSecMatchLimit), unsafe.Pointer(C.kSecMatchLimitAll))
	// Do the matching-item copy.
	var leafMatches C.CFTypeRef
	if errno := C.SecItemCopyMatching((C.CFDictionaryRef)(leafSearch), &leafMatches); errno != C.errSecSuccess {
		return nil, keychainError(errno)
	}
	defer C.CFRelease(leafMatches)
	signingIdents := C.CFArrayRef(leafMatches)
	// Dump the certs into golang x509 Certificates.
	var (
		leafIdent C.SecIdentityRef
		leaf      *x509.Certificate
	)
	// Find the first valid leaf whose issuer (CA) matches the name in filter.
	// Validation in identityToX509 covers Not Before, Not After and key alg.
	for i := 0; i < int(C.CFArrayGetCount(signingIdents)) && leaf == nil; i++ {
		identDict := C.CFArrayGetValueAtIndex(signingIdents, C.CFIndex(i))
		xc, err := identityToX509(C.SecIdentityRef(identDict))
		if err != nil {
			continue
		}
		if xc.Issuer.CommonName == issuerCN {
			leaf = xc
			leafIdent = C.SecIdentityRef(identDict)
		}
	}

	caSearch := C.CFDictionaryCreateMutable(C.kCFAllocatorDefault, 0, &C.kCFTypeDictionaryKeyCallBacks, &C.kCFTypeDictionaryValueCallBacks)
	defer C.CFRelease(C.CFTypeRef(unsafe.Pointer(caSearch)))
	// Get identities (certificates).
	C.CFDictionaryAddValue(caSearch, unsafe.Pointer(C.kSecClass), unsafe.Pointer(C.kSecClassCertificate))
	// For each identity, give us the reference to it.
	C.CFDictionaryAddValue(caSearch, unsafe.Pointer(C.kSecReturnRef), unsafe.Pointer(C.kCFBooleanTrue))
	// Be sure to list out all the matches.
	C.CFDictionaryAddValue(caSearch, unsafe.Pointer(C.kSecMatchLimit), unsafe.Pointer(C.kSecMatchLimitAll))
	// Do the matching-item copy.
	var caMatches C.CFTypeRef
	if errno := C.SecItemCopyMatching((C.CFDictionaryRef)(caSearch), &caMatches); errno != C.errSecSuccess {
		return nil, keychainError(errno)
	}
	defer C.CFRelease(caMatches)
	certRefs := C.CFArrayRef(caMatches)
	// Validate and dump the certs into golang x509 Certificates.
	var allCerts []*x509.Certificate
	for i := 0; i < int(C.CFArrayGetCount(certRefs)); i++ {
		refDict := C.CFArrayGetValueAtIndex(certRefs, C.CFIndex(i))
		if xc, err := certRefToX509(C.SecCertificateRef(refDict)); err == nil {
			allCerts = append(allCerts, xc)
		}
	}

	// Build a certificate chain from leaf by matching prev.RawIssuer to
	// next.RawSubject across all valid certificates in the keychain.
	var (
		certs      []*x509.Certificate
		prev, next *x509.Certificate
	)
	for prev = leaf; prev != nil; prev, next = next, nil {
		certs = append(certs, prev)
		for _, xc := range allCerts {
			if certIn(xc, certs) {
				continue // finite chains only, mmmmkay.
			}
			if bytes.Equal(prev.RawIssuer, xc.RawSubject) && prev.CheckSignatureFrom(xc) == nil {
				// Prefer certificates with later expirations.
				if next == nil || xc.NotAfter.After(next.NotAfter) {
					next = xc
				}
			}
		}
	}
	if len(certs) == 0 {
		return nil, fmt.Errorf("no key found with issuer common name %q", issuerCN)
	}

	skr, err := identityToPrivateSecKeyRef(leafIdent)

	if err != nil {
		return nil, err
	}
	pubKey, err := identityToPublicSecKeyRef(leafIdent)
	if err != nil {
		return nil, err
	}
	defer C.CFRelease(C.CFTypeRef(skr))
	return newKey(skr, certs, pubKey)
}

// identityToX509 converts a single CFDictionary that contains the item ref and
// attribute dictionary into an x509.Certificate.
func identityToX509(ident C.SecIdentityRef) (*x509.Certificate, error) {
	var certRef C.SecCertificateRef
	if errno := C.SecIdentityCopyCertificate(ident, &certRef); errno != 0 {
		return nil, keychainError(errno)
	}
	defer C.CFRelease(C.CFTypeRef(certRef))

	return certRefToX509(certRef)
}

// certRefToX509 converts a single C.SecCertificateRef into an *x509.Certificate.
func certRefToX509(certRef C.SecCertificateRef) (*x509.Certificate, error) {
	// Export the PEM-encoded certificate to a CFDataRef.
	var certPEMData C.CFDataRef
	if errno := C.SecItemExport(C.CFTypeRef(certRef), C.kSecFormatUnknown, C.kSecItemPemArmour, nil, &certPEMData); errno != 0 {
		return nil, keychainError(errno)
	}
	defer C.CFRelease(C.CFTypeRef(certPEMData))
	certPEM := cfDataToBytes(certPEMData)

	// This part based on crypto/tls.
	var certDERBlock *pem.Block
	for {
		certDERBlock, certPEM = pem.Decode(certPEM)
		if certDERBlock == nil {
			return nil, fmt.Errorf("failed to parse certificate PEM data")
		}
		if certDERBlock.Type == "CERTIFICATE" {
			// found it
			break
		}
	}

	// Check the certificate is OK by the x509 library, and obtain the
	// public key algorithm (which I assume is the same as the private key
	// algorithm). This also filters out certs missing critical extensions.
	xc, err := x509.ParseCertificate(certDERBlock.Bytes)
	if err != nil {
		return nil, err
	}
	switch xc.PublicKey.(type) {
	case *rsa.PublicKey, *ecdsa.PublicKey:
	default:
		return nil, fmt.Errorf("unsupported key type %T", xc.PublicKey)
	}

	// Check the certificate is valid
	if n := time.Now(); n.Before(xc.NotBefore) || n.After(xc.NotAfter) {
		return nil, fmt.Errorf("certificate not valid")
	}

	return xc, nil
}

// identityToSecKeyRef converts a single CFDictionary that contains the item ref and
// attribute dictionary into a SecKeyRef for its private key.
func identityToPrivateSecKeyRef(ident C.SecIdentityRef) (C.SecKeyRef, error) {
	// Get the private key (ref). Note that "Copy" in "CopyPrivateKey"
	// refers to "the create rule" of CoreFoundation memory management, and
	// does not actually copy the private key---it gives us a copy of the
	// reference that we now own.
	var ref C.SecKeyRef
	if errno := C.SecIdentityCopyPrivateKey(C.SecIdentityRef(ident), &ref); errno != 0 {
		return 0, keychainError(errno)
	}
	return ref, nil
}

func identityToPublicSecKeyRef(ident C.SecIdentityRef) (C.SecKeyRef, error) {
	var key C.SecKeyRef
	var certRef C.SecCertificateRef
	if errno := C.SecIdentityCopyCertificate(ident, &certRef); errno != 0 {
		return 0, keychainError(errno)
	}
	defer C.CFRelease(C.CFTypeRef(certRef))

	key = C.SecCertificateCopyKey(certRef)

	if key == invalidKey {
		return 0, fmt.Errorf("public key was NULL. Key might have an encoding issue or use an unsupported algorithm")
	}
	return key, nil
}

func stringIn(s string, ss []string) bool {
	for _, s2 := range ss {
		if s == s2 {
			return true
		}
	}
	return false
}

func certIn(xc *x509.Certificate, xcs []*x509.Certificate) bool {
	for _, xc2 := range xcs {
		if xc.Equal(xc2) {
			return true
		}
	}
	return false
}

func (k *Key) getPaddingSize() int {
	algorithms, algoErr := k.getEncryptAlgorithm()
	if algoErr != nil {
		fmt.Printf("algorithm is unsupported. only RSA algorithms are supported. %v", algoErr)
	}
	// Each padding scheme has varying number of bytes.
	pssPaddingBytes := 20
	oaepPaddingBytes := 130
	pkcsPaddingBytes := 11
	switch algorithms {
	case C.kSecKeyAlgorithmRSASignatureDigestPSSSHA256,
		C.kSecKeyAlgorithmRSASignatureDigestPSSSHA384,
		C.kSecKeyAlgorithmRSASignatureDigestPSSSHA512:
		return pssPaddingBytes
	case C.kSecKeyAlgorithmRSAEncryptionOAEPSHA256,
		C.kSecKeyAlgorithmRSAEncryptionOAEPSHA384,
		C.kSecKeyAlgorithmRSAEncryptionOAEPSHA512:
		return oaepPaddingBytes
	case C.kSecKeyAlgorithmRSASignatureDigestPKCS1v15SHA256,
		C.kSecKeyAlgorithmRSASignatureDigestPKCS1v15SHA384,
		C.kSecKeyAlgorithmRSASignatureDigestPKCS1v15SHA512:
		return pkcsPaddingBytes
	default:
		return int(unknownSecKeyAlgorithm)
	}
}

func (k *Key) checkDataSize(plaintext []byte) error {
	// Plaintext data must be smaller than the key's block size minus padding space.
	sizeLim := uint64(C.SecKeyGetBlockSize(k.publicKeyRef)) - uint64(k.getPaddingSize())
	if uint64(len(plaintext)) >= sizeLim {
		return fmt.Errorf("plaintext is too long")
	}
	return nil
}

func (k *Key) getRSAEncryptAlgorithm() (C.SecKeyAlgorithm, error) {
	var algorithms map[crypto.Hash]C.CFStringRef
	switch pub := k.Public().(type) {
	case *rsa.PublicKey:
		if C.SecKeyIsAlgorithmSupported(k.publicKeyRef, C.kSecKeyOperationTypeEncrypt, C.kSecKeyAlgorithmRSASignatureDigestPSSSHA256) == 1 {
			algorithms = rsaPSSAlgorithms
		} else if C.SecKeyIsAlgorithmSupported(k.publicKeyRef, C.kSecKeyOperationTypeEncrypt, C.kSecKeyAlgorithmRSAEncryptionOAEPSHA256) == 1 {
			algorithms = rsaOAEPAlgorithms
		} else if C.SecKeyIsAlgorithmSupported(k.publicKeyRef, C.kSecKeyOperationTypeEncrypt, C.kSecKeyAlgorithmRSASignatureDigestPKCS1v15SHA256) == 1 {
			algorithms = rsaPKCS1v15Algorithms
		} else {
			return unknownSecKeyAlgorithm, fmt.Errorf("unknown RSA argument. Only supports PSS, OAEP, and PKCS1v1.5 %T", pub)
		}
	default:
		return unknownSecKeyAlgorithm, fmt.Errorf("algorithm is unsupported. only RSA algorithms are supported. %T", pub)
	}
	return algorithms[k.hash], nil
}

func (k *Key) getEncryptAlgorithm() (C.SecKeyAlgorithm, error) {
	if k.hash == 0 {
		k.hash = crypto.SHA256
	}
	return k.getRSAEncryptAlgorithm()
}

func (k *Key) getRSADecryptAlgorithm() (C.SecKeyAlgorithm, error) {
	var algorithms map[crypto.Hash]C.CFStringRef
	switch pub := k.Public().(type) {
	case *rsa.PublicKey:
		if C.SecKeyIsAlgorithmSupported(k.publicKeyRef, C.kSecKeyOperationTypeDecrypt, C.kSecKeyAlgorithmRSASignatureDigestPSSSHA256) == 1 {
			algorithms = rsaPSSAlgorithms
		} else if C.SecKeyIsAlgorithmSupported(k.publicKeyRef, C.kSecKeyOperationTypeDecrypt, C.kSecKeyAlgorithmRSAEncryptionOAEPSHA256) == 1 {
			algorithms = rsaOAEPAlgorithms
		} else if C.SecKeyIsAlgorithmSupported(k.publicKeyRef, C.kSecKeyOperationTypeDecrypt, C.kSecKeyAlgorithmRSASignatureDigestPKCS1v15SHA256) == 1 {
			algorithms = rsaPKCS1v15Algorithms
		} else {
			return unknownSecKeyAlgorithm, fmt.Errorf("unknown RSA argument. Only supports PSS, OAEP, and PKCS1v1.5 %T", pub)
		}
	default:
		return unknownSecKeyAlgorithm, fmt.Errorf("algorithm is unsupported. only RSA algorithms are supported. %T", pub)
	}
	return algorithms[k.hash], nil
}

func (k *Key) getDecryptAlgorithm() (C.SecKeyAlgorithm, error) {
	return k.getRSADecryptAlgorithm()
}

// Encrypt encrypts a plaintext message digest using the public key. Here, we pass off the encryption to Keychain library.
func (k *Key) Encrypt(plaintext []byte, opts any) ([]byte, error) {
	if hash, ok := opts.(crypto.Hash); ok {
		k.hash = hash
	} else {
		return nil, fmt.Errorf("Unsupported encrypt opts: %v", opts)
	}
	pub := k.publicKeyRef
	algorithm, err := k.getEncryptAlgorithm()
	if err != nil {
		return nil, err
	}
	if err := k.checkDataSize(plaintext); err != nil {
		return nil, err
	}
	msg := bytesToCFData(plaintext)
	var cfErr C.CFErrorRef
	bytes := C.SecKeyCreateEncryptedData(pub, algorithm, msg, &cfErr)

	if cfErr != 0 {
		return nil, cfErrorFromRef(cfErr)
	}

	ciphertext := cfDataToBytes(bytes)
	return ciphertext, cfErrorFromRef(cfErr)
}

// Decrypt decrypts a ciphertext message digest using the private key. Here, we pass off the decryption to Keychain library.
// Currently, only *rsa.OAEPOptions is supported for opts.
func (k *Key) Decrypt(ciphertext []byte, opts crypto.DecrypterOpts) ([]byte, error) {
	if oaepOpts, ok := opts.(*rsa.OAEPOptions); ok {
		k.hash = oaepOpts.Hash
	} else {
		return nil, fmt.Errorf("Unsupported DecrypterOpts: %v", opts)
	}
	priv := k.privateKeyRef
	algorithm, err := k.getDecryptAlgorithm()
	if err != nil {
		return nil, err
	}
	msg := bytesToCFData(ciphertext)
	var cfErr C.CFErrorRef
	bytes := C.SecKeyCreateDecryptedData(priv, algorithm, msg, &cfErr)

	if cfErr != 0 {
		return nil, cfErrorFromRef(cfErr)
	}

	plaintext := cfDataToBytes(bytes)
	return plaintext, cfErrorFromRef(cfErr)
}
