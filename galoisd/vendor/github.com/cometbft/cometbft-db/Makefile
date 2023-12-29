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

export GO111MODULE = on

all: lint test

### go tests
## By default this will only test memdb & goleveldb
test:
	@echo "--> Running go test"
	@go test $(PACKAGES) -v
.PHONY: test

test-cleveldb:
	@echo "--> Running go test"
	@go test $(PACKAGES) -tags cleveldb -v
.PHONY: test-cleveldb

test-rocksdb:
	@echo "--> Running go test"
	@go test $(PACKAGES) -tags rocksdb -v
.PHONY: test-rocksdb

test-boltdb:
	@echo "--> Running go test"
	@go test $(PACKAGES) -tags boltdb -v
.PHONY: test-boltdb

test-badgerdb:
	@echo "--> Running go test"
	@go test $(PACKAGES) -tags badgerdb -v
.PHONY: test-badgerdb

test-all:
	@echo "--> Running go test"
	@go test $(PACKAGES) -tags cleveldb,boltdb,rocksdb,grocksdb_clean_link,badgerdb -v
.PHONY: test-all

test-all-with-coverage:
	@echo "--> Running go test for all databases, with coverage"
	@CGO_ENABLED=1 go test ./... \
		-mod=readonly \
		-timeout 8m \
		-race \
		-coverprofile=coverage.txt \
		-covermode=atomic \
		-tags=memdb,goleveldb,cleveldb,boltdb,rocksdb,grocksdb_clean_link,badgerdb \
		-v
.PHONY: test-all-with-coverage

lint:
	@echo "--> Running linter"
	@go run github.com/golangci/golangci-lint/cmd/golangci-lint@latest run
	@go mod verify
.PHONY: lint

format:
	find . -name '*.go' -type f -not -path "*.git*" -not -name '*.pb.go' -not -name '*pb_test.go' | xargs gofmt -w -s
	find . -name '*.go' -type f -not -path "*.git*"  -not -name '*.pb.go' -not -name '*pb_test.go' | xargs goimports -w
.PHONY: format

docker-test-image:
	@echo "--> Building Docker test image"
	@cd tools && \
		docker build -t $(DOCKER_TEST_IMAGE):$(DOCKER_TEST_IMAGE_VERSION) .
.PHONY: docker-test-image

# Runs the same test as is executed in CI, but locally.
docker-test:
	@echo "--> Running all tests with all databases with Docker (interactive flags: \"$(DOCKER_TEST_INTERACTIVE_FLAGS)\")"
	@docker run $(DOCKER_TEST_INTERACTIVE_FLAGS) --rm --name cometbft-db-test \
		-v `pwd`:/cometbft \
		-w /cometbft \
		--entrypoint "" \
		$(DOCKER_TEST_IMAGE):$(DOCKER_TEST_IMAGE_VERSION) \
		make test-all-with-coverage
.PHONY: docker-test

tools:
	go get -v $(GOTOOLS)
.PHONY: tools

# generates certificates for TLS testing in remotedb
gen_certs: clean_certs
	certstrap init --common-name "cometbft.com" --passphrase ""
	certstrap request-cert --common-name "remotedb" -ip "127.0.0.1" --passphrase ""
	certstrap sign "remotedb" --CA "cometbft.com" --passphrase ""
	mv out/remotedb.crt remotedb/test.crt
	mv out/remotedb.key remotedb/test.key
	rm -rf out
.PHONY: gen_certs

clean_certs:
	rm -f db/remotedb/test.crt
	rm -f db/remotedb/test.key
.PHONY: clean_certs

%.pb.go: %.proto
	## If you get the following error,
	## "error while loading shared libraries: libprotobuf.so.14: cannot open shared object file: No such file or directory"
	## See https://stackoverflow.com/a/25518702
	## Note the $< here is substituted for the %.proto
	## Note the $@ here is substituted for the %.pb.go
	protoc $(INCLUDE) $< --gogo_out=Mgoogle/protobuf/timestamp.proto=github.com/golang/protobuf/ptypes/timestamp,plugins=grpc:../../..

protoc_remotedb: remotedb/proto/defs.pb.go

vulncheck:
		@go run golang.org/x/vuln/cmd/govulncheck@latest ./...
.PHONY: vulncheck
