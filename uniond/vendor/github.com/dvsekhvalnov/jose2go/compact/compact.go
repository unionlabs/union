// package compact provides function to work with json compact serialization format
package compact

import (
	"github.com/dvsekhvalnov/jose2go/base64url"
	"strings"
)

// Parse splitting & decoding compact serialized json web token, returns slice of byte arrays, each representing part of token
func Parse(token string) (result [][]byte, e error) {
	parts := strings.Split(token, ".")

	result = make([][]byte, len(parts))

	for i, part := range parts {
		if result[i], e = base64url.Decode(part); e != nil {
			return nil, e
		}
	}

	return result, nil
}

// Serialize converts given parts into compact serialization format
func Serialize(parts ...[]byte) string {
	result := make([]string, len(parts))

	for i, part := range parts {
		result[i] = base64url.Encode(part)
	}

	return strings.Join(result, ".")
}
