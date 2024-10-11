// Package state_native defines how the beacon chain state for Ethereum
// functions in the running beacon node, using an advanced,
// immutable implementation of the state data structure.
//
// BeaconState getters may be accessed from inside or outside the package. To
// avoid duplicating locks, we have internal and external versions of the
// getter. The external function obtains a read lock, then calls the internal function.
// The internal function returns the required data without further locking,
// allowing it to be used by other package-level functions that already hold a lock.
// Hence the functions look something like this:
//
//	func (b *BeaconState) Foo() uint64 {
//	  // Read lock.
//	  b.lock.RLock()
//	  defer b.lock.RUnlock()
//
//	  // Internal getter.
//	  return b.foo()
//	}
//
//	func (b *BeaconState) foo() uint64 {
//	  (...) // Some processing logic.
//
//	  return b.foo
//	}
//
// Although it is technically possible to remove the short-circuit conditions
// from the external function, that would require every read to obtain a lock
// even if the data was not present, leading to potential slowdowns.
package state_native
