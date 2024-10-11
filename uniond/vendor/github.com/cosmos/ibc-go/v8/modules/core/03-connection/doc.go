/*
Package connection implements the ICS 03 - Connection Semantics specification
(https://github.com/cosmos/ibc/tree/main/spec/core/ics-003-connection-semantics). This
concrete implementation defines types and methods for safely creating two
stateful objects (connection ends) on two separate chains, each associated with a
light client of the other chain, which together facilitate cross-chain
sub-state verification and packet association (through channels).

The main type is ConnectionEnd, which defines a stateful object on a
chain connected to another.
*/
package connection
