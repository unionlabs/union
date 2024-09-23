# Integer Compression

[![Go Reference](https://pkg.go.dev/badge/github.com/ronanh/intcomp.svg)](https://pkg.go.dev/github.com/ronanh/intcomp)

This library provides high performance (GB/s) compression and decompression of integers (int32/uint32/int64/uint64).

Good compression factor can be achieved when, on average, the difference between 2 consecutive
values of the input remains small and thus can be encoded with fewer bits.

Common use cases:

- Timestamps
- Offsets
- Counter based identifiers

The encoding schemes used here are based on [Dr. Daniel Lemire research](https://lemire.me/blog/2012/09/12/fast-integer-compression-decoding-billions-of-integers-per-second/).

## Encoding Logic

Data is encoded in blocks of multiple of 128x32bit or 256x64bits inputs in the
following manner:

1. Difference of consecutive inputs is computed (differential coding)
2. ZigZag encoding is applied if a block contains at least one negative delta value
3. The result is bit packed into the optimal number of bits for the block

The remaining input that won't fit within a 128x32bits or 256x64bits block will be encoded
in an additional block using Variable Byte encoding (with delta)

## Append to compressed arrays

In stream processing systems data is usually received by chunks.
Compressing and aggregating small chunks can be inneficient and impractical.

This API provides a convenient way to handle such inputs:
When adding data to a compressed buffer, if the last block is a small block, encoded with Variable Byte,
it will be rewritten in order to provide better compression using bit packing.

## Encoding of timestamps with nanosecond resolution

Timestamps with nanosecond resolution sometimes have an actual lower internal resolution (eg. microsecond).
To provide better compression for that type of data, the encoding algorithm for int64 has a specific
optimization that will provide better compression factor in such case.

## Usage

```go
input := []int32{1, 2, 3}

// compress
compressed := intcomp.CompressInt32(input, nil)
// compress more data (append)
compressed = intcomp.CompressInt32([]int32{4, 5, 6}, compressed)

// uncompress
data := intcomp.UncompressInt32(compressed, nil)
// data: [1, 2, 3, 4, 5, 6]
```

## Performance

Benchmarks for the bitpacking compression/decompression (MacBook pro M1).
The result vary depending on the number of bits used to encode integers.

### Compression

- 32bits: between 2.6 and 3.5 GB/s
- 64bits: between 5.1 and 8.1 GB/s

### Decompression

- 32bits: between 3.5 and 9.3 GB/s
- 64bits: between 12.3 and 23.8 GB/s

## TODO

- [ ] Support float32/64 (using something similar to Gorilla compression)
- [ ] Force creation of blocks at fixed boundaries to enable arbitrary position decoding and reversed iteration
- [ ] Implement Block iterators with low memory usage
- [ ] Add Binary search for sorted arrays
