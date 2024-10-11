GOMOD="github.com/cometbft/cometbft/test/loadtime"
OUTPUT?=build/

build:
	go build $(BUILD_FLAGS) -tags '$(BUILD_TAGS)' -o $(OUTPUT)load ./cmd/load/
	go build $(BUILD_FLAGS) -tags '$(BUILD_TAGS)' -o $(OUTPUT)report ./cmd/report/
.PHONY: build

check-proto-gen-deps:
ifeq (,$(shell which protoc))
	$(error "protoc is required for Protobuf generation. See instructions for your platform on how to install it.")
endif
ifeq (,$(shell which protoc-gen-go))
	$(error "protoc-gen-go is required for Protobuf generation. See instructions for your platform on how to install it.")
endif
.PHONY: check-proto-gen-deps

check-proto-format-deps:
ifeq (,$(shell which clang-format))
	$(error "clang-format is required for Protobuf formatting. See instructions for your platform on how to install it.")
endif
.PHONY: check-proto-format-deps

proto-format: check-proto-format-deps
	@echo "Formatting Protobuf files"
	@find . -name '*.proto' -exec clang-format -i {} \;
.PHONY: proto-format

proto-gen: check-proto-gen-deps
	@echo "Generating Protobuf files"
	@find . -name '*.proto' -exec protoc \
		--go_out=paths=source_relative:. {} \;
.PHONY: proto-gen
