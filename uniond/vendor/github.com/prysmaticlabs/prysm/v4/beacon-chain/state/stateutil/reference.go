package stateutil

import "sync"

// Reference structs are shared across BeaconState copies to understand when the state must use
// copy-on-write for shared fields or may modify a field in place when it holds the only reference
// to the field value. References are tracked in a map of fieldIndex -> *reference. Whenever a state
// releases their reference to the field value, they must decrement the refs. Likewise whenever a
// copy is performed then the state must increment the refs counter.
type Reference struct {
	refs uint
	lock sync.RWMutex
}

// NewRef initializes the Reference struct.
func NewRef(refs uint) *Reference {
	return &Reference{
		refs: refs,
	}
}

// Refs returns the reference number.
func (r *Reference) Refs() uint {
	r.lock.RLock()
	defer r.lock.RUnlock()
	return r.refs
}

// AddRef adds 1 to the reference number.
func (r *Reference) AddRef() {
	r.lock.Lock()
	r.refs++
	r.lock.Unlock()
}

// MinusRef subtracts 1 to the reference number.
func (r *Reference) MinusRef() {
	r.lock.Lock()
	// Do not reduce further if object
	// already has 0 reference to prevent underflow.
	if r.refs > 0 {
		r.refs--
	}
	r.lock.Unlock()
}
