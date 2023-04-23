package jose

import (
	"crypto/rand"
	"crypto/rsa"
	"errors"
)

func init() {
	RegisterJws(&RsaUsingSha{keySizeBits: 256})
	RegisterJws(&RsaUsingSha{keySizeBits: 384})
	RegisterJws(&RsaUsingSha{keySizeBits: 512})
}

// RSA using SHA signature algorithm implementation
type RsaUsingSha struct{
	keySizeBits int
}

func (alg *RsaUsingSha) Name() string {
	switch alg.keySizeBits {
		case 256: return RS256
		case 384: return RS384
		default: return  RS512
	}
}

func (alg *RsaUsingSha) Verify(securedInput, signature []byte, key interface{}) error {
		
	if pubKey,ok:=key.(*rsa.PublicKey);ok {
		return rsa.VerifyPKCS1v15(pubKey, hashFunc(alg.keySizeBits), sha(alg.keySizeBits, securedInput), signature)	
	}
	
	return errors.New("RsaUsingSha.Verify(): expects key to be '*rsa.PublicKey'")		
}

func (alg *RsaUsingSha) Sign(securedInput []byte, key interface{}) (signature []byte, err error) {
	
	if privKey,ok:=key.(*rsa.PrivateKey);ok {
		return rsa.SignPKCS1v15(rand.Reader, privKey, hashFunc(alg.keySizeBits), sha(alg.keySizeBits, securedInput))
	}
	
	return nil,errors.New("RsaUsingSha.Sign(): expects key to be '*rsa.PrivateKey'")		
}

func sha(keySizeBits int, input []byte) (hash []byte) {
	hasher := hashAlg(keySizeBits)
	hasher.Write(input)
	return hasher.Sum(nil)	
}