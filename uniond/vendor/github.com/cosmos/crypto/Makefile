# Variables
PKG := ./...
GOFILES := $(shell find . -name '*.go' | grep -v _test.go)
TESTFILES := $(shell find . -name '*_test.go')
GOLANGCI_VERSION := v1.59.0

all: build

build:
	@if [ -z "$(TAGS)" ]; then \
		echo "Building..."; \
	else \
		echo "Building with tags: $(TAGS)"; \
	fi
	@go build -tags "$(TAGS)" $(PKG)

# Run tests
test:
	@if [ -z "$(TAGS)" ]; then \
		echo "Running tests..."; \
	else \
		echo "Running tests with tags: $(TAGS)"; \
	fi
	@go test -tags "$(TAGS)" -v $(PKG)

# Install golangci-lint
lint-install:
	@echo "--> Installing golangci-lint $(GOLANGCI_VERSION)"
	@go install github.com/golangci/golangci-lint/cmd/golangci-lint@$(GOLANGCI_VERSION)

# Run golangci-lint
lint:
	@echo "--> Running linter"
	$(MAKE) lint-install
	@golangci-lint run --timeout=15m

# Run golangci-lint and fix
lint-fix:
	@echo "--> Running linter with fix"
	$(MAKE) lint-install
	@golangci-lint run --fix

.PHONY: build test lint-install lint lint-fix
