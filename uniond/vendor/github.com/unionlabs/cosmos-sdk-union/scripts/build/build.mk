
PACKAGES_NOSIMULATION=$(shell go list ./... | grep -v '/simulation')
PACKAGES_SIMTEST=$(shell go list ./... | grep '/simulation')
export VERSION := $(shell echo $(shell git describe --tags --always --match "v*") | sed 's/^v//')
export COMMIT := $(shell git log -1 --format='%H')
LEDGER_ENABLED ?= true
BINDIR ?= $(GOPATH)/bin
BUILDDIR ?= $(CURDIR)/build
SIMAPP = simapp
MOCKS_DIR = $(CURDIR)/tests/mocks
HTTPS_GIT := https://github.com/cosmos/cosmos-sdk.git
DOCKER := $(shell which docker)
PROJECT_NAME = $(shell git remote get-url origin | xargs basename -s .git)

rocksdb_version=v9.6.1

ifeq ($(findstring .,$(VERSION)),)
	VERSION := 0.0.0
endif

# process build tags
build_tags = netgo
ifeq ($(LEDGER_ENABLED),true)
	ifeq ($(OS),Windows_NT)
	GCCEXE = $(shell where gcc.exe 2> NUL)
	ifeq ($(GCCEXE),)
		$(error gcc.exe not installed for ledger support, please install or set LEDGER_ENABLED=false)
	else
		build_tags += ledger
	endif
	else
	UNAME_S = $(shell uname -s)
	ifeq ($(UNAME_S),OpenBSD)
		$(warning OpenBSD detected, disabling ledger support (https://github.com/cosmos/cosmos-sdk/issues/1988))
	else
		GCC = $(shell command -v gcc 2> /dev/null)
		ifeq ($(GCC),)
			$(error gcc not installed for ledger support, please install or set LEDGER_ENABLED=false)
		else
			build_tags += ledger
		endif
	endif
	endif
endif

ifeq (secp,$(findstring secp,$(COSMOS_BUILD_OPTIONS)))
  build_tags += libsecp256k1_sdk
endif

ifeq (legacy,$(findstring legacy,$(COSMOS_BUILD_OPTIONS)))
  build_tags += app_v1
endif

ifeq (v2,$(findstring v2,$(COSMOS_BUILD_OPTIONS)))
  SIMAPP = simapp/v2
endif

# DB backend selection
ifeq (cleveldb,$(findstring cleveldb,$(COSMOS_BUILD_OPTIONS)))
  build_tags += gcc
endif
ifeq (badgerdb,$(findstring badgerdb,$(COSMOS_BUILD_OPTIONS)))
  build_tags += badgerdb
endif
# handle rocksdb
ifeq (rocksdb,$(findstring rocksdb,$(COSMOS_BUILD_OPTIONS)))
  CGO_ENABLED=1
  build_tags += rocksdb grocksdb_clean_link
endif
# handle boltdb
ifeq (boltdb,$(findstring boltdb,$(COSMOS_BUILD_OPTIONS)))
  build_tags += boltdb
endif

# handle blst
ifeq (blst,$(findstring blst,$(COSMOS_BUILD_OPTIONS)))
  CGO_ENABLED=1
  build_tags += blst
endif

whitespace :=
whitespace += $(whitespace)
comma := ,
build_tags_comma_sep := $(subst $(whitespace),$(comma),$(build_tags))

# process linker flags

ldflags = -X github.com/cosmos/cosmos-sdk/version.Name=sim \
		-X github.com/cosmos/cosmos-sdk/version.AppName=simd \
		-X github.com/cosmos/cosmos-sdk/version.Version=$(VERSION) \
		-X github.com/cosmos/cosmos-sdk/version.Commit=$(COMMIT) \
		-X "github.com/cosmos/cosmos-sdk/version.BuildTags=$(build_tags_comma_sep)"

ifeq (,$(findstring nostrip,$(COSMOS_BUILD_OPTIONS)))
  ldflags += -w -s
endif
ldflags += $(LDFLAGS)
ldflags := $(strip $(ldflags))

build_tags += $(BUILD_TAGS)
build_tags := $(strip $(build_tags))

BUILD_FLAGS := -tags "$(build_tags)" -ldflags '$(ldflags)'
# check for nostrip option
ifeq (,$(findstring nostrip,$(COSMOS_BUILD_OPTIONS)))
  BUILD_FLAGS += -trimpath
endif

# Check for debug option
ifeq (debug,$(findstring debug,$(COSMOS_BUILD_OPTIONS)))
  BUILD_FLAGS += -gcflags "all=-N -l"
endif

#? all: Run tools build 
all: build


BUILD_TARGETS := build install

#? build: Build simapp
build: BUILD_ARGS=-o $(BUILDDIR)/

#? build-linux-amd64: Build simapp for GOOS=linux GOARCH=amd64
build-linux-amd64:
	GOOS=linux GOARCH=amd64 LEDGER_ENABLED=false $(MAKE) build

#? build-linux-arm64: Build simapp for GOOS=linux GOARCH=arm64
build-linux-arm64:
	GOOS=linux GOARCH=arm64 LEDGER_ENABLED=false $(MAKE) build

$(BUILD_TARGETS): go.sum $(BUILDDIR)/
	cd ${CURRENT_DIR}/${SIMAPP} && go $@ -mod=readonly $(BUILD_FLAGS) $(BUILD_ARGS) ./...

$(BUILDDIR)/:
	mkdir -p $(BUILDDIR)/

#? cosmovisor: Build cosmovisor
cosmovisor:
	$(MAKE) -C tools/cosmovisor cosmovisor

#? confix: Build confix
confix:
	$(MAKE) -C tools/confix confix

#? hubl: Build hubl
hubl:
	$(MAKE) -C tools/hubl hubl

.PHONY: build build-linux-amd64 build-linux-arm64 cosmovisor confix

#? mocks: Generate mock file
mocks: $(MOCKS_DIR)
	@go install github.com/golang/mock/mockgen@v1.6.0
	sh ./scripts/mockgen.sh
.PHONY: mocks

#? vulncheck: Run govulncheck
vulncheck: $(BUILDDIR)/
	GOBIN=$(BUILDDIR) go install golang.org/x/vuln/cmd/govulncheck@latest
	$(BUILDDIR)/govulncheck ./...

$(MOCKS_DIR):
	mkdir -p $(MOCKS_DIR)

#? distclean: Run `make clean` and `make tools-clean`
distclean: clean

#? clean: Clean some auto generated directory
clean:
	rm -rf \
	$(BUILDDIR)/ \
	artifacts/ \
	tmp-swagger-gen/ \
	.testnets

.PHONY: distclean clean
