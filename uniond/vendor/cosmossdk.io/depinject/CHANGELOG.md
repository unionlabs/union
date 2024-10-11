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

Change log entries are to be added to the Unreleased section from newest to oldest.
Each entry must include the Github issue reference in the following format:

* [#<issue-number>] Changelog message.

-->

# Changelog

## [Unreleased]

## 1.0.0

* [#20540](https://github.com/cosmos/cosmos-sdk/pull/20540) Add support for defining `appconfig` module configuration types using `github.com/cosmos/gogoproto/proto` in addition to `google.golang.org/protobuf` so that users can use gogo proto across their stack.
* Move `cosmossdk.io/core/appconfig` to `cosmossdk.io/depinject/appconfig`.

## 1.0.0-alpha.x

Depinject is still in alpha stage even though its API is already quite stable.
There is no changelog during this stage.
