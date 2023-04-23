package jose

import (
	"crypto/ecdsa"
	"crypto/elliptic"
	"crypto/rand"
	"crypto/sha256"
	"errors"
	"fmt"
	"github.com/dvsekhvalnov/jose2go/arrays"
	"github.com/dvsekhvalnov/jose2go/base64url"
	"github.com/dvsekhvalnov/jose2go/kdf"
	"github.com/dvsekhvalnov/jose2go/keys/ecc"
	"github.com/dvsekhvalnov/jose2go/padding"
	"math/big"
)

func init() {
	RegisterJwa(&Ecdh{directAgreement: true})
}

// Elliptic curve Diffie–Hellman key management (key agreement) algorithm implementation
type Ecdh struct {
	directAgreement bool
}

func (alg *Ecdh) Name() string {
	return ECDH_ES
}

func (alg *Ecdh) WrapNewKey(cekSizeBits int, key interface{}, header map[string]interface{}) (cek []byte, encryptedCek []byte, err error) {

	if pubKey, ok := key.(*ecdsa.PublicKey); ok {

		if _, ok := header[alg.idHeader()].(string); !ok {
			return nil, nil, errors.New(fmt.Sprintf("Ecdh.WrapNewKey(): expected '%v' param in JWT header, but was not found.", alg.idHeader()))
		}

		var d []byte
		var x, y *big.Int

		if d, x, y, err = elliptic.GenerateKey(pubKey.Curve, rand.Reader); err != nil {
			return nil, nil, err
		}

		ephemeral := ecc.NewPrivate(x.Bytes(), y.Bytes(), d)

		xBytes := padding.Align(x.Bytes(), pubKey.Curve.Params().BitSize)
		yBytes := padding.Align(y.Bytes(), pubKey.Curve.Params().BitSize)

		epk := map[string]string{
			"kty": "EC",
			"x":   base64url.Encode(xBytes),
			"y":   base64url.Encode(yBytes),
			"crv": name(pubKey.Curve),
		}

		header["epk"] = epk

		return alg.deriveKey(pubKey, ephemeral, cekSizeBits, header), nil, nil
	}

	return nil, nil, errors.New("Ecdh.WrapNewKey(): expected key to be '*ecdsa.PublicKey'")
}

func (alg *Ecdh) Unwrap(encryptedCek []byte, key interface{}, cekSizeBits int, header map[string]interface{}) (cek []byte, err error) {

	if privKey, ok := key.(*ecdsa.PrivateKey); ok {

		var epk map[string]interface{}

		if epk, ok = header["epk"].(map[string]interface{}); !ok {
			return nil, errors.New("Ecdh.Unwrap(): expected 'epk' param in JWT header, but was not found.")
		}

		if _, ok := header[alg.idHeader()].(string); !ok {
			return nil, errors.New(fmt.Sprintf("Ecdh.Unwrap(): expected '%v' param in JWT header, but was not found.", alg.idHeader()))
		}

		var x, y, crv string
		var xBytes, yBytes []byte

		if x, ok = epk["x"].(string); !ok {
			return nil, errors.New("Ecdh.Unwrap(): expects 'epk' key to contain 'x','y' and 'crv' fields, but 'x' was not found.")
		}

		if y, ok = epk["y"].(string); !ok {
			return nil, errors.New("Ecdh.Unwrap(): expects 'epk' key to contain 'x','y' and 'crv' fields, but 'y' was not found.")
		}

		if crv, ok = epk["crv"].(string); !ok {
			return nil, errors.New("Ecdh.Unwrap(): expects 'epk' key to contain 'x','y' and 'crv' fields, but 'crv' was not found.")
		}

		if crv != "P-256" && crv != "P-384" && crv != "P-521" {
			return nil, errors.New(fmt.Sprintf("Ecdh.Unwrap(): unknown or unsupported curve %v", crv))
		}

		if xBytes, err = base64url.Decode(x); err != nil {
			return nil, err
		}
		if yBytes, err = base64url.Decode(y); err != nil {
			return nil, err
		}

		pubKey := ecc.NewPublic(xBytes, yBytes)

		if !privKey.Curve.IsOnCurve(pubKey.X, pubKey.Y) {
			return nil, errors.New(fmt.Sprintf("Ephemeral public key received in header is invalid for reciever's private key."))
		}

		return alg.deriveKey(pubKey, privKey, cekSizeBits, header), nil
	}

	return nil, errors.New("Ecdh.Unwrap(): expected key to be '*ecdsa.PrivateKey'")
}

func (alg *Ecdh) deriveKey(pubKey *ecdsa.PublicKey, privKey *ecdsa.PrivateKey, keySizeBits int, header map[string]interface{}) []byte {

	var enc, apv, apu []byte
	var err error

	enc = []byte(header[alg.idHeader()].(string))

	if a, ok := header["apv"].(string); !ok {
		if apv, err = base64url.Decode(a); err != nil {
			apv = nil
		}
	}

	if a, ok := header["apu"].(string); !ok {
		if apu, err = base64url.Decode(a); err != nil {
			apu = nil
		}
	}

	z, _ := pubKey.Curve.ScalarMult(pubKey.X, pubKey.Y, privKey.D.Bytes())
	zBytes := padding.Align(z.Bytes(), privKey.Curve.Params().BitSize)

	return kdf.DeriveConcatKDF(keySizeBits, zBytes, prependDatalen(enc), prependDatalen(apu), prependDatalen(apv), arrays.UInt32ToBytes(uint32(keySizeBits)), nil, sha256.New())
}

func (alg *Ecdh) idHeader() string {
	if alg.directAgreement {
		return "enc"
	}

	return "alg"
}

func name(curve elliptic.Curve) string {
	return fmt.Sprintf("P-%v", curve.Params().BitSize)
}

func prependDatalen(bytes []byte) []byte {
	return arrays.Concat(arrays.UInt32ToBytes(uint32(len(bytes))), bytes)
}
