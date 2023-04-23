// package padding provides various padding algorithms
package padding

import (
	"bytes"
)

// Align left pads given byte array with zeros till it have at least bitSize length. 
func Align(data []byte, bitSize int) []byte {
	
	actual:=len(data)	
	required:=bitSize >> 3
	
	if (bitSize % 8) > 0 {
		required++  //extra byte if needed
	}
	
	if (actual >= required) {
		return data
	} 
	
	return append(bytes.Repeat([]byte{0}, required-actual), data...)
}