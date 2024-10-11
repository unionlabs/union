package aes

import (	
	// "fmt"
	"crypto/aes"
	// "crypto/cipher"
	. "gopkg.in/check.v1"
)

func (s *TestSuite) TestNewECBEncryptor(c *C) {
	//given
	plaintext := []byte{0, 17, 34, 51, 68, 85, 102, 119, 136, 153, 170, 187, 204, 221, 238, 255}
	kek := []byte{0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15}
	block, _ := aes.NewCipher(kek)
	
	test := make([]byte,len(plaintext))

	//when	
	NewECBEncrypter(block).CryptBlocks(test,plaintext)	
	
	//then
	c.Assert(test, DeepEquals, []byte{105,196,224,216,106,123,4,48,216,205,183,128,112,180,197,90})
}

func (s *TestSuite) TestNewECBDecryptor(c *C) {
	//given
	ciphertext := []byte{105,196,224,216,106,123,4,48,216,205,183,128,112,180,197,90}
	kek := []byte{0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15}
	block, _ := aes.NewCipher(kek)
	
	test := make([]byte,len(ciphertext))

	//when	
	NewECBDecrypter(block).CryptBlocks(test,ciphertext)	
	
	//then
	c.Assert(test, DeepEquals, []byte{0, 17, 34, 51, 68, 85, 102, 119, 136, 153, 170, 187, 204, 221, 238, 255})
}