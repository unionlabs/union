export GO111MODULE := on
export PATH := ./bin:$(PATH)

ci: bootstrap lint cover
.PHONY: ci

#################################################
# Bootstrapping for base golang package and tool deps
#################################################

bootstrap:
	curl -sfL https://raw.githubusercontent.com/golangci/golangci-lint/master/install.sh| sh -s v1.21.0
.PHONY: bootstrap

mod-update:
	go get -u -m
	go mod tidy

mod-tidy:
	go mod tidy

.PHONY: $(CMD_PKGS)
.PHONY: mod-update mod-tidy

#################################################
# Test and linting
#################################################
# Run all the linters
lint:
	bin/golangci-lint run ./...
.PHONY: lint

test:
	CGO_ENABLED=0 go test $$(go list ./... | grep -v generated)
.PHONY: test

COVER_TEST_PKGS:=$(shell find . -type f -name '*_test.go' | rev | cut -d "/" -f 2- | rev | grep -v generated | sort -u)
$(COVER_TEST_PKGS:=-cover): %-cover: all-cover.txt
	@CGO_ENABLED=0 go test -v -coverprofile=$@.out -covermode=atomic ./$*
	@if [ -f $@.out ]; then \
		grep -v "mode: atomic" < $@.out >> all-cover.txt; \
		rm $@.out; \
	fi

all-cover.txt:
	echo "mode: atomic" > all-cover.txt

cover: all-cover.txt $(COVER_TEST_PKGS:=-cover)
.PHONY: cover all-cover.txt
