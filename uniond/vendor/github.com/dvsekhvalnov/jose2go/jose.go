//Package jose provides high level functions for producing (signing, encrypting and
// compressing) or consuming (decoding) Json Web Tokens using Java Object Signing and Encryption spec
package jose

import (
	"encoding/json"
	"errors"
	"fmt"

	"github.com/dvsekhvalnov/jose2go/compact"
)

const (
	NONE = "none" //plaintext (unprotected) without signature / encryption

	HS256 = "HS256" //HMAC using SHA-256 hash
	HS384 = "HS384" //HMAC using SHA-384 hash
	HS512 = "HS512" //HMAC using SHA-512 hash
	RS256 = "RS256" //RSASSA-PKCS-v1_5 using SHA-256 hash
	RS384 = "RS384" //RSASSA-PKCS-v1_5 using SHA-384 hash
	RS512 = "RS512" //RSASSA-PKCS-v1_5 using SHA-512 hash
	PS256 = "PS256" //RSASSA-PSS using SHA-256 hash
	PS384 = "PS384" //RSASSA-PSS using SHA-384 hash
	PS512 = "PS512" //RSASSA-PSS using SHA-512 hash
	ES256 = "ES256" //ECDSA using P-256 curve and SHA-256 hash
	ES384 = "ES384" //ECDSA using P-384 curve and SHA-384 hash
	ES512 = "ES512" //ECDSA using P-521 curve and SHA-512 hash

	A128CBC_HS256 = "A128CBC-HS256" //AES in CBC mode with PKCS #5 (NIST.800-38A) padding with HMAC using 256 bit key
	A192CBC_HS384 = "A192CBC-HS384" //AES in CBC mode with PKCS #5 (NIST.800-38A) padding with HMAC using 384 bit key
	A256CBC_HS512 = "A256CBC-HS512" //AES in CBC mode with PKCS #5 (NIST.800-38A) padding with HMAC using 512 bit key
	A128GCM       = "A128GCM"       //AES in GCM mode with 128 bit key
	A192GCM       = "A192GCM"       //AES in GCM mode with 192 bit key
	A256GCM       = "A256GCM"       //AES in GCM mode with 256 bit key

	DIR                = "dir"                //Direct use of pre-shared symmetric key
	RSA1_5             = "RSA1_5"             //RSAES with PKCS #1 v1.5 padding, RFC 3447
	RSA_OAEP           = "RSA-OAEP"           //RSAES using Optimal Assymetric Encryption Padding, RFC 3447
	RSA_OAEP_256       = "RSA-OAEP-256"       //RSAES using Optimal Assymetric Encryption Padding with SHA-256, RFC 3447
	A128KW             = "A128KW"             //AES Key Wrap Algorithm using 128 bit keys, RFC 3394
	A192KW             = "A192KW"             //AES Key Wrap Algorithm using 192 bit keys, RFC 3394
	A256KW             = "A256KW"             //AES Key Wrap Algorithm using 256 bit keys, RFC 3394
	A128GCMKW          = "A128GCMKW"          //AES GCM Key Wrap Algorithm using 128 bit keys
	A192GCMKW          = "A192GCMKW"          //AES GCM Key Wrap Algorithm using 192 bit keys
	A256GCMKW          = "A256GCMKW"          //AES GCM Key Wrap Algorithm using 256 bit keys
	PBES2_HS256_A128KW = "PBES2-HS256+A128KW" //Password Based Encryption using PBES2 schemes with HMAC-SHA and AES Key Wrap using 128 bit key
	PBES2_HS384_A192KW = "PBES2-HS384+A192KW" //Password Based Encryption using PBES2 schemes with HMAC-SHA and AES Key Wrap using 192 bit key
	PBES2_HS512_A256KW = "PBES2-HS512+A256KW" //Password Based Encryption using PBES2 schemes with HMAC-SHA and AES Key Wrap using 256 bit key
	ECDH_ES            = "ECDH-ES"            //Elliptic Curve Diffie Hellman key agreement
	ECDH_ES_A128KW     = "ECDH-ES+A128KW"     //Elliptic Curve Diffie Hellman key agreement with AES Key Wrap using 128 bit key
	ECDH_ES_A192KW     = "ECDH-ES+A192KW"     //Elliptic Curve Diffie Hellman key agreement with AES Key Wrap using 192 bit key
	ECDH_ES_A256KW     = "ECDH-ES+A256KW"     //Elliptic Curve Diffie Hellman key agreement with AES Key Wrap using 256 bit key

	DEF = "DEF" //DEFLATE compression, RFC 1951
)

var jwsHashers = map[string]JwsAlgorithm{}
var jweEncryptors = map[string]JweEncryption{}
var jwaAlgorithms = map[string]JwaAlgorithm{}
var jwcCompressors = map[string]JwcAlgorithm{}

// RegisterJwe register new encryption algorithm
func RegisterJwe(alg JweEncryption) {
	jweEncryptors[alg.Name()] = alg
}

// RegisterJwa register new key management algorithm
func RegisterJwa(alg JwaAlgorithm) {
	jwaAlgorithms[alg.Name()] = alg
}

// RegisterJws register new signing algorithm
func RegisterJws(alg JwsAlgorithm) {
	jwsHashers[alg.Name()] = alg
}

// RegisterJwc register new compression algorithm
func RegisterJwc(alg JwcAlgorithm) {
	jwcCompressors[alg.Name()] = alg
}

// JweEncryption is a contract for implementing encryption algorithm
type JweEncryption interface {
	Encrypt(aad, plainText, cek []byte) (iv, cipherText, authTag []byte, err error)
	Decrypt(aad, cek, iv, cipherText, authTag []byte) (plainText []byte, err error)
	KeySizeBits() int
	Name() string
}

// JwaAlgorithm is a contract for implementing key management algorithm
type JwaAlgorithm interface {
	WrapNewKey(cekSizeBits int, key interface{}, header map[string]interface{}) (cek []byte, encryptedCek []byte, err error)
	Unwrap(encryptedCek []byte, key interface{}, cekSizeBits int, header map[string]interface{}) (cek []byte, err error)
	Name() string
}

// JwsAlgorithm is a contract for implementing signing algorithm
type JwsAlgorithm interface {
	Verify(securedInput, signature []byte, key interface{}) error
	Sign(securedInput []byte, key interface{}) (signature []byte, err error)
	Name() string
}

// JwcAlgorithm is a contract for implementing compression algorithm
type JwcAlgorithm interface {
	Compress(plainText []byte) []byte
	Decompress(compressedText []byte) []byte
	Name() string
}

func Zip(alg string) func(cfg *JoseConfig) {
	return func(cfg *JoseConfig) {
		cfg.CompressionAlg = alg
	}
}

func Header(name string, value interface{}) func(cfg *JoseConfig) {
	return func(cfg *JoseConfig) {
		cfg.Headers[name] = value
	}
}

func Headers(headers map[string]interface{}) func(cfg *JoseConfig) {
	return func(cfg *JoseConfig) {
		for k, v := range headers {
			cfg.Headers[k] = v
		}
	}
}

type JoseConfig struct {
	CompressionAlg string
	Headers        map[string]interface{}
}

// Sign produces signed JWT token given arbitrary string payload, signature algorithm to use (see constants for list of supported algs), signing key and extra options (see option functions)
// Signing key is of different type for different signing alg, see specific
// signing alg implementation documentation.
//
// It returns 3 parts signed JWT token as string and not nil error if something went wrong.
func Sign(payload string, signingAlg string, key interface{}, options ...func(*JoseConfig)) (token string, err error) {
	return SignBytes([]byte(payload), signingAlg, key, options...)
}

// Sign produces signed JWT token given arbitrary binary payload, signature algorithm to use (see constants for list of supported algs), signing key and extra options (see option functions)
// Signing key is of different type for different signing alg, see specific
// signing alg implementation documentation.
//
// It returns 3 parts signed JWT token as string and not nil error if something went wrong.
func SignBytes(payload []byte, signingAlg string, key interface{}, options ...func(*JoseConfig)) (token string, err error) {
	if signer, ok := jwsHashers[signingAlg]; ok {

		cfg := &JoseConfig{CompressionAlg: "", Headers: make(map[string]interface{})}

		//apply extra options
		for _, option := range options {
			option(cfg)
		}

		//make sure defaults and requires are managed by us
		cfg.Headers["alg"] = signingAlg

		paloadBytes := payload
		var header []byte
		var signature []byte

		if header, err = json.Marshal(cfg.Headers); err == nil {
			securedInput := []byte(compact.Serialize(header, paloadBytes))

			if signature, err = signer.Sign(securedInput, key); err == nil {
				return compact.Serialize(header, paloadBytes, signature), nil
			}
		}

		return "", err
	}

	return "", errors.New(fmt.Sprintf("jwt.Sign(): unknown algorithm: '%v'", signingAlg))
}

// Encrypt produces encrypted JWT token given arbitrary string payload, key management and encryption algorithms to use (see constants for list of supported algs) and management key.
// Management key is of different type for different key management alg, see specific
// key management alg implementation documentation.
//
// It returns 5 parts encrypted JWT token as string and not nil error if something went wrong.
func Encrypt(payload string, alg string, enc string, key interface{}, options ...func(*JoseConfig)) (token string, err error) {
	return EncryptBytes([]byte(payload), alg, enc, key, options...)
}

// Encrypt produces encrypted JWT token given arbitrary binary payload, key management and encryption algorithms to use (see constants for list of supported algs) and management key.
// Management key is of different type for different key management alg, see specific
// key management alg implementation documentation.
//
// It returns 5 parts encrypted JWT token as string and not nil error if something went wrong.
func EncryptBytes(payload []byte, alg string, enc string, key interface{}, options ...func(*JoseConfig)) (token string, err error) {

	cfg := &JoseConfig{CompressionAlg: "", Headers: make(map[string]interface{})}

	//apply extra options
	for _, option := range options {
		option(cfg)
	}

	//make sure required headers are managed by us
	cfg.Headers["alg"] = alg
	cfg.Headers["enc"] = enc

	byteContent := payload

	if cfg.CompressionAlg != "" {
		if zipAlg, ok := jwcCompressors[cfg.CompressionAlg]; ok {
			byteContent = zipAlg.Compress([]byte(payload))
			cfg.Headers["zip"] = cfg.CompressionAlg
		} else {
			return "", errors.New(fmt.Sprintf("jwt.Compress(): Unknown compression method '%v'", cfg.CompressionAlg))
		}

	} else {
		delete(cfg.Headers, "zip") //we not allow to manage 'zip' header manually for encryption
	}

	return encrypt(byteContent, cfg.Headers, key)
}

// This method is DEPRICATED and subject to be removed in next version.
// Use Encrypt(..) with Zip option instead.
//
// Compress produces encrypted & comressed JWT token given arbitrary payload, key management , encryption and compression algorithms to use (see constants for list of supported algs) and management key.
// Management key is of different type for different key management alg, see specific
// key management alg implementation documentation.
//
// It returns 5 parts encrypted & compressed JWT token as string and not nil error if something went wrong.
func Compress(payload string, alg string, enc string, zip string, key interface{}) (token string, err error) {

	if zipAlg, ok := jwcCompressors[zip]; ok {
		compressed := zipAlg.Compress([]byte(payload))

		jwtHeader := map[string]interface{}{
			"enc": enc,
			"alg": alg,
			"zip": zip,
		}

		return encrypt(compressed, jwtHeader, key)
	}

	return "", errors.New(fmt.Sprintf("jwt.Compress(): Unknown compression method '%v'", zip))
}

// Decode verifies, decrypts and decompresses given JWT token using management key.
// Management key is of different type for different key management or signing algorithms, see specific alg implementation documentation.
//
// Returns decoded payload as a string, headers and not nil error if something went wrong.
func Decode(token string, key interface{}) (string, map[string]interface{}, error) {

	payload, headers, err := DecodeBytes(token, key)

	if err != nil {
		return "", nil, err
	}

	return string(payload), headers, nil
}

// Decode verifies, decrypts and decompresses given JWT token using management key.
// Management key is of different type for different key management or signing algorithms, see specific alg implementation documentation.
//
// Returns decoded payload as a raw bytes, headers and not nil error if something went wrong.
func DecodeBytes(token string, key interface{}) ([]byte, map[string]interface{}, error) {
	parts, err := compact.Parse(token)

	if err != nil {
		return nil, nil, err
	}

	if len(parts) == 3 {
		return verify(parts, key)
	}

	if len(parts) == 5 {
		return decrypt(parts, key)
	}

	return nil, nil, errors.New(fmt.Sprintf("jwt.DecodeBytes() expects token of 3 or 5 parts, but was given: %v parts", len(parts)))
}

func encrypt(payload []byte, jwtHeader map[string]interface{}, key interface{}) (token string, err error) {
	var ok bool
	var keyMgmtAlg JwaAlgorithm
	var encAlg JweEncryption

	alg := jwtHeader["alg"].(string)
	enc := jwtHeader["enc"].(string)

	if keyMgmtAlg, ok = jwaAlgorithms[alg]; !ok {
		return "", errors.New(fmt.Sprintf("jwt.encrypt(): Unknown key management algorithm '%v'", alg))
	}

	if encAlg, ok = jweEncryptors[enc]; !ok {
		return "", errors.New(fmt.Sprintf("jwt.encrypt(): Unknown encryption algorithm '%v'", enc))
	}

	var cek, encryptedCek, header, iv, cipherText, authTag []byte

	if cek, encryptedCek, err = keyMgmtAlg.WrapNewKey(encAlg.KeySizeBits(), key, jwtHeader); err != nil {
		return "", err
	}

	if header, err = json.Marshal(jwtHeader); err != nil {
		return "", err
	}

	if iv, cipherText, authTag, err = encAlg.Encrypt([]byte(compact.Serialize(header)), payload, cek); err != nil {
		return "", err
	}

	return compact.Serialize(header, encryptedCek, iv, cipherText, authTag), nil
}

func verify(parts [][]byte, key interface{}) (plainText []byte, headers map[string]interface{}, err error) {

	header, payload, signature := parts[0], parts[1], parts[2]

	secured := []byte(compact.Serialize(header, payload))

	var jwtHeader map[string]interface{}

	if err = json.Unmarshal(header, &jwtHeader); err != nil {
		return nil, nil, err
	}

	if alg, ok := jwtHeader["alg"].(string); ok {
		if verifier, ok := jwsHashers[alg]; ok {
			if key, err = retrieveActualKey(jwtHeader, string(payload), key); err != nil {
				return nil, nil, err
			}

			if err = verifier.Verify(secured, signature, key); err == nil {
				return payload, jwtHeader, nil
			}

			return nil, nil, err
		}

		return nil, nil, errors.New(fmt.Sprintf("jwt.Decode(): Unknown algorithm: '%v'", alg))
	}

	return nil, nil, errors.New(fmt.Sprint("jwt.Decode(): required 'alg' header is missing or of invalid type"))
}

func decrypt(parts [][]byte, key interface{}) (plainText []byte, headers map[string]interface{}, err error) {

	header, encryptedCek, iv, cipherText, authTag := parts[0], parts[1], parts[2], parts[3], parts[4]

	var jwtHeader map[string]interface{}

	if e := json.Unmarshal(header, &jwtHeader); e != nil {
		return nil, nil, e
	}

	var keyMgmtAlg JwaAlgorithm
	var encAlg JweEncryption
	var zipAlg JwcAlgorithm
	var cek, plainBytes []byte
	var ok bool
	var alg, enc string

	if alg, ok = jwtHeader["alg"].(string); !ok {
		return nil, nil, errors.New(fmt.Sprint("jwt.Decode(): required 'alg' header is missing or of invalid type"))
	}

	if enc, ok = jwtHeader["enc"].(string); !ok {
		return nil, nil, errors.New(fmt.Sprint("jwt.Decode(): required 'enc' header is missing or of invalid type"))
	}

	aad := []byte(compact.Serialize(header))

	if keyMgmtAlg, ok = jwaAlgorithms[alg]; ok {
		if encAlg, ok = jweEncryptors[enc]; ok {

			if key, err = retrieveActualKey(jwtHeader, string(cipherText), key); err != nil {
				return nil, nil, err
			}

			if cek, err = keyMgmtAlg.Unwrap(encryptedCek, key, encAlg.KeySizeBits(), jwtHeader); err == nil {
				if plainBytes, err = encAlg.Decrypt(aad, cek, iv, cipherText, authTag); err == nil {

					if zip, compressed := jwtHeader["zip"].(string); compressed {

						if zipAlg, ok = jwcCompressors[zip]; !ok {
							return nil, nil, errors.New(fmt.Sprintf("jwt.decrypt(): Unknown compression algorithm '%v'", zip))
						}

						plainBytes = zipAlg.Decompress(plainBytes)
					}

					return plainBytes, jwtHeader, nil
				}

				return nil, nil, err
			}

			return nil, nil, err
		}

		return nil, nil, errors.New(fmt.Sprintf("jwt.decrypt(): Unknown encryption algorithm '%v'", enc))
	}

	return nil, nil, errors.New(fmt.Sprintf("jwt.decrypt(): Unknown key management algorithm '%v'", alg))
}

func retrieveActualKey(headers map[string]interface{}, payload string, key interface{}) (interface{}, error) {
	if keyCallback, ok := key.(func(headers map[string]interface{}, payload string) interface{}); ok {
		result := keyCallback(headers, payload)

		if err, ok := result.(error); ok {
			return nil, err
		}

		return result, nil
	}

	return key, nil
}
