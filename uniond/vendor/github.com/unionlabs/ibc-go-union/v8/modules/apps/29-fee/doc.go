/*
Package fee implements the packet data structure, state machine handling logic,
and encoding details for handling fee payments on top of any ICS application protocol.
This implementation is based off the ICS 29 specification
(https://github.com/cosmos/ibc/tree/main/spec/app/ics-029-fee-payment) and follows
the middleware pattern specified in the ICS 30 specification
(https://github.com/cosmos/ibc/tree/main/spec/app/ics-030-middleware).
*/
package fee
