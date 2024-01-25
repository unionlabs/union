# go-bitarray

[![Go Reference](https://pkg.go.dev/badge/github.com/tunabay/go-bitarray.svg)](https://pkg.go.dev/github.com/tunabay/go-bitarray)
[![MIT License](http://img.shields.io/badge/license-MIT-blue.svg?style=flat)](LICENSE)

## Overview

Package bitarray provides data types and functions for manipulating bit arrays,
aka bit strings, of arbitrary length.

This is designed to handle bit arrays across byte boundaries naturally, without
error-prone bitwise operation code such as shifting, masking, and ORing. It may
be useful when dealing with Huffman coding, raw packet of various protocols, and
binary file formats, etc.

## Usage

### Manipulate bitarrays using the `BitArray` type.
```go
import (
	"fmt"
	"github.com/tunabay/go-bitarray"
)

func main() {
	// Parse string representation
	ba1, err := bitarray.Parse("111000")
	if err != nil {
		panic(err)
	}
	fmt.Println(ba1) // 111000

	// Slice and Repeat
	ba2 := ba1.Slice(2, 5).Repeat(2)
	fmt.Println(ba2) // 100100

	// Append
	ba3 := ba2.Append(bitarray.MustParse("101011"))
	// alternative formatting
	fmt.Printf("% b\n", ba3) // 10010010 1011

	// Extract bits from []byte across byte boundary
	buf := []byte{0xff, 0x00}
	ba4 := bitarray.NewFromBytes(buf, 4, 7)
	fmt.Println(ba4) // 1111000
}
```
[Run in Go Playground](https://play.golang.org/p/qm4fpMCPdWa)

### Use the `Buffer` type for bitwise access to byte slices.
```go
import (
	"fmt"
	"github.com/tunabay/go-bitarray"
)

// This example assumes 8-byte data with the following bit layout, and
// accesses the 5-bit integer X and the 50-bit integer Y in it.
//
// |0              |1              |2              |3              |
// |0 1 2 3 4 5 6 7|0 1 2 3 4 5 6 7|0 1 2 3 4 5 6 7|0 1 2 3 4 5 6 7|
// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
// | 9-bit flag area | 5-bit X | Upper 18 bits of the 50-bit int Y |
// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
// |               Lower 32 bits of the 50-bit int Y               |
// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
func main() {
	data := make([]byte, 8)
	buf := bitarray.NewBufferFromByteSlice(data)

	// set 9-bit flag area to 110000011
	buf.PutBitAt(0, 1)
	buf.PutBitAt(1, 1)
	buf.PutBitAt(7, 1)
	buf.PutBitAt(8, 1)

	// set 5-bit integer X
	buf.Slice(9, 14).PutUint8(25) // = 0b_11001

	// set 50-bit integer Y
	buf.Slice(14, 64).PutUint64(0x_3_f0ff_f0f0_ff0f)

	// raw bytes updated
	fmt.Printf("%08b\n%08b\n", data[:4], data[4:])

	// read fields
	fmt.Printf("F = %b\n", buf.Slice(0, 9))
	fmt.Printf("X = %d\n", buf.Slice(9, 14).Uint8())
	fmt.Printf("Y = %x\n", buf.SliceToEnd(14).Uint64())
}
```
[Run in Go Playground](https://play.golang.org/p/INosZRfZsuR)

## Documentation and more examples

- Read the [documentation](https://pkg.go.dev/github.com/tunabay/go-bitarray).

## License

go-bitarray is available under the MIT license. See the [LICENSE](LICENSE) file for more information.
