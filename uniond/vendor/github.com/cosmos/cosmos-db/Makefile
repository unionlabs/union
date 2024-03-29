all: lint test

### go tests
## By default this will only test memdb, goleveldb, and pebbledb, which do not require cgo
test:
	@echo "--> Running go test"
	@go test $(PACKAGES) -tags pebbledb -v

test-rocksdb:
	@echo "--> Running go test"
	@go test $(PACKAGES) -tags rocksdb -v

test-pebble:
	@echo "--> Running go test"
	@go test $(PACKAGES) -tags pebbledb -v


test-all:
	@echo "--> Running go test"
	@go test $(PACKAGES) -tags rocksdb,pebbledb -v

golangci_version=v1.55.0

#? lint-install: Install golangci-lint
lint-install:
	@echo "--> Installing golangci-lint $(golangci_version)"
	@go install github.com/golangci/golangci-lint/cmd/golangci-lint@$(golangci_version)
.PHONY: lint-install

lint:
	@echo "--> Running linter"
	$(MAKE) lint-install
	@golangci-lint run
	@go mod verify
.PHONY: lint

format:
	find . -name '*.go' -type f -not -path "*.git*"  -not -name '*.pb.go' -not -name '*pb_test.go' | xargs gofumpt -w -l .
	find . -name '*.go' -type f -not -path "*.git*"  -not -name '*.pb.go' -not -name '*pb_test.go' | xargs golangci-lint run --fix .
.PHONY: format
