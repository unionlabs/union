Build client and server binaries.

```sh
go build -o ./binaries/client ../../../../interop/xds/client/
go build -o ./binaries/server ../../../../interop/xds/server/
```

Run the test

```sh
go test . -v
```

The client/server paths are flags

```sh
go test . -v -client=$HOME/grpc-java/interop-testing/build/install/grpc-interop-testing/bin/xds-test-client
```
Note that grpc logs are only turned on for Go.
