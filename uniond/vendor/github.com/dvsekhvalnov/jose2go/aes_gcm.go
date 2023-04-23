package jose

import (
	"fmt"
	"errors"
	"crypto/aes"
	"crypto/cipher"	
	"github.com/dvsekhvalnov/jose2go/arrays"
)

// AES GCM authenticated encryption algorithm implementation
type AesGcm struct{
	keySizeBits int
}

func init() {
	RegisterJwe(&AesGcm{keySizeBits:128})
	RegisterJwe(&AesGcm{keySizeBits:192})
	RegisterJwe(&AesGcm{keySizeBits:256})		
}

func (alg *AesGcm) Name() string {
	switch alg.keySizeBits {
		case 128: return A128GCM
		case 192: return A192GCM
		default: return  A256GCM
	}	
}

func (alg *AesGcm) KeySizeBits() int {
	return alg.keySizeBits
}

func (alg *AesGcm) Encrypt(aad, plainText, cek []byte) (iv, cipherText, authTag []byte, err error) {	
	
	cekSizeBits := len(cek)<<3
	
	if cekSizeBits != alg.keySizeBits {
		return nil,nil,nil, errors.New(fmt.Sprintf("AesGcm.Encrypt(): expected key of size %v bits, but was given %v bits.",alg.keySizeBits, cekSizeBits))
	}			
		
	if iv,err = arrays.Random(12);err!=nil {
		return nil,nil,nil,err
	}
	
	var block cipher.Block

	if block, err = aes.NewCipher(cek);err!=nil {
		return nil,nil,nil,err
	}
	
	var aesgcm cipher.AEAD
	
	if aesgcm,err = cipher.NewGCM(block);err!=nil {
		return nil,nil,nil,err
	}

	cipherWithTag := aesgcm.Seal(nil, iv, plainText, aad)
	
	cipherText=cipherWithTag[:len(cipherWithTag)-aesgcm.Overhead()]
	authTag=cipherWithTag[len(cipherWithTag)-aesgcm.Overhead():]
	
	return iv, cipherText, authTag, nil
}

func (alg *AesGcm) Decrypt(aad, cek, iv, cipherText, authTag []byte) (plainText []byte, err error) {
	
	cekSizeBits := len(cek)<<3
	
	if cekSizeBits != alg.keySizeBits {
		return nil, errors.New(fmt.Sprintf("AesGcm.Decrypt(): expected key of size %v bits, but was given %v bits.",alg.keySizeBits, cekSizeBits))
	}	
	
	var block cipher.Block

	if block, err = aes.NewCipher(cek);err!=nil {
		return nil,err
	}
	
	var aesgcm cipher.AEAD
	
	if aesgcm,err = cipher.NewGCM(block);err!=nil {
		return nil,err
	}

	cipherWithTag:=append(cipherText,authTag...)

	if nonceSize := len(iv); nonceSize != aesgcm.NonceSize() {
		return nil, errors.New(fmt.Sprintf("AesGcm.Decrypt(): expected nonce of size %v bits, but was given %v bits.", aesgcm.NonceSize()<<3, nonceSize<<3))
	}
	
	if plainText,err = aesgcm.Open(nil, iv, cipherWithTag, aad);err!=nil {
		return nil,err
	}
	
	return plainText,nil	
}

