#!/usr/bin/make -f

all: confix condiff test

confix:
	go build -mod=readonly ./cmd/confix

condiff:
	go build -mod=readonly ./cmd/condiff

test:
	go test -mod=readonly -race ./...

.PHONY: all confix condiff test
