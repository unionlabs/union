# go-xerial-snappy

Xerial-compatible Snappy framing support for golang.

Packages using Xerial for snappy encoding use a framing format incompatible with
basically everything else in existence.

Apps that use this format include Apache Kafka (see
https://github.com/dpkp/kafka-python/issues/126#issuecomment-35478921 for
details).

# Fork

Forked from [github.com/eapache/go-xerial-snappy](https://github.com/eapache/go-xerial-snappy).

Changes:

* Uses [S2](https://github.com/klauspost/compress/tree/master/s2#snappy-compatibility) for better/faster compression and decompression.
* Fixes 0-length roundtrips.
* Adds `DecodeCapped`, which allows decompression with capped output size.
* `DecodeInto` will decode directly into destination if there is space enough.
* `Encode` will now encode directly into 'dst' if it has space enough.
* Fixes short snappy buffers returning `ErrMalformed`.
* Renames `EncodeStream` to `Encode`.
* Adds `EncodeBetter` for better than default compression at ~half the speed.


Comparison (before/after):

```
BenchmarkSnappyStreamEncode-32    	  959010	      1170 ns/op	 875.15 MB/s	    1280 B/op	       1 allocs/op
BenchmarkSnappyStreamEncode-32    	 1000000	      1107 ns/op	 925.04 MB/s	       0 B/op	       0 allocs/op
--> Output size: 913 -> 856 bytes

BenchmarkSnappyStreamEncodeBetter-32    	  477739	      2506 ns/op	 408.62 MB/s	       0 B/op	       0 allocs/op
--> Output size: 835 bytes

BenchmarkSnappyStreamEncodeMassive-32  	     100    	  10596963 ns/op	 966.31 MB/s	   40977 B/op	       1 allocs/op
BenchmarkSnappyStreamEncodeMassive-32  	     100       	  10220236 ns/op	1001.93 MB/s	       0 B/op	       0 allocs/op
--> Output size: 2365547 -> 2256991 bytes

BenchmarkSnappyStreamEncodeBetterMassive-32    	      69	  16983314 ns/op	 602.94 MB/s	       0 B/op	       0 allocs/op
--> Output size: 2011997 bytes

BenchmarkSnappyStreamDecodeInto-32    	 1887378	       639.5 ns/op	1673.19 MB/s	    1088 B/op	       3 allocs/op
BenchmarkSnappyStreamDecodeInto-32    	 2707915	       436.2 ns/op	2452.99 MB/s	       0 B/op	       0 allocs/op

BenchmarkSnappyStreamDecodeIntoMassive-32    	     267	   4559594 ns/op	2245.81 MB/s	   71120 B/op	       1 allocs/op
BenchmarkSnappyStreamDecodeIntoMassive-32    	     282	   4285844 ns/op	2389.26 MB/s	       0 B/op	       0 allocs/op
```