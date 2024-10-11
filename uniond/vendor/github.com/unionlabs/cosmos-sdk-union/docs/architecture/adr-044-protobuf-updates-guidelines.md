# ADR 044: Guidelines for Updating Protobuf Definitions

## Changelog

* 28.06.2021: Initial Draft
* 02.12.2021: Add `Since:` comment for new fields
* 21.07.2022: Remove the rule of no new `Msg` in the same proto version.

## Status

Draft

## Abstract

This ADR provides guidelines and recommended practices when updating Protobuf definitions. These guidelines are targeting module developers.

## Context

The Cosmos SDK maintains a set of [Protobuf definitions](https://github.com/cosmos/cosmos-sdk/tree/main/proto/cosmos). It is important to correctly design Protobuf definitions to avoid any breaking changes within the same version. The reasons are to not break tooling (including indexers and explorers), wallets and other third-party integrations.

When making changes to these Protobuf definitions, the Cosmos SDK currently only follows [Buf's](https://docs.buf.build/) recommendations. We noticed however that Buf's recommendations might still result in breaking changes in the SDK in some cases. For example:

* Adding fields to `Msg`s. Adding fields is a not a Protobuf spec-breaking operation. However, when adding new fields to `Msg`s, the unknown field rejection will throw an error when sending the new `Msg` to an older node.
* Marking fields as `reserved`. Protobuf proposes the `reserved` keyword for removing fields without the need to bump the package version. However, by doing so, client backwards compatibility is broken as Protobuf doesn't generate anything for `reserved` fields. See [#9446](https://github.com/cosmos/cosmos-sdk/issues/9446) for more details on this issue.

Moreover, module developers often face other questions around Protobuf definitions such as "Can I rename a field?" or "Can I deprecate a field?" This ADR aims to answer all these questions by providing clear guidelines about allowed updates for Protobuf definitions.

## Decision

We decide to keep [Buf's](https://docs.buf.build/) recommendations with the following exceptions:

* `UNARY_RPC`: the Cosmos SDK currently does not support streaming RPCs.
* `COMMENT_FIELD`: the Cosmos SDK allows fields with no comments.
* `SERVICE_SUFFIX`: we use the `Query` and `Msg` service naming convention, which doesn't use the `-Service` suffix.
* `PACKAGE_VERSION_SUFFIX`: some packages, such as `cosmos.crypto.ed25519`, don't use a version suffix.
* `RPC_REQUEST_STANDARD_NAME`: Requests for the `Msg` service don't have the `-Request` suffix to keep backwards compatibility.

On top of Buf's recommendations we add the following guidelines that are specific to the Cosmos SDK.

### Updating Protobuf Definition Without Bumping Version

#### 1. Module developers MAY add new Protobuf definitions

Module developers MAY add new `message`s, new `Service`s, new `rpc` endpoints, and new fields to existing messages. This recommendation follows the Protobuf specification, but is added in this document for clarity, as the SDK requires one additional change.

The SDK requires the Protobuf comment of the new addition to contain one line with the following format:

```protobuf
// Since: cosmos-sdk <version>{, <version>...}
```

Where each `version` denotes a minor ("0.45") or patch ("0.44.5") version from which the field is available. This will greatly help client libraries, who can optionally use reflection or custom code generation to show/hide these fields depending on the targeted node version.

As examples, the following comments are valid:

```protobuf
// Since: cosmos-sdk 0.44

// Since: cosmos-sdk 0.42.11, 0.44.5
```

and the following ones are NOT valid:

```protobuf
// Since cosmos-sdk v0.44

// since: cosmos-sdk 0.44

// Since: cosmos-sdk 0.42.11 0.44.5

// Since: Cosmos SDK 0.42.11, 0.44.5
```

#### 2. Fields MAY be marked as `deprecated`, and nodes MAY implement a protocol-breaking change for handling these fields

Protobuf supports the [`deprecated` field option](https://protobuf.dev/programming-guides/proto2/), and this option MAY be used on any field, including `Msg` fields. If a node handles a Protobuf message with a non-empty deprecated field, the node MAY change its behavior upon processing it, even in a protocol-breaking way. When possible, the node MUST handle backwards compatibility without breaking the consensus (unless we increment the proto version).

As an example, the Cosmos SDK v0.42 to v0.43 update contained two Protobuf-breaking changes, listed below. Instead of bumping the package versions from `v1beta1` to `v1`, the SDK team decided to follow this guideline, by reverting the breaking changes, marking those changes as deprecated, and modifying the node implementation when processing messages with deprecated fields. More specifically:

* The Cosmos SDK recently removed support for [time-based software upgrades](https://github.com/cosmos/cosmos-sdk/pull/8849). As such, the `time` field has been marked as deprecated in `cosmos.upgrade.v1beta1.Plan`. Moreover, the node will reject any proposal containing an upgrade Plan whose `time` field is non-empty.
* The Cosmos SDK now supports [governance split votes](./adr-037-gov-split-vote.md). When querying for votes, the returned `cosmos.gov.v1beta1.Vote` message has its `option` field (used for 1 vote option) deprecated in favor of its `options` field (allowing multiple vote options). Whenever possible, the SDK still populates the deprecated `option` field, that is, if and only if the `len(options) == 1` and `options[0].Weight == 1.0`.

#### 3. Fields MUST NOT be renamed

Whereas the official Protobuf recommendations do not prohibit renaming fields, as it does not break the Protobuf binary representation, the SDK explicitly forbids renaming fields in Protobuf structs. The main reason for this choice is to avoid introducing breaking changes for clients, which often rely on hard-coded fields from generated types. Moreover, renaming fields will lead to client-breaking JSON representations of Protobuf definitions, used in REST endpoints and in the CLI.

### Incrementing Protobuf Package Version

TODO, needs architecture review. Some topics:

* Bumping versions frequency
* When bumping versions, should the Cosmos SDK support both versions?
    * i.e. v1beta1 -> v1, should we have two folders in the Cosmos SDK, and handlers for both versions?
* mention ADR-023 Protobuf naming

## Consequences

> This section describes the resulting context, after applying the decision. All consequences should be listed here, not just the "positive" ones. A particular decision may have positive, negative, and neutral consequences, but all of them affect the team and project in the future.

### Backwards Compatibility

> All ADRs that introduce backwards incompatibilities must include a section describing these incompatibilities and their severity. The ADR must explain how the author proposes to deal with these incompatibilities. ADR submissions without a sufficient backwards compatibility treatise may be rejected outright.

### Positive

* less pain to tool developers
* more compatibility in the ecosystem
* ...

### Negative

{negative consequences}

### Neutral

* more rigor in Protobuf review

## Further Discussions

This ADR is still in the DRAFT stage, and the "Incrementing Protobuf Package Version" will be filled in once we make a decision on how to correctly do it.

## Test Cases [optional]

Test cases for an implementation are mandatory for ADRs that are affecting consensus changes. Other ADRs can choose to include links to test cases if applicable.

## References

* [#9445](https://github.com/cosmos/cosmos-sdk/issues/9445) Release proto definitions v1
* [#9446](https://github.com/cosmos/cosmos-sdk/issues/9446) Address v1beta1 proto breaking changes
