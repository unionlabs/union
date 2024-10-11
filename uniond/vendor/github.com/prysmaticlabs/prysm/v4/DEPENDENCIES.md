# Dependency Management in Prysm

Prysm is go project with many complicated dependencies, including some c++ based libraries. There
are two parts to Prysm's dependency management. Go modules and bazel managed dependencies. Be sure 
to read [Why Bazel?](https://github.com/prysmaticlabs/documentation/issues/138) to fully
understand the reasoning behind an additional layer of build tooling via Bazel rather than a pure
"go build" project.

## Go Module support

The Prysm project officially supports go modules with some caveats. 

### Caveat 1: Some c++ libraries are precompiled archives

Given some of Prysm's c++ dependencies have very complicated project structures which make building
difficult or impossible with "go build" alone. Additionally, building c++ dependencies with certain
compilers, like clang / LLVM, offer a significant performance improvement. To get around this 
issue, c++ dependencies have been precompiled as linkable archives. While there isn't necessarily
anything bad about precompiled archives, these files are a "blackbox" which a 3rd party author
could have compiled anything for this archive and detecting undesired behavior would be nearly
impossible. If your risk tolerance is low, always compile everything from source yourself, 
including complicated c++ dependencies.

*Recommendation: Use go build only for local development and use bazel build for production.*

### Caveat 2: Generated gRPC protobuf libraries

One key advantage of Bazel over vanilla `go build` is that Bazel automatically (re)builds generated
pb.go files at build time when file changes are present in any protobuf definition file or after
any updates to the protobuf compiler or other relevant dependencies. Vanilla go users should run
the following scripts often to ensure their generated files are up to date. Furthermore, Prysm
generates SSZ marshal related code based on defined data structures. These generated files must
also be updated and checked in as frequently.

```bash
./hack/update-go-pbs.sh
./hack/update-go-ssz.sh
```

*Recommendation: Use go build only for local development and use bazel build for production.*

### Caveat 3: Compile-time optimizations 

When Prysmatic Labs builds production binaries, they use the "release" configuration of bazel to
compile with several compiler optimizations and recommended production build configurations.
Additionally, the release build properly stamps the built binaries to include helpful metadata
about how and when the binary was built. 

*Recommendation: Use go build only for local development and use bazel build for production.*

```bash
bazel build //beacon-chain --config=release
```
 
## Adding / updating dependencies

1. Add your dependency as you would with go modules. I.e. `go get ...`
1. Run `gazelle update-repos -from_file=go.mod` to update the bazel managed dependencies.

Example:

```bash
go get github.com/prysmaticlabs/example@v1.2.3
bazel run //:gazelle -- update-repos -from_file=go.mod -to_macro=deps.bzl%prysm_deps -prune=true
```

The deps.bzl file should have been updated with the dependency and any transitive dependencies. 

Do NOT add new `go_repository` to the WORKSPACE file. All dependencies should live in deps.bzl.

## Running tests

To enable conditional compilation and custom configuration for tests (where compiled code has more 
debug info, while not being completely optimized), we rely on Go's build tags/constraints mechanism 
(see official docs on [build constraints](https://golang.org/pkg/go/build/#hdr-Build_Constraints)). 
Therefore, whenever using `go test`, do not forget to pass in extra build tag, eg:

```bash
go test ./beacon-chain/sync/initial-sync -tags develop 
```
