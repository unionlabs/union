package errors

import (
	"fmt"
	"os"
	"sync"

	"github.com/coinbase/rosetta-sdk-go/types"
)

type errorRegistry struct {
	mu     *sync.RWMutex
	sealed bool
	errors map[int32]*types.Error
}

func (r *errorRegistry) add(err *Error) {
	r.mu.Lock()
	defer r.mu.Unlock()
	if r.sealed {
		_, _ = fmt.Fprintln(os.Stderr, "[ROSETTA] WARNING: attempts to register errors after seal will be ignored")
	}
	if _, ok := r.errors[err.rosErr.Code]; ok {
		_, _ = fmt.Fprintln(os.Stderr, "[ROSETTA] WARNING: attempts to register an already registered error will be ignored, code: ", err.rosErr.Code)
	}
	r.errors[err.rosErr.Code] = err.rosErr
}

func (r errorRegistry) list() []*types.Error {
	r.mu.RLock()
	defer r.mu.RUnlock()
	rosErrs := make([]*types.Error, 0, len(registry.errors))
	for _, v := range r.errors {
		rosErrs = append(rosErrs, v)
	}
	return rosErrs
}

func (r *errorRegistry) seal() {
	r.mu.Lock()
	defer r.mu.Unlock()
	r.sealed = true
}

var registry = errorRegistry{
	mu:     new(sync.RWMutex),
	errors: make(map[int32]*types.Error),
}
