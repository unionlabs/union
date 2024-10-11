# ADR 103: Protobuf definition versioning

## Changelog

- 2023-04-27: First draft (@thanethomson)

## Status

Accepted

## Context

In CometBFT v0.34 through v0.38, the Protocol Buffers definitions in the `proto`
folder are organized in a relatively flat structure and evolve over time in
major releases of CometBFT.

For integrators who may want to interact with nodes running different versions
of CometBFT, this means that, when consuming those Protobuf definitions, they
need to clone the CometBFT repository and check out the definitions for each and
every major version they want to support. It also means that they need to
manually diff the types to get a precise sense of what changed between CometBFT
releases. Moreover, the differences obtained with the manual diff are often
undocumented.

We hypothesize that, if we had to version our Protobuf definitions according to
[Buf's style guide][buf-style], we could simplify integration efforts with
CometBFT in the long run. Adopting such a practice would also introduce a new
level of rigour into the way CometBFT Protobuf definitions are managed, allowing
for better expectation management with integrators. It would also allow the core
team to make better use of Protobuf-related tooling (such as Buf) in enforcing
standards, conventions and versioning practices in an automated way.

Some context is captured in [\#95] and the issues/PRs to which it links.

## Alternative Approaches

1. The main alternative here is to do nothing and keep our existing Protobuf
   definition approach, which suffers from the issues captured in the
   [Context](#context) section.
2. Version our Protobuf definitions, as per this ADR. The primary drawback of
   introducing versioning is that it represents a substantial breaking change.
   Within this approach, there are two alternative policies that @mzabaluev has
   proposed:
   1. **Conservative**, recommended by Buf: Do not create new versions when
      making non-breaking changes to Protobuf message definitions, such as
      adding a new field. The upside to this approach is that non-breaking
      changes can be made in non-breaking releases of CometBFT, but the downside
      is that some code generators, such as [prost] for Rust, cannot generate
      non-breaking code from these non-breaking changes.
   2. **Sensitive**: Create a new version of a Protobuf message definition any
      time anything changes, including adding a new field. The upside to this
      approach is that generated code in languages like Rust is additive and
      non-breaking, but the downside is that non-breaking Protobuf changes can
      only be released in new, breaking versions of CometBFT.

## Decision

From team discussion, it was decided to adopt versioning, using the **sensitive
versioning** policy.

## Detailed Design

### Implementation

In order to implement Protobuf definition versioning, it is recommended to bring
all of the Protobuf definitions for all currently maintained major CometBFT
releases thus far (v0.34, v0.37 and v0.38 as of the time of writing this ADR)
into `main`, implementing versioning for each major version's definitions.

For example, the v0.34 Protobuf definitions would be brought to `main` as `v1`
(i.e. `proto/tendermint/types/types.proto` would become
`proto/tendermint/types/v1/types.proto`, and the package would change from
`tendermint.types` to `tendermint.types.v1`).

Then, the v0.37 Protobuf definitions would be brought to `main`, and only where
changes were made according to our **sensitive** versioning policy would we
create new types (e.g. `proto/tendermint/types/params.proto` from v0.37 would
become `proto/tendermint/types/v2/params.proto` and the package would change
from `tendermint.types` to `tendermint.types.v2`).

### Minimizing breaking impact

Changing the type URLs of Protobuf message types (e.g. `tendermint.types.Block`
becoming `tendermint.types.v1.Block`), is considered a breaking change because,
in some cases, integrators may be serializing structures into Protobuf
`Any`-typed fields. When doing so, the URL of the type being serialized is also
embedded in the encoded message.

In order to minimize the impact of this breaking change, it is proposed to keep
type definitions in the `proto` folder that are _wire-compatible_ with the
_latest_ versioned types, but that are unversioned (e.g. exposing
`tendermint.types.Block`, which would be wire-compatible with
`tendermint.types.v1.Block`).

Internally, however, CometBFT would make use of types generated from the
versioned Protobuf types only.

This would facilitate a transition period for integrators who depend on the old
type URL structure, and would allow for a grace period to upgrade to use the
versioned type definitions.

### Ergonomics of generated code

One of the challenges associated with this change is that usage of the generated
Go code becomes more difficult for the core team. This is because all generated
code will also have version suffixes in package paths, meaning that the core
team needs to know exactly which versions of which messages are relevant to the
current release when wiring them up.

A simple way of mitigating the impact here is to introduce type aliases for the
latest versions of generated types, and make use of these aliases internally.

### Rollout strategy

The current recommended strategy to roll out this change is to accumulate all
incremental changes on a **feature branch**, which can then be targeted to a
specific release. At present, it is envisaged that CometBFT v0.39 or v0.40 will
introduce these changes, but this could change based on negotiation with
stakeholders.

## Consequences

### Positive

- Protobuf definitions that are wire-compatible with all versions of CometBFT
  will be able to be packaged together
- The combined Protobuf definitions could be uploaded to the Buf registry for
  easy consumption by integrators
- Changes to Protobuf definitions will be more obvious and explicit to
  integrators

### Negative

- While some of the short-term impact of this change can be mitigated,
  ultimately this is a substantial breaking change for some integrators who
  depend on the type URLs of the Protobuf definitions
- Requires slightly more maintenance than not versioning the types if type
  aliases need to be maintained for generated code

[\#95]: https://github.com/cometbft/cometbft/issues/95
[buf-style]: https://buf.build/docs/best-practices/style-guide/
[prost]: https://github.com/tokio-rs/prost/
