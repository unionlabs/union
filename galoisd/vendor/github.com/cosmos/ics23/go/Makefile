test:
	go test .

.PHONY: test 
##### Linting #####

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

