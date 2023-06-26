# Go Hashtree

GoHashtree is a SHA256 library highly optimized for Merkle tree computation. It is based on [Intel's implementation](https://github.com/intel/intel-ipsec-mb) with a few modifications like hardcoding the scheduled words of the padding block. It is written in Go Assembly instead of its native assembly counterpart  [hashtree](https://github.com/prysmaticlabs/hashtree). 

# Using the library

The library exposes a single function 
```
func Hash(digests [][32]byte, chunks [][32]byte) error
```
This function hashes each consecutive pair of 32 byte blocks from `chunks` and writes the corresponding digest to `digests`. It performs runtime detection of CPU features supported. The function returns an error if `digests` is not allocated to hold at least `len(chunks)/2` digests or if an odd number of chunks is given. 

Most vectorized implementations exploit the fact that independent branches in the Merkle tree can be hashed in "parallel" within one CPU, to take advantage of this,
Merkleization algorithms that loop over consecutive tree layers hashing two blocks at a time need to be updated to pass the entire layer, or all consecutive blocks. A naive example on how to accomplish this can be found in [this document](https://hackmd.io/80mJ75A5QeeRcrNmqcuU-g?view)

# Running tests and benchmarks
- Run the tests
```shell
$ cd gohashstree
$ go test .
ok  	github.com/prysmaticlabs/gohashtree	0.002s
```

- Some benchmarks in ARM+crypto
```
$ cd gohashtree
$ go test . -bench=.
goos: darwin
goarch: arm64
pkg: github.com/prysmaticlabs/gohashtree
BenchmarkHash_1_minio-10               8472337          122.9 ns/op
BenchmarkHash_1-10                     27011082           42.99 ns/op
BenchmarkHash_4_minio-10               2419328          500.1 ns/op
BenchmarkHash_4-10                     6900236          172.1 ns/op
BenchmarkHash_8_minio-10               1217845          985.6 ns/op
BenchmarkHash_8-10                     3471864          344.0 ns/op
BenchmarkHash_16_minio-10               597896         1974 ns/op
BenchmarkHash_16-10                    1721486          689.2 ns/op
BenchmarkHashLargeList_minio-10             38     28401697 ns/op
BenchmarkHashList-10                       138      8619502 ns/op
PASS
ok      github.com/prysmaticlabs/gohashtree    16.854s
```
- Some benchmarks on a Raspberry-Pi without crypto extensions
```
$ cd gohashtree
$ go test . -bench=.
goos: linux
goarch: arm64
pkg: github.com/prysmaticlabs/gohashtree
BenchmarkHash_1_minio-4                   338904              3668 ns/op
BenchmarkHash_1-4                        1000000              1087 ns/op
BenchmarkHash_4_minio-4                    82258             15537 ns/op
BenchmarkHash_4-4                         380631              3216 ns/op
BenchmarkHash_8_minio-4                    41265             34344 ns/op
BenchmarkHash_8-4                         181153              6569 ns/op
BenchmarkHash_16_minio-4                   16635             67142 ns/op
BenchmarkHash_16-4                         75922             13351 ns/op
BenchmarkHashLargeList_minio-4                 2         826262074 ns/op
BenchmarkHashList-4                            7         176396035 ns/op
PASS
```
- Some benchmarks on a Xeon with AVX-512
```
$ cd gohashtree
$ go test . -bench=.
goos: linux
goarch: amd64
pkg: github.com/prysmaticlabs/gohashtree
cpu: Intel(R) Xeon(R) CPU @ 2.80GHz
BenchmarkHash_1_minio-2                  2462506               473.1 ns/op
BenchmarkHash_1-2                        3040208               391.3 ns/op
BenchmarkHash_4_minio-2                   577078              1959 ns/op
BenchmarkHash_4-2                        1954473               604.9 ns/op
BenchmarkHash_8_minio-2                   298208              3896 ns/op
BenchmarkHash_8-2                        1882191               624.8 ns/op
BenchmarkHash_16_minio-2                  147230              7933 ns/op
BenchmarkHash_16-2                        557485              1988 ns/op
BenchmarkHashLargeList_minio-2                10         105404666 ns/op
BenchmarkHashList-2                           45          25368532 ns/op
PASS
ok      github.com/prysmaticlabs/gohashtree     13.969s
```

