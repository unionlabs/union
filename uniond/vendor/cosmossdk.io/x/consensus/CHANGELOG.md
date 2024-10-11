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

### Features

* (x/consensus) [#19483](https://github.com/cosmos/cosmos-sdk/pull/19483) Add consensus messages registration to consensus module.
* [#20615](https://github.com/cosmos/cosmos-sdk/pull/20615) Add consensus messages to add cometinfo to consensus modules

### API Breaking Changes

* (x/consensus) [#19488](https://github.com/cosmos/cosmos-sdk/pull/19488) Consensus module creation takes `appmodule.Environment` instead of individual services.
* (x/consensus) [#18041](https://github.com/cosmos/cosmos-sdk/pull/18041) `ToProtoConsensusParams()` returns an error

