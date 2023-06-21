
.PHONY:
build-spec-tests:
	go run github.com/prysmaticlabs/fastssz/sszgen --path ./spectests/structs.go --include ./spectests/external,./spectests/external2

build-spec-tests-tree:
	go run github.com/prysmaticlabs/fastssz/sszgen --path ./spectests/structs.go --objs AttestationData --experimental

.PHONY:
get-spec-tests:
	./scripts/download-spec-tests.sh v1.1.0-alpha.4-pre2
