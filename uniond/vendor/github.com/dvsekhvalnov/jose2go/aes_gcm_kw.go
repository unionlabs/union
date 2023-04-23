package jose

import (
	"errors"
	"fmt"
	"github.com/dvsekhvalnov/jose2go/base64url"
	"github.com/dvsekhvalnov/jose2go/arrays"
	"crypto/aes"
	"crypto/cipher"	
)

func init() {
	RegisterJwa(&AesGcmKW{ keySizeBits: 128})
	RegisterJwa(&AesGcmKW{ keySizeBits: 192})
	RegisterJwa(&AesGcmKW{ keySizeBits: 256})
}

// AES GCM Key Wrap key management algorithm implementation
type AesGcmKW struct {
	keySizeBits int
}

func (alg *AesGcmKW) Name() string {
	switch alg.keySizeBits {
		case 128: return A128GCMKW
		case 192: return A192GCMKW
		default: return  A256GCMKW
	}
}

func (alg *AesGcmKW) WrapNewKey(cekSizeBits int, key interface{}, header map[string]interface{}) (cek []byte, encryptedCek []byte, err error) {	
	if kek,ok:=key.([]byte); ok {
		
		kekSizeBits := len(kek) << 3
		
		if kekSizeBits != alg.keySizeBits {
			return nil,nil, errors.New(fmt.Sprintf("AesGcmKW.WrapNewKey(): expected key of size %v bits, but was given %v bits.",alg.keySizeBits, kekSizeBits))
		}	
		
		if cek,err = arrays.Random(cekSizeBits>>3);err!=nil {			
			return nil,nil,err
		}
		
		var iv []byte
		
		if iv,err = arrays.Random(12);err!=nil {
			return nil,nil,err
		}		
		
		var block cipher.Block

		if block, err = aes.NewCipher(kek);err!=nil {
			return nil,nil,err
		}
	
		var aesgcm cipher.AEAD
	
		if aesgcm,err = cipher.NewGCM(block);err!=nil {
			return nil,nil,err
		}

		cipherWithTag := aesgcm.Seal(nil, iv, cek, nil)
	
		cipherText := cipherWithTag[:len(cipherWithTag)-aesgcm.Overhead()]
		authTag := cipherWithTag[len(cipherWithTag)-aesgcm.Overhead():]
		
		header["iv"]=base64url.Encode(iv)
		header["tag"]=base64url.Encode(authTag)

		return cek,cipherText,nil
	}
		
	return nil,nil,errors.New("AesGcmKW.WrapNewKey(): expected key to be '[]byte' array")	
}

func (alg *AesGcmKW) Unwrap(encryptedCek []byte, key interface{}, cekSizeBits int, header map[string]interface{}) (cek []byte, err error) {
	if kek,ok:=key.([]byte); ok {
		
		kekSizeBits := len(kek) << 3
		
		if kekSizeBits != alg.keySizeBits {
			return nil,errors.New(fmt.Sprintf("AesGcmKW.Unwrap(): expected key of size %v bits, but was given %v bits.", alg.keySizeBits, kekSizeBits))
		}	
		
		var iv,tag string
		
		if iv,ok = header["iv"].(string);!ok {
			return nil,errors.New("AesGcmKW.Unwrap(): expected 'iv' param in JWT header, but was not found.")
		}
		
		if tag,ok = header["tag"].(string);!ok {
			return nil,errors.New("AesGcmKW.Unwrap(): expected 'tag' param in JWT header, but was not found.")
		}
		
		var ivBytes,tagBytes []byte
		
	    if ivBytes,err = base64url.Decode(iv);err!=nil {
	    	return nil,err
	    }
		
		if tagBytes,err = base64url.Decode(tag);err!=nil {
			return nil,err
		}
		
		var block cipher.Block

		if block, err = aes.NewCipher(kek);err!=nil {
			return nil,err
		}
	
		var aesgcm cipher.AEAD
	
		if aesgcm,err = cipher.NewGCM(block);err!=nil {
			return nil,err
		}

		cipherAndTag:=append(encryptedCek,tagBytes...)
		
		if cek,err = aesgcm.Open(nil, ivBytes,cipherAndTag , nil);err!=nil {
			fmt.Printf("err = %v\n",err)
			return nil,err
		}		
		
		return cek,nil
	}
		
	return nil,errors.New("AesGcmKW.Unwrap(): expected key to be '[]byte' array")	
}
