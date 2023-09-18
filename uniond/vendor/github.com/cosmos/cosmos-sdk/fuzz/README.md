# Fuzzing

## Running

The fuzz tests are in standard [Go format](https://go.dev/doc/fuzz/).
To run a fuzz test, use the `-fuzz` flag to `go test`. For example:

```shell
go test -fuzz FuzzCryptoHDNewParamsFromPath ./tests
```

## oss-fuzz build status

https://oss-fuzz-build-logs.storage.googleapis.com/index.html#cosmos-sdk
