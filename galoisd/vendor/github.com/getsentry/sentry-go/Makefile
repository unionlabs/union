.DEFAULT_GOAL := help

MKFILE_PATH := $(abspath $(lastword $(MAKEFILE_LIST)))
MKFILE_DIR := $(dir $(MKFILE_PATH))
ALL_GO_MOD_DIRS := $(shell find . -type f -name 'go.mod' -exec dirname {} \; | sort)
GO = go
TIMEOUT = 300

# Parse Makefile and display the help
help: ## Show help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'
.PHONY: help

build: ## Build everything
	for dir in $(ALL_GO_MOD_DIRS); do \
		cd "$${dir}"; \
		echo ">>> Running 'go build' for module: $${dir}"; \
		go build ./...; \
	done;
.PHONY: build

### Tests (inspired by https://github.com/open-telemetry/opentelemetry-go/blob/main/Makefile)
TEST_TARGETS := test-short test-verbose test-race
test-race:    ARGS=-race
test-short:   ARGS=-short
test-verbose: ARGS=-v -race
$(TEST_TARGETS): test
test: $(ALL_GO_MOD_DIRS:%=test/%)  ## Run tests
test/%: DIR=$*
test/%:
	@echo ">>> Running tests for module: $(DIR)"
	@# We use '-count=1' to disable test caching.
	(cd $(DIR) && $(GO) test -count=1 -timeout $(TIMEOUT)s $(ARGS) ./...)
.PHONY: $(TEST_TARGETS) test

# Coverage
COVERAGE_MODE    = atomic
COVERAGE_PROFILE = coverage.out
COVERAGE_REPORT_DIR = .coverage
COVERAGE_REPORT_DIR_ABS = "$(MKFILE_DIR)/$(COVERAGE_REPORT_DIR)"
$(COVERAGE_REPORT_DIR):
	mkdir -p $(COVERAGE_REPORT_DIR)
clean-report-dir: $(COVERAGE_REPORT_DIR)
	test $(COVERAGE_REPORT_DIR) && rm -f $(COVERAGE_REPORT_DIR)/*
test-coverage: $(COVERAGE_REPORT_DIR) clean-report-dir  ## Test with coverage enabled
	set -e ; \
	for dir in $(ALL_GO_MOD_DIRS); do \
	  echo ">>> Running tests with coverage for module: $${dir}"; \
	  DIR_ABS=$$(python -c 'import os, sys; print(os.path.realpath(sys.argv[1]))' $${dir}) ; \
	  REPORT_NAME=$$(basename $${DIR_ABS}); \
	  (cd "$${dir}" && \
	    $(GO) test -count=1 -timeout $(TIMEOUT)s -coverpkg=./... -covermode=$(COVERAGE_MODE) -coverprofile="$(COVERAGE_PROFILE)" ./... && \
		cp $(COVERAGE_PROFILE) "$(COVERAGE_REPORT_DIR_ABS)/$${REPORT_NAME}_$(COVERAGE_PROFILE)" && \
	    $(GO) tool cover -html=$(COVERAGE_PROFILE) -o coverage.html); \
	done;
.PHONY: test-coverage clean-report-dir

mod-tidy: ## Check go.mod tidiness
	set -e ; \
	for dir in $(ALL_GO_MOD_DIRS); do \
		cd "$${dir}"; \
		echo ">>> Running 'go mod tidy' for module: $${dir}"; \
		go mod tidy -go=1.18 -compat=1.18; \
	done; \
	git diff --exit-code;
.PHONY: mod-tidy

vet: ## Run "go vet"
	set -e ; \
	for dir in $(ALL_GO_MOD_DIRS); do \
		cd "$${dir}"; \
		echo ">>> Running 'go vet' for module: $${dir}"; \
		go vet ./...; \
	done;
.PHONY: vet

lint: ## Lint (using "golangci-lint")
	golangci-lint run
.PHONY: lint

fmt: ## Format all Go files
	gofmt -l -w -s .
.PHONY: fmt
