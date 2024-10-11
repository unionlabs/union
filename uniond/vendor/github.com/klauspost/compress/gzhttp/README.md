Gzip Middleware
===============

This Go package which wraps HTTP *server* handlers to transparently gzip the
response body, for clients which support it. 

For HTTP *clients* we provide a transport wrapper that will do gzip decompression 
faster than what the standard library offers.

Both the client and server wrappers are fully compatible with other servers and clients.

This package is forked from the dead [nytimes/gziphandler](https://github.com/nytimes/gziphandler)
and extends functionality for it.

## Install
```bash
go get -u github.com/klauspost/compress
```

## Documentation

[![Go Reference](https://pkg.go.dev/badge/github.com/klauspost/compress/gzhttp.svg)](https://pkg.go.dev/github.com/klauspost/compress/gzhttp)


## Usage

There are 2 main parts, one for http servers and one for http clients.

### Client

The standard library automatically adds gzip compression to most requests 
and handles decompression of the responses.

However, by wrapping the transport we are able to override this and provide 
our own (faster) decompressor.

Wrapping is done on the Transport of the http client:

```Go
func ExampleTransport() {
	// Get an HTTP client.
	client := http.Client{
		// Wrap the transport:
		Transport: gzhttp.Transport(http.DefaultTransport),
	}

	resp, err := client.Get("https://google.com")
	if err != nil {
		return
	}
	defer resp.Body.Close()
	
	body, _ := ioutil.ReadAll(resp.Body)
	fmt.Println("body:", string(body))
}
```

Speed compared to standard library `DefaultTransport` for an approximate 127KB JSON payload:

```
BenchmarkTransport

Single core:
BenchmarkTransport/gzhttp-32                1995        609791 ns/op     214.14 MB/s       10129 B/op         73 allocs/op
BenchmarkTransport/stdlib-32                1567        772161 ns/op     169.11 MB/s       53950 B/op         99 allocs/op
BenchmarkTransport/zstd-32                  4579        238503 ns/op     547.51 MB/s       5775 B/op          69 allocs/op

Multi Core:
BenchmarkTransport/gzhttp-par-32           29113         36802 ns/op    3548.27 MB/s       11061 B/op         73 allocs/op
BenchmarkTransport/stdlib-par-32           16114         66442 ns/op    1965.38 MB/s       54971 B/op         99 allocs/op
BenchmarkTransport/zstd-par-32             90177         13110 ns/op    9960.83 MB/s       5361 B/op          67 allocs/op
```

This includes both serving the http request, parsing requests and decompressing. 

### Server

For the simplest usage call `GzipHandler` with any handler (an object which implements the
`http.Handler` interface), and it'll return a new handler which gzips the
response. For example:

```go
package main

import (
	"io"
	"net/http"
	"github.com/klauspost/compress/gzhttp"
)

func main() {
	handler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "text/plain")
		io.WriteString(w, "Hello, World")
	})
    
	http.Handle("/", gzhttp.GzipHandler(handler))
	http.ListenAndServe("0.0.0.0:8000", nil)
}
```

This will wrap a handler using the default options. 

To specify custom options a reusable wrapper can be created that can be used to wrap
any number of handlers.

```Go
package main

import (
	"io"
	"log"
	"net/http"
	
	"github.com/klauspost/compress/gzhttp"
	"github.com/klauspost/compress/gzip"
)

func main() {
	handler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "text/plain")
		io.WriteString(w, "Hello, World")
	})
	
	// Create a reusable wrapper with custom options.
	wrapper, err := gzhttp.NewWrapper(gzhttp.MinSize(2000), gzhttp.CompressionLevel(gzip.BestSpeed))
	if err != nil {
		log.Fatalln(err)
	}
	
	http.Handle("/", wrapper(handler))
	http.ListenAndServe("0.0.0.0:8000", nil)
}

```


### Performance

Speed compared to  [nytimes/gziphandler](https://github.com/nytimes/gziphandler) with default settings, 2KB, 20KB and 100KB:

```
λ benchcmp before.txt after.txt
benchmark                         old ns/op     new ns/op     delta
BenchmarkGzipHandler_S2k-32       51302         23679         -53.84%
BenchmarkGzipHandler_S20k-32      301426        156331        -48.14%
BenchmarkGzipHandler_S100k-32     1546203       818981        -47.03%
BenchmarkGzipHandler_P2k-32       3973          1522          -61.69%
BenchmarkGzipHandler_P20k-32      20319         9397          -53.75%
BenchmarkGzipHandler_P100k-32     96079         46361         -51.75%

benchmark                         old MB/s     new MB/s     speedup
BenchmarkGzipHandler_S2k-32       39.92        86.49        2.17x
BenchmarkGzipHandler_S20k-32      67.94        131.00       1.93x
BenchmarkGzipHandler_S100k-32     66.23        125.03       1.89x
BenchmarkGzipHandler_P2k-32       515.44       1345.31      2.61x
BenchmarkGzipHandler_P20k-32      1007.92      2179.47      2.16x
BenchmarkGzipHandler_P100k-32     1065.79      2208.75      2.07x

benchmark                         old allocs     new allocs     delta
BenchmarkGzipHandler_S2k-32       22             16             -27.27%
BenchmarkGzipHandler_S20k-32      25             19             -24.00%
BenchmarkGzipHandler_S100k-32     28             21             -25.00%
BenchmarkGzipHandler_P2k-32       22             16             -27.27%
BenchmarkGzipHandler_P20k-32      25             19             -24.00%
BenchmarkGzipHandler_P100k-32     27             21             -22.22%

benchmark                         old bytes     new bytes     delta
BenchmarkGzipHandler_S2k-32       8836          2980          -66.27%
BenchmarkGzipHandler_S20k-32      69034         20562         -70.21%
BenchmarkGzipHandler_S100k-32     356582        86682         -75.69%
BenchmarkGzipHandler_P2k-32       9062          2971          -67.21%
BenchmarkGzipHandler_P20k-32      67799         20051         -70.43%
BenchmarkGzipHandler_P100k-32     300972        83077         -72.40%
```

### Stateless compression

In cases where you expect to run many thousands of compressors concurrently, 
but with very little activity you can use stateless compression. 
This is not intended for regular web servers serving individual requests.

Use `CompressionLevel(-3)` or `CompressionLevel(gzip.StatelessCompression)` to enable.
Consider adding a [`bufio.Writer`](https://golang.org/pkg/bufio/#NewWriterSize) with a small buffer.

See [more details on stateless compression](https://github.com/klauspost/compress#stateless-compression).

### Migrating from gziphandler

This package removes some of the extra constructors.
When replacing, this can be used to find a replacement.

* `GzipHandler(h)` -> `GzipHandler(h)` (keep as-is)
* `GzipHandlerWithOpts(opts...)` -> `NewWrapper(opts...)`
* `MustNewGzipLevelHandler(n)` -> `NewWrapper(CompressionLevel(n))`
* `NewGzipLevelAndMinSize(n, s)` -> `NewWrapper(CompressionLevel(n), MinSize(s))` 

By default, some mime types will now be excluded.
To re-enable compression of all types, use the `ContentTypeFilter(gzhttp.CompressAllContentTypeFilter)` option.

### Range Requests

Ranged requests are not well supported with compression.
Therefore any request with a "Content-Range" header is not compressed.

To signify that range requests are not supported any "Accept-Ranges" header set is removed when data is compressed.
If you do not want this behavior use the `KeepAcceptRanges()` option.

### Flushing data

The wrapper supports the [http.Flusher](https://golang.org/pkg/net/http/#Flusher) interface.

The only caveat is that the writer may not yet have received enough bytes to determine if `MinSize`
has been reached. In this case it will assume that the minimum size has been reached.

If nothing has been written to the response writer, nothing will be flushed.

## BREACH mitigation

[BREACH](http://css.csail.mit.edu/6.858/2020/readings/breach.pdf) is a specialized attack where attacker controlled data
is injected alongside secret data in a response body. This can lead to sidechannel attacks, where observing the compressed response
size can reveal if there are overlaps between the secret data and the injected data.

For more information see https://breachattack.com/

It can be hard to judge if you are vulnerable to BREACH. 
In general, if you do not include any user provided content in the response body you are safe,
but if you do, or you are in doubt, you can apply mitigations.

`gzhttp` can apply [Heal the Breach](https://ieeexplore.ieee.org/document/9754554), or improved content aware padding.

```Go
// RandomJitter adds 1->n random bytes to output based on checksum of payload.
// Specify the amount of input to buffer before applying jitter.
// This should cover the sensitive part of your response.
// This can be used to obfuscate the exact compressed size.
// Specifying 0 will use a buffer size of 64KB.
// 'paranoid' will use a slower hashing function, that MAY provide more safety. 
// If a negative buffer is given, the amount of jitter will not be content dependent.
// This provides *less* security than applying content based jitter.
func RandomJitter(n, buffer int, paranoid bool) option
...	
```

The jitter is added as a "Comment" field. This field has a 1 byte overhead, so actual extra size will be 2 -> n+1 (inclusive).

A good option would be to apply 32 random bytes, with default 64KB buffer: `gzhttp.RandomJitter(32, 0, false)`.

Note that flushing the data forces the padding to be applied, which means that only data before the flush is considered for content aware padding.

The *padding* in the comment is the text `Padding-Padding-Padding-Padding-Pad....`

The *length* is `1 + crc32c(payload) MOD n` or `1 + sha256(payload) MOD n` (paranoid), or just random from `crypto/rand` if buffer < 0.

### Paranoid?

The padding size is determined by the remainder of a CRC32 of the content. 

Since the payload contains elements unknown to the attacker, there is no reason to believe they can derive any information
from this remainder, or predict it.

However, for those that feel uncomfortable with a CRC32 being used for this can enable "paranoid" mode which will use SHA256 for determining the padding.

The hashing itself is about 2 orders of magnitude slower, but in overall terms will maybe only reduce speed by 10%.

Paranoid mode has no effect if buffer is < 0 (non-content aware padding).

### Examples

Adding the option `gzhttp.RandomJitter(32, 50000)` will apply from 1 up to 32 bytes of random data to the output.

The number of bytes added depends on the content of the first 50000 bytes, or all of them if the output was less than that.

Adding the option `gzhttp.RandomJitter(32, -1)` will apply from 1 up to 32 bytes of random data to the output.
Each call will apply a random amount of jitter. This should be considered less secure than content based jitter.

This can be used if responses are very big, deterministic and the buffer size would be too big to cover where the mutation occurs.  

## License

[Apache 2.0](LICENSE)


