package db

import "math/rand"

const (
	strChars = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz" // 62 characters
)

// For testing convenience.
func bz(s string) []byte {
	return []byte(s)
}

// Str constructs a random alphanumeric string of given length.
func randStr(length int) string {
	chars := []byte{}
MAIN_LOOP:
	for {
		val := rand.Int63() //nolint:gosec
		for i := 0; i < 10; i++ {
			v := int(val & 0x3f) // rightmost 6 bits
			if v >= 62 {         // only 62 characters in strChars
				val >>= 6
				continue
			}

			chars = append(chars, strChars[v])
			if len(chars) == length {
				break MAIN_LOOP
			}
			val >>= 6
		}
	}

	return string(chars)
}
