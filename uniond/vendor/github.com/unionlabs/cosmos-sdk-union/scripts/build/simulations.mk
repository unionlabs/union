
#? test-sim-nondeterminism: Run non-determinism test for simapp
test-sim-nondeterminism:
	@echo "Running non-determinism test..."
	@cd ${CURRENT_DIR}/simapp && go test -failfast -mod=readonly -timeout=30m -tags='sims' -run TestAppStateDeterminism \
		-NumBlocks=100 -BlockSize=200 -Period=0

# Requires an exported plugin. See store/streaming/README.md for documentation.
#
# example:
#   export COSMOS_SDK_ABCI_V1=<path-to-plugin-binary>
#   make test-sim-nondeterminism-streaming
#
# Using the built-in examples:
#   export COSMOS_SDK_ABCI_V1=<path-to-sdk>/store/streaming/abci/examples/file/file
#   make test-sim-nondeterminism-streaming
test-sim-nondeterminism-streaming:
	@echo "Running non-determinism-streaming test..."
	@cd ${CURRENT_DIR}/simapp && go test -failfast -mod=readonly -timeout=30m -tags='sims' -run TestAppStateDeterminism \
		-NumBlocks=100 -BlockSize=200 -Period=0 -EnableStreaming=true

test-sim-custom-genesis-fast:
	@echo "Running custom genesis simulation..."
	@echo "By default, ${HOME}/.simapp/config/genesis.json will be used."
	@cd ${CURRENT_DIR}/simapp && go test -failfast -mod=readonly -timeout=30m -tags='sims' -run TestFullAppSimulation -Genesis=${HOME}/.simapp/config/genesis.json \
		-NumBlocks=100 -BlockSize=200 -Seed=99 -Period=5 -SigverifyTx=false

test-sim-import-export:
	@echo "Running application import/export simulation. This may take several minutes..."
	@cd ${CURRENT_DIR}/simapp && go test -failfast -mod=readonly -timeout 20m -tags='sims' -run TestAppImportExport \
		-NumBlocks=50 -Period=5

test-sim-after-import:
	@echo "Running application simulation-after-import. This may take several minutes..."
	@cd ${CURRENT_DIR}/simapp && go test -failfast -mod=readonly -timeout 30m -tags='sims' -run TestAppSimulationAfterImport \
		-NumBlocks=50 -Period=5


test-sim-custom-genesis-multi-seed:
	@echo "Running multi-seed custom genesis simulation..."
	@echo "By default, ${HOME}/.simapp/config/genesis.json will be used."
	@cd ${CURRENT_DIR}/simapp && go test -failfast -mod=readonly -timeout 30m -tags='sims' -run TestFullAppSimulation -Genesis=${HOME}/.simapp/config/genesis.json \
		-NumBlocks=400 -Period=5

test-sim-multi-seed-long:
	@echo "Running long multi-seed application simulation. This may take awhile!"
	@cd ${CURRENT_DIR}/simapp && go test -failfast -mod=readonly -timeout=2h -tags='sims' -run TestFullAppSimulation \
		-NumBlocks=150 -Period=50

test-sim-multi-seed-short:
	@echo "Running short multi-seed application simulation. This may take awhile!"
	@cd ${CURRENT_DIR}/simapp && go test -failfast -mod=readonly -timeout 30m -tags='sims' -run TestFullAppSimulation \
		-NumBlocks=50 -Period=10 -FauxMerkle=true


test-sim-benchmark-invariants:
	@echo "Running simulation invariant benchmarks..."
	cd ${CURRENT_DIR}/simapp && go test -failfast -mod=readonly -benchmem -bench=BenchmarkInvariants -tags='sims' -run=^$ \
	-Enabled=true -NumBlocks=1000 -BlockSize=200 \
	-Period=1 -Commit=true -Seed=57 -v -timeout 24h

.PHONY: \
test-sim-nondeterminism \
test-sim-nondeterminism-streaming \
test-sim-custom-genesis-fast \
test-sim-import-export \
test-sim-after-import \
test-sim-custom-genesis-multi-seed \
test-sim-multi-seed-short \
test-sim-multi-seed-long \
test-sim-benchmark-invariants

SIM_NUM_BLOCKS ?= 500
SIM_BLOCK_SIZE ?= 200
SIM_COMMIT ?= true

#? test-sim-fuzz: Run fuzz test for simapp
test-sim-fuzz:
	@echo "Running application fuzz for numBlocks=2, blockSize=20. This may take awhile!"
#ld flags are a quick fix to make it work on current osx
	@cd ${CURRENT_DIR}/simapp && go test -failfast -mod=readonly -json -tags='sims' -ldflags="-extldflags=-Wl,-ld_classic" -timeout=60m -fuzztime=60m -run=^$$ -fuzz=FuzzFullAppSimulation -GenesisTime=1714720615 -NumBlocks=2 -BlockSize=20

#? test-sim-benchmark: Run benchmark test for simapp
test-sim-benchmark:
	@echo "Running application benchmark for numBlocks=$(SIM_NUM_BLOCKS), blockSize=$(SIM_BLOCK_SIZE). This may take awhile!"
	@cd ${CURRENT_DIR}/simapp && go test -failfast -mod=readonly -tags='sims' -run=^$$ $(.) -bench ^BenchmarkFullAppSimulation$$  \
		-Enabled=true -NumBlocks=$(SIM_NUM_BLOCKS) -BlockSize=$(SIM_BLOCK_SIZE) -Commit=$(SIM_COMMIT) -timeout 24h


test-sim-profile:
	@echo "Running application benchmark for numBlocks=$(SIM_NUM_BLOCKS), blockSize=$(SIM_BLOCK_SIZE). This may take awhile!"
	@cd ${CURRENT_DIR}/simapp && go test -failfast -mod=readonly -tags='sims' -benchmem -run=^$$ $(.) -bench ^BenchmarkFullAppSimulation$$ \
		-Enabled=true -NumBlocks=$(SIM_NUM_BLOCKS) -BlockSize=$(SIM_BLOCK_SIZE) -Commit=$(SIM_COMMIT) -timeout 24h -cpuprofile cpu.out -memprofile mem.out

.PHONY: test-sim-profile test-sim-benchmark test-sim-fuzz

#? benchmark: Run benchmark tests
benchmark:
	@go test -failfast -mod=readonly -bench=. $(PACKAGES_NOSIMULATION)
.PHONY: benchmark
