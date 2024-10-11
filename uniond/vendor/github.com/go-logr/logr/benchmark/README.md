# Benchmarking logr

Any major changes to the logr library must be benchmarked before and after the
change.

## Running the benchmark

```
$ go test -bench='.' -test.benchmem ./benchmark/
```

## Fixing the benchmark

If you think this benchmark can be improved, you are probably correct!  PRs are
very welcome.
