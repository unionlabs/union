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

* [#21052](https://github.com/cosmos/cosmos-sdk/pull/21052) Add a migration to v2 config.

## [v0.1.2](https://github.com/cosmos/cosmos-sdk/releases/tag/tools/confix/v0.1.2) - 2024-08-13

* [#21202](https://github.com/cosmos/cosmos-sdk/pull/21202) Allow customization of migration `PlanBuilder`.

## [v0.1.1](https://github.com/cosmos/cosmos-sdk/releases/tag/tools/confix/v0.1.1) - 2023-12-11

* [#18496](https://github.com/cosmos/cosmos-sdk/pull/18496) Remove invalid non SDK config from app.toml migration templates.
 
## [v0.1.0](https://github.com/cosmos/cosmos-sdk/releases/tag/tools/confix/v0.1.0) - 2023-11-07

* [#17904](https://github.com/cosmos/cosmos-sdk/pull/17904) Add `view` command.
* [#14568](https://github.com/cosmos/cosmos-sdk/pull/14568) Add `diff` and `home` commands.
* [#14342](https://github.com/cosmos/cosmos-sdk/pull/14342) Add `confix` tool to manage configuration files.
