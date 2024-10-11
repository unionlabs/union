GOTOOLS = github.com/golangci/golangci-lint/cmd/golangci-lint
PACKAGES=$(shell go list ./...)
INCLUDE = -I=${GOPATH}/src/github.com/cometbft/cometbft-db -I=${GOPATH}/src -I=${GOPATH}/src/github.com/gogo/protobuf/protobuf
DOCKER_TEST_IMAGE ?= cometbft/cometbft-db-testing
DOCKER_TEST_IMAGE_VERSION ?= latest
NON_INTERACTIVE ?= 0
DOCKER_TEST_INTERACTIVE_FLAGS ?= -it

ifeq (1,$(NON_INTERACTIVE))
	DOCKER_TEST_INTERACTIVE_FLAGS :=
endif

all: lint test

#? test: Run pure Go tests only
test:
	@echo "--> Running go test"
	@go test $(PACKAGES) -tags boltdb,badgerdb
.PHONY: test

#? test-cleveldb: Run cleveldb tests
test-cleveldb:
	@echo "--> Running go test"
	@go test $(PACKAGES) -tags cleveldb -v
.PHONY: test-cleveldb

#? test-rocksdb: Run rocksdb tests
test-rocksdb:
	@echo "--> Running go test"
	@go test $(PACKAGES) -tags rocksdb -v
.PHONY: test-rocksdb

#? test-boltdb: Run boltdb tests
test-boltdb:
	@echo "--> Running go test"
	@go test $(PACKAGES) -tags boltdb -v
.PHONY: test-boltdb

#? test-badgerdb: Run badgerdb tests
test-badgerdb:
	@echo "--> Running go test"
	@go test $(PACKAGES) -tags badgerdb -v
.PHONY: test-badgerdb

#? test-pebbledb: Run pebbledb tests
test-pebbledb:
	@echo "--> Running go test"
	@go test $(PACKAGES) -v

#? test-all-with-coverage: Run all tests with coverage
test-all-with-coverage:
	@echo "--> Running go test for all databases, with coverage"
	@CGO_ENABLED=1 go test ./... \
		-mod=readonly \
		-timeout 8m \
		-race \
		-coverprofile=coverage.txt \
		-covermode=atomic \
		-tags=cleveldb,boltdb,rocksdb,badgerdb\
		-v
.PHONY: test-all-with-coverage

#? lint: Run linter
lint:
	@echo "--> Running linter"
	@go run github.com/golangci/golangci-lint/cmd/golangci-lint@latest run
	@go mod verify
.PHONY: lint

#? format: Format the code
format:
	find . -name '*.go' -type f -not -path "*.git*" -not -name '*.pb.go' -not -name '*pb_test.go' | xargs gofmt -w -s
	find . -name '*.go' -type f -not -path "*.git*"  -not -name '*.pb.go' -not -name '*pb_test.go' | xargs goimports -w
.PHONY: format

#? docker-test-image: Build the Docker test image
docker-test-image:
	@echo "--> Building Docker test image"
	@cd tools && \
		docker build -t $(DOCKER_TEST_IMAGE):$(DOCKER_TEST_IMAGE_VERSION) .
.PHONY: docker-test-image

#? docker-test: Run the same test as is executed in CI, but locally.
docker-test:
	@echo "--> Running all tests with all databases with Docker (interactive flags: \"$(DOCKER_TEST_INTERACTIVE_FLAGS)\")"
	@docker run $(DOCKER_TEST_INTERACTIVE_FLAGS) --rm --name cometbft-db-test \
		-v `pwd`:/cometbft \
		-w /cometbft \
		--entrypoint "" \
		$(DOCKER_TEST_IMAGE):$(DOCKER_TEST_IMAGE_VERSION) \
		make test-all-with-coverage
.PHONY: docker-test

#? tools: Install tools
tools:
	go get -v $(GOTOOLS)
.PHONY: tools

#? vulncheck: Run go vuln check
vulncheck:
		@go run golang.org/x/vuln/cmd/govulncheck@latest ./...
.PHONY: vulncheck
