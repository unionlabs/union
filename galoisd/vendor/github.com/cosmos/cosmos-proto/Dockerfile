FROM golang:1.18-alpine

ENV GOLANG_PROTOBUF_VERSION=1.28.1

ARG PROTOC_VERSION="3.20.0"
# add dependency
RUN apk add g++ make curl protoc git
# sanity check to verify its correctly installed
RUN protoc --version
# install
RUN go install google.golang.org/protobuf/cmd/protoc-gen-go@v${GOLANG_PROTOBUF_VERSION}
RUN go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@latest

WORKDIR /build
COPY . ./

RUN go build -o protoc-gen-go-pulsar ./cmd/protoc-gen-go-pulsar

WORKDIR /codegen

RUN mv /build/protoc-gen-go-pulsar /usr/bin/protoc-gen-go-pulsar
