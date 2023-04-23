/*
Package client implements the ICS 02 - Client Semantics specification
https://github.com/cosmos/ibc/tree/master/spec/core/ics-002-client-semantics. This
concrete implementations defines types and method to store and update light
clients which tracks on other chain's state.

The main type is `Client`, which provides `commitment.Root` to verify state proofs and `ConsensusState` to
verify header proofs.
*/
package client
