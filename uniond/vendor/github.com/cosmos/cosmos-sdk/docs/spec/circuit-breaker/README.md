# Concepts

The intention of the circuit breaker is to have a contingency plan for a
running network which maintains network liveness. This can be achieved through
selectively "pausing" functionality of specific modules on a running network.
The circuit breaker is intended to be enabled through either:

* governance
* for emergencies a special subset of accounts selected by the state machine
* a transaction which proves the expected behaviour is broken

## Pause state

The basic pause state of any module simply disables all message routes to
that module. Beyond that, it may be a appropriate for different modules to
process begin-block/end-block in an altered "safe" way.
