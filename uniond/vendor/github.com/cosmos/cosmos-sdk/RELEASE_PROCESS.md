# Release Process

This document outlines the process for releasing a new version of Cosmos SDK, which involves major release and patch releases as well as maintenance for the major release.

## Major Release Procedure

A _major release_ is an increment of the first number (eg: `v1.2` → `v2.0.0`) or the _point number_ (eg: `v1.1.0 → v1.2.0`, also called _point release_). Each major release opens a _stable release series_ and receives updates outlined in the [Major Release Maintenance](#major-release-maintenance)_section.

Before making a new _major_ release we do beta and release candidate releases. For example, for release 1.0.0:

```text
v1.0.0-beta1 → v1.0.0-beta2 → ... → v1.0.0-rc1 → v1.0.0-rc2 → ... → v1.0.0
```

* Release a first beta version on the `main` branch and freeze `main` from receiving any new features. After beta is released, we focus on releasing the release candidate:
    * finish audits and reviews
    * kick off a large round of simulation testing (e.g. 400 seeds for 2k blocks)
    * perform functional tests
    * add more tests
    * release new beta version as the bugs are discovered and fixed.
* After the team feels that the `main` works fine we create a `release/vY` branch (going forward known a release branch), where `Y` is the version number, with the patch part substituted to `x` (eg: 0.42.x, 1.0.x). Ensure the release branch is protected so that pushes against the release branch are permitted only by the release manager or release coordinator.
    * **PRs targeting this branch can be merged _only_ when exceptional circumstances arise**
    * update the GitHub mergify integration by adding instructions for automatically backporting commits from `main` to the `release/vY` using the `backport/Y` label.
* In the release branch prepare a new version section in the `CHANGELOG.md`
    * All links must be link-ified: `$ python ./scripts/linkify_changelog.py CHANGELOG.md`
    * Create release notes, in `RELEASE_NOTES.md`, highlighting the changes and how to upgrade the SDK. This is needed so the bot knows which entries to add to the release page on GitHub.
* Remove GitHub workflows that should not be in the release branch (eg: `deploy-docs.yml`).
* Create a new annotated git tag for a release candidate (eg: `git tag -a v1.1.0-rc1`) in the release branch.
    * from this point we unfreeze main.
    * the SDK teams collaborate and do their best to run testnets in order to validate the release.
    * when bugs are found, create a PR for `main`, and backport fixes to the release branch.
    * create new release candidate tags after bugs are fixed.
* After the team feels the release branch is stable and everything works, create a full release:
    * update `CHANGELOG.md`.
    * run `gofumpt -w -l .` to format the code.
    * create a new annotated git tag (eg `git -a v1.1.0`) in the release branch.
    * Create a GitHub release.

Following _semver_ philosophy, point releases after `v1.0`:

* must not break API
* can break consensus

Before `v1.0`, point release can break both point API and consensus.

## Patch Release Procedure

A _patch release_ is an increment of the patch number (eg: `v1.2.0` → `v1.2.1`).

**Patch release must not break API nor consensus.**

Updates to the release branch should come from `main` by backporting PRs (usually done by automatic cherry pick followed by a PRs to the release branch). The backports must be marked using `backport/Y` label in PR for main.
It is the PR author's responsibility to fix merge conflicts, update changelog entries, and
ensure CI passes. If a PR originates from an external contributor, a core team member assumes
responsibility to perform this process instead of the original author.
Lastly, it is core team's responsibility to ensure that the PR meets all the SRU criteria.

Point Release must follow the [Stable Release Policy](#stable-release-policy).

After the release branch has all commits required for the next patch release:

* Update `CHANGELOG.md` and `RELEASE_NOTES.md` (if applicable).
* Create a new annotated git tag (eg `git -a v1.1.0`) in the release branch.
    * If the release is a submodule update, first go the submodule folder and name the tag prepending the path to the version:
      `cd core && git -a core/v1.1.0` or `cd tools/cosmovisor && git -a tools/cosmovisor/v1.4.0`
* Create a GitHub release (if applicable).

## Major Release Maintenance

Major Release series continue to receive bug fixes (released as a Patch Release) until they reach **End Of Life**.
Major Release series is maintained in compliance with the **Stable Release Policy** as described in this document.
Note: not every Major Release is denoted as stable releases.

Only the following major release series have a stable release status:

* **0.45** is supported until 6 months after **0.46.0** release. A fairly strict **bugfix-only** rule applies to pull requests that are requested to be included into a stable point-release.
* **0.46** is the last major release and will be supportted until 6 months after **0.47.0** release.
* **0.47** is the next major release and will be supported until 6 months after **0.48.0** release.

## Stable Release Policy

### Patch Releases

Once a Cosmos-SDK release has been completed and published, updates for it are released under certain circumstances
and must follow the [Patch Release Procedure](CONTRIBUTING.md#branching-model-and-release).

### Rationale

Unlike in-development `main` branch snapshots, **Cosmos-SDK** releases are subject to much wider adoption,
and by a significantly different demographic of users. During development, changes in the `main` branch
affect SDK users, application developers, early adopters, and other advanced users that elect to use
unstable experimental software at their own risk.

Conversely, users of a stable release expect a high degree of stability. They build their applications on it, and the
problems they experience with it could be potentially highly disruptive to their projects.

Stable release updates are recommended to the vast majority of developers, and so it is crucial to treat them
with great caution. Hence, when updates are proposed, they must be accompanied by a strong rationale and present
a low risk of regressions, i.e. even one-line changes could cause unexpected regressions due to side effects or
poorly tested code. We never assume that any change, no matter how little or non-intrusive, is completely exempt
of regression risks.

Therefore, the requirements for stable changes are different than those that are candidates to be merged in
the `main` branch. When preparing future major releases, our aim is to design the most elegant, user-friendly and
maintainable SDK possible which often entails fundamental changes to the SDK's architecture design, rearranging and/or
renaming packages as well as reducing code duplication so that we maintain common functions and data structures in one
place rather than leaving them scattered all over the code base. However, once a release is published, the
priority is to minimize the risk caused by changes that are not strictly required to fix qualifying bugs; this tends to
be correlated with minimizing the size of such changes. As such, the same bug may need to be fixed in different
ways in stable releases and `main` branch.

### Migrations

See the SDK's policy on migrations [here](https://docs.cosmos.network/main/migrations/intro).

### What qualifies as a Stable Release Update (SRU)

* **High-impact bugs**
    * Bugs that may directly cause a security vulnerability.
    * _Severe regressions_ from a Cosmos-SDK's previous release. This includes all sort of issues
    that may cause the core packages or the `x/` modules unusable.
    * Bugs that may cause **loss of user's data**.
* Other safe cases:
    * Bugs which don't fit in the aforementioned categories for which an obvious safe patch is known.
    * Relatively small yet strictly non-breaking features with strong support from the community.
    * Relatively small yet strictly non-breaking changes that introduce forward-compatible client
    features to smoothen the migration to successive releases.
    * Relatively small yet strictly non-breaking CLI improvements.

### What does not qualify as SRU

* State machine changes.
* Breaking changes in Protobuf definitions, as specified in [ADR-044](https://github.com/cosmos/cosmos-sdk/blob/main/docs/architecture/adr-044-protobuf-updates-guidelines.md).
* Changes that introduces API breakages (e.g. public functions and interfaces removal/renaming).
* Client-breaking changes in gRPC and HTTP request and response types.
* CLI-breaking changes.
* Cosmetic fixes, such as formatting or linter warning fixes.

### What pull requests will be included in stable point-releases

Pull requests that fix bugs and add features that fall in the following categories do not require a **Stable Release Exception** to be granted to be included in a stable point-release:

* **Severe regressions**.
* Bugs that may cause **client applications** to be **largely unusable**.
* Bugs that may cause **state corruption or data loss**.
* Bugs that may directly or indirectly cause a **security vulnerability**.
* Non-breaking features that are strongly requested by the community.
* Non-breaking CLI improvements that are strongly requested by the community.

### What pull requests will NOT be automatically included in stable point-releases

As rule of thumb, the following changes will **NOT** be automatically accepted into stable point-releases:

* **State machine changes**.
* **Protobug-breaking changes**, as specified in [ADR-044](https://github.com/cosmos/cosmos-sdk/blob/main/docs/architecture/adr-044-protobuf-updates-       guidelines.md).
* **Client-breaking changes**, i.e. changes that prevent gRPC, HTTP and RPC clients to continue interacting with the node without any change.
* **API-breaking changes**, i.e. changes that prevent client applications to _build without modifications_ to the client application's source code.
* **CLI-breaking changes**, i.e. changes that require usage changes for CLI users.

 In some circumstances, PRs that don't meet the aforementioned criteria might be raised and asked to be granted a _Stable Release Exception_.

### Stable Release Exception - Procedure

1. Check that the bug is either fixed or not reproducible in `main`. It is, in general, not appropriate to release bug fixes for stable releases without first testing them in `main`. Please apply the label [v0.43](https://github.com/cosmos/cosmos-sdk/milestone/26) to the issue.
2. Add a comment to the issue and ensure it contains the following information (see the bug template below):

   * **[Impact]** An explanation of the bug on users and justification for backporting the fix to the stable release.
   * A **[Test Case]** section containing detailed instructions on how to reproduce the bug.
   * A **[Regression Potential]** section with a clear assessment on how regressions are most likely to manifest as a result of the pull request that aims to fix the bug in the target stable release.

3. **Stable Release Managers** will review and discuss the PR. Once _consensus_ surrounding the rationale has been reached and the technical review has successfully concluded, the pull request will be merged in the respective point-release target branch (e.g. `release/v0.43.x`) and the PR included in the point-release's respective milestone (e.g. `v0.43.5`).

#### Stable Release Exception - Bug template

```md
#### Impact

Brief xplanation of the effects of the bug on users and a justification for backporting the fix to the stable release.

#### Test Case

Detailed instructions on how to reproduce the bug on Stargate's most recently published point-release.

#### Regression Potential

Explanation on how regressions might manifest - even if it's unlikely.
It is assumed that stable release fixes are well-tested and they come with a low risk of regressions.
It's crucial to make the effort of thinking about what could happen in case a regression emerges.
```

### Stable Release Managers

The **Stable Release Managers** evaluate and approve or reject updates and backports to Cosmos-SDK Stable Release series,
according to the [stable release policy](#stable-release-policy) and [release procedure](#major-release-procedure).
Decisions are made by consensus.

Their responsibilites include:

* Driving the Stable Release Exception process.
* Approving/rejecting proposed changes to a stable release series.
* Executing the release process of stable point-releases in compliance with the [Point Release Procedure](CONTRIBUTING.md).

The Stable Release Managers are appointed by the Interchain Foundation. Currently residing Stable Release Managers:

* @amaurym - Amaury Martiny
