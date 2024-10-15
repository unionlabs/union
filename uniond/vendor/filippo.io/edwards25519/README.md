# filippo.io/edwards25519

```
import "filippo.io/edwards25519"
```

This library implements the edwards25519 elliptic curve, exposing the necessary APIs to build a wide array of higher-level primitives.
Read the docs at [pkg.go.dev/filippo.io/edwards25519](https://pkg.go.dev/filippo.io/edwards25519).

The code is originally derived from Adam Langley's internal implementation in the Go standard library, and includes George Tankersley's [performance improvements](https://golang.org/cl/71950). It was then further developed by Henry de Valence for use in ristretto255, and was finally [merged back into the Go standard library](https://golang.org/cl/276272) as of Go 1.17. It now tracks the upstream codebase and extends it with additional functionality.

Most users don't need this package, and should instead use `crypto/ed25519` for signatures, `golang.org/x/crypto/curve25519` for Diffie-Hellman, or `github.com/gtank/ristretto255` for prime order group logic. However, for anyone currently using a fork of `crypto/internal/edwards25519`/`crypto/ed25519/internal/edwards25519` or `github.com/agl/edwards25519`, this package should be a safer, faster, and more powerful alternative.

Since this package is meant to curb proliferation of edwards25519 implementations in the Go ecosystem, it welcomes requests for new APIs or reviewable performance improvements.
