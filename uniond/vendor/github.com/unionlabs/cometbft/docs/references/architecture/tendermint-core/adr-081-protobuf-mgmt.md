# ADR 081: Protocol Buffers Management

## Changelog

- 2022-02-28: First draft

## Status

Accepted

[Tracking issue](https://github.com/tendermint/tendermint/issues/8121)

## Context

At present, we manage the [Protocol Buffers] schema files ("protos") that define
our wire-level data formats within the Tendermint repository itself (see the
[`proto`](../../proto/) directory). Recently, we have been making use of [Buf],
both locally and in CI, in order to generate Go stubs, and lint and check
`.proto` files for breaking changes.

The version of Buf used at the time of this decision was `v1beta1`, and it was
discussed in [\#7975] and in weekly calls as to whether we should upgrade to
`v1` and harmonize our approach with that used by the Cosmos SDK. The team
managing the Cosmos SDK was primarily interested in having our protos versioned
and easily accessible from the [Buf] registry.

The three main sets of stakeholders for the `.proto` files and their needs, as
currently understood, are as follows.

1. Tendermint needs Go code generated from `.proto` files.
2. Consumers of Tendermint's `.proto` files, specifically projects that want to
   interoperate with Tendermint and need to generate code for their own
   programming language, want to be able to access these files in a reliable and
   efficient way.
3. The Tendermint Core team wants to provide stable interfaces that are as easy
   as possible to maintain, on which consumers can depend, and to be able to
   notify those consumers promptly when those interfaces change. To this end, we
   want to:
   1. Prevent any breaking changes from being introduced in minor/patch releases
      of Tendermint. Only major version updates should be able to contain
      breaking interface changes.
   2. Prevent generated code from diverging from the Protobuf schema files.

There was also discussion surrounding the notion of automated documentation
generation and hosting, but it is not clear at this time whether this would be
that valuable to any of our stakeholders. What will, of course, be valuable at
minimum would be better documentation (in comments) of the `.proto` files
themselves.

## Alternative Approaches

### Meeting stakeholders' needs

1. Go stub generation from protos. We could use:
   1. [Buf]. This approach has been rather cumbersome up to this point, and it
      is not clear what Buf really provides beyond that which `protoc` provides
      to justify the additional complexity in configuring Buf for stub
      generation.
   2. [protoc] - the Protocol Buffers compiler.
2. Notification of breaking changes:
   1. Buf in CI for all pull requests to *release* branches only (and not on
      `master`).
   2. Buf in CI on every pull request to every branch (this was the case at the
      time of this decision, and the team decided that the signal-to-noise ratio
      for this approach was too low to be of value).
3. `.proto` linting:
   1. Buf in CI on every pull request
4. `.proto` formatting:
   1. [clang-format] locally and a [clang-format GitHub Action] in CI to check
      that files are formatted properly on every pull request.
5. Sharing of `.proto` files in a versioned, reliable manner:
   1. Consumers could simply clone the Tendermint repository, check out a
      specific commit, tag or branch and manually copy out all of the `.proto`
      files they need. This requires no effort from the Tendermint Core team and
      will continue to be an option for consumers. The drawback of this approach
      is that it requires manual coding/scripting to implement and is brittle in
      the face of bigger changes.
   2. Uploading our `.proto` files to Buf's registry on every release. This is
      by far the most seamless for consumers of our `.proto` files, but requires
      the dependency on Buf. This has the additional benefit that the Buf
      registry will automatically [generate and host
      documentation][buf-docs-gen] for these protos.
   3. We could create a process that, upon release, creates a `.zip` file
      containing our `.proto` files.

### Popular alternatives to Buf

[Prototool] was not considered as it appears deprecated, and the ecosystem seems
to be converging on Buf at this time.

### Tooling complexity

The more tools we have in our build/CI processes, the more complex and fragile
repository/CI management becomes, and the longer it takes to onboard new team
members. Maintainability is a core concern here.

### Buf sustainability and costs

One of the primary considerations regarding the usage of Buf is whether, for
example, access to its registry will eventually become a
paid-for/subscription-based service and whether this is valuable enough for us
and the ecosystem to pay for such a service. At this time, it appears as though
Buf will never charge for hosting open source projects' protos.

Another consideration was Buf's sustainability as a project - what happens when
their resources run out? Will there be a strong and broad enough open source
community to continue maintaining it?

### Local Buf usage options

Local usage of Buf (i.e. not in CI) can be accomplished in two ways:

1. Installing the relevant tools individually.
2. By way of its [Docker image][buf-docker].

Local installation of Buf requires developers to manually keep their toolchains
up-to-date. The Docker option comes with a number of complexities, including
how the file system permissions of code generated by a Docker container differ
between platforms (e.g. on Linux, Buf-generated code ends up being owned by
`root`).

The trouble with the Docker-based approach is that we make use of the
[gogoprotobuf] plugin for `protoc`. Continuing to use the Docker-based approach
to using Buf will mean that we will have to continue building our own custom
Docker image with embedded gogoprotobuf.

Along these lines, we could eventually consider coming up with a [Nix]- or
[redo]-based approach to developer tooling to ensure tooling consistency across
the team and for anyone who wants to be able to contribute to Tendermint.

## Decision

1. We will adopt Buf for now for proto generation, linting, breakage checking
   and its registry (mainly in CI, with optional usage locally).
2. Failing CI when checking for breaking changes in `.proto` files will only
   happen when performing minor/patch releases.
3. Local tooling will be favored over Docker-based tooling.

## Detailed Design

We currently aim to:

1. Update to Buf `v1` to facilitate linting, breakage checking and uploading to
   the Buf registry.
2. Configure CI appropriately for proto management:
   1. Uploading protos to the Buf registry on every release (e.g. the
      [approach][cosmos-sdk-buf-registry-ci] used by the Cosmos SDK).
   2. Linting on every pull request (e.g. the
      [approach][cosmos-sdk-buf-linting-ci] used by the Cosmos SDK). The linter
      passing should be considered a requirement for accepting PRs.
   3. Checking for breaking changes in minor/patch version releases and failing
      CI accordingly - see [\#8003].
   4. Add [clang-format GitHub Action] to check `.proto` file formatting. Format
      checking should be considered a requirement for accepting PRs.
3. Update the Tendermint [`Makefile`](../../Makefile) to primarily facilitate
   local Protobuf stub generation, linting, formatting and breaking change
   checking. More specifically:
   1. This includes removing the dependency on Docker and introducing the
      dependency on local toolchain installation. CI-based equivalents, where
      relevant, will rely on specific GitHub Actions instead of the Makefile.
   2. Go code generation will rely on `protoc` directly.

## Consequences

### Positive

- We will still offer Go stub generation, proto linting and breakage checking.
- Breakage checking will only happen on minor/patch releases to increase the
  signal-to-noise ratio in CI.
- Versioned protos will be made available via Buf's registry upon every release.

### Negative

- Developers/contributors will need to install the relevant Protocol
  Buffers-related tooling (Buf, gogoprotobuf, clang-format) locally in order to
  build, lint, format and check `.proto` files for breaking changes.

### Neutral

## References

- [Protocol Buffers]
- [Buf]
- [\#7975]
- [protoc] - The Protocol Buffers compiler

[Protocol Buffers]: https://developers.google.com/protocol-buffers
[Buf]: https://buf.build/
[\#7975]: https://github.com/tendermint/tendermint/pull/7975
[protoc]: https://github.com/protocolbuffers/protobuf
[clang-format]: https://clang.llvm.org/docs/ClangFormat.html
[clang-format GitHub Action]: https://github.com/marketplace/actions/clang-format-github-action
[buf-docker]: https://hub.docker.com/r/bufbuild/buf
[cosmos-sdk-buf-registry-ci]: https://github.com/cosmos/cosmos-sdk/blob/e6571906043b6751951a42b6546431b1c38b05bd/.github/workflows/proto-registry.yml
[cosmos-sdk-buf-linting-ci]: https://github.com/cosmos/cosmos-sdk/blob/e6571906043b6751951a42b6546431b1c38b05bd/.github/workflows/proto.yml#L15
[\#8003]: https://github.com/tendermint/tendermint/issues/8003
[Nix]: https://nixos.org/
[gogoprotobuf]: https://github.com/cosmos/gogoproto
[Prototool]: https://github.com/uber/prototool
[buf-docs-gen]: https://docs.buf.build/bsr/documentation
[redo]: https://redo.readthedocs.io/en/latest/
