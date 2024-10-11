FROM golang:1.21-alpine3.18 as builder
ARG IBC_GO_VERSION

RUN set -eux; apk add --no-cache git libusb-dev linux-headers gcc musl-dev make;

ENV GOPATH=""

# ensure the ibc go version is being specified for this image.
RUN test -n "${IBC_GO_VERSION}"

# Copy relevant files before go mod download. Replace directives to local paths break if local
# files are not copied before go mod download.
ADD internal internal
ADD testing testing
ADD modules modules
ADD LICENSE LICENSE

COPY contrib/devtools/Makefile contrib/devtools/Makefile
COPY Makefile .

COPY go.mod .
COPY go.sum .

RUN go mod download

RUN BUILD_TAGS=muslc make build

FROM alpine:3.18
ARG IBC_GO_VERSION

LABEL "org.cosmos.ibc-go" "${IBC_GO_VERSION}"

COPY --from=builder /go/build/simd /bin/simd

ENTRYPOINT ["simd"]

