package bls

import (
	"unsafe"
)

// SecretKey

func CastFromSecretKey(in *SecretKey) *Fr {
	return (*Fr)(unsafe.Pointer(in))
}

func CastToSecretKey(in *Fr) *SecretKey {
	return (*SecretKey)(unsafe.Pointer(in))
}

// PublicKey

func CastFromPublicKey(in *PublicKey) *G1 {
	return (*G1)(unsafe.Pointer(in))
}

func CastToPublicKey(in *G1) *PublicKey {
	return (*PublicKey)(unsafe.Pointer(in))
}

// Sign

func CastFromSign(in *Sign) *G2 {
	return (*G2)(unsafe.Pointer(in))
}

func CastToSign(in *G2) *Sign {
	return (*Sign)(unsafe.Pointer(in))
}
