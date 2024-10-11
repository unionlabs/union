# RFC 021: The Future of the Socket Protocol

## Changelog

- 19-May-2022: Initial draft (@creachadair)
- 19-Jul-2022: Converted from ADR to RFC (@creachadair)

## Abstract

This RFC captures some technical discussion about the ABCI socket protocol that
was originally documented to solicit an architectural decision.  This topic was
not high-enough priority as of this writing to justify making a final decision.

For that reason, the text of this RFC has the general structure of an ADR, but
should be viewed primarily as a record of the issue for future reference.

## Background

The [Application Blockchain Interface (ABCI)][abci] is a client-server protocol
used by the Tendermint consensus engine to communicate with the application on
whose behalf it performs state replication. There are currently three transport
options available for ABCI applications:

1. **In-process**: Applications written in Go can be linked directly into the
   same binary as the consensus node. Such applications use a "local" ABCI
   connection, which exposes application methods to the node as direct function
   calls.

2. **Socket protocol**: Out-of-process applications may export the ABCI service
   via a custom socket protocol that sends requests and responses over a
   Unix-domain or TCP socket connection as length-prefixed protocol buffers.
   In Tendermint, this is handled by the [socket client][socket-client].

3. **gRPC**: Out-of-process applications may export the ABCI service via gRPC.
   In Tendermint, this is handled by the [gRPC client][grpc-client].

Both the out-of-process options (2) and (3) have a long history in Tendermint.
The beginnings of the gRPC client were added in [May 2016][abci-start] when
ABCI was still hosted in a separate repository, and the socket client (formerly
called the "remote client") was part of ABCI from its inception in November
2015.

At that time when ABCI was first being developed, the gRPC project was very new
(it launched Q4 2015) and it was not an obvious choice for use in Tendermint.
It took a while before the language coverage and quality of gRPC reached a
point where it could be a viable solution for out-of-process applications.  For
that reason, it made sense for the initial design of ABCI to focus on a custom
protocol for out-of-process applications.

## Problem Statement

For practical reasons, ABCI needs an interprocess communication option to
support applications not written in Go. The two practical options are RPC and
FFI, and for operational reasons an RPC mechanism makes more sense.

The socket protocol has not changed all that substantially since its original
design, and has the advantage of being simple to implement in almost any
reasonable language.  However, its simplicity includes some limitations that
have had a negative impact on the stability and performance of out-of-process
applications using it. In particular:

- The protocol lacks request identifiers, so the client and server must return
  responses in strict FIFO order. Even if the client issues requests that have
  no dependency on each other, the protocol has no way except order of issue to
  map responses to requests.

  This reduces (in some cases substantially) the concurrency an application can
  exploit, since the parallelism of requests in flight is gated by the slowest
  active request at any moment.  There have been complaints from some network
  operators on that basis.

- The protocol lacks method identifiers, so the only way for the client and
  server to understand which operation is requested is to dispatch on the type
  of the request and response payloads. For responses, this means that [any
  error condition is terminal not only to the request, but to the entire ABCI
  client](https://github.com/tendermint/tendermint/blob/main/abci/client/socket_client.go#L149).

  The historical intent of terminating for any error seems to have been that
  all ABCI errors are unrecoverable and hence protocol fatal <!-- markdown-link-check-disable-next-line -->
  (see [Note 1](#note1)).  In practice, however, this greatly complicates
  debugging a faulty node, since the only way to respond to errors is to panic
  the node which loses valuable context that could have been logged.

- There are subtle concurrency management dependencies between the client and
  the server that are not clearly documented anywhere, and it is very easy for
  small changes in both the client and the server to lead to tricky deadlocks,
  panics, race conditions, and slowdowns. As a recent example of this, see
  <https://github.com/tendermint/tendermint/pull/8581>.

These limitations are fixable, but one important question is whether it is
worthwhile to fix them.  We can add request and method identifiers, for
example, but doing so would be a breaking change to the protocol requiring
every application using it to update.  If applications have to migrate anyway,
the stability and language coverage of gRPC have improved a lot, and today it
is probably simpler to set up and maintain an application using gRPC transport
than to reimplement the Tendermint socket protocol.

Moreover, gRPC addresses all the above issues out-of-the-box, and requires
(much) less custom code for both the server (i.e., the application) and the
client. The project is well-funded and widely-used, which makes it a safe bet
for a dependency.

## Decision

There is a set of related alternatives to consider:

- Question 1: Designate a single IPC standard for out-of-process applications?

  Claim: We should converge on one (and only one) IPC option for out-of-process
  applications. We should choose an option that, after a suitable period of
  deprecation for alternatives, will address most or all the highest-impact
  uses of Tendermint.  Maintaining multiple options increases the surface area
  for bugs and vulnerabilities, and we should not have multiple options for
  basic interfaces without a clear and well-documented reason.

- Question 2a: Choose gRPC and deprecate/remove the socket protocol?

  Claim: Maintaining and improving a custom RPC protocol is a substantial
  project and not directly relevant to the requirements of consensus. We would
  be better served by depending on a well-maintained open-source library like
  gRPC.

- Question 2b: Improve the socket protocol and deprecate/remove gRPC?

  Claim: If we find meaningful advantages to maintaining our own custom RPC
  protocol in Tendermint, we should treat it as a first-class project within
  the core and invest in making it good enough that we do not require other
  options.

**One important consideration** when discussing these questions is that _any
outcome which includes keeping the socket protocol will have eventual migration
impacts for out-of-process applications_ regardless. To fix the limitations of
the socket protocol as it is currently designed will require making _breaking
changes_ to the protocol.  So, while we may put off a migration cost for
out-of-process applications by retaining the socket protocol in the short term,
we will eventually have to pay those costs to fix the problems in its current
design.

## Detailed Design

1. If we choose to standardize on gRPC, the main work in Tendermint core will
   be removing and cleaning up the code for the socket client and server.

   Besides the code cleanup, we will also need to clearly document a
   deprecation schedule, and invest time in making the migration easier for
   applications currently using the socket protocol.

   > **Point for discussion:** Migrating from the socket protocol to gRPC
   > should mostly be a plumbing change, as long as we do it during a release
   > in which we are not making other breaking changes to ABCI. However, the
   > effort may be more or less depending on how gRPC integration works in the
   > application's implementation language, and would have to be sure networks
   > have plenty of time not only to make the change but to verify that it
   > preserves the function of the network.
   >
   > What questions should we be asking node operators and application
   > developers to understand the migration costs better?

2. If we choose to keep only the socket protocol, we will need to follow up
   with a more detailed design for extending and upgrading the protocol to fix
   the existing performance and operational issues with the protocol.

   Moreover, since the gRPC interface has been around for a long time we will
   also need a deprecation plan for it.

3. If we choose to keep both options, we will still need to do all the work of
   (2), but the gRPC implementation should not require any immediate changes.


## Alternatives Considered

- **FFI**. Another approach we could take is to use a C-based FFI interface so
  that applications written in other languages are linked directly with the
  consensus node, an option currently only available for Go applications.

  An FFI interface is possible for a lot of languages, but FFI support varies
  widely in coverage and quality across languages and the points of friction
  can be tricky to work around.  Moreover, it's much harder to add FFI support
  to a language where it's missing after-the-fact for an application developer.

  Although a basic FFI interface is not too difficult on the Go side, the C
  shims for an FFI can get complicated if there's a lot of variability in the
  runtime environment on the other end.

  If we want to have one answer for non-Go applications, we are better off
  picking an IPC-based solution (whether that's gRPC or an extension of our
  custom socket protocol or something else).

## Consequences

- **Standardize on gRPC**

    - ✅ Addresses existing performance and operational issues.
    - ✅ Replaces custom code with a well-maintained widely-used library.
    - ✅ Aligns with Cosmos SDK, which already uses gRPC extensively.
    - ✅ Aligns with priv validator interface, for which the socket protocol is already deprecated for gRPC.
    - ❓ Applications will be hard to implement in a language without gRPC support.
    - ⛔ All users of the socket protocol have to migrate to gRPC, and we believe most current out-of-process applications use the socket protocol.

- **Standardize on socket protocol**

    - ✅ Less immediate impact for existing users (but see below).
    - ✅ Simplifies ABCI API surface by removing gRPC.
    - ❓ Users of the socket protocol will have a (smaller) migration.
    - ❓ Potentially easier to implement for languages that do not have support.
    - ⛔ Need to do all the work to fix the socket protocol (which will require existing users to update anyway later).
    - ⛔ Ongoing maintenance burden for per-language server implementations.

- **Keep both options**

    - ✅ Less immediate impact for existing users (but see below).
    - ❓ Users of the socket protocol will have a (smaller) migration.
    - ⛔ Still need to do all the work to fix the socket protocol (which will require existing users to update anyway later).
    - ⛔ Requires ongoing maintenance and support of both gRPC and socket protocol integrations.


## References

- [Application Blockchain Interface (ABCI)][abci]
- [Tendermint ABCI socket client][socket-client]
- [Tendermint ABCI gRPC client][grpc-client]
- [Initial commit of gRPC client][abci-start]

[abci]: https://github.com/tendermint/tendermint/tree/main/spec/abci
[socket-client]: https://github.com/tendermint/tendermint/blob/main/abci/client/socket_client.go
[socket-server]: https://github.com/tendermint/tendermint/blob/main/abci/server/socket_server.go
[grpc-client]: https://github.com/tendermint/tendermint/blob/main/abci/client/grpc_client.go
[abci-start]: https://github.com/tendermint/abci/commit/1ab3c747182aaa38418258679c667090c2bb1e0d

## Notes

- <a id=note1></a>**Note 1**: The choice to make all ABCI errors protocol-fatal
  was intended to avoid the risk that recovering an application error could
  cause application state to diverge.  Divergence can break consensus, so it's
  essential to avoid it.

  This is a sound principle, but conflates protocol errors with "mechanical"
  errors such as timeouts, resources exhaustion, failed connections, and so on.
  Because the protocol has no way to distinguish these conditions, the only way
  for an application to report an error is to panic or crash.

  Whether a node is running in the same process as the application or as a
  separate process, application errors should not be suppressed or hidden.
  However, it's important to ensure that errors are handled at a consistent and
  well-defined point in the protocol: Having the application panic or crash
  rather than reporting an error means the node sees different results
  depending on whether the application runs in-process or out-of-process, even
  if the application logic is otherwise identical.

## Appendix: Known Implementations of ABCI Socket Protocol

This is a list of known implementations of the Tendermint custom socket
protocol. Note that in most cases I have not checked how complete or correct
these implementations are; these are based on search results and a cursory
visual inspection.

- Tendermint Core (Go): [client][socket-client], [server][socket-server]
- Informal Systems [tendermint-rs](https://github.com/informalsystems/tendermint-rs) (Rust): [client](https://github.com/informalsystems/tendermint-rs/blob/master/abci/src/client.rs), [server](https://github.com/informalsystems/tendermint-rs/blob/master/abci/src/server.rs)
- Tendermint [js-abci](https://github.com/tendermint/js-abci) (JS): [server](https://github.com/tendermint/js-abci/blob/master/src/server.js)
- [Hotmoka](https://github.com/Hotmoka/hotmoka) ABCI (Java): [server](https://github.com/Hotmoka/hotmoka/blob/master/io-hotmoka-tendermint-abci/src/main/java/io/hotmoka/tendermint_abci/Server.java)
- [Tower ABCI](https://github.com/penumbra-zone/tower-abci) (Rust): [server](https://github.com/penumbra-zone/tower-abci/blob/main/src/server.rs)
- [abci-host](https://github.com/datopia/abci-host) (Clojure): [server](https://github.com/datopia/abci-host/blob/master/src/abci/host.clj)
- [abci_server](https://github.com/KrzysiekJ/abci_server) (Erlang): [server](https://github.com/KrzysiekJ/abci_server/blob/master/src/abci_server.erl)
- [py-abci](https://github.com/davebryson/py-abci) (Python): [server](https://github.com/davebryson/py-abci/blob/master/src/abci/server.py)
- [scala-tendermint-server](https://github.com/intechsa/scala-tendermint-server) (Scala): [server](https://github.com/InTechSA/scala-tendermint-server/blob/master/src/main/scala/lu/intech/tendermint/Server.scala)
- [kepler](https://github.com/f-o-a-m/kepler) (Rust): [server](https://github.com/f-o-a-m/kepler/blob/master/hs-abci-server/src/Network/ABCI/Server.hs)
