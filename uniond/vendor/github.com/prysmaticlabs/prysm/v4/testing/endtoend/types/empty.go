package types

import (
	"context"
	"sync"
)

// EmptyComponent satisfies the component interface.
// It can be embedded in other types in order to turn them into components.
type EmptyComponent struct {
	sync.Mutex
	startc chan struct{}
}

func (c *EmptyComponent) Start(context.Context) error {
	c.chanInit()
	close(c.startc)
	return nil
}

func (c *EmptyComponent) chanInit() {
	c.Lock()
	defer c.Unlock()
	if c.startc == nil {
		c.startc = make(chan struct{})
	}
}

func (c *EmptyComponent) Started() <-chan struct{} {
	c.chanInit()
	return c.startc
}

func (*EmptyComponent) Pause() error {
	return nil
}

func (*EmptyComponent) Resume() error {
	return nil
}

func (*EmptyComponent) Stop() error {
	return nil
}
