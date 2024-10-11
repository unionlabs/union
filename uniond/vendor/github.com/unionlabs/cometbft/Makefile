include common.mk

PACKAGES=$(shell go list ./...)
BUILDDIR?=$(CURDIR)/build
OUTPUT?=$(BUILDDIR)/cometbft

HTTPS_GIT := https://github.com/cometbft/cometbft.git
CGO_ENABLED ?= 0

# Process Docker environment variable TARGETPLATFORM
# in order to build binary with correspondent ARCH
# by default will always build for linux/amd64
TARGETPLATFORM ?=
GOOS ?= linux
GOARCH ?= amd64
GOARM ?=

ifeq (linux/arm,$(findstring linux/arm,$(TARGETPLATFORM)))
	GOOS=linux
	GOARCH=arm
	GOARM=7
endif

ifeq (linux/arm/v6,$(findstring linux/arm/v6,$(TARGETPLATFORM)))
	GOOS=linux
	GOARCH=arm
	GOARM=6
endif

ifeq (linux/arm64,$(findstring linux/arm64,$(TARGETPLATFORM)))
	GOOS=linux
	GOARCH=arm64
	GOARM=7
endif

ifeq (linux/386,$(findstring linux/386,$(TARGETPLATFORM)))
	GOOS=linux
	GOARCH=386
endif

ifeq (linux/amd64,$(findstring linux/amd64,$(TARGETPLATFORM)))
	GOOS=linux
	GOARCH=amd64
endif

ifeq (linux/mips,$(findstring linux/mips,$(TARGETPLATFORM)))
	GOOS=linux
	GOARCH=mips
endif

ifeq (linux/mipsle,$(findstring linux/mipsle,$(TARGETPLATFORM)))
	GOOS=linux
	GOARCH=mipsle
endif

ifeq (linux/mips64,$(findstring linux/mips64,$(TARGETPLATFORM)))
	GOOS=linux
	GOARCH=mips64
endif

ifeq (linux/mips64le,$(findstring linux/mips64le,$(TARGETPLATFORM)))
	GOOS=linux
	GOARCH=mips64le
endif

ifeq (linux/riscv64,$(findstring linux/riscv64,$(TARGETPLATFORM)))
	GOOS=linux
	GOARCH=riscv64
endif

#? all: Run target build, test and install
all: build test install
.PHONY: all

include tests.mk

###############################################################################
###                                Build CometBFT                           ###
###############################################################################

#? build: Build CometBFT
build:
	CGO_ENABLED=$(CGO_ENABLED) go build $(BUILD_FLAGS) -tags '$(BUILD_TAGS)' -o $(OUTPUT) ./cmd/cometbft/
.PHONY: build

#? install: Install CometBFT to GOBIN
install:
	CGO_ENABLED=$(CGO_ENABLED) go install $(BUILD_FLAGS) -tags '$(BUILD_TAGS)' ./cmd/cometbft
.PHONY: install

###############################################################################
###                               Metrics                                   ###
###############################################################################

#? metrics: Generate metrics
metrics: testdata-metrics
	go generate -run="scripts/metricsgen" ./...
.PHONY: metrics

# By convention, the go tool ignores subdirectories of directories named
# 'testdata'. This command invokes the generate command on the folder directly
# to avoid this.
#? testdata-metrics: Generate test data for metrics
testdata-metrics:
	ls ./scripts/metricsgen/testdata | xargs -I{} go generate -v -run="scripts/metricsgen" ./scripts/metricsgen/testdata/{}
.PHONY: testdata-metrics

###############################################################################
###                                Mocks                                    ###
###############################################################################

#? mockery: Generate test mocks
mockery:
	go generate -run="./scripts/mockery_generate.sh" ./...
.PHONY: mockery

###############################################################################
###                                Protobuf                                 ###
###############################################################################

#? check-proto-deps: Check protobuf deps
check-proto-deps:
ifeq (,$(shell which protoc-gen-gocosmos))
	@go install github.com/cosmos/gogoproto/protoc-gen-gocosmos@latest
endif
.PHONY: check-proto-deps

#? check-proto-format-deps: Check protobuf format deps
check-proto-format-deps:
ifeq (,$(shell which clang-format))
	$(error "clang-format is required for Protobuf formatting. See instructions for your platform on how to install it.")
endif
.PHONY: check-proto-format-deps

#? proto-gen: Generate protobuf files
proto-gen: check-proto-deps
	@echo "Generating Protobuf files"
	@go run github.com/bufbuild/buf/cmd/buf@latest generate --path proto/cometbft
.PHONY: proto-gen

# These targets are provided for convenience and are intended for local
# execution only.
#? proto-lint: Lint protobuf files
proto-lint: check-proto-deps
	@echo "Linting Protobuf files"
	@go run github.com/bufbuild/buf/cmd/buf@latest lint
.PHONY: proto-lint

#? proto-format: Format protobuf files
proto-format: check-proto-format-deps
	@echo "Formatting Protobuf files"
	@find . -name '*.proto' -path "./proto/*" -exec clang-format -i {} \;
.PHONY: proto-format

#? proto-check-breaking: Check for breaking changes in Protobuf files against local branch. This is only useful if your changes have not yet been committed
proto-check-breaking: check-proto-deps
	@echo "Checking for breaking changes in Protobuf files against local branch"
	@echo "Note: This is only useful if your changes have not yet been committed."
	@echo "      Otherwise read up on buf's \"breaking\" command usage:"
	@echo "      https://docs.buf.build/breaking/usage"
	@go run github.com/bufbuild/buf/cmd/buf@latest breaking --against ".git"
.PHONY: proto-check-breaking

#? proto-check-breaking-ci: Check for breaking changes in Protobuf files against v0.34.x. This is only useful if your changes have not yet been committed
proto-check-breaking-ci:
	@go run github.com/bufbuild/buf/cmd/buf@latest breaking --against $(HTTPS_GIT)#branch=v0.34.x
.PHONY: proto-check-breaking-ci

###############################################################################
###                              Build ABCI                                 ###
###############################################################################

#? build_abci: Build abci
build_abci:
	@go build -mod=readonly -i ./abci/cmd/...
.PHONY: build_abci

#? install_abci: Install abci
install_abci:
	@go install -mod=readonly ./abci/cmd/...
.PHONY: install_abci

###############################################################################
###                              Distribution                               ###
###############################################################################

# dist builds binaries for all platforms and packages them for distribution
# TODO add abci to these scripts
#? dist: Build binaries for all platforms and package them for distribution
dist:
	@BUILD_TAGS=$(BUILD_TAGS) sh -c "'$(CURDIR)/scripts/dist.sh'"
.PHONY: dist

#? go-mod-cache: Download go modules to local cache
go-mod-cache: go.sum
	@echo "--> Download go modules to local cache"
	@go mod download
.PHONY: go-mod-cache

#? go.sum: Ensure dependencies have not been modified
go.sum: go.mod
	@echo "--> Ensure dependencies have not been modified"
	@go mod verify
	@go mod tidy
.PHONY: go.sum

#? draw_deps: Generate deps graph
draw_deps:
	@# requires brew install graphviz or apt-get install graphviz
	go get github.com/RobotsAndPencils/goviz
	@goviz -i github.com/cometbft/cometbft/cmd/cometbft -d 3 | dot -Tpng -o dependency-graph.png
.PHONY: draw_deps

get_deps_bin_size:
	@# Copy of build recipe with additional flags to perform binary size analysis
	$(eval $(shell go build -work -a $(BUILD_FLAGS) -tags $(BUILD_TAGS) -o $(OUTPUT) ./cmd/cometbft/ 2>&1))
	@find $(WORK) -type f -name "*.a" | xargs -I{} du -hxs "{}" | sort -rh | sed -e s:${WORK}/::g > deps_bin_size.log
	@echo "Results can be found here: $(CURDIR)/deps_bin_size.log"
.PHONY: get_deps_bin_size

###############################################################################
###                                  Libs                                   ###
###############################################################################

#? gen_certs: Generate certificates for TLS testing in remotedb and RPC server
gen_certs: clean_certs
	certstrap init --common-name "cometbft.com" --passphrase ""
	certstrap request-cert --common-name "server" -ip "127.0.0.1" --passphrase ""
	certstrap sign "server" --CA "cometbft.com" --passphrase ""
	mv out/server.crt rpc/jsonrpc/server/test.crt
	mv out/server.key rpc/jsonrpc/server/test.key
	rm -rf out
.PHONY: gen_certs

#? clean_certs: Delete generated certificates
clean_certs:
	rm -f rpc/jsonrpc/server/test.crt
	rm -f rpc/jsonrpc/server/test.key
.PHONY: clean_certs

###############################################################################
###                  Formatting, linting, and vetting                       ###
###############################################################################

#? lint: Lint, format and fix typos
lint: pre-commit
	@pre-commit run
.PHONY: lint

#? vulncheck: Run latest govulncheck
vulncheck:
	@go run golang.org/x/vuln/cmd/govulncheck@latest ./...
.PHONY: vulncheck

#? pre-commit: Create pre-commit hook using the pre-commit framework.
pre-commit:
	@which pre-commit || pip3 install pre-commit
	@pre-commit install
.PHONY: pre-commit

DESTINATION = ./index.html.md


###############################################################################
###                           Documentation                                 ###
###############################################################################

#? check-docs-toc: Verify that important design docs have ToC entries.
check-docs-toc:
	@./docs/presubmit.sh
.PHONY: check-docs-toc

###############################################################################
###                            Docker image                                 ###
###############################################################################

# On Linux, you may need to run `DOCKER_BUILDKIT=1 make build-docker` for this
# to work.
#? build-docker: Build docker image cometbft/cometbft
build-docker:
	docker build \
		--label=cometbft \
		--tag="cometbft/cometbft" \
		-f DOCKER/Dockerfile .
.PHONY: build-docker

###############################################################################
###                       Local testnet using docker                        ###
###############################################################################

#? build-linux: Build linux binary on other platforms
build-linux:
	GOOS=$(GOOS) GOARCH=$(GOARCH) GOARM=$(GOARM) $(MAKE) build
.PHONY: build-linux

#? build-docker-localnode: Build the "localnode" docker image
build-docker-localnode:
	@cd networks/local && make
.PHONY: build-docker-localnode


#? localnet-start: Run a 4-node testnet locally
localnet-start: localnet-stop build-docker-localnode
	@if ! [ -f build/node0/config/genesis.json ]; then docker run --rm -v $(CURDIR)/build:/cometbft:Z cometbft/localnode testnet --config /etc/cometbft/config-template.toml --o . --starting-ip-address 192.167.10.2; fi
	docker compose up -d
.PHONY: localnet-start

#? localnet-stop: Stop testnet
localnet-stop:
	docker compose down
.PHONY: localnet-stop

#? monitoring-start: Start Prometheus and Grafana servers for localnet monitoring
monitoring-start:
	cd test/monitoring && docker compose up -d

#? monitoring-stop: Stop the Prometheus and Grafana servers
monitoring-stop:
	cd test/monitoring && docker compose down

#? build-contract-tests-hooks: Build hooks for dredd, to skip or add information on some steps
build-contract-tests-hooks:
ifeq ($(OS),Windows_NT)
	go build -mod=readonly $(BUILD_FLAGS) -o build/contract_tests.exe ./cmd/contract_tests
else
	go build -mod=readonly $(BUILD_FLAGS) -o build/contract_tests ./cmd/contract_tests
endif
.PHONY: build-contract-tests-hooks

#? contract-tests: Run a nodejs tool to test endpoints against a localnet
# The command takes care of starting and stopping the network
# prerequisites: build-contract-tests-hooks build-linux
# the two build commands were not added to let this command run from generic containers or machines.
# The binaries should be built beforehand
contract-tests:
	dredd
.PHONY: contract-tests

# Implements test splitting and running. This is pulled directly from
# the github action workflows for better local reproducibility.

GO_TEST_FILES != find $(CURDIR) -name "*_test.go"

# default to four splits by default
NUM_SPLIT ?= 4

$(BUILDDIR):
	mkdir -p $@

# The format statement filters out all packages that don't have tests.
# Note we need to check for both in-package tests (.TestGoFiles) and
# out-of-package tests (.XTestGoFiles).
$(BUILDDIR)/packages.txt:$(GO_TEST_FILES) $(BUILDDIR)
	go list -f "{{ if (or .TestGoFiles .XTestGoFiles) }}{{ .ImportPath }}{{ end }}" ./... | sort > $@

split-test-packages:$(BUILDDIR)/packages.txt
	split -d -n l/$(NUM_SPLIT) $< $<.
test-group-%:split-test-packages
	cat $(BUILDDIR)/packages.txt.$* | xargs go test -mod=readonly -timeout=15m -race -coverprofile=$(BUILDDIR)/$*.profile.out

#? help: Get more info on make commands.
help: Makefile
	@echo " Choose a command run in comebft:"
	@sed -n 's/^#?//p' $< | column -t -s ':' |  sort | sed -e 's/^/ /'
.PHONY: help
