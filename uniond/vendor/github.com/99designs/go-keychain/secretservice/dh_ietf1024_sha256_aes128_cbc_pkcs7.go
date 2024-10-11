// This file implements a basic Diffie-Hellman for groups with modular
// exponentiation operators. In particular, it is used in this package
// to implement the Diffie-Hellman KEX over the Second Oakley Group.
// meant only for use for securing the channel to the D-Bus Secret Service.
// Much of the code in this file is derived from package
// golang.org/x/crypto/ssh:kex.go, and is replicated here because the relevant
// variables and methods are not exported or easily accessible.
// Note that this protocol is NOT authenticated, NOT secure against malleation
// and is NOT CCA2-secure. It is only meant to hide the D-Bus messages from any
// system services that may be logging everything.

package secretservice

import (
	"bytes"
	"crypto/aes"
	"crypto/cipher"
	cryptorand "crypto/rand"
	"crypto/sha256"
	"fmt"
	"io"
	"math/big"

	errors "github.com/pkg/errors"
	"golang.org/x/crypto/hkdf"
)

type dhGroup struct {
	g, p, pMinus1 *big.Int
}

var bigOne = big.NewInt(1)

func (group *dhGroup) NewKeypair() (private *big.Int, public *big.Int, err error) {
	for {
		if private, err = cryptorand.Int(cryptorand.Reader, group.pMinus1); err != nil {
			return nil, nil, err
		}
		if private.Sign() > 0 {
			break
		}
	}
	public = new(big.Int).Exp(group.g, private, group.p)
	return private, public, nil
}

func (group *dhGroup) diffieHellman(theirPublic, myPrivate *big.Int) (*big.Int, error) {
	if theirPublic.Cmp(bigOne) <= 0 || theirPublic.Cmp(group.pMinus1) >= 0 {
		return nil, errors.New("ssh: DH parameter out of bounds")
	}
	return new(big.Int).Exp(theirPublic, myPrivate, group.p), nil
}

func rfc2409SecondOakleyGroup() *dhGroup {
	p, _ := new(big.Int).SetString("FFFFFFFFFFFFFFFFC90FDAA22168C234C4C6628B80DC1CD129024E088A67CC74020BBEA63B139B22514A08798E3404DDEF9519B3CD3A431B302B0A6DF25F14374FE1356D6D51C245E485B576625E7EC6F44C42E9A637ED6B0BFF5CB6F406B7EDEE386BFB5A899FA5AE9F24117C4B1FE649286651ECE65381FFFFFFFFFFFFFFFF", 16)
	return &dhGroup{
		g:       new(big.Int).SetInt64(2),
		p:       p,
		pMinus1: new(big.Int).Sub(p, bigOne),
	}
}

func (group *dhGroup) keygenHKDFSHA256AES128(theirPublic *big.Int, myPrivate *big.Int) ([]byte, error) {
	sharedSecret, err := group.diffieHellman(theirPublic, myPrivate)
	if err != nil {
		return nil, err
	}
	sharedSecretBytes := sharedSecret.Bytes()

	r := hkdf.New(sha256.New, sharedSecretBytes, nil, nil)

	aesKey := make([]byte, 16)
	_, err = io.ReadFull(r, aesKey)
	if err != nil {
		return nil, err
	}

	return aesKey, nil
}

func unauthenticatedAESCBCEncrypt(unpaddedPlaintext []byte, key []byte) (iv []byte, ciphertext []byte, err error) {
	paddedPlaintext := padPKCS7(unpaddedPlaintext, aes.BlockSize)
	block, err := aes.NewCipher(key)
	if err != nil {
		return nil, nil, err
	}
	ivSize := aes.BlockSize
	iv = make([]byte, ivSize)
	ciphertext = make([]byte, len(paddedPlaintext))
	if _, err := io.ReadFull(cryptorand.Reader, iv); err != nil {
		return nil, nil, err
	}
	mode := cipher.NewCBCEncrypter(block, iv)
	mode.CryptBlocks(ciphertext, paddedPlaintext)
	return iv, ciphertext, nil
}

func unauthenticatedAESCBCDecrypt(iv []byte, ciphertext []byte, key []byte) ([]byte, error) {
	block, err := aes.NewCipher(key)
	if err != nil {
		return nil, err
	}
	if len(iv) != aes.BlockSize {
		return nil, fmt.Errorf("iv length not aes blocksize")
	}
	if len(ciphertext) < aes.BlockSize {
		return nil, fmt.Errorf("ciphertext smaller than AES block size")
	}
	if len(ciphertext)%aes.BlockSize != 0 {
		return nil, fmt.Errorf("aes ciphertext not a multiple of blocksize")
	}
	mode := cipher.NewCBCDecrypter(block, iv)
	mode.CryptBlocks(ciphertext, ciphertext) // decrypt in-place
	plaintext, err := unpadPKCS7(ciphertext, aes.BlockSize)
	if err != nil {
		return nil, err
	}
	return plaintext, nil
}

func padPKCS7(xs []byte, n int) []byte {
	m := byte(n - (len(xs) % n))
	if m == 0 {
		m = 16
	}
	return append(xs, bytes.Repeat([]byte{m}, int(m))...)
}

func unpadPKCS7(xs []byte, n int) ([]byte, error) {
	if len(xs) == 0 {
		return nil, fmt.Errorf("cannot unpad empty bytearray")
	}
	if len(xs)%n != 0 {
		return nil, fmt.Errorf("length of bytearray not a multiple of blocksize")
	}
	lastByte := xs[len(xs)-1]
	padStartIdx := len(xs) - int(lastByte)
	if padStartIdx < 0 {
		return nil, fmt.Errorf("invalid pkcs7 padding; pad byte larger than number of characters")
	}
	for i := padStartIdx; i < len(xs); i++ {
		if xs[i] != lastByte {
			return nil, fmt.Errorf("expected pad character %x, got %x", lastByte, xs[i])
		}
	}
	return xs[:padStartIdx], nil
}
