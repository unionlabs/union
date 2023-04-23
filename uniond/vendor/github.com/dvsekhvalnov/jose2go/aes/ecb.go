// Package aes contains provides AES Key Wrap and ECB mode implementations
package aes

import (
	"crypto/cipher"
)

type ecb struct {
	b cipher.Block
}

type ecbEncrypter ecb
type ecbDecrypter ecb

// NewECBEncrypter creates BlockMode for AES encryption in ECB mode
func NewECBEncrypter(b cipher.Block) cipher.BlockMode {
	return &ecbEncrypter{b: b}
}

// NewECBDecrypter creates BlockMode for AES decryption in ECB mode
func NewECBDecrypter(b cipher.Block) cipher.BlockMode {
	return &ecbDecrypter{b: b}
}

func (x *ecbEncrypter) BlockSize() int { return x.b.BlockSize() }
func (x *ecbDecrypter) BlockSize() int { return x.b.BlockSize() }

func (x *ecbDecrypter) CryptBlocks(dst, src []byte) {
	bs := x.BlockSize()

	if len(src)%bs != 0 {
		panic("ecbDecrypter.CryptBlocks(): input not full blocks")
	}

	if len(dst) < len(src) {
		panic("ecbDecrypter.CryptBlocks(): output smaller than input")
	}

	if len(src) == 0 {
		return
	}

	for len(src) > 0 {
		x.b.Decrypt(dst, src)
		src = src[bs:]
	}
}

func (x *ecbEncrypter) CryptBlocks(dst, src []byte) {
	bs := x.BlockSize()

	if len(src)%bs != 0 {
		panic("ecbEncrypter.CryptBlocks(): input not full blocks")
	}

	if len(dst) < len(src) {
		panic("ecbEncrypter.CryptBlocks(): output smaller than input")
	}

	if len(src) == 0 {
		return
	}

	for len(src) > 0 {
		x.b.Encrypt(dst, src)
		src = src[bs:]
	}
}
