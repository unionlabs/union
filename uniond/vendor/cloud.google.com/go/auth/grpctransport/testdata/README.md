# testdata

## How to regenerate proto derived files

Ensure you have installed the following tools:

- [protoc](https://grpc.io/docs/protoc-installation/)
- [protoc-gen-go](https://pkg.go.dev/google.golang.org/protobuf/cmd/protoc-gen-go)
- [protoc-gen-go-grpc](https://pkg.go.dev/google.golang.org/grpc/cmd/protoc-gen-go-grpc)

Run the following command from this directory:

```bash
protoc --go_out=. --go_opt=paths=source_relative --go-grpc_out=. --go-grpc_opt=paths=source_relative echo.proto 
```
