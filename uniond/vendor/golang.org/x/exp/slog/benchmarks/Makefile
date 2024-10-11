# Run and compare benchmarks.
# This requires a version of benchstat that supports
# the -ignore flag. The flag was added on or before 13 January 2023,
# so a compatible version can be obtained by running
#    go install golang.org/x/perf/cmd/benchstat@latest

count = 10

default: compare-zap compare-zerolog

compare-%: %_benchmarks/out.bench slog.bench
	benchstat -ignore pkg $^

slog.bench: *.go ../*.go ../../go.mod
	go test -run NONE -bench . -count $(count) > $@

slog-nopc.bench: *.go ../*.go ../../go.mod
	go test -nopc -run NONE -bench . -count $(count) > $@

%_benchmarks/out.bench: %_benchmarks/*.go %_benchmarks/go.mod
	go test -C $*_benchmarks  -bench . -count $(count) > $@

# Don't delete the out.bench files after a comparison.
.PRECIOUS: %_benchmarks/out.bench

