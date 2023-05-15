.PHONY: protoc test

# make sure we turn on go modules
export GO111MODULE := on

# PROTOC_FLAGS := -I=.. -I=./vendor -I=$(GOPATH)/src
PROTOC_FLAGS := -I=.. -I=$(GOPATH)/src

test:
	go test .

protoc:
#	@go mod vendor
	protoc --gocosmos_out=plugins=interfacetype+grpc,Mgoogle/protobuf/any.proto=github.com/gogo/protobuf/types:. $(PROTOC_FLAGS) ../proofs.proto

install-proto-dep:
	@echo "Installing protoc-gen-gocosmos..."
	@go install github.com/regen-network/cosmos-proto/protoc-gen-gocosmos


golangci_lint_cmd=golangci-lint
golangci_version=v1.50.0

lint:
	@echo "--> Running linter"
	@go install github.com/golangci/golangci-lint/cmd/golangci-lint@$(golangci_version)
	@$(golangci_lint_cmd) run --timeout=10m

lint-fix:
	@echo "--> Running linter"
	@go install github.com/golangci/golangci-lint/cmd/golangci-lint@$(golangci_version)
	@$(golangci_lint_cmd) run --fix --out-format=tab --issues-exit-code=0

.PHONY: lint lint-fix

format:
	@go install mvdan.cc/gofumpt@latest
	@go install github.com/golangci/golangci-lint/cmd/golangci-lint@$(golangci_version)
	find . -name '*.go' -type f -not -name "*.pb.go"  | xargs gofumpt -w -l
	$(golangci_lint_cmd) run --fix
.PHONY: format
