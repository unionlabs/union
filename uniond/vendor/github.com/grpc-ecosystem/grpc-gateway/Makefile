# This is a Makefile which maintains files automatically generated but to be
# shipped together with other files.
# You don't have to rebuild these targets by yourself unless you develop
# grpc-gateway itself.

GO_PLUGIN=bin/protoc-gen-go
GO_PROTOBUF_REPO=github.com/golang/protobuf
GO_PLUGIN_PKG=$(GO_PROTOBUF_REPO)/protoc-gen-go
GO_PTYPES_ANY_PKG=$(GO_PROTOBUF_REPO)/ptypes/any
SWAGGER_PLUGIN=bin/protoc-gen-swagger
SWAGGER_PLUGIN_SRC= utilities/doc.go \
		    utilities/pattern.go \
		    utilities/trie.go \
		    protoc-gen-swagger/genswagger/generator.go \
		    protoc-gen-swagger/genswagger/template.go \
		    protoc-gen-swagger/main.go
SWAGGER_PLUGIN_PKG=./protoc-gen-swagger
GATEWAY_PLUGIN=bin/protoc-gen-grpc-gateway
GATEWAY_PLUGIN_PKG=./protoc-gen-grpc-gateway
GATEWAY_PLUGIN_SRC= utilities/doc.go \
		    utilities/pattern.go \
		    utilities/trie.go \
		    protoc-gen-grpc-gateway \
		    protoc-gen-grpc-gateway/descriptor \
		    protoc-gen-grpc-gateway/descriptor/registry.go \
		    protoc-gen-grpc-gateway/descriptor/services.go \
		    protoc-gen-grpc-gateway/descriptor/types.go \
		    protoc-gen-grpc-gateway/descriptor/grpc_api_configuration.go \
		    protoc-gen-grpc-gateway/descriptor/grpc_api_service.go \
		    protoc-gen-grpc-gateway/generator \
		    protoc-gen-grpc-gateway/generator/generator.go \
		    protoc-gen-grpc-gateway/internal/gengateway \
		    protoc-gen-grpc-gateway/internal/gengateway/doc.go \
		    protoc-gen-grpc-gateway/internal/gengateway/generator.go \
		    protoc-gen-grpc-gateway/internal/gengateway/template.go \
		    protoc-gen-grpc-gateway/httprule \
		    protoc-gen-grpc-gateway/httprule/compile.go \
		    protoc-gen-grpc-gateway/httprule/parse.go \
		    protoc-gen-grpc-gateway/httprule/types.go \
		    protoc-gen-grpc-gateway/main.go
GATEWAY_PLUGIN_FLAGS?=
SWAGGER_PLUGIN_FLAGS?=

GOOGLEAPIS_DIR=third_party/googleapis
OUTPUT_DIR=_output

RUNTIME_PROTO=internal/errors.proto
RUNTIME_GO=$(RUNTIME_PROTO:.proto=.pb.go)

OPENAPIV2_PROTO=protoc-gen-swagger/options/openapiv2.proto protoc-gen-swagger/options/annotations.proto
OPENAPIV2_GO=$(OPENAPIV2_PROTO:.proto=.pb.go)

PKGMAP=Mgoogle/protobuf/field_mask.proto=google.golang.org/genproto/protobuf/field_mask,Mgoogle/protobuf/descriptor.proto=$(GO_PLUGIN_PKG)/descriptor,Mexamples/internal/proto/sub/message.proto=github.com/grpc-ecosystem/grpc-gateway/examples/internal/proto/sub
ADDITIONAL_GW_FLAGS=
ifneq "$(GATEWAY_PLUGIN_FLAGS)" ""
	ADDITIONAL_GW_FLAGS=,$(GATEWAY_PLUGIN_FLAGS)
endif
ADDITIONAL_SWG_FLAGS=
ifneq "$(SWAGGER_PLUGIN_FLAGS)" ""
	ADDITIONAL_SWG_FLAGS=,$(SWAGGER_PLUGIN_FLAGS)
endif
SWAGGER_EXAMPLES=examples/internal/proto/examplepb/echo_service.proto \
	 examples/internal/proto/examplepb/a_bit_of_everything.proto \
	 examples/internal/proto/examplepb/wrappers.proto \
	 examples/internal/proto/examplepb/stream.proto \
	 examples/internal/proto/examplepb/unannotated_echo_service.proto \
	 examples/internal/proto/examplepb/use_go_template.proto \
	 examples/internal/proto/examplepb/response_body_service.proto
SWAGGER_EXAMPLE_MERGE=examples/internal/proto/examplepb/swagger_merge_a.proto \
	 examples/internal/proto/examplepb/swagger_merge_b.proto

EXAMPLES=examples/internal/proto/examplepb/echo_service.proto \
	 examples/internal/proto/examplepb/a_bit_of_everything.proto \
	 examples/internal/proto/examplepb/stream.proto \
	 examples/internal/proto/examplepb/flow_combination.proto \
	 examples/internal/proto/examplepb/non_standard_names.proto \
	 examples/internal/proto/examplepb/wrappers.proto \
	 examples/internal/proto/examplepb/unannotated_echo_service.proto \
	 examples/internal/proto/examplepb/use_go_template.proto \
	 examples/internal/proto/examplepb/response_body_service.proto

GENERATE_UNBOUND_METHODS_EXAMPLE=examples/internal/proto/examplepb/generate_unbound_methods.proto

HELLOWORLD=examples/internal/helloworld/helloworld.proto

EXAMPLE_SVCSRCS=$(EXAMPLES:.proto=.pb.go)
EXAMPLE_GWSRCS=$(EXAMPLES:.proto=.pb.gw.go)
EXAMPLE_SWAGGERSRCS=$(SWAGGER_EXAMPLES:.proto=.swagger.json)
EXAMPLE_SWAGGER_MERGE=examples/internal/proto/examplepb/swagger_merge.swagger.json
EXAMPLE_DEPS=examples/internal/proto/pathenum/path_enum.proto examples/internal/proto/sub/message.proto examples/internal/proto/sub2/message.proto
EXAMPLE_DEPSRCS=$(EXAMPLE_DEPS:.proto=.pb.go)

GENERATE_UNBOUND_METHODS_EXAMPLE_SWAGGERSRCS=$(GENERATE_UNBOUND_METHODS_EXAMPLE:.proto=.swagger.json)
GENERATE_UNBOUND_METHODS_EXAMPLE_SVCSRCS=$(GENERATE_UNBOUND_METHODS_EXAMPLE:.proto=.pb.go)
GENERATE_UNBOUND_METHODS_EXAMPLE_GWSRCS=$(GENERATE_UNBOUND_METHODS_EXAMPLE:.proto=.pb.gw.go)

HELLOWORLD_SVCSRCS=$(HELLOWORLD:.proto=.pb.go)
HELLOWORLD_GWSRCS=$(HELLOWORLD:.proto=.pb.gw.go)

RUNTIME_TEST_PROTO=runtime/internal/examplepb/example.proto
RUNTIME_TEST_SRCS=$(RUNTIME_TEST_PROTO:.proto=.pb.go)

EXAMPLE_CLIENT_DIR=examples/internal/clients
ECHO_EXAMPLE_SPEC=examples/internal/proto/examplepb/echo_service.swagger.json
ECHO_EXAMPLE_SRCS=$(EXAMPLE_CLIENT_DIR)/echo/client.go \
		  $(EXAMPLE_CLIENT_DIR)/echo/response.go \
		  $(EXAMPLE_CLIENT_DIR)/echo/configuration.go \
		  $(EXAMPLE_CLIENT_DIR)/echo/api_echo_service.go \
		  $(EXAMPLE_CLIENT_DIR)/echo/model_examplepb_simple_message.go \
		  $(EXAMPLE_CLIENT_DIR)/echo/model_examplepb_embedded.go
ABE_EXAMPLE_SPEC=examples/internal/proto/examplepb/a_bit_of_everything.swagger.json
ABE_EXAMPLE_SRCS=$(EXAMPLE_CLIENT_DIR)/abe/model_a_bit_of_everything_nested.go \
		 $(EXAMPLE_CLIENT_DIR)/abe/api_a_bit_of_everything_service.go \
		 $(EXAMPLE_CLIENT_DIR)/abe/client.go \
		 $(EXAMPLE_CLIENT_DIR)/abe/api_camel_case_service_name.go \
		 $(EXAMPLE_CLIENT_DIR)/abe/configuration.go \
		 $(EXAMPLE_CLIENT_DIR)/abe/api_echo_rpc.go \
		 $(EXAMPLE_CLIENT_DIR)/abe/model_examplepb_a_bit_of_everything.go \
		 $(EXAMPLE_CLIENT_DIR)/abe/model_examplepb_a_bit_of_everything_repeated.go \
		 $(EXAMPLE_CLIENT_DIR)/abe/model_examplepb_body.go \
		 $(EXAMPLE_CLIENT_DIR)/abe/model_examplepb_numeric_enum.go \
		 $(EXAMPLE_CLIENT_DIR)/abe/model_examplepb_update_v2_request.go \
		 $(EXAMPLE_CLIENT_DIR)/abe/model_message_path_enum_nested_path_enum.go \
		 $(EXAMPLE_CLIENT_DIR)/abe/model_nested_deep_enum.go \
		 $(EXAMPLE_CLIENT_DIR)/abe/model_pathenum_path_enum.go \
		 $(EXAMPLE_CLIENT_DIR)/abe/model_protobuf_field_mask.go \
		 $(EXAMPLE_CLIENT_DIR)/abe/response.go
UNANNOTATED_ECHO_EXAMPLE_SPEC=examples/internal/proto/examplepb/unannotated_echo_service.swagger.json
UNANNOTATED_ECHO_EXAMPLE_SRCS=$(EXAMPLE_CLIENT_DIR)/unannotatedecho/client.go \
		 $(EXAMPLE_CLIENT_DIR)/unannotatedecho/response.go \
		 $(EXAMPLE_CLIENT_DIR)/unannotatedecho/configuration.go \
		 $(EXAMPLE_CLIENT_DIR)/unannotatedecho/model_examplepb_unannotated_simple_message.go \
		 $(EXAMPLE_CLIENT_DIR)/unannotatedecho/api_unannotated_echo_service.go
RESPONSE_BODY_EXAMPLE_SPEC=examples/internal/proto/examplepb/response_body_service.swagger.json
RESPONSE_BODY_EXAMPLE_SRCS=$(EXAMPLE_CLIENT_DIR)/responsebody/client.go \
		 $(EXAMPLE_CLIENT_DIR)/responsebody/response.go \
		 $(EXAMPLE_CLIENT_DIR)/responsebody/configuration.go \
		 $(EXAMPLE_CLIENT_DIR)/responsebody/model_examplepb_repeated_response_body_out.go \
		 $(EXAMPLE_CLIENT_DIR)/responsebody/model_examplepb_repeated_response_body_out_response.go \
		 $(EXAMPLE_CLIENT_DIR)/responsebody/model_examplepb_repeated_response_strings.go \
		 $(EXAMPLE_CLIENT_DIR)/responsebody/model_examplepb_response_body_out.go \
		 $(EXAMPLE_CLIENT_DIR)/responsebody/model_examplepb_response_body_out_response.go \
		 $(EXAMPLE_CLIENT_DIR)/responsebody/model_response_response_type.go \
		 $(EXAMPLE_CLIENT_DIR)/responsebody/api_response_body_service.go
GENERATE_UNBOUND_METHODS_EXAMPLE_SPEC=examples/internal/proto/examplepb/generate_unbound_methods.swagger.json
GENERATE_UNBOUND_METHODS_EXAMPLE_SRCS=$(EXAMPLE_CLIENT_DIR)/generateunboundmethods/client.go \
		 $(EXAMPLE_CLIENT_DIR)/generateunboundmethods/response.go \
		 $(EXAMPLE_CLIENT_DIR)/generateunboundmethods/configuration.go \
		 $(EXAMPLE_CLIENT_DIR)/generateunboundmethods/model_examplepb_generate_unbound_methods_simple_message.go \
		 $(EXAMPLE_CLIENT_DIR)/generateunboundmethods/api_generate_unbound_methods.go

EXAMPLE_CLIENT_SRCS=$(ECHO_EXAMPLE_SRCS) $(ABE_EXAMPLE_SRCS) $(UNANNOTATED_ECHO_EXAMPLE_SRCS) $(RESPONSE_BODY_EXAMPLE_SRCS) $(GENERATE_UNBOUND_METHODS_EXAMPLE_SRCS)
SWAGGER_CODEGEN=swagger-codegen

PROTOC_INC_PATH=$(dir $(shell which protoc))/../include

generate: $(RUNTIME_GO)

.SUFFIXES: .go .proto

$(GO_PLUGIN):
	go build -o $(GO_PLUGIN) $(GO_PLUGIN_PKG)

$(RUNTIME_GO): $(RUNTIME_PROTO) $(GO_PLUGIN)
	protoc -I $(PROTOC_INC_PATH) --plugin=$(GO_PLUGIN) -I. --go_out=$(PKGMAP),paths=source_relative:. $(RUNTIME_PROTO)

$(OPENAPIV2_GO): $(OPENAPIV2_PROTO) $(GO_PLUGIN)
	protoc -I $(PROTOC_INC_PATH) --plugin=$(GO_PLUGIN) -I. --go_out=$(PKGMAP),paths=source_relative:. $(OPENAPIV2_PROTO)

$(GATEWAY_PLUGIN): $(RUNTIME_GO) $(GATEWAY_PLUGIN_SRC)
	go build -o $@ $(GATEWAY_PLUGIN_PKG)

$(SWAGGER_PLUGIN): $(SWAGGER_PLUGIN_SRC) $(OPENAPIV2_GO)
	go build -o $@ $(SWAGGER_PLUGIN_PKG)

$(EXAMPLE_SVCSRCS): $(GO_PLUGIN) $(EXAMPLES)
	protoc -I $(PROTOC_INC_PATH) -I. -I$(GOOGLEAPIS_DIR) --plugin=$(GO_PLUGIN) --go_out=$(PKGMAP),plugins=grpc,paths=source_relative:. $(EXAMPLES)
$(EXAMPLE_DEPSRCS): $(GO_PLUGIN) $(EXAMPLE_DEPS)
	protoc -I $(PROTOC_INC_PATH) -I. --plugin=$(GO_PLUGIN) --go_out=$(PKGMAP),plugins=grpc,paths=source_relative:. $(@:.pb.go=.proto)

$(RUNTIME_TEST_SRCS): $(GO_PLUGIN) $(RUNTIME_TEST_PROTO)
	protoc -I $(PROTOC_INC_PATH) -I. -I$(GOOGLEAPIS_DIR) --plugin=$(GO_PLUGIN) --go_out=$(PKGMAP),plugins=grpc,paths=source_relative:. $(RUNTIME_TEST_PROTO)

$(EXAMPLE_GWSRCS): ADDITIONAL_GW_FLAGS:=$(ADDITIONAL_GW_FLAGS),grpc_api_configuration=examples/internal/proto/examplepb/unannotated_echo_service.yaml
$(EXAMPLE_GWSRCS): $(GATEWAY_PLUGIN) $(EXAMPLES)
	protoc -I $(PROTOC_INC_PATH) -I. -I$(GOOGLEAPIS_DIR) --plugin=$(GATEWAY_PLUGIN) --grpc-gateway_out=logtostderr=true,allow_repeated_fields_in_body=true,$(PKGMAP)$(ADDITIONAL_GW_FLAGS):. $(EXAMPLES)

$(EXAMPLE_SWAGGERSRCS): ADDITIONAL_SWG_FLAGS:=$(ADDITIONAL_SWG_FLAGS),grpc_api_configuration=examples/internal/proto/examplepb/unannotated_echo_service.yaml
$(EXAMPLE_SWAGGERSRCS): $(SWAGGER_PLUGIN) $(SWAGGER_EXAMPLES)
	protoc -I $(PROTOC_INC_PATH) -I. -I$(GOOGLEAPIS_DIR) --plugin=$(SWAGGER_PLUGIN) --swagger_out=logtostderr=true,allow_repeated_fields_in_body=true,use_go_templates=true,$(PKGMAP)$(ADDITIONAL_SWG_FLAGS):. $(SWAGGER_EXAMPLES)

$(EXAMPLE_SWAGGER_MERGE): $(SWAGGER_PLUGIN) $(SWAGGER_EXAMPLE_MERGE)
	protoc -I $(PROTOC_INC_PATH) -I. -I$(GOOGLEAPIS_DIR) --plugin=$(SWAGGER_PLUGIN) --swagger_out=logtostderr=true,allow_repeated_fields_in_body=true,use_go_templates=true,allow_merge=true,merge_file_name=$(EXAMPLE_SWAGGER_MERGE:.swagger.json=),$(PKGMAP):. $(SWAGGER_EXAMPLE_MERGE)

$(GENERATE_UNBOUND_METHODS_EXAMPLE_GWSRCS): $(GATEWAY_PLUGIN) $(GENERATE_UNBOUND_METHODS_EXAMPLE)
	protoc -I $(PROTOC_INC_PATH) -I. -I$(GOOGLEAPIS_DIR) --plugin=$(GATEWAY_PLUGIN) --grpc-gateway_out=logtostderr=true,allow_repeated_fields_in_body=true,generate_unbound_methods=true,$(PKGMAP)$(ADDITIONAL_GW_FLAGS):. $(GENERATE_UNBOUND_METHODS_EXAMPLE)
$(GENERATE_UNBOUND_METHODS_EXAMPLE_SVCSRCS): $(GO_PLUGIN) $(GENERATE_UNBOUND_METHODS_EXAMPLE)
	protoc -I $(PROTOC_INC_PATH) -I. -I$(GOOGLEAPIS_DIR) --plugin=$(GO_PLUGIN) --go_out=$(PKGMAP),plugins=grpc,paths=source_relative:. $(GENERATE_UNBOUND_METHODS_EXAMPLE)
$(GENERATE_UNBOUND_METHODS_EXAMPLE_SWAGGERSRCS): $(SWAGGER_PLUGIN) $(GENERATE_UNBOUND_METHODS_EXAMPLE)
	protoc -I $(PROTOC_INC_PATH) -I. -I$(GOOGLEAPIS_DIR) --plugin=$(SWAGGER_PLUGIN) --swagger_out=logtostderr=true,allow_repeated_fields_in_body=true,use_go_templates=true,generate_unbound_methods=true,$(PKGMAP):. $(GENERATE_UNBOUND_METHODS_EXAMPLE)

$(HELLOWORLD_SVCSRCS): $(GO_PLUGIN) $(HELLOWORLD)
	protoc -I $(PROTOC_INC_PATH) -I. -I$(GOOGLEAPIS_DIR) --plugin=$(GO_PLUGIN) --go_out=$(PKGMAP),plugins=grpc,paths=source_relative:. $(HELLOWORLD)

$(HELLOWORLD_GWSRCS):
$(HELLOWORLD_GWSRCS): $(GATEWAY_PLUGIN) $(HELLOWORLD)
	protoc -I $(PROTOC_INC_PATH) -I. -I$(GOOGLEAPIS_DIR) --plugin=$(GATEWAY_PLUGIN) --grpc-gateway_out=logtostderr=true,allow_repeated_fields_in_body=true,$(PKGMAP)$(ADDITIONAL_GW_FLAGS):. $(HELLOWORLD)


$(ECHO_EXAMPLE_SRCS): $(ECHO_EXAMPLE_SPEC)
	$(SWAGGER_CODEGEN) generate -i $(ECHO_EXAMPLE_SPEC) \
	    -l go -o examples/internal/clients/echo --additional-properties packageName=echo
	@rm -f $(EXAMPLE_CLIENT_DIR)/echo/README.md \
		$(EXAMPLE_CLIENT_DIR)/echo/git_push.sh
$(ABE_EXAMPLE_SRCS): $(ABE_EXAMPLE_SPEC)
	$(SWAGGER_CODEGEN) generate -i $(ABE_EXAMPLE_SPEC) \
	    -l go -o examples/internal/clients/abe --additional-properties packageName=abe
	@rm -f $(EXAMPLE_CLIENT_DIR)/abe/README.md \
		$(EXAMPLE_CLIENT_DIR)/abe/git_push.sh
$(UNANNOTATED_ECHO_EXAMPLE_SRCS): $(UNANNOTATED_ECHO_EXAMPLE_SPEC)
	$(SWAGGER_CODEGEN) generate -i $(UNANNOTATED_ECHO_EXAMPLE_SPEC) \
	    -l go -o examples/internal/clients/unannotatedecho --additional-properties packageName=unannotatedecho
	@rm -f $(EXAMPLE_CLIENT_DIR)/unannotatedecho/README.md \
		$(EXAMPLE_CLIENT_DIR)/unannotatedecho/git_push.sh
$(RESPONSE_BODY_EXAMPLE_SRCS): $(RESPONSE_BODY_EXAMPLE_SPEC)
	$(SWAGGER_CODEGEN) generate -i $(RESPONSE_BODY_EXAMPLE_SPEC) \
	    -l go -o examples/internal/clients/responsebody --additional-properties packageName=responsebody
	@rm -f $(EXAMPLE_CLIENT_DIR)/responsebody/README.md \
		$(EXAMPLE_CLIENT_DIR)/responsebody/git_push.sh
$(GENERATE_UNBOUND_METHODS_EXAMPLE_SRCS): $(GENERATE_UNBOUND_METHODS_EXAMPLE_SPEC)
	$(SWAGGER_CODEGEN) generate -i $(GENERATE_UNBOUND_METHODS_EXAMPLE_SPEC) \
	    -l go -o examples/internal/clients/generateunboundmethods --additional-properties packageName=generateunboundmethods
	@rm -f $(EXAMPLE_CLIENT_DIR)/generateunboundmethods/README.md \
		$(EXAMPLE_CLIENT_DIR)/generateunboundmethods/git_push.sh

examples: $(EXAMPLE_DEPSRCS) $(EXAMPLE_SVCSRCS) $(EXAMPLE_GWSRCS) $(EXAMPLE_SWAGGERSRCS) $(EXAMPLE_SWAGGER_MERGE) $(EXAMPLE_CLIENT_SRCS) $(HELLOWORLD_SVCSRCS) $(HELLOWORLD_GWSRCS) $(GENERATE_UNBOUND_METHODS_EXAMPLE_GWSRCS) $(GENERATE_UNBOUND_METHODS_EXAMPLE_SVCSRCS) $(GENERATE_UNBOUND_METHODS_EXAMPLE_SWAGGERSRCS)
testproto: $(RUNTIME_TEST_SRCS)
test: examples testproto
	go test -short -race ./...
	go test -race ./examples/internal/integration -args -network=unix -endpoint=test.sock
changelog:
	docker run --rm \
		--interactive \
		--tty \
		-e "CHANGELOG_GITHUB_TOKEN=${CHANGELOG_GITHUB_TOKEN}" \
		-v "$(PWD):/usr/local/src/your-app" \
		ferrarimarco/github-changelog-generator:1.14.3 \
				-u grpc-ecosystem \
				-p grpc-gateway \
				--author \
				--compare-link \
				--github-site=https://github.com \
				--unreleased-label "**Next release**" \
				--release-branch=v1 \
				--future-release=v1.16.0
lint:
	golint --set_exit_status ./runtime
	golint --set_exit_status ./utilities/...
	golint --set_exit_status ./protoc-gen-grpc-gateway/...
	golint --set_exit_status ./protoc-gen-swagger/...
	go vet ./runtime || true
	go vet ./utilities/...
	go vet ./protoc-gen-grpc-gateway/...
	go vet ./protoc-gen-swagger/...

clean:
	rm -f $(GATEWAY_PLUGIN) $(SWAGGER_PLUGIN)
distclean: clean
	rm -f $(GO_PLUGIN)
realclean: distclean
	rm -f $(EXAMPLE_SVCSRCS) $(EXAMPLE_DEPSRCS)
	rm -f $(EXAMPLE_GWSRCS)
	rm -f $(EXAMPLE_SWAGGERSRCS)
	rm -f $(EXAMPLE_CLIENT_SRCS)
	rm -f $(GENERATE_UNBOUND_METHODS_EXAMPLE_GWSRCS)
	rm -f $(GENERATE_UNBOUND_METHODS_EXAMPLE_SVCSRCS)
	rm -f $(GENERATE_UNBOUND_METHODS_EXAMPLE_SWAGGERSRCS)
	rm -f $(HELLOWORLD_SVCSRCS)
	rm -f $(HELLOWORLD_GWSRCS)
	rm -f $(OPENAPIV2_GO)
	rm -f $(RUNTIME_TEST_SRCS)

.PHONY: generate examples test lint clean distclean realclean
