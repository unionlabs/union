# Versioning

This document describes the versioning of `uniond` and how versions should be interpreted.

Two main versioning processes are elaborated on below: our use of semantic versioning & release candidates.

## Semantic Versioning

Our implementation of semantic versioning is in the form of `v{X}.{Y}.{Z}` where:

- `{X}` indicates the major version. Major versions contain new features that may be incompatible with older versions and alter union's consensus. Detailed release notes and announcements can be expected for major versions.

  Node Update: `required`

- `{Y}` indicates the minor version. Minor versions may contain non-feature changes that may still be breaking. Usually these are reserved for security updates and bug fixes. Detailed release notes and announcements can be expected for minor versions.

  Node Update: `required`

- `{Z}` indicates the patch version. Patch versions will not contain breaking changes or new features. Release notes will be auto-generated from git commits for patch versions, announcements should not be expected.

  Node Update: `optional`

If a version update is `required`, validators who fail to update will face slashing penalties. Validators who fail to upgrade to a new `optional` version will not be slashed.

## Release Candidates

The Union testnet will track the most recent release candidate. Release candidates will be marked by post-fixed version information in the form of `-rc{A}` where `{A}` is the incremented release candidate version, starting at `1`. The full release candidate version will be in the form of `v{X}.{Y}.{Z}-rc{A}`.

## Release Branch Hygiene (deprecated)

When the time comes for a new major/minor release version, a new branch of the form `release-v{X}.{Y}.{Z}` will be created. Then the first release candidate tag will be pushed: `v{X}.{Y}.{Z}-rc1`. The release candidate tag will trigger a new pre-release workflow which will bundle the release.

In case of regressions in testnet, bug fixes are cherry-picked from main to the candidate, and a new candidate is tagged. Once a release candidate passes testnet, a release tag is pushed in the form `v{X}.{Y}.{Z}`. This release tag will trigger the release workflow which will bundle a full release.

Once a release tag has been pushed, the current `release-v{X}.{Y}.{Z}` branch is frozen. Future work should then be assembled in a new release branch.

Commits should always flow from `dev-branch`→`main`→`release-v{X}.{Y}.{Z}`. In other words, commits should never be made directly to `main` or release branches.

## Release Hygiene

The Union mono-repo is made up of many different components that are maintained and updated at different rates. We've opted to release components individually rather than in all encompassing versions. To do this, version tags are to now be made in the form of `<component>-<semantic_version>` (`voyager-v0.2.1`). This will enable us to quickly update and distribute various components without creating monolithic releases.

Once a tag in the form of `<component>-<semantic_version>` is pushed to the repo, the component in question will have a release triggered for it from the "Release Component" workflow.

**NOTE**: The Release Component workflow needs to be updated to introduce new components. See the workflow file in `.github/workflows/release-component.yml` for more details.
