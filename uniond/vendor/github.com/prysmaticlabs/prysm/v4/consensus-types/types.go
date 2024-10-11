package consensus_types

import (
	"errors"
	"fmt"
	"sync/atomic"

	errors2 "github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
)

var (
	// ErrNilObjectWrapped is returned in a constructor when the underlying object is nil.
	ErrNilObjectWrapped = errors.New("attempted to wrap nil object")
	// ErrUnsupportedField is returned when a getter/setter access is not supported.
	ErrUnsupportedField = errors.New("unsupported getter")
	// ErrOutOfBounds is returned when a slice or array index does not exist.
	ErrOutOfBounds = errors.New("index out of bounds")
)

// ErrNotSupported constructs a message informing about an unsupported field access.
func ErrNotSupported(funcName string, ver int) error {
	return errors2.Wrap(ErrUnsupportedField, fmt.Sprintf("%s is not supported for %s", funcName, version.String(ver)))
}

// ThreadSafeEnumerator is a thread-safe counter of all objects created since the node's start.
type ThreadSafeEnumerator struct {
	counter uint64
}

// Inc increments the enumerator and returns the new object count.
func (c *ThreadSafeEnumerator) Inc() uint64 {
	return atomic.AddUint64(&c.counter, 1)
}
