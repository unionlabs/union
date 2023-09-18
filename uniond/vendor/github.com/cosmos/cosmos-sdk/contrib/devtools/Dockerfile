## To test locally:
# docker build --pull --rm -f "contrib/devtools/Dockerfile" -t cosmossdk-proto:latest "contrib/devtools"
# docker run --rm -v $(pwd):/workspace --workdir /workspace cosmossdk-proto sh ./scripts/protocgen.sh

FROM bufbuild/buf:1.9.0 as BUILDER
FROM golang:1.19-alpine

RUN apk add --no-cache \
  nodejs \
  npm \
  git \
  make \
  clang-extra-tools

RUN npm install -g swagger-combine

ARG UNAME=protobuild
ARG UID=1000
RUN adduser -u $UID -s /bin/sh $UNAME -D
USER $UNAME

ENV GOLANG_PROTOBUF_VERSION=1.28.1 \
  GRPC_GATEWAY_VERSION=1.16.0

RUN go install github.com/cosmos/cosmos-proto/cmd/protoc-gen-go-pulsar@latest && \
    go install google.golang.org/protobuf/cmd/protoc-gen-go@v${GOLANG_PROTOBUF_VERSION} && \
    go install github.com/grpc-ecosystem/grpc-gateway/protoc-gen-grpc-gateway@v${GRPC_GATEWAY_VERSION} \
      github.com/grpc-ecosystem/grpc-gateway/protoc-gen-swagger@v${GRPC_GATEWAY_VERSION}

# install all gogo protobuf binaries
RUN git clone https://github.com/cosmos/gogoproto.git; \
  cd gogoproto; \
  go mod download; \
  make install

COPY --from=BUILDER /usr/local/bin /usr/local/bin
