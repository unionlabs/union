/*
Package channel implements the ICS 04 - Channel and Packet Semantics specification
(https://github.com/cosmos/ibc/tree/main/spec/core/ics-004-channel-and-packet-semantics). This
concrete implementation defines types and methods for safely creating two
stateful objects (channel ends) on two separate chains, each associated with a
particular connection. A channel serves as a conduit for packets passing between a
module on one chain and a module on another, ensuring that packets are executed
only once, delivered in the order in which they were sent (if necessary),
and delivered only to the corresponding module owning the other end of the
channel on the destination chain.

The main types are Channel, which defines a stateful object on a
chain that allows for exactly-once packet delivery between specific modules
on separate blockchains, and Packet, which defines the data carried
across different chains through IBC.
*/
package channel
