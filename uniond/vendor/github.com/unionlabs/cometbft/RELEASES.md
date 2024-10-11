# Releases

CometBFT uses modified [semantic versioning](https://semver.org/) with each
release following a `vX.Y.Z` format. The versioning approach used by CometBFT
v1.x onwards differs from that of the v0.x series, and is documented in
[README.md](./README.md#versioning). The `main` branch is used for active
development and thus it is not advisable to build against it.

The latest changes are always initially merged into `main`. Releases are
specified using tags and are built from long-lived "backport" branches that are
cut from `main` when the release process begins. Each release "line" (e.g. 1.0
or 0.38) has its own long-lived backport branch, and the backport branches have
names like `v1.x` or `v0.38.x` (literally, `x`; it is not a placeholder in this
case).

## Backporting

As non-breaking changes land on `main`, they should also be backported to
these backport branches.

We use Mergify's [backport feature](https://mergify.io/features/backports) to
automatically backport to the needed branch. There should be a label for any
backport branch that you'll be targeting. To notify the bot to backport a pull
request, mark the pull request with the label corresponding to the correct
backport branch. For example, to backport to v1.x, add the label
`S:backport-to-v1.x`. Once the original pull request is merged, the bot will
try to cherry-pick the pull request to the backport branch. If the bot fails to
backport, it will open a pull request. The author of the original pull request
is responsible for solving the conflicts and merging the pull request.

### Creating a backport branch

If this is the first release candidate for a major version release, e.g. v2.0.0,
you get to have the honor of creating the backport branch!

Note that, after creating the backport branch, you'll also need to update the
tags on `main` so that `go mod` is able to order the branches correctly.

In the following example, we'll assume that we're making a backport branch for
the 2.x line.

1. Start on `main`

2. Ensure that there is a [branch protection
   rule](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/defining-the-mergeability-of-pull-requests/managing-a-branch-protection-rule)
   for the branch you are about to create (you will need admin access to the
   repository in order to do this).

3. Create and push the backport branch:

   ```sh
   git checkout -b v2.x
   git push origin v2.x
   ```

4. Create a PR to update the documentation directory for the backport branch.

   We rewrite any URLs pointing to `main` to point to the backport branch, so
   that generated documentation will link to the correct versions of files
   elsewhere in the repository. The following files are to be excluded from this
   search:

   * [`README.md`](./README.md)
   * [`CHANGELOG.md`](./CHANGELOG.md)
   * [`UPGRADING.md`](./UPGRADING.md)

   The following links are to always point to `main`, regardless of where they
   occur in the codebase:

   * `https://github.com/cometbft/cometbft/blob/main/LICENSE`

   Be sure to search for all of the following links and replace `main` with your
   corresponding branch label or version (e.g. `v2.x` or `v2.0`):

   * `github.com/cometbft/cometbft/blob/main` ->
     `github.com/cometbft/cometbft/blob/v2.x`
   * `github.com/cometbft/cometbft/tree/main` ->
     `github.com/cometbft/cometbft/tree/v2.x`
   * `docs.cometbft.com/main` -> `docs.cometbft.com/v2`

   Once you have updated all of the relevant documentation:

   ```sh
   # Create and push the PR.
   git checkout -b update-docs-v2.x
   git commit -m "Update docs for v2.x backport branch."
   git push -u origin update-docs-v2.x
   ```

   Be sure to merge this PR before making other changes on the newly-created
   backport branch.

5. Ensure that the RPC docs' `version` field in `rpc/openapi/openapi.yaml` has
   been updated from `main` to the backport branch version.

6. Prepare the [CometBFT documentation
   repository](https://github.com/cometbft/cometbft-docs) to build the release
   branch's version by updating the
   [VERSIONS](https://github.com/cometbft/cometbft-docs/blob/main/VERSIONS)
   file.

After doing these steps, go back to `main` and do the following:

1. Create a new workflow to run E2E nightlies for the new backport branch. (See
   [e2e-nightly-main.yml][e2e] for an example.)

2. Add a new section to the Mergify config (`.github/mergify.yml`) to enable the
   backport bot to work on this branch, and add a corresponding `backport-to-v2.x`
   [label](https://github.com/cometbft/cometbft/labels) so the bot can be triggered.

3. Add a new section to the Dependabot config (`.github/dependabot.yml`) to
   enable automatic update of Go dependencies on this branch. Copy and edit one
   of the existing branch configurations to set the correct `target-branch`.

4. Remove all changelog entries from `.changelog/unreleased` that are destined
   for release from the backport branch.

[e2e]: https://github.com/cometbft/cometbft/blob/main/.github/workflows/e2e-nightly-main.yml

## Pre-releases

Before creating an official release, especially a major release, we may want to
create an alpha or beta version, or release candidate (RC) for our friends and
partners to test out. We use git tags to create pre-releases, and we build them
off of backport branches, for example:

* `v2.0.0-alpha.1` - The first alpha release of `v2.0.0`. Subsequent alpha
  releases will be numbered `v2.0.0-alpha.2`, `v2.0.0-alpha.3`, etc.

  Alpha releases are to be considered the _most_ unstable of pre-releases, and
  are most likely not yet properly QA'd. These are made available to allow early
  adopters to start integrating and testing new functionality before we're done
  with QA.

* `v2.0.0-beta.1` - The first beta release of `v2.0.0`. Subsequent beta
  releases will be numbered `v2.0.0-beta.2`, `v2.0.0-beta.3`, etc.

  Beta releases can be considered more stable than alpha releases in that we
  will have QA'd them better than alpha releases, but there still may be
  minor breaking API changes if users have strong demands for such changes.

* `v2.0.0-rc1` - The first release candidate (RC) of `v2.0.0`. Subsequent RCs
  will be numbered `v2.0.0-rc2`, `v2.0.0-rc3`, etc.

  RCs are considered more stable than beta releases in that we will have
  completed our QA on them. APIs will most likely be stable at this point. The
  difference between an RC and a release is that there may still be small
  changes (bug fixes, features) that may make their way into the series before
  cutting a final release.

(Note that branches and tags _cannot_ have the same names, so it's important
that these branches have distinct names from the tags/release names.)

If this is the first pre-release for a major release, you'll have to make a new
backport branch (see above). Otherwise:

1. Start from the backport branch (e.g. `v2.x`).
2. Run the E2E nightlies (which can be triggered from the GitHub UI; e.g.,
   <https://github.com/cometbft/cometbft/actions/workflows/e2e-manual.yml>).
3. Prepare the pre-release documentation:
   * Build the changelog with [unclog] _without_ doing an unclog release (`unclog build -a > CHANGELOG.md`), and
     commit the built changelog. This ensures that all changelog entries appear
     under an "Unreleased" heading in the pre-release's changelog. The changes
     are only considered officially "released" once we cut a regular (final)
     release.
   * Ensure that `UPGRADING.md` is up-to-date and includes notes on any breaking
     changes or other upgrading flows.
4. Check the dependency to `github.com/cometbft/cometbft/api` in the `go.mod`
   file. If it does not point to an official api version, run `go get github.com/cometbft/cometbft/api`
   and `go mod tidy` (to update `go.sum`) so that it points to one. You may need to tag a new version of the api
   if the last version is too old (i.e., it does not contain the latest
   changes to the protos). If that is the case:
   * `git tag -a api/v2.0.0-rc1 -s -m "Release api module v2.0.0-rc1" origin/v2.x`
   * `git push origin api/v2.0.0-rc1`
   * Notice the prefix `api/`, which denotes that the version refers to the `api` module.
5. Prepare the versioning:
   * Bump CometBFT version in  `version.go`
   * Bump P2P and block protocol versions in  `version.go`, if necessary.
     Check the changelog for breaking changes in these components.
   * Bump ABCI protocol version in `version.go`, if necessary
6. Open a PR with these changes against the backport branch.
7. Once these changes have landed on the backport branch, be sure to pull them
   back down locally.
8. Once you have the changes locally, create the new tag, specifying a name and
   a tag "message":
   `git tag -a v2.0.0-rc1 -s -m "Release Candidate v2.0.0-rc1`
9.  Push the tag back up to origin:
   `git push origin v2.0.0-rc1`
   Now the tag should be available on the repo's releases page.
10. Future pre-releases will continue to be built off of this branch.

## Major release

This major release process assumes that this release was preceded by release
candidates. If there were no release candidates, begin by creating a backport
branch, as described above.

Before performing these steps, be sure the
[Major Release Checklist](#major-release-checklist) has been completed.

1. Start on the backport branch (e.g. `v2.x`)
2. Run the E2E nightlies (which can be triggered from the GitHub UI; e.g.,
   <https://github.com/cometbft/cometbft/actions/workflows/e2e-manual.yml>).
3. Prepare the release:
   * Check the dependency to `github.com/cometbft/cometbft/api` in the `go.mod`
     file. If it does not point to an official api version, run `go get github.com/cometbft/cometbft/api`
     and `go mod tidy` (to update `go.sum`) so that it points to one. You may need to tag a new version of the api
     if the last released version is too old (i.e., it does not contain the latest
     changes to the protos). If that is the case:
     * `git tag -a api/v2.0.0 -s -m "Release api module v2.0.0" origin/v2.x`
     * `git push origin api/v2.0.0`
     * Notice the prefix `api/`, which denotes that the version refers to the `api` module.
   * Do a [release][unclog-release] with [unclog] for the desired version,
     ensuring that you write up a good summary of the major highlights of the
     release that users would be interested in.
   * Build the changelog using unclog, and commit the built changelog.
   * Ensure that `UPGRADING.md` is up-to-date and includes notes on any breaking changes
      or other upgrading flows.
   * Bump CometBFT version in  `version.go`
   * Bump P2P and block protocol versions in  `version.go`, if necessary
   * Bump ABCI protocol version in `version.go`, if necessary
4. Open a PR with these changes against the backport branch.
5. Once these changes are on the backport branch, push a tag with prepared
   release details. This will trigger the actual release `v2.0.0`.
   * `git tag -a v2.0.0 -s -m 'Release v2.0.0'`
   * `git push origin v2.0.0`
6. Make sure that `main` is updated with the latest `CHANGELOG.md`,
   `CHANGELOG_PENDING.md`, and `UPGRADING.md`.

## Minor and patch releases

Minor and patch releases are done differently from major releases: they are
built off of long-lived backport branches, rather than from `main`.  As
non-breaking changes land on `main`, they should also be backported into these
backport branches.

Patch releases don't have release candidates by default, although any tricky
changes may merit a release candidate.

To create a patch release:

1. Checkout the long-lived backport branch: `git checkout v2.x`
2. Run the E2E nightlies (which can be triggered from the GitHub UI; e.g.,
   <https://github.com/cometbft/cometbft/actions/workflows/e2e-manual.yml>).
3. Check out a new branch and prepare the release:
   * Check the dependency to `github.com/cometbft/cometbft/api` in the `go.mod`
     file. If it does not point to an official api version, run `go get github.com/cometbft/cometbft/api`
     and `go mod tidy` (to update `go.sum`) so that it points to one. You may need to tag a new version of the api
     if the last released version is too old (i.e., it does not contain the latest
     changes to the protos). If that is the case:
     * `git tag -a api/v2.0.1 -s -m "Release api module v2.0.1" origin/v2.x`
     * `git push origin api/v2.0.1`
     * Notice the prefix `api/`, which denotes that the version refers to the `api` module.
   * Do a [release][unclog-release] with [unclog] for the desired version,
     ensuring that you write up a good summary of the major highlights of the
     release that users would be interested in.
   * Build the changelog using unclog, and commit the built changelog.
   * Bump the CometBFT in `version.go`
   * Bump the ABCI version number, if necessary. (Note that ABCI follows semver,
     and that ABCI versions are the only versions which can change during patch
     releases, and only field additions are valid patch changes.)
4. Open a PR with these changes that will land them back on `v2.x`
5. Once this change has landed on the backport branch, make sure to pull it locally, then push a tag.
   * `git tag -a v2.0.1 -s -m 'Release v2.0.1'`
   * `git push origin v2.0.1`

   The process for minor releases is similar to patch releases:
   * `git tag -a v2.1.0 -s -m 'Release v2.1.0`
   * `git push origin v2.1.0`
6. Create a pull request back to main with the CHANGELOG and version changes
   from the latest release.
   * Do not merge the backport branch into main.

After the release of `v1.0.0`, the backport branch is named `v1.x`. Any future minor or patch releases will be cut
from the `v1.x` branch. There won't be a separate backport branch for minor releases, so there won't be a `v1.1.x` backport branch.

For more details about versioning guarantees after the `v1.0.0` release, please check
the [versioning](https://github.com/cometbft/cometbft/blob/main/UPGRADING.md#versioning) section in the `UPGRADING.md` document.

## Major Release Checklist

The following set of steps are performed on all releases that increment the
_major_ version, e.g. v1.0 to v2.0. These steps ensure that CometBFT is well
tested, stable, and suitable for adoption by the various diverse projects that
rely on CometBFT.

### Feature Freeze

Ahead of any minor version release of CometBFT, the software enters 'Feature
Freeze' for at least two weeks. A feature freeze means that _no_ new features
are added to the code being prepared for release. No code changes should be made
to the code being released that do not directly improve pressing issues of code
quality. The following must not be merged during a feature freeze:

* Refactors that are not related to specific bug fixes.
* Dependency upgrades.
* New test code that does not test a discovered regression.
* New features of any kind.
* Documentation or spec improvements that are not related to the newly developed
  code.

This period directly follows the creation of the [backport
branch](#creating-a-backport-branch). The CometBFT team instead directs all
attention to ensuring that the existing code is stable and reliable. Broken
tests are fixed, flakey-tests are remedied, end-to-end test failures are
thoroughly diagnosed and all efforts of the team are aimed at improving the
quality of the code. During this period, the upgrade harness tests are run
repeatedly and a variety of in-house testnets are run to ensure CometBFT
functions at the scale it will be used by application developers and node
operators.

### Nightly End-To-End Tests

The CometBFT team maintains [a set of end-to-end
tests](https://github.com/cometbft/cometbft/blob/main/test/e2e/README.md#L1)
that run each night on the latest commit of the project and on the code in the
tip of each supported backport branch. These tests start a network of
containerized CometBFT processes and run automated checks that the network
functions as expected in both stable and unstable conditions. During the feature
freeze, these tests are run nightly and must pass consistently for a release of
CometBFT to be considered stable.

### Upgrade Harness

The CometBFT team is creating an upgrade test harness to exercise the workflow
of stopping an instance of CometBFT running one version of the software and
starting up the same application running the next version. To support upgrade
testing, we will add the ability to terminate the CometBFT process at specific
pre-defined points in its execution so that we can verify upgrades work in a
representative sample of stop conditions.

### Large Scale Testnets

The CometBFT end-to-end tests run a small network (~10s of nodes) to exercise
basic consensus interactions. Real world deployments of CometBFT often have
over a hundred nodes just in the validator set, with many others acting as full
nodes and sentry nodes. To gain more assurance before a release, we will also
run larger-scale test networks to shake out emergent behaviors at scale.

Large-scale test networks are run on a set of virtual machines (VMs). Each VM is
equipped with 4 Gigabytes of RAM and 2 CPU cores. The network runs a very simple
key-value store application. The application adds artificial delays to different
ABCI calls to simulate a slow application. Each testnet is briefly run with no
load being generated to collect a baseline performance. Once baseline is
captured, a consistent load is applied across the network. This load takes the
form of 10% of the running nodes all receiving a consistent stream of two
hundred transactions per minute each.

During each test net, the following metrics are monitored and collected on each
node:

* Consensus rounds per height
* Maximum connected peers, Minimum connected peers, Rate of change of peer connections
* Memory resident set size
* CPU utilization
* Blocks produced per minute
* Seconds for each step of consensus (Propose, Prevote, Precommit, Commit)
* Latency to receive block proposals

For these tests we intentionally target low-powered host machines (with low core
counts and limited memory) to ensure we observe similar kinds of resource contention
and limitation that real-world  deployments of CometBFT experience in production.

#### 200 Node Testnet

To test the stability and performance of CometBFT in a real world scenario,
a 200 node test network is run. The network comprises 5 seed nodes, 175
validators and 20 non-validating full nodes. All nodes begin by dialing
a subset of the seed nodes to discover peers. The network is run for several
days, with metrics being collected continuously. In cases of changes to performance
critical systems, testnets of larger sizes should be considered.

#### Rotating Node Testnet

Real-world deployments of CometBFT frequently see new nodes arrive and old nodes
exit the network. The rotating node testnet ensures that CometBFT is able to
handle this reliably. In this test, a network with 10 validators and 3 seed
nodes is started. A rolling set of 25 full nodes are started and each connects
to the network by dialing one of the seed nodes. Once the node is able to
blocksync to the head of the chain and begins producing blocks using consensus
it is stopped. Once stopped, a new node is started and takes its place. This
network is run for several days.

#### Vote-extension Testnet

CometBFT v0.38.0 introduced **vote-extensions**, which are added as the name
suggests, to precommit votes sent by validators. The Vote-extension Testnet is
used to determine how vote-extensions affect the performance of CometBFT, under
various settings. The application used in the experiment is the same used on the
(#200-node-testnet), but is configured differently to gauge de effects of
varying vote extension sizes. In the (#200-node-testnet) the application extends
pre-commit votes with a 64 bit number encoded with variable compression. In the
Vote-extension Testnet, pre-commit votes are extended with a non-compressed
extension of configurable size. Experiments are run with multiple sizes to
determine their impact and, for comparison sake, we include a run with the same
settings as in the (#200-node-testnet).

The testnet consists of 175 validators, 20 non-validator full-nodes, and 5 seed
nodes. All 195 full-nodes begin by dialing a subset of the seed nodes to
discover peers. Once all full-nodes are started, a 5 minute period is waited
before starting an experiment. For each experiment, the load generators issue
requests at a constant rate during 150 seconds, then wait for 5 minutes to allow
the system to quiesce, then repeat the load generation; the load generation step
is repeated 5 times for each experiment.

#### Network Partition Testnet

CometBFT is expected to recover from network partitions. A partition where no
subset of the nodes is left with the super-majority of the stake is expected to
stop making blocks. Upon alleviation of the partition, the network is expected
to once again become fully connected and capable of producing blocks. The
network partition testnet ensures that CometBFT is able to handle this reliably
at scale. In this test, a network with 100 validators and 95 full nodes is
started. All validators have equal stake. Once the network is producing blocks,
a set of firewall rules is deployed to create a partitioned network with 50% of
the stake on one side and 50% on the other. Once the network stops producing
blocks, the firewall rules are removed and the nodes are monitored to ensure
they reconnect and that the network again begins producing blocks.

#### Absent Stake Testnet

CometBFT networks often run with _some_ portion of the voting power offline. The
absent stake testnet ensures that large networks are able to handle this
reliably. A set of 150 validator nodes and three seed nodes is started. The set
of 150 validators is configured to only possess a cumulative stake of 67% of the
total stake. The remaining 33% of the stake is configured to belong to a
validator that is never actually run in the test network. The network is run for
multiple days, ensuring that it is able to produce blocks without issue.

[unclog]: https://github.com/informalsystems/unclog
[unclog-release]: https://github.com/informalsystems/unclog#releasing-a-new-versions-change-set
