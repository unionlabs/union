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

## Release Hygiene

The Union mono-repo is made up of many different components that are maintained and updated at different rates. We've opted to release components individually rather than in all encompassing versions. To do this, version tags are to now be made in the form of `<component>/v{X}.{Y}.{Z}` (`voyager-v0.2.1`). This will enable us to quickly update and distribute various components without creating monolithic releases.

To create a release:

- Checkout a new branch from main (or desired commit) with a name in the form of `release/<component>/v{X}.{Y}.{Z}`.
- On this branch, create the first release candidate tag in the form `<component>/v{X}.{Y}.{Z}-rc1`.
- Continue iterating release candidate tags until a viable release is generated
- Once a viable release candidate is generated, push a tag in the form `<component>/v{X}.{Y}.{Z}`

Commits should always flow from `dev-branch`→`main`→`release-v{X}.{Y}.{Z}`. In other words, commits should never be made directly to `main` or release branches.

Once a release tag has been pushed, the current `release-v{X}.{Y}.{Z}` branch is frozen. Future work should then be on a new release branch.

Once a tag in the form of `<component>/v{X}.{Y}.{Z}` is pushed to the repo, the component in question will have a release triggered for it from the "Release Component" workflow.

**NOTE**: The Release Component workflow needs to be updated to introduce new components. See the workflow file in `.github/workflows/release-component.yml` for more details.
