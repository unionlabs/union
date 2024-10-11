*July 1, 2024*

This is a major release of CometBFT that includes several substantial changes
that aim to reduce bandwidth consumption, enable modularity, improve
integrators' experience and increase the velocity of the CometBFT development
team, including:

1. Validators now proactively communicate the block parts they already have so
   others do not resend them, reducing amplification in the network and reducing
   bandwidth consumption.
2. An experimental feature in the mempool that allows limiting the number of
   peers to which transactions are forwarded, allowing operators to optimize
   gossip-related bandwidth consumption further.
3. An opt-in `nop` mempool, which allows application developers to turn off all
   mempool-related functionality in Comet such that they can build their own
   transaction dissemination mechanism, for example a standalone mempool-like
   process that can be scaled independently of the consensus engine/application.
   This requires application developers to implement their own gossip/networking
   mechanisms. See [ADR 111](./docs/architecture/adr-111-nop-mempool.md) for
   details.
4. The first officially supported release of the [data companion
   API](./docs/architecture/adr-101-data-companion-pull-api.md).
5. Versioning of both the Protobuf definitions _and_ RPC. By versioning our
   APIs, we aim to provide a level of commitment to API stability while
   simultaneously affording ourselves the ability to roll out substantial
   changes in non-breaking releases of CometBFT. See [ADR 103](./docs/architecture/adr-103-proto-versioning.md) and [ADR 107](./docs/architecture/adr-107-betaize-proto-versions.md).
6. Moving many Go packages that are currently publicly accessible into the
   `internal` directory such that the team can roll out substantial changes in
   future without needing to worry about causing breakages in users' codebases.
   The massive surface area of previous versions has in the past significantly
   hampered the team's ability to roll out impactful new changes to users, as
   previously such changes required a new breaking release (which currently
   takes 6 to 12 months to reach production use for many users). See [ADR 109](./docs/architecture/adr-109-reduce-go-api-surface.md) for more details.
7. Proposer-Based Timestamps (PBTS) support. PBTS is a Byzantine fault-tolerant
    algorithm used by CometBFT for computing block times.
    When activated on a chain, it replaces the pre-existing BFT-time algorithm.
    See [spec](./spec/consensus/proposer-based-timestamp) doc for PBTS.

None of these changes are state machine-breaking for CometBFT-based networks,
but could be breaking for some users who depend on the Protobuf definitions type
URLs. See the [upgrading guidelines](./UPGRADING.md) and specific changes below
for more details.

**NB: This version is still a release candidate, which means that
API-breaking changes, although very unlikely, might still be introduced
before the final release.** See [RELEASES.md](./RELEASES.md) for more information on
the stability guarantees we provide for pre-releases.
