DIRECTORIES_TO_BUILD := "./testpb ./internal/testprotos/test3"

pulsar:
	docker build -t dev:proto-build -f Dockerfile .
	docker run -v "$(CURDIR):/genproto" -w /genproto dev:proto-build ./scripts/fastreflect.sh "$(DIRECTORIES_TO_BUILD)"

proto-gen:
	(cd proto; buf generate)
	mv proto/cosmos_proto/cosmos.pb.go .

.PHONY: proto_gen pulsar