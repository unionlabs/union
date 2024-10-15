### secp256k1-voi - Yet another secp256k1 implementation
#### Yawning Angel (yawning at schwanenlied dot me)

> Ponzi schemes exist in stable disequilibrium. This means that while
> they can’t ultimately succeed, they can persist indefinitely—until
> they don’t.
>
> --Harry Markopolos

This is a correctness/simplicity-first implementation of the secp256k1
elliptic curve used by shitcoins, written as an elaborate cry for help.

The following techniques and tools are used for "correctness/simplicity":
- The scalar/field arithmetic implementations are produced by [fiat-crypto][1].
- The scalar/field inversion, and the field square root implementations
are produced by [addchain][2].
- Exception free point addition and doubling formulas from
["Complete addition formulas for prime order elliptic curves"][3] by
Renes, Costello, and Batina are used.

#### Warning

This product can expose you to chemicals which are know to the State of
California to cause cancer.  For more information visit www.P65Warnings.ca.gov.

#### Features

- Formally verified field and scalar arithmetic.
- Constant time curve and scalar arithmetic operations unless explicitly
noted otherwise.
- Fast `s * G` routine using precomputed tables.
- Fast variable-time `u1 * G + u2 * P` routine for signature verification.
- Safe-by-default API, that makes it extremely hard to create invalid points
and scalars.
- Point s11n per SEC 1, Version 2.0, Section 2.3.3.
- ECDH per SEC 1, Version 2.0, Section 3.3.1.
- ECDSA per SEC 1, Version 2.0, Section 4.1.3/4.1.4 and BIP-0066.
- ECDSA with RFC 6979 + SHA256 for compatibility.
- ECDSA public key recovery per the various shitcoins.
- Schnorr signatures per BIP-0340.
- Hash to curve per RFC 9380.

#### Notes

- No, this has not been audited.  Unless you are willing to pay for it,
do not ask about it.  If you do not know how much that will cost, you
can not afford it.
- The API and some interals are ***heavily*** inspired by
Filippo's [edwards25519][4] and [nistec][5] packages.
- Only the 64-bit implementations of the underlying field arithmetic are
used, as 32-bit architectures are either increasingly irrelevant (x86, ARM)
or fucking garbage (WASM).  I may reconsider this when Golang gets build
tags that make this easy (and no, keeping track of all the architectures
is not "easy").
- No attempt is made to sanitize memory.  It is a lost cause in most
languages, and totally, utterly hopeless in Go.
- SIMD is used to accelerate the constant time table lookups.  Building
with `purego` disables the use of assembly.  It is almost, but not
quite, not even worth having variable-time variants of the multiplies
on amd64 due to the vectorized table lookup.
- The fiat-crypto ToBytes/FromBytes routines are not used due to our
need to handle non-canonical encodings, and the fact that fiat expects
and outputs little-endian, while big-endian is customary for this curve.
- Worms in my brain, get them out.

##### Performance

While this does try to be reasonably performant, the primary goal is to
be the most (obviously) correct Golang secp256k1, not the fastest Golang
secp256k1.

In short (only relevant figures listed):
```
cpu: AMD Ryzen 7 5700G with Radeon Graphics
BenchmarkPoint/GLV/ScalarMult-16          	   18753	     64955 ns/op     176 B/op	       3 allocs/op
BenchmarkPoint/ScalarBaseMult-16          	   47127	     24230 ns/op       0 B/op	       0 allocs/op
BenchmarkPoint/DoubleScalarMultBasepointVartime-16         	   15546	     78549 ns/op	     176 B/op	       3 allocs/op
BenchmarkPoint/s11n/UncompressedBytes-16                   	  192446	      5517 ns/op	       0 B/op	       0 allocs/op
BenchmarkPoint/s11n/CompressedBytes-16                     	  219115	      5520 ns/op	       0 B/op	       0 allocs/op
```

"It's alright".  Compared to `dcrd/dcrec/secp256k1` (aka `btcec`),
verification performance is basically the same, signing is slower, ECDH
ranges from slightly faster (on x86-64) to slower (purego).  On the other
hand, this library is timing side-channel safe on reasonable architectures.

Potential improvements:
- Sit and wait for Go 1.21 to come out, it seems to do better.
- wNAF based point multiplication is probably a gain.
- Go and add "multiply a field element by a small integer" to fiat.
- Pippenger's multi-scalar multiply would be better in certain cases.

[1]: https://github.com/mit-plv/fiat-crypto
[2]: https://github.com/mmcloughlin/addchain
[3]: https://eprint.iacr.org/2015/1060.pdf
[4]: https://pkg.go.dev/filippo.io/edwards25519
[5]: https://pkg.go.dev/filippo.io/nistec