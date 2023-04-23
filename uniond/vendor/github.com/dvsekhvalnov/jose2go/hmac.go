package jose

import (
	"crypto/hmac"
	"hash"
)

func calculateHmac(keySizeBits int, securedInput []byte, key []byte) []byte {
	hasher := hmac.New(func() hash.Hash { return hashAlg(keySizeBits)}, key)	
	hasher.Write(securedInput)	
	
	return hasher.Sum(nil)	
}