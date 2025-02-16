//go:build tools

package tools

import (
	_ "github.com/bufbuild/buf/cmd/buf"
	_ "github.com/cosmos/cosmos-proto/cmd/protoc-gen-go-pulsar"
	_ "github.com/cosmos/gogoproto/protoc-gen-gocosmos"
	_ "github.com/grpc-ecosystem/grpc-gateway/protoc-gen-grpc-gateway"
	_ "github.com/grpc-ecosystem/grpc-gateway/v2/protoc-gen-openapiv2"
	_ "golang.org/x/tools/cmd/goimports"
	_ "google.golang.org/grpc/cmd/protoc-gen-go-grpc"
	_ "google.golang.org/protobuf/cmd/protoc-gen-go"
)
