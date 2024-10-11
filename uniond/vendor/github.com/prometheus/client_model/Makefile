# Copyright 2013 Prometheus Team
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at

# http://www.apache.org/licenses/LICENSE-2.0

# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

all:
	$(MAKE) go
	$(MAKE) build

GO_FILE := go/metrics.pb.go
PROTO_FILE := io/prometheus/client/metrics.proto

PROTOC_VERSION := 3.20.3
PROTOC_GEN_GO_VERSION := v1.30.0

# There are no protobuf releases for Darwin ARM so for
# now we always use the x86_64 release through Rosetta.
UNAME_OS := $(shell uname -s)
UNAME_ARCH := $(shell uname -m)
ifeq ($(UNAME_OS),Darwin)
PROTOC_OS := osx
PROTOC_ARCH := x86_64
endif
ifeq ($(UNAME_OS),Linux)
PROTOC_OS = linux
PROTOC_ARCH := $(UNAME_ARCH)
endif
PROTOC := tmp/versions/protoc/$(PROTOC_VERSION)
PROTOC_BIN := tmp/bin/protoc
PROTOC_INCLUDE := tmp/include/google
$(PROTOC):
	@if ! command -v curl >/dev/null 2>/dev/null; then echo "error: curl must be installed"  >&2; exit 1; fi
	@if ! command -v unzip >/dev/null 2>/dev/null; then echo "error: unzip must be installed"  >&2; exit 1; fi
	@rm -f $(PROTOC_BIN)
	@rm -rf $(PROTOC_INCLUDE)
	@mkdir -p $(dir $(PROTOC_BIN)) $(dir $(PROTOC_INCLUDE))
	$(eval PROTOC_TMP := $(shell mktemp -d))
	cd $(PROTOC_TMP); curl -sSL https://github.com/protocolbuffers/protobuf/releases/download/v$(PROTOC_VERSION)/protoc-$(PROTOC_VERSION)-$(PROTOC_OS)-$(PROTOC_ARCH).zip -o protoc.zip
	cd $(PROTOC_TMP); unzip protoc.zip && mv bin/protoc $(PWD)/$(PROTOC_BIN) && mv include/google $(PWD)/$(PROTOC_INCLUDE)
	@rm -rf $(PROTOC_TMP)
	@rm -rf $(dir $(PROTOC))
	@mkdir -p $(dir $(PROTOC))
	@touch $(PROTOC)

PROTOC_GEN_GO := tmp/versions/protoc-gen-go/$(PROTOC_GEN_GO_VERSION)
PROTOC_GEN_GO_BIN := tmp/bin/protoc-gen-go
$(PROTOC_GEN_GO):
	@rm -f $(PROTOC_GEN_GO_BIN)
	@mkdir -p $(dir $(PROTOC_GEN_GO_BIN))
	$(eval PROTOC_GEN_GO_TMP := $(shell mktemp -d))
	cd $(PROTOC_GEN_GO_TMP); GOBIN=$(PWD)/$(dir $(PROTOC_GEN_GO_BIN)) go install google.golang.org/protobuf/cmd/protoc-gen-go@$(PROTOC_GEN_GO_VERSION)
	@rm -rf $(PROTOC_GEN_GO_TMP)
	@rm -rf $(dir $(PROTOC_GEN_GO))
	@mkdir -p $(dir $(PROTOC_GEN_GO))
	@touch $(PROTOC_GEN_GO)

go: $(GO_FILE)

$(GO_FILE): $(PROTO_FILE) $(PROTOC_GEN_GO) $(PROTOC)
	@rm -rf $(dir $(GO_FILE)) tmp/go
	@mkdir -p $(dir $(GO_FILE)) tmp/go
	PATH=$(PWD)/tmp/bin:$$PATH protoc -I tmp/include -I . --go_out=paths=source_relative:tmp/go $(PROTO_FILE)
	@mv tmp/go/$(patsubst %.proto,%.pb.go,$(PROTO_FILE)) $(GO_FILE)

build: $(GO_FILE)
	go build ./go

clean:
	-rm -rf go tmp

.PHONY: all clean go build
