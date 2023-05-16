GOTOOLS = \
	github.com/golangci/golangci-lint/cmd/golangci-lint
GOTOOLS_CHECK = golangci-lint

all: check_tools test

########################################
###  Build

build:
	# Nothing to build!

install:
	# Nothing to install!


########################################
### Tools & dependencies

check_tools:
	@# https://stackoverflow.com/a/25668869
	@echo "Found tools: $(foreach tool,$(GOTOOLS_CHECK),\
        $(if $(shell which $(tool)),$(tool),$(error "No $(tool) in PATH")))"

get_tools:
	@echo "--> Installing tools"
	go get -v $(GOTOOLS)

update_tools:
	@echo "--> Updating tools"
	@go get -u -v $(GOTOOLS)


########################################
### Testing

test:
	go test $(shell go list ./... | grep -v vendor)

gofuzz_binary:
	rm -rf tests/fuzz/binary/corpus/
	rm -rf tests/fuzz/binary/crashers/
	rm -rf tests/fuzz/binary/suppressions/
	go run tests/fuzz/binary/init-corpus/main.go --corpus-parent=tests/fuzz/binary
	# TODO: update when https://github.com/dvyukov/go-fuzz/issues/195 is resolved
	GO111MODULE=off go-fuzz-build github.com/tendermint/go-amino/tests/fuzz/binary
	GO111MODULE=off go-fuzz -bin=./fuzz_binary-fuzz.zip -workdir=tests/fuzz/binary

gofuzz_json:
	rm -rf tests/fuzz/json/corpus/
	rm -rf tests/fuzz/json/crashers/
	rm -rf tests/fuzz/json/suppressions/
	# TODO: update when https://github.com/dvyukov/go-fuzz/issues/195 is resolved
	GO111MODULE=off go-fuzz-build github.com/tendermint/go-amino/tests/fuzz/json
	GO111MODULE=off go-fuzz -bin=./fuzz_json-fuzz.zip -workdir=tests/fuzz/json


########################################
### Formatting, linting, and vetting

fmt:
	@go fmt ./...

# look into .golangci.yml for enabling / disabling linters
lint:
	@echo "--> Running linter"
	@golangci-lint run

# To avoid unintended conflicts with file names, always add to .PHONY
# unless there is a reason not to.
# https://www.gnu.org/software/make/manual/html_node/Phony-Targets.html
.PHONY: build install check_tools get_tools fmt lint test
