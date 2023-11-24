# Zstd Go Wrapper

[![CircleCI](https://circleci.com/gh/DataDog/zstd/tree/1.x.svg?style=svg)](https://circleci.com/gh/DataDog/zstd/tree/1.x)
[![GoDoc](https://godoc.org/github.com/DataDog/zstd?status.svg)](https://godoc.org/github.com/DataDog/zstd)


[C Zstd Homepage](https://github.com/facebook/zstd)

The current headers and C files are from *v1.5.0* (Commit
[10f0e699](https://github.com/facebook/zstd/releases/tag/v1.5.0)).

## Usage

There are two main APIs:

* simple Compress/Decompress
* streaming API (io.Reader/io.Writer)

The compress/decompress APIs mirror that of lz4, while the streaming API was
designed to be a drop-in replacement for zlib.

### Building against an external libzstd

By default, zstd source code is vendored in this repository and the binding will be built with
the vendored source code bundled.

If you want to build this binding against an external static or shared libzstd library, you can
use the `external_libzstd` build tag. This will look for the libzstd pkg-config file and extract
build and linking parameters from that pkg-config file.

Note that it requires at least libzstd 1.4.0.

```bash
go build -tags external_libzstd
```

### Simple `Compress/Decompress`


```go
// Compress compresses the byte array given in src and writes it to dst.
// If you already have a buffer allocated, you can pass it to prevent allocation
// If not, you can pass nil as dst.
// If the buffer is too small, it will be reallocated, resized, and returned bu the function
// If dst is nil, this will allocate the worst case size (CompressBound(src))
Compress(dst, src []byte) ([]byte, error)
```

```go
// CompressLevel is the same as Compress but you can pass another compression level
CompressLevel(dst, src []byte, level int) ([]byte, error)
```

```go
// Decompress will decompress your payload into dst.
// If you already have a buffer allocated, you can pass it to prevent allocation
// If not, you can pass nil as dst (allocates a 4*src size as default).
// If the buffer is too small, it will retry 3 times by doubling the dst size
// After max retries, it will switch to the slower stream API to be sure to be able
// to decompress. Currently switches if compression ratio > 4*2**3=32.
Decompress(dst, src []byte) ([]byte, error)
```

### Stream API

```go
// NewWriter creates a new object that can optionally be initialized with
// a precomputed dictionary. If dict is nil, compress without a dictionary.
// The dictionary array should not be changed during the use of this object.
// You MUST CALL Close() to write the last bytes of a zstd stream and free C objects.
NewWriter(w io.Writer) *Writer
NewWriterLevel(w io.Writer, level int) *Writer
NewWriterLevelDict(w io.Writer, level int, dict []byte) *Writer

// Write compresses the input data and write it to the underlying writer
(w *Writer) Write(p []byte) (int, error)

// Flush writes any unwritten data to the underlying writer
(w *Writer) Flush() error

// Close flushes the buffer and frees C zstd objects
(w *Writer) Close() error
```

```go
// NewReader returns a new io.ReadCloser that will decompress data from the
// underlying reader.  If a dictionary is provided to NewReaderDict, it must
// not be modified until Close is called.  It is the caller's responsibility
// to call Close, which frees up C objects.
NewReader(r io.Reader) io.ReadCloser
NewReaderDict(r io.Reader, dict []byte) io.ReadCloser
```

### Benchmarks (benchmarked with v0.5.0)

The author of Zstd also wrote lz4. Zstd is intended to occupy a speed/ratio
level similar to what zlib currently provides.  In our tests, the can always
be made to be better than zlib by chosing an appropriate level while still
keeping compression and decompression time faster than zlib.

You can run the benchmarks against your own payloads by using the Go benchmarks tool.
Just export your payload filepath as the `PAYLOAD` environment variable and run the benchmarks:

```go
go test -bench .
```

Compression of a 7Mb pdf zstd (this wrapper) vs [czlib](https://github.com/DataDog/czlib):
```
BenchmarkCompression               5     221056624 ns/op      67.34 MB/s
BenchmarkDecompression           100      18370416 ns/op     810.32 MB/s

BenchmarkFzlibCompress             2     610156603 ns/op      24.40 MB/s
BenchmarkFzlibDecompress          20      81195246 ns/op     183.33 MB/s
```

Ratio is also better by a margin of ~20%.
Compression speed is always better than zlib on all the payloads we tested;
However, [czlib](https://github.com/DataDog/czlib) has optimisations that make it
faster at decompressiong small payloads:

```
Testing with size: 11... czlib: 8.97 MB/s, zstd: 3.26 MB/s
Testing with size: 27... czlib: 23.3 MB/s, zstd: 8.22 MB/s
Testing with size: 62... czlib: 31.6 MB/s, zstd: 19.49 MB/s
Testing with size: 141... czlib: 74.54 MB/s, zstd: 42.55 MB/s
Testing with size: 323... czlib: 155.14 MB/s, zstd: 99.39 MB/s
Testing with size: 739... czlib: 235.9 MB/s, zstd: 216.45 MB/s
Testing with size: 1689... czlib: 116.45 MB/s, zstd: 345.64 MB/s
Testing with size: 3858... czlib: 176.39 MB/s, zstd: 617.56 MB/s
Testing with size: 8811... czlib: 254.11 MB/s, zstd: 824.34 MB/s
Testing with size: 20121... czlib: 197.43 MB/s, zstd: 1339.11 MB/s
Testing with size: 45951... czlib: 201.62 MB/s, zstd: 1951.57 MB/s
```

zstd starts to shine with payloads > 1KB

### Stability - Current state: STABLE

The C library seems to be pretty stable and according to the author has been tested and fuzzed.

For the Go wrapper, the test cover most usual cases and we have succesfully tested it on all staging and prod data.
