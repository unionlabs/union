# This Dockerfile builds an image for a client_golang example.
#
# Use as (from the root for the client_golang repository):
#    docker build -f Dockerfile -t prometheus/golang-example .

# Run as
#    docker run -P prometheus/golang-example /random
# or
#    docker run -P prometheus/golang-example /simple

# Test as
#    curl $ip:$port/metrics

# Builder image, where we build the example.
FROM golang:1 AS builder
WORKDIR /go/src/github.com/prometheus/client_golang
COPY . .
WORKDIR /go/src/github.com/prometheus/client_golang/prometheus
RUN go get -d
WORKDIR /go/src/github.com/prometheus/client_golang/examples/random
RUN CGO_ENABLED=0 GOOS=linux go build -a -tags netgo -ldflags '-w'
WORKDIR /go/src/github.com/prometheus/client_golang/examples/simple
RUN CGO_ENABLED=0 GOOS=linux go build -a -tags netgo -ldflags '-w'
WORKDIR /go/src/github.com/prometheus/client_golang/examples/gocollector
RUN CGO_ENABLED=0 GOOS=linux go build -a -tags netgo -ldflags '-w'

# Final image.
FROM quay.io/prometheus/busybox:latest
LABEL maintainer="The Prometheus Authors <prometheus-developers@googlegroups.com>"
COPY --from=builder /go/src/github.com/prometheus/client_golang/examples/random \
    /go/src/github.com/prometheus/client_golang/examples/simple \
    /go/src/github.com/prometheus/client_golang/examples/gocollector ./
EXPOSE 8080
CMD ["echo", "Please run an example. Either /random, /simple or /gocollector"]
