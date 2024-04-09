FROM golang:1.14-alpine

# For CGO build
RUN apk add gcc g++

RUN mkdir /app
ADD . /app
WORKDIR /app

RUN go build -o sample ./examples/sample.go

ENTRYPOINT /app/sample

