# ZkTrie

[![License](https://img.shields.io/badge/license-Apache2-green.svg)](LICENSE)

The Rust implementation for https://github.com/scroll-tech/zktrie, using [sgxlib](https://github.com/automata-network/sgxlib) to support both Intel SGX and std environments.

## Benchmark

check the source code: src/zktrie_bench.rs

Go Version

```
zktrie> go test -bench=. ./trie
goos: linux
goarch: amd64
pkg: github.com/scroll-tech/zktrie/trie
cpu: 13th Gen Intel(R) Core(TM) i9-13900K
BenchmarkTrieAdd-32         	   40834	     33510 ns/op
BenchmarkTrieGet-32         	  294921	      3942 ns/op
BenchmarkTrieDeletion-32    	 1004121	      1048 ns/op
PASS
ok  	github.com/scroll-tech/zktrie/trie	9.178s
```

Rust Version

```
zktrie-rs> cargo bench -- bench
   Compiling zktrie v0.1.0
    Finished bench [optimized] target(s) in 0.91s
     Running unittests (target/release/deps/zktrie-932f0a0fed6427b0)

running 3 tests
test zktrie_bench::bench_trie_add      ... bench:      11,075 ns/iter (+/- 429)
test zktrie_bench::bench_trie_get      ... bench:       2,147 ns/iter (+/- 187)
test zktrie_bench::bench_trie_deletion ... bench:         397 ns/iter (+/- 17)

test result: ok. 0 passed; 0 failed; 0 ignored; 3 measured; 13 filtered out; finished in 9.60s
```
