package jose

import (
	"errors"
	"fmt"
	"github.com/dvsekhvalnov/jose2go/aes"
	"github.com/dvsekhvalnov/jose2go/arrays"
)

func init() {
	RegisterJwa(&AesKW{ keySizeBits: 128})
	RegisterJwa(&AesKW{ keySizeBits: 192})
	RegisterJwa(&AesKW{ keySizeBits: 256})
}

// AES Key Wrap key management algorithm implementation
type AesKW struct {
	keySizeBits int
}

func (alg *AesKW) Name() string {
	switch alg.keySizeBits {
		case 128: return A128KW
		case 192: return A192KW
		default: return  A256KW
	}
}

func (alg *AesKW) WrapNewKey(cekSizeBits int, key interface{}, header map[string]interface{}) (cek []byte, encryptedCek []byte, err error) {	
	if kek,ok:=key.([]byte); ok {
		
		kekSizeBits := len(kek) << 3
		
		if kekSizeBits != alg.keySizeBits {
			return nil,nil, errors.New(fmt.Sprintf("AesKW.WrapNewKey(): expected key of size %v bits, but was given %v bits.",alg.keySizeBits, kekSizeBits))
		}	
		
		if cek,err = arrays.Random(cekSizeBits>>3);err==nil {			
			encryptedCek,err=aes.KeyWrap(cek,kek)
			return
		}
		
		return nil,nil,err

	}
		
	return nil,nil,errors.New("AesKW.WrapNewKey(): expected key to be '[]byte' array")	
}

func (alg *AesKW) Unwrap(encryptedCek []byte, key interface{}, cekSizeBits int, header map[string]interface{}) (cek []byte, err error) {
	
	if kek,ok:=key.([]byte); ok {
		
		kekSizeBits := len(kek) << 3
		
		if kekSizeBits != alg.keySizeBits {
			return nil,errors.New(fmt.Sprintf("AesKW.Unwrap(): expected key of size %v bits, but was given %v bits.", alg.keySizeBits, kekSizeBits))
		}	
		
		return aes.KeyUnwrap(encryptedCek, kek)
	}
		
	return nil,errors.New("AesKW.Unwrap(): expected key to be '[]byte' array")		
}
