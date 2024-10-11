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

//go:build windows
// +build windows

// Cert_util provides helpers for working with Windows certificates via crypt32.dll

package ncrypt

import (
	"crypto"
	"crypto/x509"
	"errors"
	"fmt"
	"io"
	"syscall"
	"unsafe"

	"golang.org/x/sys/windows"
)

const (
	// wincrypt.h constants
	encodingX509ASN                   = 1                                              // X509_ASN_ENCODING
	certStoreCurrentUserID            = 1                                              // CERT_SYSTEM_STORE_CURRENT_USER_ID
	certStoreLocalMachineID           = 2                                              // CERT_SYSTEM_STORE_LOCAL_MACHINE_ID
	infoIssuerFlag                    = 4                                              // CERT_INFO_ISSUER_FLAG
	compareNameStrW                   = 8                                              // CERT_COMPARE_NAME_STR_A
	certStoreProvSystem               = 10                                             // CERT_STORE_PROV_SYSTEM
	compareShift                      = 16                                             // CERT_COMPARE_SHIFT
	locationShift                     = 16                                             // CERT_SYSTEM_STORE_LOCATION_SHIFT
	findIssuerStr                     = compareNameStrW<<compareShift | infoIssuerFlag // CERT_FIND_ISSUER_STR_W
	certStoreLocalMachine             = certStoreLocalMachineID << locationShift       // CERT_SYSTEM_STORE_LOCAL_MACHINE
	certStoreCurrentUser              = certStoreCurrentUserID << locationShift        // CERT_SYSTEM_STORE_CURRENT_USER
	signatureKeyUsage                 = 0x80                                           // CERT_DIGITAL_SIGNATURE_KEY_USAGE
	acquireCached                     = 0x1                                            // CRYPT_ACQUIRE_CACHE_FLAG
	acquireSilent                     = 0x40                                           // CRYPT_ACQUIRE_SILENT_FLAG
	acquireOnlyNCryptKey              = 0x40000                                        // CRYPT_ACQUIRE_ONLY_NCRYPT_KEY_FLAG
	ncryptKeySpec                     = 0xFFFFFFFF                                     // CERT_NCRYPT_KEY_SPEC
	certChainCacheOnlyURLRetrieval    = 0x00000004                                     // CERT_CHAIN_CACHE_ONLY_URL_RETRIEVAL
	certChainDisableAIA               = 0x00002000                                     // CERT_CHAIN_DISABLE_AIA
	certChainRevocationCheckCacheOnly = 0x80000000                                     // CERT_CHAIN_REVOCATION_CHECK_CACHE_ONLY

	hcceLocalMachine = windows.Handle(0x01) // HCCE_LOCAL_MACHINE

	// winerror.h constants
	cryptENotFound = 0x80092004 // CRYPT_E_NOT_FOUND
)

var (
	null = uintptr(unsafe.Pointer(nil))

	crypt32 = windows.MustLoadDLL("crypt32.dll")

	certFindCertificateInStore        = crypt32.MustFindProc("CertFindCertificateInStore")
	certGetIntendedKeyUsage           = crypt32.MustFindProc("CertGetIntendedKeyUsage")
	cryptAcquireCertificatePrivateKey = crypt32.MustFindProc("CryptAcquireCertificatePrivateKey")
)

// findCert wraps the CertFindCertificateInStore call. Note that any cert context passed
// into prev will be freed. If no certificate was found, nil will be returned.
func findCert(store windows.Handle, enc uint32, findFlags uint32, findType uint32, para *uint16, prev *windows.CertContext) (*windows.CertContext, error) {
	h, _, err := certFindCertificateInStore.Call(
		uintptr(store),
		uintptr(enc),
		uintptr(findFlags),
		uintptr(findType),
		uintptr(unsafe.Pointer(para)),
		uintptr(unsafe.Pointer(prev)),
	)
	if h == 0 {
		// Actual error, or simply not found?
		errno, ok := err.(syscall.Errno)
		if !ok {
			return nil, err
		}
		if errno == cryptENotFound {
			return nil, nil
		}
		return nil, err
	}
	return (*windows.CertContext)(unsafe.Pointer(h)), nil
}

// extractSimpleChain extracts the final certificate chain from a CertSimpleChain.
// Adapted from crypto.x509.root_windows
func extractSimpleChain(simpleChain **windows.CertSimpleChain, chainCount int) ([]*x509.Certificate, error) {
	if simpleChain == nil || chainCount == 0 {
		return nil, errors.New("invalid simple chain")
	}
	// Convert the simpleChain array to a huge slice and slice it to the length we want.
	// https://github.com/golang/go/wiki/cgo#turning-c-arrays-into-go-slices
	simpleChains := (*[1 << 20]*windows.CertSimpleChain)(unsafe.Pointer(simpleChain))[:chainCount:chainCount]
	// Each simple chain contains the chain of certificates, summary trust information
	// about the chain, and trust information about each certificate element in the chain.
	// Select the last chain since only expect to encounter one chain.
	lastChain := simpleChains[chainCount-1]
	chainLen := int(lastChain.NumElements)
	elements := (*[1 << 20]*windows.CertChainElement)(unsafe.Pointer(lastChain.Elements))[:chainLen:chainLen]
	chain := make([]*x509.Certificate, 0, chainLen)
	for _, element := range elements {
		xc, err := certContextToX509(element.CertContext)
		if err != nil {
			return nil, err
		}
		chain = append(chain, xc)
	}
	return chain, nil
}

// findCertChain builds a chain from a given certificate using the local machine store.
func findCertChain(cert *windows.CertContext) ([]*x509.Certificate, error) {
	var (
		chainPara windows.CertChainPara
		chainCtx  *windows.CertChainContext
	)

	// Search the system for candidate certificate chains.
	// Because we are using unsafe pointers here, we CANNOT directly call
	// CertGetCertificateChain and MUST either use the windows or syscall library
	// to validly use unsafe pointers.
	// See https://golang.org/pkg/unsafe/#Pointer for valid unsafe package patterns.
	chainPara.Size = uint32(unsafe.Sizeof(chainPara))
	err := windows.CertGetCertificateChain(
		hcceLocalMachine,
		cert,
		nil,
		cert.Store,
		&chainPara,
		certChainRevocationCheckCacheOnly|certChainCacheOnlyURLRetrieval|certChainDisableAIA,
		0,
		&chainCtx)

	if err != nil {
		return nil, fmt.Errorf("getCertificateChain: %w", err)
	}
	defer windows.CertFreeCertificateChain(chainCtx)

	x509Certs, err := extractSimpleChain(chainCtx.Chains, int(chainCtx.ChainCount))
	if err != nil {
		return nil, fmt.Errorf("getCertificateChain extractSimpleChain: %w", err)
	}
	return x509Certs, nil
}

// intendedKeyUsage wraps CertGetIntendedKeyUsage. If there are key usage bytes they will be returned,
// otherwise 0 will be returned.
func intendedKeyUsage(enc uint32, cert *windows.CertContext) (usage uint16) {
	_, _, _ = certGetIntendedKeyUsage.Call(uintptr(enc), uintptr(unsafe.Pointer(cert.CertInfo)), uintptr(unsafe.Pointer(&usage)), 2)
	return
}

// acquirePrivateKey wraps CryptAcquireCertificatePrivateKey.
func acquirePrivateKey(cert *windows.CertContext) (windows.Handle, error) {
	var (
		key      windows.Handle
		keySpec  uint32
		mustFree int
	)
	r, _, err := cryptAcquireCertificatePrivateKey.Call(
		uintptr(unsafe.Pointer(cert)),
		acquireCached|acquireSilent|acquireOnlyNCryptKey,
		null,
		uintptr(unsafe.Pointer(&key)),
		uintptr(unsafe.Pointer(&keySpec)),
		uintptr(unsafe.Pointer(&mustFree)),
	)
	if r == 0 {
		return 0, fmt.Errorf("acquiring private key: %x %w", r, err)
	}
	if mustFree != 0 {
		return 0, fmt.Errorf("wrong mustFree [%d != 0]", mustFree)
	}
	if keySpec != ncryptKeySpec {
		return 0, fmt.Errorf("wrong keySpec [%d != %d]", keySpec, ncryptKeySpec)
	}
	return key, nil
}

// certContextToX509 extracts the x509 certificate from the cert context.
func certContextToX509(ctx *windows.CertContext) (*x509.Certificate, error) {
	// To ensure we don't mess with the cert context's memory, use a copy of it.
	src := (*[1 << 20]byte)(unsafe.Pointer(ctx.EncodedCert))[:ctx.Length:ctx.Length]
	der := make([]byte, int(ctx.Length))
	copy(der, src)

	xc, err := x509.ParseCertificate(der)
	if err != nil {
		return xc, err
	}
	return xc, nil
}

// Cred returns a Key wrapping the first valid certificate in the system store
// matching a given issuer string.
func Cred(issuer string, storeName string, provider string) (*Key, error) {
	var certStore uint32
	if provider == "local_machine" {
		certStore = uint32(certStoreLocalMachine)
	} else if provider == "current_user" {
		certStore = uint32(certStoreCurrentUser)
	} else {
		return nil, errors.New("provider must be local_machine or current_user")
	}
	storeNamePtr, err := windows.UTF16PtrFromString(storeName)
	if err != nil {
		return nil, err
	}
	store, err := windows.CertOpenStore(certStoreProvSystem, 0, null, certStore, uintptr(unsafe.Pointer(storeNamePtr)))
	if err != nil {
		return nil, fmt.Errorf("opening certificate store: %w", err)
	}
	i, err := windows.UTF16PtrFromString(issuer)
	if err != nil {
		return nil, err
	}
	var prev *windows.CertContext
	for {
		nc, err := findCert(store, encodingX509ASN, 0, findIssuerStr, i, prev)
		if err != nil {
			return nil, fmt.Errorf("finding certificates: %w", err)
		}
		if nc == nil {
			return nil, errors.New("no certificate found")
		}
		prev = nc
		if (intendedKeyUsage(encodingX509ASN, nc) & signatureKeyUsage) == 0 {
			continue
		}

		xc, err := certContextToX509(nc)
		if err != nil {
			continue
		}

		machineChain, err := findCertChain(nc)
		if err != nil {
			continue
		}
		return &Key{
			cert:  xc,
			ctx:   nc,
			store: store,
			chain: machineChain,
		}, nil
	}
}

// Key is a wrapper around the certificate store and context that uses it to
// implement signing-related methods with CryptoNG functionality.
type Key struct {
	cert  *x509.Certificate
	ctx   *windows.CertContext
	store windows.Handle
	chain []*x509.Certificate
}

// CertificateChain returns the credential as a raw X509 cert chain. This
// contains the public key.
func (k *Key) CertificateChain() [][]byte {
	// Convert the certificates to a list of encoded certificate bytes.
	chain := make([][]byte, len(k.chain))
	for i, xc := range k.chain {
		chain[i] = xc.Raw
	}
	return chain
}

// Close releases resources held by the credential.
func (k *Key) Close() error {
	if err := windows.CertFreeCertificateContext(k.ctx); err != nil {
		return err
	}
	return windows.CertCloseStore(k.store, 0)
}

// Public returns the corresponding public key for this Key.
func (k *Key) Public() crypto.PublicKey {
	return k.cert.PublicKey
}

// Sign signs a message digest. Here, we pass off the signing to the Windows CryptoNG library.
func (k *Key) Sign(_ io.Reader, digest []byte, opts crypto.SignerOpts) ([]byte, error) {
	key, err := acquirePrivateKey(k.ctx)
	if err != nil {
		return nil, fmt.Errorf("cannot acquire private key handle: %w", err)
	}
	return SignHash(key, k.Public(), digest, opts)
}
