// Package circuitbreaker implements the circuit breaker pattern.
//
// Circuit breakers prevent thundering herds, and improve resiliency against
// intermittent errors. Every client-side endpoint should be wrapped in a
// circuit breaker.
//
// We provide several implementations in this package, but if you're looking
// for guidance, Gobreaker is probably the best place to start.  It has a
// simple and intuitive API, and is well-tested.
package circuitbreaker
