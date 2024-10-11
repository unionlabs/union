### curve25519-voi

> It was only machinery.  I’m surprised it’s lasted as long as it has,
> frankly.  There must still be some residual damage-repair capability.
> We Demarchists build for posterity, you know.

This package aims to provide a modern X25519/Ed25519/sr25519
implementation for Go, mostly derived from curve25519-dalek.  The
primary motivation is to hopefully provide a worthy alternative to
the current state of available Go implementations, which is best
described as "a gigantic mess of ref10 and donna ports".  The irony
of the previous statement in the light of curve25519-dalek's lineage
does not escape this author.

#### WARNING

***DO NOT BOTHER THE curve25519-dalek DEVELOPERS ABOUT THIS PACKAGE***

#### Package structure

 * curve: A mid-level API in the spirit of curve25519-dalek.
 * primitives/x25519: A X25519 implementation like `x/crypto/curve25519`.
 * primitives/ed25519: A Ed25519 implementation like `crypto/ed25519`.
 * primitives/ed25519/extra/ecvrf: A implementation of the "Verifiable Random Functions" draft (v10, v13).
 * primitives/sr25519: A sr25519 implementation like `https://github.com/w3f/schnorrkel`.
 * primitives/merlin: A Merlin transcript implementation.
 * primitives/h2c: A implementation of the "Hashing to Elliptic Curves" draft (v16).

#### Ed25519 verification semantics

At the time of this writing, Ed25519 signature verification behavior
varies based on the implementation.  The implementation provided by
this package aims to provide a sensible default, and to support
compatibility with other implementations if required.

The default verification semantics are as follows, using the terminology
from [ed25519-speccheck][1]:

 * Both iterative and batch verification are cofactored.
 * Small order A is rejected.
 * Small order R is accepted.
 * Non-canonical A is rejected.
 * Non-canonical R is rejected.
 * A signature's scalar component must be in canonical form (S < L).

Pre-defined configuration presets for compatibility with the Go standard
library, FIPS 186-5/RFC 8032, and ZIP-215 are provided for convenience.

For more details on this general problem, see [Taming the many EdDSAs][2].

#### Notes

The curve25519-dalek crate makes use of "modern" programing language
features not available in Go.  This package's mid-level API attempts
to provide something usable by developers familiar with idiomatic Go,
and thus has more sharp edges, but in all honestly, developers that
opt to use the mid-level API in theory already know what they are
getting into.  Stability of the mid-level API is currently NOT
guaranteed.

The curve25519-dalek crate has a series of nice vectorized backends
written using SIMD intrinsics.  While Go has no SIMD intrinsics, and
the assembly dialect is anything but nice, the AVX2 backend is also
present in this implementation.

Memory sanitization while maintaining reasonable performance in Go is
a hard/unsolved problem, and this package makes no attempts to do so.
Anyone that mentions memguard will be asked to re-read the previous
sentence again, and then be mercilessly mocked.  It is worth noting
that the standard library does not do this appropriately either.

The minimum required Go version for this package follows the Go
support policy, of latest version and the previous one.  Attempting
to subvert the toolchain checks will result in reduced performance
or insecurity on certain platforms.

This package uses build tags to enable the 32-bit or 64-bit backend
respectively.  Note that for 64-bit targets, this primarily depends
on if the SSA code (`src/cmd/compile/internal/ssagen/ssa.go`) has
the appropriate special cases to make `math/bits.Mul64`/`math/bits.Add64`
perform well.

 * 64-bit: `amd64`, `arm64`, `ppc64le`, `ppc64`, `s390x`
 * 32-bit: `386`, `arm`, `mips`, `mipsle`, `wasm`, `mips64`, `mips64le`, `riscv64`, `loong64`
 * Unsupported: Everything else.

**WARNING**: As a concession to the target's growing popularity, the
`wasm` target is supported using the 32-bit backend, however the
WebAssembly specification does not mandate that any opcodes are
constant time, making it difficult to provide assurances related to
timing side-channels.

The lack of a generic "just use 32-bit" fallback can be blamed on
the Go developers rejecting [adding build tags for bit-width][3].

The lattice reduction implementation currently only has a 64-bit
version, and thus it will be used on all platforms.  Note that while
Go 1.12 had a vartime implementation of `math/bits` routines, that
version of the compiler is long unsupported, and the lattice reduction
is verification only so the lack of a timing guarantee has no security
impact.

#### Special credits

curve25519-voi would not exist if it were not for the amazing work
done by various other projects.  Any bugs in curve25519-voi are the
fault of the curve25519-voi developers alone.

 * The majority of curve25519-voi is derived from curve25519-dalek.

 * The Ed25519 batch verification started off as a port of the
   implementation present in ed25519-dalek, but was later switched
   to be based off ed25519consensus.

 * The ABGLSV-Pornin multiplication implementation is derived from
   a curve25519-dalek pull request by Jack Grigg (@str4d), with
   additional inspiration taken from Thomas Pornin's paper and
   curve9767 implementation.

 * The assembly optimized field element multiplications were taken
   (with minor modifications) from George Tankersley's ristretto255
   package.

 * The Elligator 2 mapping was taken from Loup Vaillant's Monocypher
   package.

[1]: https://github.com/novifinancial/ed25519-speccheck
[2]: https://eprint.iacr.org/2020/1244.pdf
[3]: https://github.com/golang/go/issues/33388
