package jose

import (
	"crypto/rand"
	"math/big"
	"crypto/ecdsa"
	"errors"
	"github.com/dvsekhvalnov/jose2go/arrays"
	"github.com/dvsekhvalnov/jose2go/padding"
	"fmt"
)

func init() {
	RegisterJws(&EcdsaUsingSha{keySizeBits: 256, hashSizeBits: 256})
	RegisterJws(&EcdsaUsingSha{keySizeBits: 384, hashSizeBits: 384})
	RegisterJws(&EcdsaUsingSha{keySizeBits: 521, hashSizeBits: 512})
}

// ECDSA signing algorithm implementation
type EcdsaUsingSha struct{
	keySizeBits int
	hashSizeBits int
}

func (alg *EcdsaUsingSha) Name() string {
	switch alg.keySizeBits {
		case 256: return ES256
		case 384: return ES384
		default: return  ES512
	}
}

func (alg *EcdsaUsingSha) Verify(securedInput, signature []byte, key interface{}) error {
	
	if pubKey,ok:=key.(*ecdsa.PublicKey);ok {
				
		if sizeBits:=pubKey.Curve.Params().BitSize;sizeBits!=alg.keySizeBits {
			return errors.New(fmt.Sprintf("EcdsaUsingSha.Verify(): expected key of size %v bits, but was given %v bits.",alg.keySizeBits,sizeBits))
		}
				
		r:=new(big.Int).SetBytes(signature[:len(signature)/2])
		s:=new(big.Int).SetBytes(signature[len(signature)/2:])
		
		if ok:=ecdsa.Verify(pubKey, sha(alg.hashSizeBits, securedInput), r,s); ok {
			return nil
		}
		
		return errors.New("EcdsaUsingSha.Verify(): Signature is not valid.")		
	}
	
	return errors.New("EcdsaUsingSha.Verify(): expects key to be '*ecdsa.PublicKey'")
}

func (alg *EcdsaUsingSha) Sign(securedInput []byte, key interface{}) (signature []byte, err error) {
	
	if privKey,ok := key.(*ecdsa.PrivateKey);ok {
		
		if sizeBits:=privKey.Curve.Params().BitSize;sizeBits!=alg.keySizeBits {
			return nil,errors.New(fmt.Sprintf("EcdsaUsingSha.Sign(): expected key of size %v bits, but was given %v bits.",alg.keySizeBits,sizeBits))
		}		
		
		var r,s *big.Int
		
		if r,s,err = ecdsa.Sign(rand.Reader, privKey, sha(alg.hashSizeBits, securedInput));err==nil {		
			
			rBytes:=padding.Align(r.Bytes(), alg.keySizeBits)
			sBytes:=padding.Align(s.Bytes(), alg.keySizeBits)
			
			return arrays.Concat(rBytes,sBytes),nil
		}
		
		return nil, err		
	}

	return nil,errors.New("EcdsaUsingSha.Sign(): expects key to be '*ecdsa.PrivateKey'")
}