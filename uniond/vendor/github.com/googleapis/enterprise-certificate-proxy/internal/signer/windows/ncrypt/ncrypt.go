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

// Package ncrypt provides wrappers around ncrypt.h functions.
// https://docs.microsoft.com/en-us/windows/win32/api/ncrypt/
package ncrypt

import (
	"crypto"
	"crypto/ecdsa"
	"crypto/rsa"
	"fmt"
	"math/big"
	"unsafe"

	"golang.org/x/crypto/cryptobyte"
	"golang.org/x/crypto/cryptobyte/asn1"
	"golang.org/x/sys/windows"
)

const (
	// bcrypt.h constants
	bcryptPadPKCS1 = 0x00000002 // BCRYPT_PAD_PKCS1
	bcryptPadPSS   = 0x00000008 // BCRYPT_PAD_PSS

	// ncrypt.h constants
	nCryptSilentFlag = 0x00000040 // NCRYPT_SILENT_FLAG
)

var (
	nCrypt         = windows.MustLoadDLL("ncrypt.dll")
	nCryptSignHash = nCrypt.MustFindProc("NCryptSignHash")
)

// bcypt.h structs.
type pkcs1PaddingInfo struct {
	algID *uint16
}
type pssPaddingInfo struct {
	algID      *uint16
	saltLength uint32
}

func algID(hashFunc crypto.Hash) (*uint16, bool) {
	algID, ok := map[crypto.Hash][]uint16{
		crypto.SHA256: {'S', 'H', 'A', '2', '5', '6', 0}, // BCRYPT_SHA256_ALGORITHM
	}[hashFunc]
	return &algID[0], ok
}

func rsaPadding(opts crypto.SignerOpts, flags *int) (paddingInfo unsafe.Pointer, err error) {
	if o, ok := opts.(*rsa.PSSOptions); ok {
		algID, ok := algID(o.HashFunc())
		if !ok {
			err = fmt.Errorf("unsupported hash function %T", o.HashFunc())
			return
		}
		saltLength := o.SaltLength
		switch saltLength {
		case rsa.PSSSaltLengthAuto:
			err = fmt.Errorf("rsa.PSSSaltLengthAuto is not supported")
			return
		case rsa.PSSSaltLengthEqualsHash:
			saltLength = o.HashFunc().Size()
		}
		paddingInfo = unsafe.Pointer(&pssPaddingInfo{
			algID:      algID,
			saltLength: uint32(saltLength),
		})
		*flags |= bcryptPadPSS
		return
	}

	algID, ok := algID(opts.HashFunc())
	if !ok {
		err = fmt.Errorf("unsupported hash function %T", opts.HashFunc())
		return
	}
	paddingInfo = unsafe.Pointer(&pkcs1PaddingInfo{
		algID: algID,
	})
	*flags |= bcryptPadPKCS1
	return
}

func signHashInternal(priv windows.Handle, pub crypto.PublicKey, digest []byte, flags int, paddingInfo unsafe.Pointer) ([]byte, error) {
	var size uint32
	r, _, _ := nCryptSignHash.Call(
		/* hKey */ uintptr(priv),
		/* *pPaddingInfo */ uintptr(paddingInfo),
		/* pbHashValue */ uintptr(unsafe.Pointer(&digest[0])),
		/* cbHashValue */ uintptr(len(digest)),
		/* pbSignature */ 0,
		/* cbSignature */ 0,
		/* *pcbResult */ uintptr(unsafe.Pointer(&size)),
		/* dwFlagss */ uintptr(flags))
	if r != 0 {
		return nil, fmt.Errorf("NCryptSignHash: failed to get signature length: %#x", r)
	}

	sig := make([]byte, size)
	r, _, _ = nCryptSignHash.Call(
		/* hKey */ uintptr(priv),
		/* *pPaddingInfo */ uintptr(paddingInfo),
		/* pbHashValue */ uintptr(unsafe.Pointer(&digest[0])),
		/* cbHashValue */ uintptr(len(digest)),
		/* pbSignature */ uintptr(unsafe.Pointer(&sig[0])),
		/* cbSignature */ uintptr(size),
		/* *pcbResult */ uintptr(unsafe.Pointer(&size)),
		/* dwFlagss */ uintptr(flags))
	if r != 0 {
		return nil, fmt.Errorf("NCryptSignHash: failed to get signature: %#x", r)
	}
	if len(sig) != int(size) {
		return nil, fmt.Errorf("invalid length sig = %d, size = %d", sig, size)
	}

	switch pub := pub.(type) {
	case *ecdsa.PublicKey:
		var b cryptobyte.Builder
		b.AddASN1(asn1.SEQUENCE, func(b *cryptobyte.Builder) {
			b.AddASN1BigInt(new(big.Int).SetBytes(sig[:len(sig)/2]))
			b.AddASN1BigInt(new(big.Int).SetBytes(sig[len(sig)/2:]))
		})
		return b.Bytes()
	case *rsa.PublicKey:
		return sig, nil
	default:
		return nil, fmt.Errorf("unsupported public key type %T", pub)
	}
}

// SignHash is a wrapper for the NCryptSignHash function that supports only a
// subset of well-supported cryptographic primitives.
//
// Signature algorithms: ECDSA, RSA.
// Hash functions: SHA-256.
// RSA schemes: RSASSA-PKCS1 and RSASSA-PSS.
//
// https://docs.microsoft.com/en-us/windows/win32/api/ncrypt/nf-ncrypt-ncryptsignhash
func SignHash(priv windows.Handle, pub crypto.PublicKey, digest []byte, opts crypto.SignerOpts) ([]byte, error) {
	var paddingInfo unsafe.Pointer
	flags := nCryptSilentFlag
	switch pub := pub.(type) {
	case *ecdsa.PublicKey:
	case *rsa.PublicKey:
		var err error
		paddingInfo, err = rsaPadding(opts, &flags)
		if err != nil {
			return nil, err
		}
	default:
		return nil, fmt.Errorf("unsupported public key type %T", pub)
	}

	return signHashInternal(priv, pub, digest, flags, paddingInfo)
}
