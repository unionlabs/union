// Package multicommit implements commitment expansion.
//
// If the builder implements [frontend.Committer] interface, then we can commit
// to the variables and get a commitment which can be used as a unique
// randomness in the circuit. For current builders implementing this interface,
// the function can only be called once in a circuit. This makes it difficult to
// compose different gadgets which require randomness.
//
// This package extends the commitment interface by allowing to receive several
// functions unique commitment multiple times. It does this by collecting all
// variables to commit and the callbacks which want to access a commitment. Then
// we internally defer a function which computes the commitment over all input
// committed variables and then uses this commitment to derive a per-callback
// unique commitment. The callbacks are then called with these unique derived
// commitments instead.
package multicommit

import (
	"fmt"

	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/internal/kvstore"
	"github.com/consensys/gnark/std/hash/mimc"
)

type multicommiter struct {
	closed bool
	vars   []frontend.Variable
	cbs    []WithCommitmentFn
}

type ctxMulticommiterKey struct{}

// getCached gets the cached committer from the key-value storage. If it is not
// there then creates, stores and defers it, and then returns.
func getCached(api frontend.API) *multicommiter {
	kv, ok := api.(kvstore.Store)
	if !ok {
		// if the builder doesn't implement key-value store then cannot store
		// multi-committer in cache.
		panic("builder should implement key-value store")
	}
	mc := kv.GetKeyValue(ctxMulticommiterKey{})
	if mc != nil {
		if mct, ok := mc.(*multicommiter); ok {
			return mct
		} else {
			panic("stored multicommiter is of invalid type")
		}
	}
	mct := &multicommiter{}
	kv.SetKeyValue(ctxMulticommiterKey{}, mct)
	api.Compiler().Defer(mct.commitAndCall)
	return mct
}

func (mct *multicommiter) commitAndCall(api frontend.API) error {
	// close collecting input in case anyone wants to check more variables to commit to.
	mct.closed = true
	if len(mct.cbs) == 0 {
		// shouldn't happen. we defer this function on creating multicommitter
		// instance. It is probably some race.
		panic("calling commiter with zero callbacks")
	}
	commiter, ok := api.Compiler().(frontend.Committer)
	if !ok {
		panic("compiler doesn't implement frontend.Committer")
	}
	cmt, err := commiter.Commit(mct.vars...)
	if err != nil {
		return fmt.Errorf("commit: %w", err)
	}
	if len(mct.cbs) == 1 {
		if err = mct.cbs[0](api, cmt); err != nil {
			return fmt.Errorf("single callback: %w", err)
		}
	} else {
		hasher, err := mimc.NewMiMC(api)
		if err != nil {
			return fmt.Errorf("new hasher: %w", err)
		}
		for i, cb := range mct.cbs {
			hasher.Reset()
			hasher.Write(i+1, cmt)
			localcmt := hasher.Sum()
			if err = cb(api, localcmt); err != nil {
				return fmt.Errorf("with commitment callback %d: %w", i, err)
			}
		}
	}
	return nil
}

// WithCommitmentFn is the function which is called asynchronously after all
// variables have been committed to. See [WithCommitment] for scheduling a
// function of this type. Every called functions received a distinct commitment
// built from a single root.
//
// It is invalid to call [WithCommitment] in this method recursively and this
// leads to panic. However, the method can call defer for other callbacks.
type WithCommitmentFn func(api frontend.API, commitment frontend.Variable) error

// WithCommitment schedules the function cb to be called with a unique
// commitment. We append the variables committedVariables to be committed to
// with the native [frontend.Committer] interface.
func WithCommitment(api frontend.API, cb WithCommitmentFn, committedVariables ...frontend.Variable) {
	mct := getCached(api)
	if mct.closed {
		panic("called WithCommitment recursively")
	}
	mct.vars = append(mct.vars, committedVariables...)
	mct.cbs = append(mct.cbs, cb)
}
