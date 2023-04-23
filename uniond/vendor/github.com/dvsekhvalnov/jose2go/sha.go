package jose

import (
	"hash"
	"crypto"
	"crypto/sha256"
	"crypto/sha512"
)

func hashFunc(keySizeBits int) crypto.Hash {
	switch keySizeBits {
		case 256: return crypto.SHA256
		case 384: return crypto.SHA384
		 default: return crypto.SHA512
	}	
}

func hashAlg(keySizeBits int) hash.Hash {
	switch keySizeBits {
		case 256: return sha256.New()
		case 384: return sha512.New384()
		 default: return sha512.New()
	}
}