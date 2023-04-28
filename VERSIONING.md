# Versioning

This document aims to convey our standards for versioning of the uniond binary and what versions entail for validators.

Two main versioning processes are elaborated on below: our use of semantic versioning & release candidates.

## Semantic Versioning

Our implementation of semantic versioning is in the form of `v{X}.{Y}.{Z}` where:

* `{X}` indicates the major version. Major versions contain new features that may be breaking. Detailed release notes and announcements can be expected for major versions.

  Validator Update: `required`

* `{Y}` indicates the minor version. Minor versions may contain non-feature changes that may still be breaking. Detailed release notes and announcements can be expected for minor versions.

  Validator Update: `required`

* `{Z}` indicates the patch version. Patch versions will not contain breaking or feature changes. Release notes will be auto-generated from git commits for patch versions, announcements are not expected.

  Validator Update: `optional`

If a version update is `required`, validators who fail to update will face slashing penalties. Validators who fail to upgrade to a new `optional` version will not be slashed.

## Release Candidates

The Union testnet will track the most recent release candidate. Release candidates will be marked by post-fixed version information in the form of `-rc{A}` where `{A}` is the incremented release candidate version. The full release candidate version will be in the form of `v{X}.{Y}.{Z}-rc{A}`.

## Release Branch Hygiene

When the time comes for a new major/minor release version, a new branch of the form `release-v{X}.{Y}` will be created. Once the correct commits for this release have been cherry-picked from main, the first release candidate tag will be pushed: `v{X}.{Y}.{Z}-rc1`. The release candidate tag will trigger a new pre-release workflow which will bundle the release.

Release candidates will continue to be produced from cherry-picked commits until a release candidate tag sufficiently passes quality assurance. Once a release candidate tag is sufficient, a release tag is pushed of the form `v{X}.{Y}.{Z}`. This release tag will trigger the release workflow which will bundle a full release.

Once a release tag has been pushed, the next release candidate tag should be of the form `v{X}.{Y}.{Z+1}-rc1` for the next patch version. After this - the process towards the next patch version can begin again.

Commits should always flow from `dev-branch`→`main`→`release-v{X}.{Y}`. In other words, commits should never be made directly to `main` or release branches.
