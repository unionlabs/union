FROM golang:1.16.8-alpine3.13 AS build

#ARG PROTOTOOL_VERSION=1.10.0
ARG PROTODOC_VERSION=1.3.2
ARG GRPC_GATEWAY_VERSION=1.16.0
ARG REGEN_GOGOPROTO_VERSION=0.3.0
ARG REGEN_PROTOBUF_VERSION=1.3.2-alpha.regen.4
ARG BUF_VERSION=0.30.0

RUN apk --no-cache add --update curl git libc6-compat make upx

RUN go get -d \
  github.com/gogo/protobuf/gogoproto && \
  mkdir -p /usr/include/google/protobuf/ && \
  mv /go/src/github.com/gogo/protobuf/protobuf/google/protobuf/empty.proto /usr/include/google/protobuf/ && \
  mv /go/src/github.com/gogo/protobuf/protobuf/google/protobuf/descriptor.proto /usr/include/google/protobuf/

RUN GO111MODULE=on go get \
    github.com/grpc-ecosystem/grpc-gateway/protoc-gen-grpc-gateway@v${GRPC_GATEWAY_VERSION} \
    github.com/grpc-ecosystem/grpc-gateway/protoc-gen-swagger@v${GRPC_GATEWAY_VERSION} && \
    mv /go/bin/protoc-gen-grpc-gateway /usr/local/bin/ && \
    mv /go/bin/protoc-gen-swagger /usr/local/bin/

# Install regen fork of gogo proto
# To install a fix version this can only be done via this go.mod workaround
WORKDIR /work
RUN GO111MODULE=on go mod init foobar && \
    go mod edit -replace github.com/gogo/protobuf=github.com/regen-network/protobuf@v${REGEN_PROTOBUF_VERSION} && \
    go get github.com/regen-network/cosmos-proto/protoc-gen-gocosmos@v${REGEN_GOGOPROTO_VERSION} && \
    mv /go/bin/protoc-gen-gocosmos* /usr/local/bin/

RUN GO111MODULE=on go get \
  github.com/pseudomuto/protoc-gen-doc/cmd/protoc-gen-doc@v${PROTODOC_VERSION} && \
  mv /go/bin/protoc-gen-doc /usr/local/bin/

RUN GO111MODULE=on go get \
  github.com/bufbuild/buf/cmd/buf@v${BUF_VERSION} && \
  mv /go/bin/buf /usr/local/bin/

RUN upx --lzma /usr/local/bin/*

FROM golang:1.19-alpine
ENV LD_LIBRARY_PATH=/lib64:/lib

WORKDIR /work
RUN apk --no-cache add --update curl git libc6-compat make
RUN apk --no-cache add --update ca-certificates libc6-compat protoc

COPY --from=build /usr/local/bin /usr/local/bin
COPY --from=build /usr/include /usr/include
RUN chmod -R 755 /usr/include
