VERSION := $(shell echo $(shell git describe --tags) | sed 's/^v//')
COMMIT := $(shell git log -1 --format='%H')
BRANCH=$(shell git rev-parse --abbrev-ref HEAD)
DOCKER_BUF := docker run -v $(shell pwd):/workspace --workdir /workspace bufbuild/buf
DOCKER := $(shell which docker)
HTTPS_GIT := https://github.com/cosmos/iavl.git

PDFFLAGS := -pdf --nodefraction=0.1
CMDFLAGS := -ldflags -X TENDERMINT_IAVL_COLORS_ON=on 
LDFLAGS := -ldflags "-X github.com/cosmos/iavl.Version=$(VERSION) -X github.com/cosmos/iavl.Commit=$(COMMIT) -X github.com/cosmos/iavl.Branch=$(BRANCH)"

all: lint test install

install:
ifeq ($(COLORS_ON),)
	go install ./cmd/iaviewer
else
	go install $(CMDFLAGS) ./cmd/iaviewer
endif
.PHONY: install

test-short:
	@echo "--> Running go test"
	@go test ./... $(LDFLAGS) -v --race --short
.PHONY: test-short

test:
	@echo "--> Running go test"
	@go test ./... $(LDFLAGS) -v 
.PHONY: test

format:
	find . -name '*.go' -type f -not -path "*.git*" -not -name '*.pb.go' -not -name '*pb_test.go' | xargs gofmt -w -s
	find . -name '*.go' -type f -not -path "*.git*"  -not -name '*.pb.go' -not -name '*pb_test.go' | xargs goimports -format
.PHONY: format

# look into .golangci.yml for enabling / disabling linters
golangci_lint_cmd=golangci-lint
golangci_version=v1.55.2

lint:
	@echo "--> Running linter"
	@go install github.com/golangci/golangci-lint/cmd/golangci-lint@$(golangci_version)
	@$(golangci_lint_cmd) run --timeout=10m

lint-fix:
	@echo "--> Running linter"
	@go install github.com/golangci/golangci-lint/cmd/golangci-lint@$(golangci_version)
	@$(golangci_lint_cmd) run --fix --out-format=tab --issues-exit-code=0

# bench is the basic tests that shouldn't crash an aws instance
bench:
	cd benchmarks && \
		go test $(LDFLAGS) -tags pebbledb -run=NOTEST -bench=Small . && \
		go test $(LDFLAGS) -tags pebbledb -run=NOTEST -bench=Medium . && \
		go test $(LDFLAGS) -run=NOTEST -bench=RandomBytes .
.PHONY: bench

# fullbench is extra tests needing lots of memory and to run locally
fullbench:
	cd benchmarks && \
		go test $(LDFLAGS) -run=NOTEST -bench=RandomBytes . && \
		go test $(LDFLAGS) -tags rocksdb,pebbledb -run=NOTEST -bench=Small . && \
		go test $(LDFLAGS) -tags rocksdb,pebbledb -run=NOTEST -bench=Medium . && \
		go test $(LDFLAGS) -tags rocksdb,pebbledb -run=NOTEST -timeout=30m -bench=Large . && \
		go test $(LDFLAGS) -run=NOTEST -bench=Mem . && \
		go test $(LDFLAGS) -run=NOTEST -timeout=60m -bench=LevelDB .
.PHONY: fullbench

# note that this just profiles the in-memory version, not persistence
profile:
	cd benchmarks && \
		go test $(LDFLAGS) -bench=Mem -cpuprofile=cpu.out -memprofile=mem.out . && \
		go tool pprof ${PDFFLAGS} benchmarks.test cpu.out > cpu.pdf && \
		go tool pprof --alloc_space ${PDFFLAGS} benchmarks.test mem.out > mem_space.pdf && \
		go tool pprof --alloc_objects ${PDFFLAGS} benchmarks.test mem.out > mem_obj.pdf
.PHONY: profile

explorecpu:
	cd benchmarks && \
		go tool pprof benchmarks.test cpu.out
.PHONY: explorecpu

exploremem:
	cd benchmarks && \
		go tool pprof --alloc_objects benchmarks.test mem.out
.PHONY: exploremem

delve:
	dlv test ./benchmarks -- -test.bench=.
.PHONY: delve
