## To test locally:
# docker build --pull --rm -f "contrib/devtools/Dockerfile" -t cosmossdk-proto:latest "contrib/devtools"
# docker run --rm -v $(pwd):/workspace --workdir /workspace cosmossdk-proto sh ./scripts/protocgen.sh

FROM bufbuild/buf:1.36.0 as BUILDER
FROM golang:1.23-alpine

RUN apk add --no-cache \
  nodejs \
  npm \
  git \
  make \
  clang-extra-tools \
  g++ \
  jq

RUN npm install -g swagger-combine

ARG UNAME=protobuild
ARG UID=1000
RUN adduser -u $UID -s /bin/sh $UNAME -D
USER $UNAME

ENV GOLANG_PROTOBUF_VERSION=1.28.1 \
  GRPC_GATEWAY_VERSION=1.16.0 \
  GRPC_GATEWAY_PROTOC_GEN_OPENAPIV2_VERSION=2.20.0

RUN go install github.com/cosmos/cosmos-proto/cmd/protoc-gen-go-pulsar@latest && \
  go install google.golang.org/protobuf/cmd/protoc-gen-go@v${GOLANG_PROTOBUF_VERSION} && \
  go install github.com/grpc-ecosystem/grpc-gateway/protoc-gen-grpc-gateway@v${GRPC_GATEWAY_VERSION} && \
  go install github.com/grpc-ecosystem/grpc-gateway/v2/protoc-gen-openapiv2@v${GRPC_GATEWAY_PROTOC_GEN_OPENAPIV2_VERSION} && \
  go install cosmossdk.io/orm/cmd/protoc-gen-go-cosmos-orm@v1.0.0-beta.3 && \
  go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@latest

# install all gogo protobuf binaries
RUN git clone https://github.com/cosmos/gogoproto.git; \
  cd gogoproto; \
  go mod download; \
  make install

COPY --from=BUILDER /usr/local/bin /usr/local/bin
