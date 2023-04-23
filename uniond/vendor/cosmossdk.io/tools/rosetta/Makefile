#!/usr/bin/make -f

all: build

build:
	go build ./cmd/rosetta.go

test:
	go test -mod=readonly -race ./...

.PHONY: all build test