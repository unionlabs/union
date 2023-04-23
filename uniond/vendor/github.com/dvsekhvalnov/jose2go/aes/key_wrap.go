package aes

import (
	"github.com/dvsekhvalnov/jose2go/arrays"
	"crypto/cipher"
	"crypto/aes"
	"crypto/hmac"
	"errors"
)

var	defaultIV=[]byte { 0xA6, 0xA6, 0xA6, 0xA6, 0xA6, 0xA6, 0xA6, 0xA6 }

// KeyWrap encrypts provided key (CEK) with KEK key using AES Key Wrap (rfc 3394) algorithm
func KeyWrap(cek,kek []byte) ([]byte,error) {
	// 1) Initialize variables
    a := defaultIV              // Set A = IV, an initial value
    r := arrays.Slice(cek, 8)   // For i = 1 to n
								//     R[0][i] = P[i]
	n := uint64(len(r))
	
    // 2) Calculate intermediate values.
	var j,i,t uint64
	
	for j = 0; j < 6; j++ {                              // For j = 0 to 5
		for i = 0; i < n; i++ {                          //    For i=1 to n
			t = n * j + i + 1;
			b,e := aesEnc(kek, arrays.Concat(a, r[i]))   //       B=AES(K, A | R[i])
			
			if e!=nil { return nil, e }
			
			a = b[:len(b)/2]   							 //       A=MSB(64,B) ^ t where t = (n*j)+i
			r[i] = b[len(b)/2:]             			 //       R[i] = LSB(64, B)
			a = arrays.Xor(a, arrays.UInt64ToBytes(t))
		}
	}
	
	// 3) Output the results
	c := make([][]byte, n+1, n+1)
	c[0] = a;                              //  Set C[0] = A
	for i = 1; i <= n; i++ {               //  For i = 1 to n
		c[i] = r[i - 1]                    //     C[i] = R[i]
	}                      

	return arrays.Unwrap(c),nil
}

// KeyUnwrap decrypts previously encrypted key (CEK) with KEK key using AES Key Wrap (rfc 3394) algorithm
func KeyUnwrap(encryptedCek, kek []byte) ([]byte,error) {
    // 1) Initialize variables
	c := arrays.Slice(encryptedCek, 8);
	a := c[0];                           //   Set A = C[0]
	r := make([][]byte,len(c) - 1);

	for i := 1; i < len(c); i++ {         //   For i = 1 to n
		r[i - 1] = c[i];                  //       R[i] = C[i]
	}              

    n := uint64(len(r))
	
    // 2) Calculate intermediate values
	var t,j uint64

    for j = 6; j > 0; j-- {      						  // For j = 5 to 0    
        for i := n; i > 0; i-- {                          //   For i = n to 1        
			t = n * (j-1) + i;
			a = arrays.Xor(a, arrays.UInt64ToBytes(t))
			b,e := aesDec(kek, arrays.Concat(a, r[i-1]))  //     B = AES-1(K, (A ^ t) | R[i]) where t = n*j+i			
			
			if e!=nil { return nil,e }			
			
			a = b[:len(b)/2]                              //     A = MSB(64, B)
			r[i-1] = b[len(b)/2:]                         //     R[i] = LSB(64, B)
        }
    }

    // 3) Output the results
    if (!hmac.Equal(defaultIV, a)) {  // If A is an appropriate initial value 
        return nil, errors.New("aes.KeyUnwrap(): integrity check failed.")
	}

								   // For i = 1 to n
    return arrays.Unwrap(r),nil    //    P[i] = R[i]

}

func aesEnc(kek, plainText []byte) (cipherText []byte, err error) {
	var block cipher.Block
	
	if block, err = aes.NewCipher(kek);err!=nil {
		return nil,err
	}
			
	cipherText = make([]byte, len(plainText))
	
	NewECBEncrypter(block).CryptBlocks(cipherText,plainText)
	
	return cipherText,nil
}

func aesDec(kek, cipherText []byte) (plainText []byte,err error) {
		
	var block cipher.Block
	
	if block, err = aes.NewCipher(kek);err!=nil {
		return nil,err
	}
	
	plainText = make([]byte, len(cipherText))
	
	NewECBDecrypter(block).CryptBlocks(plainText,cipherText)	
	
	return plainText,nil
}