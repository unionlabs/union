COMETBFT_BUILD_OPTIONS += badgerdb,rocksdb,pebbledb,clock_skew,bls12381
IMAGE_TAG=cometbft/e2e-node:local-version

include ../../common.mk

all: docker generator runner

fast: docker-fast generator runner

# This will set up a container with all required dependencies for compiling, copy all the source
# code to the container, and compile the binary inside it.
docker: docker-clean
	@echo "Building E2E Docker image"
	@docker build --tag $(IMAGE_TAG) -f docker/Dockerfile ../..

docker-debug: docker-clean
	@echo "Building E2E Docker image for debugging"
	@docker build --tag $(IMAGE_TAG) -f docker/Dockerfile.debug ../..

# This will compile a binary to be executed in a container, set up a slim container, and copy only
# the binary to it.
docker-fast: docker-clean
	@echo "Compiling binary for slim E2E Docker image"
	@CGO_ENABLED=0 GOOS=linux GOARCH=amd64 go build -o build/node ./node
	@echo "Building slim E2E Docker image"
	@docker build --tag $(IMAGE_TAG) -f docker/Dockerfile.fast .

docker-clean:
	@docker rmi $(IMAGE_TAG) 2>/dev/null; true

# We need to build support for database backends into the app in
# order to build a binary with a CometBFT node in it (for built-in
# ABCI testing).
node:
	go build -race $(BUILD_FLAGS) -tags '$(BUILD_TAGS)' -o build/node ./node

generator:
	go build -o build/generator ./generator

runner:
	go build -o build/runner ./runner

lint:
	@echo "--> Running linter for E2E"
	@go run github.com/golangci/golangci-lint/cmd/golangci-lint@latest run

grammar-gen:
	go run github.com/goccmack/gogll/v3@latest -o pkg/grammar/grammar-auto pkg/grammar/abci_grammar.md

clean: docker-clean
	rm -drf build/
	rm -drf data/

.PHONY: all fast node docker docker-debug docker-fast docker-clean generator runner lint grammar-gen clean
