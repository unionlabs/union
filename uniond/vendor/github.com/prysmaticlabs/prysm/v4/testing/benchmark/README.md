# Benchmarks for Prysm State Transition
This package contains the functionality needed for benchmarking Prysm state transitions, this includes its block processing (with and without caching) and epoch processing functions. There is also a benchmark for HashTreeRoot on a large beacon state.

## Benchmark Configuration
The following configs are in `config.go`:
* `ValidatorCount`: Sets the amount of active validators to perform the benchmarks with. Default is 16384.
* `AttestationsPerEpoch`: Sets the amount of attestations per epoch for the benchmarks to perform with, this affects the amount of attestations in a full block and the amount of attestations per epoch in the state for the `ProcessEpoch` and `HashTreeRoot` benchmark. Default is 128.

## Generating new SSZ files
Due to the sheer size of the benchmarking configurations (16384 validators), the files used for benchmarking are pregenerated so there's no wasted computations on generating a genesis state with 16384 validators. This should only be needed if there is a breaking spec change and the tests fail from SSZ issues.

To generate new files to use for benchmarking, run the below command in the root of Prysm.

```
bazel run //tools/benchmark-files-gen -- --output-dir $PRYSMPATH/testing/benchmark/benchmark_files/ --overwrite
```

## Running the benchmarks
To run the ExecuteStateTransition benchmark:

```bazel test //beacon-chain/core/state:go_default_test --test_filter=BenchmarkExecuteStateTransition_FullBlock --test_arg=-test.bench=BenchmarkExecuteStateTransition_FullBlock```

To run the ExecuteStateTransition (with cache) benchmark:

```bazel test //beacon-chain/core/state:go_default_test --test_filter=BenchmarkExecuteStateTransition_WithCache --test_arg=-test.bench=BenchmarkExecuteStateTransition_WithCache```

To run the ProcessEpoch benchmark:

```bazel test //beacon-chain/core/state:go_default_test --test_filter=BenchmarkProcessEpoch_2FullEpochs --test_arg=-test.bench=BenchmarkProcessEpoch_2FullEpochs```

To run the HashTreeRoot benchmark:

```bazel test //beacon-chain/core/state:go_default_test --test_filter=BenchmarkHashTreeRoot_FullState --test_arg=-test.bench=BenchmarkHashTreeRoot_FullState```

To run the HashTreeRootState benchmark:

```bazel test //beacon-chain/core/state:go_default_test --test_filter=BenchmarkHashTreeRootState_FullState --test_arg=-test.bench=BenchmarkHashTreeRootState_FullState```

Extra flags needed to benchmark properly:

```--nocache_test_results --test_arg=-test.v --test_timeout=2000 --test_arg=-test.cpuprofile=/tmp/cpu.profile --test_arg=-test.memprofile=/tmp/mem.profile --test_output=streamed```

## Current Results as of January 2020
```
BenchmarkExecuteStateTransition_FullBlock-4           20	  2031438030 ns/op
BenchmarkExecuteStateTransition_WithCache-4   	      20	  1857290454 ns/op
BenchmarkHashTreeRoot_FullState-4   	              50	   297655834 ns/op
BenchmarkHashTreeRootState_FullState-4                50           155535883 ns/op
```