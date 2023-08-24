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

* [#15956](https://github.com/cosmos/cosmos-sdk/pull/15956) Introduce extra options to configure logger.

## [v1.0.0](https://github.com/cosmos/cosmos-sdk/releases/tag/log/v1.0.0) - 2023-03-30

* [#15601](https://github.com/cosmos/cosmos-sdk/pull/15601) Introduce logger options. These options allow to configure the logger with filters, different level and output format.

## [v0.1.0](https://github.com/cosmos/cosmos-sdk/releases/tag/log/v0.1.0) - 2023-03-13

* Introducing a standalone SDK logger package (`comossdk.io/log`).
  It replaces CometBFT logger and provides a common interface for all SDK components.
  The default logger (`NewLogger`) is using [zerolog](https://github.com/rs/zerolog),
  but it can be easily replaced with any implementation that implements the `log.Logger` interface.
