# blst

The `blst` package provides a Go interface to the blst BLS12-381 signature library.

## Build
The build process consists of two steps, code generation followed by compilation.

```
./generate.py # Optional - only required if making code changes
go build
go test
```

The generate.py script is used to generate both min-pk and min-sig variants of the binding from a common code base. It consumes the `*.tgo` files along with `blst_minpk_test.go` and produces `blst.go` and `blst_minsig_test.go`. The .tgo files can treated as if they were .go files, including the use of gofmt and goimports. The generate script will filter out extra imports while processing and automatically run goimports on the final blst.go file.

After running generate.py, <nobr>`go build`</nobr> and <nobr>`go test`</nobr> can be run as usual. Cgo will compile `server.c`, which includes the required C implementation files, and `assembly.S`, which includes approprate pre-generated assembly code for the platform. To compile on Windows one has to have MinGW gcc on the `%PATH%`.

If the test or target application crashes with an "illegal instruction" exception [after copying to an older system], rebuild with `CGO_CFLAGS` environment variable set to <nobr>`-O -D__BLST_PORTABLE__`</nobr>. Don't forget <nobr>`-O`</nobr>!

## Usage
There are two primary modes of operation that can be chosen based on type definitions in the application.

For minimal-pubkey-size operations:
```
type PublicKey = blst.P1Affine
type Signature = blst.P2Affine
type AggregateSignature = blst.P2Aggregate
type AggregatePublicKey = blst.P1Aggregate
```

For minimal-signature-size operations:
```
type PublicKey = blst.P2Affine
type Signature = blst.P1Affine
type AggregateSignature = blst.P1Aggregate
type AggregatePublicKey = blst.P2Aggregate
```

TODO - structures and possibly methods

A complete example for generating a key, signing a message, and verifying the message:
```
package main

import (
	"crypto/rand"
	"fmt"

	blst "github.com/supranational/blst/bindings/go"
)

type PublicKey = blst.P1Affine
type Signature = blst.P2Affine
type AggregateSignature = blst.P2Aggregate
type AggregatePublicKey = blst.P1Aggregate

func main() {
	var ikm [32]byte
	_, _ = rand.Read(ikm[:])
	sk := blst.KeyGen(ikm[:])
	pk := new(PublicKey).From(sk)

	var dst = []byte("BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_NUL_")
	msg := []byte("hello foo")
	sig := new(Signature).Sign(sk, msg, dst)

	if !sig.Verify(true, pk, true, msg, dst) {
		fmt.Println("ERROR: Invalid!")
	} else {
		fmt.Println("Valid!")
	}
}
```

See the tests for further examples of usage.

If you're cross-compiling, you have to set `CC` environment variable to the target C cross-compiler and `CGO_ENABLED` to 1. For example, to compile the test program for ARM:
```
env GOARCH=arm CC=arm-linux-gnueabi-gcc CGO_ENABLED=1 go test -c
```
