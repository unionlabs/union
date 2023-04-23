package jose

import (
	"crypto/hmac"
	"errors"
)

func init() {
	RegisterJws(&HmacUsingSha{keySizeBits: 256})
	RegisterJws(&HmacUsingSha{keySizeBits: 384})
	RegisterJws(&HmacUsingSha{keySizeBits: 512})		
}

// HMAC with SHA signing algorithm implementation
type HmacUsingSha struct{
	keySizeBits int
}

func (alg *HmacUsingSha) Name() string {
	switch alg.keySizeBits {
		case 256: return HS256
		case 384: return HS384
		default: return  HS512
	}
}

func (alg *HmacUsingSha) Verify(securedInput, signature []byte, key interface{}) error {	
	
	actualSig,_ := alg.Sign(securedInput, key)

	if !hmac.Equal(signature, actualSig) { 
		return errors.New("HmacUsingSha.Verify(): Signature is invalid")
	}
	
	return nil
}

func (alg *HmacUsingSha) Sign(securedInput []byte, key interface{}) (signature []byte, err error) {
	//TODO: assert min key size
		
	if pubKey,ok:=key.([]byte); ok {		
		return calculateHmac(alg.keySizeBits, securedInput, pubKey),nil
	}
	
	return nil,errors.New("HmacUsingSha.Sign(): expects key to be '[]byte' array")	
}