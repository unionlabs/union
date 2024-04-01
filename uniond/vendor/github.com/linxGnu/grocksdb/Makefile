GOOS ?= $(shell go env GOOS)
GOARCH ?= $(shell go env GOARCH)
GOOS_GOARCH := $(GOOS)_$(GOARCH)
GOOS_GOARCH_NATIVE := $(shell go env GOHOSTOS)_$(shell go env GOHOSTARCH)

ROOT_DIR=$(shell dirname $(realpath $(lastword $(MAKEFILE_LIST))))
DEST=$(ROOT_DIR)/dist/$(GOOS_GOARCH)
DEST_LIB=$(DEST)/lib
DEST_INCLUDE=$(DEST)/include

default: prepare libs

.PHONY: prepare
prepare:
	rm -rf $(DEST)
	mkdir -p $(DEST_LIB) $(DEST_INCLUDE)

.PHONY: libs
libs:
	./build.sh $(DEST)

.PHONY: test
test:
	go test -race -v -count=1 -tags testing,grocksdb_no_link
