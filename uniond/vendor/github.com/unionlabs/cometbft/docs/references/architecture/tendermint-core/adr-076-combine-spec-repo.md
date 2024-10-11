# ADR 076: Combine Spec and Tendermint Repositories 

## Changelog

- 2022-02-04: Initial Draft. (@tychoish)

## Status

Implemented

## Context

While the specification for Tendermint was originally in the same
repository as the Go implementation, at some point the specification
was split from the core repository and maintained separately from the
implementation. While this makes sense in promoting a conceptual
separation of specification and implementation, in practice this
separation was a premature optimization, apparently aimed at supporting
alternate implementations of Tendermint. 

The operational and documentary burden of maintaining a separate
spec repo has not returned value to justify its cost. There are no active
projects to develop alternate implementations of Tendermint based on the
common specification, and having separate repositories creates an ongoing
burden to coordinate versions, documentation, and releases.

## Decision

The specification repository will be merged back into the Tendermint
core repository.

Stakeholders including representatives from the maintainers of the
spec, the Go implementation, and the Tendermint Rust library, agreed
to merge the repositories in the Tendermint core dev meeting on 27
January 2022, including @williambanfield @cmwaters @creachadair and
@thanethomson.

## Alternative Approaches

The main alternative we considered was to keep separate repositories,
and to introduce a coordinated versioning scheme between the two, so
that users could figure out which spec versions go with which versions
of the core implementation.

We decided against this on the grounds that it would further complicate
the release process for _both_ repositories, without mitigating any of
the other existing issues.

## Detailed Design

Clone and merge the master branch of the `tendermint/spec` repository
as a branch of the `tendermint/tendermint`, to ensure the commit history
of both repositories remains intact.

### Implementation Instructions

1. Within the `tendermint` repository, execute the following commands 
   to add a new branch with the history of the master branch of `spec`:

   ```bash
   git remote add spec git@github.com:tendermint/spec.git
   git fetch spec
   git checkout -b spec-master spec/master
   mkdir spec
	git ls-tree -z --name-only HEAD | xargs -0 -I {} git mv {} subdir/
	git commit -m "spec: organize specification prior to merge"
	git checkout -b spec-merge-mainline origin/master
	git merge --allow-unrelated-histories spec-master
	```

   This merges the spec into the `tendermint/tendermint` repository as
   a normal branch. This commit can also be backported to the 0.35
   branch, if needed.

2. Migrate outstanding issues from `tendermint/spec` to the
   `tendermint/tendermint` repository.

3. In the specification repository, add redirect to the README and mark
   the repository as archived. 
   

## Consequences

### Positive

Easier maintenance for the specification will obviate a number of
complicated and annoying versioning problems, and will help prevent the
possibility of the specification and the implementation drifting apart.

Additionally, co-locating the specification will help encourage
cross-pollination and collaboration, between engineers focusing on the
specification and the protocol and engineers focusing on the implementation.

### Negative

Co-locating the spec and Go implementation has the potential effect of
prioritizing the Go implementation with regards to the spec, and
making it difficult to think about alternate implementations of the
Tendermint algorithm. Although we may want to foster additional
Tendermint implementations in the future, this isn't an active goal
in our current roadmap, and *not* merging these repos doesn't
change the fact that the Go implementation of Tendermint is already the
primary implementation.

### Neutral

N/A

## References

- https://github.com/tendermint/tendermint/tree/main/spec
- https://github.com/tendermint/tendermint
