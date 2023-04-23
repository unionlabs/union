package jose

import (
	"errors"
	"crypto/rsa"
	"crypto/rand"
	"github.com/dvsekhvalnov/jose2go/arrays"
)

func init() {
	RegisterJwa(new(RsaPkcs1v15))
}

// RS-AES using PKCS #1 v1.5 padding key management algorithm implementation
type RsaPkcs1v15 struct{
}

func (alg *RsaPkcs1v15) Name() string {
	return RSA1_5
}

func (alg *RsaPkcs1v15) WrapNewKey(cekSizeBits int, key interface{}, header map[string]interface{}) (cek []byte, encryptedCek []byte, err error) {
	if pubKey,ok:=key.(*rsa.PublicKey);ok {		
		if cek,err = arrays.Random(cekSizeBits>>3);err==nil {			
			encryptedCek,err=rsa.EncryptPKCS1v15(rand.Reader,pubKey,cek)			
			return
		}
		
		return nil,nil,err
	}
	
	return nil,nil,errors.New("RsaPkcs1v15.WrapNewKey(): expected key to be '*rsa.PublicKey'")		
}

func (alg *RsaPkcs1v15) Unwrap(encryptedCek []byte, key interface{}, cekSizeBits int, header map[string]interface{}) (cek []byte, err error) {
	if privKey,ok:=key.(*rsa.PrivateKey);ok {
		return rsa.DecryptPKCS1v15(rand.Reader,privKey,encryptedCek)
	}
	
	return nil,errors.New("RsaPkcs1v15.Unwrap(): expected key to be '*rsa.PrivateKey'")		
}
