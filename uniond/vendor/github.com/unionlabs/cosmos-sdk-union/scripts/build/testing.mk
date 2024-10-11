
# make init-simapp initializes a single local node network
# it is useful for testing and development
# Usage: make install && make init-simapp && simd start
# Warning: make init-simapp will remove all data in simapp home directory
#? init-simapp: Initializes a single local node network
init-simapp:
	./scripts/init-simapp.sh
init-simapp-v2:
	./scripts/init-simapp-v2.sh

#? test: Run `make test-unit`
test: test-unit
#? test-e2e: Run `make -C tests test-e2e`
test-e2e:
	$(MAKE) -C tests test-e2e
#? test-e2e-cov: Run `make -C tests test-e2e-cov`
test-e2e-cov:
	$(MAKE) -C tests test-e2e-cov
#? test-integration: Run `make -C tests test-integration`
test-integration:
	$(MAKE) -C tests test-integration
#? test-integration-cov: Run `make -C tests test-integration-cov`
test-integration-cov:
	$(MAKE) -C tests test-integration-cov
#? test-all: Run all test
test-all: test-unit test-e2e test-integration test-ledger-mock test-race

.PHONY: test-system
test-system: build
	mkdir -p ./tests/systemtests/binaries/
	cp $(BUILDDIR)/simd$(if $(findstring v2,$(COSMOS_BUILD_OPTIONS)),v2) ./tests/systemtests/binaries/
	$(MAKE) -C tests/systemtests test


TEST_PACKAGES=./...
TEST_TARGETS := test-unit test-unit-amino test-unit-proto test-ledger-mock test-race test-ledger test-race

# Test runs-specific rules. To add a new test target, just add
# a new rule, customise ARGS or TEST_PACKAGES ad libitum, and
# append the new rule to the TEST_TARGETS list.
test-unit: test_tags += cgo ledger test_ledger_mock norace
test-ledger: test_tags += cgo ledger norace
test-ledger-mock: test_tags += ledger test_ledger_mock norace
test-race: test_tags += cgo ledger test_ledger_mock
test-race: ARGS=-race
test-race: TEST_PACKAGES=$(PACKAGES_NOSIMULATION)
$(TEST_TARGETS): run-tests

# check-* compiles and collects tests without running them
# note: go test -c doesn't support multiple packages yet (https://github.com/golang/go/issues/15513)
CHECK_TEST_TARGETS := check-test-unit check-test-unit-amino
check-test-unit: test_tags += cgo ledger test_ledger_mock norace
$(CHECK_TEST_TARGETS): EXTRA_ARGS=-run=none
$(CHECK_TEST_TARGETS): run-tests

ARGS += -tags "$(test_tags)"
SUB_MODULES = $(shell find . -type f -name 'go.mod' -print0 | xargs -0 -n1 dirname | sort)
CURRENT_DIR = $(shell pwd)
#? run-tests: Run every sub modules' tests
run-tests:
ifneq (,$(shell which tparse 2>/dev/null))
	@echo "Starting unit tests"; \
	finalec=0; \
	for module in $(SUB_MODULES); do \
		cd ${CURRENT_DIR}/$$module; \
		echo "Running unit tests for $$(grep '^module' go.mod)"; \
		go test -mod=readonly -json $(ARGS) $(TEST_PACKAGES) ./... | tparse; \
		ec=$$?; \
		if [ "$$ec" -ne '0' ]; then finalec=$$ec; fi; \
	done; \
	exit $$finalec
else
	@echo "Starting unit tests"; \
	finalec=0; \
	for module in $(SUB_MODULES); do \
		cd ${CURRENT_DIR}/$$module; \
		echo "Running unit tests for $$(grep '^module' go.mod)"; \
		go test -mod=readonly $(ARGS) $(TEST_PACKAGES) ./... ; \
		ec=$$?; \
		if [ "$$ec" -ne '0' ]; then finalec=$$ec; fi; \
	done; \
	exit $$finalec
endif

.PHONY: run-tests test test-all $(TEST_TARGETS)
