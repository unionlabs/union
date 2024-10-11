# Copyright 2018 The Prometheus Authors
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
# http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

include .bingo/Variables.mk
include Makefile.common

.PHONY: test
test: deps common-test

.PHONY: test-short
test-short: deps common-test-short

# Overriding Makefile.common check_license target to add
# dagger paths
.PHONY: common-check_license
common-check_license:
	@echo ">> checking license header"
	@licRes=$$(for file in $$(find . -type f -iname '*.go' ! -path './vendor/*' ! -path './dagger/internal/*') ; do \
               awk 'NR<=3' $$file | grep -Eq "(Copyright|generated|GENERATED)" || echo $$file; \
       done); \
       if [ -n "$${licRes}" ]; then \
               echo "license header checking failed:"; echo "$${licRes}"; \
               exit 1; \
       fi

.PHONY: generate-go-collector-test-files
file := supported_go_versions.txt
VERSIONS := $(shell cat ${file})
generate-go-collector-test-files:
	for GO_VERSION in $(VERSIONS); do \
		docker run \
			--platform linux/amd64 \
			--rm -v $(PWD):/workspace \
			-w /workspace \
			golang:$$GO_VERSION \
			bash ./generate-go-collector.bash; \
	done; \
	go mod tidy

.PHONY: fmt
fmt: common-format
	$(GOIMPORTS) -local github.com/prometheus/client_golang -w .
