FROM golang:1.21-alpine3.18 as builder

ARG LIBWASM_VERSION
ARG LIBWASM_CHECKSUM

RUN test -n "${LIBWASM_VERSION}"
RUN test -n "${LIBWASM_CHECKSUM}"

RUN set -eux; apk add --no-cache git libusb-dev linux-headers gcc musl-dev make;

ENV GOPATH=""

# Grab the static library and copy it to location that will be found by the linker flag `-lwasmvm_muslc`.
ADD https://github.com/CosmWasm/wasmvm/releases/download/${LIBWASM_VERSION}/libwasmvm_muslc.x86_64.a /lib/libwasmvm_muslc.x86_64.a
RUN sha256sum /lib/libwasmvm_muslc.x86_64.a | grep ${LIBWASM_CHECKSUM}
RUN cp /lib/libwasmvm_muslc.x86_64.a /lib/libwasmvm_muslc.a

# Copy relevant files before go mod download. Replace directives to local paths break if local
# files are not copied before go mod download.
ADD internal internal
ADD testing testing
ADD modules modules
ADD LICENSE LICENSE

COPY go.mod .
COPY go.sum .

WORKDIR /go/modules/light-clients/08-wasm

RUN go mod download

RUN GOOS=linux GOARCH=amd64 go build -mod=readonly -tags "netgo ledger muslc" -ldflags '-X github.com/cosmos/cosmos-sdk/version.Name=sim -X github.com/cosmos/cosmos-sdk/version.AppName=simd -X github.com/cosmos/cosmos-sdk/version.Version= -X github.com/cosmos/cosmos-sdk/version.Commit= -X "github.com/cosmos/cosmos-sdk/version.BuildTags=netgo ledger muslc," -w -s -linkmode=external -extldflags "-Wl,-z,muldefs -static"' -trimpath -o /go/build/ ./...

FROM alpine:3.18

COPY --from=builder /go/build/simd /bin/simd

ENTRYPOINT ["simd"]
