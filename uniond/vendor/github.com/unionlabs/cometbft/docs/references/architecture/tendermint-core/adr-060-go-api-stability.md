# ADR 060: Go API Stability

## Changelog

- 2020-09-08: Initial version. (@erikgrinaker)

- 2020-09-09: Tweak accepted changes, add initial public API packages, add consequences. (@erikgrinaker)

- 2020-09-17: Clarify initial public API. (@erikgrinaker)

## Context

With the release of Tendermint 1.0 we will adopt [semantic versioning](https://semver.org). One major implication is a guarantee that we will not make backwards-incompatible changes until Tendermint 2.0 (except in pre-release versions). In order to provide this guarantee for our Go API, we must clearly define which of our APIs are public, and what changes are considered backwards-compatible.

Currently, we list packages that we consider public in our [README](https://github.com/tendermint/tendermint#versioning), but since we are still at version 0.x we do not provide any backwards compatibility guarantees at all.

### Glossary

* **External project:** a different Git/VCS repository or code base.

* **External package:** a different Go package, can be a child or sibling package in the same project.

* **Internal code:** code not intended for use in external projects.

* **Internal directory:** code under `internal/` which cannot be imported in external projects.

* **Exported:** a Go identifier starting with an uppercase letter, which can therefore be accessed by an external package.

* **Private:** a Go identifier starting with a lowercase letter, which therefore cannot be accessed by an external package unless via an exported field, variable, or function/method return value.

* **Public API:** any Go identifier that can be imported or accessed by an external project, except test code in `_test.go` files.

* **Private API:** any Go identifier that is not accessible via a public API, including all code in the internal directory.

## Alternative Approaches

- Split all public APIs out to separate Go modules in separate Git repositories, and consider all Tendermint code internal and not subject to API backwards compatibility at all. This was rejected, since it has been attempted by the Tendermint project earlier, resulting in too much dependency management overhead.

- Simply document which APIs are public and which are private. This is the current approach, but users should not be expected to self-enforce this, the documentation is not always up-to-date, and external projects will often end up depending on internal code anyway.

## Decision

From Tendermint 1.0, all internal code (except private APIs) will be placed in a root-level [`internal` directory](https://golang.org/cmd/go/#hdr-Internal_Directories), which the Go compiler will block for use by external projects. All exported items outside of the `internal` directory are considered a public API and subject to backwards compatibility guarantees, except files ending in `_test.go`.

The `crypto` package may be split out to a separate module in a separate repo. This is the main general-purpose package used by external projects, and is the only Tendermint dependency in e.g. IAVL which can cause some problems for projects depending on both IAVL and Tendermint. This will be decided after further discussion.

The `tm-db` package will remain a separate module in a separate repo. The `crypto` package may possibly be split out, pending further discussion, as this is the main general-purpose package used by other projects.

## Detailed Design

### Public API

When preparing our public API for 1.0, we should keep these principles in mind:

- Limit the number of public APIs that we start out with - we can always add new APIs later, but we can't change or remove APIs once they're made public.

- Before an API is made public, do a thorough review of the API to make sure it covers any future needs, can accommodate expected changes, and follows good API design practices.

The following is the minimum set of public APIs that will be included in 1.0, in some form:

- `abci`
- packages used for constructing nodes `config`, `libs/log`, and `version`
- Client APIs, i.e. `rpc/client`, `light`, and `privval`.
- `crypto` (possibly as a separate repo)

We may offer additional APIs as well, following further discussions internally and with other stakeholders. However, public APIs for providing custom components (e.g. reactors and mempools) are not planned for 1.0, but may be added in a later 1.x version if this is something we want to offer.

For comparison, the following are the number of Tendermint imports in the Cosmos SDK (excluding tests), which should be mostly satisfied by the planned APIs.

```
      1 github.com/tendermint/tendermint/abci/server
     73 github.com/tendermint/tendermint/abci/types
      2 github.com/tendermint/tendermint/cmd/tendermint/commands
      7 github.com/tendermint/tendermint/config
     68 github.com/tendermint/tendermint/crypto
      1 github.com/tendermint/tendermint/crypto/armor
     10 github.com/tendermint/tendermint/crypto/ed25519
      2 github.com/tendermint/tendermint/crypto/encoding
      3 github.com/tendermint/tendermint/crypto/merkle
      3 github.com/tendermint/tendermint/crypto/sr25519
      8 github.com/tendermint/tendermint/crypto/tmhash
      1 github.com/tendermint/tendermint/crypto/xsalsa20symmetric
     11 github.com/tendermint/tendermint/libs/bytes
      2 github.com/tendermint/tendermint/libs/bytes.HexBytes
     15 github.com/tendermint/tendermint/libs/cli
      2 github.com/tendermint/tendermint/libs/cli/flags
      2 github.com/tendermint/tendermint/libs/json
     30 github.com/tendermint/tendermint/libs/log
      1 github.com/tendermint/tendermint/libs/math
     11 github.com/tendermint/tendermint/libs/os
      4 github.com/tendermint/tendermint/libs/rand
      1 github.com/tendermint/tendermint/libs/strings
      5 github.com/tendermint/tendermint/light
      1 github.com/tendermint/tendermint/internal/mempool
      3 github.com/tendermint/tendermint/node
      5 github.com/tendermint/tendermint/internal/p2p
      4 github.com/tendermint/tendermint/privval
     10 github.com/tendermint/tendermint/proto/tendermint/crypto
      1 github.com/tendermint/tendermint/proto/tendermint/libs/bits
     24 github.com/tendermint/tendermint/proto/tendermint/types
      3 github.com/tendermint/tendermint/proto/tendermint/version
      2 github.com/tendermint/tendermint/proxy
      3 github.com/tendermint/tendermint/rpc/client
      1 github.com/tendermint/tendermint/rpc/client/http
      2 github.com/tendermint/tendermint/rpc/client/local
      3 github.com/tendermint/tendermint/rpc/core/types
      1 github.com/tendermint/tendermint/rpc/jsonrpc/server
     33 github.com/tendermint/tendermint/types
      2 github.com/tendermint/tendermint/types/time
      1 github.com/tendermint/tendermint/version
```

### Backwards-Compatible Changes

In Go, [almost all API changes are backwards-incompatible](https://blog.golang.org/module-compatibility) and thus exported items in public APIs generally cannot be changed until Tendermint 2.0. The only backwards-compatible changes we can make to public APIs are:

- Adding a package.

- Adding a new identifier to the package scope (e.g. const, var, func, struct, interface, etc.).

- Adding a new method to a struct.

- Adding a new field to a struct, if the zero-value preserves any old behavior.

- Changing the order of fields in a struct.

- Adding a variadic parameter to a named function or struct method, if the function type itself is not assignable in any public APIs (e.g. a callback).

- Adding a new method to an interface, or a variadic parameter to an interface method, _if the interface already has a private method_ (which prevents external packages from implementing it).

- Widening a numeric type as long as it is a named type (e.g. `type Number int32` can change to `int64`, but not `int8` or `uint32`).

Note that public APIs can expose private types (e.g. via an exported variable, field, or function/method return value), in which case the exported fields and methods on these private types are also part of the public API and covered by its backwards compatibility guarantees. In general, private types should never be accessible via public APIs unless wrapped in an exported interface.

Also note that if we accept, return, export, or embed types from a dependency, we assume the backwards compatibility responsibility for that dependency, and must make sure any dependency upgrades comply with the above constraints.

We should run CI linters for minor version branches to enforce this, e.g. [apidiff](https://go.googlesource.com/exp/+/refs/heads/master/apidiff/README.md), [breakcheck](https://github.com/gbbr/breakcheck), and [apicombat](https://github.com/bradleyfalzon/apicompat).

#### Accepted Breakage

The above changes can still break programs in a few ways - these are _not_ considered backwards-incompatible changes, and users are advised to avoid this usage:

- If a program uses unkeyed struct literals (e.g. `Foo{"bar", "baz"}`) and we add fields or change the field order, the program will no longer compile or may have logic errors.

- If a program embeds two structs in a struct, and we add a new field or method to an embedded Tendermint struct which also exists in the other embedded struct, the program will no longer compile.

- If a program compares two structs (e.g. with `==`), and we add a new field of an incomparable type (slice, map, func, or struct that contains these) to a Tendermint struct which is compared, the program will no longer compile.

- If a program assigns a Tendermint function to an identifier, and we add a variadic parameter to the function signature, the program will no longer compile.

### Strategies for API Evolution

The API guarantees above can be fairly constraining, but are unavoidable given the Go language design. The following tricks can be employed where appropriate to allow us to make changes to the API:

- We can add a new function or method with a different name that takes additional parameters, and have the old function call the new one.

- Functions and methods can take an options struct instead of separate parameters, to allow adding new options - this is particularly suitable for functions that take many parameters and are expected to be extended, and especially for interfaces where we cannot add new methods with different parameters at all.

- Interfaces can include a private method, e.g. `interface { private() }`, to make them unimplementable by external packages and thus allow us to add new methods to the interface without breaking other programs. Of course, this can't be used for interfaces that should be implementable externally.

- We can use [interface upgrades](https://avtok.com/2014/11/05/interface-upgrades.html) to allow implementers of an existing interface to also implement a new interface, as long as the old interface can still be used - e.g. the new interface `BetterReader` may have a method `ReadBetter()`, and a function that takes a `Reader` interface as an input can check if the implementer also implements `BetterReader` and in that case call `ReadBetter()` instead of `Read()`.

## Status

Proposed

## Consequences

### Positive

- Users can safely upgrade with less fear of applications breaking, and know whether an upgrade only includes bug fixes or also functional enhancements

- External developers have a predictable and well-defined API to build on that will be supported for some time

- Less synchronization between teams, since there is a clearer contract and timeline for changes and they happen less frequently

- More documentation will remain accurate, since it's not chasing a moving target

- Less time will be spent on code churn and more time spent on functional improvements, both for the community and for our teams

### Negative

- Many improvements, changes, and bug fixes will have to be postponed until the next major version, possibly for a year or more

- The pace of development will slow down, since we must work within the existing API constraints, and spend more time planning public APIs

- External developers may lose access to some currently exported APIs and functionality

## References

- [#4451: Place internal APIs under internal package](https://github.com/tendermint/tendermint/issues/4451)

- [On Pluggability](https://docs.google.com/document/d/1G08LnwSyb6BAuCVSMF3EKn47CGdhZ5wPZYJQr4-bw58/edit?ts=5f609f11)
