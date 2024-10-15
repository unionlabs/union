APP ?= "./cmd/qrterminal"

build:
	@go build "$(APP)"

release:
	./.goreleaser release --rm-dist

reltest:
	./.goreleaser release --snapshot --rm-dist --skip-publish

test:
	@go test -cover
