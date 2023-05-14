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

* [#14010](https://github.com/cosmos/cosmos-sdk/pull/14010) Optimize FormatInt to not do plain string concentation when formatting thousands and instead build it more efficiently.
* [#13381](https://github.com/cosmos/cosmos-sdk/pull/13381) Add uint `IsNil` method.
* [#12634](https://github.com/cosmos/cosmos-sdk/pull/12634) Move `sdk.Dec` to math package, call it `LegacyDec`.
* [#14166](https://github.com/cosmos/cosmos-sdk/pull/14166) Add generics versions of Max and Min, catering to all numeric types and allow for variadic calls, replacing the prior typed and strenuous code
* [#15043](https://github.com/cosmos/cosmos-sdk/pull/15043) Add rand functions for testing purposes
