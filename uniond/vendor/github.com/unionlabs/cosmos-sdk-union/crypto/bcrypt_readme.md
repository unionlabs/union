# Security parameter choice

The present Bcrypt security parameter used is 12, which should take about a quarter of a second on midrange consumer hardware (see [Benchmarking](#benchmarking) section below).

For some background into security parameter considerations, see [here](https://auth0.com/blog/hashing-in-action-understanding-bcrypt/) and [here](https://security.stackexchange.com/questions/3959/recommended-of-iterations-when-using-pkbdf2-sha256/3993#3993).

Given our security model, where an attacker would need to already have access to a victim's computer and copy the `~/.gaiacli` directory (as opposed to e.g. web authentication), this parameter choice seems sufficient for the time being. Bcrypt always generates a 448-bit key, so the security in practice is determined by the length & complexity of a user's password and the time taken to generate a Bcrypt key from their password (which we can choose with the security parameter). Users would be well-advised to use difficult-to-guess passwords.

## Benchmarking

To run Bcrypt benchmarks:

```bash
go test -v --bench github.com/cosmos/cosmos-sdk/crypto/keys/mintkey
```

On the test machine (midrange ThinkPad; i7 6600U), this results in:

```bash
goos: linux
goarch: amd64
pkg: github.com/cosmos/cosmos-sdk/crypto/keys/mintkey
BenchmarkBcryptGenerateFromPassword/benchmark-security-param-9-4         	      50	  34609268 ns/op
BenchmarkBcryptGenerateFromPassword/benchmark-security-param-10-4        	      20	  67874471 ns/op
BenchmarkBcryptGenerateFromPassword/benchmark-security-param-11-4        	      10	 135515404 ns/op
BenchmarkBcryptGenerateFromPassword/benchmark-security-param-12-4        	       5	 274824600 ns/op
BenchmarkBcryptGenerateFromPassword/benchmark-security-param-13-4        	       2	 547012903 ns/op
BenchmarkBcryptGenerateFromPassword/benchmark-security-param-14-4        	       1	1083685904 ns/op
BenchmarkBcryptGenerateFromPassword/benchmark-security-param-15-4        	       1	2183674041 ns/op
PASS
ok  	github.com/cosmos/cosmos-sdk/crypto/keys/mintkey	12.093s
```

Benchmark results are in nanoseconds, so security parameter 12 takes about a quarter of a second to generate the Bcrypt key, security param 13 takes half a second, and so on.
