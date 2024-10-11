#? localnet-build-env: Run `make -C contrib/images simd-env`
localnet-build-env:
	$(MAKE) -C contrib/images simd-env
#? localnet-build-dlv: Run `make -C contrib/images simd-dlv`
localnet-build-dlv:
	$(MAKE) -C contrib/images simd-dlv
#? localnet-build-nodes: Start localnet node
localnet-build-nodes:
	$(DOCKER) run --rm -v $(CURDIR)/.testnets:/data cosmossdk/simd \
			  testnet init-files -n 4 -o /data --starting-ip-address 192.168.10.2 --keyring-backend=test --listen-ip-address 0.0.0.0
	docker compose up -d

#? localnet-stop: Stop localnet node
localnet-stop:
	docker compose down

# localnet-start will run a 4-node testnet locally. The nodes are
# based off the docker images in: ./contrib/images/simd-env
#? localnet-start: Run a 4-node testnet locally
localnet-start: localnet-stop localnet-build-env localnet-build-nodes

# localnet-debug will run a 4-node testnet locally in debug mode
# you can read more about the debug mode here: ./contrib/images/simd-dlv/README.md
#? localnet-debug: Run a 4-node testnet locally in debug mode
localnet-debug: localnet-stop localnet-build-dlv localnet-build-nodes

.PHONY: localnet-start localnet-stop localnet-debug localnet-build-env localnet-build-dlv localnet-build-nodes

#? help: Get more info on make commands.
help:
	@echo " Choose a command run in "$(PROJECT_NAME)":"
	@cat $(MAKEFILE_LIST) | sed -n 's/^#?//p' | column -t -s ':' |  sort | sed -e 's/^/ /'
.PHONY: help
