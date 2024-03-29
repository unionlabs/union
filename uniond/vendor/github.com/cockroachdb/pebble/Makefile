GO := go
PKG := ./...
GOFLAGS :=
STRESSFLAGS :=
TAGS := invariants
TESTS := .
PREV_RELEASE := crl-release-23.1
COVER_PROFILE := coverprofile.out

.PHONY: all
all:
	@echo usage:
	@echo "  make test"
	@echo "  make testrace"
	@echo "  make stress"
	@echo "  make stressrace"
	@echo "  make stressmeta"
	@echo "  make crossversion-meta"
	@echo "  make testcoverage"
	@echo "  make mod-update"
	@echo "  make generate"
	@echo "  make generate-test-data"
	@echo "  make clean"

override testflags :=
.PHONY: test
test:
	${GO} test -tags '$(TAGS)' ${testflags} -run ${TESTS} ${PKG}

.PHONY: testcoverage
testcoverage:
	${GO} test -tags '$(TAGS)' ${testflags} -run ${TESTS} ${PKG} -coverprofile ${COVER_PROFILE}

.PHONY: testrace
testrace: testflags += -race -timeout 20m
testrace: test

testasan: testflags += -asan -timeout 20m
testasan: test

testmsan: export CC=clang
testmsan: testflags += -msan -timeout 20m
testmsan: test

.PHONY: testobjiotracing
testobjiotracing:
	${GO} test -tags '$(TAGS) pebble_obj_io_tracing' ${testflags} -run ${TESTS} ./objstorage/objstorageprovider/objiotracing

.PHONY: lint
lint:
	${GO} test -tags '$(TAGS)' ${testflags} -run ${TESTS} ./internal/lint

.PHONY: stress stressrace
stressrace: testflags += -race
stress stressrace: testflags += -exec 'stress ${STRESSFLAGS}' -timeout 0 -test.v
stress stressrace: test

.PHONY: stressmeta
stressmeta: override PKG = ./internal/metamorphic
stressmeta: override STRESSFLAGS += -p 1
stressmeta: override TESTS = TestMeta$$
stressmeta: stress

.PHONY: crossversion-meta
crossversion-meta:
	git checkout ${PREV_RELEASE}; \
		${GO} test -c ./internal/metamorphic -o './internal/metamorphic/crossversion/${PREV_RELEASE}.test'; \
		git checkout -; \
		${GO} test -c ./internal/metamorphic -o './internal/metamorphic/crossversion/head.test'; \
		${GO} test -tags '$(TAGS)' ${testflags} -v -run 'TestMetaCrossVersion' ./internal/metamorphic/crossversion --version '${PREV_RELEASE},${PREV_RELEASE},${PREV_RELEASE}.test' --version 'HEAD,HEAD,./head.test'

.PHONY: stress-crossversion
stress-crossversion:
	STRESS=1 ./scripts/run-crossversion-meta.sh crl-release-21.2 crl-release-22.1 crl-release-22.2 crl-release-23.1 master

.PHONY: generate
generate:
	${GO} generate ${PKG}

generate:

# Note that the output of generate-test-data is not deterministic. This should
# only be run manually as needed.
.PHONY: generate-test-data
generate-test-data:
	${GO} run -tags make_incorrect_manifests ./tool/make_incorrect_manifests.go
	${GO} run -tags make_test_find_db ./tool/make_test_find_db.go
	${GO} run -tags make_test_sstables ./tool/make_test_sstables.go
	${GO} run -tags make_test_remotecat ./tool/make_test_remotecat.go

mod-update:
	${GO} get -u
	${GO} mod tidy

.PHONY: clean
clean:
	rm -f $(patsubst %,%.test,$(notdir $(shell go list ${PKG})))

git_dirty := $(shell git status -s)

.PHONY: git-clean-check
git-clean-check:
ifneq ($(git_dirty),)
	@echo "Git repository is dirty!"
	@false
else
	@echo "Git repository is clean."
endif

.PHONY: mod-tidy-check
mod-tidy-check:
ifneq ($(git_dirty),)
	$(error mod-tidy-check must be invoked on a clean repository)
endif
	@${GO} mod tidy
	$(MAKE) git-clean-check

# TODO(radu): switch back to @latest once bogus doc changes are
# addressed; see https://github.com/cockroachdb/crlfmt/pull/44
.PHONY: format
format:
	go install github.com/cockroachdb/crlfmt@44a36ec7 && crlfmt -w -tab 2 .

.PHONY: format-check
format-check:
ifneq ($(git_dirty),)
	$(error format-check must be invoked on a clean repository)
endif
	$(MAKE) format
	git diff
	$(MAKE) git-clean-check
