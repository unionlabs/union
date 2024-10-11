# RFC 106: Separation of non-idempotent methods in data companion API

## Changelog

- 2023-10-27: Initial revision (@mzabaluev)

## Abstract

[ADR 101] defined gRPC APIs to retrieve information on blocks and
block execution results for a data companion. As a special case, the caller
can specify the height parameter of 0 to retrieve data on the latest block
known to the node. In the process of implementation however, the developers
thought it necessary to also define special methods for this case. To reduce
potential for mistaken use and separate the time-dependent processing of the
"get latest" requests, we propose to eliminate the special case in the
"get by height" methods, making them idempotent. The use case for "get latest"
also needs to be scrutinized here.

[ADR 101]: https://github.com/cometbft/cometbft/blob/main/docs/architecture/adr-101-data-companion-pull-api.md

## Background

In process of implementing ADR 101, convenience methods were added to retrieve
data on the latest block:
`GetLatest` to `BlockService` ([#1209]) and
`GetLatestBlockResults` to `BlockResultsService` ([#1168])

[#1209]: https://github.com/cometbft/cometbft/pull/1209
[#1168]: https://github.com/cometbft/cometbft/pull/1168

The special treatment of the height value of 0 in `GetByHeight` and
`GetBlockResults` has been seemingly forgotten.

### References

* [Discussion](https://github.com/cometbft/cometbft/pull/1533#discussion_r1370861999)
  on complications arising from the additional methods in
  proto cleanup work for [#1530](https://github.com/cometbft/cometbft/issues/1530).

## Discussion

In this case, the changes driven by practicalities of implementation actually
highlight the different use cases and semantics for the respective methods.
`GetByHeight` (with a valid height) returns the same data for the same input
and so the responses can be cached at the client side, while the response
of `GetLatest` varies with time. It's also easy to erroneously use 0 as the
actual requested height in workflows using `GetByHeight` and process the result
to some ill effects down the road, rather than be stopped by a timely failure.

For these reasons, it seems better to cleanly separate the two use cases and
remove the documentation language about the height parameter of 0 as the special
case to retrieve the latest value. Furthermore, the usefulness of the
non-idempotent "get latest block data" API for a data companion
(other than the `GetLatestHeight` stream subscription used to follow the
progress of the chain) has never been discussed or demonstrated.
If this was only a "nice to throw in" idea, perhaps it's better to remove
these methods until the need for them becomes apparent.

## Proposed actions

* Remove the following gRPC methods:
  - `GetLatest` from `BlockService`;
  - `GetLatestBlockResults` from `BlockResultsService`.
* Change the documentation for the `GetByHeight` and `GetBlockResults` methods
  to not treat the height parameter of 0 as a special case.
* Revise the specifications in ADR-101 to remove the special treatment of
  the height value 0.
