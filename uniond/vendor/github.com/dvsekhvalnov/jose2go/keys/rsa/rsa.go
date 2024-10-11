//package Rsa provides helpers for creating rsa leys
package Rsa

import (
		"crypto/rsa"
	    "crypto/x509"
		"encoding/pem"
		"errors"
)

// ReadPrivate loads rsa.PrivateKey from PKCS1 or PKCS8 blobs 
func ReadPrivate(raw []byte) (key *rsa.PrivateKey,err error) {	
	var encoded *pem.Block

	if encoded, _ = pem.Decode(raw); encoded == nil {
		return nil, errors.New("Rsa.NewPrivate(): Key must be PEM encoded PKCS1 or PKCS8 private key")
	}

	var parsedKey interface{}

	if parsedKey,err=x509.ParsePKCS1PrivateKey(encoded.Bytes);err!=nil {
		if parsedKey, err = x509.ParsePKCS8PrivateKey(encoded.Bytes);err!=nil {
			return nil,err
		}
	}

	var ok bool
		
	if key,ok=parsedKey.(*rsa.PrivateKey);!ok {
		return nil, errors.New("Rsa.NewPrivate(): Key is not valid *rsa.PrivateKey")
	}
	
	return key,nil
}

// ReadPublic loads rsa.PublicKey from PKIX or PKCS1 X509 blobs
func ReadPublic(raw []byte) (key *rsa.PublicKey,err error)  {
	var encoded *pem.Block
	
	if encoded, _ = pem.Decode(raw); encoded == nil {
		return nil, errors.New("Rsa.NewPublic(): Key must be PEM encoded PKCS1 X509 certificate or PKIX public key")
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
	
	if key, ok = parsedKey.(*rsa.PublicKey); !ok {
		return nil, errors.New("Rsa.NewPublic(): Key is not a valid RSA public key")
	}
	
	return key, nil
}