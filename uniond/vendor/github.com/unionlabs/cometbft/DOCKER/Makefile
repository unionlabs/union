build:
	@sh -c "'$(CURDIR)/build.sh'"

push:
	@sh -c "'$(CURDIR)/push.sh'"

build_testing:
	docker build --tag cometbft/testing -f ./Dockerfile.testing .

.PHONY: build push build_testing
