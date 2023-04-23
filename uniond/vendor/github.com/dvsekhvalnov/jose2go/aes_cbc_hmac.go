package jose

import (
	"crypto/aes"
	"crypto/cipher"
	"crypto/hmac"
	"errors"
	"fmt"
	"github.com/dvsekhvalnov/jose2go/arrays"
	"github.com/dvsekhvalnov/jose2go/padding"
)

// AES CBC with HMAC authenticated encryption algorithm implementation
type AesCbcHmac struct {
	keySizeBits int
}

func init() {
	RegisterJwe(&AesCbcHmac{keySizeBits: 256})
	RegisterJwe(&AesCbcHmac{keySizeBits: 384})
	RegisterJwe(&AesCbcHmac{keySizeBits: 512})
}

func (alg *AesCbcHmac) Name() string {
	switch alg.keySizeBits {
	case 256:
		return A128CBC_HS256
	case 384:
		return A192CBC_HS384
	default:
		return A256CBC_HS512
	}
}

func (alg *AesCbcHmac) KeySizeBits() int {
	return alg.keySizeBits
}

func (alg *AesCbcHmac) SetKeySizeBits(bits int) {
	alg.keySizeBits = bits
}

func (alg *AesCbcHmac) Encrypt(aad, plainText, cek []byte) (iv, cipherText, authTag []byte, err error) {

	cekSizeBits := len(cek) << 3
	if cekSizeBits != alg.keySizeBits {
		return nil, nil, nil, errors.New(fmt.Sprintf("AesCbcHmac.Encrypt(): expected key of size %v bits, but was given %v bits.", alg.keySizeBits, cekSizeBits))
	}

	hmacKey := cek[0 : len(cek)/2]
	aesKey := cek[len(cek)/2:]

	if iv, err = arrays.Random(16); err != nil {
		return nil, nil, nil, err
	}

	var block cipher.Block

	if block, err = aes.NewCipher(aesKey); err != nil {
		return nil, nil, nil, err
	}

	padded := padding.AddPkcs7(plainText, 16)

	cipherText = make([]byte, len(padded), cap(padded))
	mode := cipher.NewCBCEncrypter(block, iv)
	mode.CryptBlocks(cipherText, padded)

	authTag = alg.computeAuthTag(aad, iv, cipherText, hmacKey)

	return iv, cipherText, authTag, nil
}

func (alg *AesCbcHmac) Decrypt(aad, cek, iv, cipherText, authTag []byte) (plainText []byte, err error) {

	cekSizeBits := len(cek) << 3

	if cekSizeBits != alg.keySizeBits {
		return nil, errors.New(fmt.Sprintf("AesCbcHmac.Decrypt(): expected key of size %v bits, but was given %v bits.", alg.keySizeBits, cekSizeBits))
	}

	hmacKey := cek[0 : len(cek)/2]
	aesKey := cek[len(cek)/2:]

	// Check MAC
	expectedAuthTag := alg.computeAuthTag(aad, iv, cipherText, hmacKey)

	if !hmac.Equal(expectedAuthTag, authTag) {
		return nil, errors.New("AesCbcHmac.Decrypt(): Authentication tag do not match.")
	}

	var block cipher.Block

	if block, err = aes.NewCipher(aesKey); err == nil {
		mode := cipher.NewCBCDecrypter(block, iv)

		var padded []byte = make([]byte, len(cipherText), cap(cipherText))
		mode.CryptBlocks(padded, cipherText)

		return padding.RemovePkcs7(padded, 16), nil
	}

	return nil, err
}

func (alg *AesCbcHmac) computeAuthTag(aad []byte, iv []byte, cipherText []byte, hmacKey []byte) (signature []byte) {
	al := arrays.UInt64ToBytes(uint64(len(aad) << 3))
	hmacInput := arrays.Concat(aad, iv, cipherText, al)
	hmac := calculateHmac(alg.keySizeBits, hmacInput, hmacKey)

	return hmac[0 : len(hmac)/2]
}
