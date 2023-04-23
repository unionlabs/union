[![Godoc Reference](https://godoc.org/github.com/minio/highwayhash?status.svg)](https://godoc.org/github.com/minio/highwayhash)
[![Build Status](https://travis-ci.org/minio/highwayhash.svg?branch=master)](https://travis-ci.org/minio/highwayhash)

## HighwayHash

[HighwayHash](https://github.com/google/highwayhash) is a pseudo-random-function (PRF) developed by Jyrki Alakuijala, Bill Cox and Jan Wassenberg (Google research). HighwayHash takes a 256 bit key and computes 64, 128 or 256 bit hash values of given messages.

It can be used to prevent hash-flooding attacks or authenticate short-lived messages. Additionally it can be used as a fingerprinting function. HighwayHash is not a general purpose cryptographic hash function (such as Blake2b, SHA-3 or SHA-2) and should not be used if strong collision resistance is required. 

This repository contains a native Go version and optimized assembly implementations for Intel, ARM and ppc64le architectures.

### High performance

HighwayHash is an approximately 5x faster SIMD hash function as compared to [SipHash](https://www.131002.net/siphash/siphash.pdf) which in itself is a fast and 'cryptographically strong' pseudo-random function designed by Aumasson and Bernstein.

HighwayHash uses a new way of mixing inputs with AVX2 multiply and permute instructions. The multiplications are 32x32 bit giving 64 bits-wide results and are therefore infeasible to reverse. Additionally permuting equalizes the distribution of the resulting bytes. The algorithm outputs digests ranging from 64 bits up to 256 bits at no extra cost.

### Stable

All three output sizes of HighwayHash have been declared [stable](https://github.com/google/highwayhash/#versioning-and-stability) as of January 2018. This means that the hash results for any given input message are guaranteed not to change.

### Installation

Install: `go get -u github.com/minio/highwayhash`

### Intel Performance

Below are the single core results on an Intel Core i7 (3.1 GHz) for 256 bit outputs:

```
BenchmarkSum256_16      		  204.17 MB/s
BenchmarkSum256_64      		 1040.63 MB/s
BenchmarkSum256_1K      		 8653.30 MB/s
BenchmarkSum256_8K      		13476.07 MB/s
BenchmarkSum256_1M      		14928.71 MB/s
BenchmarkSum256_5M      		14180.04 MB/s
BenchmarkSum256_10M     		12458.65 MB/s
BenchmarkSum256_25M     		11927.25 MB/s
```

So for moderately sized messages it tops out at about 15 GB/sec. Also for small messages (1K) the performance is already at approximately 60% of the maximum throughput. 

### ARM Performance

Below are the single core results on an EC2 m6g.4xlarge (Graviton2) instance for 256 bit outputs:

```
BenchmarkSum256_16                 96.82 MB/s
BenchmarkSum256_64                445.35 MB/s
BenchmarkSum256_1K               2782.46 MB/s
BenchmarkSum256_8K               4083.58 MB/s
BenchmarkSum256_1M               4986.41 MB/s
BenchmarkSum256_5M               4992.72 MB/s
BenchmarkSum256_10M              4993.32 MB/s
BenchmarkSum256_25M              4992.55 MB/s
```

### ppc64le Performance

The ppc64le accelerated version is roughly 10x faster compared to the non-optimized version:

```
benchmark              old MB/s     new MB/s     speedup
BenchmarkWrite_8K      531.19       5566.41      10.48x
BenchmarkSum64_8K      518.86       4971.88      9.58x
BenchmarkSum256_8K     502.45       4474.20      8.90x
```

### Performance compared to other hashing techniques

On a Skylake CPU (3.0 GHz Xeon Platinum 8124M) the table below shows how HighwayHash compares to other hashing techniques for 5 MB messages (single core performance, all Golang implementations, see [benchmark](https://github.com/fwessels/HashCompare/blob/master/benchmarks_test.go)).

```
BenchmarkHighwayHash      	    	11986.98 MB/s
BenchmarkSHA256_AVX512    	    	 3552.74 MB/s
BenchmarkBlake2b          	    	  972.38 MB/s
BenchmarkSHA1             	    	  950.64 MB/s
BenchmarkMD5              	    	  684.18 MB/s
BenchmarkSHA512           	    	  562.04 MB/s
BenchmarkSHA256           	    	  383.07 MB/s
```

*Note: the AVX512 version of SHA256 uses the [multi-buffer crypto library](https://github.com/intel/intel-ipsec-mb) technique as developed by Intel, more details can be found in [sha256-simd](https://github.com/minio/sha256-simd/).*

### Qualitative assessment

We have performed a 'qualitative' assessment of how HighwayHash compares to Blake2b in terms of the distribution of the checksums for varying numbers of messages. It shows that HighwayHash behaves similarly according to the following graph:

![Hash Comparison Overview](https://s3.amazonaws.com/s3git-assets/hash-comparison-final.png)

More information can be found in [HashCompare](https://github.com/fwessels/HashCompare).

### Requirements

All Go versions >= 1.11 are supported (needed for required assembly support for the different platforms).

### Contributing

Contributions are welcome, please send PRs for any enhancements.