package jose

import (
	"crypto/rand"
	"crypto/rsa"
	"errors"
)

func init() {
	RegisterJws(&RsaPssUsingSha{keySizeBits: 256, saltSizeBytes: 32})
	RegisterJws(&RsaPssUsingSha{keySizeBits: 384, saltSizeBytes: 48})
	RegisterJws(&RsaPssUsingSha{keySizeBits: 512, saltSizeBytes: 64})
}

// RSA with PSS using SHA signing algorithm implementation
type RsaPssUsingSha struct{
	keySizeBits int
	saltSizeBytes int
}

func (alg *RsaPssUsingSha) Name() string {
	switch alg.keySizeBits {
		case 256: return PS256
		case 384: return PS384
		default: return  PS512
	}
}

func (alg *RsaPssUsingSha) Verify(securedInput, signature []byte, key interface{}) error {
	if pubKey,ok:=key.(*rsa.PublicKey);ok {
		return rsa.VerifyPSS(pubKey, hashFunc(alg.keySizeBits), sha(alg.keySizeBits, securedInput), signature, &rsa.PSSOptions{SaltLength:alg.saltSizeBytes})	
	}
	
	return errors.New("RsaPssUsingSha.Verify(): expects key to be '*rsa.PublicKey'")		
}

func (alg *RsaPssUsingSha) Sign(securedInput []byte, key interface{}) (signature []byte, err error) {
	if privKey,ok:=key.(*rsa.PrivateKey);ok {
		return rsa.SignPSS(rand.Reader, privKey, hashFunc(alg.keySizeBits), sha(alg.keySizeBits, securedInput), &rsa.PSSOptions{SaltLength:alg.saltSizeBytes})
	}
	
	return nil,errors.New("RsaPssUsingSha.Sign(): expects key to be '*rsa.PrivateKey'")		
}
