package jose

import (
	"crypto/sha256"
	"crypto/sha512"
	"errors"
	"hash"

	"github.com/dvsekhvalnov/jose2go/arrays"
	"github.com/dvsekhvalnov/jose2go/base64url"
	"github.com/dvsekhvalnov/jose2go/kdf"
)

func init() {
	RegisterJwa(&Pbse2HmacAesKW{keySizeBits: 128, aesKW: &AesKW{keySizeBits: 128}})
	RegisterJwa(&Pbse2HmacAesKW{keySizeBits: 192, aesKW: &AesKW{keySizeBits: 192}})
	RegisterJwa(&Pbse2HmacAesKW{keySizeBits: 256, aesKW: &AesKW{keySizeBits: 256}})
}

// PBSE2 with HMAC key management algorithm implementation
type Pbse2HmacAesKW struct {
	keySizeBits int
	aesKW       JwaAlgorithm
}

func (alg *Pbse2HmacAesKW) Name() string {
	switch alg.keySizeBits {
	case 128:
		return PBES2_HS256_A128KW
	case 192:
		return PBES2_HS384_A192KW
	default:
		return PBES2_HS512_A256KW
	}
}

func (alg *Pbse2HmacAesKW) WrapNewKey(cekSizeBits int, key interface{}, header map[string]interface{}) (cek []byte, encryptedCek []byte, err error) {
	if passphrase, ok := key.(string); ok {

		algId := []byte(header["alg"].(string))

		iterationCount := 8192
		var saltInput []byte

		if saltInput, err = arrays.Random(12); err != nil {
			return nil, nil, err
		}

		header["p2c"] = iterationCount
		header["p2s"] = base64url.Encode(saltInput)

		salt := arrays.Concat(algId, []byte{0}, saltInput)

		kek := kdf.DerivePBKDF2([]byte(passphrase), salt, iterationCount, alg.keySizeBits, alg.prf)
		return alg.aesKW.WrapNewKey(cekSizeBits, kek, header)
	}

	return nil, nil, errors.New("Pbse2HmacAesKW.WrapNewKey(): expected key to be 'string' array")
}

func (alg *Pbse2HmacAesKW) Unwrap(encryptedCek []byte, key interface{}, cekSizeBits int, header map[string]interface{}) (cek []byte, err error) {

	if passphrase, ok := key.(string); ok {

		var p2s string
		var p2c float64

		if p2c, ok = header["p2c"].(float64); !ok {
			return nil, errors.New("Pbse2HmacAesKW.Unwrap(): expected 'p2c' param in JWT header, but was not found.")
		}

		if p2s, ok = header["p2s"].(string); !ok {
			return nil, errors.New("Pbse2HmacAesKW.Unwrap(): expected 'p2s' param in JWT header, but was not found.")
		}

		var saltInput []byte

		algId := []byte(header["alg"].(string))

		if saltInput, err = base64url.Decode(p2s); err != nil {
			return nil, err
		}

		salt := arrays.Concat(algId, []byte{0}, saltInput)

		kek := kdf.DerivePBKDF2([]byte(passphrase), salt, int(p2c), alg.keySizeBits, alg.prf)

		return alg.aesKW.Unwrap(encryptedCek, kek, cekSizeBits, header)
	}

	return nil, errors.New("Pbse2HmacAesKW.Unwrap(): expected key to be 'string' array")
}

func (alg *Pbse2HmacAesKW) prf() hash.Hash {
	switch alg.keySizeBits {
	case 128:
		return sha256.New()
	case 192:
		return sha512.New384()
	default:
		return sha512.New()
	}
}
