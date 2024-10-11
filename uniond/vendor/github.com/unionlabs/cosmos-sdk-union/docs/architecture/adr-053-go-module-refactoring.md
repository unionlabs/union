# ADR 053: Go Module Refactoring

## Changelog

* 2022-04-27: First Draft

## Status

PROPOSED

## Abstract

The current SDK is built as a single monolithic go module. This ADR describes
how we refactor the SDK into smaller independently versioned go modules
for ease of maintenance.

## Context

Go modules impose certain requirements on software projects with respect to
stable version numbers (anything above 0.x) in that [any API breaking changes
necessitate a major version](https://go.dev/doc/modules/release-workflow#breaking)
increase which technically creates a new go module
(with a v2, v3, etc. suffix).

[Keeping modules API compatible](https://go.dev/blog/module-compatibility) in
this way requires a fair amount of fair thought and discipline.

The Cosmos SDK is a fairly large project which originated before go modules
came into existence and has always been under a v0.x release even though
it has been used in production for years now, not because it isn't production
quality software, but rather because the API compatibility guarantees required
by go modules are fairly complex to adhere to with such a large project.
Up to now, it has generally been deemed more important to be able to break the
API if needed rather than require all users update all package import paths
to accommodate breaking changes causing v2, v3, etc. releases. This is in
addition to the other complexities related to protobuf generated code that will
be addressed in a separate ADR.

Nevertheless, the desire for semantic versioning has been [strong in the
community](https://github.com/cosmos/cosmos-sdk/discussions/10162) and the
single go module release process has made it very hard to
release small changes to isolated features in a timely manner. Release cycles
often exceed six months which means small improvements done in a day or
two get bottle-necked by everything else in the monolithic release cycle.

## Decision

To improve the current situation, the SDK is being refactored into multiple
go modules within the current repository. There has been a [fair amount of
debate](https://github.com/cosmos/cosmos-sdk/discussions/10582#discussioncomment-1813377)
as to how to do this, with some developers arguing for larger vs smaller
module scopes. There are pros and cons to both approaches (which will be
discussed below in the [Consequences](#consequences) section), but the
approach being adopted is the following:

* a go module should generally be scoped to a specific coherent set of
functionality (such as math, errors, store, etc.)
* when code is removed from the core SDK and moved to a new module path, every 
effort should be made to avoid API breaking changes in the existing code using
aliases and wrapper types (as done in https://github.com/cosmos/cosmos-sdk/pull/10779
and https://github.com/cosmos/cosmos-sdk/pull/11788)
* new go modules should be moved to a standalone domain (`cosmossdk.io`) before
being tagged as `v1.0.0` to accommodate the possibility that they may be
better served by a standalone repository in the future
* all go modules should follow the guidelines in https://go.dev/blog/module-compatibility
before `v1.0.0` is tagged and should make use of `internal` packages to limit
the exposed API surface
* the new go module's API may deviate from the existing code where there are
clear improvements to be made or to remove legacy dependencies (for instance on
amino or gogo proto), as long the old package attempts
to avoid API breakage with aliases and wrappers
* care should be taken when simply trying to turn an existing package into a
new go module: https://github.com/golang/go/wiki/Modules#is-it-possible-to-add-a-module-to-a-multi-module-repository.
In general, it seems safer to just create a new module path (appending v2, v3, etc.
if necessary), rather than trying to make an old package a new module.

## Consequences

### Backwards Compatibility

If the above guidelines are followed to use aliases or wrapper types pointing
in existing APIs that point back to the new go modules, there should be no or
very limited breaking changes to existing APIs.

### Positive

* standalone pieces of software will reach `v1.0.0` sooner
* new features to specific functionality will be released sooner 

### Negative

* there will be more go module versions to update in the SDK itself and
per-project, although most of these will hopefully be indirect

### Neutral

## Further Discussions

Further discussions are occurring in primarily in
https://github.com/cosmos/cosmos-sdk/discussions/10582 and within
the Cosmos SDK Framework Working Group.

## References

* https://go.dev/doc/modules/release-workflow
* https://go.dev/blog/module-compatibility
* https://github.com/cosmos/cosmos-sdk/discussions/10162
* https://github.com/cosmos/cosmos-sdk/discussions/10582
* https://github.com/cosmos/cosmos-sdk/pull/10779
* https://github.com/cosmos/cosmos-sdk/pull/11788
