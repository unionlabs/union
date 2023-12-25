## Benchmark results

```
goos: windows
goarch: amd64
pkg: github.com/getsentry/sentry-go/internal/trace
cpu: 12th Gen Intel(R) Core(TM) i7-12700K
BenchmarkEqualBytes-20                  44323621                26.08 ns/op
BenchmarkStringEqual-20                 60980257                18.27 ns/op
BenchmarkEqualPrefix-20                 41369181                31.12 ns/op
BenchmarkFullParse-20                     702012              1507 ns/op        1353.42 MB/s        1024 B/op          6 allocs/op
BenchmarkFramesIterator-20               1229971               969.3 ns/op           896 B/op          5 allocs/op
BenchmarkFramesReversedIterator-20       1271061               944.5 ns/op           896 B/op          5 allocs/op
BenchmarkSplitOnly-20                    2250800               534.0 ns/op      3818.23 MB/s         128 B/op          1 allocs/op
```
