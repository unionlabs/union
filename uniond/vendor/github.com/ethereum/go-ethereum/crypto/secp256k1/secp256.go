// Copyright 2015 Jeffrey Wilcke, Felix Lange, Gustav Simonsson. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

//go:build !gofuzz && cgo
// +build !gofuzz,cgo

// Package secp256k1 wraps the bitcoin secp256k1 C library.
package secp256k1

/*
#cgo CFLAGS: -I./libsecp256k1
#cgo CFLAGS: -I./libsecp256k1/src/

#ifdef __SIZEOF_INT128__
#  define HAVE___INT128
#  define USE_FIELD_5X52
#  define USE_SCALAR_4X64
#else
#  define USE_FIELD_10X26
#  define USE_SCALAR_8X32
#endif

#ifndef NDEBUG
#  define NDEBUG
#endif

#define USE_ENDOMORPHISM
#define USE_NUM_NONE
#define USE_FIELD_INV_BUILTIN
#define USE_SCALAR_INV_BUILTIN
#include "./libsecp256k1/src/secp256k1.c"
#include "./libsecp256k1/src/modules/recovery/main_impl.h"
#include "ext.h"

typedef void (*callbackFunc) (const char* msg, void* data);
extern void secp256k1GoPanicIllegal(const char* msg, void* data);
extern void secp256k1GoPanicError(const char* msg, void* data);
*/
import "C"

import (
	"errors"
	"math/big"
	"unsafe"
)

var context *C.secp256k1_context

func init() {
	// around 20 ms on a modern CPU.
	context = C.secp256k1_context_create_sign_verify()
	C.secp256k1_context_set_illegal_callback(context, C.callbackFunc(C.secp256k1GoPanicIllegal), nil)
	C.secp256k1_context_set_error_callback(context, C.callbackFunc(C.secp256k1GoPanicError), nil)
}

var (
	ErrInvalidMsgLen       = errors.New("invalid message length, need 32 bytes")
	ErrInvalidSignatureLen = errors.New("invalid signature length")
	ErrInvalidRecoveryID   = errors.New("invalid signature recovery id")
	ErrInvalidKey          = errors.New("invalid private key")
	ErrInvalidPubkey       = errors.New("invalid public key")
	ErrSignFailed          = errors.New("signing failed")
	ErrRecoverFailed       = errors.New("recovery failed")
)

// Sign creates a recoverable ECDSA signature.
// The produced signature is in the 65-byte [R || S || V] format where V is 0 or 1.
//
// The caller is responsible for ensuring that msg cannot be chosen
// directly by an attacker. It is usually preferable to use a cryptographic
// hash function on any input before handing it to this function.
func Sign(msg []byte, seckey []byte) ([]byte, error) {
	if len(msg) != 32 {
		return nil, ErrInvalidMsgLen
	}
	if len(seckey) != 32 {
		return nil, ErrInvalidKey
	}
	seckeydata := (*C.uchar)(unsafe.Pointer(&seckey[0]))
	if C.secp256k1_ec_seckey_verify(context, seckeydata) != 1 {
		return nil, ErrInvalidKey
	}

	var (
		msgdata   = (*C.uchar)(unsafe.Pointer(&msg[0]))
		noncefunc = C.secp256k1_nonce_function_rfc6979
		sigstruct C.secp256k1_ecdsa_recoverable_signature
	)
	if C.secp256k1_ecdsa_sign_recoverable(context, &sigstruct, msgdata, seckeydata, noncefunc, nil) == 0 {
		return nil, ErrSignFailed
	}

	var (
		sig     = make([]byte, 65)
		sigdata = (*C.uchar)(unsafe.Pointer(&sig[0]))
		recid   C.int
	)
	C.secp256k1_ecdsa_recoverable_signature_serialize_compact(context, sigdata, &recid, &sigstruct)
	sig[64] = byte(recid) // add back recid to get 65 bytes sig
	return sig, nil
}

// RecoverPubkey returns the public key of the signer.
// msg must be the 32-byte hash of the message to be signed.
// sig must be a 65-byte compact ECDSA signature containing the
// recovery id as the last element.
func RecoverPubkey(msg []byte, sig []byte) ([]byte, error) {
	if len(msg) != 32 {
		return nil, ErrInvalidMsgLen
	}
	if err := checkSignature(sig); err != nil {
		return nil, err
	}

	var (
		pubkey  = make([]byte, 65)
		sigdata = (*C.uchar)(unsafe.Pointer(&sig[0]))
		msgdata = (*C.uchar)(unsafe.Pointer(&msg[0]))
	)
	if C.secp256k1_ext_ecdsa_recover(context, (*C.uchar)(unsafe.Pointer(&pubkey[0])), sigdata, msgdata) == 0 {
		return nil, ErrRecoverFailed
	}
	return pubkey, nil
}

// VerifySignature checks that the given pubkey created signature over message.
// The signature should be in [R || S] format.
func VerifySignature(pubkey, msg, signature []byte) bool {
	if len(msg) != 32 || len(signature) != 64 || len(pubkey) == 0 {
		return false
	}
	sigdata := (*C.uchar)(unsafe.Pointer(&signature[0]))
	msgdata := (*C.uchar)(unsafe.Pointer(&msg[0]))
	keydata := (*C.uchar)(unsafe.Pointer(&pubkey[0]))
	return C.secp256k1_ext_ecdsa_verify(context, sigdata, msgdata, keydata, C.size_t(len(pubkey))) != 0
}

// DecompressPubkey parses a public key in the 33-byte compressed format.
// It returns non-nil coordinates if the public key is valid.
func DecompressPubkey(pubkey []byte) (x, y *big.Int) {
	if len(pubkey) != 33 {
		return nil, nil
	}
	var (
		pubkeydata = (*C.uchar)(unsafe.Pointer(&pubkey[0]))
		pubkeylen  = C.size_t(len(pubkey))
		out        = make([]byte, 65)
		outdata    = (*C.uchar)(unsafe.Pointer(&out[0]))
		outlen     = C.size_t(len(out))
	)
	if C.secp256k1_ext_reencode_pubkey(context, outdata, outlen, pubkeydata, pubkeylen) == 0 {
		return nil, nil
	}
	return new(big.Int).SetBytes(out[1:33]), new(big.Int).SetBytes(out[33:])
}

// CompressPubkey encodes a public key to 33-byte compressed format.
func CompressPubkey(x, y *big.Int) []byte {
	var (
		pubkey     = S256().Marshal(x, y)
		pubkeydata = (*C.uchar)(unsafe.Pointer(&pubkey[0]))
		pubkeylen  = C.size_t(len(pubkey))
		out        = make([]byte, 33)
		outdata    = (*C.uchar)(unsafe.Pointer(&out[0]))
		outlen     = C.size_t(len(out))
	)
	if C.secp256k1_ext_reencode_pubkey(context, outdata, outlen, pubkeydata, pubkeylen) == 0 {
		panic("libsecp256k1 error")
	}
	return out
}

func checkSignature(sig []byte) error {
	if len(sig) != 65 {
		return ErrInvalidSignatureLen
	}
	if sig[64] >= 4 {
		return ErrInvalidRecoveryID
	}
	return nil
}
