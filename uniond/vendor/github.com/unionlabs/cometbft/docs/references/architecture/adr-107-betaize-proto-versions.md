# ADR 107: Rename proto versions preceding 1.0 to pre-v1 betas

## Changelog

- 2023-07-11: Initial draft (@mzabaluev)

## Status

Accepted

## Context

The drive to introduce [protobuf versioning][cometbft#95] resulted in
introducing versioned packages for protobuf definitions corresponding to
each of the major CometBFT releases starting from 0.34. By the upcoming 0.39
release, some of the packages will get up to `v4`, as the development churn
and the intent to perform code style grooming have resulted in
backward-incompatible changes every time. All this storied history
has not yet been released with semver commitments on [buf schema registry][bsr]
or even merged into the main branch at the time of this writing.

Efforts to conform to the [buf style guide][buf-style]
(started with [#736][cometbft#736]) have been confined to latter versions
of the proto packages in order to preserve source code compatibility
and ease migration for developers. The earlier packages, therefore, do not
constitute exemplary protobuf material and may even be rejected by a schema
repository linter.

## Alternative Approaches

We can do nothing and go ahead with the current versioning rework as per
[ADR 103], which does solve the main problem of managing backward-incompatible
changes in the proto-derived definitions. Come 1.0 release time, we should find
ourselves with a storied collection of versioned protobuf packages going up to
`v4` or `v5` for some packages, where earlier versions refer to pre-1.0 releases
and are in places stylistically bad.

## Decision

Rename the current version suffixes to `v1beta1`, `v1beta2`, ...,
with the intent that the definitions in the 1.0 release become `v1`.

## Detailed Design

Make the historic status of these protocol versions explicit by renaming
the current suffixes to `v1beta1`, `v1beta2`, and so on.
The protobufs that make it to 1.0 will be consistently placed in new packages
suffixed with `v1`, representing a long-term commitment to maintaining
backward compatibility.

At the time of this proposal, the changes will only affect the
`feature/proto-update` feature branch, with no impact on users of any releases.
[ADR 103] details the changes caused by the versioning approach in general.

## Consequences

### Positive

The beta versioning clearly denotes developmental status of these early
specifications. By the 1.0 release, the then current set of specifications is
published as a set of packages suffixed with `.v1`, with no confusion about
what constitutes our "v1 protocol".

### Negative

No negative consequences expected, at this early stage we still have the
freedom to rename version suffixes as we like. Some extra work, mostly
mechanical renaming, is required to implement the change on the feature branch.

### Neutral

The protocol history of the widely deployed 0.x releases will still be present
and consumers can generate usable code from the proto files for those
versions.

## References

* [ADR 103]: Protobuf definition versioning
* [cometbft#95], the meta tracking issue.

[ADR 103]: https://github.com/cometbft/cometbft/blob/main/docs/architecture/adr-103-proto-versioning.md
[cometbft#95]: https://github.com/cometbft/cometbft/issues/95
[cometbft#736]: https://github.com/cometbft/cometbft/issues/736
[bsr]: https://buf.build/product/bsr/
[buf-style]: https://buf.build/docs/best-practices/style-guide
