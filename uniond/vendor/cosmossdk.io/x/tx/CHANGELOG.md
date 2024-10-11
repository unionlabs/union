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

Since v0.13.0, x/tx follows Cosmos SDK semver: https://github.com/cosmos/cosmos-sdk/blob/main/RELEASES.md
-->

# Changelog

## [Unreleased]

## [v0.13.4](https://github.com/cosmos/cosmos-sdk/releases/tag/x/tx/v0.13.4) - 2024-08-02

### Improvements

* [#21073](https://github.com/cosmos/cosmos-sdk/pull/21073) In Context use sync.Map `getSignersFuncs` map from concurrent writes, we also call Validate when creating the Context.

## [v0.13.3](https://github.com/cosmos/cosmos-sdk/releases/tag/x/tx/v0.13.3) - 2024-04-22

### Improvements

* [#20049](https://github.com/cosmos/cosmos-sdk/pull/20049) Sort JSON attributes for `inline_json` encoder.

## [v0.13.2](https://github.com/cosmos/cosmos-sdk/releases/tag/x/tx/v0.13.2) - 2024-04-12

### Features

* [#19786](https://github.com/cosmos/cosmos-sdk/pull/19786)/[#19919](https://github.com/cosmos/cosmos-sdk/pull/19919) Add "inline_json" option to Amino JSON encoder.

### Improvements

* [#19845](https://github.com/cosmos/cosmos-sdk/pull/19845) Use hybrid resolver instead of only protov2 registry

### Bug Fixes

* [#19955](https://github.com/cosmos/cosmos-sdk/pull/19955) Don't shadow Amino marshalling error messages

## [v0.13.1](https://github.com/cosmos/cosmos-sdk/releases/tag/x/tx/v0.13.1) - 2024-03-05

### Features

* [#19618](https://github.com/cosmos/cosmos-sdk/pull/19618) Add enum as string option to encoder.

### Improvements

* [#18857](https://github.com/cosmos/cosmos-sdk/pull/18857) Moved `FormatCoins` from `core/coins` to this package under `signing/textual`.

### Bug Fixes

* [#19265](https://github.com/cosmos/cosmos-sdk/pull/19265) Reject denoms that contain a comma.

## [v0.13.0](https://github.com/cosmos/cosmos-sdk/releases/tag/x/tx/v0.13.0) - 2023-12-19

### Improvements

* [#18740](https://github.com/cosmos/cosmos-sdk/pull/18740) Support nested messages when fetching signers up to a default depth of 32.
