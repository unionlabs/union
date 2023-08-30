<!--
Guiding Principles:

Changelogs are for humans, not machines.
There should be an entry for every single version.
The same types of changes should be grouped.
Versions and sections should be linkable.
The latest version comes first.
The release date of each version is displayed.
Mention whether you follow Semantic Versioning.

Usage:

Change log entries are to be added to the Unreleased section under the
appropriate stanza (see below). Each entry should ideally include a tag and
the Github issue reference in the following format:

* (<tag>) [#<issue-number>] Changelog message.

Types of changes (Stanzas):

"Features" for new features.
"Improvements" for changes in existing functionality.
"Deprecated" for soon-to-be removed features.
"Bug Fixes" for any bug fixes.
"API Breaking" for breaking exported APIs used by developers building on SDK.
Ref: https://keepachangelog.com/en/1.0.0/
-->

# Changelog

## [Unreleased]

## [v1.0.0](https://github.com/cosmos/cosmos-sdk/releases/tag/errors%2Fv1.0.0)

### Features

* [#15989](https://github.com/cosmos/cosmos-sdk/pull/15989) Add `ErrStopIterating` for modules to use for breaking out of iteration.
* [#10779](https://github.com/cosmos/cosmos-sdk/pull/10779) Import code from the `github.com/cosmos/cosmos-sdk/types/errors` package.
* [#11274](https://github.com/cosmos/cosmos-sdk/pull/11274) Add `RegisterWithGRPCCode` function to associate a gRPC error code with errors.

### Improvements

* [#11762](https://github.com/cosmos/cosmos-sdk/pull/11762) Improve error messages.

### API Breaking

* [#11274](https://github.com/cosmos/cosmos-sdk/pull/11274) `New` now is an alias for `Register` and should only be used in initialization code.

### Bug Fixes

* [#11714](https://github.com/cosmos/cosmos-sdk/pull/11714) Add wrapped error messages in `GRPCStatus()`
