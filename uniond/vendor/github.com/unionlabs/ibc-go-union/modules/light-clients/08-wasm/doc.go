/*
Package wasm implements a concrete ClientState, ConsensusState,
ClientMessage and types for the proxy light client module communicating
with underlying Wasm light clients.
This implementation is based off the ICS 08 specification
(https://github.com/cosmos/ibc/blob/main/spec/client/ics-008-wasm-client)

By default the 08-wasm module requires cgo and libwasmvm dependencies available on the system.
However, users of this module may want to depend only on types, without incurring the dependency on cgo or libwasmvm.
In this case, it is possible to build the code with either cgo disabled or a custom build directive: nolink_libwasmvm.
This allows disabling linking of libwasmvm and not forcing users to have specific libraries available on their systems.

Please refer to the 08-wasm module documentation for more information.
*/
package wasm
