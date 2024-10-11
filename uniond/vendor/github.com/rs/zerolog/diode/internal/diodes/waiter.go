package diodes

import (
	"context"
	"sync"
)

// Waiter will use a conditional mutex to alert the reader to when data is
// available.
type Waiter struct {
	Diode
	mu  sync.Mutex
	c   *sync.Cond
	ctx context.Context
}

// WaiterConfigOption can be used to setup the waiter.
type WaiterConfigOption func(*Waiter)

// WithWaiterContext sets the context to cancel any retrieval (Next()). It
// will not change any results for adding data (Set()). Default is
// context.Background().
func WithWaiterContext(ctx context.Context) WaiterConfigOption {
	return func(c *Waiter) {
		c.ctx = ctx
	}
}

// NewWaiter returns a new Waiter that wraps the given diode.
func NewWaiter(d Diode, opts ...WaiterConfigOption) *Waiter {
	w := new(Waiter)
	w.Diode = d
	w.c = sync.NewCond(&w.mu)
	w.ctx = context.Background()

	for _, opt := range opts {
		opt(w)
	}

	go func() {
		<-w.ctx.Done()

		// Mutex is strictly necessary here to avoid a race in Next() (between
		// w.isDone() and w.c.Wait()) and w.c.Broadcast() here.
		w.mu.Lock()
		w.c.Broadcast()
		w.mu.Unlock()
	}()

	return w
}

// Set invokes the wrapped diode's Set with the given data and uses Broadcast
// to wake up any readers.
func (w *Waiter) Set(data GenericDataType) {
	w.Diode.Set(data)
	w.c.Broadcast()
}

// Next returns the next data point on the wrapped diode. If there is not any
// new data, it will Wait for set to be called or the context to be done.
// If the context is done, then nil will be returned.
func (w *Waiter) Next() GenericDataType {
	w.mu.Lock()
	defer w.mu.Unlock()

	for {
		data, ok := w.Diode.TryNext()
		if !ok {
			if w.isDone() {
				return nil
			}

			w.c.Wait()
			continue
		}
		return data
	}
}

func (w *Waiter) isDone() bool {
	select {
	case <-w.ctx.Done():
		return true
	default:
		return false
	}
}
