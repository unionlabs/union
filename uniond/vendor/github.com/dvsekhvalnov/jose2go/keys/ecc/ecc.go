//package ecc provides helpers for creating elliptic curve leys
package ecc

import (
	"math/big"
	"crypto/ecdsa"
	"crypto/elliptic"	
    "crypto/x509"
	"encoding/pem"	
	"errors"
)

// ReadPublic loads ecdsa.PublicKey from given PKCS1 X509 or PKIX blobs
func ReadPublic(raw []byte) (key *ecdsa.PublicKey,err error) {
	var encoded *pem.Block
	
	if encoded, _ = pem.Decode(raw); encoded == nil {
		return nil, errors.New("Ecc.ReadPublic(): Key must be PEM encoded PKCS1 X509 certificate or PKIX EC public key")
	}
		
	var parsedKey interface{}
	var cert *x509.Certificate
	
	if parsedKey, err = x509.ParsePKIXPublicKey(encoded.Bytes); err != nil {
		if cert,err = x509.ParseCertificate(encoded.Bytes);err!=nil {
			return nil, err
		}
		
		parsedKey=cert.PublicKey
	}
	
	var ok bool
	
	if key, ok = parsedKey.(*ecdsa.PublicKey); !ok {
		return nil, errors.New("Ecc.ReadPublic(): Key is not a valid *ecdsa.PublicKey")
	}
	
	return key, nil
}

// ReadPrivate loads ecdsa.PrivateKey from given PKCS1 or PKCS8 blobs
func ReadPrivate(raw []byte) (key *ecdsa.PrivateKey,err error) {
	var encoded *pem.Block

	if encoded, _ = pem.Decode(raw); encoded == nil {
		return nil, errors.New("Ecc.ReadPrivate(): Key must be PEM encoded PKCS1 or PKCS8 EC private key")
	}

	var parsedKey interface{}

	if parsedKey,err=x509.ParseECPrivateKey(encoded.Bytes);err!=nil {
		if parsedKey, err = x509.ParsePKCS8PrivateKey(encoded.Bytes);err!=nil {
			return nil,err
		}
	}

	var ok bool
		
	if key,ok=parsedKey.(*ecdsa.PrivateKey);!ok {
		return nil, errors.New("Ecc.ReadPrivate(): Key is not valid *ecdsa.PrivateKey")
	}
	
	return key,nil
}

// NewPublic constructs ecdsa.PublicKey from given (X,Y)
func NewPublic(x,y []byte) (*ecdsa.PublicKey) {
	return &ecdsa.PublicKey{ Curve: curve(len(x)), 
							 X:new(big.Int).SetBytes(x), 
							 Y:new(big.Int).SetBytes(y) }
}

// NewPrivate constructs ecdsa.PrivateKey from given (X,Y) and D
func NewPrivate(x,y,d []byte) (*ecdsa.PrivateKey) {
	return &ecdsa.PrivateKey {D:new(big.Int).SetBytes(d),
							  PublicKey: ecdsa.PublicKey{ Curve:curve(len(x)),
								  						  X:new(big.Int).SetBytes(x), 
														  Y:new(big.Int).SetBytes(y)}}
}

func curve(size int) (elliptic.Curve) {
	switch size {
		case 32: return elliptic.P256()
		case 48: return elliptic.P384()
		case 65,66: return elliptic.P521() //adjust for P-521 curve, which can be 65 or 66 bytes
		default: return nil //unsupported curve
	}
}